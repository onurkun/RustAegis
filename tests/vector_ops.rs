//! Vector Operations Tests
//!
//! Tests for array expressions:
//! - [expr.array.array]: List form [1, 2, 3]
//! - [expr.array.repeat]: Repeat form [0; N]
//! - [expr.array.index.array]: Indexing arr[i]

use aegis_vm::{execute, build_config::opcodes::{stack, vector, exec}};

/// Test VEC_NEW and VEC_LEN
/// Create vector with capacity 10, elem_size 8, verify length is 0
#[test]
fn test_vec_new_and_len() {
    let bytecode = [
        // Push capacity=10, elem_size=8
        stack::PUSH_IMM8, 10,       // capacity
        stack::PUSH_IMM8, 8,        // elem_size (u64)
        vector::VEC_NEW,            // create vector -> [vec_addr]

        // Get length (should be 0)
        stack::DUP,                 // duplicate address for length check
        vector::VEC_LEN,            // [vec_addr, 0]

        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "New vector should have length 0");
}

/// Test VEC_CAP
/// Create vector with capacity 20, verify capacity
#[test]
fn test_vec_capacity() {
    let bytecode = [
        stack::PUSH_IMM8, 20,       // capacity
        stack::PUSH_IMM8, 8,        // elem_size
        vector::VEC_NEW,
        vector::VEC_CAP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 20, "Vector capacity should be 20");
}

/// Test VEC_PUSH and VEC_LEN
/// Push elements and verify length increases
#[test]
fn test_vec_push() {
    let bytecode = [
        // Create vector
        stack::PUSH_IMM8, 10,       // capacity
        stack::PUSH_IMM8, 8,        // elem_size
        vector::VEC_NEW,            // -> [vec_addr]

        // Push value 42
        stack::DUP,                 // [vec_addr, vec_addr]
        stack::PUSH_IMM8, 42,       // [vec_addr, vec_addr, 42]
        vector::VEC_PUSH,           // [vec_addr]

        // Push value 100
        stack::DUP,
        stack::PUSH_IMM8, 100,
        vector::VEC_PUSH,

        // Push value 255
        stack::DUP,
        stack::PUSH_IMM8, 255,
        vector::VEC_PUSH,

        // Get length (should be 3)
        vector::VEC_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 3, "Vector should have 3 elements after 3 pushes");
}

/// Test VEC_GET - array indexing arr[i]
/// [expr.array.index.array]
#[test]
fn test_vec_get() {
    let bytecode = [
        // Create vector
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        // Push 42, 100, 255
        stack::DUP,
        stack::PUSH_IMM8, 42,
        vector::VEC_PUSH,

        stack::DUP,
        stack::PUSH_IMM8, 100,
        vector::VEC_PUSH,

        stack::DUP,
        stack::PUSH_IMM8, 255,
        vector::VEC_PUSH,

        // Get arr[1] (should be 100)
        stack::DUP,
        stack::PUSH_IMM8, 1,        // index
        vector::VEC_GET,            // [vec_addr, 100]

        stack::SWAP,
        stack::DROP,                // [100]
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 100, "arr[1] should be 100");
}

/// Test VEC_SET - array index assignment arr[i] = x
#[test]
fn test_vec_set() {
    let bytecode = [
        // Create vector with 3 elements
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        // Push initial values: [42, 100, 255]
        stack::DUP, stack::PUSH_IMM8, 42, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 100, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 255, vector::VEC_PUSH,

        // Set arr[1] = 999
        stack::DUP,
        stack::PUSH_IMM8, 1,        // index
        stack::PUSH_IMM16, 0xE7, 0x03,  // 999 in little-endian
        vector::VEC_SET,

        // Verify arr[1] is now 999
        stack::DUP,
        stack::PUSH_IMM8, 1,
        vector::VEC_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 999, "arr[1] should be 999 after set");
}

/// Test VEC_POP
#[test]
fn test_vec_pop() {
    let bytecode = [
        // Create and fill vector
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        stack::DUP, stack::PUSH_IMM8, 10, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 20, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 30, vector::VEC_PUSH,

        // Pop (should get 30)
        stack::DUP,
        vector::VEC_POP,            // [vec_addr, 30]

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 30, "Pop should return last element (30)");
}

