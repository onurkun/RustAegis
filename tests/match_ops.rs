//! Tests for match expression support in vm_protect

use aegis_vm::vm_protect;

// ============================================================================
// Basic Literal Matching
// ============================================================================

/// Test simple integer match
#[test]
fn test_match_integer_literal() {
    #[vm_protect(level = "debug")]
    fn match_int(x: u64) -> u64 {
        match x {
            1 => 100,
            2 => 200,
            3 => 300,
            _ => 0,
        }
    }

    assert_eq!(match_int(1), 100);
    assert_eq!(match_int(2), 200);
    assert_eq!(match_int(3), 300);
    assert_eq!(match_int(99), 0);
}

/// Test match as expression (returns value)
#[test]
fn test_match_as_expression() {
    #[vm_protect(level = "debug")]
    fn match_expr(x: u64) -> u64 {
        let result = match x {
            0 => 10,
            1 => 20,
            _ => 30,
        };
        result * 2
    }

    assert_eq!(match_expr(0), 20);
    assert_eq!(match_expr(1), 40);
    assert_eq!(match_expr(5), 60);
}

/// Test match with computation in arms
#[test]
fn test_match_with_computation() {
    #[vm_protect(level = "debug")]
    fn compute_match(x: u64) -> u64 {
        match x {
            0 => 1 + 2 + 3,
            1 => 10 * 10,
            2 => 500 - 200,
            _ => x * x,
        }
    }

    assert_eq!(compute_match(0), 6);
    assert_eq!(compute_match(1), 100);
    assert_eq!(compute_match(2), 300);
    assert_eq!(compute_match(5), 25);
}

// ============================================================================
// Variable Binding Patterns
// ============================================================================

/// Test wildcard pattern
#[test]
fn test_match_wildcard() {
    #[vm_protect(level = "debug")]
    fn wildcard_match(x: u64) -> u64 {
        match x {
            0 => 0,
            _ => 999,
        }
    }

    assert_eq!(wildcard_match(0), 0);
    assert_eq!(wildcard_match(1), 999);
    assert_eq!(wildcard_match(100), 999);
}

/// Test variable binding in match
#[test]
fn test_match_variable_binding() {
    #[vm_protect(level = "debug")]
    fn bind_match(x: u64) -> u64 {
        match x {
            0 => 0,
            n => n * 2,
        }
    }

    assert_eq!(bind_match(0), 0);
    assert_eq!(bind_match(5), 10);
    assert_eq!(bind_match(50), 100);
}

/// Test match with underscore-prefixed binding
#[test]
fn test_match_underscore_binding() {
    #[vm_protect(level = "debug")]
    fn underscore_match(x: u64) -> u64 {
        match x {
            0 => 100,
            _other => 200,  // bound but intentionally unused
        }
    }

    assert_eq!(underscore_match(0), 100);
    assert_eq!(underscore_match(5), 200);
}

// ============================================================================
// Range Patterns
// ============================================================================

/// Test inclusive range pattern
#[test]
fn test_match_range_inclusive() {
    #[vm_protect(level = "debug")]
    fn range_match(x: u64) -> u64 {
        match x {
            0..=5 => 1,
            6..=10 => 2,
            _ => 3,
        }
    }

    assert_eq!(range_match(0), 1);
    assert_eq!(range_match(3), 1);
    assert_eq!(range_match(5), 1);
    assert_eq!(range_match(6), 2);
    assert_eq!(range_match(10), 2);
    assert_eq!(range_match(11), 3);
}

/// Test grade-like matching with ranges
#[test]
fn test_match_grade_ranges() {
    #[vm_protect(level = "debug")]
    fn grade(score: u64) -> u64 {
        match score {
            90..=100 => 5,  // A
            80..=89 => 4,   // B
            70..=79 => 3,   // C
            60..=69 => 2,   // D
            0..=59 => 1,    // F
            _ => 0,         // Invalid
        }
    }

    assert_eq!(grade(95), 5);
    assert_eq!(grade(85), 4);
    assert_eq!(grade(75), 3);
    assert_eq!(grade(65), 2);
    assert_eq!(grade(55), 1);
    assert_eq!(grade(150), 0);
}

