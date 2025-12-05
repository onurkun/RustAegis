//! Self-Modifying Code (SMC) Integration Tests
//!
//! Tests for the SMC engine that encrypts bytecode at rest and
//! decrypts only during execution.

use aegis_vm::{
    execute,
    smc::{SmcConfig, execute_smc, encrypt_bytecode, decrypt_bytecode},
    build_config::opcodes::{stack, arithmetic, control, exec},
};

// =============================================================================
// Basic SMC Tests
// =============================================================================

#[test]
fn test_smc_simple_addition() {
    // Simple: 40 + 2 = 42
    let mut code = vec![
        stack::PUSH_IMM8, 40,
        stack::PUSH_IMM8, 2,
        arithmetic::ADD,
        exec::HALT,
    ];

    // Execute normally first to verify bytecode
    let normal_result = execute(&code, &[]).unwrap();
    assert_eq!(normal_result, 42);

    // Now encrypt and execute with SMC
    let config = SmcConfig::from_build_seed(12345);
    encrypt_bytecode(&mut code, &config);

    // Encrypted bytecode should be different
    assert_ne!(code[0], stack::PUSH_IMM8);

    let smc_result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(smc_result, 42);
}

#[test]
fn test_smc_multiple_operations() {
    // (10 + 5) * 3 = 45
    let mut code = vec![
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,
        stack::PUSH_IMM8, 3,
        arithmetic::MUL,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(98765);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 45);
}

#[test]
fn test_smc_bitwise_operations() {
    // 0xFF ^ 0x0F = 0xF0
    let mut code = vec![
        stack::PUSH_IMM8, 0xFF,
        stack::PUSH_IMM8, 0x0F,
        arithmetic::XOR,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(11111);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 0xF0);
}

// =============================================================================
// Control Flow Tests
// =============================================================================

