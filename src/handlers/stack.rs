//! Stack Operation Handlers
//!
//! PUSH_IMM, PUSH_IMM8, PUSH_IMM16, PUSH_IMM32, PUSH_REG, POP_REG, DUP, SWAP, DROP

use crate::error::VmResult;
use crate::state::VmState;

/// PUSH_IMM: Push 64-bit immediate to stack
pub fn handle_push_imm(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u64()?;
    state.push(value)
}

/// PUSH_IMM8: Push 8-bit immediate to stack (zero-extended)
pub fn handle_push_imm8(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u8()? as u64;
    state.push(value)
}

/// PUSH_IMM16: Push 16-bit immediate to stack (zero-extended)
pub fn handle_push_imm16(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u16()? as u64;
    state.push(value)
}

/// PUSH_IMM32: Push 32-bit immediate to stack (zero-extended)
pub fn handle_push_imm32(state: &mut VmState) -> VmResult<()> {
    let value = state.read_u32()? as u64;
    state.push(value)
}

/// PUSH_REG: Push register value to stack
pub fn handle_push_reg(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.get_reg(reg_idx)?;
    state.push(value)
}

/// POP_REG: Pop stack to register
pub fn handle_pop_reg(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.pop()?;
    state.set_reg(reg_idx, value)
}

/// DUP: Duplicate top of stack
pub fn handle_dup(state: &mut VmState) -> VmResult<()> {
    let value = state.peek()?;
    state.push(value)
}

/// SWAP: Swap top two stack values
/// Stack before: [..., below, top]
/// Stack after:  [..., top, below]
pub fn handle_swap(state: &mut VmState) -> VmResult<()> {
    let top = state.pop()?;
    let below = state.pop()?;
    state.push(top)?;
    state.push(below)
}

/// DROP: Drop top of stack
pub fn handle_drop(state: &mut VmState) -> VmResult<()> {
    state.pop()?;
    Ok(())
}