// ============================================================================
// Or Patterns
// ============================================================================

/// Test or-pattern with literals
#[test]
fn test_match_or_pattern() {
    #[vm_protect(level = "debug")]
    fn or_match(x: u64) -> u64 {
        match x {
            1 | 2 | 3 => 10,
            4 | 5 | 6 => 20,
            _ => 30,
        }
    }

    assert_eq!(or_match(1), 10);
    assert_eq!(or_match(2), 10);
    assert_eq!(or_match(3), 10);
    assert_eq!(or_match(4), 20);
    assert_eq!(or_match(5), 20);
    assert_eq!(or_match(6), 20);
    assert_eq!(or_match(7), 30);
}

/// Test is_weekend-like function
#[test]
fn test_match_weekend() {
    #[vm_protect(level = "debug")]
    fn is_weekend(day: u64) -> u64 {
        match day {
            0 | 6 => 1,  // Sunday or Saturday
            1 | 2 | 3 | 4 | 5 => 0,  // Weekdays
            _ => 2,  // Invalid
        }
    }

    assert_eq!(is_weekend(0), 1);  // Sunday
    assert_eq!(is_weekend(6), 1);  // Saturday
    assert_eq!(is_weekend(1), 0);  // Monday
    assert_eq!(is_weekend(3), 0);  // Wednesday
    assert_eq!(is_weekend(7), 2);  // Invalid
}

// ============================================================================
// Match Guards
// ============================================================================

/// Test match with guard condition
#[test]
fn test_match_guard() {
    #[vm_protect(level = "debug")]
    fn guarded_match(x: u64) -> u64 {
        match x {
            n if n > 100 => 3,
            n if n > 50 => 2,
            n if n > 0 => 1,
            _ => 0,
        }
    }

    assert_eq!(guarded_match(0), 0);
    assert_eq!(guarded_match(25), 1);
    assert_eq!(guarded_match(75), 2);
    assert_eq!(guarded_match(150), 3);
}

/// Test match guard with computation
#[test]
fn test_match_guard_computation() {
    #[vm_protect(level = "debug")]
    fn guard_compute(x: u64) -> u64 {
        match x {
            n if n % 2 == 0 => n / 2,
            n => n * 3 + 1,
        }
    }

    assert_eq!(guard_compute(10), 5);   // even
    assert_eq!(guard_compute(7), 22);   // odd: 7*3+1
    assert_eq!(guard_compute(0), 0);    // even
}

// ============================================================================
// Complex Match Scenarios
// ============================================================================

/// Test nested match
#[test]
fn test_nested_match() {
    #[vm_protect(level = "debug")]
    fn nested(x: u64, y: u64) -> u64 {
        match x {
            0 => match y {
                0 => 0,
                _ => 1,
            },
            _ => match y {
                0 => 2,
                _ => 3,
            },
        }
    }

    assert_eq!(nested(0, 0), 0);
    assert_eq!(nested(0, 5), 1);
    assert_eq!(nested(5, 0), 2);
    assert_eq!(nested(5, 5), 3);
}

/// Test match in loop
#[test]
fn test_match_in_loop() {
    #[vm_protect(level = "debug")]
    fn loop_match() -> u64 {
        let mut sum = 0;
        for i in 0..10 {
            sum = sum + match i {
                0 | 5 => 10,
                1..=4 => 1,
                _ => 2,
            };
        }
        sum
    }

    // i=0: 10, i=1-4: 1*4=4, i=5: 10, i=6-9: 2*4=8
    // Total: 10 + 4 + 10 + 8 = 32
    assert_eq!(loop_match(), 32);
}

/// Test match with variable from outer scope
#[test]
fn test_match_outer_variable() {
    #[vm_protect(level = "debug")]
    fn outer_var(x: u64) -> u64 {
        let multiplier = 10;
        match x {
            0 => multiplier,
            n => n * multiplier,
        }
    }

    assert_eq!(outer_var(0), 10);
    assert_eq!(outer_var(5), 50);
}

