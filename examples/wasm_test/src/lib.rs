//! WASM test for aegis_vm
//!
//! This module tests that the VM works correctly in a WASM environment
//! using the vm_protect macro.

use wasm_bindgen::prelude::*;
use aegis_vm::vm_protect;

/// Simple addition test - returns a + b using VM
#[vm_protect(level = "debug")]
pub fn vm_add_impl(a: u64, b: u64) -> u64 {
    a + b
}

#[wasm_bindgen]
pub fn vm_add(a: u64, b: u64) -> u64 {
    vm_add_impl(a, b)
}

/// Test multiplication
#[vm_protect(level = "debug")]
pub fn vm_multiply_impl(a: u64, b: u64) -> u64 {
    a * b
}

#[wasm_bindgen]
pub fn vm_multiply(a: u64, b: u64) -> u64 {
    vm_multiply_impl(a, b)
}

/// Test XOR operation
#[vm_protect(level = "debug")]
pub fn vm_xor_impl(a: u64, b: u64) -> u64 {
    a ^ b
}

#[wasm_bindgen]
pub fn vm_xor(a: u64, b: u64) -> u64 {
    vm_xor_impl(a, b)
}

/// Complex test: (a + b) * c
#[vm_protect(level = "debug")]
pub fn vm_complex_impl(a: u64, b: u64, c: u64) -> u64 {
    (a + b) * c
}

#[wasm_bindgen]
pub fn vm_complex(a: u64, b: u64, c: u64) -> u64 {
    vm_complex_impl(a, b, c)
}

/// Test subtraction
#[vm_protect(level = "debug")]
pub fn vm_sub_impl(a: u64, b: u64) -> u64 {
    a - b
}

#[wasm_bindgen]
pub fn vm_sub(a: u64, b: u64) -> u64 {
    vm_sub_impl(a, b)
}

/// Test boolean return
#[vm_protect(level = "debug")]
pub fn vm_is_equal_impl(a: u64, b: u64) -> bool {
    a == b
}

#[wasm_bindgen]
pub fn vm_is_equal(a: u64, b: u64) -> bool {
    vm_is_equal_impl(a, b)
}

// ============================================================================
// WASM Tests - These run in the WASM environment
// ============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_vm_add() {
        assert_eq!(vm_add(10, 32), 42);
        assert_eq!(vm_add(0, 0), 0);
        assert_eq!(vm_add(100, 200), 300);
    }

    #[wasm_bindgen_test]
    fn test_vm_multiply() {
        assert_eq!(vm_multiply(6, 7), 42);
        assert_eq!(vm_multiply(0, 100), 0);
        assert_eq!(vm_multiply(1000, 1000), 1_000_000);
    }

    #[wasm_bindgen_test]
    fn test_vm_xor() {
        assert_eq!(vm_xor(0xFF, 0x0F), 0xF0);
        assert_eq!(vm_xor(42, 42), 0); // x ^ x = 0
        assert_eq!(vm_xor(0, 123), 123); // 0 ^ x = x
    }

    #[wasm_bindgen_test]
    fn test_vm_complex() {
        // (a + b) * c
        assert_eq!(vm_complex(2, 3, 4), 20); // (2+3)*4 = 20
        assert_eq!(vm_complex(10, 10, 10), 200); // (10+10)*10 = 200
        assert_eq!(vm_complex(0, 0, 100), 0); // (0+0)*100 = 0
    }

    #[wasm_bindgen_test]
    fn test_vm_sub() {
        assert_eq!(vm_sub(100, 58), 42);
        assert_eq!(vm_sub(50, 50), 0);
    }

    #[wasm_bindgen_test]
    fn test_vm_is_equal() {
        assert!(vm_is_equal(42, 42));
        assert!(!vm_is_equal(10, 20));
    }

    #[wasm_bindgen_test]
    fn test_vm_large_numbers() {
        let large = 0xDEADBEEF_CAFEBABE_u64;
        assert_eq!(vm_add(large, 0), large);
        assert_eq!(vm_xor(large, large), 0);
    }

    #[wasm_bindgen_test]
    fn test_vm_wrapping() {
        // Test overflow wrapping
        assert_eq!(vm_add(u64::MAX, 1), 0);
    }
}
