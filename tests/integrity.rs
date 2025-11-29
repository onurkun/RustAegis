//! Tests for Region-based Integrity Checking
//!
//! Verifies that the integrity checking system works correctly
//! at both compile-time (hash computation) and runtime (verification).

use aegis_vm_macro::vm_protect;

// =============================================================================
// Functions with integrity protection
// =============================================================================

/// Standard level - has full hash integrity check
#[vm_protect]
fn standard_with_integrity(x: u64) -> u64 {
    x + 0xDEADBEEF
}

/// Paranoid level - has both full hash and region-based integrity checks
#[vm_protect(level = "paranoid")]
fn paranoid_with_integrity(x: u64) -> u64 {
    x + 0xDEADBEEF
}

/// Debug level - no integrity checks (plaintext)
#[vm_protect(level = "debug")]
fn debug_no_integrity(x: u64) -> u64 {
    x + 0xDEADBEEF
}

/// Larger function to test multiple regions
#[vm_protect(level = "paranoid")]
fn multi_region_function(a: u64, b: u64, c: u64) -> u64 {
    let x = a + b;
    let y = x * c;
    let z = y ^ 0x1234567890ABCDEF;
    let w = z - a;
    let v = w | b;
    let u = v & c;
    let t = u << 4;
    let s = t >> 2;
    if s > 100 {
        s + 1
    } else {
        s - 1
    }
}

// =============================================================================
// Basic integrity tests
// =============================================================================

#[test]
fn test_standard_integrity_works() {
    // Standard level should work with integrity check
    let result = standard_with_integrity(100);
    assert_eq!(result, 100 + 0xDEADBEEF);
}

#[test]
fn test_paranoid_integrity_works() {
    // Paranoid level should work with region-based integrity check
    let result = paranoid_with_integrity(100);
    assert_eq!(result, 100 + 0xDEADBEEF);
}

#[test]
fn test_debug_no_integrity_works() {
    // Debug level should work without integrity check
    let result = debug_no_integrity(100);
    assert_eq!(result, 100 + 0xDEADBEEF);
}

#[test]
fn test_multi_region_function() {
    // Test function that spans multiple regions
    let result = multi_region_function(10, 20, 3);

    // Compute expected: complex expression
    let x = 10u64 + 20;
    let y = x * 3;
    let z = y ^ 0x1234567890ABCDEF;
    let w = z - 10;
    let v = w | 20;
    let u = v & 3;
    let t = u << 4;
    let s = t >> 2;
    let expected = if s > 100 { s + 1 } else { s - 1 };

    assert_eq!(result, expected);
}

// =============================================================================
// Consistency tests across protection levels
// =============================================================================

#[test]
fn test_all_levels_produce_same_result() {
    for x in [0u64, 1, 100, 0xFFFF, 0xFFFFFFFF] {
        let debug_result = debug_no_integrity(x);
        let standard_result = standard_with_integrity(x);
        let paranoid_result = paranoid_with_integrity(x);

        assert_eq!(debug_result, standard_result,
            "debug vs standard mismatch for x={}", x);
        assert_eq!(standard_result, paranoid_result,
            "standard vs paranoid mismatch for x={}", x);
    }
}

// =============================================================================
// IntegrityTable unit tests
// =============================================================================

#[test]
fn test_integrity_table_api() {
    use aegis_vm::integrity::{IntegrityTable, DEFAULT_REGION_SIZE};

    // Create test bytecode
    let bytecode = vec![0x42u8; 256];

    // Create integrity table
    let table = IntegrityTable::new(&bytecode, DEFAULT_REGION_SIZE);

    // Verify valid bytecode passes
    assert!(table.verify(&bytecode).is_ok());
    assert!(table.verify_quick(&bytecode));

    // All regions should be valid
    for i in 0..table.regions.len() {
        assert!(table.verify_region(&bytecode, i));
    }
}

#[test]
fn test_integrity_detects_tampering() {
    use aegis_vm::integrity::IntegrityTable;

    let bytecode = vec![0x42u8; 128];
    let table = IntegrityTable::new(&bytecode, 32);

    // Tamper with bytecode
    let mut tampered = bytecode.clone();
    tampered[50] = 0xFF; // Modify middle byte

    // Should detect tampering
    assert!(table.verify(&tampered).is_err());
    assert!(!table.verify_quick(&tampered));

    // Should identify which region was tampered
    assert!(table.verify_region(&tampered, 0)); // OK
    assert!(!table.verify_region(&tampered, 1)); // Tampered (bytes 32-64)
    assert!(table.verify_region(&tampered, 2)); // OK
    assert!(table.verify_region(&tampered, 3)); // OK
}

#[test]
fn test_compute_hash_consistency() {
    use aegis_vm::compute_hash;

    let data = b"test bytecode data";

    // Same data should produce same hash
    let hash1 = compute_hash(data);
    let hash2 = compute_hash(data);
    assert_eq!(hash1, hash2);

    // Different data should produce different hash
    let hash3 = compute_hash(b"different data");
    assert_ne!(hash1, hash3);
}

#[test]
fn test_verify_hash() {
    use aegis_vm::{compute_hash, verify_hash};

    let data = b"some bytecode";
    let expected_hash = compute_hash(data);

    assert!(verify_hash(data, expected_hash));
    assert!(!verify_hash(data, expected_hash + 1));
    assert!(!verify_hash(b"wrong data", expected_hash));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_bytecode_integrity() {
    use aegis_vm::integrity::IntegrityTable;

    let bytecode: Vec<u8> = vec![];
    let table = IntegrityTable::new(&bytecode, 64);

    assert!(table.verify(&bytecode).is_ok());
}

#[test]
fn test_small_bytecode_integrity() {
    use aegis_vm::integrity::IntegrityTable;

    // Bytecode smaller than region size
    let bytecode = vec![0x42u8; 10];
    let table = IntegrityTable::new(&bytecode, 64);

    assert_eq!(table.regions.len(), 1);
    assert!(table.verify(&bytecode).is_ok());
}

#[test]
fn test_exact_region_size_bytecode() {
    use aegis_vm::integrity::IntegrityTable;

    // Bytecode exactly one region
    let bytecode = vec![0x42u8; 64];
    let table = IntegrityTable::new(&bytecode, 64);

    assert_eq!(table.regions.len(), 1);
    assert!(table.verify(&bytecode).is_ok());
}

#[test]
fn test_multiple_regions_bytecode() {
    use aegis_vm::integrity::IntegrityTable;

    // Bytecode spanning 4 regions
    let bytecode = vec![0x42u8; 256];
    let table = IntegrityTable::new(&bytecode, 64);

    assert_eq!(table.regions.len(), 4);
    assert!(table.verify(&bytecode).is_ok());

    // Verify region boundaries
    assert_eq!(table.regions[0].start, 0);
    assert_eq!(table.regions[0].end, 64);
    assert_eq!(table.regions[1].start, 64);
    assert_eq!(table.regions[1].end, 128);
    assert_eq!(table.regions[2].start, 128);
    assert_eq!(table.regions[2].end, 192);
    assert_eq!(table.regions[3].start, 192);
    assert_eq!(table.regions[3].end, 256);
}