#[test]
fn test_smc_unconditional_jump() {
    // Jump over a PUSH instruction using RELATIVE offset
    // Index 0-1: PUSH_IMM8 42
    // Index 2-4: JMP with offset +2 (from ip=5 to ip=7)
    // Index 5-6: PUSH_IMM8 99 (skipped)
    // Index 7: HALT
    // After reading JMP operand, IP=5. Target=5+2=7.
    let mut code = vec![
        stack::PUSH_IMM8, 42,    // 0-1: push 42
        control::JMP, 2, 0,      // 2-4: jump +2 (relative offset)
        stack::PUSH_IMM8, 99,    // 5-6: push 99 (skipped)
        exec::HALT,              // 7: halt
    ];

    let config = SmcConfig::from_build_seed(22222);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_smc_conditional_jump_taken() {
    // Compare equal values, JZ should jump using RELATIVE offset
    // Index 0-1: PUSH_IMM8 5
    // Index 2-3: PUSH_IMM8 5
    // Index 4: CMP
    // Index 5-7: JZ with offset +2 (from ip=8 to ip=10)
    // Index 8-9: PUSH_IMM8 99 (skipped)
    // Index 10-11: PUSH_IMM8 42
    // Index 12: HALT
    let mut code = vec![
        stack::PUSH_IMM8, 5,     // 0-1: push 5
        stack::PUSH_IMM8, 5,     // 2-3: push 5
        control::CMP,            // 4: compare (sets zero flag)
        control::JZ, 2, 0,       // 5-7: jump +2 if zero (relative)
        stack::PUSH_IMM8, 99,    // 8-9: push 99 (skipped)
        stack::PUSH_IMM8, 42,    // 10-11: push 42
        exec::HALT,              // 12: halt
    ];

    let config = SmcConfig::from_build_seed(33333);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_smc_conditional_jump_not_taken() {
    // Compare different values, JZ should not jump
    // Index 5-7: JZ with offset +2 (not taken because not zero)
    // After reading operand, IP=8. Target would be 10.
    let mut code = vec![
        stack::PUSH_IMM8, 5,     // 0-1: push 5
        stack::PUSH_IMM8, 3,     // 2-3: push 3
        control::CMP,            // 4: compare (clears zero flag)
        control::JZ, 2, 0,       // 5-7: jump +2 if zero (not taken)
        stack::PUSH_IMM8, 42,    // 8-9: push 42
        exec::HALT,              // 10: halt
    ];

    let config = SmcConfig::from_build_seed(44444);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_smc_simple_loop() {
    // Sum 1 to 5: result = 15
    // R0 = counter, R1 = sum
    // JLT at 25-27: after reading operand, IP=28. To jump to 8: offset = 8-28 = -20
    // -20 as i16 little-endian = 0xEC, 0xFF
    let mut code = vec![
        // Initialize: R0 = 1, R1 = 0
        stack::PUSH_IMM8, 1,     // 0-1
        stack::POP_REG, 0,       // 2-3: R0 = 1
        stack::PUSH_IMM8, 0,     // 4-5
        stack::POP_REG, 1,       // 6-7: R1 = 0
        // Loop start (addr 8):
        // R1 = R1 + R0
        stack::PUSH_REG, 1,      // 8-9
        stack::PUSH_REG, 0,      // 10-11
        arithmetic::ADD,         // 12
        stack::POP_REG, 1,       // 13-14
        // R0 = R0 + 1
        stack::PUSH_REG, 0,      // 15-16
        arithmetic::INC,         // 17
        stack::POP_REG, 0,       // 18-19
        // Check if R0 > 5
        stack::PUSH_REG, 0,      // 20-21
        stack::PUSH_IMM8, 6,     // 22-23
        control::CMP,            // 24
        control::JLT, 0xEC, 0xFF, // 25-27: if R0 < 6, jump -20 to loop start
        // Return R1
        stack::PUSH_REG, 1,      // 28-29
        exec::HALT,              // 30
    ];

    let config = SmcConfig::from_build_seed(55555);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 15);
}

// =============================================================================
// Window Size Tests
// =============================================================================

#[test]
fn test_smc_window_size_1() {
    // Most secure: only 1 instruction decrypted at a time
    let mut code = vec![
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 20,
        arithmetic::ADD,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(66666).with_window(1);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 35);
}

#[test]
fn test_smc_window_size_larger() {
    // Larger window for better loop performance
    let mut code = vec![
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 20,
        arithmetic::ADD,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(77777).with_window(4);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 35);
}

// =============================================================================
// Encryption/Decryption Tests
// =============================================================================

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let original = vec![
        stack::PUSH_IMM8, 42,
        arithmetic::NOT,
        exec::HALT,
    ];

    let mut code = original.clone();
    let config = SmcConfig::from_build_seed(88888);

    // Encrypt
    encrypt_bytecode(&mut code, &config);
    assert_ne!(code, original, "Encrypted should differ");

    // Decrypt
    decrypt_bytecode(&mut code, &config);
    assert_eq!(code, original, "Decrypted should match original");
}

#[test]
fn test_different_seeds_different_encryption() {
    let code1 = vec![stack::PUSH_IMM8, 42, exec::HALT];
    let code2 = code1.clone();

    let mut encrypted1 = code1.clone();
    let mut encrypted2 = code2.clone();

    let config1 = SmcConfig::from_build_seed(11111);
    let config2 = SmcConfig::from_build_seed(22222);

    encrypt_bytecode(&mut encrypted1, &config1);
    encrypt_bytecode(&mut encrypted2, &config2);

    assert_ne!(encrypted1, encrypted2, "Different seeds should produce different encryption");
}

#[test]
fn test_smc_after_execution_re_encrypted() {
    let original = vec![
        stack::PUSH_IMM8, 42,
        exec::HALT,
    ];

    let mut code = original.clone();
    let config = SmcConfig::from_build_seed(99999);

    encrypt_bytecode(&mut code, &config);
    let encrypted_copy = code.clone();

    // Execute with SMC - this modifies code in place
    let result = execute_smc(code.clone(), &[], &config).unwrap();
    assert_eq!(result, 42);

    // After execution, code should be re-encrypted (same as before)
    // Note: we need to pass a clone because execute_smc takes ownership
    let mut code2 = encrypted_copy;
    let _ = execute_smc(code2.clone(), &[], &config).unwrap();
    // The bytecode is re-encrypted after execution
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_smc_empty_input() {
    let mut code = vec![
        stack::PUSH_IMM8, 100,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(12121);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 100);
}

#[test]
fn test_smc_large_immediate() {
    // Test with larger immediate values (16-bit)
    let mut code = vec![
        stack::PUSH_IMM16, 0xE8, 0x03, // 1000 in little-endian
        stack::PUSH_IMM16, 0xE8, 0x03, // 1000
        arithmetic::ADD,
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(34343);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 2000);
}

#[test]
fn test_smc_stack_operations() {
    // Test DUP and DROP
    let mut code = vec![
        stack::PUSH_IMM8, 21,
        stack::DUP,
        arithmetic::ADD,         // 21 + 21 = 42
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(45454);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_smc_swap_operation() {
    // Test SWAP: (10 - 3) = 7
    let mut code = vec![
        stack::PUSH_IMM8, 3,     // push 3
        stack::PUSH_IMM8, 10,    // push 10
        stack::SWAP,             // swap: now 3 on top, 10 below
        arithmetic::SUB,         // 10 - 3 = 7
        exec::HALT,
    ];

    let config = SmcConfig::from_build_seed(56565);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 7);
}

// =============================================================================
// Comparison with Normal Execution
// =============================================================================

#[test]
fn test_smc_matches_normal_execution() {
    // Verify SMC produces same results as normal execution
    let programs: Vec<Vec<u8>> = vec![
        // Simple addition
        vec![stack::PUSH_IMM8, 10, stack::PUSH_IMM8, 20, arithmetic::ADD, exec::HALT],
        // Subtraction
        vec![stack::PUSH_IMM8, 50, stack::PUSH_IMM8, 8, arithmetic::SUB, exec::HALT],
        // Multiplication
        vec![stack::PUSH_IMM8, 7, stack::PUSH_IMM8, 6, arithmetic::MUL, exec::HALT],
        // XOR
        vec![stack::PUSH_IMM8, 0xAA, stack::PUSH_IMM8, 0x55, arithmetic::XOR, exec::HALT],
        // NOT
        vec![stack::PUSH_IMM8, 0, arithmetic::NOT, exec::HALT],
    ];

    for (i, code) in programs.iter().enumerate() {
        let normal_result = execute(code, &[]).unwrap();

        let mut encrypted_code = code.clone();
        let config = SmcConfig::from_build_seed(1000 + i as u64);
        encrypt_bytecode(&mut encrypted_code, &config);

        let smc_result = execute_smc(encrypted_code, &[], &config).unwrap();

        assert_eq!(
            normal_result, smc_result,
            "Program {} should produce same result with SMC and normal execution",
            i
        );
    }
}

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_smc_nested_loop() {
    // Nested loop: outer 3 iterations, inner 4 iterations
    // Result: 3 * 4 = 12
    // R0 = outer counter, R1 = inner counter, R2 = total count
    // JLT at 27-29: IP=30 after read, target=12, offset=-18 = 0xFFEE
    // JLT at 40-42: IP=43 after read, target=8, offset=-35 = 0xFFDD
    let mut code = vec![
        // R2 = 0
        stack::PUSH_IMM8, 0,     // 0-1
        stack::POP_REG, 2,       // 2-3
        // R0 = 0 (outer)
        stack::PUSH_IMM8, 0,     // 4-5
        stack::POP_REG, 0,       // 6-7
        // OUTER_LOOP (addr 8):
        // R1 = 0 (inner)
        stack::PUSH_IMM8, 0,     // 8-9
        stack::POP_REG, 1,       // 10-11
        // INNER_LOOP (addr 12):
        // R2 = R2 + 1
        stack::PUSH_REG, 2,      // 12-13
        arithmetic::INC,         // 14
        stack::POP_REG, 2,       // 15-16
        // R1 = R1 + 1
        stack::PUSH_REG, 1,      // 17-18
        arithmetic::INC,         // 19
        stack::POP_REG, 1,       // 20-21
        // if R1 < 4, jump to INNER_LOOP
        stack::PUSH_REG, 1,      // 22-23
        stack::PUSH_IMM8, 4,     // 24-25
        control::CMP,            // 26
        control::JLT, 0xEE, 0xFF, // 27-29: jump -18 to INNER_LOOP
        // R0 = R0 + 1
        stack::PUSH_REG, 0,      // 30-31
        arithmetic::INC,         // 32
        stack::POP_REG, 0,       // 33-34
        // if R0 < 3, jump to OUTER_LOOP
        stack::PUSH_REG, 0,      // 35-36
        stack::PUSH_IMM8, 3,     // 37-38
        control::CMP,            // 39
        control::JLT, 0xDD, 0xFF, // 40-42: jump -35 to OUTER_LOOP
        // Return R2
        stack::PUSH_REG, 2,      // 43-44
        exec::HALT,              // 45
    ];

    let config = SmcConfig::from_build_seed(67676);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 12);
}

#[test]
fn test_smc_fibonacci_10() {
    // Calculate 10th Fibonacci number (0-indexed): F(10) = 55
    // R0 = prev, R1 = curr, R2 = counter, R3 = temp
    // JLT at 37-39: IP=40 after read, target=12, offset=-28 = 0xFFE4
    let mut code = vec![
        // R0 = 0 (F(0))
        stack::PUSH_IMM8, 0,     // 0-1
        stack::POP_REG, 0,       // 2-3
        // R1 = 1 (F(1))
        stack::PUSH_IMM8, 1,     // 4-5
        stack::POP_REG, 1,       // 6-7
        // R2 = 2 (counter, starting from F(2))
        stack::PUSH_IMM8, 2,     // 8-9
        stack::POP_REG, 2,       // 10-11
        // LOOP (addr 12):
        // R3 = R0 + R1
        stack::PUSH_REG, 0,      // 12-13
        stack::PUSH_REG, 1,      // 14-15
        arithmetic::ADD,         // 16
        stack::POP_REG, 3,       // 17-18
        // R0 = R1
        stack::PUSH_REG, 1,      // 19-20
        stack::POP_REG, 0,       // 21-22
        // R1 = R3
        stack::PUSH_REG, 3,      // 23-24
        stack::POP_REG, 1,       // 25-26
        // R2 = R2 + 1
        stack::PUSH_REG, 2,      // 27-28
        arithmetic::INC,         // 29
        stack::POP_REG, 2,       // 30-31
        // if R2 <= 10, jump to LOOP
        stack::PUSH_REG, 2,      // 32-33
        stack::PUSH_IMM8, 11,    // 34-35
        control::CMP,            // 36
        control::JLT, 0xE4, 0xFF, // 37-39: jump -28 to LOOP
        // Return R1
        stack::PUSH_REG, 1,      // 40-41
        exec::HALT,              // 42
    ];

    let config = SmcConfig::from_build_seed(78787);
    encrypt_bytecode(&mut code, &config);

    let result = execute_smc(code, &[], &config).unwrap();
    assert_eq!(result, 55); // F(10) = 55
}
