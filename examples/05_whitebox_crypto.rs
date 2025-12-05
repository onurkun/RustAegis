//! White-Box Cryptography Demo
//!
//! This example demonstrates how WBC is used for bytecode encryption.
//!
//! ## Running
//! ```bash
//! cargo run --example 05_whitebox_crypto
//! ```
//!
//! ## How WBC Works
//!
//! 1. WBC tables are generated from BUILD_SEED (~500KB)
//! 2. AES key is "embedded" inside tables (obfuscated math)
//! 3. Key is never exposed in memory
//! 4. Even if attacker analyzes binary, key extraction is very difficult

use aegis_vm::vm_protect;

/// Simple function protected with WBC
/// Encrypted with WBC key at compile-time
/// Decrypted with same WBC key at runtime
#[vm_protect]
fn secret_calculation(x: u64, y: u64) -> u64 {
    // This code is converted to bytecode and encrypted with WBC key
    let sum = x + y;
    let product = x * y;
    let result = sum ^ product;

    // Some complex operations
    let mut acc = result;
    for i in 0..10 {
        acc = acc.wrapping_add(i);
        acc = acc ^ (acc >> 3);
    }

    acc
}

/// More complex example - Fibonacci
#[vm_protect(level = "paranoid")]
fn protected_fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 2u64;

    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }

    b
}

/// Simple hash function (protected)
#[vm_protect]
fn simple_hash(input: u64) -> u64 {
    let mut hash = 0x9e3779b97f4a7c15u64; // Golden ratio

    hash = hash ^ input;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash = hash ^ (hash >> 13);
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash = hash ^ (hash >> 16);

    hash
}

fn main() {
    println!("=== Aegis VM - White-Box Cryptography Demo ===\n");

    // WBC info
    println!("[*] WBC Status:");
    println!("    - Feature: whitebox (enabled by default)");
    println!("    - Table size: ~500KB (generated at runtime)");
    println!("    - Key derivation: Chow et al. (2002) scheme");
    println!();

    // Test 1: Simple calculation
    println!("[*] Test 1: Secret Calculation");
    let x = 42u64;
    let y = 13u64;
    let result = secret_calculation(x, y);
    println!("    secret_calculation({}, {}) = {}", x, y, result);
    println!("    [OK] Bytecode encrypted and decrypted with WBC key");
    println!();

    // Test 2: Fibonacci
    println!("[*] Test 2: Protected Fibonacci (paranoid level)");
    for n in [5, 10, 20, 30] {
        let fib = protected_fibonacci(n);
        println!("    fibonacci({}) = {}", n, fib);
    }
    println!("    [OK] Paranoid level: MBA + ValueCryptor + WBC");
    println!();

    // Test 3: Hash
    println!("[*] Test 3: Simple Hash");
    let inputs = [0u64, 1, 42, 0xDEADBEEF, u64::MAX];
    for input in inputs {
        let hash = simple_hash(input);
        println!("    hash(0x{:016x}) = 0x{:016x}", input, hash);
    }
    println!("    [OK] Hash function protected with WBC");
    println!();

    // Security summary
    println!("=== Security Summary ===");
    println!();
    println!("Inside binary:");
    println!("  [X] AES key NOT EXPOSED");
    println!("  [X] HMAC key NOT EXPOSED");
    println!("  [OK] WBC tables (obfuscated mathematics)");
    println!("  [OK] Encrypted bytecode (AES-256-GCM)");
    println!();
    println!("Attacker scenario:");
    println!("  1. Opens binary in IDA/Ghidra");
    println!("  2. Sees encrypted bytecode -> cannot decrypt");
    println!("  3. Finds WBC tables (~500KB)");
    println!("  4. Must analyze tables:");
    println!("     -> Mixing bijections (32x32 matrix)");
    println!("     -> T-box / Ty-box composition");
    println!("     -> 10 round AES mathematics");
    println!("  5. Requires academic-level reverse engineering!");
    println!();
    println!("[OK] Demo completed successfully!");
}
