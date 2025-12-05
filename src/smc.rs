//! Self-Modifying Code (SMC) Engine
//!
//! Bytecode is encrypted at rest and decrypted only during execution.
//! After each instruction executes, it's re-encrypted.
//! This makes memory dumps useless - the full bytecode is never visible.
//!
//! ## How It Works
//!
//! ```text
//! T=0 (Start):     [???] [???] [???] [???] [???]  All encrypted
//! T=1 (IP=0):      [ADD] [???] [???] [???] [???]  Only current decrypted
//! T=2 (IP=1):      [???] [SUB] [???] [???] [???]  Previous re-encrypted
//! ```
//!
//! ## Security
//!
//! - Rolling XOR key derived from instruction position
//! - Each byte encrypted with: `encrypted = plain ^ key_for_position(pos)`
//! - Key derivation uses build-time seed for polymorphism

use crate::error::{VmError, VmResult};
use crate::native::NativeRegistry;
use crate::state::{VmState, FreeBlock, MAX_INSTRUCTIONS, DEFAULT_REGISTER_CAPACITY};
use crate::build_config::OPCODE_DECODE;
use crate::handlers;
use crate::opcodes::{arithmetic, control, convert, exec, heap, memory, native, register, special, stack, string, vector};

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

/// SMC Configuration
#[derive(Clone)]
pub struct SmcConfig {
    /// Base key for encryption (derived from build seed)
    pub key: [u8; 32],
    /// Number of instructions to keep decrypted (sliding window)
    /// 1 = most secure, higher = better performance for loops
    pub window_size: usize,
}

impl Default for SmcConfig {
    fn default() -> Self {
        Self {
            key: [0; 32],
            window_size: 1,
        }
    }
}

impl SmcConfig {
    /// Create config with build-time derived key
    pub fn from_build_seed(seed: u64) -> Self {
        let mut key = [0u8; 32];
        let mut state = seed;
        for i in 0..32 {
            state = state.wrapping_mul(0x5DEECE66D).wrapping_add(0xB);
            key[i] = (state >> 24) as u8;
        }
        Self {
            key,
            window_size: 1,
        }
    }

    /// Set window size
    pub fn with_window(mut self, size: usize) -> Self {
        self.window_size = size.max(1);
        self
    }
}

/// Generate position-dependent key byte
#[inline]
fn key_at(config: &SmcConfig, pos: usize) -> u8 {
    // Mix position with key bytes for rolling encryption
    let key_idx = pos % 32;
    let position_mix = (pos as u64)
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(0xC6A4A7935BD1E995);
    config.key[key_idx] ^ (position_mix >> 32) as u8 ^ (position_mix as u8)
}

/// Decrypt a single byte in place
#[inline]
fn decrypt_byte(code: &mut [u8], pos: usize, config: &SmcConfig) {
    code[pos] ^= key_at(config, pos);
}

/// Encrypt a single byte in place (same as decrypt - XOR is symmetric)
#[inline]
fn encrypt_byte(code: &mut [u8], pos: usize, config: &SmcConfig) {
    code[pos] ^= key_at(config, pos);
}

/// Decrypt a range of bytes
fn decrypt_range(code: &mut [u8], start: usize, len: usize, config: &SmcConfig) {
    for i in 0..len {
        if start + i < code.len() {
            decrypt_byte(code, start + i, config);
        }
    }
}

/// Encrypt a range of bytes
fn encrypt_range(code: &mut [u8], start: usize, len: usize, config: &SmcConfig) {
    for i in 0..len {
        if start + i < code.len() {
            encrypt_byte(code, start + i, config);
        }
    }
}

