//! Execution Control Handlers
//!
//! HALT, HALT_ERR

use crate::error::{VmError, VmResult};
use crate::state::VmState;

/// HALT: Stop execution, result is top of stack
pub fn handle_halt(state: &mut VmState) -> VmResult<()> {
    state.halted = true;
    state.result = state.pop().unwrap_or(0);
    Ok(())
}

/// HALT_ERR: Stop execution with error
pub fn handle_halt_err(state: &mut VmState) -> VmResult<()> {
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
