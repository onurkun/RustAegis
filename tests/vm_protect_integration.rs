//! Integration Tests - Mixed Operations
//!
//! Tests combining arrays, strings, numeric methods, casts, and control flow.

use aegis_vm::vm_protect;

// =============================================================================
// SECTION 1: Array + String Length Comparison
// =============================================================================

/// Compare array length with string length
#[vm_protect(level = "standard")]
fn array_string_len_compare() -> u64 {
    let arr = [1, 2, 3, 4, 5];      // len = 5
    let s = "hello";                 // len = 5

    let arr_len = arr.len();
    let str_len = s.len();

    if arr_len == str_len { 1 } else { 0 }
}

#[test]
fn test_array_string_len_compare() {
    assert_eq!(array_string_len_compare(), 1, "Both should have length 5");
}

/// Sum array elements, multiply by string length
#[vm_protect(level = "standard")]
fn array_sum_times_strlen() -> u64 {
    let arr = [10, 20, 30];          // sum = 60
    let s = "ab";                     // len = 2

    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < arr.len() {
        sum += arr.get(i);
        i += 1;
    }

    sum * s.len()  // 60 * 2 = 120
}

#[test]
fn test_array_sum_times_strlen() {
    assert_eq!(array_sum_times_strlen(), 120);
}

// =============================================================================
// SECTION 2: Numeric Methods + Type Casts
// =============================================================================

/// Use min/max with casted values
#[vm_protect(level = "standard")]
fn minmax_with_casts() -> u64 {
    let big: u64 = 0x1234_5678_9ABC;
    let small = big as u16;           // Truncate to 0x9ABC = 39612
    let tiny = big as u8;             // Truncate to 0xBC = 188

    let a = (small as u64).min(50000);  // min(39612, 50000) = 39612
    let b = (tiny as u64).max(100);     // max(188, 100) = 188

    a + b  // 39612 + 188 = 39800
}

#[test]
fn test_minmax_with_casts() {
    assert_eq!(minmax_with_casts(), 39800);
}

/// Chain multiple casts and operations
#[vm_protect(level = "standard")]
fn cast_chain() -> u64 {
    let x: u64 = 0xABCD;
    let a = x as u8;         // 0xCD = 205
    let b = a as u64;        // 205
    let c = b.wrapping_mul(2);  // 410
    let d = c as u8;         // 410 & 0xFF = 154
    d as u64
}

#[test]
fn test_cast_chain() {
    assert_eq!(cast_chain(), 154);
}

// =============================================================================
// SECTION 3: String Operations + Control Flow
// =============================================================================

/// Count specific byte in string
#[vm_protect(level = "standard")]
fn count_byte_in_string() -> u64 {
    let s = "hello";
    let target: u64 = 108;  // 'l' = 108

    let mut count: u64 = 0;
    let mut i: u64 = 0;
    while i < s.len() {
        if s.get(i) == target {
            count += 1;
        }
        i += 1;
    }
    count  // 'l' appears 2 times
}

#[test]
fn test_count_byte_in_string() {
    assert_eq!(count_byte_in_string(), 2);
}

/// String hash affects array indexing
#[vm_protect(level = "standard")]
fn hash_based_index() -> u64 {
    let s = "key";
    let arr = [100, 200, 300, 400, 500];

    let h = s.hash();
    let idx = h % arr.len();  // hash mod 5

    arr.get(idx)
}

#[test]
fn test_hash_based_index() {
    // Just verify it runs without panic and returns a valid array element
    let result = hash_based_index();
    assert!(result == 100 || result == 200 || result == 300 || result == 400 || result == 500);
}

// =============================================================================
// SECTION 4: Complex Mixed Operations
// =============================================================================

/// Build result from multiple sources
#[vm_protect(level = "standard")]
fn mixed_computation() -> u64 {
    let arr = [5, 10, 15, 20, 25];
    let s1 = "abc";
    let s2 = "defgh";

    // Array: get min and max elements
    let mut min_val: u64 = arr.get(0);
    let mut max_val: u64 = arr.get(0);
    let mut i: u64 = 1;
    while i < arr.len() {
        let v = arr.get(i);
        min_val = min_val.min(v);
        max_val = max_val.max(v);
        i += 1;
    }

    // Strings: compare lengths
    let len_diff = if s2.len() > s1.len() {
        s2.len() - s1.len()
    } else {
        s1.len() - s2.len()
    };

    // Combine: (max - min) * len_diff + first_char
    let range = max_val - min_val;        // 25 - 5 = 20
    let first_char = s1.get(0) as u64;    // 'a' = 97

    range * len_diff + first_char  // 20 * 2 + 97 = 137
}

#[test]
fn test_mixed_computation() {
    assert_eq!(mixed_computation(), 137);
}