/// Test match result used in expression
#[test]
fn test_match_in_expression() {
    #[vm_protect(level = "debug")]
    fn expr_match(x: u64) -> u64 {
        let base = match x {
            0 => 100,
            _ => 200,
        };
        base + x * 10
    }

    assert_eq!(expr_match(0), 100);
    assert_eq!(expr_match(5), 250);  // 200 + 50
}

// ============================================================================
// Fibonacci and Factorial with Match
// ============================================================================

/// Test fibonacci using match (iterative)
#[test]
fn test_match_fibonacci() {
    #[vm_protect(level = "debug")]
    fn fib(n: u64) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => {
                let mut a = 0;
                let mut b = 1;
                for _i in 2..=n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                b
            }
        }
    }

    assert_eq!(fib(0), 0);
    assert_eq!(fib(1), 1);
    assert_eq!(fib(10), 55);
}

/// Test factorial using match
#[test]
fn test_match_factorial() {
    #[vm_protect(level = "debug")]
    fn factorial(n: u64) -> u64 {
        match n {
            0 | 1 => 1,
            _ => {
                let mut result = 1;
                for i in 2..=n {
                    result = result * i;
                }
                result
            }
        }
    }

    assert_eq!(factorial(0), 1);
    assert_eq!(factorial(1), 1);
    assert_eq!(factorial(5), 120);
    assert_eq!(factorial(10), 3628800);
}

// ============================================================================
// Protection Levels
// ============================================================================

/// Test match with standard protection
#[test]
fn test_match_standard() {
    #[vm_protect(level = "standard")]
    fn standard_match(x: u64) -> u64 {
        match x {
            0 => 0xCAFE,
            1 => 0xBABE,
            _ => 0xDEAD,
        }
    }

    assert_eq!(standard_match(0), 0xCAFE);
    assert_eq!(standard_match(1), 0xBABE);
    assert_eq!(standard_match(2), 0xDEAD);
}

/// Test match with paranoid protection
#[test]
fn test_match_paranoid() {
    #[vm_protect(level = "paranoid")]
    fn paranoid_match(x: u64) -> u64 {
        match x {
            0..=10 => x * 10,
            11..=20 => x * 20,
            _ => x * 100,
        }
    }

    assert_eq!(paranoid_match(5), 50);
    assert_eq!(paranoid_match(15), 300);
    assert_eq!(paranoid_match(50), 5000);
}

// ============================================================================
// State Machine with Match
// ============================================================================

/// Test simple state machine
#[test]
fn test_match_state_machine() {
    #[vm_protect(level = "debug")]
    fn state_machine(initial_state: u64, input: u64) -> u64 {
        // States: 0=idle, 1=running, 2=paused, 3=stopped
        // Inputs: 0=start, 1=pause, 2=resume, 3=stop

        let mut state = initial_state;

        match state {
            0 => {  // idle
                state = match input {
                    0 => 1,  // start -> running
                    _ => 0,  // stay idle
                };
            }
            1 => {  // running
                state = match input {
                    1 => 2,  // pause -> paused
                    3 => 3,  // stop -> stopped
                    _ => 1,  // stay running
                };
            }
            2 => {  // paused
                state = match input {
                    2 => 1,  // resume -> running
                    3 => 3,  // stop -> stopped
                    _ => 2,  // stay paused
                };
            }
            _ => {
                state = 3;  // stopped (terminal)
            }
        }

        state
    }

    // idle + start = running
    assert_eq!(state_machine(0, 0), 1);
    // running + pause = paused
    assert_eq!(state_machine(1, 1), 2);
    // paused + resume = running
    assert_eq!(state_machine(2, 2), 1);
    // running + stop = stopped
    assert_eq!(state_machine(1, 3), 3);
}

// ============================================================================
// Tuple Destructuring Patterns
// ============================================================================

/// Test basic tuple destructuring
#[test]
fn test_match_tuple_destructure() {
    #[vm_protect(level = "debug")]
    fn tuple_match() -> u64 {
        let pair = (10, 20);
        match pair {
            (a, b) => a + b,
        }
    }

    assert_eq!(tuple_match(), 30);
}

