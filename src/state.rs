//! VM State management

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::{VmError, VmResult};
use crate::opcodes::flags;

/// Maximum stack size (entries, not bytes)
pub const MAX_STACK_SIZE: usize = 1024;

/// Maximum instructions per execution (prevent infinite loops)
pub const MAX_INSTRUCTIONS: u64 = 1_000_000;

/// Number of general-purpose registers (R0-R7)
pub const NUM_REGISTERS: usize = 8;

/// VM execution state
#[derive(Debug, Clone)]
pub struct VmState<'a> {
    /// General-purpose registers (R0-R7)
    pub regs: [u64; NUM_REGISTERS],

    /// Value stack
    pub stack: Vec<u64>,

    /// Call stack (return addresses)
    pub call_stack: Vec<usize>,

    /// Instruction pointer
    pub ip: usize,

    /// CPU flags (Zero, Carry, Overflow, Sign)
    pub flags: u8,

    /// Bytecode being executed
    pub code: &'a [u8],

    /// Input data buffer (read-only)
    pub input: &'a [u8],

    /// Output data buffer
    pub output: Vec<u8>,

    /// Instruction counter (for max instruction limit)
    pub instruction_count: u64,

    /// Halted flag
    pub halted: bool,

    /// Result value (set by HALT)
    pub result: u64,

    /// Last error (if any)
    pub last_error: VmError,

    /// Last timing checkpoint (for anti-debug)
    /// Stores nanoseconds since execution start
    pub last_timing_ns: u64,

    /// Execution start time (for timing checks)
    pub start_time_ns: u64,
}

impl<'a> VmState<'a> {
    /// Create new VM state with given bytecode and input
    pub fn new(code: &'a [u8], input: &'a [u8]) -> Self {
        Self {
            regs: [0u64; NUM_REGISTERS],
            stack: Vec::with_capacity(64),
            call_stack: Vec::with_capacity(16),
            ip: 0,
            flags: 0,
            code,
            input,
            output: Vec::new(),
            instruction_count: 0,
            halted: false,
            result: 0,
            last_error: VmError::Ok,
            last_timing_ns: 0,
            start_time_ns: 0,
        }
    }

    /// Initialize timing for anti-debug checks
    #[inline]
    pub fn init_timing(&mut self) {
        #[cfg(all(feature = "std", not(feature = "vm_debug")))]
        {
            // Use system time in release mode with std
            use std::time::{SystemTime, UNIX_EPOCH};
            self.start_time_ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            self.last_timing_ns = self.start_time_ns;
        }
        #[cfg(any(not(feature = "std"), feature = "vm_debug"))]
        {
            // Skip timing in no_std or debug mode
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
        self.regs = [0u64; NUM_REGISTERS];
        self.stack.clear();
        self.call_stack.clear();
        self.ip = 0;
        self.flags = 0;
        self.output.clear();
        self.instruction_count = 0;
        self.halted = false;
        self.result = 0;
        self.last_error = VmError::Ok;
        self.last_timing_ns = 0;
        self.start_time_ns = 0;
    }

    // ========== Stack Operations ==========

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

    // ========== Register Operations ==========

    /// Get register value
    #[inline]
    pub fn get_reg(&self, idx: u8) -> VmResult<u64> {
        if idx as usize >= NUM_REGISTERS {
            return Err(VmError::InvalidRegister);
        }
        Ok(self.regs[idx as usize])
    }

    /// Set register value
    #[inline]
    pub fn set_reg(&mut self, idx: u8, value: u64) -> VmResult<()> {
        if idx as usize >= NUM_REGISTERS {
            return Err(VmError::InvalidRegister);
        }
        self.regs[idx as usize] = value;
        Ok(())
    }

    // ========== Flag Operations ==========

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

    // ========== Bytecode Reading ==========

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

    // ========== Input/Output ==========

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
        // Expand output buffer if needed
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
