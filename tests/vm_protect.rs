//! VM Protect Macro Tests
//!
//! Comprehensive tests for the vm_protect procedural macro.
//! Tests arrays, strings, methods, type casts, and protection levels.

use aegis_vm::vm_protect;

// =============================================================================
// DEBUG MODE TESTS (with parameters)
// =============================================================================

#[vm_protect(level = "debug")]
fn add_42(x: u64) -> u64 {
    x + 42
}

#[vm_protect(level = "debug")]
fn multiply(a: u64, b: u64) -> u64 {
    a * b
}

#[vm_protect(level = "debug")]
fn xor_secret(x: u64) -> u64 {
    x ^ 0xDEADBEEF
}

#[vm_protect(level = "debug")]
fn complex_arithmetic(x: u64) -> u64 {
    ((x + 10) * 2) ^ 0xFF
}

#[vm_protect(level = "debug")]
fn bool_return(x: u64) -> bool {
    x > 100
}

#[test]
fn test_add_42() {
    assert_eq!(add_42(0), 42);
    assert_eq!(add_42(8), 50);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(6, 7), 42);
}

#[test]
fn test_xor_secret() {
    assert_eq!(xor_secret(0), 0xDEADBEEF);
    assert_eq!(xor_secret(0xDEADBEEF), 0);
}

#[test]
fn test_complex_arithmetic() {
    assert_eq!(complex_arithmetic(0), 235);
}

#[test]
fn test_bool_return() {
    assert!(!bool_return(50));
    assert!(bool_return(101));
}

// =============================================================================
// BASIC OPERATIONS
// =============================================================================

#[vm_protect(level = "standard")]
fn simple_add() -> u64 {
    let a: u64 = 10;
    let b: u64 = 20;
    a + b
}

#[test]
fn test_simple_add() {
    assert_eq!(simple_add(), 30);
}

#[vm_protect(level = "standard")]
fn while_loop_sum() -> u64 {
    let mut sum: u64 = 0;
    let mut i: u64 = 1;
    while i <= 5 {
        sum += i;
        i += 1;
    }
    sum
}

#[test]
fn test_while_loop() {
    assert_eq!(while_loop_sum(), 15);
}

#[vm_protect(level = "standard")]
fn for_loop_sum() -> u64 {
    let mut sum: u64 = 0;
    for i in 0..5 {
        sum += i;
    }
    sum
}

#[test]
fn test_for_loop() {
    assert_eq!(for_loop_sum(), 10);
}

// =============================================================================
// VARIABLE SHADOWING
// =============================================================================

#[vm_protect(level = "standard")]
fn shadowing_simple() -> u64 {
    let x: u64 = 10;
    let x: u64 = 20;  // Shadows outer x
    x
}

#[test]
fn test_shadowing_simple() {
    assert_eq!(shadowing_simple(), 20);
}

#[vm_protect(level = "standard")]
fn shadowing_in_block() -> u64 {
    let x: u64 = 10;
    let y: u64 = {
        let x: u64 = 100;  // Shadows outer x in block
        x + 5             // Uses inner x = 105
    };
    x + y  // Uses outer x = 10 + 105 = 115
}

#[test]
fn test_shadowing_in_block() {
    assert_eq!(shadowing_in_block(), 115);
}

#[vm_protect(level = "standard")]
fn shadowing_in_loop() -> u64 {
    let mut sum: u64 = 0;
    let x: u64 = 100;
    for i in 0..3 {
        let x: u64 = i * 10;  // Shadows outer x in each iteration
        sum += x;
    }
    sum + x  // sum = 0+10+20 = 30, x = 100, total = 130
}

#[test]
fn test_shadowing_in_loop() {
    assert_eq!(shadowing_in_loop(), 130);
}

#[vm_protect(level = "standard")]
fn shadowing_nested_blocks() -> u64 {
    let a: u64 = 1;
    let result: u64 = {
        let a: u64 = 10;  // First shadow
        let inner: u64 = {
            let a: u64 = 100;  // Second shadow
            a + 1  // 101
        };
        a + inner  // 10 + 101 = 111
    };
    a + result  // 1 + 111 = 112
}

#[test]
fn test_shadowing_nested_blocks() {
    assert_eq!(shadowing_nested_blocks(), 112);
}

// =============================================================================
// ARRAY OPERATIONS
// =============================================================================

#[vm_protect(level = "standard")]
fn array_literal() -> u64 {
    let arr = [10, 20, 30, 40, 50];
    arr[0] + arr[2] + arr[4]
}

#[test]
fn test_array_literal() {
    assert_eq!(array_literal(), 90);
}

#[vm_protect(level = "standard")]
fn array_len() -> u64 {
    let arr = [1, 2, 3, 4, 5];
    arr.len()
}

