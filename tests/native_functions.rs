//! Tests for native function bridging

use aegis_vm::engine::execute_with_natives;
use aegis_vm::native::{NativeRegistry, NativeRegistryBuilder, standard_ids};
use aegis_vm::build_config::opcodes::{stack, native, exec};

// ============================================================================
// Basic Native Call Tests
// ============================================================================

#[test]
fn test_native_call_no_args() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_args| 42).unwrap();

    // NATIVE_CALL func_id=0, arg_count=0, HALT
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // Call function 0 with 0 args
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_native_call_one_arg() {
    let mut registry = NativeRegistry::new();
    registry.register(1, |args| args[0] * 2).unwrap();

    // PUSH 21, NATIVE_CALL func_id=1, arg_count=1, HALT
    let code = vec![
        stack::PUSH_IMM8, 21,
        native::NATIVE_CALL, 1, 1,  // Call function 1 with 1 arg
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_native_call_two_args() {
    let mut registry = NativeRegistry::new();
    registry.register(2, |args| args[0] + args[1]).unwrap();

    // PUSH 30, PUSH 12, NATIVE_CALL func_id=2, arg_count=2, HALT
    let code = vec![
        stack::PUSH_IMM8, 30,
        stack::PUSH_IMM8, 12,
        native::NATIVE_CALL, 2, 2,
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_native_call_many_args() {
    let mut registry = NativeRegistry::new();
    registry.register(3, |args| args.iter().sum()).unwrap();

    // Sum of 1+2+3+4+5 = 15
    let code = vec![
        stack::PUSH_IMM8, 1,
        stack::PUSH_IMM8, 2,
        stack::PUSH_IMM8, 3,
        stack::PUSH_IMM8, 4,
        stack::PUSH_IMM8, 5,
        native::NATIVE_CALL, 3, 5,
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 15);
}

// ============================================================================
// Multiple Native Calls
// ============================================================================

#[test]
fn test_multiple_native_calls() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 10).unwrap();
    registry.register(1, |_| 20).unwrap();
    registry.register(2, |args| args[0] + args[1]).unwrap();

    // Call func0, call func1, add them with func2
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // Push 10
        native::NATIVE_CALL, 1, 0,  // Push 20
        native::NATIVE_CALL, 2, 2,  // Add them: 10 + 20 = 30
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 30);
}

#[test]
fn test_native_call_chain() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |args| args[0] * 2).unwrap();

    // Start with 1, multiply by 2 five times: 1*2*2*2*2*2 = 32
    let code = vec![
        stack::PUSH_IMM8, 1,
        native::NATIVE_CALL, 0, 1,  // 2
        native::NATIVE_CALL, 0, 1,  // 4
        native::NATIVE_CALL, 0, 1,  // 8
        native::NATIVE_CALL, 0, 1,  // 16
        native::NATIVE_CALL, 0, 1,  // 32
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 32);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_registry_builder() {
    let registry = NativeRegistryBuilder::new()
        .with_function(0, |_| 100)
        .with_function(1, |args| args.get(0).copied().unwrap_or(0) + 1)
        .with_hash()
        .build();

    let code1 = vec![native::NATIVE_CALL, 0, 0, exec::HALT];
    assert_eq!(execute_with_natives(&code1, &[], &registry).unwrap(), 100);

    let code2 = vec![
        stack::PUSH_IMM8, 41,
        native::NATIVE_CALL, 1, 1,
        exec::HALT,
    ];
    assert_eq!(execute_with_natives(&code2, &[], &registry).unwrap(), 42);
}

