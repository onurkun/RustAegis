//! Async VM Execution Engine
//!
//! Async version of the VM execution loop. The async/await syntax
//! causes Rust to generate a state machine, complicating reverse engineering.

use crate::error::{VmError, VmResult};
use crate::native::NativeRegistry;
use crate::state::{VmState, MAX_INSTRUCTIONS};
use crate::handlers::dispatch::dispatch_indirect;

use super::executor::block_on;
use super::yielder::YieldNow;

/// Execute bytecode asynchronously (blocking wrapper)
///
/// This is the main entry point for async VM execution.
/// Internally uses async/await for state machine obfuscation.
#[inline]
pub fn execute_async(code: &[u8], input: &[u8]) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    block_on(run_async(&mut state))?;
    Ok(state.result)
}

/// Execute bytecode with native function registry (async version)
#[inline]
pub fn execute_async_with_natives(
    code: &[u8],
    input: &[u8],
    registry: &NativeRegistry,
) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    block_on(run_async_with_natives(&mut state, registry))?;
    Ok(state.result)
}

/// Execute bytecode with native function table (async version)
/// This is used by the vm_protect macro when async mode is enabled
#[inline]
pub fn execute_async_with_native_table(
    code: &[u8],
    input: &[u8],
    native_table: &[fn(&[u64]) -> u64],
) -> VmResult<u64> {
    let mut state = VmState::new(code, input);
    state.set_native_table(native_table);
    block_on(run_async_with_native_table(&mut state))?;
    Ok(state.result)
}

/// Async execution loop with native function table support
pub async fn run_async_with_native_table(state: &mut VmState<'_>) -> VmResult<()> {
    let empty_registry = NativeRegistry::new();
    let yield_mask = state.get_yield_mask();

    while !state.halted && state.ip < state.code.len() {
        state.instruction_count += 1;
        if state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        let opcode = state.read_u8()?;
        dispatch_indirect(state, opcode, &empty_registry)?;

        if (state.instruction_count & yield_mask) == 0 {
            YieldNow::new().await;
        }
    }

    Ok(())
}

/// Async execution loop (without native functions)
#[inline]
pub async fn run_async(state: &mut VmState<'_>) -> VmResult<()> {
    let empty_registry = NativeRegistry::new();
    run_async_with_natives(state, &empty_registry).await
}

/// Async execution loop with native function support
///
/// This is where the magic happens: The async/await syntax causes
/// Rust to transform this function into a state machine enum.
/// Each `await` point becomes a state transition, making the
/// control flow graph much harder to analyze.
pub async fn run_async_with_natives(
    state: &mut VmState<'_>,
    registry: &NativeRegistry,
) -> VmResult<()> {
    // Get yield mask from state (polymorphic per-build)
    // Lower bits = more frequent yields = more state transitions
    // Default: 0xFF (yield every 256 instructions)
    let yield_mask = state.get_yield_mask();

    while !state.halted && state.ip < state.code.len() {
        // Instruction count limit (DoS protection)
        state.instruction_count += 1;
        if state.instruction_count > MAX_INSTRUCTIONS {
            return Err(VmError::MaxInstructionsExceeded);
        }

        // Fetch opcode
        let opcode = state.read_u8()?;

        // Indirect dispatch (same as sync version)
        dispatch_indirect(state, opcode, registry)?;

        // Anti-Analysis: Controlled yield
        // Inject state machine transitions at regular intervals
        // This breaks up the control flow graph
        if (state.instruction_count & yield_mask) == 0 {
            YieldNow::new().await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_config::opcodes::{stack, arithmetic, exec};

    #[test]
    fn test_async_execute_simple() {
        // PUSH 42, HALT
        let code = vec![
            stack::PUSH_IMM8, 42,
            exec::HALT,
        ];

        let result = execute_async(&code, &[]).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_execute_arithmetic() {
        // PUSH 10, PUSH 5, ADD, HALT
        let code = vec![
            stack::PUSH_IMM8, 10,
            stack::PUSH_IMM8, 5,
            arithmetic::ADD,
            exec::HALT,
        ];

        let result = execute_async(&code, &[]).unwrap();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_async_matches_sync() {
        use crate::engine::execute;

        // More complex bytecode: (100 - 50) * 2 = 100
        let code = vec![
            stack::PUSH_IMM8, 100,
            stack::PUSH_IMM8, 50,
            arithmetic::SUB,
            stack::PUSH_IMM8, 2,
            arithmetic::MUL,
            exec::HALT,
        ];

        let sync_result = execute(&code, &[]).unwrap();
        let async_result = execute_async(&code, &[]).unwrap();

        assert_eq!(sync_result, async_result);
    }

    #[test]
    fn test_async_with_many_instructions() {
        // Test that yields happen correctly with many instructions
        // PUSH 0, then 100x (PUSH 1, ADD), then HALT
        let mut code = vec![stack::PUSH_IMM8, 0];
        for _ in 0..100 {
            code.push(stack::PUSH_IMM8);
            code.push(1);
            code.push(arithmetic::ADD);
        }
        code.push(exec::HALT);

        let result = execute_async(&code, &[]).unwrap();
        assert_eq!(result, 100);
    }
}
