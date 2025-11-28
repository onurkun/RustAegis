//! Tests for encrypted #[vm_protect] proc-macro
//!
//! Default mode (no level attribute) uses encrypted bytecode.
//! Note: This requires matching BUILD_SEED between compile-time and runtime.

use aegis_vm_macro::vm_protect;

// ============================================================================
// Encrypted Mode Tests (default - AES-256-GCM encrypted bytecode)
// ============================================================================

/// Simple encrypted function
#[vm_protect]
fn encrypted_add(x: u64) -> u64 {
    x + 100
}

/// Encrypted function with secret constant
#[vm_protect]
fn encrypted_secret(x: u64) -> u64 {
    x ^ 0xCAFEBABE
}

#[test]
fn test_encrypted_add() {
    assert_eq!(encrypted_add(0), 100);
    assert_eq!(encrypted_add(42), 142);
}

#[test]
fn test_encrypted_secret() {
    let secret = 0xCAFEBABE;
    assert_eq!(encrypted_secret(0), secret);
    assert_eq!(encrypted_secret(secret), 0);
}
