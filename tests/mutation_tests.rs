//! Tests for build-time mutated handlers
//!
//! These tests verify that mutated handlers produce correct results
//! regardless of which variant was selected at build time.

use aegis_vm::state::VmState;
use aegis_vm::handlers::mutation::*;

// =============================================================================
// ADD Handler Tests
// =============================================================================

#[test]
fn test_mutated_add_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(10).unwrap();
    state.push(20).unwrap();
    mutated_add(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 30);
}

#[test]
fn test_mutated_add_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(42).unwrap();
    state.push(0).unwrap();
    mutated_add(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 42);
}

#[test]
fn test_mutated_add_overflow() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(u64::MAX).unwrap();
    state.push(1).unwrap();
    mutated_add(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0); // Wrapping
}

#[test]
fn test_mutated_add_large() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0x1234567890ABCDEF).unwrap();
    state.push(0xFEDCBA0987654321).unwrap();
    mutated_add(&mut state).unwrap();
    let result = state.pop().unwrap();
    assert_eq!(result, 0x1234567890ABCDEFu64.wrapping_add(0xFEDCBA0987654321u64));
}

// =============================================================================
// SUB Handler Tests
// =============================================================================

#[test]
fn test_mutated_sub_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(30).unwrap();
    state.push(20).unwrap();
    mutated_sub(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 10);
}

#[test]
fn test_mutated_sub_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(42).unwrap();
    state.push(0).unwrap();
    mutated_sub(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 42);
}

#[test]
fn test_mutated_sub_underflow() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0).unwrap();
    state.push(1).unwrap();
    mutated_sub(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), u64::MAX); // Wrapping
}

#[test]
fn test_mutated_sub_large() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0xFEDCBA0987654321).unwrap();
    state.push(0x1234567890ABCDEF).unwrap();
    mutated_sub(&mut state).unwrap();
    let result = state.pop().unwrap();
    assert_eq!(result, 0xFEDCBA0987654321u64.wrapping_sub(0x1234567890ABCDEFu64));
}

// =============================================================================
// MUL Handler Tests
// =============================================================================

#[test]
fn test_mutated_mul_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(6).unwrap();
    state.push(7).unwrap();
    mutated_mul(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 42);
}

#[test]
fn test_mutated_mul_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(0).unwrap();
    mutated_mul(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

#[test]
fn test_mutated_mul_one() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(1).unwrap();
    mutated_mul(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

#[test]
fn test_mutated_mul_overflow() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(u64::MAX).unwrap();
    state.push(2).unwrap();
    mutated_mul(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), u64::MAX.wrapping_mul(2));
}

// =============================================================================
// XOR Handler Tests
// =============================================================================

#[test]
fn test_mutated_xor_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0b1100).unwrap();
    state.push(0b1010).unwrap();
    mutated_xor(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0b0110);
}

#[test]
fn test_mutated_xor_same() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(12345).unwrap();
    mutated_xor(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

#[test]
fn test_mutated_xor_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(0).unwrap();
    mutated_xor(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

#[test]
fn test_mutated_xor_max() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(u64::MAX).unwrap();
    mutated_xor(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), !12345u64);
}

// =============================================================================
// AND Handler Tests
// =============================================================================

#[test]
fn test_mutated_and_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0b1100).unwrap();
    state.push(0b1010).unwrap();
    mutated_and(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0b1000);
}

#[test]
fn test_mutated_and_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(0).unwrap();
    mutated_and(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

#[test]
fn test_mutated_and_max() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(u64::MAX).unwrap();
    mutated_and(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

// =============================================================================
// OR Handler Tests
// =============================================================================

#[test]
fn test_mutated_or_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0b1100).unwrap();
    state.push(0b1010).unwrap();
    mutated_or(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0b1110);
}

#[test]
fn test_mutated_or_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(0).unwrap();
    mutated_or(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

#[test]
fn test_mutated_or_max() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    state.push(u64::MAX).unwrap();
    mutated_or(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), u64::MAX);
}

// =============================================================================
// NOT Handler Tests
// =============================================================================

#[test]
fn test_mutated_not_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0b1100).unwrap();
    mutated_not(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), !0b1100u64);
}

#[test]
fn test_mutated_not_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0).unwrap();
    mutated_not(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), u64::MAX);
}

#[test]
fn test_mutated_not_max() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(u64::MAX).unwrap();
    mutated_not(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

#[test]
fn test_mutated_not_double() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    mutated_not(&mut state).unwrap();
    mutated_not(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

// =============================================================================
// INC Handler Tests
// =============================================================================

#[test]
fn test_mutated_inc_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(41).unwrap();
    mutated_inc(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 42);
}

#[test]
fn test_mutated_inc_zero() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0).unwrap();
    mutated_inc(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 1);
}

#[test]
fn test_mutated_inc_overflow() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(u64::MAX).unwrap();
    mutated_inc(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

// =============================================================================
// DEC Handler Tests
// =============================================================================

#[test]
fn test_mutated_dec_basic() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(43).unwrap();
    mutated_dec(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 42);
}

