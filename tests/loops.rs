//! Tests for loop constructs (while, for, loop with break/continue)

use aegis_vm_macro::vm_protect;

// ============================================================================
// WHILE LOOP TESTS
// ============================================================================

/// Simple while loop - count down
#[vm_protect(level = "debug")]
fn while_countdown(n: u64) -> u64 {
    let mut count = n;  // Copy to local since args are read-only
    let mut sum = 0u64;
    while count > 0 {
        sum += count;
        count -= 1;
    }
    sum
}

/// While loop with early exit condition
#[vm_protect(level = "debug")]
fn while_find_first_div_3(start: u64) -> u64 {
    let mut i = start;
    while i < start + 100 {
        if i % 3 == 0 {
            return i;
        }
        i += 1;
    }
    0
}

/// Nested while loops
#[vm_protect(level = "debug")]
fn while_nested(rows: u64, cols: u64) -> u64 {
    let mut total = 0u64;
    let mut i = 0u64;
    while i < rows {
        let mut j = 0u64;
        while j < cols {
            total += 1;
            j += 1;
        }
        i += 1;
    }
    total
}

// ============================================================================
// FOR LOOP TESTS
// ============================================================================

/// Simple for loop - sum range
#[vm_protect(level = "debug")]
fn for_sum_range(n: u64) -> u64 {
    let mut sum = 0u64;
    for i in 0..n {
        sum += i;
    }
    sum
}

/// For loop with inclusive range
#[vm_protect(level = "debug")]
fn for_sum_inclusive(n: u64) -> u64 {
    let mut sum = 0u64;
    for i in 1..=n {
        sum += i;
    }
    sum
}

/// Nested for loops
#[vm_protect(level = "debug")]
fn for_nested(outer: u64, inner: u64) -> u64 {
    let mut count = 0u64;
    for _i in 0..outer {
        for _j in 0..inner {
            count += 1;
        }
    }
    count
}

// ============================================================================
// INFINITE LOOP WITH BREAK TESTS
// ============================================================================

/// Loop with break
#[vm_protect(level = "debug")]
fn loop_with_break(limit: u64) -> u64 {
    let mut count = 0u64;
    loop {
        count += 1;
        if count >= limit {
            break;
        }
    }
    count
}

/// Loop with conditional break
#[vm_protect(level = "debug")]
fn loop_find_power_of_2(start: u64) -> u64 {
    let mut n = start;
    loop {
        // Check if power of 2: n & (n-1) == 0 for powers of 2 (and n > 0)
        // Simplified to avoid complex short-circuit issues
        if n > 0 {
            let check = n & (n - 1);
            if check == 0 {
                break;
            }
        }
        n += 1;
        if n > 1000 {
            return 0; // Safety limit
        }
    }
    n
}

// ============================================================================
// CONTINUE TESTS
// ============================================================================

/// While loop with continue - skip evens
#[vm_protect(level = "debug")]
fn while_skip_evens(n: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0u64;
    while i < n {
        i += 1;
        if i % 2 == 0 {
            continue;
        }
        sum += i;
    }
    sum
}

/// For loop with continue - skip multiples of 3
#[vm_protect(level = "debug")]
fn for_skip_mult_3(n: u64) -> u64 {
    let mut sum = 0u64;
    for i in 1..=n {
        if i % 3 == 0 {
            continue;
        }
        sum += i;
    }
    sum
}

// ============================================================================
// COMPOUND ASSIGNMENT TESTS
// ============================================================================

/// Test all compound assignment operators
#[vm_protect(level = "debug")]
fn compound_assignments(a: u64, b: u64) -> u64 {
    let mut result = a;
    result += b;      // Add
    result -= 1;      // Sub
    result *= 2;      // Mul
    result /= 2;      // Div
    result %= 100;    // Mod
    result ^= 0xFF;   // XOR
    result &= 0xFFFF; // AND
    result |= 0x100;  // OR
    result <<= 1;     // SHL
    result >>= 1;     // SHR
    result
}

// ============================================================================
// TEST CASES
// ============================================================================

#[test]
fn test_while_countdown() {
    // 5 + 4 + 3 + 2 + 1 = 15
    assert_eq!(while_countdown(5), 15);
    // 10 + 9 + ... + 1 = 55
    assert_eq!(while_countdown(10), 55);
    // Edge case: 0
    assert_eq!(while_countdown(0), 0);
}

#[test]
fn test_while_find_first_div_3() {
    assert_eq!(while_find_first_div_3(1), 3);
    assert_eq!(while_find_first_div_3(4), 6);
    assert_eq!(while_find_first_div_3(9), 9);
    assert_eq!(while_find_first_div_3(10), 12);
}

#[test]
fn test_while_nested() {
    assert_eq!(while_nested(3, 4), 12);
    assert_eq!(while_nested(5, 5), 25);
    assert_eq!(while_nested(1, 10), 10);
    assert_eq!(while_nested(0, 5), 0);
}

#[test]
fn test_for_sum_range() {
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(for_sum_range(5), 10);
    // 0 + 1 + ... + 9 = 45
    assert_eq!(for_sum_range(10), 45);
    // Edge case: empty range
    assert_eq!(for_sum_range(0), 0);
}

#[test]
fn test_for_sum_inclusive() {
    // 1 + 2 + 3 + 4 + 5 = 15
    assert_eq!(for_sum_inclusive(5), 15);
    // 1 + 2 + ... + 10 = 55
    assert_eq!(for_sum_inclusive(10), 55);
}

#[test]
fn test_for_nested() {
    assert_eq!(for_nested(3, 4), 12);
    assert_eq!(for_nested(5, 5), 25);
    assert_eq!(for_nested(2, 3), 6);
}

#[test]
fn test_loop_with_break() {
    assert_eq!(loop_with_break(5), 5);
    assert_eq!(loop_with_break(10), 10);
    assert_eq!(loop_with_break(1), 1);
}

#[test]
fn test_loop_find_power_of_2() {
    assert_eq!(loop_find_power_of_2(1), 1);
    assert_eq!(loop_find_power_of_2(3), 4);
    assert_eq!(loop_find_power_of_2(5), 8);
    assert_eq!(loop_find_power_of_2(9), 16);
}

#[test]
fn test_while_skip_evens() {
    // 1 + 3 + 5 = 9 (for n=5, i goes 1,2,3,4,5 but skips 2,4)
    assert_eq!(while_skip_evens(5), 9);
    // 1 + 3 + 5 + 7 + 9 = 25
    assert_eq!(while_skip_evens(10), 25);
}

#[test]
fn test_for_skip_mult_3() {
    // 1 + 2 + 4 + 5 + 7 + 8 + 10 = 37 (skip 3, 6, 9)
    assert_eq!(for_skip_mult_3(10), 37);
}

#[test]
fn test_compound_assignments() {
    // a=10, b=5
    // result = 10
    // += 5 => 15
    // -= 1 => 14
    // *= 2 => 28
    // /= 2 => 14
    // %= 100 => 14
    // ^= 0xFF => 14 ^ 255 = 241
    // &= 0xFFFF => 241
    // |= 0x100 => 241 | 256 = 497
    // <<= 1 => 994
    // >>= 1 => 497
    assert_eq!(compound_assignments(10, 5), 497);
}
