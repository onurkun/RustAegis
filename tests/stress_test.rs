//! Stress tests for VM polymorphism
//!
//! These tests verify that polymorphic transformations don't break
//! functionality across many different inputs and edge cases.

use aegis_vm_macro::vm_protect;

// ============================================================================
// Test Functions - Various protection levels
// ============================================================================

#[vm_protect(level = "debug")]
fn debug_add(x: u64, y: u64) -> u64 {
    x + y
}

#[vm_protect]
fn standard_add(x: u64, y: u64) -> u64 {
    x + y
}

#[vm_protect(level = "paranoid")]
fn paranoid_add(x: u64, y: u64) -> u64 {
    x + y
}

#[vm_protect(level = "debug")]
fn debug_mul(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect]
fn standard_mul(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect(level = "paranoid")]
fn paranoid_mul(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect(level = "debug")]
fn debug_xor(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect]
fn standard_xor(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect(level = "paranoid")]
fn paranoid_xor(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect(level = "debug")]
fn debug_complex(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xDEADBEEF
}

#[vm_protect]
fn standard_complex(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xDEADBEEF
}

#[vm_protect(level = "paranoid")]
fn paranoid_complex(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xDEADBEEF
}

#[vm_protect]
fn bitwise_ops(x: u64, y: u64) -> u64 {
    (x & y) | (x ^ y)
}

#[vm_protect]
fn shift_ops(x: u64, shift: u64) -> u64 {
    (x << (shift & 63)) ^ (x >> (shift & 63))
}

#[vm_protect]
fn single_arg(x: u64) -> u64 {
    x + 12345
}

#[vm_protect]
fn constant_only() -> u64 {
    0xCAFEBABE
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn stress_test_add_random_inputs() {
    // Test with 1000 random-ish inputs
    for i in 0..1000u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);  // Golden ratio hash
        let y = i.wrapping_mul(0x6C62272E07BB0142);  // Another prime

        let expected = x.wrapping_add(y);

        assert_eq!(debug_add(x, y), expected, "debug_add failed for i={}", i);
        assert_eq!(standard_add(x, y), expected, "standard_add failed for i={}", i);
        assert_eq!(paranoid_add(x, y), expected, "paranoid_add failed for i={}", i);
    }
}

#[test]
fn stress_test_mul_random_inputs() {
    for i in 0..1000u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15) % 0xFFFF;  // Keep small to avoid overflow issues
        let y = i.wrapping_mul(0x6C62272E07BB0142) % 0xFFFF;

        let expected = x.wrapping_mul(y);

        assert_eq!(debug_mul(x, y), expected, "debug_mul failed for i={}", i);
        assert_eq!(standard_mul(x, y), expected, "standard_mul failed for i={}", i);
        assert_eq!(paranoid_mul(x, y), expected, "paranoid_mul failed for i={}", i);
    }
}

#[test]
fn stress_test_xor_random_inputs() {
    for i in 0..1000u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let y = i.wrapping_mul(0x6C62272E07BB0142);

        let expected = x ^ y;

        assert_eq!(debug_xor(x, y), expected, "debug_xor failed for i={}", i);
        assert_eq!(standard_xor(x, y), expected, "standard_xor failed for i={}", i);
        assert_eq!(paranoid_xor(x, y), expected, "paranoid_xor failed for i={}", i);
    }
}

#[test]
fn stress_test_complex_random_inputs() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let y = i.wrapping_mul(0x6C62272E07BB0142);

        let expected = ((x.wrapping_add(y)).wrapping_mul(2)) ^ 0xDEADBEEF;

        assert_eq!(debug_complex(x, y), expected, "debug_complex failed for i={}", i);
        assert_eq!(standard_complex(x, y), expected, "standard_complex failed for i={}", i);
        assert_eq!(paranoid_complex(x, y), expected, "paranoid_complex failed for i={}", i);
    }
}