#[test]
fn test_mutated_dec_one() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(1).unwrap();
    mutated_dec(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 0);
}

#[test]
fn test_mutated_dec_underflow() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(0).unwrap();
    mutated_dec(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), u64::MAX);
}

// =============================================================================
// Combined Operations Tests
// =============================================================================

#[test]
fn test_mutated_add_sub_identity() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(100).unwrap();
    state.push(50).unwrap();
    mutated_add(&mut state).unwrap();
    state.push(50).unwrap();
    mutated_sub(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 100);
}

#[test]
fn test_mutated_xor_and_or_demorgan() {
    // Test De Morgan's law: a XOR b = (a OR b) AND NOT(a AND b)
    let a = 0b1100u64;
    let b = 0b1010u64;

    // Direct XOR
    let code = [];
    let mut state1 = VmState::new(&code, &[]);
    state1.push(a).unwrap();
    state1.push(b).unwrap();
    mutated_xor(&mut state1).unwrap();
    let xor_result = state1.pop().unwrap();

    // (a OR b) AND NOT(a AND b)
    let code2 = [];
    let mut state2 = VmState::new(&code2, &[]);
    state2.push(a).unwrap();
    state2.push(b).unwrap();
    mutated_or(&mut state2).unwrap();  // a | b
    let or_result = state2.pop().unwrap();

    state2.push(a).unwrap();
    state2.push(b).unwrap();
    mutated_and(&mut state2).unwrap();  // a & b
    mutated_not(&mut state2).unwrap();  // !(a & b)
    let not_and_result = state2.pop().unwrap();

    state2.push(or_result).unwrap();
    state2.push(not_and_result).unwrap();
    mutated_and(&mut state2).unwrap();  // (a | b) & !(a & b)
    let computed = state2.pop().unwrap();

    assert_eq!(xor_result, computed);
}

#[test]
fn test_mutated_inc_dec_identity() {
    let code = [];
    let mut state = VmState::new(&code, &[]);
    state.push(12345).unwrap();
    mutated_inc(&mut state).unwrap();
    mutated_dec(&mut state).unwrap();
    assert_eq!(state.pop().unwrap(), 12345);
}

#[test]
fn test_mutated_mul_add_distribution() {
    // Test: a * (b + c) = a*b + a*c
    let a = 5u64;
    let b = 3u64;
    let c = 7u64;

    // a * (b + c)
    let code1 = [];
    let mut state1 = VmState::new(&code1, &[]);
    state1.push(a).unwrap();
    state1.push(b).unwrap();
    state1.push(c).unwrap();
    mutated_add(&mut state1).unwrap();  // b + c on top
    let bc = state1.pop().unwrap();
    state1.push(bc).unwrap();
    mutated_mul(&mut state1).unwrap();  // a * (b+c)
    let left = state1.pop().unwrap();

    // a*b + a*c
    let code2 = [];
    let mut state2 = VmState::new(&code2, &[]);
    state2.push(a).unwrap();
    state2.push(b).unwrap();
    mutated_mul(&mut state2).unwrap();
    let ab = state2.pop().unwrap();

    state2.push(a).unwrap();
    state2.push(c).unwrap();
    mutated_mul(&mut state2).unwrap();
    let ac = state2.pop().unwrap();

    state2.push(ab).unwrap();
    state2.push(ac).unwrap();
    mutated_add(&mut state2).unwrap();
    let right = state2.pop().unwrap();

    assert_eq!(left, right);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_mutated_handlers_comprehensive() {
    // Run a comprehensive test with all handlers
    let code = [];
    let mut state = VmState::new(&code, &[]);

    // Test sequence: ((5 + 3) * 2) ^ 0xFF
    state.push(5).unwrap();
    state.push(3).unwrap();
    mutated_add(&mut state).unwrap();  // 8
    state.push(2).unwrap();
    mutated_mul(&mut state).unwrap();  // 16
    state.push(0xFF).unwrap();
    mutated_xor(&mut state).unwrap();  // 16 ^ 255 = 239

    assert_eq!(state.pop().unwrap(), 239);
}

#[test]
fn test_mutated_bitwise_comprehensive() {
    let code = [];
    let mut state = VmState::new(&code, &[]);

    // Verify: (a | b) & ~(a ^ b) == (a & b)
    let a = 0xABCDu64;
    let b = 0x1234u64;

    // Left side: (a | b) & ~(a ^ b)
    state.push(a).unwrap();
    state.push(b).unwrap();
    mutated_or(&mut state).unwrap();
    let or_ab = state.pop().unwrap();

    state.push(a).unwrap();
    state.push(b).unwrap();
    mutated_xor(&mut state).unwrap();
    mutated_not(&mut state).unwrap();
    let not_xor = state.pop().unwrap();

    state.push(or_ab).unwrap();
    state.push(not_xor).unwrap();
    mutated_and(&mut state).unwrap();
    let left = state.pop().unwrap();

    // Right side: a & b
    state.push(a).unwrap();
    state.push(b).unwrap();
    mutated_and(&mut state).unwrap();
    let right = state.pop().unwrap();

    assert_eq!(left, right);
}
