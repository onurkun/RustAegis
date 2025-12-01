//! Edge case tests for loop constructs
//! Tests empty bodies, break/continue edge cases, etc.

use aegis_vm_macro::vm_protect;

// ============================================================================
// EMPTY BODY TESTS - These would cause stack underflow before the fix
// ============================================================================

/// While with empty body (condition never true)
#[vm_protect(level = "debug")]
fn while_empty_body_never_runs(n: u64) -> u64 {
    // This loop body is empty but condition is never true
    while n > 1000 {}
    42
}

/// While with empty body that runs once (would stack underflow before fix)
#[vm_protect(level = "debug")]
fn while_empty_body_runs_once() -> u64 {
    let mut x = 1u64;
    while x > 0 {
        // Empty body - would cause stack underflow before fix
        x = 0;
    }
    100
}

/// For with empty body
#[vm_protect(level = "debug")]
fn for_empty_body() -> u64 {
    for _i in 0..5 {
        // Empty body
    }
    200
}

/// Infinite loop with immediate break (empty body effectively)
#[vm_protect(level = "debug")]
fn loop_immediate_break_empty() -> u64 {
    loop {
        break;
    }
    300
}

// ============================================================================
// BODY WITH ONLY STATEMENTS (no expression value)
// ============================================================================

/// While body with only let statement
#[vm_protect(level = "debug")]
fn while_only_let_stmt() -> u64 {
    let mut count = 0u64;
    let mut i = 0u64;
    while i < 3 {
        let _temp = i * 2;  // Only a let statement
        count += 1;
        i += 1;
    }
    count
}

/// For body with only assignment
#[vm_protect(level = "debug")]
fn for_only_assignment() -> u64 {
    let mut sum = 0u64;
    for i in 0..5 {
        sum += i;  // Only assignment, no expression result
    }
    sum
}

// ============================================================================
// NESTED EMPTY LOOPS
// ============================================================================

/// Nested while with empty inner body
#[vm_protect(level = "debug")]
fn nested_while_empty_inner() -> u64 {
    let mut outer = 0u64;
    let mut i = 0u64;
    while i < 3 {
        let mut j = 0u64;
        while j < 2 {
            // Empty inner body
            j += 1;
        }
        outer += 1;
        i += 1;
    }
    outer
}

/// Nested for with empty bodies
#[vm_protect(level = "debug")]
fn nested_for_empty_both() -> u64 {
    let mut count = 0u64;
    for _i in 0..3 {
        for _j in 0..2 {
            // Empty
        }
        count += 1;
    }
    count
}

// ============================================================================
// BREAK/CONTINUE IN EDGE CASES
// ============================================================================

/// Break immediately in loop
#[vm_protect(level = "debug")]
fn break_first_thing() -> u64 {
    let mut x = 0u64;
    loop {
        break;
        #[allow(unreachable_code)]
        {
            x = 999;  // Never reached
        }
    }
    x
}

/// Continue skipping to next iteration
#[vm_protect(level = "debug")]
fn continue_skip_all() -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < 5 {
        i += 1;
        continue;  // Skip everything below
        #[allow(unreachable_code)]
        {
            sum += 100;  // Never reached
        }
    }
    sum
}

/// Multiple breaks in different branches
#[vm_protect(level = "debug")]
fn multiple_breaks(n: u64) -> u64 {
    let mut i = 0u64;
    loop {
        if i >= n {
            break;
        }
        if i >= 100 {
            break;
        }
        i += 1;
    }
    i
}

// ============================================================================
// COMPLEX CONTROL FLOW
// ============================================================================

/// Early return from nested loops
#[vm_protect(level = "debug")]
fn early_return_nested() -> u64 {
    for i in 0..10 {
        for j in 0..10 {
            if i * j == 15 {
                return i * 100 + j;  // Return when i*j = 15 (3*5 or 5*3)
            }
        }
    }
    0
}

/// While with return inside
#[vm_protect(level = "debug")]
fn while_with_return(limit: u64) -> u64 {
    let mut i = 0u64;
    while i < 1000 {
        if i >= limit {
            return i;
        }
        i += 1;
    }
    i
}

// ============================================================================
// TEST CASES
// ============================================================================

#[test]
fn test_while_empty_body_never_runs() {
    assert_eq!(while_empty_body_never_runs(0), 42);
    assert_eq!(while_empty_body_never_runs(500), 42);
}

#[test]
fn test_while_empty_body_runs_once() {
    // This was the critical bug - empty body would cause stack underflow
    assert_eq!(while_empty_body_runs_once(), 100);
}

#[test]
fn test_for_empty_body() {
    // Empty for body should not crash
    assert_eq!(for_empty_body(), 200);
}

#[test]
fn test_loop_immediate_break_empty() {
    assert_eq!(loop_immediate_break_empty(), 300);
}

#[test]
fn test_while_only_let_stmt() {
    assert_eq!(while_only_let_stmt(), 3);
}

#[test]
fn test_for_only_assignment() {
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(for_only_assignment(), 10);
}

#[test]
fn test_nested_while_empty_inner() {
    assert_eq!(nested_while_empty_inner(), 3);
}

#[test]
fn test_nested_for_empty_both() {
    assert_eq!(nested_for_empty_both(), 3);
}

