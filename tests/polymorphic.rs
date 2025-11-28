//! Tests for polymorphic code generation
//!
//! These tests verify that polymorphism works correctly
//! while maintaining functional equivalence.

use aegis_vm_macro::vm_protect;

// ============================================================================
// SECTION 1: Same logic, different names → different bytecode
// ============================================================================

/// Function with polymorphism (Standard level)
#[vm_protect]
fn poly_add_a(x: u64) -> u64 {
    x + 42
}

/// Same logic, different name → different bytecode
#[vm_protect]
fn poly_add_b(x: u64) -> u64 {
    x + 42
}

/// Same logic, another name
#[vm_protect]
fn poly_add_c(x: u64) -> u64 {
    x + 42
}

/// Yet another variant
#[vm_protect]
fn poly_add_d(x: u64) -> u64 {
    x + 42
}

/// Fifth variant
#[vm_protect]
fn poly_add_e(x: u64) -> u64 {
    x + 42
}

// ============================================================================
// SECTION 2: Complex functions with different names
// ============================================================================

#[vm_protect]
fn poly_complex_a(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xFF
}

#[vm_protect]
fn poly_complex_b(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xFF
}

#[vm_protect]
fn poly_complex_c(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xFF
}

// ============================================================================
// SECTION 3: Paranoid level (heavy polymorphism)
// ============================================================================

#[vm_protect(level = "paranoid")]
fn paranoid_add(x: u64) -> u64 {
    x + 100
}

#[vm_protect(level = "paranoid")]
fn paranoid_add_v2(x: u64) -> u64 {
    x + 100
}

#[vm_protect(level = "paranoid")]
fn paranoid_mul(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect(level = "paranoid")]
fn paranoid_mul_v2(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect(level = "paranoid")]
fn paranoid_xor(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect(level = "paranoid")]
fn paranoid_complex(x: u64, y: u64) -> u64 {
    ((x ^ y) + (x & y)) * 2
}

// ============================================================================
// SECTION 4: Debug level (no polymorphism)
// ============================================================================

#[vm_protect(level = "debug")]
fn debug_add(x: u64) -> u64 {
    x + 42
}

#[vm_protect(level = "debug")]
fn debug_mul(x: u64, y: u64) -> u64 {
    x * y
}

#[vm_protect(level = "debug")]
fn debug_complex(x: u64, y: u64) -> u64 {
    ((x + y) * 2) ^ 0xFF
}

// ============================================================================
// SECTION 5: Bitwise operations
// ============================================================================

#[vm_protect]
fn poly_bitwise_and_a(x: u64, y: u64) -> u64 {
    x & y
}

#[vm_protect]
fn poly_bitwise_and_b(x: u64, y: u64) -> u64 {
    x & y
}

#[vm_protect]
fn poly_bitwise_or_a(x: u64, y: u64) -> u64 {
    x | y
}

#[vm_protect]
fn poly_bitwise_or_b(x: u64, y: u64) -> u64 {
    x | y
}

#[vm_protect]
fn poly_bitwise_xor_a(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect]
fn poly_bitwise_xor_b(x: u64, y: u64) -> u64 {
    x ^ y
}

#[vm_protect]
fn poly_not_a(x: u64) -> u64 {
    !x
}

#[vm_protect]
fn poly_not_b(x: u64) -> u64 {
    !x
}

// ============================================================================
// SECTION 6: Shift operations
// ============================================================================

#[vm_protect]
fn poly_shl_a(x: u64, shift: u64) -> u64 {
    x << (shift & 63)
}

#[vm_protect]
fn poly_shl_b(x: u64, shift: u64) -> u64 {
    x << (shift & 63)
}

#[vm_protect]
fn poly_shr_a(x: u64, shift: u64) -> u64 {
    x >> (shift & 63)
}

#[vm_protect]
fn poly_shr_b(x: u64, shift: u64) -> u64 {
    x >> (shift & 63)
}

// ============================================================================
// SECTION 7: Large constants
// ============================================================================

