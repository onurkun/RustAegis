//! Opcode definitions for the VM
//!
//! Instruction format:
//! - Most instructions are 1 byte opcode + optional operands
//! - Immediate values are little-endian
//! - Register indices are 0-255 (R0-R255)

/// Stack Operations
pub mod stack {
    /// Push 64-bit immediate value to stack
    /// Format: PUSH_IMM <u64 little-endian>
    pub const PUSH_IMM: u8 = 0x01;

    /// Push 8-bit immediate value to stack (zero-extended)
    /// Format: PUSH_IMM8 <u8>
    pub const PUSH_IMM8: u8 = 0x02;

    /// Push register value to stack
    /// Format: PUSH_REG <reg_idx>
    pub const PUSH_REG: u8 = 0x03;

    /// Pop stack to register
    /// Format: POP_REG <reg_idx>
    pub const POP_REG: u8 = 0x04;

    /// Duplicate top of stack
    /// Format: DUP
    pub const DUP: u8 = 0x05;

    /// Swap top two stack values
    /// Format: SWAP
    pub const SWAP: u8 = 0x06;

    /// Drop top of stack
    /// Format: DROP
    pub const DROP: u8 = 0x07;

    /// Push 16-bit immediate value to stack (zero-extended)
    /// Format: PUSH_IMM16 <u16 little-endian>
    pub const PUSH_IMM16: u8 = 0x08;

    /// Push 32-bit immediate value to stack (zero-extended)
    /// Format: PUSH_IMM32 <u32 little-endian>
    pub const PUSH_IMM32: u8 = 0x09;
}

/// Register Operations (R0-R7)
pub mod register {
    /// Load 64-bit immediate to register
    /// Format: MOV_IMM <reg_idx> <u64 little-endian>
    pub const MOV_IMM: u8 = 0x10;

    /// Copy register to register
    /// Format: MOV_REG <dst_reg> <src_reg>
    pub const MOV_REG: u8 = 0x11;

    /// Load from memory address in register
    /// Format: LOAD_MEM <dst_reg> <addr_reg>
    pub const LOAD_MEM: u8 = 0x12;

    /// Store to memory address in register
    /// Format: STORE_MEM <addr_reg> <src_reg>
    pub const STORE_MEM: u8 = 0x13;
}

/// Arithmetic Operations (Stack-based)
pub mod arithmetic {
    /// Pop 2, push sum
    /// Format: ADD
    pub const ADD: u8 = 0x20;

    /// Pop 2, push difference (a - b where b is top)
    /// Format: SUB
    pub const SUB: u8 = 0x21;

    /// Pop 2, push product
    /// Format: MUL
    pub const MUL: u8 = 0x22;

    /// Pop 2, push XOR
    /// Format: XOR
    pub const XOR: u8 = 0x23;

    /// Pop 2, push AND
    /// Format: AND
    pub const AND: u8 = 0x24;

    /// Pop 2, push OR
    /// Format: OR
    pub const OR: u8 = 0x25;

    /// Pop 2, push left shift (a << b)
    /// Format: SHL
    pub const SHL: u8 = 0x26;

    /// Pop 2, push right shift (a >> b)
    /// Format: SHR
    pub const SHR: u8 = 0x27;

    /// Pop 1, push bitwise NOT
    /// Format: NOT
    pub const NOT: u8 = 0x28;

    /// Pop 2, push rotate left
    /// Format: ROL
    pub const ROL: u8 = 0x29;

    /// Pop 2, push rotate right
    /// Format: ROR
    pub const ROR: u8 = 0x2A;

    /// Increment top of stack
    /// Format: INC
    pub const INC: u8 = 0x2B;

    /// Decrement top of stack
    /// Format: DEC
    pub const DEC: u8 = 0x2C;

    /// Unsigned division: a / b (division by zero returns 0)
    /// Format: DIV
    pub const DIV: u8 = 0x46;

    /// Unsigned modulo: a % b (division by zero returns 0)
    /// Format: MOD
    pub const MOD: u8 = 0x47;

    /// Signed division: (a as i64) / (b as i64)
    /// Format: IDIV
    pub const IDIV: u8 = 0x48;

