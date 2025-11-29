//! Tests specifically for ValueCryptor (VMProtect-style constant encryption)
//!
//! ValueCryptor encrypts constants at compile-time and emits decryption
//! chains in the bytecode. These tests verify the encryption works correctly
//! for various constant types and edge cases.

use aegis_vm_macro::vm_protect;

// =============================================================================
// Basic constant tests
// =============================================================================

/// Test small constants (fit in u8)
#[vm_protect(level = "paranoid")]
fn small_constant() -> u64 {
    42
}

/// Test medium constants (fit in u16)
#[vm_protect(level = "paranoid")]
fn medium_constant() -> u64 {
    0xBEEF
}

/// Test large constants (fit in u32)
#[vm_protect(level = "paranoid")]
fn large_constant() -> u64 {
    0xDEADBEEF
}

/// Test very large constants (full u64)
#[vm_protect(level = "paranoid")]
fn very_large_constant() -> u64 {
    0xDEADBEEF_CAFEBABE
}

/// Test zero (edge case)
#[vm_protect(level = "paranoid")]
fn zero_constant() -> u64 {
    0
}

/// Test one (edge case)
#[vm_protect(level = "paranoid")]
fn one_constant() -> u64 {
    1
}

/// Test max u64 (edge case)
#[vm_protect(level = "paranoid")]
fn max_constant() -> u64 {
    0xFFFFFFFFFFFFFFFF // u64::MAX
}

/// Test powers of two
#[vm_protect(level = "paranoid")]
fn power_of_two() -> u64 {
    0x8000_0000_0000_0000 // 2^63
}

#[test]
fn test_small_constant() {
    assert_eq!(small_constant(), 42);
}

#[test]
fn test_medium_constant() {
    assert_eq!(medium_constant(), 0xBEEF);
}

#[test]
fn test_large_constant() {
    assert_eq!(large_constant(), 0xDEADBEEF);
}

#[test]
fn test_very_large_constant() {
    assert_eq!(very_large_constant(), 0xDEADBEEF_CAFEBABE);
}

#[test]
fn test_zero_constant() {
    assert_eq!(zero_constant(), 0);
}

#[test]
fn test_one_constant() {
    assert_eq!(one_constant(), 1);
}

#[test]
fn test_max_constant() {
    assert_eq!(max_constant(), u64::MAX);
}

#[test]
fn test_power_of_two() {
    assert_eq!(power_of_two(), 0x8000_0000_0000_0000);
}

// =============================================================================
// Constants in expressions
// =============================================================================

/// Multiple constants in same function
#[vm_protect(level = "paranoid")]
fn multiple_constants(x: u64) -> u64 {
    let a = x + 0x1111;
    let b = a ^ 0x2222;
    let c = b - 0x3333;
    c & 0xFFFF
}

/// Constant in if condition
#[vm_protect(level = "paranoid")]
fn constant_in_condition(x: u64) -> u64 {
    if x > 100 {
        0xAAAA
    } else {
        0xBBBB
    }
}

/// Nested constant operations
#[vm_protect(level = "paranoid")]
fn nested_constants(x: u64) -> u64 {
    ((x + 0x1000) ^ 0x2000) & 0xFF00
}

/// Secret key pattern (common use case)
#[vm_protect(level = "paranoid")]
fn secret_key_check(input: u64) -> u64 {
    let key = 0x5ECE7_0123_4567;
    if input == key {
        1
    } else {
        0
    }
}

#[test]
fn test_multiple_constants() {
    // (10 + 0x1111) ^ 0x2222 - 0x3333 & 0xFFFF
    let result = multiple_constants(10);
    let expected = ((10u64 + 0x1111) ^ 0x2222).wrapping_sub(0x3333) & 0xFFFF;
    assert_eq!(result, expected);
}

#[test]
fn test_constant_in_condition() {
    assert_eq!(constant_in_condition(50), 0xBBBB);   // x <= 100
    assert_eq!(constant_in_condition(101), 0xAAAA); // x > 100
    assert_eq!(constant_in_condition(100), 0xBBBB); // x == 100 (not greater)
}

