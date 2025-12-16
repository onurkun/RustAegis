//! VM State management
//!
//! This module defines the VM execution state including:
//! - Dynamic registers (up to 256, R0-R255)
//! - Managed heap with free-list allocator
//! - Value stack and call stack
//! - CPU flags and execution control

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::error::{VmError, VmResult};
use crate::opcodes::flags;

// =============================================================================
// Free List Allocator Support
// =============================================================================

/// Size of allocation header (stores block size for free)
/// Layout: [size_with_flag: u64][user data...]
/// The MSB of size is used as "allocated" flag for double-free protection
const ALLOC_HEADER_SIZE: usize = 8;

/// Flag in header MSB indicating block is allocated (not free)
/// When set: block is in use; When clear: block is free
const ALLOCATED_FLAG: u64 = 0x8000_0000_0000_0000;

/// Mask to extract actual size from header (clear MSB)
const SIZE_MASK: u64 = !ALLOCATED_FLAG;

/// Represents a free block in the heap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeBlock {
    /// Start address of the free block (including header space)
    pub addr: usize,
    /// Total size of the free block (including header)
    pub size: usize,
}

// =============================================================================
// Constants
// =============================================================================

/// Maximum stack size (entries, not bytes)
pub const MAX_STACK_SIZE: usize = 1024;

/// Maximum instructions per execution (prevent infinite loops)
pub const MAX_INSTRUCTIONS: u64 = 1_000_000;

/// Maximum number of registers (R0-R255)
/// Limited by u8 index in opcodes
pub const MAX_REGISTERS: usize = 256;

/// Default number of pre-allocated registers
/// Optimized for common case (most functions use < 32 registers)
pub const DEFAULT_REGISTER_CAPACITY: usize = 32;

/// Default heap size (1 MB)
pub const DEFAULT_HEAP_SIZE: usize = 1024 * 1024;

/// Maximum heap size (10 MB) - DoS protection
pub const MAX_HEAP_SIZE: usize = 10 * 1024 * 1024;

/// Default heap capacity (start with 4 KB, grow as needed)
pub const DEFAULT_HEAP_CAPACITY: usize = 4 * 1024;

// =============================================================================
// Memory Address Layout (Unified Addressing)
// =============================================================================
//
// Address Range          | Region
// ---------------------- | ------
// 0x0000_0000 - heap_ptr | Heap (dynamic, grows upward)
// 0x8000_0000 - ...      | Input buffer (read-only)
// 0xC000_0000 - ...      | Output buffer (write)
//
// This layout allows heap to grow without conflicting with I/O regions.

/// Base address for input buffer in unified memory space
pub const INPUT_BASE_ADDR: u64 = 0x8000_0000;

/// Base address for output buffer in unified memory space
pub const OUTPUT_BASE_ADDR: u64 = 0xC000_0000;

// =============================================================================
// VM State
// =============================================================================

/// VM execution state
#[derive(Debug, Clone)]
pub struct VmState<'a> {
    // ========== Registers ==========
    /// General-purpose registers (R0-R255)
    /// Dynamically sized, grows on demand up to MAX_REGISTERS
    pub regs: Vec<u64>,

    // ========== Heap (Free-List Allocator) ==========
    /// Managed heap memory
    pub heap: Vec<u8>,
    /// Current heap allocation pointer (bump pointer for new allocations)
    pub heap_ptr: usize,
    /// Maximum heap size (DoS protection)
    pub heap_limit: usize,
    /// Free list for recycled memory blocks
    pub free_list: Vec<FreeBlock>,

    // ========== Stacks ==========
    /// Value stack
    pub stack: Vec<u64>,
    /// Call stack (return addresses)
    pub call_stack: Vec<usize>,

    // ========== Execution Control ==========
    /// Instruction pointer
    pub ip: usize,
    /// CPU flags (Zero, Carry, Overflow, Sign)
    pub flags: u8,
    /// Instruction counter (for max instruction limit)
    pub instruction_count: u64,
    /// Halted flag
    pub halted: bool,
    /// Result value (set by HALT)
    pub result: u64,
    /// Last error (if any)
    pub last_error: VmError,

    // ========== I/O Buffers ==========
    /// Bytecode being executed
    pub code: &'a [u8],
    /// Input data buffer (read-only)
    pub input: &'a [u8],
    /// Output data buffer
    pub output: Vec<u8>,

    // ========== Timing (Anti-Debug) ==========
    /// Last timing checkpoint (for anti-debug)
    pub last_timing_ns: u64,
    /// Execution start time (for timing checks)
    pub start_time_ns: u64,

    // ========== Native Function Table ==========
    /// Optional native function table for NATIVE_CALL opcode
    /// Used by vm_protect macro for compiled native calls
    #[allow(clippy::type_complexity)]
    pub native_table: Option<&'a [fn(&[u64]) -> u64]>,
}