    /// Signed modulo: (a as i64) % (b as i64)
    /// Format: IMOD
    pub const IMOD: u8 = 0x49;
}

/// Comparison & Control Flow
pub mod control {
    /// Compare top two stack values, set flags
    /// Format: CMP
    pub const CMP: u8 = 0x30;

    /// Unconditional jump
    /// Format: JMP <i16 relative offset>
    pub const JMP: u8 = 0x31;

    /// Jump if zero flag set
    /// Format: JZ <i16 relative offset>
    pub const JZ: u8 = 0x32;

    /// Jump if zero flag not set
    /// Format: JNZ <i16 relative offset>
    pub const JNZ: u8 = 0x33;

    /// Jump if greater (signed)
    /// Format: JGT <i16 relative offset>
    pub const JGT: u8 = 0x34;

    /// Jump if less (signed)
    /// Format: JLT <i16 relative offset>
    pub const JLT: u8 = 0x35;

    /// Jump if greater or equal
    /// Format: JGE <i16 relative offset>
    pub const JGE: u8 = 0x36;

    /// Jump if less or equal
    /// Format: JLE <i16 relative offset>
    pub const JLE: u8 = 0x37;

    /// Call subroutine (push return address, jump)
    /// Format: CALL <i16 relative offset>
    pub const CALL: u8 = 0x38;

    /// Return from subroutine
    /// Format: RET
    pub const RET: u8 = 0x39;
}

/// Special Operations (Anti-analysis)
pub mod special {
    /// No operation (1 byte)
    /// Format: NOP
    pub const NOP: u8 = 0x40;

    /// Variable-length NOP
    /// Format: NOP_N <u8 count>
    pub const NOP_N: u8 = 0x41;

    /// Opaque predicate (always true, but hard to prove statically)
    /// Format: OPAQUE_TRUE
    pub const OPAQUE_TRUE: u8 = 0x42;

    /// Opaque predicate (always false)
    /// Format: OPAQUE_FALSE
    pub const OPAQUE_FALSE: u8 = 0x43;

    /// Inline hash check
    /// Format: HASH_CHECK <expected_hash u32>
    pub const HASH_CHECK: u8 = 0x44;

    /// Timing check (anti-debug)
    /// Format: TIMING_CHECK
    pub const TIMING_CHECK: u8 = 0x45;
}

/// Type Conversion Operations
pub mod convert {
    /// Sign-extend 8-bit value to 64-bit
    /// Format: SEXT8
    pub const SEXT8: u8 = 0x50;

    /// Sign-extend 16-bit value to 64-bit
    /// Format: SEXT16
    pub const SEXT16: u8 = 0x51;

    /// Sign-extend 32-bit value to 64-bit
    /// Format: SEXT32
    pub const SEXT32: u8 = 0x52;

    /// Truncate to 8-bit (mask with 0xFF)
    /// Format: TRUNC8
    pub const TRUNC8: u8 = 0x53;

    /// Truncate to 16-bit (mask with 0xFFFF)
    /// Format: TRUNC16
    pub const TRUNC16: u8 = 0x54;

    /// Truncate to 32-bit (mask with 0xFFFFFFFF)
    /// Format: TRUNC32
    pub const TRUNC32: u8 = 0x55;
}

/// Memory Operations (sized loads/stores)
pub mod memory {
    /// Load 8-bit value from input buffer (zero-extended)
    /// Format: LOAD8 <offset u16>
    pub const LOAD8: u8 = 0x60;

    /// Load 16-bit value from input buffer (zero-extended, little-endian)
    /// Format: LOAD16 <offset u16>
    pub const LOAD16: u8 = 0x61;

    /// Load 32-bit value from input buffer (zero-extended, little-endian)
    /// Format: LOAD32 <offset u16>
    pub const LOAD32: u8 = 0x62;

    /// Load 64-bit value from input buffer (little-endian)
    /// Format: LOAD64 <offset u16>
    pub const LOAD64: u8 = 0x63;

    /// Store 8-bit value to output buffer
    /// Format: STORE8 <offset u16>
    pub const STORE8: u8 = 0x64;

