//! WASM Usage Example for aegis_vm
//!
//! This example demonstrates how to use aegis_vm with WebAssembly.
//!
//! ## Setup
//!
//! 1. Add wasm target:
//!    ```bash
//!    rustup target add wasm32-unknown-unknown
//!    ```
//!
//! 2. Install wasm-pack (optional, for testing):
//!    ```bash
//!    cargo install wasm-pack
//!    ```
//!
//! ## Project Structure
//!
//! Create a new crate for your WASM module:
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! name = "my_wasm_app"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [lib]
//! crate-type = ["cdylib", "rlib"]
//!
//! [dependencies]
//! aegis_vm = { version = "0.1", default-features = false }
//! wasm-bindgen = "0.2"
//!
//! [dev-dependencies]
//! wasm-bindgen-test = "0.3"
//! ```
//!
//! ## Usage Pattern
//!
//! Since `#[vm_protect]` and `#[wasm_bindgen]` cannot be combined directly,
//! use a wrapper pattern:
//!
//! ```ignore
//! use aegis_vm::vm_protect;
//! use wasm_bindgen::prelude::*;
//!
//! // VM-protected implementation (internal)
//! #[vm_protect(level = "debug")]  // or "standard" for encryption
//! fn secret_algorithm_impl(input: u64) -> u64 {
//!     // Your sensitive logic here
//!     input ^ 0xDEADBEEF
//! }
//!
//! // WASM-exported wrapper (public)
//! #[wasm_bindgen]
//! pub fn secret_algorithm(input: u64) -> u64 {
//!     secret_algorithm_impl(input)
//! }
//! ```
//!
//! ## Protection Levels
//!
//! - `debug`: Plaintext bytecode, for development
//! - `standard`: Encrypted bytecode (default)
//! - `paranoid`: Maximum protection with additional checks
//!
//! ## Building
//!
//! ```bash
//! # Build for WASM
//! cargo build --target wasm32-unknown-unknown --release
//!
//! # Or use wasm-pack for npm package
//! wasm-pack build --target web
//! ```
//!
//! ## Testing
//!
//! ```bash
//! wasm-pack test --node
//! # or
//! wasm-pack test --headless --firefox
//! ```
//!
//! ## JavaScript Usage
//!
//! ```javascript
//! import init, { secret_algorithm } from './my_wasm_app.js';
//!
//! async function main() {
//!     await init();
//!     const result = secret_algorithm(BigInt(42));
//!     console.log('Result:', result);
//! }
//!
//! main();
//! ```
//!
//! ## Full Example
//!
//! See `examples/wasm_test/` for a complete working example with tests.

use aegis_vm::vm_protect;

// Example: License validation (VM-protected)
#[vm_protect(level = "debug")]
fn validate_license_impl(key: u64, product_id: u64) -> bool {
    // Simple validation: key must be product_id XOR'd with magic constant
    let expected = product_id ^ 0xCAFEBABE_DEADBEEF;
    key == expected
}

// Example: Score calculation (VM-protected)
#[vm_protect(level = "debug")]
fn calculate_score_impl(base: u64, multiplier: u64, bonus: u64) -> u64 {
    (base * multiplier) + bonus
}

// Example: Anti-cheat value check (VM-protected)
#[vm_protect(level = "debug")]
fn verify_game_state_impl(health: u64, max_health: u64, score: u64) -> bool {
    // Health cannot exceed max, score must be reasonable
    health <= max_health && score < 1_000_000_000
}

fn main() {
    // Demonstrate the functions work natively
    println!("=== aegis_vm WASM Example ===\n");

    // License validation
    let product_id = 12345u64;
    let valid_key = product_id ^ 0xCAFEBABE_DEADBEEF;
    let invalid_key = 0u64;

    println!("License Validation:");
    println!("  Product ID: {}", product_id);
    println!("  Valid key: {} -> {}", valid_key, validate_license_impl(valid_key, product_id));
    println!("  Invalid key: {} -> {}", invalid_key, validate_license_impl(invalid_key, product_id));

    // Score calculation
    println!("\nScore Calculation:");
    let score = calculate_score_impl(100, 5, 50);
    println!("  (100 * 5) + 50 = {}", score);

    // Game state verification
    println!("\nGame State Verification:");
    println!("  Health 80/100, Score 5000: {}", verify_game_state_impl(80, 100, 5000));
    println!("  Health 150/100, Score 5000: {}", verify_game_state_impl(150, 100, 5000));
    println!("  Health 80/100, Score 999999999999: {}", verify_game_state_impl(80, 100, 999_999_999_999));

    println!("\n=== For WASM usage, see examples/wasm_test/ ===");
}
