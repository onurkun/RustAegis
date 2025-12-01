//! Tests for tuple operations in vm_protect

use aegis_vm::vm_protect;

// ============================================================================
// Basic Tuple Literals
// ============================================================================

/// Test unit tuple ()
#[test]
fn test_unit_tuple() {
    #[vm_protect(level = "debug")]
    fn unit_tuple() -> u64 {
        let _unit = ();
        42
    }

    assert_eq!(unit_tuple(), 42);
}

/// Test single-element tuple
#[test]
fn test_single_element_tuple() {
    #[vm_protect(level = "debug")]
    fn single_tuple() -> u64 {
        let t = (100,);
        t.0
    }

    assert_eq!(single_tuple(), 100);
}

/// Test two-element tuple
#[test]
fn test_two_element_tuple() {
    #[vm_protect(level = "debug")]
    fn pair_tuple() -> u64 {
        let pair = (10, 20);
        pair.0 + pair.1
    }

    assert_eq!(pair_tuple(), 30);
}

/// Test three-element tuple
#[test]
fn test_three_element_tuple() {
    #[vm_protect(level = "debug")]
    fn triple_tuple() -> u64 {
        let triple = (1, 2, 3);
        triple.0 + triple.1 + triple.2
    }

    assert_eq!(triple_tuple(), 6);
}

/// Test tuple with expressions
#[test]
fn test_tuple_with_expressions() {
    #[vm_protect(level = "debug")]
    fn expr_tuple() -> u64 {
        let a = 5;
        let b = 10;
        let t = (a + b, a * b);
        t.0 + t.1  // 15 + 50 = 65
    }

    assert_eq!(expr_tuple(), 65);
}

// ============================================================================
// Tuple Indexing
// ============================================================================

/// Test tuple index 0
#[test]
fn test_tuple_index_0() {
    #[vm_protect(level = "debug")]
    fn idx0() -> u64 {
        let t = (100, 200, 300);
        t.0
    }

    assert_eq!(idx0(), 100);
}

/// Test tuple index 1
#[test]
fn test_tuple_index_1() {
    #[vm_protect(level = "debug")]
    fn idx1() -> u64 {
        let t = (100, 200, 300);
        t.1
    }

    assert_eq!(idx1(), 200);
}

/// Test tuple index 2
#[test]
fn test_tuple_index_2() {
    #[vm_protect(level = "debug")]
    fn idx2() -> u64 {
        let t = (100, 200, 300);
        t.2
    }

    assert_eq!(idx2(), 300);
}

/// Test tuple index in expression
#[test]
fn test_tuple_index_in_expr() {
    #[vm_protect(level = "debug")]
    fn tuple_expr() -> u64 {
        let t = (3, 4);
        t.0 * t.0 + t.1 * t.1  // 9 + 16 = 25
    }

    assert_eq!(tuple_expr(), 25);
}

// ============================================================================
// Tuple Assignment
// ============================================================================

/// Test mutable tuple element assignment
#[test]
fn test_tuple_element_assignment() {
    #[vm_protect(level = "debug")]
    fn mutate_tuple() -> u64 {
        let mut t = (10, 20);
        t.0 = 100;
        t.0 + t.1  // 100 + 20 = 120
    }

    assert_eq!(mutate_tuple(), 120);
}

/// Test multiple tuple assignments
#[test]
fn test_tuple_multiple_assignments() {
    #[vm_protect(level = "debug")]
    fn swap_tuple() -> u64 {
        let mut t = (1, 2);
        let temp = t.0;
        t.0 = t.1;
        t.1 = temp;
        t.0 * 100 + t.1  // 2*100 + 1 = 201
    }

    assert_eq!(swap_tuple(), 201);
}

// ============================================================================
// Tuple Struct
// ============================================================================

/// Test basic tuple struct
#[test]
fn test_tuple_struct_basic() {
    #[vm_protect(level = "debug")]
    fn tuple_struct() -> u64 {
        struct Point(u64, u64);
        let p = Point(10, 20);
        p.0 + p.1
    }

    assert_eq!(tuple_struct(), 30);
}

/// Test tuple struct with three fields
#[test]
fn test_tuple_struct_three_fields() {
    #[vm_protect(level = "debug")]
    fn rgb() -> u64 {
        struct Color(u64, u64, u64);
        let c = Color(255, 128, 64);
        c.0 + c.1 + c.2
    }

    assert_eq!(rgb(), 447);
}

