//! Test for if/else conditions and conditional branching
//!
//! These tests ensure functions with if/else work correctly after
//! the flag-based condition fix.

use aegis_vm_macro::vm_protect;

/// Simple identity function
#[vm_protect(level = "debug")]
fn identity(x: u64) -> u64 {
    x
}

/// Return constant if zero, else return parameter
#[vm_protect(level = "debug")]
fn check_zero(x: u64) -> u64 {
    if x == 0 {
        return 42;
    }
    x
}

/// Multiple if checks (chained returns)
#[vm_protect(level = "debug")]
fn threat_penalty_debug(level: u64) -> u64 {
    if level == 0 {
        return 5;
    }
    if level == 1 {
        return 15;
    }
    if level == 2 {
        return 30;
    }
    if level == 3 {
        return 50;
    }
    if level == 4 {
        return 100;
    }
    100
}

/// Same with standard protection level
#[vm_protect]
fn threat_penalty_standard(level: u64) -> u64 {
    if level == 0 {
        return 5;
    }
    if level == 1 {
        return 15;
    }
    if level == 2 {
        return 30;
    }
    if level == 3 {
        return 50;
    }
    if level == 4 {
        return 100;
    }
    100
}

/// Same with paranoid protection level
#[vm_protect(level = "paranoid")]
fn threat_penalty_paranoid(level: u64) -> u64 {
    if level == 0 {
        return 5;
    }
    if level == 1 {
        return 15;
    }
    if level == 2 {
        return 30;
    }
    if level == 3 {
        return 50;
    }
    if level == 4 {
        return 100;
    }
    100
}

#[test]
fn test_identity() {
    assert_eq!(identity(0), 0);
    assert_eq!(identity(42), 42);
    assert_eq!(identity(123), 123);
}

#[test]
fn test_check_zero() {
    assert_eq!(check_zero(0), 42);
    assert_eq!(check_zero(1), 1);
    assert_eq!(check_zero(100), 100);
}

#[test]
fn test_threat_penalty_debug() {
    assert_eq!(threat_penalty_debug(0), 5);
    assert_eq!(threat_penalty_debug(1), 15);
    assert_eq!(threat_penalty_debug(2), 30);
    assert_eq!(threat_penalty_debug(3), 50);
    assert_eq!(threat_penalty_debug(4), 100);
    assert_eq!(threat_penalty_debug(99), 100);
}

#[test]
fn test_threat_penalty_standard() {
    assert_eq!(threat_penalty_standard(0), 5);
    assert_eq!(threat_penalty_standard(1), 15);
    assert_eq!(threat_penalty_standard(2), 30);
    assert_eq!(threat_penalty_standard(3), 50);
    assert_eq!(threat_penalty_standard(4), 100);
    assert_eq!(threat_penalty_standard(99), 100);
}

#[test]
fn test_threat_penalty_paranoid() {
    assert_eq!(threat_penalty_paranoid(0), 5);
    assert_eq!(threat_penalty_paranoid(1), 15);
    assert_eq!(threat_penalty_paranoid(2), 30);
    assert_eq!(threat_penalty_paranoid(3), 50);
    assert_eq!(threat_penalty_paranoid(4), 100);
    assert_eq!(threat_penalty_paranoid(99), 100);
}

#[test]
fn test_all_protection_levels_equivalent() {
    for level in 0..=10 {
        let debug = threat_penalty_debug(level);
        let standard = threat_penalty_standard(level);
        let paranoid = threat_penalty_paranoid(level);

        assert_eq!(debug, standard, "debug vs standard mismatch for level {}", level);
        assert_eq!(standard, paranoid, "standard vs paranoid mismatch for level {}", level);
    }
}

// ============================================================================
// If-Else Expression Assignment Tests
// ============================================================================

/// Test if-else expression as value (not statement)
#[vm_protect(level = "debug")]
fn max_value(a: u64, b: u64) -> u64 {
    let result = if a > b { a } else { b };
    result
}

/// Test if-else expression directly returned
#[vm_protect(level = "debug")]
fn max_value_direct(a: u64, b: u64) -> u64 {
    if a > b { a } else { b }
}

/// Test nested if-else expressions
#[vm_protect(level = "debug")]
fn clamp_value(val: u64, min: u64, max: u64) -> u64 {
    let clamped = if val < min {
        min
    } else {
        if val > max { max } else { val }
    };
    clamped
}

#[test]
fn test_if_else_expression_max() {
    assert_eq!(max_value(10, 5), 10);
    assert_eq!(max_value(5, 10), 10);
    assert_eq!(max_value(7, 7), 7);
}

#[test]
fn test_if_else_expression_direct() {
    assert_eq!(max_value_direct(10, 5), 10);
    assert_eq!(max_value_direct(5, 10), 10);
    assert_eq!(max_value_direct(7, 7), 7);
}