/// Test tuple with literal patterns
#[test]
fn test_match_tuple_with_literals() {
    #[vm_protect(level = "debug")]
    fn tuple_lit(x: u64, y: u64) -> u64 {
        let t = (x, y);
        match t {
            (0, 0) => 0,
            (0, _) => 1,
            (_, 0) => 2,
            (_, _) => 3,
        }
    }

    assert_eq!(tuple_lit(0, 0), 0);
    assert_eq!(tuple_lit(0, 5), 1);
    assert_eq!(tuple_lit(5, 0), 2);
    assert_eq!(tuple_lit(5, 5), 3);
}

/// Test tuple pattern with variable binding
#[test]
fn test_match_tuple_binding() {
    #[vm_protect(level = "debug")]
    fn tuple_bind() -> u64 {
        let t = (3, 4);
        match t {
            (x, y) => x * x + y * y,  // Pythagorean
        }
    }

    assert_eq!(tuple_bind(), 25);  // 9 + 16
}

/// Test match on nested tuple - destructure fully
#[test]
fn test_match_nested_tuple() {
    #[vm_protect(level = "debug")]
    fn nested_tuple() -> u64 {
        let t = ((1, 2), 3);
        match t {
            ((a, b), c) => a + b + c,
        }
    }

    assert_eq!(nested_tuple(), 6);  // 1 + 2 + 3
}

// ============================================================================
// Struct Destructuring Patterns
// ============================================================================

/// Test basic struct destructuring
#[test]
fn test_match_struct_destructure() {
    #[vm_protect(level = "debug")]
    fn struct_match() -> u64 {
        struct Point { x: u64, y: u64 }

        let p = Point { x: 10, y: 20 };
        match p {
            Point { x, y } => x + y,
        }
    }

    assert_eq!(struct_match(), 30);
}

/// Test struct with field renaming
#[test]
fn test_match_struct_rename() {
    #[vm_protect(level = "debug")]
    fn struct_rename() -> u64 {
        struct Coord { x: u64, y: u64 }

        let c = Coord { x: 5, y: 7 };
        match c {
            Coord { x: a, y: b } => a * b,
        }
    }

    assert_eq!(struct_rename(), 35);
}

/// Test struct pattern with literal field
#[test]
fn test_match_struct_literal_field() {
    #[vm_protect(level = "debug")]
    fn struct_lit() -> u64 {
        struct State { code: u64, value: u64 }

        let s = State { code: 1, value: 100 };
        match s {
            State { code: 0, value } => value,
            State { code: 1, value } => value * 2,
            State { code: _, value } => value * 3,
        }
    }

    assert_eq!(struct_lit(), 200);  // code=1, value=100*2
}

/// Test struct with wildcard fields
#[test]
fn test_match_struct_wildcard() {
    #[vm_protect(level = "debug")]
    fn struct_wild() -> u64 {
        struct Data { a: u64, b: u64, c: u64 }

        let d = Data { a: 1, b: 2, c: 3 };
        match d {
            Data { a, .. } => a * 100,
        }
    }

    assert_eq!(struct_wild(), 100);
}

/// Test multiple struct patterns
#[test]
fn test_match_struct_multiple_patterns() {
    #[vm_protect(level = "debug")]
    fn multi_struct(code: u64) -> u64 {
        struct Msg { kind: u64, data: u64 }

        let msg = Msg { kind: code, data: 42 };
        match msg {
            Msg { kind: 0, data } => data,
            Msg { kind: 1, data } => data + 100,
            Msg { kind: 2, data } => data + 200,
            Msg { kind: _, data } => data + 1000,
        }
    }

    assert_eq!(multi_struct(0), 42);
    assert_eq!(multi_struct(1), 142);
    assert_eq!(multi_struct(2), 242);
    assert_eq!(multi_struct(99), 1042);
}

// ============================================================================
// Combined Patterns
// ============================================================================

/// Test tuple struct destructuring
#[test]
fn test_match_tuple_struct() {
    #[vm_protect(level = "debug")]
    fn tuple_struct_match() -> u64 {
        struct Point(u64, u64);

        let p = Point(10, 20);
        // Access via tuple struct fields
        p.0 + p.1
    }

    assert_eq!(tuple_struct_match(), 30);
}

