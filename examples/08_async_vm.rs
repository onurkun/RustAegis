//! Async VM Example (Experimental)
//!
//! This example demonstrates the async VM engine which transforms
//! the execution loop into an async/await state machine for
//! additional obfuscation against reverse engineering.
//!
//! Run with:
//!   cargo run --example 08_async_vm --features async_vm

#[cfg(feature = "async_vm")]
mod tests {
    use aegis_vm::vm_protect;

    // Native functions (called from VM)
    fn get_multiplier() -> u64 {
        7
    }

    fn compute_bonus(base: u64) -> u64 {
        base * 2 + 10
    }

    fn validate_range(value: u64, min: u64, max: u64) -> bool {
        value >= min && value <= max
    }

    fn log_result(value: u64) {
        println!("   [Native] Logged value: {}", value);
    }

    // VM-protected function with native calls
    #[vm_protect]
    pub fn calculate_score(input: u64) -> u64 {
        let multiplier: u64 = get_multiplier();
        let base = input * multiplier;
        let bonus: u64 = compute_bonus(base);

        log_result(bonus);
        bonus
    }

    // VM-protected function with boolean native call
    #[vm_protect]
    pub fn check_valid(value: u64) -> bool {
        let in_range: bool = validate_range(value, 10, 100);

        if !in_range {
            return false;
        }

        value % 2 == 0
    }

    // VM-protected with loop and native call
    #[vm_protect]
    pub fn sum_with_bonus(n: u64) -> u64 {
        let mut total: u64 = 0;

        for i in 1..=n {
            total = total + i;
        }

        let bonus: u64 = compute_bonus(total);
        bonus
    }

    // Complex arithmetic
    #[vm_protect(level = "paranoid")]
    pub fn paranoid_hash(seed: u64) -> u64 {
        let a = seed ^ 0xDEADBEEF;
        let b = a + 0x12345678;
        let c = b * 3;
        let d = c ^ seed;
        d
    }

    pub fn run_tests() {
        println!("=== Async VM with Native Calls ===\n");

        // Test 1: Native function calls
        println!("1. Native Function Calls");
        println!("   Input: 5");
        let result = calculate_score(5);
        println!("   Score: {}", result);
        // 5 * 7 = 35, bonus = 35 * 2 + 10 = 80
        assert_eq!(result, 80);
        println!("   PASS\n");

        // Test 2: Boolean native calls
        println!("2. Boolean Validation");
        let valid1 = check_valid(50);  // in range, even -> true
        let valid2 = check_valid(51);  // in range, odd -> false
        let valid3 = check_valid(5);   // out of range -> false
        println!("   check_valid(50) = {} (expected true)", valid1);
        println!("   check_valid(51) = {} (expected false)", valid2);
        println!("   check_valid(5)  = {} (expected false)", valid3);
        assert!(valid1);
        assert!(!valid2);
        assert!(!valid3);
        println!("   PASS\n");

        // Test 3: Loop with native call
        println!("3. Loop with Native Bonus");
        let result = sum_with_bonus(10);
        // sum(1..10) = 55, bonus = 55 * 2 + 10 = 120
        println!("   sum_with_bonus(10) = {}", result);
        assert_eq!(result, 120);
        println!("   PASS\n");

        // Test 4: Paranoid level
        println!("4. Paranoid Level (Heavy MBA)");
        let hash1 = paranoid_hash(12345);
        let hash2 = paranoid_hash(12345);
        let hash3 = paranoid_hash(54321);
        println!("   paranoid_hash(12345) = 0x{:X}", hash1);
        println!("   paranoid_hash(12345) = 0x{:X} (deterministic)", hash2);
        println!("   paranoid_hash(54321) = 0x{:X} (different)", hash3);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        println!("   PASS\n");

        println!("=== All tests passed! ===");
        println!("\nThe async VM executes identical logic but with");
        println!("state machine obfuscation for anti-analysis.");
    }
}

#[cfg(feature = "async_vm")]
fn main() {
    tests::run_tests();
}

#[cfg(not(feature = "async_vm"))]
fn main() {
    eprintln!("This example requires the 'async_vm' feature.");
    eprintln!("Run with: cargo run --example 08_async_vm --features async_vm");
    std::process::exit(1);
}
