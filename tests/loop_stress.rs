//! Stress tests for loop constructs
//! Tests edge cases, deeply nested loops, large iterations, and various break/continue patterns

use aegis_vm_macro::vm_protect;

// ============================================================================
// EDGE CASES - EMPTY AND BOUNDARY CONDITIONS
// ============================================================================

/// While loop that never executes
#[vm_protect(level = "paranoid")]
fn while_never_runs(limit: u64) -> u64 {
    let mut count = 0u64;
    while limit > 1000 {  // Never true for inputs <= 1000
        count += 1;
    }
    count
}

/// For loop with empty range
#[vm_protect(level = "paranoid")]
fn for_empty_range() -> u64 {
    let mut sum = 0u64;
    for i in 5..3 {  // Empty range (5 > 3)
        sum += i;
    }
    sum
}

/// For loop with single iteration
#[vm_protect(level = "paranoid")]
fn for_single_iteration() -> u64 {
    let mut sum = 0u64;
    for i in 0..1 {
        sum += i + 100;
    }
    sum
}

/// Loop that breaks immediately
#[vm_protect(level = "paranoid")]
fn loop_immediate_break() -> u64 {
    let mut count = 0u64;
    loop {
        count += 1;
        break;
    }
    count
}

// ============================================================================
// LARGE ITERATION TESTS
// ============================================================================

/// While loop with 1000 iterations
#[vm_protect(level = "paranoid")]
fn while_1000_iterations() -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < 1000 {
        sum += i;
        i += 1;
    }
    sum
}

/// For loop with 1000 iterations
#[vm_protect(level = "paranoid")]
fn for_1000_iterations() -> u64 {
    let mut sum = 0u64;
    for i in 0..1000 {
        sum += i;
    }
    sum
}

/// For loop with inclusive 1000 range
#[vm_protect(level = "paranoid")]
fn for_inclusive_1000() -> u64 {
    let mut sum = 0u64;
    for i in 1..=1000 {
        sum += i;
    }
    sum
}

// ============================================================================
// DEEPLY NESTED LOOPS
// ============================================================================