#[test]
fn test_array_len() {
    assert_eq!(array_len(), 5);
}

#[vm_protect(level = "standard")]
fn array_get() -> u64 {
    let arr = [10, 20, 30];
    arr.get(1)
}

#[test]
fn test_array_get() {
    assert_eq!(array_get(), 20);
}

#[vm_protect(level = "standard")]
fn array_modify() -> u64 {
    let mut arr = [1, 2, 3];
    arr[1] = 100;
    arr[0] + arr[1] + arr[2]
}

#[test]
fn test_array_modify() {
    assert_eq!(array_modify(), 104);
}

#[vm_protect(level = "standard")]
fn array_sum_loop() -> u64 {
    let arr = [10, 20, 30, 40, 50];
    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < arr.len() {
        sum += arr.get(i);
        i += 1;
    }
    sum
}

#[test]
fn test_array_sum_loop() {
    assert_eq!(array_sum_loop(), 150);
}

// =============================================================================
// STRING OPERATIONS
// =============================================================================

#[vm_protect(level = "standard")]
fn string_len() -> u64 {
    let s = "hello";
    s.len()
}

#[test]
fn test_string_len() {
    assert_eq!(string_len(), 5);
}

#[vm_protect(level = "standard")]
fn string_get_byte() -> u64 {
    let s = "hello";
    s.get(0)  // 'h' = 104
}

#[test]
fn test_string_get_byte() {
    assert_eq!(string_get_byte(), 104);
}

#[vm_protect(level = "standard")]
fn string_is_empty() -> u64 {
    let s = "";
    if s.is_empty() { 1 } else { 0 }
}

#[test]
fn test_string_is_empty() {
    assert_eq!(string_is_empty(), 1);
}

#[vm_protect(level = "standard")]
fn string_eq() -> u64 {
    let s1 = "test";
    let s2 = "test";
    if s1.eq(s2) { 1 } else { 0 }
}

#[test]
fn test_string_eq() {
    assert_eq!(string_eq(), 1);
}

#[vm_protect(level = "standard")]
fn string_hash() -> u64 {
    let s1 = "key";
    let s2 = "key";
    if s1.hash() == s2.hash() { 1 } else { 0 }
}

#[test]
fn test_string_hash() {
    assert_eq!(string_hash(), 1);
}

#[vm_protect(level = "standard")]
fn string_concat() -> u64 {
    let s1 = "hello";
    let s2 = "world";
    s1.concat(s2).len()
}

#[test]
fn test_string_concat() {
    assert_eq!(string_concat(), 10);
}

// =============================================================================
// TYPE CASTS
// =============================================================================

#[vm_protect(level = "standard")]
fn cast_to_u32() -> u64 {
    let x: u64 = 0x1234_5678_9ABC_DEF0;
    (x as u32) as u64
}

#[test]
fn test_cast_to_u32() {
    assert_eq!(cast_to_u32(), 0x9ABC_DEF0);
}

#[vm_protect(level = "standard")]
fn cast_to_u16() -> u64 {
    let x: u64 = 0x1234_5678;
    (x as u16) as u64
}

#[test]
fn test_cast_to_u16() {
    assert_eq!(cast_to_u16(), 0x5678);
}

#[vm_protect(level = "standard")]
fn cast_to_u8() -> u64 {
    let x: u64 = 0x1234;
    (x as u8) as u64
}

#[test]
fn test_cast_to_u8() {
    assert_eq!(cast_to_u8(), 0x34);
}

#[vm_protect(level = "standard")]
fn cast_sign_extend() -> u64 {
    let x: u64 = 0xFF;
    let y = x as i8;
    (y as i64) as u64
}

#[test]
fn test_cast_sign_extend() {
    assert_eq!(cast_sign_extend(), 0xFFFFFFFFFFFFFFFF);
}

// =============================================================================
// NUMERIC METHODS
// =============================================================================

#[vm_protect(level = "standard")]
fn numeric_min() -> u64 {
    let a: u64 = 10;
    a.min(20)
}

#[test]
fn test_numeric_min() {
    assert_eq!(numeric_min(), 10);
}

#[vm_protect(level = "standard")]
fn numeric_max() -> u64 {
    let a: u64 = 10;
    a.max(20)
}

#[test]
fn test_numeric_max() {
    assert_eq!(numeric_max(), 20);
}

#[vm_protect(level = "standard")]
fn wrapping_add() -> u64 {
    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    a.wrapping_add(1)
}

#[test]
fn test_wrapping_add() {
    assert_eq!(wrapping_add(), 0);
}

#[vm_protect(level = "standard")]
fn wrapping_sub() -> u64 {
    let a: u64 = 0;
    a.wrapping_sub(1)
}