    /// Store 16-bit value to output buffer (little-endian)
    /// Format: STORE16 <offset u16>
    pub const STORE16: u8 = 0x65;

    /// Store 32-bit value to output buffer (little-endian)
    /// Format: STORE32 <offset u16>
    pub const STORE32: u8 = 0x66;

    /// Store 64-bit value to output buffer (little-endian)
    /// Format: STORE64 <offset u16>
    pub const STORE64: u8 = 0x67;
}

/// Vector Operations (Dynamic Arrays)
/// Layout in heap: [capacity: u64, length: u64, elem_size: u64, data...]
pub mod vector {
    /// Create new vector with capacity
    /// Stack: [capacity, elem_size] -> [vec_addr]
    /// Format: VEC_NEW
    pub const VEC_NEW: u8 = 0x80;

    /// Get vector length
    /// Stack: [vec_addr] -> [length]
    /// Format: VEC_LEN
    pub const VEC_LEN: u8 = 0x81;

    /// Get vector capacity
    /// Stack: [vec_addr] -> [capacity]
    /// Format: VEC_CAP
    pub const VEC_CAP: u8 = 0x82;

    /// Push element to vector (auto-grows if needed)
    /// Stack: [vec_addr, value] -> []
    /// Format: VEC_PUSH
    pub const VEC_PUSH: u8 = 0x83;

    /// Pop element from vector
    /// Stack: [vec_addr] -> [value]
    /// Format: VEC_POP
    pub const VEC_POP: u8 = 0x84;

    /// Get element at index (arr[i])
    /// Stack: [vec_addr, index] -> [value]
    /// Format: VEC_GET
    pub const VEC_GET: u8 = 0x85;

    /// Set element at index (arr[i] = x)
    /// Stack: [vec_addr, index, value] -> []
    /// Format: VEC_SET
    pub const VEC_SET: u8 = 0x86;

    /// Create vector with repeated value ([value; count])
    /// Stack: [value, count, elem_size] -> [vec_addr]
    /// Format: VEC_REPEAT
    pub const VEC_REPEAT: u8 = 0x87;

    /// Clear vector (set length to 0)
    /// Stack: [vec_addr] -> []
    /// Format: VEC_CLEAR
    pub const VEC_CLEAR: u8 = 0x88;

    /// Reserve additional capacity
    /// Stack: [vec_addr, additional] -> []
    /// Format: VEC_RESERVE
    pub const VEC_RESERVE: u8 = 0x89;
}

/// String Operations (UTF-8 byte sequences)
/// String is Vec<u8> with elem_size=1
pub mod string {
    /// Create new string with capacity
    /// Stack: [capacity] -> [str_addr]
    /// Format: STR_NEW
    pub const STR_NEW: u8 = 0x90;

    /// Get string length (byte count)
    /// Stack: [str_addr] -> [length]
    /// Format: STR_LEN
    pub const STR_LEN: u8 = 0x91;

    /// Push byte/char to string
    /// Stack: [str_addr, byte] -> []
    /// Format: STR_PUSH
    pub const STR_PUSH: u8 = 0x92;

    /// Get byte at index
    /// Stack: [str_addr, index] -> [byte]
    /// Format: STR_GET
    pub const STR_GET: u8 = 0x93;

    /// Set byte at index
    /// Stack: [str_addr, index, byte] -> []
    /// Format: STR_SET
    pub const STR_SET: u8 = 0x94;

    /// Compare two strings (0=equal, 1=greater, -1=less)
    /// Stack: [str1_addr, str2_addr] -> [result]
    /// Format: STR_CMP
    pub const STR_CMP: u8 = 0x95;

    /// Check string equality (0 or 1)
    /// Stack: [str1_addr, str2_addr] -> [0/1]
    /// Format: STR_EQ
    pub const STR_EQ: u8 = 0x96;

    /// Hash string (FNV-1a)
    /// Stack: [str_addr] -> [hash]
    /// Format: STR_HASH
    pub const STR_HASH: u8 = 0x97;

    /// Concatenate two strings into new string
    /// Stack: [str1_addr, str2_addr] -> [new_str_addr]
    /// Format: STR_CONCAT
    pub const STR_CONCAT: u8 = 0x98;
}