/// Test tuple struct field access
#[test]
fn test_tuple_struct_field_access() {
    #[vm_protect(level = "debug")]
    fn distance_sq() -> u64 {
        struct Point(u64, u64);
        let p = Point(3, 4);
        p.0 * p.0 + p.1 * p.1  // 9 + 16 = 25
    }

    assert_eq!(distance_sq(), 25);
}

/// Test mutable tuple struct
#[test]
fn test_tuple_struct_mutation() {
    #[vm_protect(level = "debug")]
    fn mutate() -> u64 {
        struct Pair(u64, u64);
        let mut p = Pair(1, 2);
        p.0 = 100;
        p.1 = 200;
        p.0 + p.1
    }

    assert_eq!(mutate(), 300);
}

// ============================================================================
// Tuple in Control Flow
// ============================================================================

/// Test tuple in if condition
#[test]
fn test_tuple_in_condition() {
    #[vm_protect(level = "debug")]
    fn cond_tuple() -> u64 {
        let t = (10, 5);
        if t.0 > t.1 {
            t.0 - t.1
        } else {
            t.1 - t.0
        }
    }

    assert_eq!(cond_tuple(), 5);
}

/// Test tuple in loop
#[test]
fn test_tuple_in_loop() {
    #[vm_protect(level = "debug")]
    fn loop_tuple() -> u64 {
        let mut t = (0, 0);
        for i in 0..5 {
            t.0 = t.0 + i;
            t.1 = t.1 + 1;
        }
        t.0 + t.1 * 100  // 10 + 5*100 = 510
    }

    assert_eq!(loop_tuple(), 510);
}

// ============================================================================
// Protection Levels
// ============================================================================

/// Test tuple with standard protection
#[test]
fn test_tuple_standard() {
    #[vm_protect(level = "standard")]
    fn std_tuple() -> u64 {
        let t = (0xCAFE, 0xBABE);
        t.0 ^ t.1
    }

    assert_eq!(std_tuple(), 0xCAFE ^ 0xBABE);
}

/// Test tuple with paranoid protection
#[test]
fn test_tuple_paranoid() {
    #[vm_protect(level = "paranoid")]
    fn paranoid_tuple() -> u64 {
        let t = (123, 456);
        t.0 + t.1
    }

    assert_eq!(paranoid_tuple(), 579);
}

// ============================================================================
// Nested Tuples
// ============================================================================

/// Test nested tuple creation
#[test]
fn test_nested_tuple_basic() {
    #[vm_protect(level = "debug")]
    fn nested() -> u64 {
        let t = ((1, 2), 3);
        t.1  // Access outer element
    }

    assert_eq!(nested(), 3);
}

/// Test nested tuple inner access
#[test]
fn test_nested_tuple_inner_access() {
    #[vm_protect(level = "debug")]
    fn nested_inner() -> u64 {
        let t = ((10, 20), 30);
        let inner = t.0;  // Get inner tuple
        inner.0 + inner.1  // 10 + 20 = 30
    }

    assert_eq!(nested_inner(), 30);
}

/// Test deeply nested tuple
#[test]
fn test_nested_tuple_deep() {
    #[vm_protect(level = "debug")]
    fn deep_nested() -> u64 {
        let t = (((1, 2), 3), 4);
        t.1  // 4
    }

    assert_eq!(deep_nested(), 4);
}

/// Test tuple with multiple nested tuples
#[test]
fn test_tuple_multiple_nested() {
    #[vm_protect(level = "debug")]
    fn multi_nested() -> u64 {
        let t = ((1, 2), (3, 4));
        let a = t.0;
        let b = t.1;
        a.0 + a.1 + b.0 + b.1  // 1 + 2 + 3 + 4 = 10
    }

    assert_eq!(multi_nested(), 10);
}

/// Test nested tuple with computation
#[test]
fn test_nested_tuple_compute() {
    #[vm_protect(level = "debug")]
    fn compute_nested() -> u64 {
        let point = (10, 20);
        let size = (100, 200);
        let rect = (point, size);

        let p = rect.0;
        let s = rect.1;
        p.0 + p.1 + s.0 + s.1  // 10 + 20 + 100 + 200 = 330
    }

    assert_eq!(compute_nested(), 330);
}

// ============================================================================
// Type-aware Tuple Tests
// ============================================================================

/// Test that tuple element types are tracked
#[test]
fn test_tuple_type_tracking() {
    #[vm_protect(level = "debug")]
    fn type_track() -> u64 {
        let t = (100, 200, 300);
        // All elements should be accessible
        t.0 + t.1 + t.2
    }

    assert_eq!(type_track(), 600);
}