/// Get instruction length based on opcode
/// Returns the total bytes including opcode
fn instruction_length(base_opcode: u8) -> usize {
    match base_opcode {
        // 1-byte instructions (opcode only)
        stack::DUP | stack::SWAP | stack::DROP |
        arithmetic::ADD | arithmetic::SUB | arithmetic::MUL |
        arithmetic::XOR | arithmetic::AND | arithmetic::OR |
        arithmetic::SHL | arithmetic::SHR | arithmetic::NOT |
        arithmetic::ROL | arithmetic::ROR | arithmetic::INC | arithmetic::DEC |
        arithmetic::DIV | arithmetic::MOD | arithmetic::IDIV | arithmetic::IMOD |
        control::CMP | control::RET |
        convert::SEXT8 | convert::SEXT16 | convert::SEXT32 |
        convert::TRUNC8 | convert::TRUNC16 | convert::TRUNC32 |
        special::NOP | exec::HALT |
        vector::VEC_NEW | vector::VEC_LEN | vector::VEC_CAP |
        vector::VEC_PUSH | vector::VEC_POP | vector::VEC_GET | vector::VEC_SET |
        vector::VEC_REPEAT | vector::VEC_CLEAR | vector::VEC_RESERVE |
        string::STR_NEW | string::STR_LEN | string::STR_PUSH |
        string::STR_GET | string::STR_SET | string::STR_CMP |
        string::STR_EQ | string::STR_HASH | string::STR_CONCAT |
        heap::HEAP_ALLOC | heap::HEAP_FREE |
        heap::HEAP_LOAD8 | heap::HEAP_LOAD16 | heap::HEAP_LOAD32 | heap::HEAP_LOAD64 |
        heap::HEAP_STORE8 | heap::HEAP_STORE16 | heap::HEAP_STORE32 | heap::HEAP_STORE64 |
        heap::HEAP_SIZE |
        special::OPAQUE_TRUE | special::OPAQUE_FALSE => 1,

        // 2-byte instructions (opcode + u8)
        stack::PUSH_IMM8 | stack::PUSH_REG | stack::POP_REG |
        special::NOP_N => 2,

        // 3-byte instructions (opcode + u16 or 2xu8)
        stack::PUSH_IMM16 |
        control::JMP | control::JZ | control::JNZ |
        control::JGT | control::JLT | control::JGE | control::JLE |
        control::CALL |
        register::MOV_REG |
        native::NATIVE_READ | native::NATIVE_WRITE => 3,

        // 5-byte instructions (opcode + u32)
        stack::PUSH_IMM32 => 5,

        // 9-byte instructions (opcode + u64)
        stack::PUSH_IMM => 9,

        // 10-byte instructions (opcode + u8 + u64)
        register::MOV_IMM => 10,

        // Variable length - use max safe value
        _ => 1,
    }
}

/// Persistent execution state for SMC (without code reference)
/// This allows us to mutate code while preserving execution state
struct SmcExecState {
    regs: Vec<u64>,
    heap: Vec<u8>,
    heap_ptr: usize,
    heap_limit: usize,
    free_list: Vec<FreeBlock>,
    stack: Vec<u64>,
    call_stack: Vec<usize>,
    ip: usize,
    flags: u8,
    instruction_count: u64,
    halted: bool,
    result: u64,
    last_error: VmError,
    output: Vec<u8>,
    last_timing_ns: u64,
    start_time_ns: u64,
}

impl SmcExecState {
    fn new() -> Self {
        Self {
            regs: vec![0u64; DEFAULT_REGISTER_CAPACITY],
            heap: Vec::with_capacity(4096),
            heap_ptr: 0,
            heap_limit: 1024 * 1024,
            free_list: Vec::with_capacity(16),
            stack: Vec::with_capacity(64),
            call_stack: Vec::with_capacity(16),
            ip: 0,
            flags: 0,
            instruction_count: 0,
            halted: false,
            result: 0,
            last_error: VmError::Ok,
            output: Vec::new(),
            last_timing_ns: 0,
            start_time_ns: 0,
        }
    }

