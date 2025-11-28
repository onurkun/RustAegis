//! Tests for anti-analysis and instruction substitution
//!
//! These tests verify that the protected functions still work correctly
//! after anti-analysis transformations are applied.

use aegis_vm_macro::vm_protect;

// =============================================================================
// Test functions with various protection levels
// =============================================================================

#[vm_protect(level = "debug")]
fn debug_arithmetic(x: u64, y: u64) -> u64 {
    x + y + 1
}

#[vm_protect]
fn standard_arithmetic(x: u64, y: u64) -> u64 {
    x + y + 1
}

#[vm_protect(level = "paranoid")]
fn paranoid_arithmetic(x: u64, y: u64) -> u64 {
    x + y + 1
}

// More complex functions that would trigger substitution

#[vm_protect]
fn inc_chain(x: u64) -> u64 {
    let a = x + 1;
    let b = a + 1;
    let c = b + 1;
    c + 1
}

#[vm_protect(level = "paranoid")]
fn dec_chain(x: u64) -> u64 {
    let a = x - 1;
    let b = a - 1;
    let c = b - 1;
    c - 1
}

#[vm_protect]
fn mixed_ops(x: u64, y: u64) -> u64 {
    let sum = x + y;
    let diff = x - y;
    let xored = sum ^ diff;
    let anded = xored & 0xFF;
    let ored = anded | 0x10;
    !ored
}

#[vm_protect(level = "paranoid")]
fn paranoid_mixed_ops(x: u64, y: u64) -> u64 {
    let sum = x + y;
    let diff = x - y;
    let xored = sum ^ diff;
    let anded = xored & 0xFF;
    let ored = anded | 0x10;
    !ored
}

#[vm_protect]
fn bitwise_not_chain(x: u64) -> u64 {
    let a = !x;
    let b = !a;
    let c = !b;
    !c
}