/// Test tuple with variable elements
#[test]
fn test_tuple_variable_elements() {
    #[vm_protect(level = "debug")]
    fn var_elems() -> u64 {
        let a = 5;
        let b = 10;
        let c = 15;
        let t = (a, b, c);
        t.0 * t.1 + t.2  // 5 * 10 + 15 = 65
    }

    assert_eq!(var_elems(), 65);
}

// ============================================================================
// Complex Tuple Tests
// ============================================================================

/// Test tuple with many elements
#[test]
fn test_tuple_many_elements() {
    #[vm_protect(level = "debug")]
    fn many_elems() -> u64 {
        let t = (1, 2, 3, 4, 5, 6, 7, 8);
        t.0 + t.1 + t.2 + t.3 + t.4 + t.5 + t.6 + t.7
    }

    assert_eq!(many_elems(), 36);
}

/// Test multiple tuple instances
#[test]
fn test_tuple_multiple_instances() {
    #[vm_protect(level = "debug")]
    fn multi_tuples() -> u64 {
        let t1 = (10, 20);
        let t2 = (30, 40);
        let t3 = (50, 60);

        t1.0 + t2.1 + t3.0 + t1.1 + t2.0 + t3.1
    }

    // 10 + 40 + 50 + 20 + 30 + 60 = 210
    assert_eq!(multi_tuples(), 210);
}

/// Test tuple in complex arithmetic
#[test]
fn test_tuple_complex_arithmetic() {
    #[vm_protect(level = "debug")]
    fn complex_math() -> u64 {
        let t = (5, 3, 7);
        (t.0 * t.1 + t.2) * 2 + t.0
    }

    // (5 * 3 + 7) * 2 + 5 = 22 * 2 + 5 = 49
    assert_eq!(complex_math(), 49);
}

/// Test tuple bitwise operations
#[test]
fn test_tuple_bitwise() {
    #[vm_protect(level = "debug")]
    fn bitwise_ops() -> u64 {
        let t = (0xFF, 0x12AB, 4);
        (t.1 & t.0) | ((t.1 >> t.2) & t.0)
    }

    // (0x12AB & 0xFF) | ((0x12AB >> 4) & 0xFF)
    // = 0xAB | (0x12A & 0xFF)
    // = 0xAB | 0x2A
    // = 0xAB
    assert_eq!(bitwise_ops(), 0xAB);
}

/// Test tuple in nested loops
#[test]
fn test_tuple_nested_loops() {
    #[vm_protect(level = "debug")]
    fn nested_loop_tuple() -> u64 {
        let mut t = (0, 0, 0);  // x, y, total

        for i in 0..3 {
            t.0 = t.0 + 1;
            for j in 0..4 {
                t.1 = t.1 + 1;
                t.2 = t.2 + i + j;
            }
        }
        t.0 * 1000 + t.1 * 10 + t.2
    }

    // x = 3, y = 12, total = 30
    // Result: 3*1000 + 12*10 + 30 = 3150
    assert_eq!(nested_loop_tuple(), 3150);
}

/// Test tuple element conditional updates
#[test]
fn test_tuple_conditional_update() {
    #[vm_protect(level = "debug")]
    fn conditional() -> u64 {
        let mut t = (10, 1);  // value, flag

        if t.1 > 0 {
            t.0 = t.0 * 2;
        }

        if t.0 > 15 {
            t.1 = 0;
        }

        t.0 * 100 + t.1
    }

    // value = 20, flag = 0
    // Result: 2000
    assert_eq!(conditional(), 2000);
}

/// Test tuple with while loop
#[test]
fn test_tuple_while_loop() {
    #[vm_protect(level = "debug")]
    fn while_loop_tuple() -> u64 {
        let mut t = (0, 7, 50);  // current, step, limit
        let mut count = 0;

        while t.0 < t.2 {
            t.0 = t.0 + t.1;
            count = count + 1;
        }

        count * 1000 + t.0
    }

    // Steps: 0->7->14->21->28->35->42->49->56 (8 iterations)
    // Result: 8 * 1000 + 56 = 8056
    assert_eq!(while_loop_tuple(), 8056);
}

/// Test tuple elements in comparison chain
#[test]
fn test_tuple_comparison_chain() {
    #[vm_protect(level = "debug")]
    fn comparison_chain() -> u64 {
        let t = (10, 25, 50);  // min, val, max

        if t.1 >= t.0 {
            if t.1 <= t.2 {
                1  // in range
            } else {
                2  // above max
            }
        } else {
            3  // below min
        }
    }

    assert_eq!(comparison_chain(), 1);
}