    /// Copy state from VmState
    fn copy_from(&mut self, state: &VmState) {
        self.regs.clone_from(&state.regs);
        self.heap.clone_from(&state.heap);
        self.heap_ptr = state.heap_ptr;
        self.heap_limit = state.heap_limit;
        self.free_list.clone_from(&state.free_list);
        self.stack.clone_from(&state.stack);
        self.call_stack.clone_from(&state.call_stack);
        self.ip = state.ip;
        self.flags = state.flags;
        self.instruction_count = state.instruction_count;
        self.halted = state.halted;
        self.result = state.result;
        self.last_error = state.last_error;
        self.output.clone_from(&state.output);
        self.last_timing_ns = state.last_timing_ns;
        self.start_time_ns = state.start_time_ns;
    }

    /// Apply state to VmState
    fn apply_to<'a>(&self, state: &mut VmState<'a>) {
        state.regs.clone_from(&self.regs);
        state.heap.clone_from(&self.heap);
        state.heap_ptr = self.heap_ptr;
        state.heap_limit = self.heap_limit;
        state.free_list.clone_from(&self.free_list);
        state.stack.clone_from(&self.stack);
        state.call_stack.clone_from(&self.call_stack);
        state.ip = self.ip;
        state.flags = self.flags;
        state.instruction_count = self.instruction_count;
        state.halted = self.halted;
        state.result = self.result;
        state.last_error = self.last_error;
        state.output.clone_from(&self.output);
        state.last_timing_ns = self.last_timing_ns;
        state.start_time_ns = self.start_time_ns;
    }
}

/// SMC-enabled execution
/// Takes ownership of bytecode for in-place modification
pub fn execute_smc(mut code: Vec<u8>, input: &[u8], config: &SmcConfig) -> VmResult<u64> {
    let registry = NativeRegistry::new();
    execute_smc_with_natives(&mut code, input, config, &registry)
}

/// SMC-enabled execution with native functions
pub fn execute_smc_with_natives(
    code: &mut Vec<u8>,
    input: &[u8],
    config: &SmcConfig,
    registry: &NativeRegistry,
) -> VmResult<u64> {
    // Track decrypted regions for sliding window
    let mut decrypted: Vec<(usize, usize)> = Vec::with_capacity(config.window_size + 1);

    // Persistent state (separate from VmState)
    let mut exec_state = SmcExecState::new();

    while !exec_state.halted && exec_state.ip < code.len() {
        let ip = exec_state.ip;

        // Instruction count limit
        exec_state.instruction_count += 1;
        if exec_state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        // Decrypt current instruction opcode
        decrypt_byte(code, ip, config);
        let opcode = code[ip];

        // Decode to get instruction length
        let base_opcode = OPCODE_DECODE[opcode as usize];
        let inst_len = instruction_length(base_opcode);

        // Decrypt operands if any
        if inst_len > 1 {
            decrypt_range(code, ip + 1, inst_len - 1, config);
        }

        // Track this decrypted region
        decrypted.push((ip, inst_len));

        // Execute instruction in a temporary scope
        {
            // Create temporary VmState with current code view
            let mut state = VmState::new(code.as_slice(), input);
            exec_state.apply_to(&mut state);

            // IMPORTANT: Advance IP past opcode before calling handler
            // Handlers expect IP to point AFTER the opcode (at operands)
            state.ip = ip + 1;

            // Execute instruction
            dispatch_smc(&mut state, opcode, registry)?;

            // Copy state back
            exec_state.copy_from(&state);
        }
        // VmState dropped here, code can be mutated

        // Re-encrypt old instructions outside window
        while decrypted.len() > config.window_size {
            let (old_ip, old_len) = decrypted.remove(0);
            encrypt_range(code, old_ip, old_len, config);
        }
    }

    // Re-encrypt any remaining decrypted instructions
    for (old_ip, old_len) in decrypted {
        encrypt_range(code, old_ip, old_len, config);
    }

    Ok(exec_state.result)
}

/// Encrypt bytecode for SMC execution
pub fn encrypt_bytecode(code: &mut [u8], config: &SmcConfig) {
    for i in 0..code.len() {
        encrypt_byte(code, i, config);
    }
}

