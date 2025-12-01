//! Special Operation Handlers (Anti-analysis)
//!
//! NOP_N, OPAQUE_TRUE, OPAQUE_FALSE, HASH_CHECK, TIMING_CHECK

use crate::error::{VmError, VmResult};
use crate::state::VmState;

/// NOP_N: Skip N bytes
pub fn handle_nop_n(state: &mut VmState) -> VmResult<()> {
    let count = state.read_u8()?;
    state.ip += count as usize;
    if state.ip > state.code.len() {
        return Err(VmError::InvalidJumpTarget);
    }
    Ok(())
}

/// OPAQUE_TRUE: Opaque predicate that always evaluates to true
/// Pushes 1 to stack
pub fn handle_opaque_true(state: &mut VmState) -> VmResult<()> {
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
pub fn handle_opaque_false(state: &mut VmState) -> VmResult<()> {
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
pub fn handle_hash_check(state: &mut VmState) -> VmResult<()> {
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
pub fn handle_timing_check(state: &mut VmState) -> VmResult<()> {
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
