//! Register Operation Handlers
//!
//! MOV_IMM, MOV_REG, LOAD_MEM, STORE_MEM

use crate::error::VmResult;
use crate::state::VmState;

/// MOV_IMM: Load immediate to register
pub fn handle_mov_imm(state: &mut VmState) -> VmResult<()> {
    let reg_idx = state.read_u8()?;
    let value = state.read_u64()?;
    state.set_reg(reg_idx, value)
}

/// MOV_REG: Copy register to register
pub fn handle_mov_reg(state: &mut VmState) -> VmResult<()> {
    let dst = state.read_u8()?;
    let src = state.read_u8()?;
    let value = state.get_reg(src)?;
    state.set_reg(dst, value)
}

/// LOAD_MEM: Load from input buffer using register as offset
pub fn handle_load_mem(state: &mut VmState) -> VmResult<()> {
    let dst_reg = state.read_u8()?;
    let addr_reg = state.read_u8()?;
    let offset = state.get_reg(addr_reg)? as usize;
    let value = state.read_input_u64(offset)?;
    state.set_reg(dst_reg, value)
}

/// STORE_MEM: Store to output buffer
pub fn handle_store_mem(state: &mut VmState) -> VmResult<()> {
    let _addr_reg = state.read_u8()?;
    let src_reg = state.read_u8()?;
    let value = state.get_reg(src_reg)?;
    state.output.extend_from_slice(&value.to_le_bytes());
    Ok(())
}
