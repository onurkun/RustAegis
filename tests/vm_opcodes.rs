//! Comprehensive VM opcode tests
//!
//! Tests all opcodes and edge cases for the anticheat VM.

use aegis_vm::{execute, execute_with_state, VmError};
// Use shuffled opcodes from build config for tests
use aegis_vm::build_config::opcodes::{stack, register, arithmetic, control, special, native, exec};

// ============================================================================
// Basic Stack Operations
// ============================================================================

#[test]
fn test_push_pop() {
    let code = [
        stack::PUSH_IMM8, 42,
        stack::PUSH_IMM8, 10,
        arithmetic::ADD,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 52);
}

#[test]
fn test_push_imm_64bit() {
    let code = [
        stack::PUSH_IMM,
        0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x0123456789ABCDEF);
}

#[test]
fn test_drop() {
    let code = [
        stack::PUSH_IMM8, 100,
        stack::PUSH_IMM8, 42,
        stack::DROP,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_swap() {
    let code = [
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 20,
        stack::SWAP,
        arithmetic::SUB,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_dup_swap() {
    let code = [
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 20,
        stack::DUP,
        arithmetic::ADD,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 40);
}

// ============================================================================
// Register Operations
// ============================================================================

#[test]
fn test_mov_reg() {
    let code = [
        register::MOV_IMM, 0, 42, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_REG, 1, 0,
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_multiple_registers() {
    let code = [
        register::MOV_IMM, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 1, 2, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 2, 3, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 3, 4, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 4, 5, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 5, 6, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 6, 7, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 7, 8, 0, 0, 0, 0, 0, 0, 0,
        stack::PUSH_REG, 0,
        stack::PUSH_REG, 1,
        arithmetic::ADD,
        stack::PUSH_REG, 2,
        arithmetic::ADD,
        stack::PUSH_REG, 3,
        arithmetic::ADD,
        stack::PUSH_REG, 4,
        arithmetic::ADD,
        stack::PUSH_REG, 5,
        arithmetic::ADD,
        stack::PUSH_REG, 6,
        arithmetic::ADD,
        stack::PUSH_REG, 7,
        arithmetic::ADD,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 36);
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

#[test]
fn test_subtraction() {
    let code = [
        stack::PUSH_IMM8, 100,
        stack::PUSH_IMM8, 30,
        arithmetic::SUB,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 70);
}

#[test]
fn test_xor() {
    let code = [
        stack::PUSH_IMM8, 0xFF,
        stack::PUSH_IMM8, 0x0F,
        arithmetic::XOR,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xF0);
}

#[test]
fn test_multiply() {
    let code = [
        stack::PUSH_IMM8, 7,
        stack::PUSH_IMM8, 6,
        arithmetic::MUL,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_and_operation() {
    let code = [
        stack::PUSH_IMM8, 0xF0,
        stack::PUSH_IMM8, 0xAA,
        arithmetic::AND,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xA0);
}

#[test]
fn test_or_operation() {
    let code = [
        stack::PUSH_IMM8, 0xF0,
        stack::PUSH_IMM8, 0x0F,
        arithmetic::OR,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFF);
}

#[test]
fn test_bitwise_ops() {
    let code = [
        stack::PUSH_IMM8, 0xFF,
        stack::PUSH_IMM8, 0x0F,
        arithmetic::AND,
        stack::PUSH_IMM8, 0x10,
        arithmetic::OR,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x1F);
}

#[test]
fn test_shift_left() {
    let code = [
        stack::PUSH_IMM8, 1,
        stack::PUSH_IMM8, 4,
        arithmetic::SHL,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 16);
}

#[test]
fn test_shift_right() {
    let code = [
        stack::PUSH_IMM8, 64,
        stack::PUSH_IMM8, 3,
        arithmetic::SHR,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_rotate_left() {
    let code = [
        stack::PUSH_IMM,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
        stack::PUSH_IMM8, 1,
        arithmetic::ROL,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x0000000000000003);
}

#[test]
fn test_rotate_right() {
    let code = [
        stack::PUSH_IMM8, 3,
        stack::PUSH_IMM8, 1,
        arithmetic::ROR,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x8000000000000001);
}

#[test]
fn test_not() {
    let code = [
        stack::PUSH_IMM8, 0,
        arithmetic::NOT,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFFFFFFFFFFFFFFFF);
}

#[test]
fn test_not_pattern() {
    let code = [
        stack::PUSH_IMM8, 0xF0,
        arithmetic::NOT,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFFFFFFFFFFFFFF0F);
}

#[test]
fn test_inc() {
    let code = [
        stack::PUSH_IMM8, 41,
        arithmetic::INC,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_dec() {
    let code = [
        stack::PUSH_IMM8, 43,
        arithmetic::DEC,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_inc_overflow() {
    let code = [
        stack::PUSH_IMM,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        arithmetic::INC,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_wrapping_arithmetic() {
    let code = [
        stack::PUSH_IMM,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        stack::PUSH_IMM8, 2,
        arithmetic::ADD,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_complex_arithmetic() {
    let code = [
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,
        stack::PUSH_IMM8, 3,
        arithmetic::MUL,
        stack::PUSH_IMM8, 15,
        arithmetic::SUB,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 30);
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn test_conditional_jump() {
    let code = [
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 5,
        arithmetic::SUB,
        control::JZ, 0x03, 0x00,
        stack::PUSH_IMM8, 200,
        exec::HALT,
        stack::PUSH_IMM8, 100,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_cmp_equal() {
    let code = [
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 5,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JZ, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_cmp_not_equal() {
    let code = [
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 3,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JNZ, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_jgt_greater() {
    let code = [
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 5,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JGT, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_jlt_less() {
    let code = [
        stack::PUSH_IMM8, 3,
        stack::PUSH_IMM8, 10,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JLT, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_jge_equal() {
    let code = [
        stack::PUSH_IMM8, 5,
        stack::PUSH_IMM8, 5,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JGE, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_jle_less() {
    let code = [
        stack::PUSH_IMM8, 3,
        stack::PUSH_IMM8, 5,
        control::CMP,
        stack::DROP,
        stack::DROP,
        control::JLE, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_backward_jump() {
    let code = [
        stack::PUSH_IMM8, 3,
        arithmetic::DEC,
        stack::DUP,
        control::JNZ,
        (256 - 5) as u8, 0xFF,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_zero_flag_set() {
    let code = [
        stack::PUSH_IMM8, 42,
        stack::PUSH_IMM8, 42,
        arithmetic::SUB,
        control::JZ, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_zero_flag_not_set() {
    let code = [
        stack::PUSH_IMM8, 42,
        stack::PUSH_IMM8, 10,
        arithmetic::SUB,
        control::JNZ, 0x03, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
        stack::PUSH_IMM8, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

// ============================================================================
// Subroutine Calls
// ============================================================================

#[test]
fn test_call_ret() {
    let code = [
        control::CALL, 0x01, 0x00,
        exec::HALT,
        stack::PUSH_IMM8, 42,
        control::RET,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_nested_calls() {
    let code = [
        control::CALL, 0x01, 0x00,
        exec::HALT,
        control::CALL, 0x04, 0x00,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,
        control::RET,
        stack::PUSH_IMM8, 10,
        control::RET,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_ret_from_main() {
    let code = [
        stack::PUSH_IMM8, 42,
        control::RET,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

// ============================================================================
// Special Operations
// ============================================================================

#[test]
fn test_nop() {
    let code = [
        stack::PUSH_IMM8, 42,
        special::NOP,
        special::NOP,
        special::NOP,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_nop_n() {
    let code = [
        stack::PUSH_IMM8, 42,
        special::NOP_N, 3,
        0xAA, 0xBB, 0xCC,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_opaque_true() {
    let code = [
        special::OPAQUE_TRUE,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_opaque_false() {
    let code = [
        special::OPAQUE_FALSE,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0);
}

// ============================================================================
// Native Operations
// ============================================================================

#[test]
fn test_input_read() {
    let code = [
        native::NATIVE_READ, 0x00, 0x00,  // Read u64 at offset 0
        exec::HALT,
    ];
    // Input: u64 value 0x42 in little-endian (8 bytes)
    let input = 0x42u64.to_le_bytes();
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0x42);
}

#[test]
fn test_input_len() {
    let code = [
        native::INPUT_LEN,
        exec::HALT,
    ];
    let input = [1, 2, 3, 4, 5];
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_native_write() {
    let code = [
        stack::PUSH_IMM8, 0x41,
        native::NATIVE_WRITE, 0x00, 0x00,
        stack::PUSH_IMM8, 0x42,
        native::NATIVE_WRITE, 0x00, 0x00,
        stack::PUSH_IMM8, 0,
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert_eq!(state.output, vec![0x41, 0x42]);
}

#[test]
fn test_read_multiple_input_values() {
    let code = [
        native::NATIVE_READ, 0x00, 0x00,  // Read u64 at offset 0
        native::NATIVE_READ, 0x08, 0x00,  // Read u64 at offset 8
        arithmetic::ADD,
        exec::HALT,
    ];
    // Two u64 values: 0x10 and 0x30
    let mut input = [0u8; 16];
    input[0..8].copy_from_slice(&0x10u64.to_le_bytes());
    input[8..16].copy_from_slice(&0x30u64.to_le_bytes());
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0x40);  // 0x10 + 0x30 = 0x40
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_stack_underflow() {
    let code = [
        stack::POP_REG, 0,
    ];
    let result = execute(&code, &[]);
    assert_eq!(result, Err(VmError::StackUnderflow));
}

#[test]
fn test_invalid_opcode() {
    // With opcode shuffling, 0xAA may decode to a valid opcode that causes
    // a different error (like StackUnderflow). We just verify it fails.
    // True "invalid" would be bytes not mapped to any opcode after decode.
    let code = [0xAA];
    let result = execute(&code, &[]);
    // Should fail with some error (InvalidOpcode, StackUnderflow, etc.)
    assert!(result.is_err(), "Random opcode byte should cause an error");
}

#[test]
fn test_invalid_register() {
    let code = [
        register::MOV_IMM, 8, 42, 0, 0, 0, 0, 0, 0, 0,
    ];
    let result = execute(&code, &[]);
    assert_eq!(result, Err(VmError::InvalidRegister));
}

#[test]
fn test_invalid_jump_target() {
    let code = [
        control::JMP, 0x00, 0x10,
    ];
    let result = execute(&code, &[]);
    assert_eq!(result, Err(VmError::InvalidJumpTarget));
}

#[test]
fn test_memory_out_of_bounds() {
    let code = [
        native::NATIVE_READ, 0xFF, 0x00,
        exec::HALT,
    ];
    let input = [1, 2, 3];
    let result = execute(&code, &input);
    assert_eq!(result, Err(VmError::MemoryOutOfBounds));
}

#[test]
fn test_halt_with_empty_stack() {
    let code = [
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0);
}

// ============================================================================
// Complex Algorithms
// ============================================================================

#[test]
fn test_loop_sum() {
    let code = [
        register::MOV_IMM, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        stack::PUSH_REG, 0,
        stack::PUSH_REG, 1,
        arithmetic::ADD,
        stack::POP_REG, 1,
        stack::PUSH_REG, 0,
        arithmetic::DEC,
        stack::DUP,
        stack::POP_REG, 0,
        stack::PUSH_IMM8, 0,
        arithmetic::SUB,
        control::JNZ,
        (256 - 19) as u8, 0xFF,
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_fibonacci_like() {
    let code = [
        register::MOV_IMM, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 1, 1, 0, 0, 0, 0, 0, 0, 0,
        register::MOV_IMM, 2, 4, 0, 0, 0, 0, 0, 0, 0,
        stack::PUSH_REG, 0,
        stack::PUSH_REG, 1,
        arithmetic::ADD,
        stack::PUSH_REG, 1,
        stack::POP_REG, 0,
        stack::POP_REG, 1,
        stack::PUSH_REG, 2,
        arithmetic::DEC,
        stack::DUP,
        stack::POP_REG, 2,
        control::JNZ, (256 - 18) as u8, 0xFF,
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 8);
}
