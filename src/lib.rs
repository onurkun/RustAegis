//! # RustAegis VM
//!
//! Custom Virtual Machine for code virtualization and protection.
//!
//! This VM converts critical Rust code to custom bytecode that is interpreted
//! at runtime, making static analysis and patching extremely difficult.
//!
//! ## Features
//!
//! - **Hybrid Architecture**: Stack + Register based for flexibility
//! - **60+ Opcodes**: Complete instruction set for complex logic
//! - **Anti-Analysis**: Opaque predicates, timing checks, integrity verification
//! - **Cross-Platform**: Works on iOS and Android
//! - **OLLVM Compatible**: Uses fastrand instead of rand crate
//!
//! ## Example
//!
//! ```rust
//! use aegis_vm::{execute, build_config::opcodes::{stack, arithmetic, exec}};
//!
//! // Simple bytecode: 40 + 2 = 42
//! let bytecode = [
//!     stack::PUSH_IMM8, 40,
//!     stack::PUSH_IMM8, 2,
//!     arithmetic::ADD,
//!     exec::HALT,
//! ];
//!
//! let result = execute(&bytecode, &[]).unwrap();
//! assert_eq!(result, 42);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

// CRITICAL: Whitebox feature is required for encrypted bytecode execution
// The proc-macro encrypts bytecode using WBC-derived keys at compile time.
// Without this feature, the runtime cannot derive the same keys to decrypt.
#[cfg(not(any(feature = "whitebox", feature = "whitebox_lite")))]
compile_error!(
    "Aegis VM: 'whitebox' or 'whitebox_lite' feature is required for encrypted bytecode execution! \
     Add `features = [\"whitebox\"]` to your aegis_vm dependency in Cargo.toml."
);

#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export the proc-macros for easier usage
pub use aegis_vm_macro::vm_protect;
pub use aegis_vm_macro::obfuscate_strings;
pub use aegis_vm_macro::aegis_str;

pub mod error;
pub mod opcodes;
pub mod state;
pub mod handlers;
pub mod engine;
pub mod bytecode;
pub mod crypto;
pub mod native;
pub mod integrity;
pub mod smc;
pub mod string_obfuscation;

// White-box cryptography module (required for encrypted bytecode)
// The proc-macro uses WBC for key derivation, runtime must match.
#[cfg(any(feature = "whitebox", feature = "whitebox_lite"))]
pub mod whitebox;

// Re-exports
pub use error::{VmError, VmResult};
pub use state::VmState;
pub use engine::{execute, execute_with_state, execute_with_natives, execute_with_native_table, run, run_with_natives, run_with_native_table};
pub use bytecode::{BytecodeHeader, BytecodePackage, ProtectionLevel, BuildInfo};
pub use crypto::CryptoContext;
pub use native::{NativeRegistry, NativeRegistryBuilder, NativeFunction, standard_ids};
pub use integrity::{IntegrityTable, IntegrityError, compute_hash, verify_hash};
pub use smc::{SmcConfig, execute_smc, execute_smc_with_natives, encrypt_bytecode, decrypt_bytecode};

/// Build-time generated configuration
pub mod build_config {
    include!(concat!(env!("OUT_DIR"), "/build_config.rs"));
}

/// VM version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// FNV-1a hash for bytecode integrity (randomized constants per build)
///
/// Used for HASH_CHECK opcode and general integrity verification
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash = build_config::FNV_BASIS_64;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(build_config::FNV_PRIME_64);
    }
    hash
}

/// FNV-1a 32-bit hash (for smaller checksums, randomized constants per build)
pub fn fnv1a_hash32(data: &[u8]) -> u32 {
    let mut hash = build_config::FNV_BASIS_32;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(build_config::FNV_PRIME_32);
    }
    hash
}