#[vm_protect]
fn shift_ops(x: u64) -> u64 {
    let a = x << 1;
    let b = a >> 1;
    let c = b << 2;
    c >> 2
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_debug_arithmetic() {
    assert_eq!(debug_arithmetic(10, 20), 31);
    assert_eq!(debug_arithmetic(0, 0), 1);
    assert_eq!(debug_arithmetic(100, 200), 301);
}

#[test]
fn test_standard_arithmetic() {
    assert_eq!(standard_arithmetic(10, 20), 31);
    assert_eq!(standard_arithmetic(0, 0), 1);
    assert_eq!(standard_arithmetic(100, 200), 301);
}

#[test]
fn test_paranoid_arithmetic() {
    assert_eq!(paranoid_arithmetic(10, 20), 31);
    assert_eq!(paranoid_arithmetic(0, 0), 1);
    assert_eq!(paranoid_arithmetic(100, 200), 301);
}

#[test]
fn test_inc_chain() {
    assert_eq!(inc_chain(0), 4);
    assert_eq!(inc_chain(10), 14);
    assert_eq!(inc_chain(100), 104);
}

#[test]
fn test_dec_chain() {
    assert_eq!(dec_chain(100), 96);
    assert_eq!(dec_chain(10), 6);
    assert_eq!(dec_chain(4), 0);
}

#[test]
fn test_mixed_ops() {
    for x in [10u64, 50, 100, 255] {
        for y in [5u64, 25, 50, 100] {
            let expected = {
                let sum = x.wrapping_add(y);
                let diff = x.wrapping_sub(y);
                let xored = sum ^ diff;
                let anded = xored & 0xFF;
                let ored = anded | 0x10;
                !ored
            };
            assert_eq!(mixed_ops(x, y), expected, "mixed_ops({}, {})", x, y);
        }
    }
}

#[test]
fn test_paranoid_mixed_ops() {
    for x in [10u64, 50, 100, 255] {
        for y in [5u64, 25, 50, 100] {
            let expected = {
                let sum = x.wrapping_add(y);
                let diff = x.wrapping_sub(y);
                let xored = sum ^ diff;
                let anded = xored & 0xFF;
                let ored = anded | 0x10;
                !ored
            };
            assert_eq!(paranoid_mixed_ops(x, y), expected, "paranoid_mixed_ops({}, {})", x, y);
        }
    }
}

#[test]
fn test_bitwise_not_chain() {
    // !!!!x = x
    assert_eq!(bitwise_not_chain(0), 0);
    assert_eq!(bitwise_not_chain(0xFF), 0xFF);
    assert_eq!(bitwise_not_chain(u64::MAX), u64::MAX);
    assert_eq!(bitwise_not_chain(0xDEADBEEF), 0xDEADBEEF);
}

#[test]
fn test_shift_ops() {
    // (((x << 1) >> 1) << 2) >> 2 = x (for values that don't overflow)
    assert_eq!(shift_ops(0), 0);
    assert_eq!(shift_ops(1), 1);
    assert_eq!(shift_ops(100), 100);
    assert_eq!(shift_ops(0x0FFFFFFF), 0x0FFFFFFF);
}

// Cross-level equivalence tests

#[test]
fn test_cross_level_equivalence() {
    for x in [0u64, 10, 50, 100, 1000] {
        for y in [0u64, 5, 25, 50, 500] {
            let debug = debug_arithmetic(x, y);
            let standard = standard_arithmetic(x, y);
            let paranoid = paranoid_arithmetic(x, y);

            assert_eq!(debug, standard,
                "debug vs standard mismatch for ({}, {})", x, y);
            assert_eq!(standard, paranoid,
                "standard vs paranoid mismatch for ({}, {})", x, y);
        }
    }
}

#[test]
fn test_mixed_ops_equivalence() {
    for x in [10u64, 100, 255] {
        for y in [5u64, 50, 100] {
            let standard = mixed_ops(x, y);
            let paranoid = paranoid_mixed_ops(x, y);

            assert_eq!(standard, paranoid,
                "standard vs paranoid mismatch for mixed_ops({}, {})", x, y);
        }
    }
}

// Stress tests with random inputs

#[test]
fn test_stress_inc_chain() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let expected = x.wrapping_add(4);
        assert_eq!(inc_chain(x), expected, "inc_chain failed for i={}", i);
    }
}

#[test]
fn test_stress_dec_chain() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let expected = x.wrapping_sub(4);
        assert_eq!(dec_chain(x), expected, "dec_chain failed for i={}", i);
    }
}

#[test]
fn test_stress_mixed_ops() {
    for i in 0..200u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15) % 1000;
        let y = i.wrapping_mul(0x6C62272E07BB0142) % 500;

        let expected = {
            let sum = x.wrapping_add(y);
            let diff = x.wrapping_sub(y);
            let xored = sum ^ diff;
            let anded = xored & 0xFF;
            let ored = anded | 0x10;
            !ored
        };

        assert_eq!(mixed_ops(x, y), expected, "stress mixed_ops failed for i={}", i);
        assert_eq!(paranoid_mixed_ops(x, y), expected, "stress paranoid_mixed_ops failed for i={}", i);
    }
}

// Edge case tests

#[test]
fn test_edge_cases() {
    // Zero values
    assert_eq!(debug_arithmetic(0, 0), 1);
    assert_eq!(inc_chain(0), 4);
    assert_eq!(dec_chain(4), 0);
    assert_eq!(mixed_ops(0, 0), !0x10u64);

    // Max values
    assert_eq!(inc_chain(u64::MAX - 4), u64::MAX);
    assert_eq!(inc_chain(u64::MAX), 3);  // Wraps around
    assert_eq!(dec_chain(3), u64::MAX);  // Wraps around

    // NOT chain is identity
    assert_eq!(bitwise_not_chain(0), 0);
    assert_eq!(bitwise_not_chain(u64::MAX), u64::MAX);
}
