//! VM State management
//!
//! This module defines the VM execution state including:
//! - Dynamic registers (up to 256, R0-R255)
//! - Managed heap with bump allocator
//! - Value stack and call stack
//! - CPU flags and execution control

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::{VmError, VmResult};
use crate::opcodes::flags;

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

    // ========== Heap (Bump Allocator) ==========
    /// Managed heap memory
    pub heap: Vec<u8>,
    /// Current heap allocation pointer (bump pointer)
    pub heap_ptr: usize,
    /// Maximum heap size (DoS protection)
    pub heap_limit: usize,

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
        }
    }

    /// Create VM state with custom heap limit
    pub fn with_heap_limit(code: &'a [u8], input: &'a [u8], heap_limit: usize) -> Self {
        let mut state = Self::new(code, input);
        state.heap_limit = heap_limit.min(MAX_HEAP_SIZE);
        state
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
    // Heap Operations (Bump Allocator)
    // =========================================================================

    /// Allocate memory on the heap
    /// Returns the start address of the allocated block
    /// Alignment is guaranteed to be 8-byte aligned
    #[inline]
    pub fn heap_alloc(&mut self, size: usize) -> VmResult<u64> {
        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;

        // Check if allocation would exceed limit
        let new_ptr = self.heap_ptr + aligned_size;
        if new_ptr > self.heap_limit {
            return Err(VmError::HeapOutOfMemory);
        }

        // Grow heap vector if needed
        if new_ptr > self.heap.len() {
            self.heap.resize(new_ptr, 0);
        }

        // Record allocation start
        let addr = self.heap_ptr as u64;
        self.heap_ptr = new_ptr;

        Ok(addr)
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
    #[inline]
    pub fn heap_write_u8(&mut self, addr: usize, value: u8) -> VmResult<()> {
        if addr >= self.heap_ptr {
            return Err(VmError::HeapOutOfBounds);
        }
        self.heap[addr] = value;
        Ok(())
    }

    /// Write u16 to heap (little-endian)
    #[inline]
    pub fn heap_write_u16(&mut self, addr: usize, value: u16) -> VmResult<()> {
        if addr + 2 > self.heap_ptr {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 2].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write u32 to heap (little-endian)
    #[inline]
    pub fn heap_write_u32(&mut self, addr: usize, value: u32) -> VmResult<()> {
        if addr + 4 > self.heap_ptr {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write u64 to heap (little-endian)
    #[inline]
    pub fn heap_write_u64(&mut self, addr: usize, value: u64) -> VmResult<()> {
        if addr + 8 > self.heap_ptr {
            return Err(VmError::HeapOutOfBounds);
        }
        let bytes = value.to_le_bytes();
        self.heap[addr..addr + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Write bytes to heap
    #[inline]
    pub fn heap_write_bytes(&mut self, addr: usize, data: &[u8]) -> VmResult<()> {
        if addr + data.len() > self.heap_ptr {
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
    // String Operations (Heap-based)
    // =========================================================================

    /// Allocate and write a string to heap
    /// Returns address of the string (length-prefixed: u64 length + bytes)
    pub fn heap_alloc_string(&mut self, s: &str) -> VmResult<u64> {
        let bytes = s.as_bytes();
        let total_size = 8 + bytes.len(); // 8 bytes for length + string data

        let addr = self.heap_alloc(total_size)?;
        let addr_usize = addr as usize;

        // Write length prefix
        self.heap_write_u64(addr_usize, bytes.len() as u64)?;
        // Write string data
        if !bytes.is_empty() {
            self.heap_write_bytes(addr_usize + 8, bytes)?;
        }

        Ok(addr)
    }

    /// Read a string from heap (length-prefixed format)
    pub fn heap_read_string(&self, addr: u64) -> VmResult<&str> {
        let addr_usize = addr as usize;
        let len = self.heap_read_u64(addr_usize)? as usize;
        let bytes = self.heap_read_bytes(addr_usize + 8, len)?;
        core::str::from_utf8(bytes).map_err(|_| VmError::HeapOutOfBounds)
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

        // Allocate some memory
        let addr1 = state.heap_alloc(100).unwrap();
        assert_eq!(addr1, 0);
        assert_eq!(state.heap_used(), 104); // Aligned to 8

        let addr2 = state.heap_alloc(50).unwrap();
        assert_eq!(addr2, 104);
        assert_eq!(state.heap_used(), 160); // 104 + 56 (aligned)
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
    fn test_heap_string() {
        let code = &[];
        let input = &[];
        let mut state = VmState::new(code, input);

        // Allocate string
        let addr = state.heap_alloc_string("Hello, VM!").unwrap();

        // Read back
        let s = state.heap_read_string(addr).unwrap();
        assert_eq!(s, "Hello, VM!");
    }

    #[test]
    fn test_heap_limit() {
        let code = &[];
        let input = &[];
        let mut state = VmState::with_heap_limit(code, input, 100);

        // First allocation should succeed
        state.heap_alloc(50).unwrap();

        // Second allocation should fail (exceeds limit)
        let result = state.heap_alloc(100);
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
    }
}