/// Test tuple element reuse in expression
#[test]
fn test_tuple_element_reuse() {
    #[vm_protect(level = "debug")]
    fn elem_reuse() -> u64 {
        let t = (5,);
        t.0 + t.0 * t.0 + t.0 * t.0 * t.0
    }

    // 5 + 25 + 125 = 155
    assert_eq!(elem_reuse(), 155);
}

/// Test tuple with computed values from another tuple
#[test]
fn test_tuple_from_tuple() {
    #[vm_protect(level = "debug")]
    fn tuple_from_tuple() -> u64 {
        let input = (6, 7);
        let output = (input.0 + input.1, input.0 * input.1);

        output.0 + output.1
    }

    // sum = 13, prod = 42
    // Result: 55
    assert_eq!(tuple_from_tuple(), 55);
}

/// Test tuple element swapping
#[test]
fn test_tuple_swap() {
    #[vm_protect(level = "debug")]
    fn swap() -> u64 {
        let mut t = (100, 200);

        let temp = t.0;
        t.0 = t.1;
        t.1 = temp;

        t.0 * 1000 + t.1
    }

    // After swap: (200, 100)
    // Result: 200100
    assert_eq!(swap(), 200100);
}

/// Test fibonacci using tuple
#[test]
fn test_tuple_fibonacci() {
    #[vm_protect(level = "debug")]
    fn fib_tuple() -> u64 {
        let mut f = (0, 1);  // prev, curr

        for _i in 0..10 {
            let next = f.0 + f.1;
            f.0 = f.1;
            f.1 = next;
        }

        f.1
    }

    // After 10 iterations: 1,1,2,3,5,8,13,21,34,55,89
    assert_eq!(fib_tuple(), 89);
}

/// Test tuple accumulator pattern
#[test]
fn test_tuple_accumulator() {
    #[vm_protect(level = "debug")]
    fn accumulator() -> u64 {
        let mut stats = (0, 0, 999, 0);  // sum, count, min, max

        let values = [10, 25, 5, 30, 15];

        for i in 0..5 {
            let v = values[i];
            stats.0 = stats.0 + v;
            stats.1 = stats.1 + 1;
            if v < stats.2 {
                stats.2 = v;
            }
            if v > stats.3 {
                stats.3 = v;
            }
        }

        stats.0 + stats.1 * 100 + stats.2 * 10 + stats.3
    }

    // sum = 85, count = 5, min = 5, max = 30
    // Result: 85 + 500 + 50 + 30 = 665
    assert_eq!(accumulator(), 665);
}

/// Test tuple with break in loop
#[test]
fn test_tuple_loop_break() {
    #[vm_protect(level = "debug")]
    fn loop_break() -> u64 {
        let mut search = (25, 999);  // target, found_at
        let arr = [10, 15, 20, 25, 30, 35];

        for i in 0..6 {
            if arr[i] == search.0 {
                search.1 = i as u64;
                break;
            }
        }

        search.1
    }

    // Found 25 at index 3
    assert_eq!(loop_break(), 3);
}

/// Test tuple with continue in loop
#[test]
fn test_tuple_loop_continue() {
    #[vm_protect(level = "debug")]
    fn loop_continue() -> u64 {
        let mut filter = (15, 0);  // threshold, sum
        let arr = [10, 20, 5, 25, 12, 30];

        for i in 0..6 {
            if arr[i] < filter.0 {
                continue;
            }
            filter.1 = filter.1 + arr[i];
        }

        filter.1
    }

    // Skip 10, 5, 12 (< 15)
    // Sum 20 + 25 + 30 = 75
    assert_eq!(loop_continue(), 75);
}

/// Test deeply nested tuple extraction
#[test]
fn test_tuple_deep_extraction() {
    #[vm_protect(level = "debug")]
    fn deep_extract() -> u64 {
        let deep = (((1, 2), (3, 4)), ((5, 6), (7, 8)));

        let outer1 = deep.0;
        let outer2 = deep.1;
        let inner1 = outer1.0;
        let inner2 = outer1.1;
        let inner3 = outer2.0;
        let inner4 = outer2.1;

        inner1.0 + inner1.1 + inner2.0 + inner2.1 +
        inner3.0 + inner3.1 + inner4.0 + inner4.1
    }

    // 1+2+3+4+5+6+7+8 = 36
    assert_eq!(deep_extract(), 36);
}

