//! VM execution engine

use crate::error::{VmError, VmResult};
use crate::native::{NativeRegistry, MAX_NATIVE_ARGS};
// Use base opcode values for matching after decode
use crate::opcodes::{arithmetic, control, convert, exec, heap, memory, native, register, special, stack};
use crate::state::{VmState, MAX_INSTRUCTIONS};
use crate::build_config::OPCODE_DECODE;

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
        stack::PUSH_IMM => handle_push_imm(state),
        stack::PUSH_IMM8 => handle_push_imm8(state),
        stack::PUSH_IMM16 => handle_push_imm16(state),
        stack::PUSH_IMM32 => handle_push_imm32(state),
        stack::PUSH_REG => handle_push_reg(state),
        stack::POP_REG => handle_pop_reg(state),
        stack::DUP => handle_dup(state),
        stack::SWAP => handle_swap(state),
        stack::DROP => handle_drop(state),

        // ========== Register Operations ==========
        register::MOV_IMM => handle_mov_imm(state),
        register::MOV_REG => handle_mov_reg(state),
        register::LOAD_MEM => handle_load_mem(state),
        register::STORE_MEM => handle_store_mem(state),

        // ========== Arithmetic ==========
        arithmetic::ADD => handle_add(state),
        arithmetic::SUB => handle_sub(state),
        arithmetic::MUL => handle_mul(state),
        arithmetic::XOR => handle_xor(state),
        arithmetic::AND => handle_and(state),
        arithmetic::OR => handle_or(state),
        arithmetic::SHL => handle_shl(state),
        arithmetic::SHR => handle_shr(state),
        arithmetic::NOT => handle_not(state),
        arithmetic::ROL => handle_rol(state),
        arithmetic::ROR => handle_ror(state),
        arithmetic::INC => handle_inc(state),
        arithmetic::DEC => handle_dec(state),
        arithmetic::DIV => handle_div(state),
        arithmetic::MOD => handle_mod(state),
        arithmetic::IDIV => handle_idiv(state),
        arithmetic::IMOD => handle_imod(state),

        // ========== Control Flow ==========
        control::CMP => handle_cmp(state),
        control::JMP => handle_jmp(state),
        control::JZ => handle_jz(state),
        control::JNZ => handle_jnz(state),
        control::JGT => handle_jgt(state),
        control::JLT => handle_jlt(state),
        control::JGE => handle_jge(state),
        control::JLE => handle_jle(state),
        control::CALL => handle_call(state),
        control::RET => handle_ret(state),

        // ========== Special ==========
        special::NOP => Ok(()),
        special::NOP_N => handle_nop_n(state),
        special::OPAQUE_TRUE => handle_opaque_true(state),
        special::OPAQUE_FALSE => handle_opaque_false(state),
        special::HASH_CHECK => handle_hash_check(state),
        special::TIMING_CHECK => handle_timing_check(state),

        // ========== Type Conversion ==========
        convert::SEXT8 => handle_sext8(state),
        convert::SEXT16 => handle_sext16(state),
        convert::SEXT32 => handle_sext32(state),
        convert::TRUNC8 => handle_trunc8(state),
        convert::TRUNC16 => handle_trunc16(state),
        convert::TRUNC32 => handle_trunc32(state),

        // ========== Memory (sized) ==========
        memory::LOAD8 => handle_load8(state),
        memory::LOAD16 => handle_load16(state),
        memory::LOAD32 => handle_load32(state),
        memory::LOAD64 => handle_load64(state),
        memory::STORE8 => handle_store8(state),
        memory::STORE16 => handle_store16(state),
        memory::STORE32 => handle_store32(state),
        memory::STORE64 => handle_store64(state),

        // ========== Heap ==========
        heap::HEAP_ALLOC => handle_heap_alloc(state),
        heap::HEAP_FREE => handle_heap_free(state),
        heap::HEAP_LOAD8 => handle_heap_load8(state),
        heap::HEAP_LOAD16 => handle_heap_load16(state),
        heap::HEAP_LOAD32 => handle_heap_load32(state),
        heap::HEAP_LOAD64 => handle_heap_load64(state),
        heap::HEAP_STORE8 => handle_heap_store8(state),
        heap::HEAP_STORE16 => handle_heap_store16(state),
        heap::HEAP_STORE32 => handle_heap_store32(state),
        heap::HEAP_STORE64 => handle_heap_store64(state),
        heap::HEAP_SIZE => handle_heap_size(state),

        // ========== Native ==========
        native::NATIVE_CALL => handle_native_call(state, registry),
        native::NATIVE_READ => handle_native_read(state),
        native::NATIVE_WRITE => handle_native_write(state),
        native::INPUT_LEN => handle_input_len(state),

        // ========== Execution Control ==========
        exec::HALT => handle_halt(state),
        exec::HALT_ERR => handle_halt_err(state),

        _ => Err(VmError::InvalidOpcode),
    }
}

