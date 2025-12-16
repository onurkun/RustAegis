//! Native Call Example - Issue #1 Solution
//!
//! Demonstrates calling external functions from vm_protect code.
//! Note: Rust macros (println!, log::error!) cannot be used inside VM.
//! Use native function wrappers instead.

use aegis_vm::vm_protect;

// Native wrappers for logging (can't use macros inside VM)
fn log_vm_detected() {
    println!("  [!] VM detected! Exiting...");
}

fn log_invalid_license() {
    println!("  [!] Invalid license! Exiting...");
}

fn log_check(msg: u64) {
    // 1 = inside_vm_check, 2 = is_license_valid
    match msg {
        1 => println!("  [check] inside_vm()"),
        2 => println!("  [check] is_license_valid()"),
        _ => {}
    }
}

#[vm_protect(level="paranoid")]
fn is_license_valid() -> bool {
    log_check(2);
    // Placeholder for license validation logic
    true
}

#[vm_protect(level="paranoid")]
fn inside_vm_check() -> bool {
    log_check(1);
    // Placeholder for VM detection
    false
}

#[vm_protect(level="paranoid")]
fn exit_conditions() -> bool {
    // If we are inside a vm...
    // IMPORTANT: Use explicit bool type annotation for native call results
    let vm_check: bool = inside_vm_check();
    if vm_check {
        log_vm_detected();
        return true;
    }

    let license: bool = is_license_valid();
    if !license {
        log_invalid_license();
        return true;
    }

    false
}

fn main() {
    println!("=== Native Call Example ===\n");

    let should_exit = exit_conditions();

    // exit_conditions returns true if we should exit (bad)
    // returns false if all checks passed (good)
    println!("\nResult: {}", if should_exit { "DENIED" } else { "AUTHORIZED" });
}