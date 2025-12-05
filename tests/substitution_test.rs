//! Tests for instruction substitution correctness
//!
//! These tests verify that the complex substitution variants
//! produce mathematically equivalent results to the original operations.

use aegis_vm_macro::vm_protect;

// ============================================================================
// ADD SUBSTITUTION TESTS
// ============================================================================

/// Test basic addition (baseline)
#[vm_protect(level = "debug")]
fn add_baseline(a: u64, b: u64) -> u64 {
    a + b
}

/// Test addition with wrapping (edge cases)
#[vm_protect(level = "debug")]
fn add_wrapping(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

/// Test multiple additions in sequence
#[vm_protect(level = "debug")]
fn add_chain(a: u64, b: u64, c: u64) -> u64 {
    a + b + c
}

/// Test addition with zero
#[vm_protect(level = "debug")]
fn add_zero(a: u64) -> u64 {
    a + 0
}

/// Test addition identity
#[vm_protect(level = "debug")]
fn add_identity(a: u64) -> u64 {
    a + 0 + 0 + 0
}

// ============================================================================
// SUB SUBSTITUTION TESTS
// ============================================================================

/// Test basic subtraction (baseline)
#[vm_protect(level = "debug")]
fn sub_baseline(a: u64, b: u64) -> u64 {
    a - b
}

/// Test subtraction with wrapping (underflow)
#[vm_protect(level = "debug")]
fn sub_wrapping(a: u64, b: u64) -> u64 {
    a.wrapping_sub(b)
}

/// Test subtraction chain
#[vm_protect(level = "debug")]
fn sub_chain(a: u64, b: u64, c: u64) -> u64 {
    a - b - c
}

/// Test subtraction with zero
#[vm_protect(level = "debug")]
fn sub_zero(a: u64) -> u64 {
    a - 0
}

/// Test self-subtraction (should be 0)
#[vm_protect(level = "debug")]
fn sub_self(a: u64) -> u64 {
    a - a
}

// ============================================================================
// MIXED OPERATIONS TESTS
// ============================================================================

/// Test add then sub
#[vm_protect(level = "debug")]
fn add_then_sub(a: u64, b: u64, c: u64) -> u64 {
    (a + b) - c
}

/// Test sub then add
#[vm_protect(level = "debug")]
fn sub_then_add(a: u64, b: u64, c: u64) -> u64 {
    (a - b) + c
}

/// Test complex expression
#[vm_protect(level = "debug")]
fn complex_expr(a: u64, b: u64, c: u64, d: u64) -> u64 {
    ((a + b) - c) + d
}

/// Test with XOR (another substituted operation)
#[vm_protect(level = "debug")]
fn add_with_xor(a: u64, b: u64) -> u64 {
    (a ^ b) + (a & b)
}

/// Test NOT interaction
#[vm_protect(level = "debug")]
fn add_with_not(a: u64, b: u64) -> u64 {
    (!a).wrapping_add(b)
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

/// Test with u64::MAX
#[vm_protect(level = "debug")]
fn add_max(a: u64) -> u64 {
    a.wrapping_add(u64::MAX)
}

/// Test with powers of 2
#[vm_protect(level = "debug")]
fn add_powers_of_two() -> u64 {
    let mut sum = 0u64;
    sum = sum + 1;
    sum = sum + 2;
    sum = sum + 4;
    sum = sum + 8;
    sum = sum + 16;
    sum
}

/// Test alternating add/sub
#[vm_protect(level = "debug")]
fn alternating(a: u64, b: u64) -> u64 {
    a + b - b + b - b
}

// ============================================================================
// STANDARD PROTECTION LEVEL TESTS (Substitutions Active)
// ============================================================================

/// Test add with standard protection (substitutions enabled)
#[vm_protect(level = "standard")]
fn add_standard(a: u64, b: u64) -> u64 {
    a + b
}

/// Test sub with standard protection
#[vm_protect(level = "standard")]
fn sub_standard(a: u64, b: u64) -> u64 {
    a - b
}

/// Test complex with standard protection
#[vm_protect(level = "standard")]
fn complex_standard(a: u64, b: u64, c: u64) -> u64 {
    ((a + b) - c) + (a - b)
}

// ============================================================================
// PARANOID PROTECTION LEVEL TESTS (MBA + Substitutions)
// ============================================================================

/// Test add with paranoid protection (MBA enabled)
#[vm_protect(level = "paranoid")]
fn add_paranoid(a: u64, b: u64) -> u64 {
    a + b
}

/// Test sub with paranoid protection
#[vm_protect(level = "paranoid")]
fn sub_paranoid(a: u64, b: u64) -> u64 {
    a - b
}

/// Test complex with paranoid protection
#[vm_protect(level = "paranoid")]
fn complex_paranoid(a: u64, b: u64, c: u64) -> u64 {
    ((a + b) - c) + (a - b)
}

// ============================================================================
// TEST RUNNER
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test values for comprehensive coverage
    const TEST_VALUES: &[u64] = &[
        0,
        1,
        2,
        42,
        127,
        128,
        255,
        256,
        1000,
        u64::MAX / 2,
        u64::MAX - 1,
        u64::MAX,
    ];

    #[test]
    fn test_add_baseline() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_add(b);
                let result = add_baseline(a, b);
                assert_eq!(result, expected, "add_baseline({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_add_wrapping() {
        assert_eq!(add_wrapping(u64::MAX, 1), 0);
        assert_eq!(add_wrapping(u64::MAX, 2), 1);
        assert_eq!(add_wrapping(u64::MAX, u64::MAX), u64::MAX - 1);
    }

    #[test]
    fn test_add_chain() {
        assert_eq!(add_chain(1, 2, 3), 6);
        assert_eq!(add_chain(100, 200, 300), 600);
    }

    #[test]
    fn test_add_zero() {
        for &a in TEST_VALUES {
            assert_eq!(add_zero(a), a);
        }
    }

    #[test]
    fn test_add_identity() {
        for &a in TEST_VALUES {
            assert_eq!(add_identity(a), a);
        }
    }

    #[test]
    fn test_sub_baseline() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_sub(b);
                let result = sub_baseline(a, b);
                assert_eq!(result, expected, "sub_baseline({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_sub_wrapping() {
        assert_eq!(sub_wrapping(0, 1), u64::MAX);
        assert_eq!(sub_wrapping(0, 2), u64::MAX - 1);
        assert_eq!(sub_wrapping(1, 2), u64::MAX);
    }

    #[test]
    fn test_sub_chain() {
        assert_eq!(sub_chain(100, 30, 20), 50);
        assert_eq!(sub_chain(1000, 100, 50), 850);
    }

    #[test]
    fn test_sub_zero() {
        for &a in TEST_VALUES {
            assert_eq!(sub_zero(a), a);
        }
    }

    #[test]
    fn test_sub_self() {
        for &a in TEST_VALUES {
            assert_eq!(sub_self(a), 0);
        }
    }

    #[test]
    fn test_add_then_sub() {
        assert_eq!(add_then_sub(10, 5, 3), 12);
        assert_eq!(add_then_sub(100, 50, 25), 125);
    }

    #[test]
    fn test_sub_then_add() {
        assert_eq!(sub_then_add(10, 3, 5), 12);
        assert_eq!(sub_then_add(100, 25, 50), 125);
    }

    #[test]
    fn test_complex_expr() {
        assert_eq!(complex_expr(10, 20, 5, 3), 28);
    }

    #[test]
    fn test_add_with_xor() {
        // a ^ b + a & b is part of MBA for addition
        // For a=5 (101), b=3 (011): xor=6 (110), and=1 (001), sum=7
        // But full formula is (a^b) + 2*(a&b) = a+b
        // Here we just test (a^b) + (a&b)
        assert_eq!(add_with_xor(5, 3), 7);
    }

    #[test]
    fn test_add_with_not() {
        // !0 + 1 = MAX + 1 = 0 (wrapping)
        assert_eq!(add_with_not(0, 1), 0);
        // !1 + 1 = (MAX-1) + 1 = MAX
        assert_eq!(add_with_not(1, 1), u64::MAX);
    }

    #[test]
    fn test_add_max() {
        // a + MAX = a - 1 (wrapping)
        assert_eq!(add_max(0), u64::MAX);
        assert_eq!(add_max(1), 0);
        assert_eq!(add_max(10), 9);
    }

    #[test]
    fn test_add_powers_of_two() {
        assert_eq!(add_powers_of_two(), 31); // 1+2+4+8+16
    }

    #[test]
    fn test_alternating() {
        // a + b - b + b - b = a
        for &a in &[0u64, 1, 42, 1000, u64::MAX] {
            for &b in &[0u64, 1, 42, 1000, u64::MAX] {
                assert_eq!(alternating(a, b), a);
            }
        }
    }

    // Standard protection tests
    #[test]
    fn test_add_standard() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_add(b);
                let result = add_standard(a, b);
                assert_eq!(result, expected, "add_standard({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_sub_standard() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_sub(b);
                let result = sub_standard(a, b);
                assert_eq!(result, expected, "sub_standard({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_complex_standard() {
        assert_eq!(complex_standard(100, 50, 25), 175); // (100+50-25) + (100-50) = 125 + 50
    }

    // Paranoid protection tests
    #[test]
    fn test_add_paranoid() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_add(b);
                let result = add_paranoid(a, b);
                assert_eq!(result, expected, "add_paranoid({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_sub_paranoid() {
        for &a in TEST_VALUES {
            for &b in TEST_VALUES {
                let expected = a.wrapping_sub(b);
                let result = sub_paranoid(a, b);
                assert_eq!(result, expected, "sub_paranoid({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_complex_paranoid() {
        assert_eq!(complex_paranoid(100, 50, 25), 175);
    }
}