#[test]
fn test_if_else_nested_clamp() {
    // Below min
    assert_eq!(clamp_value(5, 10, 100), 10);
    // Above max
    assert_eq!(clamp_value(150, 10, 100), 100);
    // In range
    assert_eq!(clamp_value(50, 10, 100), 50);
    // At boundaries
    assert_eq!(clamp_value(10, 10, 100), 10);
    assert_eq!(clamp_value(100, 10, 100), 100);
}

// ============================================================================
// If-Else Expression INSIDE Loop Tests
// ============================================================================

/// Test if-else expression inside while loop
#[vm_protect(level = "debug")]
fn max_in_loop(packed: u64) -> u64 {
    let mut max_val: u64 = 0;
    let mut remaining = packed;
    let mut i: u64 = 0;

    while i < 8 {
        let byte_val = remaining & 0xFF;
        // This is the problematic pattern
        let new_max = if byte_val > max_val { byte_val } else { max_val };
        max_val = new_max;
        remaining = remaining >> 8;
        i = i + 1;
    }

    max_val
}

#[test]
fn test_if_else_in_loop() {
    // Pack: [10, 50, 30, 0, 0, 0, 0, 0]
    let packed = 10 | (50 << 8) | (30 << 16);
    assert_eq!(max_in_loop(packed), 50);
}