// ============================================================================
// Stack Operation Handlers
// ============================================================================

/// PUSH_IMM: Push 64-bit immediate to stack
fn handle_push_imm(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u64()?;
    state.push(value)
}

/// PUSH_IMM8: Push 8-bit immediate to stack (zero-extended)
fn handle_push_imm8(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u8()? as u64;
    state.push(value)
}

/// PUSH_IMM16: Push 16-bit immediate to stack (zero-extended)
fn handle_push_imm16(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u16()? as u64;
    state.push(value)
}

/// PUSH_IMM32: Push 32-bit immediate to stack (zero-extended)
fn handle_push_imm32(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u32()? as u64;
    state.push(value)
}

/// PUSH_REG: Push register value to stack
fn handle_push_reg(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.get_reg(reg_idx)?;
    state.push(value)
}

/// POP_REG: Pop stack to register
fn handle_pop_reg(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.pop()?;
    state.set_reg(reg_idx, value)
}

/// DUP: Duplicate top of stack
fn handle_dup(state: &mut VmState) -> VmResult<()> {
    let value = state.peek()?;
    state.push(value)
}

/// SWAP: Swap top two stack values
/// Stack before: [..., below, top]
/// Stack after:  [..., top, below]
fn handle_swap(state: &mut VmState) -> VmResult<()> {
    let top = state.pop()?;
    let below = state.pop()?;
    state.push(top)?;
    state.push(below)
}

/// DROP: Drop top of stack
fn handle_drop(state: &mut VmState) -> VmResult<()> {
    state.pop()?;
    Ok(())
}

// ============================================================================
// Register Operation Handlers
// ============================================================================

/// MOV_IMM: Load immediate to register
fn handle_mov_imm(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.read_u64()?;
    state.set_reg(reg_idx, value)
}

/// MOV_REG: Copy register to register
fn handle_mov_reg(state: &mut VmState) -> VmResult<()> {
    let dst = state.read_u8()?;
    let src = state.read_u8()?;
    let value = state.get_reg(src)?;
    state.set_reg(dst, value)
}

/// LOAD_MEM: Load from input buffer using register as offset
fn handle_load_mem(state: &mut VmState) -> VmResult<()> {
    let dst_reg = state.read_u8()?;
    let addr_reg = state.read_u8()?;
    let offset = state.get_reg(addr_reg)? as usize;
    let value = state.read_input_u64(offset)?;
    state.set_reg(dst_reg, value)
}

