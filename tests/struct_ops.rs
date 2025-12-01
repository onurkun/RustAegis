//! Tests for struct operations in vm_protect

use aegis_vm::vm_protect;

/// Test basic struct creation and field access
#[test]
fn test_struct_basic() {
    #[vm_protect(level = "debug")]
    fn create_struct() -> u64 {
        struct Point {
            x: u64,
            y: u64,
        }

        let p = Point { x: 10, y: 20 };
        p.x + p.y
    }

    assert_eq!(create_struct(), 30);
}

/// Test struct with more fields
#[test]
fn test_struct_three_fields() {
    #[vm_protect(level = "debug")]
    fn three_fields() -> u64 {
        struct Triple {
            a: u64,
            b: u64,
            c: u64,
        }

        let t = Triple { a: 1, b: 2, c: 3 };
        t.a + t.b + t.c
    }

    assert_eq!(three_fields(), 6);
}

/// Test field assignment
#[test]
fn test_struct_field_assignment() {
    #[vm_protect(level = "debug")]
    fn modify_struct() -> u64 {
        struct Counter {
            value: u64,
        }

        let mut c = Counter { value: 0 };
        c.value = 42;
        c.value
    }

    assert_eq!(modify_struct(), 42);
}

/// Test multiple field assignments
#[test]
fn test_struct_multiple_assignments() {
    #[vm_protect(level = "debug")]
    fn swap_fields() -> u64 {
        struct Pair {
            first: u64,
            second: u64,
        }

        let mut p = Pair { first: 10, second: 20 };
        let temp = p.first;
        p.first = p.second;
        p.second = temp;
        p.first + p.second * 100
    }

    // After swap: first=20, second=10
    // Result: 20 + 10*100 = 1020
    assert_eq!(swap_fields(), 1020);
}

/// Test struct in expressions
#[test]
fn test_struct_in_expression() {
    #[vm_protect(level = "debug")]
    fn distance_squared() -> u64 {
        struct Point {
            x: u64,
            y: u64,
        }

        let p = Point { x: 3, y: 4 };
        p.x * p.x + p.y * p.y
    }

    assert_eq!(distance_squared(), 25);
}

/// Test struct with field access in condition
#[test]
fn test_struct_in_condition() {
    #[vm_protect(level = "debug")]
    fn check_threshold() -> u64 {
        struct Measurement {
            value: u64,
            threshold: u64,
        }

        let m = Measurement { value: 75, threshold: 50 };
        if m.value > m.threshold {
            1
        } else {
            0
        }
    }

    assert_eq!(check_threshold(), 1);
}

/// Test struct in loop
#[test]
fn test_struct_in_loop() {
    #[vm_protect(level = "debug")]
    fn accumulate() -> u64 {
        struct Accumulator {
            sum: u64,
            count: u64,
        }

        let mut acc = Accumulator { sum: 0, count: 0 };

        for i in 0..5 {
            acc.sum = acc.sum + i;
            acc.count = acc.count + 1;
        }

        acc.sum + acc.count * 1000
    }

    // sum = 0+1+2+3+4 = 10, count = 5
    // Result: 10 + 5*1000 = 5010
    assert_eq!(accumulate(), 5010);
}

/// Test multiple structs
#[test]
fn test_multiple_structs() {
    #[vm_protect(level = "debug")]
    fn two_structs() -> u64 {
        struct PointA {
            x: u64,
            y: u64,
        }

        struct PointB {
            a: u64,
            b: u64,
        }

        let p1 = PointA { x: 10, y: 20 };
        let p2 = PointB { a: 100, b: 200 };

        p1.x + p1.y + p2.a + p2.b
    }

    assert_eq!(two_structs(), 330);
}

/// Test unit struct (no fields)
#[test]
fn test_unit_struct() {
    #[vm_protect(level = "debug")]
    fn unit_struct() -> u64 {
        struct Marker;
        let _m = Marker;  // Unit struct instantiation without braces
        42
    }

    assert_eq!(unit_struct(), 42);
}

/// Test unit struct with braces
#[test]
fn test_unit_struct_with_braces() {
    #[vm_protect(level = "debug")]
    fn unit_struct_braces() -> u64 {
        struct Empty {}
        let _e = Empty {};  // Unit struct with braces
        99
    }

    assert_eq!(unit_struct_braces(), 99);
}