/// Test tuple in chained computation
#[test]
fn test_tuple_chained_computation() {
    #[vm_protect(level = "debug")]
    fn chained() -> u64 {
        let a = (1, 2);
        let b = (a.0 + a.1, a.0 * a.1);  // (3, 2)
        let c = (b.0 + b.1, b.0 * b.1);  // (5, 6)
        let d = (c.0 + c.1, c.0 * c.1);  // (11, 30)

        d.0 + d.1
    }

    // Result: 11 + 30 = 41
    assert_eq!(chained(), 41);
}

/// Test tuple with power-like computation
#[test]
fn test_tuple_power() {
    #[vm_protect(level = "debug")]
    fn power() -> u64 {
        let mut t = (2, 1);  // base, result

        for _i in 0..10 {
            t.1 = t.1 * t.0;
        }

        t.1
    }

    // 2^10 = 1024
    assert_eq!(power(), 1024);
}

/// Test tuple GCD-like algorithm
#[test]
fn test_tuple_gcd() {
    #[vm_protect(level = "debug")]
    fn gcd() -> u64 {
        let mut t = (48, 18);  // a, b

        while t.1 > 0 {
            let temp = t.1;
            t.1 = t.0 % t.1;
            t.0 = temp;
        }

        t.0
    }

    // GCD(48, 18) = 6
    assert_eq!(gcd(), 6);
}

/// Test tuple struct with complex operations
#[test]
fn test_tuple_struct_complex() {
    #[vm_protect(level = "debug")]
    fn complex_tuple_struct() -> u64 {
        struct Vector3(u64, u64, u64);

        let v1 = Vector3(1, 2, 3);
        let v2 = Vector3(4, 5, 6);

        // Dot product
        v1.0 * v2.0 + v1.1 * v2.1 + v1.2 * v2.2
    }

    // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_eq!(complex_tuple_struct(), 32);
}

/// Test multiple tuple structs with computation
#[test]
fn test_tuple_structs_interaction() {
    #[vm_protect(level = "debug")]
    fn interaction() -> u64 {
        struct Point(u64, u64);
        struct Size(u64, u64);

        let p = Point(10, 20);
        let s = Size(100, 50);

        // Area calculation: (p.x + s.width) * (p.y + s.height)
        // But we only have integers, so compute differently
        p.0 * s.0 + p.1 * s.1
    }

    // 10*100 + 20*50 = 1000 + 1000 = 2000
    assert_eq!(interaction(), 2000);
}

/// Test tuple with all protection levels
#[test]
fn test_tuple_protection_levels() {
    #[vm_protect(level = "debug")]
    fn debug_tuple() -> u64 {
        let t = ((1, 2), (3, 4));
        let a = t.0;
        let b = t.1;
        a.0 + a.1 + b.0 + b.1
    }

    #[vm_protect(level = "standard")]
    fn standard_tuple() -> u64 {
        let t = ((1, 2), (3, 4));
        let a = t.0;
        let b = t.1;
        a.0 + a.1 + b.0 + b.1
    }

    #[vm_protect(level = "paranoid")]
    fn paranoid_tuple() -> u64 {
        let t = ((1, 2), (3, 4));
        let a = t.0;
        let b = t.1;
        a.0 + a.1 + b.0 + b.1
    }

    assert_eq!(debug_tuple(), 10);
    assert_eq!(standard_tuple(), 10);
    assert_eq!(paranoid_tuple(), 10);
}

/// Test tuple in factorial-like computation
#[test]
fn test_tuple_factorial() {
    #[vm_protect(level = "debug")]
    fn factorial() -> u64 {
        let mut t = (1, 1);  // i, result

        while t.0 <= 10 {
            t.1 = t.1 * t.0;
            t.0 = t.0 + 1;
        }

        t.1
    }

    // 10! = 3628800
    assert_eq!(factorial(), 3628800);
}

/// Test tuple collatz-like sequence
#[test]
fn test_tuple_collatz_steps() {
    #[vm_protect(level = "debug")]
    fn collatz_steps() -> u64 {
        let mut t = (27, 0);  // n, steps

        while t.0 > 1 {
            if t.0 % 2 == 0 {
                t.0 = t.0 / 2;
            } else {
                t.0 = t.0 * 3 + 1;
            }
            t.1 = t.1 + 1;
        }

        t.1
    }

    // Collatz(27) takes 111 steps to reach 1
    assert_eq!(collatz_steps(), 111);
}