/// STORE_MEM: Store to output buffer
fn handle_store_mem(state: &mut VmState) -> VmResult<()> {
    let _addr_reg = state.read_u8()?;
    let src_reg = state.read_u8()?;
    let value = state.get_reg(src_reg)?;
    state.output.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

// ============================================================================
// Arithmetic Handlers
// ============================================================================

/// ADD: Pop 2, push sum
fn handle_add(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.wrapping_add(b);
    state.set_zero_flag(result);
    state.push(result)
}

/// SUB: Pop 2, push difference
fn handle_sub(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.wrapping_sub(b);
    state.set_zero_flag(result);
    state.push(result)
}

/// MUL: Pop 2, push product
fn handle_mul(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.wrapping_mul(b);
    state.set_zero_flag(result);
    state.push(result)
}

/// XOR: Pop 2, push XOR
fn handle_xor(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a ^ b;
    state.set_zero_flag(result);
    state.push(result)
}

/// AND: Pop 2, push AND
fn handle_and(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a & b;
    state.set_zero_flag(result);
    state.push(result)
}

/// OR: Pop 2, push OR
fn handle_or(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a | b;
    state.set_zero_flag(result);
    state.push(result)
}

/// SHL: Pop 2, push left shift
fn handle_shl(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.wrapping_shl(b as u32);
    state.set_zero_flag(result);
    state.push(result)
}

/// SHR: Pop 2, push right shift
fn handle_shr(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.wrapping_shr(b as u32);
    state.set_zero_flag(result);
    state.push(result)
}

/// NOT: Pop 1, push bitwise NOT
fn handle_not(state: &mut VmState) -> VmResult<()> {
    let a = state.pop()?;
    let result = !a;
    state.set_zero_flag(result);
    state.push(result)
}

/// ROL: Rotate left
fn handle_rol(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.rotate_left(b as u32);
    state.push(result)
}

/// ROR: Rotate right
fn handle_ror(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = a.rotate_right(b as u32);
    state.push(result)
}

/// INC: Increment top of stack
fn handle_inc(state: &mut VmState) -> VmResult<()> {
    let a = state.pop()?;
    let result = a.wrapping_add(1);
    state.set_zero_flag(result);
    state.push(result)
}

/// DEC: Decrement top of stack
fn handle_dec(state: &mut VmState) -> VmResult<()> {
    let a = state.pop()?;
    let result = a.wrapping_sub(1);
    state.set_zero_flag(result);
    state.push(result)
}

/// DIV: Unsigned division (a / b), division by zero returns 0
fn handle_div(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = if b == 0 { 0 } else { a / b };
    state.set_zero_flag(result);
    state.push(result)
}

/// MOD: Unsigned modulo (a % b), division by zero returns 0
fn handle_mod(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    let result = if b == 0 { 0 } else { a % b };
    state.set_zero_flag(result);
    state.push(result)
}

/// IDIV: Signed division ((a as i64) / (b as i64))
fn handle_idiv(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()? as i64;
    let a = state.pop()? as i64;
    let result = if b == 0 { 0 } else { (a / b) as u64 };
    state.set_zero_flag(result);
    state.push(result)
}

/// IMOD: Signed modulo ((a as i64) % (b as i64))
fn handle_imod(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()? as i64;
    let a = state.pop()? as i64;
    let result = if b == 0 { 0 } else { (a % b) as u64 };
    state.set_zero_flag(result);
    state.push(result)
}

// ============================================================================
// Control Flow Handlers
// ============================================================================

/// CMP: Compare top two stack values, set flags
fn handle_cmp(state: &mut VmState) -> VmResult<()> {
    let b = state.pop()?;
    let a = state.pop()?;
    state.update_cmp_flags(a, b);
    // Push values back (CMP doesn't consume)
    state.push(a)?;
    state.push(b)
}

/// JMP: Unconditional jump
fn handle_jmp(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    jump_relative(state, offset)
}

/// JZ: Jump if zero flag set
fn handle_jz(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    if state.is_zero() {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// JNZ: Jump if zero flag not set
fn handle_jnz(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    if !state.is_zero() {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// JGT: Jump if greater (signed)
fn handle_jgt(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    // Greater: not zero AND sign == overflow
    if !state.is_zero() && (state.is_negative() == state.is_overflow()) {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// JLT: Jump if less (signed)
fn handle_jlt(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    // Less: sign != overflow
    if state.is_negative() != state.is_overflow() {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// JGE: Jump if greater or equal
fn handle_jge(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    // Greater or equal: sign == overflow
    if state.is_negative() == state.is_overflow() {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// JLE: Jump if less or equal
fn handle_jle(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    // Less or equal: zero OR (sign != overflow)
    if state.is_zero() || (state.is_negative() != state.is_overflow()) {
        jump_relative(state, offset)
    } else {
        Ok(())
    }
}

/// Helper: Jump by relative offset
fn jump_relative(state: &mut VmState, offset: i16) -> VmResult<()> {
    let new_ip = if offset >= 0 {
        state.ip.checked_add(offset as usize)
    } else {
        state.ip.checked_sub((-offset) as usize)
    };

    match new_ip {
        Some(ip) if ip <= state.code.len() => {
            state.ip = ip;
            Ok(())
        }
        _ => Err(VmError::InvalidJumpTarget),
    }
}

/// CALL: Call subroutine
fn handle_call(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_i16()?;
    // Push return address
    state.call_stack.push(state.ip);
    jump_relative(state, offset)
}

/// RET: Return from subroutine
fn handle_ret(state: &mut VmState) -> VmResult<()> {
    match state.call_stack.pop() {
        Some(return_addr) => {
            state.ip = return_addr;
            Ok(())
        }
        None => {
            // Return from main = halt
            state.halted = true;
            state.result = state.peek().unwrap_or(0);
            Ok(())
        }
    }
}

// ============================================================================
// Special Handlers (Anti-analysis)
// ============================================================================

/// NOP_N: Skip N bytes
fn handle_nop_n(state: &mut VmState) -> VmResult<()> {
    let count = state.read_u8()?;
    state.ip += count as usize;
    if state.ip > state.code.len() {
        return Err(VmError::InvalidJumpTarget);
    }
    Ok(())
}

/// OPAQUE_TRUE: Opaque predicate that always evaluates to true
/// Pushes 1 to stack
fn handle_opaque_true(state: &mut VmState) -> VmResult<()> {
    // x * (x + 1) is always even (product of two consecutive integers)
    // So x * (x + 1) % 2 == 0 is always true
    // We use a runtime value to prevent static analysis
    let x = state.instruction_count;
    let product = x.wrapping_mul(x.wrapping_add(1));
    #[allow(clippy::manual_is_multiple_of)]
    let result = if product % 2 == 0 {
        1u64
    } else {
        0u64 // Never reached
    };
    state.push(result)
}

/// OPAQUE_FALSE: Opaque predicate that always evaluates to false
/// Pushes 0 to stack
fn handle_opaque_false(state: &mut VmState) -> VmResult<()> {
    // x * (x + 1) is always even (product of two consecutive integers)
    // So x * (x + 1) % 2 != 0 is always false
    let x = state.instruction_count;
    let product = x.wrapping_mul(x.wrapping_add(1));
    #[allow(clippy::manual_is_multiple_of)]
    let result = if product % 2 != 0 {
        1u64 // Never reached
    } else {
        0u64
    };
    state.push(result)
}

/// HASH_CHECK: Verify bytecode integrity
fn handle_hash_check(state: &mut VmState) -> VmResult<()> {
    let expected = state.read_u32()?;

    // FNV-1a hash of bytecode (randomized constants per build)
    let mut hash = crate::build_config::FNV_BASIS_32;
    for &byte in state.code {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(crate::build_config::FNV_PRIME_32);
    }

    if hash != expected {
        return Err(VmError::IntegrityFailed);
    }
    Ok(())
}

/// TIMING_CHECK: Anti-debug timing check
///
/// Checks for timing anomalies that indicate:
/// - Debugger single-stepping (very slow execution)
/// - Code instrumentation/hooking (overhead)
/// - VM introspection
///
/// If execution is too slow between checkpoints, returns TimingAnomaly error.
fn handle_timing_check(state: &mut VmState) -> VmResult<()> {
    #[cfg(feature = "vm_debug")]
    {
        // Skip timing check in debug mode
        let _ = state;
        return Ok(());
    }

    #[cfg(not(feature = "vm_debug"))]
    {
        // Get current time
        let current_ns = state.current_time_ns();

        // If this is the first timing check, just record the time
        if state.last_timing_ns == 0 {
            state.last_timing_ns = current_ns;
            return Ok(());
        }

        // Calculate delta since last check
        let delta_ns = current_ns.saturating_sub(state.last_timing_ns);

        // Threshold: 100ms = 100_000_000 nanoseconds
        // Normal VM execution between checkpoints should be < 10ms
        // Debugging single-stepping would make this much slower
        const MAX_DELTA_NS: u64 = 100_000_000; // 100ms

        if delta_ns > MAX_DELTA_NS {
            // Timing anomaly detected - possible debugger
            return Err(VmError::TimingAnomaly);
        }

        // Update last timing checkpoint
        state.last_timing_ns = current_ns;
        Ok(())
    }
}

// ============================================================================
// Native Handlers
// ============================================================================

/// NATIVE_CALL: Call native function from registry
///
/// Format: NATIVE_CALL <func_id u8> <arg_count u8>
/// Pops arg_count values from stack (in reverse order), calls function, pushes result
fn handle_native_call(state: &mut VmState, registry: &NativeRegistry) -> VmResult<()> {
    let func_id = state.read_u8()?;
    let arg_count = state.read_u8()? as usize;

    // Check argument count limit
    if arg_count > MAX_NATIVE_ARGS {
        return Err(VmError::NativeTooManyArgs);
    }

    // Pop arguments from stack (they're in reverse order on stack)
    let mut args = [0u64; MAX_NATIVE_ARGS];
    for i in (0..arg_count).rev() {
        args[i] = state.pop()?;
    }

    // Call the native function
    let result = registry.call(func_id, &args[..arg_count])?;

    // Push result
    state.push(result)
}

/// NATIVE_READ: Read u64 from input buffer
fn handle_native_read(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u64(offset)?;
    state.push(value)
}

/// NATIVE_WRITE: Write to output buffer
fn handle_native_write(state: &mut VmState) -> VmResult<()> {
    let _offset = state.read_u16()?;
    let value = state.pop()?;
    state.output.push(value as u8);
    Ok(())
}

/// INPUT_LEN: Push input length to stack
fn handle_input_len(state: &mut VmState) -> VmResult<()> {
    let len = state.input_len() as u64;
    state.push(len)
}

// ============================================================================
// Type Conversion Handlers
// ============================================================================

/// SEXT8: Sign-extend 8-bit value to 64-bit
fn handle_sext8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i8) as i64 as u64;
    state.push(result)
}

/// SEXT16: Sign-extend 16-bit value to 64-bit
fn handle_sext16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i16) as i64 as u64;
    state.push(result)
}

/// SEXT32: Sign-extend 32-bit value to 64-bit
fn handle_sext32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i32) as i64 as u64;
    state.push(result)
}

/// TRUNC8: Truncate to 8-bit (mask with 0xFF)
fn handle_trunc8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFF;
    state.push(result)
}

/// TRUNC16: Truncate to 16-bit (mask with 0xFFFF)
fn handle_trunc16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFFFF;
    state.push(result)
}

/// TRUNC32: Truncate to 32-bit (mask with 0xFFFFFFFF)
fn handle_trunc32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFFFFFFFF;
    state.push(result)
}

// ============================================================================
// Memory Handlers (sized loads/stores)
// ============================================================================

/// LOAD8: Load 8-bit value from input buffer (zero-extended)
fn handle_load8(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u8(offset)? as u64;
    state.push(value)
}

/// LOAD16: Load 16-bit value from input buffer (zero-extended)
fn handle_load16(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u16(offset)? as u64;
    state.push(value)
}

/// LOAD32: Load 32-bit value from input buffer (zero-extended)
fn handle_load32(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u32(offset)? as u64;
    state.push(value)
}

/// LOAD64: Load 64-bit value from input buffer
fn handle_load64(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u64(offset)?;
    state.push(value)
}

/// STORE8: Store 8-bit value to output buffer
fn handle_store8(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u8(offset, value as u8)
}

/// STORE16: Store 16-bit value to output buffer
fn handle_store16(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u16(offset, value as u16)
}

/// STORE32: Store 32-bit value to output buffer
fn handle_store32(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u32(offset, value as u32)
}

/// STORE64: Store 64-bit value to output buffer
fn handle_store64(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u64(offset, value)
}

// ============================================================================
// Heap Handlers
// ============================================================================

/// HEAP_ALLOC: Allocate memory on heap
/// Stack: [size] -> [address]
fn handle_heap_alloc(state: &mut VmState) -> VmResult<()> {
    let size = state.pop()? as usize;
    let addr = state.heap_alloc(size)?;
    state.push(addr as u64)
}

/// HEAP_FREE: Free heap memory (no-op for bump allocator)
/// Stack: [address] -> []
fn handle_heap_free(state: &mut VmState) -> VmResult<()> {
    let _addr = state.pop()?;
    // No-op for bump allocator - reserved for future use
    Ok(())
}

/// HEAP_LOAD8: Read u8 from heap
/// Stack: [address] -> [value]
fn handle_heap_load8(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u8(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD16: Read u16 from heap (little-endian)
/// Stack: [address] -> [value]
fn handle_heap_load16(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u16(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD32: Read u32 from heap (little-endian)
/// Stack: [address] -> [value]
fn handle_heap_load32(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u32(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD64: Read u64 from heap (little-endian)
/// Stack: [address] -> [value]
fn handle_heap_load64(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u64(addr)?;
    state.push(value)
}

/// HEAP_STORE8: Write u8 to heap
/// Stack: [address, value] -> []
fn handle_heap_store8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u8;
    let addr = state.pop()? as usize;
    state.heap_write_u8(addr, value)
}

/// HEAP_STORE16: Write u16 to heap (little-endian)
/// Stack: [address, value] -> []
fn handle_heap_store16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u16;
    let addr = state.pop()? as usize;
    state.heap_write_u16(addr, value)
}

/// HEAP_STORE32: Write u32 to heap (little-endian)
/// Stack: [address, value] -> []
fn handle_heap_store32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u32;
    let addr = state.pop()? as usize;
    state.heap_write_u32(addr, value)
}

/// HEAP_STORE64: Write u64 to heap (little-endian)
/// Stack: [address, value] -> []
fn handle_heap_store64(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let addr = state.pop()? as usize;
    state.heap_write_u64(addr, value)
}

/// HEAP_SIZE: Get current heap pointer (bytes used)
/// Stack: [] -> [heap_ptr]
fn handle_heap_size(state: &mut VmState) -> VmResult<()> {
    let size = state.heap_size() as u64;
    state.push(size)
}

// ============================================================================
// Execution Control Handlers
// ============================================================================

/// HALT: Stop execution, result is top of stack
fn handle_halt(state: &mut VmState) -> VmResult<()> {
    state.halted = true;
    state.result = state.pop().unwrap_or(0);
    Ok(())
}

/// HALT_ERR: Stop execution with error
fn handle_halt_err(state: &mut VmState) -> VmResult<()> {
    let error_code = state.read_u8()?;
    state.halted = true;
    state.last_error = match error_code {
        1 => VmError::InvalidOpcode,
        2 => VmError::StackUnderflow,
        3 => VmError::StackOverflow,
        7 => VmError::IntegrityFailed,
        _ => VmError::StateCorrupt,
    };
    Err(state.last_error)
}