/// Test struct with standard protection level
#[test]
fn test_struct_standard() {
    #[vm_protect(level = "standard")]
    fn standard_struct() -> u64 {
        struct Data {
            key: u64,
            value: u64,
        }

        let d = Data { key: 0xCAFE, value: 0xBABE };
        d.key ^ d.value
    }

    let result = standard_struct();
    assert_eq!(result, 0xCAFE ^ 0xBABE);
}

/// Test struct with paranoid protection level
#[test]
fn test_struct_paranoid() {
    #[vm_protect(level = "paranoid")]
    fn paranoid_struct() -> u64 {
        struct Secret {
            a: u64,
            b: u64,
        }

        let s = Secret { a: 123, b: 456 };
        s.a + s.b
    }

    assert_eq!(paranoid_struct(), 579);
}

/// Test struct field used as array index
#[test]
fn test_struct_field_as_index() {
    #[vm_protect(level = "debug")]
    fn index_from_struct() -> u64 {
        struct IndexHolder {
            idx: u64,
        }

        let arr = [10, 20, 30, 40, 50];
        let holder = IndexHolder { idx: 2 };

        arr[holder.idx as usize]
    }

    assert_eq!(index_from_struct(), 30);
}

/// Test struct with complex field expressions
#[test]
fn test_struct_complex_init() {
    #[vm_protect(level = "debug")]
    fn complex_init() -> u64 {
        struct Result {
            sum: u64,
            product: u64,
        }

        let a = 5;
        let b = 7;
        let r = Result {
            sum: a + b,
            product: a * b,
        };

        r.sum + r.product
    }

    // sum = 5+7 = 12, product = 5*7 = 35
    assert_eq!(complex_init(), 47);
}

/// Test all protection levels produce same result
#[test]
fn test_struct_all_levels_equivalent() {
    #[vm_protect(level = "debug")]
    fn debug_impl() -> u64 {
        struct Point { x: u64, y: u64 }
        let p = Point { x: 100, y: 200 };
        p.x * p.y
    }

    #[vm_protect(level = "standard")]
    fn standard_impl() -> u64 {
        struct Point { x: u64, y: u64 }
        let p = Point { x: 100, y: 200 };
        p.x * p.y
    }

    #[vm_protect(level = "paranoid")]
    fn paranoid_impl() -> u64 {
        struct Point { x: u64, y: u64 }
        let p = Point { x: 100, y: 200 };
        p.x * p.y
    }

    let debug_result = debug_impl();
    let standard_result = standard_impl();
    let paranoid_result = paranoid_impl();

    assert_eq!(debug_result, 20000);
    assert_eq!(standard_result, 20000);
    assert_eq!(paranoid_result, 20000);
}

// ============================================================================
// Field Init Shorthand Tests
// ============================================================================

/// Test field init shorthand: Point { x, y } instead of Point { x: x, y: y }
#[test]
fn test_field_init_shorthand() {
    #[vm_protect(level = "debug")]
    fn shorthand() -> u64 {
        struct Point {
            x: u64,
            y: u64,
        }

        let x = 10;
        let y = 20;
        let p = Point { x, y };  // shorthand syntax
        p.x + p.y
    }

    assert_eq!(shorthand(), 30);
}

/// Test mixed shorthand and explicit fields
#[test]
fn test_field_init_mixed() {
    #[vm_protect(level = "debug")]
    fn mixed() -> u64 {
        struct Triple {
            a: u64,
            b: u64,
            c: u64,
        }

        let a = 1;
        let c = 3;
        let t = Triple { a, b: 2, c };  // mixed syntax
        t.a + t.b + t.c
    }

    assert_eq!(mixed(), 6);
}

// ============================================================================
// Functional Update Syntax Tests
// ============================================================================

/// Test functional update: Point { x: 10, ..base }
#[test]
fn test_functional_update_basic() {
    #[vm_protect(level = "debug")]
    fn update() -> u64 {
        struct Point {
            x: u64,
            y: u64,
        }

        let base = Point { x: 1, y: 2 };
        let updated = Point { x: 100, ..base };  // y copied from base
        updated.x + updated.y  // 100 + 2 = 102
    }

    assert_eq!(update(), 102);
}

