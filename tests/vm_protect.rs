//! Tests for #[vm_protect] proc-macro
//!
//! These tests use `level = "debug"` for plaintext bytecode to avoid
//! seed matching issues between compile-time and runtime.

use aegis_vm_macro::vm_protect;

// ============================================================================
// Debug Mode Tests (plaintext bytecode)
// ============================================================================

#[vm_protect(level = "debug")]
fn add_42(x: u64) -> u64 {
    x + 42
}

#[vm_protect(level = "debug")]
fn multiply(a: u64, b: u64) -> u64 {
    a * b
}

#[vm_protect(level = "debug")]
fn xor_secret(x: u64) -> u64 {
    x ^ 0xDEADBEEF
}

#[vm_protect(level = "debug")]
fn complex_arithmetic(x: u64) -> u64 {
    ((x + 10) * 2) ^ 0xFF
}

#[vm_protect(level = "debug")]
fn bitwise_ops(x: u64) -> u64 {
    (x & 0xFF) | 0x100
}

#[vm_protect(level = "debug")]
fn shift_ops(x: u64) -> u64 {
    (x << 4) | (x >> 4)
}

#[vm_protect(level = "debug")]
fn constant_only() -> u64 {
    42
}

#[vm_protect(level = "debug")]
fn large_constant() -> u64 {
    0xDEADBEEFCAFEBABE
}

#[vm_protect(level = "debug")]
fn bool_return(x: u64) -> bool {
    x > 100
}

#[vm_protect(level = "debug")]
fn negation(x: u64) -> u64 {
    !x
}

#[test]
fn test_add_42() {
    assert_eq!(add_42(0), 42);
    assert_eq!(add_42(8), 50);
    assert_eq!(add_42(100), 142);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(6, 7), 42);
    assert_eq!(multiply(0, 100), 0);
    assert_eq!(multiply(10, 10), 100);
}

#[test]
fn test_xor_secret() {
    let secret = 0xDEADBEEF;
    assert_eq!(xor_secret(0), secret);
    assert_eq!(xor_secret(secret), 0);
    assert_eq!(xor_secret(xor_secret(123)), 123);
}

#[test]
fn test_complex_arithmetic() {
    // ((0 + 10) * 2) ^ 0xFF = 20 ^ 0xFF = 235
    assert_eq!(complex_arithmetic(0), 235);
    // ((5 + 10) * 2) ^ 0xFF = 30 ^ 0xFF = 225
    assert_eq!(complex_arithmetic(5), 225);
}

#[test]
fn test_bitwise_ops() {
    // (0x1234 & 0xFF) | 0x100 = 0x34 | 0x100 = 0x134
    assert_eq!(bitwise_ops(0x1234), 0x134);
    // (0 & 0xFF) | 0x100 = 0x100
    assert_eq!(bitwise_ops(0), 0x100);
}

#[test]
fn test_shift_ops() {
    // (0xAB << 4) | (0xAB >> 4) = 0xAB0 | 0xA = 0xABA
    assert_eq!(shift_ops(0xAB), 0xABA);
}

#[test]
fn test_constant_only() {
    assert_eq!(constant_only(), 42);
}

#[test]
fn test_large_constant() {
    assert_eq!(large_constant(), 0xDEADBEEFCAFEBABE);
}

#[test]
fn test_bool_return() {
    assert_eq!(bool_return(50), false);
    assert_eq!(bool_return(101), true);
    assert_eq!(bool_return(100), false);
}

#[test]
fn test_negation() {
    assert_eq!(negation(0), !0u64);
    assert_eq!(negation(0xFF), !0xFFu64);
}
