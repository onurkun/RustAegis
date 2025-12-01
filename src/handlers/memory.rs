//! Memory Operation Handlers (sized loads/stores)
//!
//! LOAD8, LOAD16, LOAD32, LOAD64, STORE8, STORE16, STORE32, STORE64

use crate::error::VmResult;
use crate::state::VmState;

/// LOAD8: Load 8-bit value from input buffer (zero-extended)
pub fn handle_load8(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u8(offset)? as u64;
    state.push(value)
}

/// LOAD16: Load 16-bit value from input buffer (zero-extended)
pub fn handle_load16(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u16(offset)? as u64;
    state.push(value)
}

/// LOAD32: Load 32-bit value from input buffer (zero-extended)
pub fn handle_load32(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u32(offset)? as u64;
    state.push(value)
}

/// LOAD64: Load 64-bit value from input buffer
pub fn handle_load64(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u64(offset)?;
    state.push(value)
}

/// STORE8: Store 8-bit value to output buffer
pub fn handle_store8(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u8(offset, value as u8)
}

/// STORE16: Store 16-bit value to output buffer
pub fn handle_store16(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u16(offset, value as u16)
}

/// STORE32: Store 32-bit value to output buffer
pub fn handle_store32(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u32(offset, value as u32)
}

/// STORE64: Store 64-bit value to output buffer
pub fn handle_store64(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.pop()?;
    state.write_output_u64(offset, value)
}
