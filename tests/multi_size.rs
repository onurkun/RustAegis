//! Tests for multi-size integer operations
//! Tests PUSH_IMM16, PUSH_IMM32, type conversions (SEXT/TRUNC), and sized memory ops

use aegis_vm::engine::execute;
use aegis_vm::build_config::opcodes::{stack, convert, memory, exec};

// ============================================================================
// PUSH_IMM16 and PUSH_IMM32 Tests
// ============================================================================

#[test]
fn test_push_imm16() {
    // PUSH_IMM16 0x1234, HALT
    let code = vec![
        stack::PUSH_IMM16, 0x34, 0x12,  // Push 0x1234 (little-endian)
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x1234);
}

#[test]
fn test_push_imm16_max() {
    // PUSH_IMM16 0xFFFF
    let code = vec![
        stack::PUSH_IMM16, 0xFF, 0xFF,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFFFF);
}

#[test]
fn test_push_imm32() {
    // PUSH_IMM32 0x12345678
    let code = vec![
        stack::PUSH_IMM32, 0x78, 0x56, 0x34, 0x12,  // Little-endian
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x12345678);
}

#[test]
fn test_push_imm32_max() {
    // PUSH_IMM32 0xFFFFFFFF
    let code = vec![
        stack::PUSH_IMM32, 0xFF, 0xFF, 0xFF, 0xFF,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFFFFFFFF);
}

// ============================================================================
// Sign Extension Tests
// ============================================================================

#[test]
fn test_sext8_positive() {
    // PUSH 0x7F (127), SEXT8, HALT
    let code = vec![
        stack::PUSH_IMM8, 0x7F,
        convert::SEXT8,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 127);
}

#[test]
fn test_sext8_negative() {
    // PUSH 0x80 (-128 as i8), SEXT8, HALT
    let code = vec![
        stack::PUSH_IMM8, 0x80,
        convert::SEXT8,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    // -128 as i8 sign-extended to i64 = 0xFFFFFFFFFFFFFF80
    assert_eq!(result as i64, -128);
}

#[test]
fn test_sext8_minus_one() {
    // PUSH 0xFF (-1 as i8), SEXT8, HALT
    let code = vec![
        stack::PUSH_IMM8, 0xFF,
        convert::SEXT8,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result as i64, -1);
}

#[test]
fn test_sext16_positive() {
    // PUSH 0x7FFF, SEXT16, HALT
    let code = vec![
        stack::PUSH_IMM16, 0xFF, 0x7F,  // 0x7FFF
        convert::SEXT16,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x7FFF);
}

#[test]
fn test_sext16_negative() {
    // PUSH 0x8000 (-32768 as i16), SEXT16, HALT
    let code = vec![
        stack::PUSH_IMM16, 0x00, 0x80,  // 0x8000
        convert::SEXT16,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result as i64, -32768);
}

#[test]
fn test_sext32_positive() {
    // PUSH 0x7FFFFFFF, SEXT32, HALT
    let code = vec![
        stack::PUSH_IMM32, 0xFF, 0xFF, 0xFF, 0x7F,  // 0x7FFFFFFF
        convert::SEXT32,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x7FFFFFFF);
}

#[test]
fn test_sext32_negative() {
    // PUSH 0x80000000 (-2147483648 as i32), SEXT32, HALT
    let code = vec![
        stack::PUSH_IMM32, 0x00, 0x00, 0x00, 0x80,  // 0x80000000
        convert::SEXT32,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result as i64, -2147483648i64);
}

// ============================================================================
// Truncation Tests
// ============================================================================

#[test]
fn test_trunc8() {
    // PUSH 0x123456FF, TRUNC8, HALT
    let code = vec![
        stack::PUSH_IMM32, 0xFF, 0x56, 0x34, 0x12,
        convert::TRUNC8,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFF);
}

#[test]
fn test_trunc16() {
    // PUSH 0x12345678, TRUNC16, HALT
    let code = vec![
        stack::PUSH_IMM32, 0x78, 0x56, 0x34, 0x12,
        convert::TRUNC16,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x5678);
}

#[test]
fn test_trunc32() {
    // PUSH 0x123456789ABCDEF0, TRUNC32, HALT
    let code = vec![
        stack::PUSH_IMM,
        0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12,  // Little-endian
        convert::TRUNC32,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0x9ABCDEF0);
}

// ============================================================================
// Sized Memory Load Tests
// ============================================================================

#[test]
fn test_load8() {
    // Input: [0xAB, 0xCD, 0xEF]
    // LOAD8 at offset 1 should give 0xCD
    let code = vec![
        memory::LOAD8, 0x01, 0x00,  // Load 8-bit from offset 1
        exec::HALT,
    ];
    let input = vec![0xAB, 0xCD, 0xEF];
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0xCD);
}

#[test]
fn test_load16() {
    // Input: [0x12, 0x34, 0x56, 0x78]
    // LOAD16 at offset 1 should give 0x5634 (little-endian: bytes[1]=0x34, bytes[2]=0x56)
    let code = vec![
        memory::LOAD16, 0x01, 0x00,
        exec::HALT,
    ];
    let input = vec![0x12, 0x34, 0x56, 0x78];
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0x5634);
}