#[test]
fn test_hash_function() {
    let registry = NativeRegistryBuilder::new()
        .with_hash()
        .build();

    // Hash of a single value
    let code = vec![
        stack::PUSH_IMM8, 42,
        native::NATIVE_CALL, standard_ids::HASH_FNV1A, 1,
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    // Result should be non-zero and consistent
    assert_ne!(result, 0);

    // Call again to verify consistency
    let result2 = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, result2);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_native_function_not_found() {
    let registry = NativeRegistry::new();  // Empty registry

    let code = vec![
        native::NATIVE_CALL, 99, 0,  // Function 99 doesn't exist
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry);
    assert!(result.is_err());
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_anticheat_simulation() {
    let mut registry = NativeRegistry::new();

    // Simulate anticheat checks returning risk scores
    registry.register(standard_ids::CHECK_ROOT, |_| 0).unwrap();      // No root
    registry.register(standard_ids::CHECK_EMULATOR, |_| 0).unwrap();  // No emulator
    registry.register(standard_ids::CHECK_HOOKS, |_| 10).unwrap();    // Some hooks detected
    registry.register(standard_ids::CHECK_DEBUGGER, |_| 0).unwrap();  // No debugger

    // Sum all check results
    registry.register(10, |args| args.iter().sum()).unwrap();

    // Call all checks, sum results
    let code = vec![
        native::NATIVE_CALL, standard_ids::CHECK_ROOT, 0,
        native::NATIVE_CALL, standard_ids::CHECK_EMULATOR, 0,
        native::NATIVE_CALL, standard_ids::CHECK_HOOKS, 0,
        native::NATIVE_CALL, standard_ids::CHECK_DEBUGGER, 0,
        native::NATIVE_CALL, 10, 4,  // Sum all 4 results
        exec::HALT,
    ];

    let total_risk = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(total_risk, 10);  // Only hooks detected
}

#[test]
fn test_native_result_used_in_computation() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 50).unwrap();  // Returns 50
    registry.register(1, |args| if args[0] > 30 { 1 } else { 0 }).unwrap();

    // Get native result, then use another native to check threshold
    // native_call(0) returns 50, native_call(1, 50) returns 1 (since 50 > 30)
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // Push 50
        native::NATIVE_CALL, 1, 1,  // Call func 1 with arg 50, returns 1
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 1);  // 50 > 30, so native returns 1
}

#[test]
fn test_conditional_jump_with_natives() {
    use aegis_vm::build_config::opcodes::control;

    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 50).unwrap();  // Returns 50

    // Test: if native_call(0) > 30 then return 1 else return 0
    // Byte positions carefully calculated:
    // [0-2]   NATIVE_CALL 0, 0  -> push 50
    // [3-4]   PUSH_IMM8 30
    // [5]     CMP                -> sets flags, pushes values back
    // [6]     DROP
    // [7]     DROP
    // [8-10]  JGT offset=5       -> IP after reading = 11, jump to 11+5=16
    // [11-12] PUSH_IMM8 0        (false case)
    // [13-15] JMP offset=2       -> IP after reading = 16, jump to 16+2=18
    // [16-17] PUSH_IMM8 1        (true case)
    // [18]    HALT
    let code = vec![
        native::NATIVE_CALL, 0, 0,      // [0-2]
        stack::PUSH_IMM8, 30,           // [3-4]
        control::CMP,                   // [5]
        stack::DROP,                    // [6]
        stack::DROP,                    // [7]
        control::JGT, 0x05, 0x00,       // [8-10] offset=5 -> jump to true case
        stack::PUSH_IMM8, 0,            // [11-12] false case
        control::JMP, 0x02, 0x00,       // [13-15] offset=2 -> skip to HALT
        stack::PUSH_IMM8, 1,            // [16-17] true case
        exec::HALT,                     // [18]
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 1);  // 50 > 30, so should take true branch
}

#[test]
fn test_conditional_jump_false_branch() {
    use aegis_vm::build_config::opcodes::control;

    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 20).unwrap();  // Returns 20

    // Same structure but native returns 20, which is NOT > 30
    let code = vec![
        native::NATIVE_CALL, 0, 0,      // [0-2] push 20
        stack::PUSH_IMM8, 30,           // [3-4] push 30
        control::CMP,                   // [5]
        stack::DROP,                    // [6]
        stack::DROP,                    // [7]
        control::JGT, 0x05, 0x00,       // [8-10] 20 > 30 is false, fall through
        stack::PUSH_IMM8, 0,            // [11-12] false case -> this executes
        control::JMP, 0x02, 0x00,       // [13-15] skip true case
        stack::PUSH_IMM8, 1,            // [16-17] true case (skipped)
        exec::HALT,                     // [18]
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 0);  // 20 > 30 is false, should take false branch
}

#[test]
fn test_native_with_vm_arithmetic() {
    use aegis_vm::build_config::opcodes::arithmetic;

    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 100).unwrap();

    // (native_call() * 2) + 5 = 205
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // 100
        stack::PUSH_IMM8, 2,
        arithmetic::MUL,            // 200
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,            // 205
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 205);
}

// ============================================================================
// State Modification Tests
// ============================================================================

#[test]
fn test_native_with_closure_state() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicU64::new(0));
    let counter_clone = counter.clone();

    let mut registry = NativeRegistry::new();
    registry.register(0, move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst) + 1
    }).unwrap();

    // Call native function 3 times
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // Returns 1, counter = 1
        stack::DROP,
        native::NATIVE_CALL, 0, 0,  // Returns 2, counter = 2
        stack::DROP,
        native::NATIVE_CALL, 0, 0,  // Returns 3, counter = 3
        exec::HALT,
    ];

    let result = execute_with_natives(&code, &[], &registry).unwrap();
    assert_eq!(result, 3);
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
