//! Native Call Handlers
//!
//! NATIVE_CALL, NATIVE_READ, NATIVE_WRITE, INPUT_LEN

use crate::error::{VmError, VmResult};
use crate::native::{NativeRegistry, MAX_NATIVE_ARGS};
use crate::state::VmState;

/// NATIVE_CALL: Call native function from registry or table
///
/// Format: NATIVE_CALL <func_id u8> <arg_count u8>
/// Pops arg_count values from stack (in reverse order), calls function, pushes result
///
/// Priority:
/// 1. If native_table is set on VmState, use that (for vm_protect macro)
/// 2. Otherwise fall back to NativeRegistry
pub fn handle_native_call(state: &mut VmState, registry: &NativeRegistry) -> VmResult<()> {
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

    // Try native table first (for vm_protect macro)
    if let Some(native_fn) = state.get_native_fn(func_id as usize) {
        let result = native_fn(&args[..arg_count]);
        return state.push(result);
    }

    // Fall back to registry
    let result = registry.call(func_id, &args[..arg_count])?;

    // Push result
    state.push(result)
}

/// NATIVE_READ: Read u64 from input buffer
pub fn handle_native_read(state: &mut VmState) -> VmResult<()> {
    let offset = state.read_u16()? as usize;
    let value = state.read_input_u64(offset)?;
    state.push(value)
}

/// NATIVE_WRITE: Write to output buffer
pub fn handle_native_write(state: &mut VmState) -> VmResult<()> {
    let _offset = state.read_u16()?;
    let value = state.pop()?;
    state.output.push(value as u8);
    Ok(())
}

/// INPUT_LEN: Push input length to stack
pub fn handle_input_len(state: &mut VmState) -> VmResult<()> {
    let len = state.input_len() as u64;
    state.push(len)
}