/// Exact copy of the failing function for debugging
#[vm_protect(level = "debug")]
fn validate_timing_deltas_debug(packed_deltas: u64, max_delta: u64) -> u64 {
    let mut remaining = packed_deltas;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < 8 {
        let current = remaining & 0xFF;
        // Check if delta between consecutive measurements is too large
        if i > 0 {
            // Compute absolute difference
            let larger = if current > prev { current } else { prev };
            let smaller = if current > prev { prev } else { current };
            let delta = larger - smaller;
            if delta > max_delta {
                return 1;
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_timing_deltas_debug() {
    // Need all 8 bytes valid: [10, 12, 11, 13, 12, 11, 13, 12]
    let smooth = 10u64 | (12u64 << 8) | (11u64 << 16) | (13u64 << 24) |
                 (12u64 << 32) | (11u64 << 40) | (13u64 << 48) | (12u64 << 56);
    assert_eq!(validate_timing_deltas_debug(smooth, 5), 0);

    // Spike: put large delta early so test doesn't depend on trailing zeros
    // [10, 50, ...] - immediate spike
    let spike = 10 | (50 << 8);
    assert_eq!(validate_timing_deltas_debug(spike, 5), 1);
}

/// Simplified version - remove nested if
#[vm_protect(level = "debug")]
fn timing_v2(packed_deltas: u64, max_delta: u64) -> u64 {
    let mut remaining = packed_deltas;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < 8 {
        let current = remaining & 0xFF;

        // Stop at trailing zeros
        if current == 0 {
            let rest = remaining >> 8;
            if rest == 0 {
                return 0;
            }
        }

        // Removed nested if-else
        if i > 0 {
            // Just compute delta directly without nested if
            if current > prev {
                let delta = current - prev;
                if delta > max_delta {
                    return 1;
                }
            } else {
                let delta = prev - current;
                if delta > max_delta {
                    return 1;
                }
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_timing_v2() {
    let smooth = 10 | (12 << 8) | (11 << 16) | (13 << 24) | (12u64 << 32);
    assert_eq!(timing_v2(smooth, 5), 0);

    let spike = 10 | (12 << 8) | (50 << 16) | (13 << 24);
    assert_eq!(timing_v2(spike, 5), 1);
}

/// Even simpler - single if-else in loop
#[vm_protect(level = "debug")]
fn timing_v3(packed_deltas: u64, max_delta: u64) -> u64 {
    let mut remaining = packed_deltas;
    let mut i: u64 = 0;

    while i < 4 {
        let current = remaining & 0xFF;
        if current > max_delta {
            return 1;
        }
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_timing_v3() {
    let packed = 5 | (3 << 8) | (10 << 16) | (2 << 24);  // max=10
    assert_eq!(timing_v3(packed, 15), 0);  // all under 15
    assert_eq!(timing_v3(packed, 5), 1);   // 10 > 5
}

/// Test: if-else-if in loop - MINIMAL
#[vm_protect(level = "debug")]
fn nested_if_in_loop(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut result: u64 = 0;

    while i < n {
        if i == 1 {
            result = result + 10;
        } else {
            if i == 2 {
                result = result + 20;
            }
        }
        i = i + 1;
    }

    result
}

#[test]
fn test_nested_if_in_loop() {
    // i=0: neither branch
    // i=1: +10
    // i=2: +20
    // i=3: neither
    // Total: 30
    assert_eq!(nested_if_in_loop(4), 30);
}

/// Test: simple if (no else) in loop
#[vm_protect(level = "debug")]
fn simple_if_in_loop(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut result: u64 = 0;

    while i < n {
        if i == 1 {
            result = result + 10;
        }
        i = i + 1;
    }

    result
}

#[test]
fn test_simple_if_in_loop() {
    assert_eq!(simple_if_in_loop(4), 10);
}

/// Test: let binding INSIDE if branch in loop
#[vm_protect(level = "debug")]
fn let_inside_if_in_loop(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut result: u64 = 0;

    while i < n {
        if i > 0 {
            let x: u64 = i * 2;  // let binding inside if
            result = result + x;
        }
        i = i + 1;
    }

    result
}

#[test]
fn test_let_inside_if_in_loop() {
    // i=0: skipped (i>0 false)
    // i=1: x=2, result=2
    // i=2: x=4, result=6
    // i=3: x=6, result=12
    assert_eq!(let_inside_if_in_loop(4), 12);
}

/// CRITICAL: let = if-else expression INSIDE if branch in loop
#[vm_protect(level = "debug")]
fn let_if_expr_inside_if_in_loop(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut result: u64 = 0;

    while i < n {
        if i > 0 {
            // This is the exact failing pattern
            let x: u64 = if i > 2 { i * 3 } else { i * 2 };
            result = result + x;
        }
        i = i + 1;
    }

    result
}

#[test]
fn test_let_if_expr_inside_if_in_loop() {
    // i=0: skipped
    // i=1: x = 1*2 = 2, result=2
    // i=2: x = 2*2 = 4, result=6
    // i=3: x = 3*3 = 9, result=15
    assert_eq!(let_if_expr_inside_if_in_loop(4), 15);
}

/// CRITICAL: let binding in BOTH branches of if-else in loop
#[vm_protect(level = "debug")]
fn let_in_both_branches(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut result: u64 = 0;

    while i < n {
        if i > 0 {
            if i > 2 {
                let x: u64 = i * 3;
                result = result + x;
            } else {
                let y: u64 = i * 2;
                result = result + y;
            }
        }
        i = i + 1;
    }

    result
}

#[test]
fn test_let_in_both_branches() {
    // i=0: skipped
    // i=1: y = 2, result=2
    // i=2: y = 4, result=6
    // i=3: x = 9, result=15
    assert_eq!(let_in_both_branches(4), 15);
}

/// EXACT copy of failing v2 pattern
#[vm_protect(level = "debug")]
fn exact_v2_copy(packed_deltas: u64, max_delta: u64) -> u64 {
    let mut remaining = packed_deltas;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < 4 {
        let current = remaining & 0xFF;
        if i > 0 {
            if current > prev {
                let delta = current - prev;
                if delta > max_delta {
                    return 1;
                }
            } else {
                let delta = prev - current;
                if delta > max_delta {
                    return 1;
                }
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_exact_v2() {
    // [10, 12, 11, 13] -> deltas: 2, 1, 2 - all < 5
    let smooth = 10 | (12 << 8) | (11 << 16) | (13 << 24);
    assert_eq!(exact_v2_copy(smooth, 5), 0);
}

/// Now with 8 iterations like original
#[vm_protect(level = "debug")]
fn v2_with_8_iter(packed_deltas: u64, max_delta: u64) -> u64 {
    let mut remaining = packed_deltas;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < 8 {
        let current = remaining & 0xFF;

        // Stop at trailing zeros
        if current == 0 {
            let rest = remaining >> 8;
            if rest == 0 {
                return 0;
            }
        }

        if i > 0 {
            if current > prev {
                let delta = current - prev;
                if delta > max_delta {
                    return 1;
                }
            } else {
                let delta = prev - current;
                if delta > max_delta {
                    return 1;
                }
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_v2_with_8_iter() {
    let smooth = 10 | (12 << 8) | (11 << 16) | (13 << 24) | (12u64 << 32);
    assert_eq!(v2_with_8_iter(smooth, 5), 0);
}

/// Test iteration count where it breaks
#[vm_protect(level = "debug")]
fn test_iter_count(n: u64, packed: u64, max_delta: u64) -> u64 {
    let mut remaining = packed;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < n {
        let current = remaining & 0xFF;
        if i > 0 {
            if current > prev {
                let delta = current - prev;
                if delta > max_delta {
                    return 1;
                }
            } else {
                let delta = prev - current;
                if delta > max_delta {
                    return 1;
                }
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_iter_counts() {
    // Fix: Need all 8 bytes to have valid deltas
    // Using [10, 12, 11, 13, 12, 11, 13, 12] - all deltas ≤ 2
    let smooth = 10u64 | (12u64 << 8) | (11u64 << 16) | (13u64 << 24) |
                 (12u64 << 32) | (11u64 << 40) | (13u64 << 48) | (12u64 << 56);

    // All iterations should pass with max_delta=5
    assert_eq!(test_iter_count(1, smooth, 5), 0, "n=1 failed");
    assert_eq!(test_iter_count(2, smooth, 5), 0, "n=2 failed");
    assert_eq!(test_iter_count(3, smooth, 5), 0, "n=3 failed");
    assert_eq!(test_iter_count(4, smooth, 5), 0, "n=4 failed");
    assert_eq!(test_iter_count(5, smooth, 5), 0, "n=5 failed");
    assert_eq!(test_iter_count(6, smooth, 5), 0, "n=6 failed");
    assert_eq!(test_iter_count(7, smooth, 5), 0, "n=7 failed");
    assert_eq!(test_iter_count(8, smooth, 5), 0, "n=8 failed");
}

/// Simplest possible failing case - no delta variable
#[vm_protect(level = "debug")]
fn minimal_loop(n: u64) -> u64 {
    let mut i: u64 = 0;
    while i < n {
        if i > 0 {
            if i > 2 {
                // Just return, no let
                return 99;
            }
        }
        i = i + 1;
    }
    0
}

#[test]
fn test_minimal_loop() {
    assert_eq!(minimal_loop(2), 0);  // i goes 0,1 - never hits i>2
    assert_eq!(minimal_loop(4), 99); // i goes 0,1,2,3 - hits i>2 at i=3
    assert_eq!(minimal_loop(8), 99); // Should also work
}

/// Test with different variable names in each branch
#[vm_protect(level = "debug")]
fn diff_names_in_branches(n: u64, packed: u64, max_delta: u64) -> u64 {
    let mut remaining = packed;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < n {
        let current = remaining & 0xFF;
        if i > 0 {
            if current > prev {
                let delta_pos = current - prev;  // Different name
                if delta_pos > max_delta {
                    return 1;
                }
            } else {
                let delta_neg = prev - current;  // Different name
                if delta_neg > max_delta {
                    return 1;
                }
            }
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_diff_names() {
    // Full 8 bytes: [10, 12, 11, 13, 12, 11, 13, 12] - all deltas <= 2
    let smooth = 10u64 | (12u64 << 8) | (11u64 << 16) | (13u64 << 24) |
                 (12u64 << 32) | (11u64 << 40) | (13u64 << 48) | (12u64 << 56);
    assert_eq!(diff_names_in_branches(6, smooth, 5), 0, "diff names n=6");
    assert_eq!(diff_names_in_branches(8, smooth, 5), 0, "diff names n=8");
}

/// Super minimal - just counting with if
#[vm_protect(level = "debug")]
fn count_with_if(n: u64) -> u64 {
    let mut i: u64 = 0;
    let mut count: u64 = 0;
    while i < n {
        if i > 0 {
            count = count + 1;
        }
        i = i + 1;
    }
    count
}

#[test]
fn test_count_with_if() {
    // n=0: loop never runs -> 0
    // n=1: i=0 (skip), -> 0
    // n=2: i=0 (skip), i=1 (count=1) -> 1
    // n=8: i=0 (skip), i=1..7 (count=7) -> 7
    assert_eq!(count_with_if(0), 0);
    assert_eq!(count_with_if(1), 0);
    assert_eq!(count_with_if(2), 1);
    assert_eq!(count_with_if(8), 7);
}

/// Test with variable comparison in loop
#[vm_protect(level = "debug")]
fn compare_vars_in_loop(n: u64, packed: u64) -> u64 {
    let mut remaining = packed;
    let mut prev: u64 = 0;
    let mut i: u64 = 0;

    while i < n {
        let current = remaining & 0xFF;
        if current > prev {
            // Just return early if current > prev
            return 1;
        }
        prev = current;
        remaining = remaining >> 8;
        i = i + 1;
    }

    0
}

#[test]
fn test_compare_vars_in_loop() {
    // First iteration: prev=0, current=10, 10>0=true -> returns 1
    // So test must account for initial prev=0
    let decreasing = 10 | (5 << 8) | (3 << 16) | (1 << 24);
    // First byte 10 > 0 (initial prev), so immediately returns 1
    assert_eq!(compare_vars_in_loop(4, decreasing), 1);

    // Test with 0 as first byte - then check if second > first
    // [0, 5, 3, 1] - 0 not > 0, then 5 > 0 = true -> returns 1
    let zero_first = 0 | (5 << 8) | (3 << 16) | (1 << 24);
    assert_eq!(compare_vars_in_loop(4, zero_first), 1);

    // [5, 3, 1, 0] - 5>0=true -> returns 1
    let normal = 5 | (3 << 8) | (1 << 16) | (0 << 24);
    assert_eq!(compare_vars_in_loop(4, normal), 1);
}