impl<'a> VmState<'a> {
    /// Create new VM state with given bytecode and input
    pub fn new(code: &'a [u8], input: &'a [u8]) -> Self {
        Self {
            // Pre-allocate registers for common case
            regs: vec![0u64; DEFAULT_REGISTER_CAPACITY],
            // Heap with default capacity (grows on demand)
            heap: Vec::with_capacity(DEFAULT_HEAP_CAPACITY),
            heap_ptr: 0,
            heap_limit: DEFAULT_HEAP_SIZE,
            free_list: Vec::with_capacity(16), // Pre-allocate for common case
            // Stacks
            stack: Vec::with_capacity(64),
            call_stack: Vec::with_capacity(16),
            // Execution
            ip: 0,
            flags: 0,
            instruction_count: 0,
            halted: false,
            result: 0,
            last_error: VmError::Ok,
            // I/O
            code,
            input,
            output: Vec::new(),
            // Timing
            last_timing_ns: 0,
            start_time_ns: 0,
            // Native function table
            native_table: None,
        }
    }

    /// Create VM state with custom heap limit
    pub fn with_heap_limit(code: &'a [u8], input: &'a [u8], heap_limit: usize) -> Self {
        let mut state = Self::new(code, input);
        state.heap_limit = heap_limit.min(MAX_HEAP_SIZE);
        state
    }

    /// Create VM state with new code reference but preserving execution state
    /// Used by SMC engine to update code view after decryption
    pub fn with_code_and_state(code: &'a [u8], input: &'a [u8], old: &VmState<'a>) -> Self {
        Self {
            // Copy registers
            regs: old.regs.clone(),
            // Copy heap state
            heap: old.heap.clone(),
            heap_ptr: old.heap_ptr,
            heap_limit: old.heap_limit,
            free_list: old.free_list.clone(),
            // Copy stacks
            stack: old.stack.clone(),
            call_stack: old.call_stack.clone(),
            // Copy execution state
            ip: old.ip,
            flags: old.flags,
            instruction_count: old.instruction_count,
            halted: old.halted,
            result: old.result,
            last_error: old.last_error,
            // New code reference
            code,
            input,
            // Copy output
            output: old.output.clone(),
            // Copy timing
            last_timing_ns: old.last_timing_ns,
            start_time_ns: old.start_time_ns,
            // Copy native table
            native_table: old.native_table,
        }
    }

