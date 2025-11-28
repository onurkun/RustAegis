//! Tests for division and modulo operations

use aegis_vm_macro::vm_protect;

/// Unsigned division
#[vm_protect(level = "debug")]
fn div_test(a: u64, b: u64) -> u64 {
    a / b
}

/// Unsigned modulo
#[vm_protect(level = "debug")]
fn mod_test(a: u64, b: u64) -> u64 {
    a % b
}

/// Division with zero (should return 0, not panic)
#[vm_protect(level = "debug")]
fn div_by_zero(a: u64) -> u64 {
    a / 0
}

/// Modulo with zero (should return 0, not panic)
#[vm_protect(level = "debug")]
fn mod_by_zero(a: u64) -> u64 {
    a % 0
}

/// Combined division and modulo
#[vm_protect(level = "debug")]
fn divmod_combined(a: u64, b: u64) -> u64 {
    let quotient = a / b;
    let remainder = a % b;
    quotient * 100 + remainder
}

/// Euclidean algorithm for GCD
#[vm_protect(level = "debug")]
fn gcd_step(a: u64, b: u64) -> u64 {
    if b == 0 {
        return a;
    }
    a % b
}

#[test]
fn test_unsigned_division() {
    assert_eq!(div_test(10, 3), 3);
    assert_eq!(div_test(100, 10), 10);
    assert_eq!(div_test(7, 2), 3);
    assert_eq!(div_test(0, 5), 0);
    assert_eq!(div_test(5, 5), 1);
    assert_eq!(div_test(1, 2), 0);
}

#[test]
fn test_unsigned_modulo() {
    assert_eq!(mod_test(10, 3), 1);
    assert_eq!(mod_test(100, 10), 0);
    assert_eq!(mod_test(7, 2), 1);
    assert_eq!(mod_test(0, 5), 0);
    assert_eq!(mod_test(5, 5), 0);
    assert_eq!(mod_test(1, 2), 1);
}

#[test]
fn test_division_by_zero() {
    // VM returns 0 for division by zero (security choice - no panic)
    assert_eq!(div_by_zero(10), 0);
    assert_eq!(div_by_zero(0), 0);
}

#[test]
fn test_modulo_by_zero() {
    // VM returns 0 for modulo by zero (security choice - no panic)
    assert_eq!(mod_by_zero(10), 0);
    assert_eq!(mod_by_zero(0), 0);
}

#[test]
fn test_divmod_combined() {
    // 17 / 5 = 3, 17 % 5 = 2, result = 302
    assert_eq!(divmod_combined(17, 5), 302);
    // 100 / 7 = 14, 100 % 7 = 2, result = 1402
    assert_eq!(divmod_combined(100, 7), 1402);
}

#[test]
fn test_gcd_step() {
    // GCD(48, 18): 48 % 18 = 12
    assert_eq!(gcd_step(48, 18), 12);
    // GCD(18, 12): 18 % 12 = 6
    assert_eq!(gcd_step(18, 12), 6);
    // GCD(12, 6): 12 % 6 = 0
    assert_eq!(gcd_step(12, 6), 0);
    // GCD(6, 0): return 6
    assert_eq!(gcd_step(6, 0), 6);
}

#[test]
fn test_large_numbers() {
    let large = u64::MAX / 2;
    assert_eq!(div_test(large, 2), large / 2);
    assert_eq!(mod_test(large, 1000), large % 1000);
}