#[vm_protect]
fn poly_large_const_a(x: u64) -> u64 {
    x ^ 0xDEADBEEFCAFEBABE
}

#[vm_protect]
fn poly_large_const_b(x: u64) -> u64 {
    x ^ 0xDEADBEEFCAFEBABE
}

#[vm_protect]
fn poly_large_add_a(x: u64) -> u64 {
    x + 0x123456789ABCDEF0
}

#[vm_protect]
fn poly_large_add_b(x: u64) -> u64 {
    x + 0x123456789ABCDEF0
}

// ============================================================================
// SECTION 8: Single argument functions
// ============================================================================

#[vm_protect]
fn poly_single_inc_a(x: u64) -> u64 {
    x + 1
}

#[vm_protect]
fn poly_single_inc_b(x: u64) -> u64 {
    x + 1
}

#[vm_protect]
fn poly_single_dec_a(x: u64) -> u64 {
    x - 1
}

#[vm_protect]
fn poly_single_dec_b(x: u64) -> u64 {
    x - 1
}

#[vm_protect]
fn poly_single_double_a(x: u64) -> u64 {
    x * 2
}

#[vm_protect]
fn poly_single_double_b(x: u64) -> u64 {
    x * 2
}

// ============================================================================
// SECTION 9: No argument functions
// ============================================================================

#[vm_protect]
fn poly_const_only_a() -> u64 {
    0xCAFEBABE
}

#[vm_protect]
fn poly_const_only_b() -> u64 {
    0xCAFEBABE
}

#[vm_protect]
fn poly_const_expr_a() -> u64 {
    (0xDEAD + 0xBEEF) ^ 0xFF
}

#[vm_protect]
fn poly_const_expr_b() -> u64 {
    (0xDEAD + 0xBEEF) ^ 0xFF
}

// ============================================================================
// SECTION 10: Nested expressions
// ============================================================================

#[vm_protect]
fn poly_nested_a(x: u64, y: u64) -> u64 {
    ((x + y) * (x - y)) ^ ((x & y) | (x ^ y))
}

#[vm_protect]
fn poly_nested_b(x: u64, y: u64) -> u64 {
    ((x + y) * (x - y)) ^ ((x & y) | (x ^ y))
}

#[vm_protect(level = "paranoid")]
fn poly_nested_paranoid_a(x: u64, y: u64) -> u64 {
    ((x + y) * (x - y)) ^ ((x & y) | (x ^ y))
}

#[vm_protect(level = "paranoid")]
fn poly_nested_paranoid_b(x: u64, y: u64) -> u64 {
    ((x + y) * (x - y)) ^ ((x & y) | (x ^ y))
}

// ============================================================================
// SECTION 11: Many variants with different name patterns
// ============================================================================

