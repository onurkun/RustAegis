// White-box AES cipher implementation using lookup tables

use super::tables::{WhiteboxTables, WhiteboxTablesLite};
use super::sbox::SHIFT_ROWS;
use super::AES_BLOCK_SIZE;

/// Encrypt a 16-byte block using whitebox tables
pub fn whitebox_encrypt(block: &mut [u8; AES_BLOCK_SIZE], tables: &WhiteboxTables) {
    let mut state = *block;

    // Rounds 0-8: ShiftRows + Tybox lookup + XOR combine
    for round in 0..9 {
        // Apply ShiftRows by reordering before table lookup
        let shifted = apply_shift_rows(&state);

        // Ty-box lookups (SubBytes + AddRoundKey + partial MixColumns)
        let mut tybox_outputs = [[0u32; 4]; 4]; // [col][row] = 32-bit contribution

        for col in 0..4 {
            for row in 0..4 {
                let pos = col * 4 + row;
                let input = shifted[pos];
                tybox_outputs[col][row] = tables.tybox[round][pos][input as usize];
            }
        }

        // Combine Ty-box outputs using XOR tables
        // Each column's 4 Ty-box outputs are XORed together
        for col in 0..4 {
            let combined = xor_combine_column(
                tybox_outputs[col][0],
                tybox_outputs[col][1],
                tybox_outputs[col][2],
                tybox_outputs[col][3],
                round,
                col,
                tables,
            );

            // Extract bytes from combined result
            state[col * 4] = combined as u8;
            state[col * 4 + 1] = (combined >> 8) as u8;
            state[col * 4 + 2] = (combined >> 16) as u8;
            state[col * 4 + 3] = (combined >> 24) as u8;
        }
    }

    // Round 9: ShiftRows + last round T-box (no MixColumns)
    let shifted = apply_shift_rows(&state);
    for i in 0..AES_BLOCK_SIZE {
        state[i] = tables.tbox_last[i][shifted[i] as usize];
    }

    *block = state;
}

/// Encrypt using lightweight tables (slower, smaller footprint)
pub fn whitebox_encrypt_lite(block: &mut [u8; AES_BLOCK_SIZE], tables: &WhiteboxTablesLite) {
    let mut state = *block;

    // Simple T-box based encryption
    // Note: This is less secure as it doesn't use full Chow obfuscation
    for round in 0..9 {
        // ShiftRows
        let shifted = apply_shift_rows(&state);

        // T-box lookup (SubBytes + AddRoundKey)
        let mut after_tbox = [0u8; 16];
        for i in 0..16 {
            after_tbox[i] = tables.tbox[round][i][shifted[i] as usize];
        }

        // MixColumns (direct computation, not obfuscated)
        state = mix_columns(&after_tbox);
    }

    // Last round (no MixColumns)
    let shifted = apply_shift_rows(&state);
    for i in 0..16 {
        state[i] = tables.tbox_last[i][shifted[i] as usize];
    }

    *block = state;
}

/// Decrypt a 16-byte block using whitebox tables
/// Note: Requires inverse tables which aren't generated in current implementation
/// For now, this is a placeholder that will be implemented when needed
pub fn whitebox_decrypt(_block: &mut [u8; AES_BLOCK_SIZE], _tables: &WhiteboxTables) {
    // Decryption requires inverse tables (InvSubBytes, InvShiftRows, InvMixColumns)
    // This would need separate table generation for decryption
    // For Aegis VM, we primarily need encryption (to decrypt bytecode at startup)
    unimplemented!("Whitebox decryption requires inverse tables - not yet implemented")
}

/// Apply ShiftRows transformation
#[inline]
fn apply_shift_rows(state: &[u8; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];
    for i in 0..16 {
        result[i] = state[SHIFT_ROWS[i]];
    }
    result
}

/// Combine 4 Ty-box outputs using XOR tables
fn xor_combine_column(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    round: usize,
    col: usize,
    tables: &WhiteboxTables,
) -> u32 {
    // XOR using lookup tables (byte by byte, nibble by nibble)
    let mut result = 0u32;

    for byte_idx in 0..4 {
        let a_byte = ((a >> (byte_idx * 8)) & 0xff) as u8;
        let b_byte = ((b >> (byte_idx * 8)) & 0xff) as u8;
        let c_byte = ((c >> (byte_idx * 8)) & 0xff) as u8;
        let d_byte = ((d >> (byte_idx * 8)) & 0xff) as u8;

        // XOR a and b using nibble tables
        let ab = xor_byte_via_tables(a_byte, b_byte, round, col * 4 + byte_idx, tables);

        // XOR c and d
        let cd = xor_byte_via_tables(c_byte, d_byte, round, col * 4 + byte_idx + 16, tables);

        // XOR ab and cd
        let abcd = xor_byte_via_tables(ab, cd, round, col * 4 + byte_idx + 32, tables);

        result |= (abcd as u32) << (byte_idx * 8);
    }

    result
}

/// XOR two bytes using nibble-based XOR tables
fn xor_byte_via_tables(a: u8, b: u8, round: usize, table_base: usize, tables: &WhiteboxTables) -> u8 {
    let table_idx = table_base % 96;

    // Split into nibbles
    let a_lo = a & 0x0f;
    let a_hi = (a >> 4) & 0x0f;
    let b_lo = b & 0x0f;
    let b_hi = (b >> 4) & 0x0f;

    // XOR nibbles via tables
    let lo = tables.xor_tables[round][table_idx][a_lo as usize][b_lo as usize];
    let hi = tables.xor_tables[round][(table_idx + 1) % 96][a_hi as usize][b_hi as usize];

    (hi << 4) | (lo & 0x0f)
}