    /// Initialize timing for anti-debug checks
    #[inline]
    pub fn init_timing(&mut self) {
        #[cfg(all(feature = "std", not(feature = "vm_debug")))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            self.start_time_ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            self.last_timing_ns = self.start_time_ns;
        }
        #[cfg(any(not(feature = "std"), feature = "vm_debug"))]
        {
            self.start_time_ns = 0;
            self.last_timing_ns = 0;
        }
    }

    /// Get current time in nanoseconds
    #[inline]
    pub fn current_time_ns(&self) -> u64 {
        #[cfg(all(feature = "std", not(feature = "vm_debug")))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0)
        }
        #[cfg(any(not(feature = "std"), feature = "vm_debug"))]
        {
            0
        }
    }

    /// Reset state for re-execution
    pub fn reset(&mut self) {
        // Reset registers (keep capacity)
        self.regs.clear();
        self.regs.resize(DEFAULT_REGISTER_CAPACITY, 0);
        // Reset heap
        self.heap.clear();
        self.heap_ptr = 0;
        self.free_list.clear();
        // Reset stacks
        self.stack.clear();
        self.call_stack.clear();
        // Reset execution
        self.ip = 0;
        self.flags = 0;
        self.instruction_count = 0;
        self.halted = false;
        self.result = 0;
        self.last_error = VmError::Ok;
        // Reset output
        self.output.clear();
        // Reset timing
        self.last_timing_ns = 0;
        self.start_time_ns = 0;
        // Reset native table
        self.native_table = None;
    }

    /// Set native function table for NATIVE_CALL opcode
    /// Used by vm_protect macro for compiled native calls
    #[inline]
    pub fn set_native_table(&mut self, table: &'a [fn(&[u64]) -> u64]) {
        self.native_table = Some(table);
    }

    /// Get native function by index
    #[inline]
    pub fn get_native_fn(&self, index: usize) -> Option<fn(&[u64]) -> u64> {
        self.native_table.and_then(|t| t.get(index).copied())
    }

    // =========================================================================
    // Stack Operations
    // =========================================================================

    /// Push value to stack
    #[inline]
    pub fn push(&mut self, value: u64) -> VmResult<()> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    #[inline]
    pub fn pop(&mut self) -> VmResult<u64> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Peek at top of stack without popping
    #[inline]
    pub fn peek(&self) -> VmResult<u64> {
        self.stack.last().copied().ok_or(VmError::StackUnderflow)
    }

    /// Get stack length
    #[inline]
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    // =========================================================================
    // Register Operations (Dynamic, up to 256)
    // =========================================================================

    /// Get register value
    /// Returns 0 for uninitialized registers (lazy allocation)
    #[inline]
    pub fn get_reg(&self, idx: u8) -> VmResult<u64> {
        let index = idx as usize;
        if index < self.regs.len() {
            Ok(self.regs[index])
        } else {
            // Uninitialized register reads as 0
            Ok(0)
        }
    }

    /// Set register value
    /// Automatically grows register file if needed (up to MAX_REGISTERS)
    #[inline]
    pub fn set_reg(&mut self, idx: u8, value: u64) -> VmResult<()> {
        let index = idx as usize;

        // Grow register file if needed
        if index >= self.regs.len() {
            if index >= MAX_REGISTERS {
                return Err(VmError::InvalidRegister);
            }
            // Grow to accommodate new register (fill with 0)
            self.regs.resize(index + 1, 0);
        }

        self.regs[index] = value;
        Ok(())
    }

    /// Get number of currently allocated registers
    #[inline]
    pub fn reg_count(&self) -> usize {
        self.regs.len()
    }

    // =========================================================================
    // Heap Operations (Free-List Allocator)
    // =========================================================================

    /// Allocate memory on the heap
    /// Returns the start address of the allocated block (user data, after header)
    ///
    /// Layout: [header: u64 (size | ALLOCATED_FLAG)][user data...]
    /// Alignment is guaranteed to be 8-byte aligned
    #[inline]
    pub fn heap_alloc(&mut self, size: usize) -> VmResult<u64> {
        // Align user size to 8 bytes
        let aligned_user_size = (size + 7) & !7;
        // Total size includes header
        let total_size = ALLOC_HEADER_SIZE + aligned_user_size;

        // Strategy 1: Try to find a suitable block in free list (first-fit)
        if let Some(idx) = self.find_free_block(total_size) {
            let block = self.free_list.remove(idx);
            let user_addr = block.addr + ALLOC_HEADER_SIZE;

            // Write header with size + ALLOCATED_FLAG
            let header = (total_size as u64) | ALLOCATED_FLAG;
            self.heap_write_u64_internal(block.addr, header);

            // If block is significantly larger, split it
            let remaining = block.size - total_size;
            if remaining >= ALLOC_HEADER_SIZE + 8 {
                // Put remainder back in free list (sorted insert)
                self.insert_free_block_sorted(FreeBlock {
                    addr: block.addr + total_size,
                    size: remaining,
                });
            }

            return Ok(user_addr as u64);
        }

        // Strategy 2: Bump allocate from end
        let new_ptr = self.heap_ptr + total_size;
        if new_ptr > self.heap_limit {
            return Err(VmError::HeapOutOfMemory);
        }

        // Grow heap vector if needed
        if new_ptr > self.heap.len() {
            self.heap.resize(new_ptr, 0);
        }

        // Write header with size + ALLOCATED_FLAG
        let block_addr = self.heap_ptr;
        let header = (total_size as u64) | ALLOCATED_FLAG;
        self.heap_write_u64_internal(block_addr, header);

        // User address is after header
        let user_addr = block_addr + ALLOC_HEADER_SIZE;
        self.heap_ptr = new_ptr;

        Ok(user_addr as u64)
    }

    /// Find a free block that can fit the requested size (first-fit)
    #[inline]
    fn find_free_block(&self, total_size: usize) -> Option<usize> {
        self.free_list
            .iter()
            .position(|block| block.size >= total_size)
    }

    /// Internal write for header (bypasses bounds check since we're writing to new area)
    #[inline]
    fn heap_write_u64_internal(&mut self, addr: usize, value: u64) {
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 8].copy_from_slice(&bytes);
    }

    /// Free a previously allocated block
    /// Returns the freed block back to the free list for reuse
    ///
    /// Double-free protection: checks ALLOCATED_FLAG in header
    pub fn heap_free(&mut self, user_addr: usize) -> VmResult<()> {
        if user_addr < ALLOC_HEADER_SIZE {
            return Err(VmError::HeapOutOfBounds);
        }

        // Header is right before user data
        let header_addr = user_addr - ALLOC_HEADER_SIZE;

        // Read header (contains size | ALLOCATED_FLAG)
        let header = self.heap_read_u64(header_addr)?;

        // Double-free protection: check if block is still allocated
        if header & ALLOCATED_FLAG == 0 {
            // Block is already free - this is a double-free!
            return Err(VmError::DoubleFree);
        }

        // Extract actual size (mask out the flag)
        let total_size = (header & SIZE_MASK) as usize;
        if total_size == 0 || total_size > self.heap_ptr {
            return Err(VmError::HeapOutOfBounds);
        }

        // Clear ALLOCATED_FLAG in header (mark as free)
        self.heap_write_u64_internal(header_addr, total_size as u64);

        // Create free block and add to list with merge
        let new_block = FreeBlock {
            addr: header_addr,
            size: total_size,
        };
        self.add_free_block_with_merge(new_block);

        Ok(())
    }

    /// Insert a free block into the sorted free list (binary search)
    fn insert_free_block_sorted(&mut self, block: FreeBlock) {
        let pos = self.free_list
            .binary_search_by_key(&block.addr, |b| b.addr)
            .unwrap_or_else(|i| i);
        self.free_list.insert(pos, block);
    }

    /// Add a free block to the list, merging with adjacent blocks if possible
    /// Optimized: uses binary search, no full sort needed
    fn add_free_block_with_merge(&mut self, mut block: FreeBlock) {
        // Find insertion position using binary search
        let pos = self.free_list
            .binary_search_by_key(&block.addr, |b| b.addr)
            .unwrap_or_else(|i| i);

        // Check if we can merge with previous block
        if pos > 0 {
            let prev = &self.free_list[pos - 1];
            if prev.addr + prev.size == block.addr {
                // Merge with previous: extend previous block
                block.addr = prev.addr;
                block.size += prev.size;
                self.free_list.remove(pos - 1);
                // Recurse to check for more merges (now at pos-1)
                return self.add_free_block_with_merge(block);
            }
        }

        // Check if we can merge with next block
        if pos < self.free_list.len() {
            let next = &self.free_list[pos];
            if block.addr + block.size == next.addr {
                // Merge with next: extend our block
                block.size += next.size;
                self.free_list.remove(pos);
                // Recurse to check for more merges
                return self.add_free_block_with_merge(block);
            }
        }

        // No merge possible, insert at correct position
        self.free_list.insert(pos, block);
    }

    /// Get total free space available (free list + remaining bump space)
    #[inline]
    pub fn heap_free_space(&self) -> usize {
        let free_list_space: usize = self.free_list.iter().map(|b| b.size).sum();
        let bump_space = self.heap_limit.saturating_sub(self.heap_ptr);
        free_list_space + bump_space
    }

    /// Get number of blocks in free list
    #[inline]
    pub fn free_block_count(&self) -> usize {
        self.free_list.len()
    }

    /// Read byte from heap
    #[inline]
    pub fn heap_read_u8(&self, addr: usize) -> VmResult<u8> {
        if addr >= self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        Ok(self.heap[addr])
    }

    /// Read u16 from heap (little-endian)
    #[inline]
    pub fn heap_read_u16(&self, addr: usize) -> VmResult<u16> {
        if addr + 2 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        Ok(u16::from_le_bytes([self.heap[addr], self.heap[addr + 1]]))
    }

    /// Read u32 from heap (little-endian)
    #[inline]
    pub fn heap_read_u32(&self, addr: usize) -> VmResult<u32> {
        if addr + 4 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        Ok(u32::from_le_bytes([
            self.heap[addr],
            self.heap[addr + 1],
            self.heap[addr + 2],
            self.heap[addr + 3],
        ]))
    }

    /// Read u64 from heap (little-endian)
    #[inline]
    pub fn heap_read_u64(&self, addr: usize) -> VmResult<u64> {
        if addr + 8 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        Ok(u64::from_le_bytes([
            self.heap[addr],
            self.heap[addr + 1],
            self.heap[addr + 2],
            self.heap[addr + 3],
            self.heap[addr + 4],
            self.heap[addr + 5],
            self.heap[addr + 6],
            self.heap[addr + 7],
        ]))
    }

    /// Write byte to heap
    /// Note: Uses heap.len() for bounds check (not heap_ptr) to support free-list reuse
    #[inline]
    pub fn heap_write_u8(&mut self, addr: usize, value: u8) -> VmResult<()> {
        if addr >= self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        self.heap[addr] = value;
        Ok(())
    }

    /// Write u16 to heap (little-endian)
    #[inline]
    pub fn heap_write_u16(&mut self, addr: usize, value: u16) -> VmResult<()> {
        if addr + 2 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 2].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write u32 to heap (little-endian)
    #[inline]
    pub fn heap_write_u32(&mut self, addr: usize, value: u32) -> VmResult<()> {
        if addr + 4 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write u64 to heap (little-endian)
    #[inline]
    pub fn heap_write_u64(&mut self, addr: usize, value: u64) -> VmResult<()> {
        if addr + 8 > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write bytes to heap
    #[inline]
    pub fn heap_write_bytes(&mut self, addr: usize, data: &[u8]) -> VmResult<()> {
        if addr + data.len() > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        self.heap[addr..addr + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Read bytes from heap
    #[inline]
    pub fn heap_read_bytes(&self, addr: usize, len: usize) -> VmResult<&[u8]> {
        if addr + len > self.heap.len() {
            return Err(VmError::HeapOutOfBounds);
        }
        Ok(&self.heap[addr..addr + len])
    }

    /// Get current heap size (bytes allocated)
    #[inline]
    pub fn heap_size(&self) -> usize {
        self.heap_ptr
    }

    /// Get current heap usage (alias for heap_size)
    #[inline]
    pub fn heap_used(&self) -> usize {
        self.heap_ptr
    }

    /// Get remaining heap space
    #[inline]
    pub fn heap_remaining(&self) -> usize {
        self.heap_limit.saturating_sub(self.heap_ptr)
    }

    // =========================================================================
    // Flag Operations
    // =========================================================================

    /// Set zero flag based on value
    #[inline]
    pub fn set_zero_flag(&mut self, value: u64) {
        if value == 0 {
            self.flags |= flags::ZERO;
        } else {
            self.flags &= !flags::ZERO;
        }
    }

    /// Set sign flag based on value
    #[inline]
    pub fn set_sign_flag(&mut self, value: u64) {
        if (value as i64) < 0 {
            self.flags |= flags::SIGN;
        } else {
            self.flags &= !flags::SIGN;
        }
    }

    /// Check if zero flag is set
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.flags & flags::ZERO != 0
    }

    /// Check if sign flag is set
    #[inline]
    pub fn is_negative(&self) -> bool {
        self.flags & flags::SIGN != 0
    }

    /// Check if carry flag is set
    #[inline]
    pub fn is_carry(&self) -> bool {
        self.flags & flags::CARRY != 0
    }

    /// Check if overflow flag is set
    #[inline]
    pub fn is_overflow(&self) -> bool {
        self.flags & flags::OVERFLOW != 0
    }

    /// Update flags after comparison
    pub fn update_cmp_flags(&mut self, a: u64, b: u64) {
        let result = a.wrapping_sub(b);
        self.set_zero_flag(result);
        self.set_sign_flag(result);

        // Carry flag: set if a < b (unsigned)
        if a < b {
            self.flags |= flags::CARRY;
        } else {
            self.flags &= !flags::CARRY;
        }

        // Overflow flag: set if signed overflow occurred
        let sa = (a as i64) < 0;
        let sb = (b as i64) < 0;
        let sr = (result as i64) < 0;
        if (sa != sb) && (sr != sa) {
            self.flags |= flags::OVERFLOW;
        } else {
            self.flags &= !flags::OVERFLOW;
        }
    }

    // =========================================================================
    // Bytecode Reading
    // =========================================================================

    /// Read u8 from bytecode at current IP, advance IP
    #[inline]
    pub fn read_u8(&mut self) -> VmResult<u8> {
        if self.ip >= self.code.len() {
            return Err(VmError::InvalidBytecode);
        }
        let val = self.code[self.ip];
        self.ip += 1;
        Ok(val)
    }

    /// Read i16 from bytecode (little-endian), advance IP
    #[inline]
    pub fn read_i16(&mut self) -> VmResult<i16> {
        if self.ip + 2 > self.code.len() {
            return Err(VmError::InvalidBytecode);
        }
        let val = i16::from_le_bytes([self.code[self.ip], self.code[self.ip + 1]]);
        self.ip += 2;
        Ok(val)
    }

    /// Read u16 from bytecode (little-endian), advance IP
    #[inline]
    pub fn read_u16(&mut self) -> VmResult<u16> {
        if self.ip + 2 > self.code.len() {
            return Err(VmError::InvalidBytecode);
        }
        let val = u16::from_le_bytes([self.code[self.ip], self.code[self.ip + 1]]);
        self.ip += 2;
        Ok(val)
    }

    /// Read u32 from bytecode (little-endian), advance IP
    #[inline]
    pub fn read_u32(&mut self) -> VmResult<u32> {
        if self.ip + 4 > self.code.len() {
            return Err(VmError::InvalidBytecode);
        }
        let val = u32::from_le_bytes([
            self.code[self.ip],
            self.code[self.ip + 1],
            self.code[self.ip + 2],
            self.code[self.ip + 3],
        ]);
        self.ip += 4;
        Ok(val)
    }

    /// Read u64 from bytecode (little-endian), advance IP
    #[inline]
    pub fn read_u64(&mut self) -> VmResult<u64> {
        if self.ip + 8 > self.code.len() {
            return Err(VmError::InvalidBytecode);
        }
        let val = u64::from_le_bytes([
            self.code[self.ip],
            self.code[self.ip + 1],
            self.code[self.ip + 2],
            self.code[self.ip + 3],
            self.code[self.ip + 4],
            self.code[self.ip + 5],
            self.code[self.ip + 6],
            self.code[self.ip + 7],
        ]);
        self.ip += 8;
        Ok(val)
    }

    // =========================================================================
    // Input/Output Operations
    // =========================================================================

    /// Read byte from input buffer
    #[inline]
    pub fn read_input(&self, offset: usize) -> VmResult<u8> {
        if offset >= self.input.len() {
            return Err(VmError::MemoryOutOfBounds);
        }
        Ok(self.input[offset])
    }

    /// Read u8 from input buffer (alias for read_input)
    #[inline]
    pub fn read_input_u8(&self, offset: usize) -> VmResult<u8> {
        self.read_input(offset)
    }

    /// Read u16 from input buffer (little-endian)
    #[inline]
    pub fn read_input_u16(&self, offset: usize) -> VmResult<u16> {
        if offset + 2 > self.input.len() {
            return Err(VmError::MemoryOutOfBounds);
        }
        Ok(u16::from_le_bytes([
            self.input[offset],
            self.input[offset + 1],
        ]))
    }

    /// Read u32 from input buffer (little-endian)
    #[inline]
    pub fn read_input_u32(&self, offset: usize) -> VmResult<u32> {
        if offset + 4 > self.input.len() {
            return Err(VmError::MemoryOutOfBounds);
        }
        Ok(u32::from_le_bytes([
            self.input[offset],
            self.input[offset + 1],
            self.input[offset + 2],
            self.input[offset + 3],
        ]))
    }

    /// Read u64 from input buffer (little-endian)
    #[inline]
    pub fn read_input_u64(&self, offset: usize) -> VmResult<u64> {
        if offset + 8 > self.input.len() {
            return Err(VmError::MemoryOutOfBounds);
        }
        Ok(u64::from_le_bytes([
            self.input[offset],
            self.input[offset + 1],
            self.input[offset + 2],
            self.input[offset + 3],
            self.input[offset + 4],
            self.input[offset + 5],
            self.input[offset + 6],
            self.input[offset + 7],
        ]))
    }

    /// Write u8 to output buffer
    #[inline]
    pub fn write_output_u8(&mut self, offset: usize, value: u8) -> VmResult<()> {
        if offset >= self.output.len() {
            self.output.resize(offset + 1, 0);
        }
        self.output[offset] = value;
        Ok(())
    }

    /// Write u16 to output buffer (little-endian)
    #[inline]
    pub fn write_output_u16(&mut self, offset: usize, value: u16) -> VmResult<()> {
        if offset + 2 > self.output.len() {
            self.output.resize(offset + 2, 0);
        }
        let bytes = value.to_le_bytes();
        self.output[offset] = bytes[0];
        self.output[offset + 1] = bytes[1];
        Ok(())
    }

    /// Write u32 to output buffer (little-endian)
    #[inline]
    pub fn write_output_u32(&mut self, offset: usize, value: u32) -> VmResult<()> {
        if offset + 4 > self.output.len() {
            self.output.resize(offset + 4, 0);
        }
        let bytes = value.to_le_bytes();
        self.output[offset] = bytes[0];
        self.output[offset + 1] = bytes[1];
        self.output[offset + 2] = bytes[2];
        self.output[offset + 3] = bytes[3];
        Ok(())
    }

    /// Write u64 to output buffer (little-endian)
    #[inline]
    pub fn write_output_u64(&mut self, offset: usize, value: u64) -> VmResult<()> {
        if offset + 8 > self.output.len() {
            self.output.resize(offset + 8, 0);
        }
        let bytes = value.to_le_bytes();
        self.output[offset] = bytes[0];
        self.output[offset + 1] = bytes[1];
        self.output[offset + 2] = bytes[2];
        self.output[offset + 3] = bytes[3];
        self.output[offset + 4] = bytes[4];
        self.output[offset + 5] = bytes[5];
        self.output[offset + 6] = bytes[6];
        self.output[offset + 7] = bytes[7];
        Ok(())
    }

    /// Get input length
    #[inline]
    pub fn input_len(&self) -> usize {
        self.input.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_registers() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Initial capacity
        assert_eq!(state.reg_count(), DEFAULT_REGISTER_CAPACITY);

        // Read uninitialized register (should return 0)
        assert_eq!(state.get_reg(100).unwrap(), 0);

        // Write to high register (should grow)
        state.set_reg(100, 42).unwrap();
        assert!(state.reg_count() > 100);
        assert_eq!(state.get_reg(100).unwrap(), 42);

        // Max register (255)
        state.set_reg(255, 999).unwrap();
        assert_eq!(state.get_reg(255).unwrap(), 999);
    }

    #[test]
    fn test_heap_allocation() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate some memory (now includes 8-byte header)
        // Layout: [header:8][user_data:104] = 112 bytes total
        let addr1 = state.heap_alloc(100).unwrap();
        assert_eq!(addr1, 8); // User addr is after 8-byte header
        assert_eq!(state.heap_used(), 112); // 8 (header) + 104 (aligned user data)

        // Second allocation: [header:8][user_data:56] = 64 bytes
        let addr2 = state.heap_alloc(50).unwrap();
        assert_eq!(addr2, 120); // 112 + 8 (header)
        assert_eq!(state.heap_used(), 176); // 112 + 64
    }

    #[test]
    fn test_heap_read_write() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate and write
        let addr = state.heap_alloc(16).unwrap() as usize;
        state.heap_write_u64(addr, 0xDEADBEEF_CAFEBABE).unwrap();
        state.heap_write_u8(addr + 8, 42).unwrap();

        // Read back
        assert_eq!(state.heap_read_u64(addr).unwrap(), 0xDEADBEEF_CAFEBABE);
        assert_eq!(state.heap_read_u8(addr + 8).unwrap(), 42);
    }

    #[test]
    fn test_heap_limit() {
        let code = &[];
        let input = &[];
        // Limit must account for header overhead (8 bytes per allocation)
        let mut state = VmState::with_heap_limit(code, input, 200);

        // First allocation: 8 (header) + 56 (aligned 50) = 64 bytes
        state.heap_alloc(50).unwrap();

        // Second allocation: needs 8 + 104 = 112 bytes, total would be 176 < 200
        state.heap_alloc(100).unwrap();

        // Third allocation should fail (64 + 112 + 64 = 240 > 200)
        let result = state.heap_alloc(50);
        assert_eq!(result, Err(VmError::HeapOutOfMemory));
    }

    #[test]
    fn test_reset() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Modify state
        state.set_reg(100, 42).unwrap();
        state.heap_alloc(1000).unwrap();
        state.push(123).unwrap();

        // Reset
        state.reset();

        // Verify reset
        assert_eq!(state.reg_count(), DEFAULT_REGISTER_CAPACITY);
        assert_eq!(state.heap_used(), 0);
        assert_eq!(state.stack_len(), 0);
        assert_eq!(state.free_block_count(), 0);
    }

    #[test]
    fn test_heap_free_basic() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate memory
        let addr = state.heap_alloc(100).unwrap() as usize;
        let heap_after_alloc = state.heap_used();
        assert_eq!(state.free_block_count(), 0);

        // Free it
        state.heap_free(addr).unwrap();
        assert_eq!(state.free_block_count(), 1);

        // heap_ptr doesn't change, but free list has the block
        assert_eq!(state.heap_used(), heap_after_alloc);
    }

    #[test]
    fn test_heap_free_reuse() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate 3 blocks
        let addr1 = state.heap_alloc(100).unwrap() as usize;
        let addr2 = state.heap_alloc(100).unwrap() as usize;
        let _addr3 = state.heap_alloc(100).unwrap();

        let heap_after_allocs = state.heap_used();

        // Free middle block
        state.heap_free(addr2).unwrap();
        assert_eq!(state.free_block_count(), 1);

        // Allocate again - should reuse freed block
        let addr4 = state.heap_alloc(100).unwrap() as usize;
        assert_eq!(addr4, addr2); // Same address reused!
        assert_eq!(state.heap_used(), heap_after_allocs); // No growth

        // Free first block
        state.heap_free(addr1).unwrap();

        // Allocate smaller - should reuse and possibly split
        let addr5 = state.heap_alloc(50).unwrap() as usize;
        assert_eq!(addr5, addr1); // Reused first block
    }

    #[test]
    fn test_heap_free_merge_adjacent() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate 3 adjacent blocks
        let addr1 = state.heap_alloc(64).unwrap() as usize;
        let addr2 = state.heap_alloc(64).unwrap() as usize;
        let addr3 = state.heap_alloc(64).unwrap() as usize;

        // Free in order: 1, 3, then 2
        state.heap_free(addr1).unwrap();
        assert_eq!(state.free_block_count(), 1);

        state.heap_free(addr3).unwrap();
        assert_eq!(state.free_block_count(), 2);

        // Free middle - should merge all three into one block
        state.heap_free(addr2).unwrap();
        assert_eq!(state.free_block_count(), 1); // Merged!

        // The merged block should be large enough for a bigger allocation
        let big_addr = state.heap_alloc(200).unwrap() as usize;
        assert_eq!(big_addr, addr1); // Reused merged block
    }

    #[test]
    fn test_heap_free_invalid() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Try to free invalid address (too small, before header would be)
        let result = state.heap_free(4);
        assert_eq!(result, Err(VmError::HeapOutOfBounds));

        // Allocate something
        let _addr = state.heap_alloc(100).unwrap();

        // Try to free address 0 (where header would be negative)
        let result = state.heap_free(0);
        assert_eq!(result, Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_heap_free_space() {
        let code = &[];
        let input = &[];
        let mut state = VmState::with_heap_limit(code, input, 1000);

        let initial_free = state.heap_free_space();
        assert_eq!(initial_free, 1000);

        // Allocate 100 bytes (+ 8 header = 108, aligned to 112)
        let addr = state.heap_alloc(100).unwrap() as usize;
        assert_eq!(state.heap_free_space(), 1000 - 112);

        // Free it - free space should include free list block
        state.heap_free(addr).unwrap();
        assert_eq!(state.heap_free_space(), 1000 - 112 + 112); // Back to 1000
    }
}
