//! Native Function Calls - Workaround for Issue #1
//!
//! https://github.com/onurkun/RustAegis/issues/1
//!
//! Problem: Function calls inside #[vm_protect] are not supported yet.
//! Solution: Use NativeRegistry to call external functions from VM.

use aegis_vm::engine::execute_with_natives;
use aegis_vm::native::NativeRegistry;
use aegis_vm::build_config::opcodes::{stack, native, exec};

// Your external functions
fn inside_vm_check() -> bool {
    println!("  inside_vm() -> false");
    false
}

fn is_license_valid() -> bool {
    println!("  is_license_valid() -> true");
    true
}

fn log_error(code: u64) {
    println!("  log_error({})", code);
}

fn main() {
    println!("=== Issue #1 Workaround ===\n");

    // 1. Register your external functions
    let mut registry = NativeRegistry::new();

    registry.register(0, |_| if inside_vm_check() { 1 } else { 0 }).unwrap();
    registry.register(1, |_| if is_license_valid() { 1 } else { 0 }).unwrap();
    registry.register(2, |args| { log_error(args.get(0).copied().unwrap_or(0)); 0 }).unwrap();

    // 2. VM bytecode that calls these functions
    // Equivalent to:
    //   inside_vm();
    //   if !is_license_valid() { log_error(1); return 0; }
    //   return 1;
    let code = vec![
        native::NATIVE_CALL, 0, 0,  // inside_vm()
        stack::DROP,
        native::NATIVE_CALL, 1, 0,  // license = is_license_valid()
        exec::HALT,
    ];

    // 3. Execute
    let result = execute_with_natives(&code, &[], &registry).unwrap();
    println!("\nResult: {}", if result == 1 { "AUTHORIZED" } else { "DENIED" });
}