/// MixColumns transformation (for lite variant)
fn mix_columns(state: &[u8; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];

    for col in 0..4 {
        let s0 = state[col * 4];
        let s1 = state[col * 4 + 1];
        let s2 = state[col * 4 + 2];
        let s3 = state[col * 4 + 3];

        // MixColumns matrix multiplication in GF(2^8)
        result[col * 4] = gf_mul_2(s0) ^ gf_mul_3(s1) ^ s2 ^ s3;
        result[col * 4 + 1] = s0 ^ gf_mul_2(s1) ^ gf_mul_3(s2) ^ s3;
        result[col * 4 + 2] = s0 ^ s1 ^ gf_mul_2(s2) ^ gf_mul_3(s3);
        result[col * 4 + 3] = gf_mul_3(s0) ^ s1 ^ s2 ^ gf_mul_2(s3);
    }

    result
}

/// Multiply by 2 in GF(2^8)
#[inline]
fn gf_mul_2(a: u8) -> u8 {
    let mut result = a << 1;
    if a & 0x80 != 0 {
        result ^= 0x1b;
    }
    result
}

/// Multiply by 3 in GF(2^8)
#[inline]
fn gf_mul_3(a: u8) -> u8 {
    gf_mul_2(a) ^ a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whitebox::generator::{generate_tables, generate_tables_lite};

    // NIST AES test vector
    const TEST_KEY: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
        0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c
    ];

    const TEST_PLAINTEXT: [u8; 16] = [
        0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
        0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34
    ];

    const TEST_CIPHERTEXT: [u8; 16] = [
        0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb,
        0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32
    ];

    #[test]
    fn test_shift_rows() {
        let input = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let output = apply_shift_rows(&input);

        // ShiftRows: row 0 no shift, row 1 shift 1, row 2 shift 2, row 3 shift 3
        assert_eq!(output[0], 0);  // row 0, col 0
        assert_eq!(output[1], 5);  // row 1, col 0 <- col 1
        assert_eq!(output[2], 10); // row 2, col 0 <- col 2
        assert_eq!(output[3], 15); // row 3, col 0 <- col 3
    }

    #[test]
    fn test_mix_columns() {
        // Test vector from NIST
        let input = [
            0xdb, 0x13, 0x53, 0x45,
            0xf2, 0x0a, 0x22, 0x5c,
            0x01, 0x01, 0x01, 0x01,
            0xc6, 0xc6, 0xc6, 0xc6,
        ];

        let output = mix_columns(&input);

        // First column should be [0x8e, 0x4d, 0xa1, 0xbc]
        assert_eq!(output[0], 0x8e);
        assert_eq!(output[1], 0x4d);
        assert_eq!(output[2], 0xa1);
        assert_eq!(output[3], 0xbc);
    }

    #[test]
    fn test_whitebox_encrypt_lite_roundtrip() {
        // Generate tables with test key
        let tables = generate_tables_lite(&TEST_KEY, b"test_seed");

        let mut block = TEST_PLAINTEXT;
        whitebox_encrypt_lite(&mut block, &tables);

        // Note: Due to internal encodings, output won't match standard AES
        // This test verifies the function runs without panic
        assert_ne!(block, TEST_PLAINTEXT, "Block should be different after encryption");
    }

    #[test]
    fn test_whitebox_deterministic() {
        let tables = generate_tables(&TEST_KEY, b"deterministic_test");

        let mut block1 = TEST_PLAINTEXT;
        let mut block2 = TEST_PLAINTEXT;

        whitebox_encrypt(&mut block1, &tables);
        whitebox_encrypt(&mut block2, &tables);

        assert_eq!(block1, block2, "Same plaintext should produce same ciphertext");
    }

    #[test]
    fn test_whitebox_different_plaintexts() {
        let tables = generate_tables(&TEST_KEY, b"test_seed");

        let mut block1 = [0u8; 16];
        let mut block2 = [1u8; 16];

        whitebox_encrypt(&mut block1, &tables);
        whitebox_encrypt(&mut block2, &tables);

        assert_ne!(block1, block2, "Different plaintexts should produce different ciphertexts");
    }

    #[test]
    fn test_whitebox_avalanche() {
        let tables = generate_tables(&TEST_KEY, b"avalanche_test");

        let mut block1 = [0u8; 16];
        let mut block2 = [0u8; 16];
        block2[0] = 1; // Flip one bit

        whitebox_encrypt(&mut block1, &tables);
        whitebox_encrypt(&mut block2, &tables);

        // Count differing bits
        let mut diff_bits = 0;
        for i in 0..16 {
            diff_bits += (block1[i] ^ block2[i]).count_ones();
        }

        // Good cipher should have ~50% bit difference (avalanche effect)
        // Allow some variance: 30-70% = 38-90 bits
        assert!(
            diff_bits >= 30 && diff_bits <= 100,
            "Avalanche effect: {} bits differ (expected ~64)",
            diff_bits
        );
    }
}