#[test]
fn test_load32() {
    // Input: [0x12, 0x34, 0x56, 0x78, 0x9A]
    // LOAD32 at offset 0 should give 0x78563412
    let code = vec![
        memory::LOAD32, 0x00, 0x00,
        exec::HALT,
    ];
    let input = vec![0x12, 0x34, 0x56, 0x78, 0x9A];
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0x78563412);
}

#[test]
fn test_load64() {
    // Input: 8 bytes
    let code = vec![
        memory::LOAD64, 0x00, 0x00,
        exec::HALT,
    ];
    let input = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let result = execute(&code, &input).unwrap();
    assert_eq!(result, 0x0807060504030201);
}

// ============================================================================
// Sized Memory Store Tests (verify output buffer)
// ============================================================================

#[test]
fn test_store8() {
    use aegis_vm::engine::execute_with_state;

    let code = vec![
        stack::PUSH_IMM8, 0xAB,
        memory::STORE8, 0x00, 0x00,  // Store to offset 0
        stack::PUSH_IMM8, 0x00,       // Push result for HALT
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert!(state.output.len() >= 1);
    assert_eq!(state.output[0], 0xAB);
}

#[test]
fn test_store16() {
    use aegis_vm::engine::execute_with_state;

    let code = vec![
        stack::PUSH_IMM16, 0x34, 0x12,  // 0x1234
        memory::STORE16, 0x00, 0x00,
        stack::PUSH_IMM8, 0x00,
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert!(state.output.len() >= 2);
    assert_eq!(state.output[0], 0x34);  // Little-endian low byte
    assert_eq!(state.output[1], 0x12);  // Little-endian high byte
}

#[test]
fn test_store32() {
    use aegis_vm::engine::execute_with_state;

    let code = vec![
        stack::PUSH_IMM32, 0x78, 0x56, 0x34, 0x12,  // 0x12345678
        memory::STORE32, 0x00, 0x00,
        stack::PUSH_IMM8, 0x00,
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert!(state.output.len() >= 4);
    assert_eq!(state.output[0..4], [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn test_store64() {
    use aegis_vm::engine::execute_with_state;

    let code = vec![
        stack::PUSH_IMM,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        memory::STORE64, 0x00, 0x00,
        stack::PUSH_IMM8, 0x00,
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert!(state.output.len() >= 8);
    assert_eq!(state.output[0..8], [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
}

// ============================================================================
// Combined Tests
// ============================================================================

#[test]
fn test_sext_trunc_roundtrip() {
    // Push -1 as 8-bit, sign-extend, truncate back to 8-bit
    let code = vec![
        stack::PUSH_IMM8, 0xFF,     // -1 as u8
        convert::SEXT8,              // Sign-extend to i64
        convert::TRUNC8,             // Truncate back to 8-bit
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0xFF);
}

#[test]
fn test_chained_conversions() {
    // Push value, extend, truncate through multiple sizes
    let code = vec![
        stack::PUSH_IMM8, 0x80,     // -128 as i8
        convert::SEXT8,              // Sign-extend to i64
        convert::TRUNC16,            // Truncate to 16-bit (keeps sign)
        convert::SEXT16,             // Sign-extend 16-bit back to 64-bit
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    // 0x80 -> 0xFFFFFFFFFFFFFF80 -> 0xFF80 -> 0xFFFFFFFFFFFFFF80
    assert_eq!(result as i64, -128);
}

#[test]
fn test_load_different_sizes_same_data() {
    // Same data, load as different sizes
    let input = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

    // Load u8
    let code8 = vec![memory::LOAD8, 0x00, 0x00, exec::HALT];
    assert_eq!(execute(&code8, &input).unwrap(), 0x01);

    // Load u16
    let code16 = vec![memory::LOAD16, 0x00, 0x00, exec::HALT];
    assert_eq!(execute(&code16, &input).unwrap(), 0x0201);

    // Load u32
    let code32 = vec![memory::LOAD32, 0x00, 0x00, exec::HALT];
    assert_eq!(execute(&code32, &input).unwrap(), 0x04030201);

    // Load u64
    let code64 = vec![memory::LOAD64, 0x00, 0x00, exec::HALT];
    assert_eq!(execute(&code64, &input).unwrap(), 0x0807060504030201);
}

#[test]
fn test_store_at_offset() {
    use aegis_vm::engine::execute_with_state;

    // Store at non-zero offset
    let code = vec![
        stack::PUSH_IMM16, 0xAB, 0xCD,  // 0xCDAB
        memory::STORE16, 0x02, 0x00,    // Store at offset 2
        stack::PUSH_IMM8, 0x00,
        exec::HALT,
    ];
    let state = execute_with_state(&code, &[]).unwrap();
    assert!(state.output.len() >= 4);
    assert_eq!(state.output[2], 0xAB);
    assert_eq!(state.output[3], 0xCD);
}
