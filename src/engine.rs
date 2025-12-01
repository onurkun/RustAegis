//! VM Execution Engine
//!
//! Main dispatch loop - delegates to handler modules

use crate::error::{VmError, VmResult};
use crate::native::NativeRegistry;
use crate::opcodes::{arithmetic, control, convert, exec, heap, memory, native, register, special, stack, string, vector};
use crate::state::{VmState, MAX_INSTRUCTIONS};
use crate::build_config::OPCODE_DECODE;

// Handler modules
use crate::handlers;

/// Execute bytecode with given input, return result
pub fn execute(code: &[u8], input: &[u8]) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    run(&mut state)?;
    Ok(state.result)
}

/// Execute bytecode with native function registry
pub fn execute_with_natives(code: &[u8], input: &[u8], registry: &NativeRegistry) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    run_with_natives(&mut state, registry)?;
    Ok(state.result)
}

/// Execute bytecode, return full state (for debugging)
pub fn execute_with_state<'a>(code: &'a [u8], input: &'a [u8]) -> VmResult<VmState<'a>> {
    let mut state = VmState::new(code, input);
    run(&mut state)?;
    Ok(state)
}

/// Main execution loop (without native functions)
pub fn run(state: &mut VmState) -> VmResult<()> {
    let empty_registry = NativeRegistry::new();
    run_with_natives(state, &empty_registry)
}

/// Main execution loop with native function support
pub fn run_with_natives(state: &mut VmState, registry: &NativeRegistry) -> VmResult<()> {
    while !state.halted && state.ip < state.code.len() {
        // Instruction count limit
        state.instruction_count += 1;
        if state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        // Fetch opcode
        let opcode = state.read_u8()?;

        // Dispatch
        dispatch(state, opcode, registry)?;
    }

    Ok(())
}

/// Dispatch opcode to handler
#[inline]
fn dispatch(state: &mut VmState, opcode: u8, registry: &NativeRegistry) -> VmResult<()> {
    // Decode shuffled opcode to base opcode
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
        special::HASH_CHECK => handlers::handle_hash_check(state),
        special::TIMING_CHECK => handlers::handle_timing_check(state),

        // ========== Type Conversion ==========
        convert::SEXT8 => handlers::handle_sext8(state),
        convert::SEXT16 => handlers::handle_sext16(state),
        convert::SEXT32 => handlers::handle_sext32(state),
        convert::TRUNC8 => handlers::handle_trunc8(state),
        convert::TRUNC16 => handlers::handle_trunc16(state),
        convert::TRUNC32 => handlers::handle_trunc32(state),

        // ========== Memory (sized) ==========
        memory::LOAD8 => handlers::handle_load8(state),
        memory::LOAD16 => handlers::handle_load16(state),
        memory::LOAD32 => handlers::handle_load32(state),
        memory::LOAD64 => handlers::handle_load64(state),
        memory::STORE8 => handlers::handle_store8(state),
        memory::STORE16 => handlers::handle_store16(state),
        memory::STORE32 => handlers::handle_store32(state),
        memory::STORE64 => handlers::handle_store64(state),

        // ========== Heap ==========
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

        // ========== Vector ==========
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

        // ========== String ==========
        string::STR_NEW => handlers::handle_str_new(state),
        string::STR_LEN => handlers::handle_str_len(state),
        string::STR_PUSH => handlers::handle_str_push(state),
        string::STR_GET => handlers::handle_str_get(state),
        string::STR_SET => handlers::handle_str_set(state),
        string::STR_CMP => handlers::handle_str_cmp(state),
        string::STR_EQ => handlers::handle_str_eq(state),
        string::STR_HASH => handlers::handle_str_hash(state),
        string::STR_CONCAT => handlers::handle_str_concat(state),

        // ========== Native ==========
        native::NATIVE_CALL => handlers::handle_native_call(state, registry),
        native::NATIVE_READ => handlers::handle_native_read(state),
        native::NATIVE_WRITE => handlers::handle_native_write(state),
        native::INPUT_LEN => handlers::handle_input_len(state),

        // ========== Execution Control ==========
        exec::HALT => handlers::handle_halt(state),
        exec::HALT_ERR => handlers::handle_halt_err(state),

        _ => Err(VmError::InvalidOpcode),
    }
}
