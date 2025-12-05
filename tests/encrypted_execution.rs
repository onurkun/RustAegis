//! Integration tests for encrypted bytecode execution
//!
//! This demonstrates the full flow:
//! 1. Create VM bytecode
//! 2. Encrypt with AES-256-GCM
//! 3. Package with header
//! 4. Decrypt at runtime
//! 5. Execute through VM

use aegis_vm::{
    execute, VmError,
    bytecode::{BytecodeHeader, BytecodePackage, BytecodeFlags, ProtectionLevel},
    crypto::CryptoContext,
    build_config::opcodes::{stack, arithmetic, control, exec, register, native},
};

/// Helper: Execute encrypted bytecode
fn execute_encrypted(
    ctx: &CryptoContext,
    package: &BytecodePackage,
    input: &[u8],
) -> Result<u64, VmError> {
    // Check if encrypted
    if !package.header.is_encrypted() {
        // Plaintext execution
        return execute(&package.code, input);
    }

    // Decrypt
    let decrypted = ctx.decrypt(
        &package.code,
        &package.header.nonce,
        &package.header.tag,
    )?;

    // Execute
    execute(&decrypted, input)
}

#[test]
fn test_encrypted_simple_addition() {
    // Create bytecode: 40 + 2 = 42
    let bytecode = [
        stack::PUSH_IMM8, 40,
        stack::PUSH_IMM8, 2,
        arithmetic::ADD,
        exec::HALT,
    ];

    // Create crypto context with test seed
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    // Encrypt bytecode
    let (ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    // Create package with header
    let mut header = BytecodeHeader::new(
        ctx.build_id,
        1234567890,
        BytecodeFlags::Encrypted as u16,
    );
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage {
        header,
        code: ciphertext,
    };

    // Execute encrypted bytecode
    let result = execute_encrypted(&ctx, &package, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_encrypted_complex_arithmetic() {
    // Complex arithmetic: ((100 * 3) + 50) - 8 = 342
    let bytecode = [
        stack::PUSH_IMM8, 100,
        stack::PUSH_IMM8, 3,
        arithmetic::MUL,    // 300
        stack::PUSH_IMM8, 50,
        arithmetic::ADD,    // 350
        stack::PUSH_IMM8, 8,
        arithmetic::SUB,    // 342
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let (ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    let mut header = BytecodeHeader::new(ctx.build_id, 0, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    let result = execute_encrypted(&ctx, &package, &[]).unwrap();
    assert_eq!(result, 342);
}

#[test]
fn test_encrypted_with_input() {
    // Read input u64 and add 10
    let bytecode = [
        native::NATIVE_READ, 0x00, 0x00,  // Read u64 at offset 0
        stack::PUSH_IMM8, 10,
        arithmetic::ADD,
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let (ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    let mut header = BytecodeHeader::new(ctx.build_id, 0, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    // Input: u64 value 32, expected output: 32 + 10 = 42
    let input = 32u64.to_le_bytes();
    let result = execute_encrypted(&ctx, &package, &input).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_encrypted_loop() {
    // Sum 1 to 5 = 15
    let bytecode = [
        // R0 = 5 (counter)
        register::MOV_IMM, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        // R1 = 0 (sum)
        register::MOV_IMM, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        // loop:
        stack::PUSH_REG, 0,        // push counter
        stack::PUSH_REG, 1,        // push sum
        arithmetic::ADD,           // sum + counter
        stack::POP_REG, 1,         // store in R1
        stack::PUSH_REG, 0,        // push counter
        arithmetic::DEC,           // counter--
        stack::DUP,                // dup for check
        stack::POP_REG, 0,         // store back
        stack::PUSH_IMM8, 0,
        arithmetic::SUB,
        control::JNZ, (256 - 19) as u8, 0xFF,
        stack::PUSH_REG, 1,
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let (ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    let mut header = BytecodeHeader::new(ctx.build_id, 0, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    let result = execute_encrypted(&ctx, &package, &[]).unwrap();
    assert_eq!(result, 15);
}

#[test]
fn test_tampered_encrypted_bytecode_fails() {
    let bytecode = [
        stack::PUSH_IMM8, 42,
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let (mut ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    // Tamper with ciphertext
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0xFF;
    }

    let mut header = BytecodeHeader::new(ctx.build_id, 0, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    // Decryption should fail due to authentication
    let result = execute_encrypted(&ctx, &package, &[]);
    assert!(result.is_err());
}

/// Test that wrong key fails decryption
/// Note: When whitebox feature is enabled, CryptoContext always uses build-time
/// derived keys, so this test only applies to non-whitebox mode.
#[test]
#[cfg(not(feature = "whitebox"))]
fn test_wrong_key_fails() {
    let bytecode = [
        stack::PUSH_IMM8, 42,
        exec::HALT,
    ];

    // Encrypt with one key
    let seed1 = [0x42u8; 32];
    let mut ctx1 = CryptoContext::new(seed1);

    let (ciphertext, nonce, tag) = ctx1.encrypt(&bytecode).unwrap();

    let mut header = BytecodeHeader::new(ctx1.build_id, 0, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    // Try to decrypt with different key
    let seed2 = [0x43u8; 32];
    let ctx2 = CryptoContext::new(seed2);

    let result = execute_encrypted(&ctx2, &package, &[]);
    assert!(result.is_err());
}

#[test]
fn test_plaintext_mode() {
    // Test plaintext (debug) mode - no encryption
    let bytecode = vec![
        stack::PUSH_IMM8, 42,
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let ctx = CryptoContext::new(seed);

    // Create package without encryption flag
    let mut header = BytecodeHeader::new(ctx.build_id, 0, 0);  // No flags = plaintext
    header.code_len = bytecode.len() as u32;

    let package = BytecodePackage { header, code: bytecode };

    let result = execute_encrypted(&ctx, &package, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_package_serialization() {
    let bytecode = [
        stack::PUSH_IMM8, 42,
        exec::HALT,
    ];

    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let (ciphertext, nonce, tag) = ctx.encrypt(&bytecode).unwrap();

    let mut header = BytecodeHeader::new(ctx.build_id, 1234567890, BytecodeFlags::Encrypted as u16);
    header.nonce = nonce;
    header.tag = tag;
    header.code_len = ciphertext.len() as u32;

    let package = BytecodePackage { header, code: ciphertext };

    // Serialize to bytes
    let serialized = package.to_bytes();

    // Deserialize
    let deserialized = BytecodePackage::from_bytes(&serialized).unwrap();

    // Verify header
    assert_eq!(deserialized.header.build_id, ctx.build_id);
    assert_eq!(deserialized.header.timestamp, 1234567890);
    assert!(deserialized.header.is_encrypted());

    // Execute deserialized package
    let result = execute_encrypted(&ctx, &deserialized, &[]).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn test_protection_level_flags() {
    // Test different protection levels
    assert_eq!(ProtectionLevel::Debug.to_flags(), 0);
    assert_eq!(ProtectionLevel::Low.to_flags(), BytecodeFlags::Encrypted as u16);
    assert_eq!(
        ProtectionLevel::Medium.to_flags(),
        BytecodeFlags::Encrypted as u16 | BytecodeFlags::HasIntegrity as u16
    );
    assert_eq!(
        ProtectionLevel::Paranoid.to_flags(),
        BytecodeFlags::Encrypted as u16
            | BytecodeFlags::HasIntegrity as u16
            | BytecodeFlags::HasTimingChecks as u16
            | BytecodeFlags::Paranoid as u16
    );
}