/// Heap Operations (Dynamic Memory)
pub mod heap {
    /// Allocate memory on heap
    /// Stack: [size] -> [address]
    /// Format: HEAP_ALLOC
    pub const HEAP_ALLOC: u8 = 0x70;

    /// Free heap memory (no-op for bump allocator, reserved for future)
    /// Stack: [address] -> []
    /// Format: HEAP_FREE
    pub const HEAP_FREE: u8 = 0x71;

    /// Read u8 from heap
    /// Stack: [address] -> [value]
    /// Format: HEAP_LOAD8
    pub const HEAP_LOAD8: u8 = 0x72;

    /// Read u16 from heap (little-endian)
    /// Stack: [address] -> [value]
    /// Format: HEAP_LOAD16
    pub const HEAP_LOAD16: u8 = 0x73;

    /// Read u32 from heap (little-endian)
    /// Stack: [address] -> [value]
    /// Format: HEAP_LOAD32
    pub const HEAP_LOAD32: u8 = 0x74;

    /// Read u64 from heap (little-endian)
    /// Stack: [address] -> [value]
    /// Format: HEAP_LOAD64
    pub const HEAP_LOAD64: u8 = 0x75;

    /// Write u8 to heap
    /// Stack: [address, value] -> []
    /// Format: HEAP_STORE8
    pub const HEAP_STORE8: u8 = 0x76;

    /// Write u16 to heap (little-endian)
    /// Stack: [address, value] -> []
    /// Format: HEAP_STORE16
    pub const HEAP_STORE16: u8 = 0x77;

    /// Write u32 to heap (little-endian)
    /// Stack: [address, value] -> []
    /// Format: HEAP_STORE32
    pub const HEAP_STORE32: u8 = 0x78;

    /// Write u64 to heap (little-endian)
    /// Stack: [address, value] -> []
    /// Format: HEAP_STORE64
    pub const HEAP_STORE64: u8 = 0x79;

    /// Get current heap pointer (bytes used)
    /// Stack: [] -> [heap_ptr]
    /// Format: HEAP_SIZE
    pub const HEAP_SIZE: u8 = 0x7A;
}

/// Native Calls (Escape to Rust)
pub mod native {
    /// Call registered native function
    /// Format: NATIVE_CALL <func_id u8> <arg_count u8>
    pub const NATIVE_CALL: u8 = 0xF0;

    /// Read from native memory (input buffer)
    /// Format: NATIVE_READ <offset u16>
    pub const NATIVE_READ: u8 = 0xF1;

    /// Write to native memory (output buffer)
    /// Format: NATIVE_WRITE <offset u16>
    pub const NATIVE_WRITE: u8 = 0xF2;

    /// Load input length
    /// Format: INPUT_LEN
    pub const INPUT_LEN: u8 = 0xF3;
}

/// Execution Control
pub mod exec {
    /// Halt execution, return top of stack as result
    /// Format: HALT
    pub const HALT: u8 = 0xFF;

    /// Halt with error code
    /// Format: HALT_ERR <error_code u8>
    pub const HALT_ERR: u8 = 0xFE;
}

/// VM Flags (shuffled per-build for anti-analysis)
pub mod flags {
    pub use crate::build_config::flags::*;
}

