//! Native Call Macro Test - Issue #1 Direct Solution
//!
//! This tests the automatic native call support in #[vm_protect] macro.
//! External function calls are automatically wrapped and executed.

use aegis_vm_macro::vm_protect;

// External functions that will be called from VM
fn add_numbers(a: u64, b: u64) -> u64 {
    println!("  add_numbers({}, {}) called", a, b);
    a + b
}

fn multiply(x: u64, y: u64) -> u64 {
    println!("  multiply({}, {}) called", x, y);
    x * y
}

fn get_magic_number() -> u64 {
    println!("  get_magic_number() called");
    42
}

// Test: VM-protected function that calls external functions
#[vm_protect(level="paranoid")]
fn compute_with_native_calls(a: u64, b: u64) -> u64 {
    let sum = add_numbers(a, b);
    let magic = get_magic_number();
    let result = multiply(sum, magic);
    result
}

fn main() {
    println!("=== Native Call Macro Test ===\n");

    let a = 10u64;
    let b = 5u64;

    println!("Calling compute_with_native_calls({}, {})", a, b);
    let result = compute_with_native_calls(a, b);

    println!("\nExpected: ({} + {}) * 42 = {}", a, b, (a + b) * 42);
    println!("Got:      {}", result);

    if result == (a + b) * 42 {
        println!("\nSUCCESS!");
    } else {
        println!("\nFAILED!");
    }
}