#[test]
fn test_nested_constants() {
    let result = nested_constants(0x500);
    let expected = ((0x500u64 + 0x1000) ^ 0x2000) & 0xFF00;
    assert_eq!(result, expected);
}

#[test]
fn test_secret_key_check() {
    let correct_key = 0x5ECE7_0123_4567;
    assert_eq!(secret_key_check(correct_key), 1);
    assert_eq!(secret_key_check(0), 0);
    assert_eq!(secret_key_check(correct_key + 1), 0);
    assert_eq!(secret_key_check(correct_key - 1), 0);
}

// =============================================================================
// Comparison with non-paranoid (verify same results)
// =============================================================================

#[vm_protect(level = "debug")]
fn debug_computation(x: u64) -> u64 {
    (x + 0xDEAD) ^ 0xBEEF
}

#[vm_protect]
fn standard_computation(x: u64) -> u64 {
    (x + 0xDEAD) ^ 0xBEEF
}

#[vm_protect(level = "paranoid")]
fn paranoid_computation(x: u64) -> u64 {
    (x + 0xDEAD) ^ 0xBEEF
}

#[test]
fn test_all_levels_same_result() {
    for x in [0u64, 1, 42, 100, 0xFF, 0xFFFF, 0xFFFFFFFF, u64::MAX] {
        let debug_result = debug_computation(x);
        let standard_result = standard_computation(x);
        let paranoid_result = paranoid_computation(x);

        assert_eq!(
            debug_result, standard_result,
            "debug vs standard mismatch for x={}", x
        );
        assert_eq!(
            standard_result, paranoid_result,
            "standard vs paranoid mismatch for x={}", x
        );
    }
}

// =============================================================================
// Stress test with many constants
// =============================================================================

#[vm_protect(level = "paranoid")]
fn many_constants_stress(x: u64) -> u64 {
    let a = x + 0x1111111111111111;
    let b = a ^ 0x2222222222222222;
    let c = b - 0x3333333333333333;
    let d = c & 0x4444444444444444;
    let e = d | 0x5555555555555555;
    let f = e + 0x6666666666666666;
    let g = f ^ 0x7777777777777777;
    g - 0x8888888888888888
}

#[test]
fn test_many_constants_stress() {
    // Compute expected result
    let x = 0x1234567890ABCDEFu64;
    let a = x.wrapping_add(0x1111111111111111);
    let b = a ^ 0x2222222222222222;
    let c = b.wrapping_sub(0x3333333333333333);
    let d = c & 0x4444444444444444;
    let e = d | 0x5555555555555555;
    let f = e.wrapping_add(0x6666666666666666);
    let g = f ^ 0x7777777777777777;
    let expected = g.wrapping_sub(0x8888888888888888);

    assert_eq!(many_constants_stress(x), expected);
}

// =============================================================================
// Bit manipulation with constants
// =============================================================================

#[vm_protect(level = "paranoid")]
fn bit_set(x: u64, bit: u64) -> u64 {
    x | (1 << bit)
}

#[vm_protect(level = "paranoid")]
fn bit_clear(x: u64, bit: u64) -> u64 {
    x & !(1 << bit)
}

#[vm_protect(level = "paranoid")]
fn bit_toggle(x: u64, bit: u64) -> u64 {
    x ^ (1 << bit)
}

#[test]
fn test_bit_operations() {
    // Test bit_set
    assert_eq!(bit_set(0, 0), 1);
    assert_eq!(bit_set(0, 3), 8);
    assert_eq!(bit_set(0xFF, 8), 0x1FF);

    // Test bit_clear
    assert_eq!(bit_clear(0xFF, 0), 0xFE);
    assert_eq!(bit_clear(0xFF, 7), 0x7F);

    // Test bit_toggle
    assert_eq!(bit_toggle(0, 0), 1);
    assert_eq!(bit_toggle(1, 0), 0);
    assert_eq!(bit_toggle(0xF0, 4), 0xE0);
}
