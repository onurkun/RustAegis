//! FNV-1a hash tests - both native Rust and VM bytecode implementations
//!
//! Note: FNV constants are randomized per-build, so we test properties
//! rather than specific hash values.

use aegis_vm::{execute, fnv1a_hash, fnv1a_hash32};
use aegis_vm::build_config::{FNV_BASIS_64, FNV_PRIME_64};
use aegis_vm::build_config::opcodes::{stack, arithmetic, exec};

#[test]
fn test_fnv1a_consistency() {
    // Same input should produce same hash
    let hash1 = fnv1a_hash(b"hello");
    let hash2 = fnv1a_hash(b"hello");
    assert_eq!(hash1, hash2);

    // Different inputs should produce different hashes
    let hash3 = fnv1a_hash(b"world");
    assert_ne!(hash1, hash3);
}

#[test]
fn test_fnv1a_32_consistency() {
    // Same input should produce same hash
    let hash1 = fnv1a_hash32(b"hello");
    let hash2 = fnv1a_hash32(b"hello");
    assert_eq!(hash1, hash2);

    // Different inputs should produce different hashes
    let hash3 = fnv1a_hash32(b"world");
    assert_ne!(hash1, hash3);
}

#[test]
fn test_simple_execution() {
    let code = [
        stack::PUSH_IMM8, 2,
        stack::PUSH_IMM8, 3,
        arithmetic::ADD,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_fnv1a_bytecode() {
    // FNV-1a for single byte 'A' (0x41) using build-time constants
    let basis_bytes = FNV_BASIS_64.to_le_bytes();
    let prime_bytes = FNV_PRIME_64.to_le_bytes();

    let code = [
        stack::PUSH_IMM,
        basis_bytes[0], basis_bytes[1], basis_bytes[2], basis_bytes[3],
        basis_bytes[4], basis_bytes[5], basis_bytes[6], basis_bytes[7],
        stack::PUSH_IMM8, 0x41,
        arithmetic::XOR,
        stack::PUSH_IMM,
        prime_bytes[0], prime_bytes[1], prime_bytes[2], prime_bytes[3],
        prime_bytes[4], prime_bytes[5], prime_bytes[6], prime_bytes[7],
        arithmetic::MUL,
        exec::HALT,
    ];
    let result = execute(&code, &[]).unwrap();
    let expected = fnv1a_hash(b"A");
    assert_eq!(result, expected);
}

#[test]
fn test_fnv1a_bytecode_loop() {
    // Full FNV-1a for "hi" (2 bytes) - unrolled loop using build-time constants
    let basis_bytes = FNV_BASIS_64.to_le_bytes();
    let prime_bytes = FNV_PRIME_64.to_le_bytes();

    let code = [
        // Start with basis
        stack::PUSH_IMM,
        basis_bytes[0], basis_bytes[1], basis_bytes[2], basis_bytes[3],
        basis_bytes[4], basis_bytes[5], basis_bytes[6], basis_bytes[7],
        // XOR with 'h'
        stack::PUSH_IMM8, b'h',
        arithmetic::XOR,
        // Multiply by prime
        stack::PUSH_IMM,
        prime_bytes[0], prime_bytes[1], prime_bytes[2], prime_bytes[3],
        prime_bytes[4], prime_bytes[5], prime_bytes[6], prime_bytes[7],
        arithmetic::MUL,
        // XOR with 'i'
        stack::PUSH_IMM8, b'i',
        arithmetic::XOR,
        // Multiply by prime
        stack::PUSH_IMM,
        prime_bytes[0], prime_bytes[1], prime_bytes[2], prime_bytes[3],
        prime_bytes[4], prime_bytes[5], prime_bytes[6], prime_bytes[7],
        arithmetic::MUL,
        exec::HALT,
    ];
    let result = execute(&code, b"hi").unwrap();
    let expected = fnv1a_hash(b"hi");
    assert_eq!(result, expected);
}