/// Get opcode name for debugging
#[cfg(feature = "vm_debug")]
pub fn opcode_name(op: u8) -> &'static str {
    match op {
        stack::PUSH_IMM => "PUSH_IMM",
        stack::PUSH_IMM8 => "PUSH_IMM8",
        stack::PUSH_IMM16 => "PUSH_IMM16",
        stack::PUSH_IMM32 => "PUSH_IMM32",
        stack::PUSH_REG => "PUSH_REG",
        stack::POP_REG => "POP_REG",
        stack::DUP => "DUP",
        stack::SWAP => "SWAP",
        stack::DROP => "DROP",

        register::MOV_IMM => "MOV_IMM",
        register::MOV_REG => "MOV_REG",
        register::LOAD_MEM => "LOAD_MEM",
        register::STORE_MEM => "STORE_MEM",

        arithmetic::ADD => "ADD",
        arithmetic::SUB => "SUB",
        arithmetic::MUL => "MUL",
        arithmetic::XOR => "XOR",
        arithmetic::AND => "AND",
        arithmetic::OR => "OR",
        arithmetic::SHL => "SHL",
        arithmetic::SHR => "SHR",
        arithmetic::NOT => "NOT",
        arithmetic::ROL => "ROL",
        arithmetic::ROR => "ROR",
        arithmetic::INC => "INC",
        arithmetic::DEC => "DEC",
        arithmetic::DIV => "DIV",
        arithmetic::MOD => "MOD",
        arithmetic::IDIV => "IDIV",
        arithmetic::IMOD => "IMOD",

        control::CMP => "CMP",
        control::JMP => "JMP",
        control::JZ => "JZ",
        control::JNZ => "JNZ",
        control::JGT => "JGT",
        control::JLT => "JLT",
        control::JGE => "JGE",
        control::JLE => "JLE",
        control::CALL => "CALL",
        control::RET => "RET",

        special::NOP => "NOP",
        special::NOP_N => "NOP_N",
        special::OPAQUE_TRUE => "OPAQUE_TRUE",
        special::OPAQUE_FALSE => "OPAQUE_FALSE",
        special::HASH_CHECK => "HASH_CHECK",
        special::TIMING_CHECK => "TIMING_CHECK",

        convert::SEXT8 => "SEXT8",
        convert::SEXT16 => "SEXT16",
        convert::SEXT32 => "SEXT32",
        convert::TRUNC8 => "TRUNC8",
        convert::TRUNC16 => "TRUNC16",
        convert::TRUNC32 => "TRUNC32",

        memory::LOAD8 => "LOAD8",
        memory::LOAD16 => "LOAD16",
        memory::LOAD32 => "LOAD32",
        memory::LOAD64 => "LOAD64",
        memory::STORE8 => "STORE8",
        memory::STORE16 => "STORE16",
        memory::STORE32 => "STORE32",
        memory::STORE64 => "STORE64",

        vector::VEC_NEW => "VEC_NEW",
        vector::VEC_LEN => "VEC_LEN",
        vector::VEC_CAP => "VEC_CAP",
        vector::VEC_PUSH => "VEC_PUSH",
        vector::VEC_POP => "VEC_POP",
        vector::VEC_GET => "VEC_GET",
        vector::VEC_SET => "VEC_SET",
        vector::VEC_REPEAT => "VEC_REPEAT",
        vector::VEC_CLEAR => "VEC_CLEAR",
        vector::VEC_RESERVE => "VEC_RESERVE",

        string::STR_NEW => "STR_NEW",
        string::STR_LEN => "STR_LEN",
        string::STR_PUSH => "STR_PUSH",
        string::STR_GET => "STR_GET",
        string::STR_SET => "STR_SET",
        string::STR_CMP => "STR_CMP",
        string::STR_EQ => "STR_EQ",
        string::STR_HASH => "STR_HASH",
        string::STR_CONCAT => "STR_CONCAT",

        heap::HEAP_ALLOC => "HEAP_ALLOC",
        heap::HEAP_FREE => "HEAP_FREE",
        heap::HEAP_LOAD8 => "HEAP_LOAD8",
        heap::HEAP_LOAD16 => "HEAP_LOAD16",
        heap::HEAP_LOAD32 => "HEAP_LOAD32",
        heap::HEAP_LOAD64 => "HEAP_LOAD64",
        heap::HEAP_STORE8 => "HEAP_STORE8",
        heap::HEAP_STORE16 => "HEAP_STORE16",
        heap::HEAP_STORE32 => "HEAP_STORE32",
        heap::HEAP_STORE64 => "HEAP_STORE64",
        heap::HEAP_SIZE => "HEAP_SIZE",

        native::NATIVE_CALL => "NATIVE_CALL",
        native::NATIVE_READ => "NATIVE_READ",
        native::NATIVE_WRITE => "NATIVE_WRITE",
        native::INPUT_LEN => "INPUT_LEN",

        exec::HALT => "HALT",
        exec::HALT_ERR => "HALT_ERR",

        _ => "UNKNOWN",
    }
}