/// 3-level nested while loops
#[vm_protect(level = "paranoid")]
fn while_nested_3_levels(a: u64, b: u64, c: u64) -> u64 {
    let mut total = 0u64;
    let mut i = 0u64;
    while i < a {
        let mut j = 0u64;
        while j < b {
            let mut k = 0u64;
            while k < c {
                total += 1;
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    total
}

/// 3-level nested for loops
#[vm_protect(level = "paranoid")]
fn for_nested_3_levels(a: u64, b: u64, c: u64) -> u64 {
    let mut total = 0u64;
    for _i in 0..a {
        for _j in 0..b {
            for _k in 0..c {
                total += 1;
            }
        }
    }
    total
}

/// Mixed loop types - for inside while
#[vm_protect(level = "paranoid")]
fn mixed_for_in_while(outer: u64, inner: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < outer {
        for j in 0..inner {
            sum += i + j;
        }
        i += 1;
    }
    sum
}

/// Mixed loop types - while inside for
#[vm_protect(level = "paranoid")]
fn mixed_while_in_for(outer: u64, inner: u64) -> u64 {
    let mut sum = 0u64;
    for i in 0..outer {
        let mut j = 0u64;
        while j < inner {
            sum += i + j;
            j += 1;
        }
    }
    sum
}

// ============================================================================
// BREAK IN NESTED LOOPS
// ============================================================================

/// Break from inner while loop only
#[vm_protect(level = "paranoid")]
fn while_break_inner(outer: u64, inner_limit: u64) -> u64 {
    let mut count = 0u64;
    let mut i = 0u64;
    while i < outer {
        let mut j = 0u64;
        while j < 100 {  // Would go to 100, but we break early
            j += 1;
            count += 1;
            if j >= inner_limit {
                break;  // Only breaks inner loop
            }
        }
        i += 1;
    }
    count
}

/// Break from inner for loop only
#[vm_protect(level = "paranoid")]
fn for_break_inner(outer: u64, inner_limit: u64) -> u64 {
    let mut count = 0u64;
    for _i in 0..outer {
        for j in 0..100 {
            count += 1;
            if j >= inner_limit {
                break;
            }
        }
    }
    count
}

/// Early return from nested loops
#[vm_protect(level = "paranoid")]
fn nested_return_when_found(target: u64) -> u64 {
    for i in 0..10 {
        for j in 0..10 {
            let product = (i + 1) * (j + 1);
            if product == target {
                return product;  // Return immediately when found
            }
        }
    }
    0  // Not found
}

// ============================================================================
// CONTINUE IN NESTED LOOPS
// ============================================================================

/// Continue in inner while loop
#[vm_protect(level = "paranoid")]
fn while_continue_inner(outer: u64, inner: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < outer {
        let mut j = 0u64;
        while j < inner {
            j += 1;
            if j % 2 == 0 {
                continue;  // Skip even numbers
            }
            sum += j;
        }
        i += 1;
    }
    sum
}

/// Continue in inner for loop
#[vm_protect(level = "paranoid")]
fn for_continue_inner(outer: u64, inner: u64) -> u64 {
    let mut sum = 0u64;
    for _i in 0..outer {
        for j in 1..=inner {
            if j % 3 == 0 {
                continue;  // Skip multiples of 3
            }
            sum += j;
        }
    }
    sum
}

// ============================================================================
// COMPLEX BREAK/CONTINUE PATTERNS
// ============================================================================

/// Multiple breaks in same loop with conditions
#[vm_protect(level = "paranoid")]
fn loop_multiple_break_conditions(n: u64) -> u64 {
    let mut count = 0u64;
    loop {
        count += 1;
        if count > n {
            break;
        }
        if count > 1000 {
            break;  // Safety limit
        }
    }
    count
}

/// Break after multiple continues
#[vm_protect(level = "paranoid")]
fn while_mixed_continue_break(n: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < 100 {
        i += 1;
        if i % 2 == 0 {
            continue;
        }
        if i % 3 == 0 {
            continue;
        }
        if i > n {
            break;
        }
        sum += i;
    }
    sum
}

/// Fibonacci with while loop
#[vm_protect(level = "paranoid")]
fn fibonacci_while(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 2u64;
    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i += 1;
    }
    b
}

/// Factorial with for loop
#[vm_protect(level = "paranoid")]
fn factorial_for(n: u64) -> u64 {
    let mut result = 1u64;
    for i in 1..=n {
        result *= i;
    }
    result
}

// ============================================================================
// COMPOUND ASSIGNMENT STRESS
// ============================================================================

/// All compound operators in a loop
#[vm_protect(level = "paranoid")]
fn compound_ops_in_loop(iterations: u64) -> u64 {
    let mut val = 1000u64;
    let mut i = 0u64;
    while i < iterations {
        val += 10;
        val -= 5;
        val *= 2;
        val /= 2;
        val %= 10000;
        i += 1;
    }
    val
}

/// XOR compound in loop
#[vm_protect(level = "paranoid")]
fn xor_in_loop(n: u64) -> u64 {
    let mut result = 0u64;
    for i in 0..n {
        result ^= i;
    }
    result
}

/// Bit shift in loop
#[vm_protect(level = "paranoid")]
fn shift_in_loop(n: u64) -> u64 {
    let mut val = 1u64;
    let mut i = 0u64;
    while i < n {
        val <<= 1;
        if val > 1000000 {
            val >>= 5;
        }
        i += 1;
    }
    val
}

// ============================================================================
// PROTECTION LEVEL EQUIVALENCE
// ============================================================================

#[vm_protect(level = "standard")]
fn for_sum_standard(n: u64) -> u64 {
    let mut sum = 0u64;
    for i in 0..n {
        sum += i;
    }
    sum
}

#[vm_protect(level = "paranoid")]
fn for_sum_paranoid(n: u64) -> u64 {
    let mut sum = 0u64;
    for i in 0..n {
        sum += i;
    }
    sum
}

#[vm_protect(level = "standard")]
fn while_nested_standard(a: u64, b: u64) -> u64 {
    let mut total = 0u64;
    let mut i = 0u64;
    while i < a {
        let mut j = 0u64;
        while j < b {
            total += 1;
            j += 1;
        }
        i += 1;
    }
    total
}

#[vm_protect(level = "paranoid")]
fn while_nested_paranoid(a: u64, b: u64) -> u64 {
    let mut total = 0u64;
    let mut i = 0u64;
    while i < a {
        let mut j = 0u64;
        while j < b {
            total += 1;
            j += 1;
        }
        i += 1;
    }
    total
}

// ============================================================================
// TEST CASES
// ============================================================================

#[test]
fn test_edge_while_never_runs() {
    assert_eq!(while_never_runs(0), 0);
    assert_eq!(while_never_runs(100), 0);
    assert_eq!(while_never_runs(1000), 0);
}

#[test]
fn test_edge_for_empty_range() {
    assert_eq!(for_empty_range(), 0);
}

#[test]
fn test_edge_for_single_iteration() {
    assert_eq!(for_single_iteration(), 100);
}

#[test]
fn test_edge_loop_immediate_break() {
    assert_eq!(loop_immediate_break(), 1);
}

#[test]
fn test_large_while_1000() {
    // Sum of 0..1000 = 999*1000/2 = 499500
    assert_eq!(while_1000_iterations(), 499500);
}

#[test]
fn test_large_for_1000() {
    assert_eq!(for_1000_iterations(), 499500);
}

#[test]
fn test_large_for_inclusive_1000() {
    // Sum of 1..=1000 = 1000*1001/2 = 500500
    assert_eq!(for_inclusive_1000(), 500500);
}

#[test]
fn test_nested_while_3_levels() {
    assert_eq!(while_nested_3_levels(3, 4, 5), 60);
    assert_eq!(while_nested_3_levels(2, 2, 2), 8);
    assert_eq!(while_nested_3_levels(0, 5, 5), 0);
}

#[test]
fn test_nested_for_3_levels() {
    assert_eq!(for_nested_3_levels(3, 4, 5), 60);
    assert_eq!(for_nested_3_levels(2, 2, 2), 8);
    assert_eq!(for_nested_3_levels(5, 0, 5), 0);
}

#[test]
fn test_mixed_for_in_while() {
    // outer=2, inner=3: i=0: 0+0+0+1+0+2=3, i=1: 1+0+1+1+1+2=6, total=9
    assert_eq!(mixed_for_in_while(2, 3), 9);
}

#[test]
fn test_mixed_while_in_for() {
    assert_eq!(mixed_while_in_for(2, 3), 9);
}

#[test]
fn test_while_break_inner() {
    // outer=3, inner_limit=5: 3 outer iterations, each breaking at j=5
    // count increments 5 times per outer iteration = 15
    assert_eq!(while_break_inner(3, 5), 15);
}

#[test]
fn test_for_break_inner() {
    // outer=3, inner_limit=4: breaks when j>=4, so j=0,1,2,3,4 (5 iterations each)
    assert_eq!(for_break_inner(3, 4), 15);
}

#[test]
fn test_nested_return_when_found() {
    assert_eq!(nested_return_when_found(6), 6);   // 2*3=6
    assert_eq!(nested_return_when_found(12), 12); // 3*4=12
    assert_eq!(nested_return_when_found(1), 1);   // 1*1=1
    assert_eq!(nested_return_when_found(999), 0); // Not found
}

#[test]
fn test_while_continue_inner() {
    // outer=2, inner=4: each inner loop sums odd numbers 1,3 = 4
    // 2 outer iterations = 8
    assert_eq!(while_continue_inner(2, 4), 8);
}

#[test]
fn test_for_continue_inner() {
    // outer=2, inner=6: sums 1,2,4,5 (skips 3,6) = 12 per outer = 24
    assert_eq!(for_continue_inner(2, 6), 24);
}

#[test]
fn test_loop_multiple_break_conditions() {
    assert_eq!(loop_multiple_break_conditions(5), 6);
    assert_eq!(loop_multiple_break_conditions(10), 11);
}

#[test]
fn test_while_mixed_continue_break() {
    // Numbers not divisible by 2 or 3 up to n: 1,5,7,11,13,...
    // For n=10: 1 + 5 + 7 = 13
    assert_eq!(while_mixed_continue_break(10), 13);
}

#[test]
fn test_fibonacci_while() {
    assert_eq!(fibonacci_while(0), 0);
    assert_eq!(fibonacci_while(1), 1);
    assert_eq!(fibonacci_while(2), 1);
    assert_eq!(fibonacci_while(10), 55);
    assert_eq!(fibonacci_while(20), 6765);
}

#[test]
fn test_factorial_for() {
    assert_eq!(factorial_for(0), 1);
    assert_eq!(factorial_for(1), 1);
    assert_eq!(factorial_for(5), 120);
    assert_eq!(factorial_for(10), 3628800);
}

#[test]
fn test_compound_ops_in_loop() {
    // Verify it doesn't crash and produces consistent results
    let result1 = compound_ops_in_loop(10);
    let result2 = compound_ops_in_loop(10);
    assert_eq!(result1, result2);
}

#[test]
fn test_xor_in_loop() {
    assert_eq!(xor_in_loop(0), 0);
    assert_eq!(xor_in_loop(1), 0);
    assert_eq!(xor_in_loop(2), 0 ^ 1);
    assert_eq!(xor_in_loop(5), 0 ^ 1 ^ 2 ^ 3 ^ 4);
}

#[test]
fn test_shift_in_loop() {
    // Verify consistent results
    let result1 = shift_in_loop(20);
    let result2 = shift_in_loop(20);
    assert_eq!(result1, result2);
    assert!(result1 > 0);  // Should produce some value
}

#[test]
fn test_protection_level_equivalence_for() {
    for n in [0, 1, 5, 10, 50, 100] {
        assert_eq!(for_sum_standard(n), for_sum_paranoid(n), "Mismatch at n={}", n);
    }
}

#[test]
fn test_protection_level_equivalence_while() {
    for a in [0, 1, 3, 5] {
        for b in [0, 1, 3, 5] {
            assert_eq!(
                while_nested_standard(a, b),
                while_nested_paranoid(a, b),
                "Mismatch at a={}, b={}", a, b
            );
        }
    }
}

#[test]
fn test_determinism() {
    // Run the same loop multiple times to ensure deterministic results
    for _ in 0..10 {
        assert_eq!(for_1000_iterations(), 499500);
        assert_eq!(while_1000_iterations(), 499500);
        assert_eq!(fibonacci_while(20), 6765);
    }
}

#[test]
fn test_loop_with_zero_iterations() {
    assert_eq!(for_nested_3_levels(0, 0, 0), 0);
    assert_eq!(while_nested_3_levels(0, 0, 0), 0);
    assert_eq!(mixed_for_in_while(0, 10), 0);
    assert_eq!(mixed_while_in_for(10, 0), 0);
}