#[test]
fn test_wrapping_sub() {
    assert_eq!(wrapping_sub(), u64::MAX);
}

#[vm_protect(level = "standard")]
fn rotate_left() -> u64 {
    let x: u64 = 0x8000_0000_0000_0001;
    x.rotate_left(1)
}

#[test]
fn test_rotate_left() {
    assert_eq!(rotate_left(), 0x8000_0000_0000_0001u64.rotate_left(1));
}

// =============================================================================
// SIGNED DIVISION
// =============================================================================

#[vm_protect(level = "standard")]
fn signed_division() -> u64 {
    let x: i64 = -10;
    let y: i64 = 3;
    let result = x / y;  // -10 / 3 = -3 (truncates toward zero)
    result as u64
}

#[test]
fn test_signed_division() {
    assert_eq!(signed_division(), (-3i64) as u64);
}

#[vm_protect(level = "standard")]
fn signed_modulo() -> u64 {
    let x: i64 = -10;
    let y: i64 = 3;
    let result = x % y;  // -10 % 3 = -1
    result as u64
}

#[test]
fn test_signed_modulo() {
    assert_eq!(signed_modulo(), (-1i64) as u64);
}

#[vm_protect(level = "standard")]
fn signed_div_positive() -> u64 {
    let x: i64 = 10;
    let y: i64 = 3;
    let result = x / y;  // 10 / 3 = 3
    result as u64
}

#[test]
fn test_signed_div_positive() {
    assert_eq!(signed_div_positive(), 3);
}

// =============================================================================
// BIT COUNTING
// =============================================================================

#[vm_protect(level = "standard")]
fn bit_count_ones() -> u64 {
    let x: u64 = 0b1010_1010_1010_1010;
    x.count_ones() as u64
}

#[test]
fn test_bit_count_ones() {
    assert_eq!(bit_count_ones(), 8);
}

#[vm_protect(level = "standard")]
fn bit_count_zeros() -> u64 {
    let x: u64 = 0b1111_1111;  // 8 ones, 56 zeros
    x.count_zeros() as u64
}

#[test]
fn test_bit_count_zeros() {
    assert_eq!(bit_count_zeros(), 56);
}

#[vm_protect(level = "standard")]
fn bit_leading_zeros() -> u64 {
    let x: u64 = 0x0000_0000_0000_00FF;  // 56 leading zeros
    x.leading_zeros() as u64
}

#[test]
fn test_bit_leading_zeros() {
    assert_eq!(bit_leading_zeros(), 56);
}

#[vm_protect(level = "standard")]
fn bit_trailing_zeros() -> u64 {
    let x: u64 = 0x0000_0000_0000_0100;  // 8 trailing zeros
    x.trailing_zeros() as u64
}

#[test]
fn test_bit_trailing_zeros() {
    assert_eq!(bit_trailing_zeros(), 8);
}

#[vm_protect(level = "standard")]
fn bit_count_ones_zero() -> u64 {
    let x: u64 = 0;
    x.count_ones() as u64
}

#[test]
fn test_bit_count_ones_zero() {
    assert_eq!(bit_count_ones_zero(), 0);
}

#[vm_protect(level = "standard")]
fn bit_leading_zeros_zero() -> u64 {
    let x: u64 = 0;
    x.leading_zeros() as u64
}

#[test]
fn test_bit_leading_zeros_zero() {
    assert_eq!(bit_leading_zeros_zero(), 64);
}

// =============================================================================
// PARANOID LEVEL
// =============================================================================

#[vm_protect(level = "paranoid")]
fn paranoid_combined() -> u64 {
    let arr = [10, 20, 30];
    let s = "abc";

    let arr_sum = arr.get(0) + arr.get(1) + arr.get(2);  // 60
    let str_len = s.len();  // 3
    let first_byte = s.get(0);  // 'a' = 97

    arr_sum + str_len + first_byte  // 60 + 3 + 97 = 160
}

#[test]
fn test_paranoid_combined() {
    assert_eq!(paranoid_combined(), 160);
}

#[vm_protect(level = "paranoid")]
fn paranoid_nested_loops() -> u64 {
    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < 3 {
        let mut j: u64 = 0;
        while j < 3 {
            sum += i * 10 + j;
            j += 1;
        }
        i += 1;
    }
    sum  // 0+1+2 + 10+11+12 + 20+21+22 = 99
}

#[test]
fn test_paranoid_nested_loops() {
    assert_eq!(paranoid_nested_loops(), 99);
}

// =============================================================================
// ENCRYPTED LEVEL
// =============================================================================

#[vm_protect(level = "encrypted")]
fn encrypted_simple() -> u64 {
    let x: u64 = 42;
    x * 2
}

#[test]
fn test_encrypted_simple() {
    assert_eq!(encrypted_simple(), 84);
}