#[vm_protect]
fn hash_variant_001(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_002(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_003(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_004(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_005(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_006(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_007(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_008(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_009(x: u64) -> u64 { x + 1 }
#[vm_protect]
fn hash_variant_010(x: u64) -> u64 { x + 1 }

// ============================================================================
// TESTS
// ============================================================================

// --- Basic equivalence tests ---

#[test]
fn test_poly_add_equivalence() {
    for x in [0, 1, 42, 100, 255, 1000, u64::MAX - 42] {
        let a = poly_add_a(x);
        let b = poly_add_b(x);
        let c = poly_add_c(x);
        let d = poly_add_d(x);
        let e = poly_add_e(x);
        let expected = x.wrapping_add(42);

        assert_eq!(a, expected, "poly_add_a failed for x={}", x);
        assert_eq!(b, expected, "poly_add_b failed for x={}", x);
        assert_eq!(c, expected, "poly_add_c failed for x={}", x);
        assert_eq!(d, expected, "poly_add_d failed for x={}", x);
        assert_eq!(e, expected, "poly_add_e failed for x={}", x);
    }
}

#[test]
fn test_poly_complex_equivalence() {
    for x in [0u64, 10, 50, 100, 1000] {
        for y in [0u64, 5, 20, 100, 500] {
            let expected = ((x.wrapping_add(y)).wrapping_mul(2)) ^ 0xFF;
            let a = poly_complex_a(x, y);
            let b = poly_complex_b(x, y);
            let c = poly_complex_c(x, y);

            assert_eq!(a, expected, "poly_complex_a failed for x={}, y={}", x, y);
            assert_eq!(b, expected, "poly_complex_b failed for x={}, y={}", x, y);
            assert_eq!(c, expected, "poly_complex_c failed for x={}, y={}", x, y);
        }
    }
}

// --- Paranoid level tests ---

#[test]
fn test_paranoid_add() {
    for x in [0, 50, 100, 155, 1000, u64::MAX - 100] {
        let expected = x.wrapping_add(100);
        assert_eq!(paranoid_add(x), expected);
        assert_eq!(paranoid_add_v2(x), expected);
    }
}

#[test]
fn test_paranoid_mul() {
    for (x, y) in [(6u64, 7u64), (10, 10), (0, 1000), (100, 100), (1, u64::MAX)] {
        let expected = x.wrapping_mul(y);
        assert_eq!(paranoid_mul(x, y), expected);
        assert_eq!(paranoid_mul_v2(x, y), expected);
    }
}

#[test]
fn test_paranoid_xor() {
    for x in [0, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        for y in [0, 0xFF, 0xAAAA, 0x5555, u64::MAX] {
            let expected = x ^ y;
            assert_eq!(paranoid_xor(x, y), expected);
        }
    }
}

#[test]
fn test_paranoid_complex() {
    for x in [0u64, 10, 100, 0xFF] {
        for y in [0u64, 5, 50, 0xAA] {
            let expected = ((x ^ y) + (x & y)).wrapping_mul(2);
            assert_eq!(paranoid_complex(x, y), expected);
        }
    }
}

// --- Debug level tests ---

#[test]
fn test_debug_add() {
    for x in [0, 42, 100, 1000] {
        assert_eq!(debug_add(x), x + 42);
    }
}

#[test]
fn test_debug_mul() {
    for (x, y) in [(2, 3), (10, 10), (0, 100)] {
        assert_eq!(debug_mul(x, y), x * y);
    }
}

#[test]
fn test_debug_complex() {
    for x in [0, 10, 50] {
        for y in [0, 5, 20] {
            let expected = ((x + y) * 2) ^ 0xFF;
            assert_eq!(debug_complex(x, y), expected);
        }
    }
}

// --- Bitwise operation tests ---

#[test]
fn test_poly_bitwise_and() {
    for x in [0, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        for y in [0, 0xFF, 0xAAAA, 0x5555, u64::MAX] {
            let expected = x & y;
            assert_eq!(poly_bitwise_and_a(x, y), expected, "and_a failed");
            assert_eq!(poly_bitwise_and_b(x, y), expected, "and_b failed");
        }
    }
}

#[test]
fn test_poly_bitwise_or() {
    for x in [0, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        for y in [0, 0xFF, 0xAAAA, 0x5555, u64::MAX] {
            let expected = x | y;
            assert_eq!(poly_bitwise_or_a(x, y), expected, "or_a failed");
            assert_eq!(poly_bitwise_or_b(x, y), expected, "or_b failed");
        }
    }
}

#[test]
fn test_poly_bitwise_xor() {
    for x in [0, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        for y in [0, 0xFF, 0xAAAA, 0x5555, u64::MAX] {
            let expected = x ^ y;
            assert_eq!(poly_bitwise_xor_a(x, y), expected, "xor_a failed");
            assert_eq!(poly_bitwise_xor_b(x, y), expected, "xor_b failed");
        }
    }
}

#[test]
fn test_poly_not() {
    for x in [0, 1, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        let expected = !x;
        assert_eq!(poly_not_a(x), expected, "not_a failed for x={}", x);
        assert_eq!(poly_not_b(x), expected, "not_b failed for x={}", x);
    }
}

// --- Shift operation tests ---

#[test]
fn test_poly_shl() {
    for x in [1, 0xFF, 0xDEADBEEF] {
        for shift in [0, 1, 4, 8, 16, 32, 63] {
            let expected = x << shift;
            assert_eq!(poly_shl_a(x, shift), expected, "shl_a failed");
            assert_eq!(poly_shl_b(x, shift), expected, "shl_b failed");
        }
    }
}

#[test]
fn test_poly_shr() {
    for x in [u64::MAX, 0xFFFFFFFF, 0xDEADBEEF00000000] {
        for shift in [0, 1, 4, 8, 16, 32, 63] {
            let expected = x >> shift;
            assert_eq!(poly_shr_a(x, shift), expected, "shr_a failed");
            assert_eq!(poly_shr_b(x, shift), expected, "shr_b failed");
        }
    }
}

// --- Large constant tests ---

#[test]
fn test_poly_large_const() {
    for x in [0, 1, 0xFF, 0xFFFFFFFF, u64::MAX] {
        let expected = x ^ 0xDEADBEEFCAFEBABE;
        assert_eq!(poly_large_const_a(x), expected, "large_const_a failed");
        assert_eq!(poly_large_const_b(x), expected, "large_const_b failed");
    }
}

#[test]
fn test_poly_large_add() {
    for x in [0u64, 1, 100, 0xFFFFFFFF] {
        let expected = x.wrapping_add(0x123456789ABCDEF0);
        assert_eq!(poly_large_add_a(x), expected, "large_add_a failed");
        assert_eq!(poly_large_add_b(x), expected, "large_add_b failed");
    }
}

// --- Single argument tests ---

#[test]
fn test_poly_single_inc() {
    for x in [0, 1, 100, u64::MAX - 1, u64::MAX] {
        let expected = x.wrapping_add(1);
        assert_eq!(poly_single_inc_a(x), expected);
        assert_eq!(poly_single_inc_b(x), expected);
    }
}

#[test]
fn test_poly_single_dec() {
    for x in [0, 1, 100, u64::MAX] {
        let expected = x.wrapping_sub(1);
        assert_eq!(poly_single_dec_a(x), expected);
        assert_eq!(poly_single_dec_b(x), expected);
    }
}

#[test]
fn test_poly_single_double() {
    for x in [0u64, 1, 100, 0x7FFFFFFFFFFFFFFF] {
        let expected = x.wrapping_mul(2);
        assert_eq!(poly_single_double_a(x), expected);
        assert_eq!(poly_single_double_b(x), expected);
    }
}

// --- No argument tests ---

#[test]
fn test_poly_const_only() {
    assert_eq!(poly_const_only_a(), 0xCAFEBABE);
    assert_eq!(poly_const_only_b(), 0xCAFEBABE);
}

#[test]
fn test_poly_const_expr() {
    let expected = (0xDEAD + 0xBEEF) ^ 0xFF;
    assert_eq!(poly_const_expr_a(), expected);
    assert_eq!(poly_const_expr_b(), expected);
}

// --- Nested expression tests ---

#[test]
fn test_poly_nested() {
    for x in [10u64, 50, 100, 1000] {
        for y in [5u64, 25, 50, 100] {
            let expected = ((x.wrapping_add(y)).wrapping_mul(x.wrapping_sub(y)))
                ^ ((x & y) | (x ^ y));
            assert_eq!(poly_nested_a(x, y), expected, "nested_a failed");
            assert_eq!(poly_nested_b(x, y), expected, "nested_b failed");
        }
    }
}

#[test]
fn test_poly_nested_paranoid() {
    for x in [10u64, 50, 100] {
        for y in [5u64, 25, 50] {
            let expected = ((x.wrapping_add(y)).wrapping_mul(x.wrapping_sub(y)))
                ^ ((x & y) | (x ^ y));
            assert_eq!(poly_nested_paranoid_a(x, y), expected, "nested_paranoid_a failed");
            assert_eq!(poly_nested_paranoid_b(x, y), expected, "nested_paranoid_b failed");
        }
    }
}

// --- Many variants test ---

#[test]
fn test_hash_variants() {
    for x in [0, 1, 42, 100, 1000, u64::MAX - 1] {
        let expected = x.wrapping_add(1);
        assert_eq!(hash_variant_001(x), expected);
        assert_eq!(hash_variant_002(x), expected);
        assert_eq!(hash_variant_003(x), expected);
        assert_eq!(hash_variant_004(x), expected);
        assert_eq!(hash_variant_005(x), expected);
        assert_eq!(hash_variant_006(x), expected);
        assert_eq!(hash_variant_007(x), expected);
        assert_eq!(hash_variant_008(x), expected);
        assert_eq!(hash_variant_009(x), expected);
        assert_eq!(hash_variant_010(x), expected);
    }
}

// --- Determinism tests ---

#[test]
fn test_determinism_single_function() {
    // Same function called multiple times should always return same result
    for _ in 0..100 {
        assert_eq!(poly_add_a(42), 84);
        assert_eq!(paranoid_mul(6, 7), 42);
        assert_eq!(poly_const_only_a(), 0xCAFEBABE);
    }
}

#[test]
fn test_determinism_across_variants() {
    // All variants should produce identical results
    for i in 0..100u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let y = i.wrapping_mul(0x6C62272E07BB0142);

        // All poly_add variants must match
        let add_result = poly_add_a(x);
        assert_eq!(poly_add_b(x), add_result);
        assert_eq!(poly_add_c(x), add_result);
        assert_eq!(poly_add_d(x), add_result);
        assert_eq!(poly_add_e(x), add_result);

        // All poly_complex variants must match
        let complex_result = poly_complex_a(x, y);
        assert_eq!(poly_complex_b(x, y), complex_result);
        assert_eq!(poly_complex_c(x, y), complex_result);
    }
}

// --- Cross-level equivalence tests ---

#[test]
fn test_cross_level_add() {
    // Debug, Standard, and Paranoid should all produce same result
    for x in [0, 42, 100, 1000, u64::MAX - 42] {
        let debug_result = debug_add(x);
        let standard_result = poly_add_a(x);
        // Note: paranoid_add uses +100, not +42, so we can't compare directly

        assert_eq!(debug_result, standard_result,
            "debug vs standard mismatch for x={}", x);
    }
}

#[test]
fn test_cross_level_mul() {
    for x in [0, 6, 10, 100] {
        for y in [0, 7, 10, 50] {
            let debug_result = debug_mul(x, y);
            let paranoid_result = paranoid_mul(x, y);
            let paranoid_v2_result = paranoid_mul_v2(x, y);

            assert_eq!(debug_result, paranoid_result,
                "debug vs paranoid mismatch for ({}, {})", x, y);
            assert_eq!(paranoid_result, paranoid_v2_result,
                "paranoid vs paranoid_v2 mismatch for ({}, {})", x, y);
        }
    }
}

#[test]
fn test_cross_level_complex() {
    for x in [0, 10, 50] {
        for y in [0, 5, 20] {
            let debug_result = debug_complex(x, y);
            let standard_a = poly_complex_a(x, y);
            let standard_b = poly_complex_b(x, y);

            assert_eq!(debug_result, standard_a,
                "debug vs standard_a mismatch for ({}, {})", x, y);
            assert_eq!(standard_a, standard_b,
                "standard_a vs standard_b mismatch for ({}, {})", x, y);
        }
    }
}

// --- Edge case tests ---

#[test]
fn test_edge_zero() {
    assert_eq!(poly_add_a(0), 42);
    assert_eq!(paranoid_mul(0, 100), 0);
    assert_eq!(poly_bitwise_and_a(0, u64::MAX), 0);
    assert_eq!(poly_bitwise_or_a(0, 0xFF), 0xFF);
    assert_eq!(poly_bitwise_xor_a(0, 0), 0);
    assert_eq!(poly_shl_a(0, 10), 0);
    assert_eq!(poly_shr_a(0, 10), 0);
}

#[test]
fn test_edge_max() {
    assert_eq!(poly_add_a(u64::MAX), u64::MAX.wrapping_add(42));
    assert_eq!(paranoid_mul(u64::MAX, 1), u64::MAX);
    assert_eq!(poly_bitwise_and_a(u64::MAX, u64::MAX), u64::MAX);
    assert_eq!(poly_bitwise_or_a(u64::MAX, 0), u64::MAX);
    assert_eq!(poly_bitwise_xor_a(u64::MAX, u64::MAX), 0);
    assert_eq!(poly_not_a(u64::MAX), 0);
}

#[test]
fn test_edge_powers_of_two() {
    for i in 0..64 {
        let pow2 = 1u64 << i;
        assert_eq!(poly_add_a(pow2), pow2.wrapping_add(42));
        assert_eq!(poly_bitwise_and_a(pow2, pow2), pow2);
        assert_eq!(poly_bitwise_and_a(pow2, pow2 - 1), 0);
        assert_eq!(poly_bitwise_xor_a(pow2, pow2), 0);
    }
}

#[test]
fn test_edge_overflow() {
    // Addition overflow
    assert_eq!(poly_add_a(u64::MAX - 41), 0);  // MAX - 41 + 42 = 0 (wraps)
    assert_eq!(poly_add_a(u64::MAX), u64::MAX.wrapping_add(42));

    // Multiplication overflow
    assert_eq!(paranoid_mul(u64::MAX, 2), u64::MAX.wrapping_mul(2));
    assert_eq!(poly_single_double_a(u64::MAX), u64::MAX.wrapping_mul(2));

    // Subtraction underflow
    assert_eq!(poly_single_dec_a(0), u64::MAX);  // 0 - 1 = MAX (wraps)
}

// --- Stress tests ---

#[test]
fn test_stress_random_inputs() {
    for i in 0..500u64 {
        let x = i.wrapping_mul(0x9E3779B97F4A7C15);
        let y = i.wrapping_mul(0x6C62272E07BB0142);

        // Test all operations produce expected results
        assert_eq!(poly_add_a(x), x.wrapping_add(42), "add failed for i={}", i);
        assert_eq!(paranoid_mul(x % 0xFFFF, y % 0xFFFF),
            (x % 0xFFFF).wrapping_mul(y % 0xFFFF), "mul failed for i={}", i);
        assert_eq!(poly_bitwise_xor_a(x, y), x ^ y, "xor failed for i={}", i);
        assert_eq!(poly_bitwise_and_a(x, y), x & y, "and failed for i={}", i);
        assert_eq!(poly_bitwise_or_a(x, y), x | y, "or failed for i={}", i);
    }
}

#[test]
fn test_stress_shifts() {
    for x in [1u64, 0xFF, 0xFFFF, 0xDEADBEEF, u64::MAX] {
        for shift in 0..64 {
            assert_eq!(poly_shl_a(x, shift), x << shift,
                "shl failed for x={:#x}, shift={}", x, shift);
            assert_eq!(poly_shr_a(x, shift), x >> shift,
                "shr failed for x={:#x}, shift={}", x, shift);
        }
    }
}

#[test]
fn test_stress_nested() {
    for i in 0..200u64 {
        let x = (i + 10).wrapping_mul(17);
        let y = (i + 5).wrapping_mul(13);

        let expected = ((x.wrapping_add(y)).wrapping_mul(x.wrapping_sub(y)))
            ^ ((x & y) | (x ^ y));

        assert_eq!(poly_nested_a(x, y), expected, "nested_a failed for i={}", i);
        assert_eq!(poly_nested_b(x, y), expected, "nested_b failed for i={}", i);
        assert_eq!(poly_nested_paranoid_a(x, y), expected, "nested_paranoid_a failed for i={}", i);
        assert_eq!(poly_nested_paranoid_b(x, y), expected, "nested_paranoid_b failed for i={}", i);
    }
}