/// Rotate and compare with string byte
#[vm_protect(level = "standard")]
fn rotate_and_string() -> u64 {
    let x: u64 = 0x80;              // 128
    let rotated = x.rotate_right(1); // 0x40 = 64

    let s = "ABC";
    let byte_a = s.get(0);  // 'A' = 65

    // 64 < 65, so rotated < byte_a
    if rotated < byte_a { 1 } else { 0 }
}

#[test]
fn test_rotate_and_string() {
    assert_eq!(rotate_and_string(), 1);
}

// =============================================================================
// SECTION 5: Array Transformation with Casts
// =============================================================================

/// Transform array elements with casts
#[vm_protect(level = "standard")]
fn array_transform_cast() -> u64 {
    let arr = [256, 512, 768, 1024];

    // Sum of low bytes
    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < arr.len() {
        let val = arr.get(i);
        let low_byte = val as u8;  // Truncate to 8 bits
        sum += low_byte as u64;
        i += 1;
    }
    sum  // 0 + 0 + 0 + 0 = 0 (all are multiples of 256)
}

#[test]
fn test_array_transform_cast() {
    assert_eq!(array_transform_cast(), 0);
}

/// Array with non-trivial low bytes
#[vm_protect(level = "standard")]
fn array_lowbyte_sum() -> u64 {
    let arr = [257, 514, 771, 1028];  // low bytes: 1, 2, 3, 4

    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < arr.len() {
        let val = arr.get(i);
        let low_byte = val as u8;
        sum += low_byte as u64;
        i += 1;
    }
    sum  // 1 + 2 + 3 + 4 = 10
}

#[test]
fn test_array_lowbyte_sum() {
    assert_eq!(array_lowbyte_sum(), 10);
}

// =============================================================================
// SECTION 6: String Comparison + Branching
// =============================================================================

/// Different paths based on string equality
#[vm_protect(level = "standard")]
fn string_branch() -> u64 {
    let s1 = "test";
    let s2 = "test";
    let s3 = "other";

    let mut result: u64 = 0;

    if s1.eq(s2) {
        result += 100;
    }

    if s1.eq(s3) {
        result += 1000;  // Should not execute
    }

    result  // 100
}

#[test]
fn test_string_branch() {
    assert_eq!(string_branch(), 100);
}

/// Use string length in loop bound
#[vm_protect(level = "standard")]
fn string_driven_loop() -> u64 {
    let s = "count";  // len = 5
    let arr = [2, 4, 6, 8, 10, 12, 14, 16];

    // Sum first s.len() elements of array
    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < s.len() {
        sum += arr.get(i);
        i += 1;
    }
    sum  // 2 + 4 + 6 + 8 + 10 = 30
}

#[test]
fn test_string_driven_loop() {
    assert_eq!(string_driven_loop(), 30);
}

// =============================================================================
// SECTION 7: Wrapping Operations in Real Scenarios
// =============================================================================

/// Checksum-like computation
#[vm_protect(level = "standard")]
fn simple_checksum() -> u64 {
    let data = [0x12, 0x34, 0x56, 0x78, 0x9A];

    let mut sum: u64 = 0;
    let mut i: u64 = 0;
    while i < data.len() {
        sum = sum.wrapping_add(data.get(i));
        i += 1;
    }

    // XOR with length
    sum ^ data.len()
}

#[test]
fn test_simple_checksum() {
    let expected = (0x12u64 + 0x34 + 0x56 + 0x78 + 0x9A) ^ 5;
    assert_eq!(simple_checksum(), expected);
}

/// Overflow detection scenario
#[vm_protect(level = "standard")]
fn wrapping_overflow() -> u64 {
    let a: u64 = 0xFFFFFFFFFFFFFFFF;  // MAX
    let b: u64 = 10;

    let wrapped = a.wrapping_add(b);  // Should wrap to 9
    wrapped
}

#[test]
fn test_wrapping_overflow() {
    assert_eq!(wrapping_overflow(), 9);
}

// =============================================================================
// SECTION 8: Paranoid Level Mixed Tests
// =============================================================================

/// Paranoid: All features combined
#[vm_protect(level = "paranoid")]
fn paranoid_all_features() -> u64 {
    let arr = [100, 200, 300];
    let s = "xyz";

    // Array operations
    let arr_sum = arr.get(0) + arr.get(1) + arr.get(2);  // 600

    // String operations
    let str_len = s.len();  // 3
    let first_byte = s.get(0);  // 'x' = 120

    // Numeric methods
    let min_val = arr_sum.min(1000);  // 600
    let max_val = str_len.max(5);     // 5

    // Cast
    let byte_cast = (arr_sum as u8) as u64;  // 600 & 0xFF = 88

    // Combine all
    min_val + max_val + byte_cast + first_byte
    // 600 + 5 + 88 + 120 = 813
}

#[test]
fn test_paranoid_all_features() {
    assert_eq!(paranoid_all_features(), 813);
}

