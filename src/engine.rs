//! VM Execution Engine
//!
//! Main dispatch loop using indirect threading (function pointer table)
//! This eliminates the switch-case pattern visible in binary analysis.

use crate::error::{VmError, VmResult};
use crate::native::NativeRegistry;
use crate::state::{VmState, MAX_INSTRUCTIONS};

// Indirect dispatch via function pointer table
use crate::handlers::dispatch::dispatch_indirect;

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

/// Execute bytecode with native function table (array of function pointers)
/// This is used by the vm_protect macro for compiled native calls
pub fn execute_with_native_table(code: &[u8], input: &[u8], native_table: &[fn(&[u64]) -> u64]) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    state.set_native_table(native_table);
    run_with_native_table(&mut state)?;
    Ok(state.result)
}

/// Main execution loop with native function table support
pub fn run_with_native_table(state: &mut VmState) -> VmResult<()> {
    let empty_registry = NativeRegistry::new();
    while !state.halted && state.ip < state.code.len() {
        // Instruction count limit
        state.instruction_count += 1;
        if state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        // Fetch opcode
        let opcode = state.read_u8()?;

        // Indirect dispatch via function pointer table
        dispatch_indirect(state, opcode, &empty_registry)?;
    }

    Ok(())
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
/// Uses indirect threading (function pointer table) for opcode dispatch
pub fn run_with_natives(state: &mut VmState, registry: &NativeRegistry) -> VmResult<()> {
    while !state.halted && state.ip < state.code.len() {
        // Instruction count limit
        state.instruction_count += 1;
        if state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        // Fetch opcode
        let opcode = state.read_u8()?;

        // Indirect dispatch via function pointer table
        // This replaces the switch-case pattern for better obfuscation
        dispatch_indirect(state, opcode, registry)?;
    }

    Ok(())
}