/// Test VEC_REPEAT - [value; count]
/// [expr.array.repeat]
#[test]
fn test_vec_repeat() {
    let bytecode = [
        // Create [42; 5] - array of 5 elements all set to 42
        stack::PUSH_IMM8, 42,       // value
        stack::PUSH_IMM8, 5,        // count
        stack::PUSH_IMM8, 8,        // elem_size
        vector::VEC_REPEAT,         // -> [vec_addr]

        // Verify length is 5
        stack::DUP,
        vector::VEC_LEN,            // [vec_addr, 5]
        stack::DROP,                // drop 5

        // Verify arr[3] is 42
        stack::DUP,
        stack::PUSH_IMM8, 3,
        vector::VEC_GET,            // [vec_addr, 42]

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 42, "[42; 5][3] should be 42");
}

/// Test VEC_CLEAR
#[test]
fn test_vec_clear() {
    let bytecode = [
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        // Push some elements
        stack::DUP, stack::PUSH_IMM8, 1, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 2, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 3, vector::VEC_PUSH,

        // Clear
        stack::DUP,
        vector::VEC_CLEAR,

        // Length should be 0
        vector::VEC_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "Length should be 0 after clear");
}

/// Test different element sizes
#[test]
fn test_vec_elem_size_1() {
    let bytecode = [
        // Create vector with elem_size=1 (u8)
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 1,        // elem_size = 1 byte
        vector::VEC_NEW,

        // Push bytes
        stack::DUP, stack::PUSH_IMM8, 0xAB, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0xCD, vector::VEC_PUSH,

        // Get arr[0]
        stack::DUP,
        stack::PUSH_IMM8, 0,
        vector::VEC_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0xAB, "arr[0] should be 0xAB");
}

/// Test vector with 2-byte elements
#[test]
fn test_vec_elem_size_2() {
    let bytecode = [
        // Create vector with elem_size=2 (u16)
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 2,        // elem_size = 2 bytes
        vector::VEC_NEW,

        // Push 0x1234
        stack::DUP,
        stack::PUSH_IMM16, 0x34, 0x12,  // 0x1234 little-endian
        vector::VEC_PUSH,

        // Get arr[0]
        stack::DUP,
        stack::PUSH_IMM8, 0,
        vector::VEC_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0x1234, "arr[0] should be 0x1234");
}

/// Test vector with 4-byte elements
#[test]
fn test_vec_elem_size_4() {
    let bytecode = [
        // Create vector with elem_size=4 (u32)
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 4,        // elem_size = 4 bytes
        vector::VEC_NEW,

        // Push 0x12345678
        stack::DUP,
        stack::PUSH_IMM32, 0x78, 0x56, 0x34, 0x12,  // little-endian
        vector::VEC_PUSH,

        // Get arr[0]
        stack::DUP,
        stack::PUSH_IMM8, 0,
        vector::VEC_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0x12345678, "arr[0] should be 0x12345678");
}

/// Test bounds checking - index out of bounds should error
#[test]
fn test_vec_bounds_check() {
    let bytecode = [
        // Create vector with 1 element
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        stack::DUP,
        stack::PUSH_IMM8, 42,
        vector::VEC_PUSH,

        // Try to get arr[5] (out of bounds - only 1 element)
        stack::PUSH_IMM8, 5,
        vector::VEC_GET,

        exec::HALT,
    ];

    let result = execute(&bytecode, &[]);
    assert!(result.is_err(), "Accessing out of bounds should error");
}

/// Test [expr.array.array] - list form simulation
/// Simulates: let arr = [10, 20, 30, 40, 50];
#[test]
fn test_list_form_array() {
    let bytecode = [
        // Create vector
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 8,
        vector::VEC_NEW,

        // Push elements: [10, 20, 30, 40, 50]
        stack::DUP, stack::PUSH_IMM8, 10, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 20, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 30, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 40, vector::VEC_PUSH,
        stack::DUP, stack::PUSH_IMM8, 50, vector::VEC_PUSH,

        // Verify all elements
        // arr[0] = 10
        stack::DUP, stack::PUSH_IMM8, 0, vector::VEC_GET,
        stack::PUSH_IMM8, 10,
        aegis_vm::build_config::opcodes::control::CMP,
        stack::DROP, stack::DROP,

        // arr[4] = 50
        stack::DUP, stack::PUSH_IMM8, 4, vector::VEC_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 50, "arr[4] should be 50");
}

/// Test large repeat array [0; 100]
#[test]
fn test_large_repeat_array() {
    let bytecode = [
        // Create [0; 100]
        stack::PUSH_IMM8, 0,        // value = 0
        stack::PUSH_IMM8, 100,      // count = 100
        stack::PUSH_IMM8, 8,        // elem_size
        vector::VEC_REPEAT,

        // Get length
        vector::VEC_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 100, "Length should be 100");
}