/// Paranoid: String-array interleaved
#[vm_protect(level = "paranoid")]
fn paranoid_interleaved() -> u64 {
    let s1 = "ab";
    let arr = [10, 20, 30];
    let s2 = "cde";

    let a = s1.len();           // 2
    let b = arr.get(1);          // 20
    let c = s2.get(0) as u64;    // 'c' = 99
    let d = arr.len();           // 3
    let e = s1.get(1) as u64;    // 'b' = 98

    a + b + c + d + e  // 2 + 20 + 99 + 3 + 98 = 222
}

#[test]
fn test_paranoid_interleaved() {
    assert_eq!(paranoid_interleaved(), 222);
}

// =============================================================================
// SECTION 9: Edge Cases
// =============================================================================

/// Empty string with array
#[vm_protect(level = "standard")]
fn empty_string_with_array() -> u64 {
    let s = "";
    let arr = [1, 2, 3];

    if s.is_empty() {
        arr.get(0) + arr.get(1) + arr.get(2)  // 6
    } else {
        0
    }
}

#[test]
fn test_empty_string_with_array() {
    assert_eq!(empty_string_with_array(), 6);
}

/// Single element scenarios
#[vm_protect(level = "standard")]
fn single_elements() -> u64 {
    let arr = [42];
    let s = "x";

    let a = arr.len();     // 1
    let b = arr.get(0);    // 42
    let c = s.len();       // 1
    let d = s.get(0);      // 'x' = 120

    a * b + c * d  // 1*42 + 1*120 = 162
}

#[test]
fn test_single_elements() {
    assert_eq!(single_elements(), 162);
}

/// Nested conditionals with mixed types
#[vm_protect(level = "standard")]
fn nested_mixed_conditionals() -> u64 {
    let arr = [5, 10, 15];
    let s = "hello";

    let result = if arr.len() > 2 {
        if s.len() == 5 {
            if arr.get(1) == 10 {
                if s.get(0) == 104 {  // 'h'
                    1000  // All conditions true
                } else {
                    100
                }
            } else {
                10
            }
        } else {
            1
        }
    } else {
        0
    };

    result
}

#[test]
fn test_nested_mixed_conditionals() {
    assert_eq!(nested_mixed_conditionals(), 1000);
}

// =============================================================================
// SECTION 10: Real-World-ish Scenarios
// =============================================================================

/// Simple scoring system
#[vm_protect(level = "standard")]
fn scoring_system() -> u64 {
    let scores = [85, 92, 78, 95, 88];
    let grade_name = "final";

    // Calculate average (integer division)
    let mut total: u64 = 0;
    let mut i: u64 = 0;
    while i < scores.len() {
        total += scores.get(i);
        i += 1;
    }
    let avg = total / scores.len();  // 438 / 5 = 87

    // Bonus based on grade name length
    let bonus = grade_name.len() * 2;  // 5 * 2 = 10

    avg + bonus  // 87 + 10 = 97
}

#[test]
fn test_scoring_system() {
    assert_eq!(scoring_system(), 97);
}

/// Data validation pattern
#[vm_protect(level = "standard")]
fn validate_data() -> u64 {
    let header = "HDR";
    let data = [0x48, 0x44, 0x52, 0x00];  // "HDR\0"

    // Check if data starts with header bytes
    let mut valid: u64 = 1;
    let mut i: u64 = 0;
    while i < header.len() {
        if header.get(i) != data.get(i) {
            valid = 0;
        }
        i += 1;
    }

    valid  // Should be 1 (all match)
}

#[test]
fn test_validate_data() {
    assert_eq!(validate_data(), 1);
}

/// XOR encryption/decryption pattern
#[vm_protect(level = "standard")]
fn xor_cipher() -> u64 {
    let key = "key";
    let data = [0x0B, 0x00, 0x1E];  // XOR'd with "key"

    // XOR each byte with key
    let mut result: u64 = 0;
    let mut i: u64 = 0;
    while i < data.len() {
        let decrypted = data.get(i) ^ key.get(i);
        result = result * 256 + decrypted;
        i += 1;
    }

    // data[0] ^ 'k' = 0x0B ^ 0x6B = 0x60 = 96
    // data[1] ^ 'e' = 0x00 ^ 0x65 = 0x65 = 101
    // data[2] ^ 'y' = 0x1E ^ 0x79 = 0x67 = 103
    // result = 96*256*256 + 101*256 + 103 = 6316391
    result
}

#[test]
fn test_xor_cipher() {
    // 'k' = 107, 'e' = 101, 'y' = 121
    // 0x0B ^ 107 = 11 ^ 107 = 96
    // 0x00 ^ 101 = 101
    // 0x1E ^ 121 = 30 ^ 121 = 103
    let expected = 96u64 * 256 * 256 + 101 * 256 + 103;
    assert_eq!(xor_cipher(), expected);
}