/// Decrypt bytecode (for debugging/testing)
pub fn decrypt_bytecode(code: &mut [u8], config: &SmcConfig) {
    for i in 0..code.len() {
        decrypt_byte(code, i, config);
    }
}

/// SMC dispatch - same as normal dispatch
#[inline]
fn dispatch_smc(state: &mut VmState, opcode: u8, registry: &NativeRegistry) -> VmResult<()> {
    let base_opcode = OPCODE_DECODE[opcode as usize];

    match base_opcode {
        // ========== Stack Operations ==========
        stack::PUSH_IMM => handlers::handle_push_imm(state),
        stack::PUSH_IMM8 => handlers::handle_push_imm8(state),
        stack::PUSH_IMM16 => handlers::handle_push_imm16(state),
        stack::PUSH_IMM32 => handlers::handle_push_imm32(state),
        stack::PUSH_REG => handlers::handle_push_reg(state),
        stack::POP_REG => handlers::handle_pop_reg(state),
        stack::DUP => handlers::handle_dup(state),
        stack::SWAP => handlers::handle_swap(state),
        stack::DROP => handlers::handle_drop(state),

        // ========== Register Operations ==========
        register::MOV_IMM => handlers::handle_mov_imm(state),
        register::MOV_REG => handlers::handle_mov_reg(state),
        register::LOAD_MEM => handlers::handle_load_mem(state),
        register::STORE_MEM => handlers::handle_store_mem(state),

        // ========== Arithmetic ==========
        arithmetic::ADD => handlers::handle_add(state),
        arithmetic::SUB => handlers::handle_sub(state),
        arithmetic::MUL => handlers::handle_mul(state),
        arithmetic::XOR => handlers::handle_xor(state),
        arithmetic::AND => handlers::handle_and(state),
        arithmetic::OR => handlers::handle_or(state),
        arithmetic::SHL => handlers::handle_shl(state),
        arithmetic::SHR => handlers::handle_shr(state),
        arithmetic::NOT => handlers::handle_not(state),
        arithmetic::ROL => handlers::handle_rol(state),
        arithmetic::ROR => handlers::handle_ror(state),
        arithmetic::INC => handlers::handle_inc(state),
        arithmetic::DEC => handlers::handle_dec(state),
        arithmetic::DIV => handlers::handle_div(state),
        arithmetic::MOD => handlers::handle_mod(state),
        arithmetic::IDIV => handlers::handle_idiv(state),
        arithmetic::IMOD => handlers::handle_imod(state),

        // ========== Control Flow ==========
        control::CMP => handlers::handle_cmp(state),
        control::JMP => handlers::handle_jmp(state),
        control::JZ => handlers::handle_jz(state),
        control::JNZ => handlers::handle_jnz(state),
        control::JGT => handlers::handle_jgt(state),
        control::JLT => handlers::handle_jlt(state),
        control::JGE => handlers::handle_jge(state),
        control::JLE => handlers::handle_jle(state),
        control::CALL => handlers::handle_call(state),
        control::RET => handlers::handle_ret(state),

        // ========== Special ==========
        special::NOP => Ok(()),
        special::NOP_N => handlers::handle_nop_n(state),
        special::OPAQUE_TRUE => handlers::handle_opaque_true(state),
        special::OPAQUE_FALSE => handlers::handle_opaque_false(state),
        exec::HALT => handlers::handle_halt(state),

        // ========== Type Conversions ==========
        convert::SEXT8 => handlers::handle_sext8(state),
        convert::SEXT16 => handlers::handle_sext16(state),
        convert::SEXT32 => handlers::handle_sext32(state),
        convert::TRUNC8 => handlers::handle_trunc8(state),
        convert::TRUNC16 => handlers::handle_trunc16(state),
        convert::TRUNC32 => handlers::handle_trunc32(state),

        // ========== Native Functions ==========
        native::NATIVE_READ => handlers::handle_native_read(state),
        native::NATIVE_WRITE => handlers::handle_native_write(state),
        native::NATIVE_CALL => handlers::handle_native_call(state, registry),

        // ========== Memory Operations ==========
        memory::LOAD8 => handlers::handle_load8(state),
        memory::LOAD16 => handlers::handle_load16(state),
        memory::LOAD32 => handlers::handle_load32(state),
        memory::LOAD64 => handlers::handle_load64(state),
        memory::STORE8 => handlers::handle_store8(state),
        memory::STORE16 => handlers::handle_store16(state),
        memory::STORE32 => handlers::handle_store32(state),
        memory::STORE64 => handlers::handle_store64(state),

        // ========== Vector Operations ==========
        vector::VEC_NEW => handlers::handle_vec_new(state),
        vector::VEC_LEN => handlers::handle_vec_len(state),
        vector::VEC_CAP => handlers::handle_vec_cap(state),
        vector::VEC_PUSH => handlers::handle_vec_push(state),
        vector::VEC_POP => handlers::handle_vec_pop(state),
        vector::VEC_GET => handlers::handle_vec_get(state),
        vector::VEC_SET => handlers::handle_vec_set(state),
        vector::VEC_REPEAT => handlers::handle_vec_repeat(state),
        vector::VEC_CLEAR => handlers::handle_vec_clear(state),
        vector::VEC_RESERVE => handlers::handle_vec_reserve(state),

        // ========== String Operations ==========
        string::STR_NEW => handlers::handle_str_new(state),
        string::STR_LEN => handlers::handle_str_len(state),
        string::STR_PUSH => handlers::handle_str_push(state),
        string::STR_GET => handlers::handle_str_get(state),
        string::STR_SET => handlers::handle_str_set(state),
        string::STR_CMP => handlers::handle_str_cmp(state),
        string::STR_EQ => handlers::handle_str_eq(state),
        string::STR_HASH => handlers::handle_str_hash(state),
        string::STR_CONCAT => handlers::handle_str_concat(state),

        // ========== Heap Operations ==========
        heap::HEAP_ALLOC => handlers::handle_heap_alloc(state),
        heap::HEAP_FREE => handlers::handle_heap_free(state),
        heap::HEAP_LOAD8 => handlers::handle_heap_load8(state),
        heap::HEAP_LOAD16 => handlers::handle_heap_load16(state),
        heap::HEAP_LOAD32 => handlers::handle_heap_load32(state),
        heap::HEAP_LOAD64 => handlers::handle_heap_load64(state),
        heap::HEAP_STORE8 => handlers::handle_heap_store8(state),
        heap::HEAP_STORE16 => handlers::handle_heap_store16(state),
        heap::HEAP_STORE32 => handlers::handle_heap_store32(state),
        heap::HEAP_STORE64 => handlers::handle_heap_store64(state),
        heap::HEAP_SIZE => handlers::handle_heap_size(state),

        _ => Err(VmError::InvalidOpcode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_symmetric() {
        let config = SmcConfig::from_build_seed(12345);
        let original = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut code = original.clone();

        encrypt_bytecode(&mut code, &config);
        assert_ne!(code, original, "Encrypted should differ from original");

        decrypt_bytecode(&mut code, &config);
        assert_eq!(code, original, "Decrypted should match original");
    }

    #[test]
    fn test_key_at_deterministic() {
        let config = SmcConfig::from_build_seed(12345);
        let key1 = key_at(&config, 0);
        let key2 = key_at(&config, 0);
        assert_eq!(key1, key2, "Same position should give same key");

        let key3 = key_at(&config, 1);
        assert_ne!(key1, key3, "Different positions should give different keys");
    }

    #[test]
    fn test_instruction_length() {
        assert_eq!(instruction_length(arithmetic::ADD), 1);
        assert_eq!(instruction_length(stack::PUSH_IMM8), 2);
        assert_eq!(instruction_length(control::JMP), 3);
        assert_eq!(instruction_length(stack::PUSH_IMM), 9);
    }
}
