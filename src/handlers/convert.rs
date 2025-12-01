//! Type Conversion Handlers
//!
//! SEXT8, SEXT16, SEXT32, TRUNC8, TRUNC16, TRUNC32

use crate::error::VmResult;
use crate::state::VmState;

/// SEXT8: Sign-extend 8-bit value to 64-bit
pub fn handle_sext8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i8) as i64 as u64;
    state.push(result)
}

/// SEXT16: Sign-extend 16-bit value to 64-bit
pub fn handle_sext16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i16) as i64 as u64;
    state.push(result)
}

/// SEXT32: Sign-extend 32-bit value to 64-bit
pub fn handle_sext32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = (value as i32) as i64 as u64;
    state.push(result)
}

/// TRUNC8: Truncate to 8-bit (mask with 0xFF)
pub fn handle_trunc8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFF;
    state.push(result)
}

/// TRUNC16: Truncate to 16-bit (mask with 0xFFFF)
pub fn handle_trunc16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFFFF;
    state.push(result)
}

/// TRUNC32: Truncate to 32-bit (mask with 0xFFFFFFFF)
pub fn handle_trunc32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let result = value & 0xFFFFFFFF;
    state.push(result)
}