/// Test match with complex computation
#[test]
fn test_match_complex_compute() {
    #[vm_protect(level = "debug")]
    fn complex() -> u64 {
        let data = (100, 50);
        match data {
            (a, b) if a > b => a - b,
            (a, b) if a < b => b - a,
            (a, _) => a,  // equal
        }
    }

    assert_eq!(complex(), 50);  // 100 - 50
}

// ============================================================================
// @ Bindings (Bind and Match)
// ============================================================================

/// Test @ binding with range
#[test]
fn test_match_at_binding_range() {
    #[vm_protect(level = "debug")]
    fn at_range(x: u64) -> u64 {
        match x {
            n @ 1..=5 => n * 10,
            n @ 6..=10 => n * 20,
            _ => 0,
        }
    }

    assert_eq!(at_range(3), 30);   // 3 * 10
    assert_eq!(at_range(5), 50);   // 5 * 10
    assert_eq!(at_range(7), 140);  // 7 * 20
    assert_eq!(at_range(10), 200); // 10 * 20
    assert_eq!(at_range(15), 0);   // no match
}

/// Test @ binding with or-pattern
#[test]
fn test_match_at_binding_or() {
    #[vm_protect(level = "debug")]
    fn at_or(x: u64) -> u64 {
        match x {
            n @ (1 | 3 | 5) => n + 100,
            n @ (2 | 4 | 6) => n + 200,
            _ => 0,
        }
    }

    assert_eq!(at_or(1), 101);
    assert_eq!(at_or(3), 103);
    assert_eq!(at_or(5), 105);
    assert_eq!(at_or(2), 202);
    assert_eq!(at_or(4), 204);
    assert_eq!(at_or(6), 206);
    assert_eq!(at_or(7), 0);
}

/// Test @ binding simple
#[test]
fn test_match_at_binding_simple() {
    #[vm_protect(level = "debug")]
    fn at_simple(x: u64) -> u64 {
        match x {
            n @ 42 => n * 2,
            _ => 0,
        }
    }

    assert_eq!(at_simple(42), 84);
    assert_eq!(at_simple(10), 0);
}

// ============================================================================
// TupleStruct Pattern Matching
// ============================================================================

/// Test basic tuple struct destructuring in match
#[test]
fn test_match_tuple_struct_destructure() {
    #[vm_protect(level = "debug")]
    fn tuple_struct_match() -> u64 {
        struct Point(u64, u64);

        let p = Point(10, 20);
        match p {
            Point(x, y) => x + y,
        }
    }

    assert_eq!(tuple_struct_match(), 30);
}

/// Test tuple struct with literal patterns
#[test]
fn test_match_tuple_struct_literals() {
    #[vm_protect(level = "debug")]
    fn tuple_struct_lit() -> u64 {
        struct Pair(u64, u64);

        let p = Pair(1, 2);
        match p {
            Pair(0, y) => y,
            Pair(1, y) => y * 10,
            Pair(_, _) => 0,
        }
    }

    assert_eq!(tuple_struct_lit(), 20);  // 2 * 10
}

/// Test tuple struct with multiple arms
#[test]
fn test_match_tuple_struct_multi_arm() {
    #[vm_protect(level = "debug")]
    fn multi_arm(a: u64, b: u64) -> u64 {
        struct Coord(u64, u64);

        let c = Coord(a, b);
        match c {
            Coord(0, 0) => 0,
            Coord(0, y) => y,
            Coord(x, 0) => x * 10,
            Coord(x, y) => x * y,
        }
    }

    assert_eq!(multi_arm(0, 0), 0);
    assert_eq!(multi_arm(0, 5), 5);
    assert_eq!(multi_arm(5, 0), 50);
    assert_eq!(multi_arm(3, 4), 12);
}

/// Test tuple struct with 3 fields
#[test]
fn test_match_tuple_struct_three_fields() {
    #[vm_protect(level = "debug")]
    fn three_field() -> u64 {
        struct Triple(u64, u64, u64);

        let t = Triple(1, 2, 3);
        match t {
            Triple(a, b, c) => a + b * 10 + c * 100,
        }
    }

    assert_eq!(three_field(), 321);  // 1 + 20 + 300
}

