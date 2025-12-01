//! String Operations Tests

use aegis_vm::{execute, build_config::opcodes::{stack, string, exec}};

/// Test STR_NEW and STR_LEN
#[test]
fn test_str_new_and_len() {
    let bytecode = [
        // Create string with capacity 100
        stack::PUSH_IMM8, 100,
        string::STR_NEW,

        // Length should be 0
        string::STR_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "New string should have length 0");
}

/// Test STR_PUSH and STR_LEN
#[test]
fn test_str_push() {
    let bytecode = [
        stack::PUSH_IMM8, 100,
        string::STR_NEW,

        // Push 'H' (0x48)
        stack::DUP,
        stack::PUSH_IMM8, 0x48,
        string::STR_PUSH,

        // Push 'i' (0x69)
        stack::DUP,
        stack::PUSH_IMM8, 0x69,
        string::STR_PUSH,

        // Length should be 2
        string::STR_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 2, "String 'Hi' should have length 2");
}

/// Test STR_GET - get byte at index
#[test]
fn test_str_get() {
    let bytecode = [
        stack::PUSH_IMM8, 100,
        string::STR_NEW,

        // Build "abc" = [0x61, 0x62, 0x63]
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,  // 'a'
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,  // 'b'
        stack::DUP, stack::PUSH_IMM8, 0x63, string::STR_PUSH,  // 'c'

        // Get str[1] = 'b' = 0x62
        stack::DUP,
        stack::PUSH_IMM8, 1,
        string::STR_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0x62, "str[1] should be 'b' (0x62)");
}

/// Test STR_SET
#[test]
fn test_str_set() {
    let bytecode = [
        stack::PUSH_IMM8, 100,
        string::STR_NEW,

        // Build "abc"
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x63, string::STR_PUSH,

        // Set str[1] = 'X' (0x58)
        stack::DUP,
        stack::PUSH_IMM8, 1,
        stack::PUSH_IMM8, 0x58,
        string::STR_SET,

        // Verify str[1] = 'X'
        stack::DUP,
        stack::PUSH_IMM8, 1,
        string::STR_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0x58, "str[1] should be 'X' (0x58) after set");
}

/// Test STR_EQ - string equality
#[test]
fn test_str_eq_equal() {
    let bytecode = [
        // Create string 1: "ab"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Create string 2: "ab"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Compare: should be equal (1)
        string::STR_EQ,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 1, "Equal strings should return 1");
}

/// Test STR_EQ - string inequality
#[test]
fn test_str_eq_not_equal() {
    let bytecode = [
        // Create string 1: "ab"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Create string 2: "ac"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x63, string::STR_PUSH,  // 'c' not 'b'

        // Compare: should not be equal (0)
        string::STR_EQ,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "Different strings should return 0");
}

/// Test STR_EQ - different lengths
#[test]
fn test_str_eq_different_lengths() {
    let bytecode = [
        // Create string 1: "ab"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Create string 2: "abc" (longer)
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x63, string::STR_PUSH,

        string::STR_EQ,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "Different length strings should return 0");
}

/// Test STR_CMP - less than
#[test]
fn test_str_cmp_less() {
    let bytecode = [
        // Create "aa"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,

        // Create "ab"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Compare "aa" < "ab" -> should return -1 (u64::MAX)
        string::STR_CMP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, u64::MAX, "\"aa\" < \"ab\" should return -1 (u64::MAX)");
}

/// Test STR_CMP - greater than
#[test]
fn test_str_cmp_greater() {
    let bytecode = [
        // Create "b"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x62, string::STR_PUSH,

        // Create "a"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, 0x61, string::STR_PUSH,

        // Compare "b" > "a" -> should return 1
        string::STR_CMP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 1, "\"b\" > \"a\" should return 1");
}

/// Test STR_CMP - equal
#[test]
fn test_str_cmp_equal() {
    let bytecode = [
        // Create "test"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'e', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b's', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,

        // Create "test"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'e', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b's', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,

        string::STR_CMP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 0, "Equal strings should return 0");
}

/// Test STR_HASH
#[test]
fn test_str_hash() {
    let bytecode = [
        // Create "test"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'e', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b's', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b't', string::STR_PUSH,

        string::STR_HASH,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    // Hash should be non-zero and consistent
    assert_ne!(result, 0, "Hash should not be 0");
}

/// Test STR_HASH - same string should produce same hash
#[test]
fn test_str_hash_consistency() {
    // Create first hash
    let bytecode1 = [
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'a', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'b', string::STR_PUSH,
        string::STR_HASH,
        exec::HALT,
    ];

    // Create second hash (same string)
    let bytecode2 = [
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'a', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'b', string::STR_PUSH,
        string::STR_HASH,
        exec::HALT,
    ];

    let hash1 = execute(&bytecode1, &[]).unwrap();
    let hash2 = execute(&bytecode2, &[]).unwrap();
    assert_eq!(hash1, hash2, "Same string should produce same hash");
}

/// Test STR_CONCAT
#[test]
fn test_str_concat() {
    let bytecode = [
        // Create "Hello"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'H', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'e', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'l', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'l', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'o', string::STR_PUSH,

        // Create "World"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'W', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'o', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'r', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'l', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'd', string::STR_PUSH,

        // Concat -> "HelloWorld"
        string::STR_CONCAT,

        // Length should be 10
        string::STR_LEN,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 10, "\"HelloWorld\" should have length 10");
}

/// Test STR_CONCAT - verify concatenated content
#[test]
fn test_str_concat_content() {
    let bytecode = [
        // Create "AB"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'A', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'B', string::STR_PUSH,

        // Create "CD"
        stack::PUSH_IMM8, 10,
        string::STR_NEW,
        stack::DUP, stack::PUSH_IMM8, b'C', string::STR_PUSH,
        stack::DUP, stack::PUSH_IMM8, b'D', string::STR_PUSH,

        // Concat -> "ABCD"
        string::STR_CONCAT,

        // Get str[2] = 'C'
        stack::DUP,
        stack::PUSH_IMM8, 2,
        string::STR_GET,

        stack::SWAP,
        stack::DROP,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, b'C' as u64, "\"ABCD\"[2] should be 'C'");
}

/// Test empty string operations
#[test]
fn test_empty_string() {
    let bytecode = [
        // Create empty string
        stack::PUSH_IMM8, 10,
        string::STR_NEW,

        // Create another empty string
        stack::PUSH_IMM8, 10,
        string::STR_NEW,

        // They should be equal
        string::STR_EQ,
        exec::HALT,
    ];

    let result = execute(&bytecode, &[]).unwrap();
    assert_eq!(result, 1, "Empty strings should be equal");
}

/// Test bounds check for string
#[test]
fn test_str_bounds_check() {
    let bytecode = [
        stack::PUSH_IMM8, 10,
        string::STR_NEW,

        // Push one byte
        stack::DUP, stack::PUSH_IMM8, b'a', string::STR_PUSH,

        // Try to get str[5] (out of bounds)
        stack::PUSH_IMM8, 5,
        string::STR_GET,

        exec::HALT,
    ];

    let result = execute(&bytecode, &[]);
    assert!(result.is_err(), "Accessing out of bounds should error");
}
