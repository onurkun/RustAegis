//! White-box AES integration tests
//!
//! These tests verify that the whitebox crypto implementation works correctly
//! with build-time derived keys and runtime table generation.

#![cfg(feature = "whitebox")]

use aegis_vm::whitebox::{
    init_tables, init_tables_lite, whitebox_encrypt, whitebox_encrypt_lite,
    encrypt_block, encrypt_blocks, WhiteboxTables, AES_BLOCK_SIZE, WHITEBOX_TABLE_SIZE,
};

#[test]
fn test_init_tables() {
    // This should use build-time derived key
    let tables = init_tables();

    // Verify tables are populated (not all zeros)
    let has_nonzero = tables.tybox.iter()
        .flat_map(|round| round.iter())
        .flat_map(|pos| pos.iter())
        .any(|&v| v != 0);

    assert!(has_nonzero, "Tables should contain non-zero values");
}

#[test]
fn test_init_tables_lite() {
    let tables = init_tables_lite();

    // Verify T-boxes are populated
    let has_nonzero = tables.tbox.iter()
        .flat_map(|round| round.iter())
        .flat_map(|pos| pos.iter())
        .any(|&v| v != 0);

    assert!(has_nonzero, "Lite tables should contain non-zero values");
}

#[test]
fn test_table_memory_size() {
    let tables = init_tables();
    let size = tables.memory_size();

    // Should be around 500-600KB
    assert!(
        size > 500_000 && size < 700_000,
        "Full tables should be ~500-600KB, got {}",
        size
    );
}

#[test]
fn test_lite_table_memory_size() {
    let tables = init_tables_lite();
    let size = tables.memory_size();

    // Should be around 40-50KB
    assert!(
        size > 40_000 && size < 60_000,
        "Lite tables should be ~40-50KB, got {}",
        size
    );
}

#[test]
fn test_encrypt_changes_block() {
    let tables = init_tables();

    let original = [0u8; AES_BLOCK_SIZE];
    let mut encrypted = original;

    whitebox_encrypt(&mut encrypted, &tables);

    assert_ne!(
        original, encrypted,
        "Encryption should change the block"
    );
}

#[test]
fn test_encrypt_deterministic() {
    let tables = init_tables();

    let mut block1 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let mut block2 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    whitebox_encrypt(&mut block1, &tables);
    whitebox_encrypt(&mut block2, &tables);

    assert_eq!(
        block1, block2,
        "Same plaintext should produce same ciphertext"
    );
}

#[test]
fn test_encrypt_different_plaintexts() {
    let tables = init_tables();

    let mut block1 = [0u8; AES_BLOCK_SIZE];
    let mut block2 = [1u8; AES_BLOCK_SIZE];

    whitebox_encrypt(&mut block1, &tables);
    whitebox_encrypt(&mut block2, &tables);

    assert_ne!(
        block1, block2,
        "Different plaintexts should produce different ciphertexts"
    );
}