/// Test tuple struct pattern with wildcard
#[test]
fn test_match_tuple_struct_wildcard() {
    #[vm_protect(level = "debug")]
    fn ts_wildcard() -> u64 {
        struct Data(u64, u64, u64);

        let d = Data(10, 20, 30);
        match d {
            Data(x, _, _) => x * 100,
        }
    }

    assert_eq!(ts_wildcard(), 1000);
}

// ============================================================================
// Slice/Array Pattern Matching (using Tuple as array stand-in)
// ============================================================================

/// Test slice-like pattern with tuple (simulating fixed array)
#[test]
fn test_match_array_like_pattern() {
    #[vm_protect(level = "debug")]
    fn array_match() -> u64 {
        // Using tuple as fixed-size array
        let arr = (10, 20, 30);
        match arr {
            (a, b, c) => a + b + c,
        }
    }

    assert_eq!(array_match(), 60);
}

/// Test first/last element extraction
#[test]
fn test_match_first_last() {
    #[vm_protect(level = "debug")]
    fn first_last() -> u64 {
        let data = (1, 2, 3, 4, 5);
        match data {
            (first, _, _, _, last) => first * 100 + last,
        }
    }

    assert_eq!(first_last(), 105);  // 1*100 + 5
}

// ============================================================================
// Tagged Union / Simple Enum Pattern (struct-based)
// ============================================================================

/// Test Option-like pattern matching using tagged struct
#[test]
fn test_match_tagged_union_option() {
    #[vm_protect(level = "debug")]
    fn option_match() -> u64 {
        // Option<u64> as tagged union: tag=0 -> None, tag=1 -> Some
        struct Option { tag: u64, value: u64 }

        let some_val = Option { tag: 1, value: 42 };
        let none_val = Option { tag: 0, value: 0 };

        let r1 = match some_val {
            Option { tag: 0, .. } => 0,
            Option { tag: 1, value } => value * 2,
            Option { .. } => 999,
        };

        let r2 = match none_val {
            Option { tag: 0, .. } => 0,
            Option { tag: 1, value } => value * 2,
            Option { .. } => 999,
        };

        r1 + r2  // 84 + 0 = 84
    }

    assert_eq!(option_match(), 84);
}

/// Test Result-like pattern matching using tagged struct
#[test]
fn test_match_tagged_union_result() {
    #[vm_protect(level = "debug")]
    fn result_match(is_ok: u64, val: u64) -> u64 {
        // Result<u64, u64> as tagged union: tag=0 -> Ok, tag=1 -> Err
        struct Result { tag: u64, value: u64 }

        let r = Result { tag: is_ok, value: val };

        match r {
            Result { tag: 0, value } => value,       // Ok(value)
            Result { tag: 1, value } => value + 1000, // Err(value)
            Result { .. } => 9999,
        }
    }

    assert_eq!(result_match(0, 42), 42);     // Ok(42) -> 42
    assert_eq!(result_match(1, 5), 1005);    // Err(5) -> 1005
}

/// Test state machine with tagged union
#[test]
fn test_match_state_tagged() {
    #[vm_protect(level = "debug")]
    fn state_machine() -> u64 {
        // State: Idle=0, Running=1, Paused=2, Error=3
        struct State { kind: u64, data: u64 }

        let s0 = State { kind: 0, data: 0 };    // Idle
        let s1 = State { kind: 1, data: 100 };  // Running with progress

        let mut total = 0;

        // Process state 0 (Idle)
        total = total + match s0 {
            State { kind: 0, .. } => 1,
            State { kind: 1, data } => data,
            State { kind: 2, data } => data / 2,
            State { kind: 3, data } => data,
            State { .. } => 0,
        };

        // Process state 1 (Running)
        total = total + match s1 {
            State { kind: 0, .. } => 1,
            State { kind: 1, data } => data,
            State { kind: 2, data } => data / 2,
            State { kind: 3, data } => data,
            State { .. } => 0,
        };

        total  // 1 + 100 = 101
    }

    assert_eq!(state_machine(), 101);
}