/// Test functional update with multiple fields
#[test]
fn test_functional_update_multiple() {
    #[vm_protect(level = "debug")]
    fn update_multi() -> u64 {
        struct Config {
            a: u64,
            b: u64,
            c: u64,
            d: u64,
        }

        let base = Config { a: 1, b: 2, c: 3, d: 4 };
        let updated = Config { b: 20, d: 40, ..base };  // a, c copied from base
        updated.a + updated.b + updated.c + updated.d  // 1 + 20 + 3 + 40 = 64
    }

    assert_eq!(update_multi(), 64);
}

/// Test functional update copying all fields (just ..base)
#[test]
fn test_functional_update_copy_all() {
    #[vm_protect(level = "debug")]
    fn copy_all() -> u64 {
        struct Data {
            x: u64,
            y: u64,
        }

        let original = Data { x: 42, y: 58 };
        let copy = Data { ..original };  // all fields copied
        copy.x + copy.y
    }

    assert_eq!(copy_all(), 100);
}

/// Test functional update with computed values
#[test]
fn test_functional_update_computed() {
    #[vm_protect(level = "debug")]
    fn computed() -> u64 {
        struct Rect {
            x: u64,
            y: u64,
            w: u64,
            h: u64,
        }

        let r1 = Rect { x: 0, y: 0, w: 10, h: 20 };
        let r2 = Rect { x: r1.x + 5, y: r1.y + 5, ..r1 };  // move by (5, 5)
        r2.x + r2.y + r2.w + r2.h  // 5 + 5 + 10 + 20 = 40
    }

    assert_eq!(computed(), 40);
}

// ============================================================================
// Tuple Struct Tests
// ============================================================================