#[test]
fn test_break_first_thing() {
    assert_eq!(break_first_thing(), 0);
}

#[test]
fn test_continue_skip_all() {
    assert_eq!(continue_skip_all(), 0);
}

#[test]
fn test_multiple_breaks() {
    assert_eq!(multiple_breaks(5), 5);
    assert_eq!(multiple_breaks(50), 50);
    assert_eq!(multiple_breaks(200), 100);  // Hits second break
}

#[test]
fn test_early_return_nested() {
    // 3 * 5 = 15, so return 3*100 + 5 = 305
    assert_eq!(early_return_nested(), 305);
}

#[test]
fn test_while_with_return() {
    assert_eq!(while_with_return(5), 5);
    assert_eq!(while_with_return(100), 100);
}

// ============================================================================
// MEMORY CLEANUP ON EARLY EXIT TESTS
// Tests that heap memory is properly freed on break, continue, and return
// ============================================================================

/// Test that structs are cleaned up on break
#[vm_protect(level = "debug")]
fn break_with_struct_cleanup() -> u64 {
    struct Data { x: u64, y: u64 }

    let mut result = 0;
    for i in 0..10 {
        let d = Data { x: i, y: i * 2 };  // Allocates on heap
        result = result + d.x;
        if i == 5 {
            break;  // Should free 'd' before jumping
        }
        // d would normally be freed here at end of scope
    }
    result  // 0 + 1 + 2 + 3 + 4 + 5 = 15
}

/// Test that tuples are cleaned up on break
#[vm_protect(level = "debug")]
fn break_with_tuple_cleanup() -> u64 {
    let mut sum = 0;
    for i in 0..10 {
        let t = (i, i + 1, i + 2);  // Tuple on heap
        sum = sum + t.0 + t.1 + t.2;
        if i == 3 {
            break;  // Should free 't' before jumping
        }
    }
    sum  // (0+1+2) + (1+2+3) + (2+3+4) + (3+4+5) = 3 + 6 + 9 + 12 = 30
}

/// Test that continue properly cleans up inner scope
#[vm_protect(level = "debug")]
fn continue_with_struct_cleanup() -> u64 {
    struct Point { x: u64, y: u64 }

    let mut sum = 0;
    for i in 0..5 {
        if i % 2 == 0 {
            let p = Point { x: i, y: i };  // Only created for even i
            sum = sum + p.x;
            continue;  // Should free 'p' before jumping
        }
        sum = sum + 100;  // Add 100 for odd numbers
    }
    sum  // i=0: 0, i=1: 100, i=2: 2, i=3: 100, i=4: 4 = 206
}

/// Test that return properly cleans up all scopes
#[vm_protect(level = "debug")]
fn return_with_deep_cleanup() -> u64 {
    struct Outer { a: u64 }
    struct Inner { b: u64 }

    let outer = Outer { a: 10 };

    for i in 0..5 {
        let inner = Inner { b: i };

        if inner.b == 3 {
            // Return should clean up: inner, then outer
            return outer.a + inner.b;  // 10 + 3 = 13
        }
    }

    outer.a  // Never reached
}

/// Test nested loops with break and struct
#[vm_protect(level = "debug")]
fn nested_break_cleanup() -> u64 {
    struct Counter { val: u64 }

    let mut total = 0;

    for i in 0..3 {
        let outer_c = Counter { val: i * 100 };

        for j in 0..5 {
            let inner_c = Counter { val: j };
            total = total + inner_c.val;

            if j == 2 {
                break;  // Should clean up inner_c, NOT outer_c
            }
        }

        total = total + outer_c.val;  // outer_c should still be valid
    }

    // Inner loops: (0+1+2) * 3 = 9
    // Outer: 0 + 100 + 200 = 300
    // Total: 309
    total
}

/// Test continue in nested loop with cleanup
#[vm_protect(level = "debug")]
fn nested_continue_cleanup() -> u64 {
    struct Val { n: u64 }

    let mut sum = 0;

    for i in 0..3 {
        let outer_v = Val { n: i };

        for j in 0..4 {
            if j % 2 == 1 {
                continue;  // Skip odd j, no struct to clean here
            }
            let inner_v = Val { n: j * 10 };
            sum = sum + inner_v.n;
        }

        sum = sum + outer_v.n;
    }

    // Inner: (0 + 20) * 3 = 60
    // Outer: 0 + 1 + 2 = 3
    // Total: 63
    sum
}

#[test]
fn test_break_with_struct_cleanup() {
    assert_eq!(break_with_struct_cleanup(), 15);
}

#[test]
fn test_break_with_tuple_cleanup() {
    assert_eq!(break_with_tuple_cleanup(), 30);
}

#[test]
fn test_continue_with_struct_cleanup() {
    assert_eq!(continue_with_struct_cleanup(), 206);
}

#[test]
fn test_return_with_deep_cleanup() {
    assert_eq!(return_with_deep_cleanup(), 13);
}

#[test]
fn test_nested_break_cleanup() {
    assert_eq!(nested_break_cleanup(), 309);
}

#[test]
fn test_nested_continue_cleanup() {
    assert_eq!(nested_continue_cleanup(), 63);
}
