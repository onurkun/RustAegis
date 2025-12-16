//! Test for spin crate dependency (GitHub Issue #1)
//!
//! This example tests that the vm_protect macro works correctly
//! with both std (OnceLock) and no_std (spin::Once) builds.
//!
//! Run with std (default):
//!   cargo run --example 09_spin_test
//!
//! Run without std:
//!   cargo run --example 09_spin_test --no-default-features

use aegis_vm::vm_protect;

// Simple protected function
#[vm_protect]
fn add_numbers(a: u64, b: u64) -> u64 {
    a + b
}

// Another protected function to test multiple usages
#[vm_protect]
fn multiply(x: u64, y: u64) -> u64 {
    x * y
}

fn main() {
    println!("Testing vm_protect with spin/OnceLock caching...\n");

    // Test add_numbers
    let result1 = add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", result1);
    assert_eq!(result1, 30);

    // Call again to test caching (should use cached bytecode)
    let result2 = add_numbers(5, 7);
    println!("add_numbers(5, 7) = {} (cached)", result2);
    assert_eq!(result2, 12);

    // Test multiply
    let result3 = multiply(6, 7);
    println!("multiply(6, 7) = {}", result3);
    assert_eq!(result3, 42);

    // Call again to test caching
    let result4 = multiply(8, 8);
    println!("multiply(8, 8) = {} (cached)", result4);
    assert_eq!(result4, 64);

    println!("\nAll tests passed!");
    println!("Using spin::Once for bytecode caching (works for both std and no_std)");
}