/// Test basic tuple struct
#[test]
fn test_tuple_struct() {
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
fn test_tuple_struct_triple() {
    #[vm_protect(level = "debug")]
    fn triple() -> u64 {
        struct Vec3(u64, u64, u64);
        let v = Vec3(1, 2, 3);
        v.0 + v.1 + v.2
    }

    assert_eq!(triple(), 6);
}

/// Test tuple struct mutation
#[test]
fn test_tuple_struct_mutation() {
    #[vm_protect(level = "debug")]
    fn mutate() -> u64 {
        struct Pair(u64, u64);
        let mut p = Pair(1, 2);
        p.0 = 100;
        p.0 + p.1  // 100 + 2 = 102
    }

    assert_eq!(mutate(), 102);
}

/// Test tuple struct in expression
#[test]
fn test_tuple_struct_expr() {
    #[vm_protect(level = "debug")]
    fn distance() -> u64 {
        struct Point(u64, u64);
        let p = Point(3, 4);
        p.0 * p.0 + p.1 * p.1  // 9 + 16 = 25
    }

    assert_eq!(distance(), 25);
}

/// Test multiple tuple structs
#[test]
fn test_multiple_tuple_structs() {
    #[vm_protect(level = "debug")]
    fn multi_tuple() -> u64 {
        struct Point(u64, u64);
        struct Size(u64, u64);

        let p = Point(10, 20);
        let s = Size(100, 200);
        p.0 + p.1 + s.0 + s.1  // 10 + 20 + 100 + 200 = 330
    }

    assert_eq!(multi_tuple(), 330);
}

// ============================================================================
// Numeric Field Access (Color{0: 255, 1: 128, 2: 64})
// ============================================================================

/// Test numeric field initialization in tuple struct style
#[test]
fn test_numeric_field_init() {
    #[vm_protect(level = "debug")]
    fn numeric_init() -> u64 {
        struct Color(u64, u64, u64);
        // Using call syntax for tuple struct
        let c = Color(255, 128, 64);
        c.0 + c.1 + c.2
    }

    assert_eq!(numeric_init(), 447);
}

// ============================================================================
// Complex Struct Tests
// ============================================================================

/// Test struct with many fields (stress test)
#[test]
fn test_struct_many_fields() {
    #[vm_protect(level = "debug")]
    fn many_fields() -> u64 {
        struct BigStruct {
            a: u64,
            b: u64,
            c: u64,
            d: u64,
            e: u64,
            f: u64,
            g: u64,
            h: u64,
        }

        let s = BigStruct {
            a: 1, b: 2, c: 3, d: 4,
            e: 5, f: 6, g: 7, h: 8,
        };
        s.a + s.b + s.c + s.d + s.e + s.f + s.g + s.h
    }

    assert_eq!(many_fields(), 36);
}

/// Test multiple struct instances
#[test]
fn test_struct_multiple_instances() {
    #[vm_protect(level = "debug")]
    fn multi_instances() -> u64 {
        struct Point { x: u64, y: u64 }

        let p1 = Point { x: 10, y: 20 };
        let p2 = Point { x: 30, y: 40 };
        let p3 = Point { x: 50, y: 60 };

        p1.x + p2.y + p3.x + p1.y + p2.x + p3.y
    }

    // 10 + 40 + 50 + 20 + 30 + 60 = 210
    assert_eq!(multi_instances(), 210);
}

/// Test struct field in complex arithmetic
#[test]
fn test_struct_complex_arithmetic() {
    #[vm_protect(level = "debug")]
    fn complex_math() -> u64 {
        struct Calc {
            base: u64,
            mult: u64,
            add: u64,
        }

        let c = Calc { base: 5, mult: 3, add: 7 };
        (c.base * c.mult + c.add) * 2 + c.base
    }

    // (5 * 3 + 7) * 2 + 5 = (15 + 7) * 2 + 5 = 22 * 2 + 5 = 44 + 5 = 49
    assert_eq!(complex_math(), 49);
}

/// Test struct field bitwise operations
#[test]
fn test_struct_bitwise() {
    #[vm_protect(level = "debug")]
    fn bitwise_ops() -> u64 {
        struct Bits {
            mask: u64,
            value: u64,
            shift: u64,
        }

        let b = Bits { mask: 0xFF, value: 0x12AB, shift: 4 };
        (b.value & b.mask) | ((b.value >> b.shift) & b.mask)
    }

    // (0x12AB & 0xFF) | ((0x12AB >> 4) & 0xFF)
    // = 0xAB | (0x12A & 0xFF)
    // = 0xAB | 0x2A
    // = 0xAB
    assert_eq!(bitwise_ops(), 0xAB);
}

/// Test struct in nested loops
#[test]
fn test_struct_nested_loops() {
    #[vm_protect(level = "debug")]
    fn nested_loop_struct() -> u64 {
        struct Counter { x: u64, y: u64, total: u64 }

        let mut c = Counter { x: 0, y: 0, total: 0 };

        for i in 0..3 {
            c.x = c.x + 1;
            for j in 0..4 {
                c.y = c.y + 1;
                c.total = c.total + i + j;
            }
        }
        c.x * 1000 + c.y * 10 + c.total
    }

    // x = 3, y = 12
    // total = (0+0)+(0+1)+(0+2)+(0+3) + (1+0)+(1+1)+(1+2)+(1+3) + (2+0)+(2+1)+(2+2)+(2+3)
    //       = 0+1+2+3 + 1+2+3+4 + 2+3+4+5 = 6 + 10 + 14 = 30
    // Result: 3*1000 + 12*10 + 30 = 3000 + 120 + 30 = 3150
    assert_eq!(nested_loop_struct(), 3150);
}

/// Test struct field conditional updates
#[test]
fn test_struct_conditional_update() {
    #[vm_protect(level = "debug")]
    fn conditional() -> u64 {
        struct State { value: u64, flag: u64 }

        let mut s = State { value: 10, flag: 1 };

        if s.flag > 0 {
            s.value = s.value * 2;
        }

        if s.value > 15 {
            s.flag = 0;
        }

        s.value * 100 + s.flag
    }

    // value = 10 * 2 = 20, then flag = 0 (since 20 > 15)
    // Result: 20 * 100 + 0 = 2000
    assert_eq!(conditional(), 2000);
}

/// Test struct with while loop
#[test]
fn test_struct_while_loop() {
    #[vm_protect(level = "debug")]
    fn while_loop_struct() -> u64 {
        struct Iterator { current: u64, step: u64, limit: u64 }

        let mut iter = Iterator { current: 0, step: 7, limit: 50 };
        let mut count = 0;

        while iter.current < iter.limit {
            iter.current = iter.current + iter.step;
            count = count + 1;
        }

        count * 1000 + iter.current
    }

    // Steps: 0->7->14->21->28->35->42->49->56 (8 iterations, stops at 56)
    // Result: 8 * 1000 + 56 = 8056
    assert_eq!(while_loop_struct(), 8056);
}

/// Test struct fields in comparison chain
#[test]
fn test_struct_comparison_chain() {
    #[vm_protect(level = "debug")]
    fn comparison_chain() -> u64 {
        struct Range { min: u64, val: u64, max: u64 }

        let r = Range { min: 10, val: 25, max: 50 };

        if r.val >= r.min {
            if r.val <= r.max {
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

/// Test struct field used multiple times in expression
#[test]
fn test_struct_field_reuse() {
    #[vm_protect(level = "debug")]
    fn field_reuse() -> u64 {
        struct Num { v: u64 }

        let n = Num { v: 5 };
        // Use n.v many times in one expression
        n.v + n.v * n.v + n.v * n.v * n.v
    }

    // 5 + 25 + 125 = 155
    assert_eq!(field_reuse(), 155);
}

/// Test struct with computed field values from other struct
#[test]
fn test_struct_field_from_struct() {
    #[vm_protect(level = "debug")]
    fn struct_from_struct() -> u64 {
        struct Input { a: u64, b: u64 }
        struct Output { sum: u64, prod: u64 }

        let input = Input { a: 6, b: 7 };
        let output = Output {
            sum: input.a + input.b,
            prod: input.a * input.b,
        };

        output.sum + output.prod
    }

    // sum = 13, prod = 42
    // Result: 13 + 42 = 55
    assert_eq!(struct_from_struct(), 55);
}

/// Test struct field swapping
#[test]
fn test_struct_field_swap() {
    #[vm_protect(level = "debug")]
    fn field_swap() -> u64 {
        struct Pair { left: u64, right: u64 }

        let mut p = Pair { left: 100, right: 200 };

        // Swap left and right
        let temp = p.left;
        p.left = p.right;
        p.right = temp;

        p.left * 1000 + p.right
    }

    // After swap: left=200, right=100
    // Result: 200 * 1000 + 100 = 200100
    assert_eq!(field_swap(), 200100);
}

/// Test fibonacci using struct
#[test]
fn test_struct_fibonacci() {
    #[vm_protect(level = "debug")]
    fn fib_struct() -> u64 {
        struct Fib { prev: u64, curr: u64 }

        let mut f = Fib { prev: 0, curr: 1 };

        for _i in 0..10 {
            let next = f.prev + f.curr;
            f.prev = f.curr;
            f.curr = next;
        }

        f.curr
    }

    // After 10 iterations: 1,1,2,3,5,8,13,21,34,55,89
    assert_eq!(fib_struct(), 89);
}

/// Test struct accumulation pattern
#[test]
fn test_struct_accumulator_pattern() {
    #[vm_protect(level = "debug")]
    fn accumulator() -> u64 {
        struct Stats { sum: u64, count: u64, min: u64, max: u64 }

        let mut stats = Stats { sum: 0, count: 0, min: 999, max: 0 };

        let values = [10, 25, 5, 30, 15];

        for i in 0..5 {
            let v = values[i];
            stats.sum = stats.sum + v;
            stats.count = stats.count + 1;
            if v < stats.min {
                stats.min = v;
            }
            if v > stats.max {
                stats.max = v;
            }
        }

        stats.sum + stats.count * 100 + stats.min * 10 + stats.max
    }

    // sum = 85, count = 5, min = 5, max = 30
    // Result: 85 + 500 + 50 + 30 = 665
    assert_eq!(accumulator(), 665);
}

/// Test struct with break in loop
#[test]
fn test_struct_loop_break() {
    #[vm_protect(level = "debug")]
    fn loop_break() -> u64 {
        struct Search { target: u64, found_at: u64 }

        let mut s = Search { target: 25, found_at: 999 };
        let arr = [10, 15, 20, 25, 30, 35];

        for i in 0..6 {
            if arr[i] == s.target {
                s.found_at = i as u64;
                break;
            }
        }

        s.found_at
    }

    // Found 25 at index 3
    assert_eq!(loop_break(), 3);
}

/// Test struct with continue in loop
#[test]
fn test_struct_loop_continue() {
    #[vm_protect(level = "debug")]
    fn loop_continue() -> u64 {
        struct Filter { threshold: u64, sum: u64 }

        let mut f = Filter { threshold: 15, sum: 0 };
        let arr = [10, 20, 5, 25, 12, 30];

        for i in 0..6 {
            if arr[i] < f.threshold {
                continue;
            }
            f.sum = f.sum + arr[i];
        }

        f.sum
    }

    // Skip 10, 5, 12 (< 15)
    // Sum 20 + 25 + 30 = 75
    assert_eq!(loop_continue(), 75);
}