#[test]
fn stress_test_edge_cases() {
    // Zero values
    assert_eq!(standard_add(0, 0), 0);
    assert_eq!(standard_mul(0, 0), 0);
    assert_eq!(standard_xor(0, 0), 0);

    // Max values
    assert_eq!(standard_add(u64::MAX, 0), u64::MAX);
    assert_eq!(standard_add(0, u64::MAX), u64::MAX);
    assert_eq!(standard_mul(u64::MAX, 1), u64::MAX);
    assert_eq!(standard_mul(1, u64::MAX), u64::MAX);
    assert_eq!(standard_xor(u64::MAX, 0), u64::MAX);
    assert_eq!(standard_xor(0, u64::MAX), u64::MAX);

    // Overflow wrapping
    assert_eq!(standard_add(u64::MAX, 1), 0);  // Wraps to 0
    assert_eq!(standard_add(u64::MAX, u64::MAX), u64::MAX - 1);  // Wraps

    // XOR with self = 0
    assert_eq!(standard_xor(12345, 12345), 0);
    assert_eq!(standard_xor(u64::MAX, u64::MAX), 0);

    // Powers of 2
    for i in 0..64 {
        let pow2 = 1u64 << i;
        assert_eq!(standard_add(pow2, 0), pow2, "pow2 add failed for 2^{}", i);
        assert_eq!(standard_mul(pow2, 1), pow2, "pow2 mul failed for 2^{}", i);
        assert_eq!(standard_xor(pow2, 0), pow2, "pow2 xor failed for 2^{}", i);
    }
}

#[test]
fn stress_test_bitwise_ops() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let y = i.wrapping_mul(0x6C62272E07BB0142);

        let expected = (x & y) | (x ^ y);
        assert_eq!(bitwise_ops(x, y), expected, "bitwise_ops failed for i={}", i);
    }
}

#[test]
fn stress_test_shift_ops() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let shift = i % 64;

        let expected = (x << shift) ^ (x >> shift);
        assert_eq!(shift_ops(x, shift), expected, "shift_ops failed for i={}, shift={}", i, shift);
    }
}

#[test]
fn stress_test_single_arg() {
    for i in 0..1000u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let expected = x.wrapping_add(12345);
        assert_eq!(single_arg(x), expected, "single_arg failed for i={}", i);
    }
}

#[test]
fn stress_test_constant_only() {
    // Should always return the same constant
    for _ in 0..100 {
        assert_eq!(constant_only(), 0xCAFEBABE);
    }
}

#[test]
fn stress_test_determinism() {
    // Same inputs should always give same outputs
    let test_pairs: Vec<(u64, u64)> = vec![
        (0, 0),
        (1, 1),
        (100, 200),
        (u64::MAX, 0),
        (0x12345678, 0x87654321),
        (0xDEADBEEF, 0xCAFEBABE),
    ];

    for (x, y) in test_pairs {
        let result1 = standard_add(x, y);
        let result2 = standard_add(x, y);
        let result3 = standard_add(x, y);

        assert_eq!(result1, result2, "determinism failed for add({}, {})", x, y);
        assert_eq!(result2, result3, "determinism failed for add({}, {})", x, y);

        let result1 = paranoid_mul(x, y);
        let result2 = paranoid_mul(x, y);

        assert_eq!(result1, result2, "determinism failed for mul({}, {})", x, y);
    }
}

#[test]
fn stress_test_all_levels_equivalent() {
    // All protection levels should produce the same result
    for i in 0..100u64 {
        let x = i * 17;
        let y = i * 31;

        let debug_result = debug_add(x, y);
        let standard_result = standard_add(x, y);
        let paranoid_result = paranoid_add(x, y);

        assert_eq!(debug_result, standard_result,
            "debug vs standard mismatch for add({}, {})", x, y);
        assert_eq!(standard_result, paranoid_result,
            "standard vs paranoid mismatch for add({}, {})", x, y);

        let debug_result = debug_mul(x, y);
        let standard_result = standard_mul(x, y);
        let paranoid_result = paranoid_mul(x, y);

        assert_eq!(debug_result, standard_result,
            "debug vs standard mismatch for mul({}, {})", x, y);
        assert_eq!(standard_result, paranoid_result,
            "standard vs paranoid mismatch for mul({}, {})", x, y);
    }
}