#[test]
fn test_encrypt_block_convenience() {
    let mut block = [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
                     0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
    let original = block;

    encrypt_block(&mut block);

    assert_ne!(original, block, "encrypt_block should modify the block");
}

#[test]
fn test_encrypt_blocks_batch() {
    let mut blocks = [
        [0u8; AES_BLOCK_SIZE],
        [1u8; AES_BLOCK_SIZE],
        [2u8; AES_BLOCK_SIZE],
    ];

    let originals = blocks.clone();

    encrypt_blocks(&mut blocks);

    for (i, (original, encrypted)) in originals.iter().zip(blocks.iter()).enumerate() {
        assert_ne!(
            original, encrypted,
            "Block {} should be encrypted",
            i
        );
    }
}

#[test]
fn test_avalanche_effect() {
    let tables = init_tables();

    // Two plaintexts differing by one bit
    let mut block1 = [0u8; AES_BLOCK_SIZE];
    let mut block2 = [0u8; AES_BLOCK_SIZE];
    block2[0] = 1; // Flip one bit

    whitebox_encrypt(&mut block1, &tables);
    whitebox_encrypt(&mut block2, &tables);

    // Count differing bits
    let diff_bits: u32 = block1.iter()
        .zip(block2.iter())
        .map(|(a, b)| (a ^ b).count_ones())
        .sum();

    // Good cipher: ~50% bit difference (64 bits for 128-bit block)
    // Allow 30-90 bits variance
    assert!(
        diff_bits >= 30 && diff_bits <= 100,
        "Avalanche effect: {} bits differ (expected ~64)",
        diff_bits
    );
}

#[test]
fn test_lite_encrypt_changes_block() {
    let tables = init_tables_lite();

    let original = [0u8; AES_BLOCK_SIZE];
    let mut encrypted = original;

    whitebox_encrypt_lite(&mut encrypted, &tables);

    assert_ne!(
        original, encrypted,
        "Lite encryption should change the block"
    );
}

#[test]
fn test_tables_reusable() {
    let tables = init_tables();

    // Encrypt multiple blocks with same tables
    for i in 0..10 {
        let mut block = [i as u8; AES_BLOCK_SIZE];
        whitebox_encrypt(&mut block, &tables);
        // Should not panic
    }
}

#[test]
fn test_all_zero_block() {
    let tables = init_tables();
    let mut block = [0u8; AES_BLOCK_SIZE];

    whitebox_encrypt(&mut block, &tables);

    // Even all-zeros should produce non-zero output
    let is_all_zero = block.iter().all(|&b| b == 0);
    assert!(!is_all_zero, "All-zero input should not produce all-zero output");
}

#[test]
fn test_all_ones_block() {
    let tables = init_tables();
    let mut block = [0xff; AES_BLOCK_SIZE];

    whitebox_encrypt(&mut block, &tables);

    // Should produce non-all-ones output
    let is_all_ones = block.iter().all(|&b| b == 0xff);
    assert!(!is_all_ones, "All-ones input should not produce all-ones output");
}

#[test]
fn test_whitebox_table_size_constant() {
    // Verify the constant is reasonable
    assert!(
        WHITEBOX_TABLE_SIZE > 500_000,
        "WHITEBOX_TABLE_SIZE should be > 500KB"
    );
}

#[test]
fn test_build_config_keys_exist() {
    // Verify build-time generated keys are accessible
    use aegis_vm::build_config::whitebox_config::{WBC_KEY, WBC_TABLE_SEED};

    // Keys should not be all zeros (extremely unlikely with proper key derivation)
    assert!(
        WBC_KEY.iter().any(|&b| b != 0),
        "WBC_KEY should not be all zeros"
    );
    assert!(
        WBC_TABLE_SEED.iter().any(|&b| b != 0),
        "WBC_TABLE_SEED should not be all zeros"
    );

    // Keys should be proper length
    assert_eq!(WBC_KEY.len(), 16, "WBC_KEY should be 16 bytes");
    assert_eq!(WBC_TABLE_SEED.len(), 32, "WBC_TABLE_SEED should be 32 bytes");
}

#[test]
fn test_encrypt_performance_sanity() {
    use std::time::Instant;

    let tables = init_tables();

    // Encrypt 1000 blocks
    let start = Instant::now();
    for i in 0..1000 {
        let mut block = [i as u8; AES_BLOCK_SIZE];
        whitebox_encrypt(&mut block, &tables);
    }
    let elapsed = start.elapsed();

    // Should complete in reasonable time (< 1 second for 1000 blocks)
    assert!(
        elapsed.as_secs() < 1,
        "1000 encryptions took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_table_generation_deterministic() {
    // Generate tables twice with same key/seed
    let tables1 = init_tables();
    let tables2 = init_tables();

    // Encrypt same block with both
    let mut block1 = [0x42; AES_BLOCK_SIZE];
    let mut block2 = [0x42; AES_BLOCK_SIZE];

    whitebox_encrypt(&mut block1, &tables1);
    whitebox_encrypt(&mut block2, &tables2);

    assert_eq!(
        block1, block2,
        "Tables generated from same key should produce same results"
    );
}
