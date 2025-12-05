//! VM Error types
//!
//! Error strings are obfuscated at compile time to prevent binary analysis.

use aegis_vm_macro::aegis_str_internal;
use core::fmt;

/// VM execution errors
///
/// Note: Debug impl only shows error code (E00-E20) to prevent string leakage.
/// Use `as_str()` for human-readable messages (decrypted at runtime).
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VmError {
    /// No error
    Ok = 0,
    /// Invalid or unknown opcode
    InvalidOpcode = 1,
    /// Stack underflow (pop from empty stack)
    StackUnderflow = 2,
    /// Stack overflow (exceeded max stack size)
    StackOverflow = 3,
    /// Invalid register index (> R7)
    InvalidRegister = 4,
    /// Division by zero
    DivisionByZero = 5,
    /// Jump target out of bounds
    InvalidJumpTarget = 6,
    /// Integrity check failed
    IntegrityFailed = 7,
    /// Timing anomaly detected (possible debugging)
    TimingAnomaly = 8,
    /// VM state corruption detected
    StateCorrupt = 9,
    /// Native function call failed
    NativeCallFailed = 10,
    /// Bytecode decryption failed
    DecryptionFailed = 11,
    /// Maximum instruction count exceeded
    MaxInstructionsExceeded = 12,
    /// Invalid bytecode format
    InvalidBytecode = 13,
    /// Memory access out of bounds
    MemoryOutOfBounds = 14,
    /// Native function not found
    NativeFunctionNotFound = 15,
    /// Native function already registered
    NativeFunctionAlreadyRegistered = 16,
    /// Too many arguments for native call
    NativeTooManyArgs = 17,
    /// Heap allocation failed (out of memory)
    HeapOutOfMemory = 18,
    /// Heap access out of bounds
    HeapOutOfBounds = 19,
    /// Double-free detected (freeing already freed block)
    DoubleFree = 20,
}

// Manual Debug impl - only shows error code, no string leakage
impl fmt::Debug for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:02}", self.code())
    }
}

// Display impl uses obfuscated strings
impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl VmError {
    /// Get error code string for debugging (obfuscated)
    ///
    /// Strings are encrypted at compile time and decrypted on first access.
    pub fn as_str(&self) -> &'static str {
        match self {
            VmError::Ok => aegis_str_internal!("VM_OK"),
            VmError::InvalidOpcode => aegis_str_internal!("VM_ERR_INVALID_OPCODE"),
            VmError::StackUnderflow => aegis_str_internal!("VM_ERR_STACK_UNDERFLOW"),
            VmError::StackOverflow => aegis_str_internal!("VM_ERR_STACK_OVERFLOW"),
            VmError::InvalidRegister => aegis_str_internal!("VM_ERR_INVALID_REGISTER"),
            VmError::DivisionByZero => aegis_str_internal!("VM_ERR_DIVISION_BY_ZERO"),
            VmError::InvalidJumpTarget => aegis_str_internal!("VM_ERR_INVALID_JUMP_TARGET"),
            VmError::IntegrityFailed => aegis_str_internal!("VM_ERR_INTEGRITY_FAILED"),
            VmError::TimingAnomaly => aegis_str_internal!("VM_ERR_TIMING_ANOMALY"),
            VmError::StateCorrupt => aegis_str_internal!("VM_ERR_STATE_CORRUPT"),
            VmError::NativeCallFailed => aegis_str_internal!("VM_ERR_NATIVE_CALL_FAILED"),
            VmError::DecryptionFailed => aegis_str_internal!("VM_ERR_DECRYPTION_FAILED"),
            VmError::MaxInstructionsExceeded => aegis_str_internal!("VM_ERR_MAX_INSTRUCTIONS"),
            VmError::InvalidBytecode => aegis_str_internal!("VM_ERR_INVALID_BYTECODE"),
            VmError::MemoryOutOfBounds => aegis_str_internal!("VM_ERR_MEMORY_OOB"),
            VmError::NativeFunctionNotFound => aegis_str_internal!("VM_ERR_NATIVE_NOT_FOUND"),
            VmError::NativeFunctionAlreadyRegistered => aegis_str_internal!("VM_ERR_NATIVE_ALREADY_REG"),
            VmError::NativeTooManyArgs => aegis_str_internal!("VM_ERR_NATIVE_TOO_MANY_ARGS"),
            VmError::HeapOutOfMemory => aegis_str_internal!("VM_ERR_HEAP_OOM"),
            VmError::HeapOutOfBounds => aegis_str_internal!("VM_ERR_HEAP_OOB"),
            VmError::DoubleFree => aegis_str_internal!("VM_ERR_DOUBLE_FREE"),
        }
    }

    /// Get numeric error code
    pub const fn code(&self) -> u8 {
        *self as u8
    }
}

/// Result type for VM operations
pub type VmResult<T> = Result<T, VmError>;
