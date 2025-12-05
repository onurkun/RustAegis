// White-box AES table generator (Chow et al. scheme)
// Generates lookup tables with embedded key

use super::sbox::{SBOX, key_expansion, gf_mul, MIX_COLS, SHIFT_ROWS};
use super::tables::{
    WhiteboxTables, WhiteboxTablesLite, Bijection8, Bijection4, MixingBijection32
};
use super::{AES_BLOCK_SIZE, AES_ROUNDS};

/// Seeded random number generator for deterministic table generation
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: &[u8]) -> Self {
        // Combine seed bytes into initial state
        let mut state = 0x853c49e6748fea9bu64;
        for (i, &byte) in seed.iter().enumerate() {
            state ^= (byte as u64) << ((i % 8) * 8);
            state = state.wrapping_mul(0x5851f42d4c957f2d);
            state ^= state >> 33;
        }
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545f4914f6cdd1d)
    }

    pub fn next_u8(&mut self) -> u8 {
        self.next_u64() as u8
    }

    /// Generate random permutation (Fisher-Yates shuffle)
    pub fn random_permutation(&mut self, n: usize) -> alloc::vec::Vec<u8> {
        // Build permutation manually to avoid u8 overflow for n=256
        let mut perm: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(n);
        for i in 0..n {
            perm.push(i as u8);
        }
        for i in (1..n).rev() {
            let j = (self.next_u64() as usize) % (i + 1);
            perm.swap(i, j);
        }
        perm
    }
}

extern crate alloc;

/// Generate complete whitebox tables for AES-128
pub fn generate_tables(key: &[u8; 16], seed: &[u8]) -> WhiteboxTables {
    let mut tables = WhiteboxTables::new();
    let mut rng = SeededRng::new(seed);

    // Expand key to get round keys
    let round_keys = key_expansion(key);

    // Generate internal encodings (bijections between rounds)
    let encodings = generate_encodings(&mut rng);

    // Generate mixing bijections
    let mixing_bijections = generate_mixing_bijections(&mut rng);

    // Generate T-boxes and Ty-boxes
    generate_tboxes_tyboxes(
        &round_keys,
        &encodings,
        &mixing_bijections,
        &mut tables,
    );

    // Generate XOR tables with encodings
    generate_xor_tables(&encodings, &mut tables);

    // Generate MBL tables (inverse MB × L encoding)
    generate_mbl_tables(&mixing_bijections, &encodings, &mut tables);

    // Generate last round T-boxes (no MixColumns)
    generate_last_round_tboxes(&round_keys, &encodings, &mut tables);

    tables
}

/// Generate lightweight whitebox tables (T-boxes only)
pub fn generate_tables_lite(key: &[u8; 16], seed: &[u8]) -> WhiteboxTablesLite {
    let mut tables = WhiteboxTablesLite::new();
    let mut rng = SeededRng::new(seed);

    let round_keys = key_expansion(key);
    let encodings = generate_encodings(&mut rng);

    // Generate simple T-boxes with encodings
    for round in 0..AES_ROUNDS {
        for pos in 0..AES_BLOCK_SIZE {
            for x in 0..256 {
                // Input: decode from previous round's output encoding
                let decoded = if round == 0 {
                    x as u8
                } else {
                    encodings.round_output[round - 1][pos].decode(x as u8)
                };

                // AddRoundKey + SubBytes
                let after_key = decoded ^ round_keys[round][pos];
                let after_sbox = SBOX[after_key as usize];

                // Output: encode for this round's output
                let encoded = encodings.round_output[round][pos].encode(after_sbox);

                tables.tbox[round][pos][x] = encoded;
            }
        }
    }

    // Last round T-boxes
    for pos in 0..AES_BLOCK_SIZE {
        for x in 0..256 {
            let decoded = encodings.round_output[AES_ROUNDS - 2][pos].decode(x as u8);
            let after_key = decoded ^ round_keys[AES_ROUNDS - 1][pos];
            tables.tbox_last[pos][x] = SBOX[after_key as usize];
        }
    }

    tables
}

/// Internal encodings for each round
struct InternalEncodings {
    /// Output encoding for each round and position
    round_output: [[Bijection8; AES_BLOCK_SIZE]; AES_ROUNDS],
    /// Nibble encodings for XOR tables
    nibble_encodings: [[[Bijection4; 2]; 96]; 9],
}

/// Generate random bijection encodings
fn generate_encodings(rng: &mut SeededRng) -> InternalEncodings {
    let mut encodings = InternalEncodings {
        round_output: [[Bijection8::identity(); AES_BLOCK_SIZE]; AES_ROUNDS],
        nibble_encodings: [[[Bijection4::identity(); 2]; 96]; 9],
    };

    // Generate 8-bit bijections for round outputs
    for round in 0..AES_ROUNDS {
        for pos in 0..AES_BLOCK_SIZE {
            let perm = rng.random_permutation(256);
            let mut bij = Bijection8::identity();
            for (i, &p) in perm.iter().enumerate() {
                bij.forward[i] = p;
                bij.inverse[p as usize] = i as u8;
            }
            encodings.round_output[round][pos] = bij;
        }
    }

    // Generate 4-bit bijections for XOR tables
    for round in 0..9 {
        for table in 0..96 {
            for nibble in 0..2 {
                let perm = rng.random_permutation(16);
                let mut bij = Bijection4::identity();
                for (i, &p) in perm.iter().enumerate() {
                    bij.forward[i] = p;
                    bij.inverse[p as usize] = i as u8;
                }
                encodings.nibble_encodings[round][table][nibble] = bij;
            }
        }
    }

    encodings
}

/// Generate mixing bijection matrices
fn generate_mixing_bijections(rng: &mut SeededRng) -> [MixingBijection32; 9] {
    let mut mbs: [MixingBijection32; 9] = core::array::from_fn(|_| MixingBijection32::default());

    for round in 0..9 {
        // Generate random invertible 32x32 binary matrix
        // For simplicity, we use a variant with known structure
        let mut matrix = [[0u8; 32]; 32];
        let mut inverse = [[0u8; 32]; 32];

        // Start with identity
        for i in 0..32 {
            matrix[i][i] = 1;
            inverse[i][i] = 1;
        }

        // Apply random row operations (keeps matrix invertible)
        for _ in 0..64 {
            let i = (rng.next_u64() as usize) % 32;
            let j = (rng.next_u64() as usize) % 32;
            if i != j {
                // Add row j to row i (XOR in GF(2))
                for k in 0..32 {
                    matrix[i][k] ^= matrix[j][k];
                }
                // Inverse: add column i to column j
                for k in 0..32 {
                    inverse[k][j] ^= inverse[k][i];
                }
            }
        }

        mbs[round] = MixingBijection32 { matrix, inverse };
    }

    mbs
}

/// Generate T-boxes and Ty-boxes with MixColumns
fn generate_tboxes_tyboxes(
    round_keys: &[[u8; 16]; 11],
    encodings: &InternalEncodings,
    mixing_bijections: &[MixingBijection32; 9],
    tables: &mut WhiteboxTables,
) {
    // For rounds 0-8 (with MixColumns)
    for round in 0..9 {
        for col in 0..4 {
            for row in 0..4 {
                let pos = col * 4 + row;
                let shifted_pos = SHIFT_ROWS[pos];

                for x in 0..256 {
                    // Decode input from previous round's encoding
                    let decoded = if round == 0 {
                        x as u8
                    } else {
                        encodings.round_output[round - 1][pos].decode(x as u8)
                    };

                    // AddRoundKey
                    let after_key = decoded ^ round_keys[round][shifted_pos];

                    // SubBytes
                    let after_sbox = SBOX[after_key as usize];

                    // MixColumns contribution (this byte's contribution to column output)
                    // Each input byte contributes to 4 output bytes via MixColumns
                    let mut mc_out = [0u8; 4];
                    for out_row in 0..4 {
                        mc_out[out_row] = gf_mul(MIX_COLS[out_row][row], after_sbox);
                    }

                    // Pack into 32-bit value
                    let packed = (mc_out[0] as u32)
                        | ((mc_out[1] as u32) << 8)
                        | ((mc_out[2] as u32) << 16)
                        | ((mc_out[3] as u32) << 24);

                    // Apply mixing bijection
                    let mixed = mixing_bijections[round].apply(packed);

                    tables.tybox[round][pos][x] = mixed;

                    // Also store in tbox for reference
                    tables.tbox[round][pos][x] = after_sbox;
                }
            }
        }
    }
}

/// Generate XOR tables with nibble encodings
fn generate_xor_tables(encodings: &InternalEncodings, tables: &mut WhiteboxTables) {
    for round in 0..9 {
        for table_idx in 0..96 {
            for a in 0..16u8 {
                for b in 0..16u8 {
                    // Decode inputs
                    let a_decoded = encodings.nibble_encodings[round][table_idx][0].decode(a);
                    let b_decoded = encodings.nibble_encodings[round][table_idx][1].decode(b);

                    // XOR
                    let result = a_decoded ^ b_decoded;

                    // For output encoding, use next table's input or identity
                    let encoded_result = if table_idx + 1 < 96 {
                        encodings.nibble_encodings[round][(table_idx + 1) % 96][0].encode(result)
                    } else {
                        result
                    };

                    tables.xor_tables[round][table_idx][a as usize][b as usize] = encoded_result;
                }
            }
        }
    }
}

/// Generate MBL tables (inverse mixing bijection × L encoding)
fn generate_mbl_tables(
    mixing_bijections: &[MixingBijection32; 9],
    encodings: &InternalEncodings,
    tables: &mut WhiteboxTables,
) {
    for round in 0..9 {
        for pos in 0..AES_BLOCK_SIZE {
            for x in 0..256 {
                // L encoding: spread 8-bit value into 32-bit representation
                let l_encoded = (x as u32) << ((pos % 4) * 8);

                // Apply inverse mixing bijection
                let unmixed = mixing_bijections[round].apply_inverse(l_encoded);

                // Apply output encoding
                let out_bytes = [
                    unmixed as u8,
                    (unmixed >> 8) as u8,
                    (unmixed >> 16) as u8,
                    (unmixed >> 24) as u8,
                ];

                let encoded_bytes = [
                    encodings.round_output[round][pos * 4 / 16 * 4].encode(out_bytes[0]),
                    encodings.round_output[round][pos * 4 / 16 * 4 + 1].encode(out_bytes[1]),
                    encodings.round_output[round][pos * 4 / 16 * 4 + 2].encode(out_bytes[2]),
                    encodings.round_output[round][pos * 4 / 16 * 4 + 3].encode(out_bytes[3]),
                ];

                tables.mbl[round][pos][x] = (encoded_bytes[0] as u32)
                    | ((encoded_bytes[1] as u32) << 8)
                    | ((encoded_bytes[2] as u32) << 16)
                    | ((encoded_bytes[3] as u32) << 24);
            }
        }
    }
}

/// Generate last round T-boxes (no MixColumns)
fn generate_last_round_tboxes(
    round_keys: &[[u8; 16]; 11],
    encodings: &InternalEncodings,
    tables: &mut WhiteboxTables,
) {
    let round = AES_ROUNDS - 1; // Round 9

    for pos in 0..AES_BLOCK_SIZE {
        let shifted_pos = SHIFT_ROWS[pos];

        for x in 0..256 {
            // Decode from round 8 output encoding
            let decoded = encodings.round_output[round - 1][pos].decode(x as u8);

            // AddRoundKey
            let after_key = decoded ^ round_keys[round][shifted_pos];

            // SubBytes (no MixColumns in last round)
            let after_sbox = SBOX[after_key as usize];

            // Final AddRoundKey
            let result = after_sbox ^ round_keys[AES_ROUNDS][shifted_pos];

            tables.tbox_last[pos][x] = result;
            tables.tbox[round][pos][x] = result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_rng_deterministic() {
        let seed = b"test_seed_12345";
        let mut rng1 = SeededRng::new(seed);
        let mut rng2 = SeededRng::new(seed);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let mut rng1 = SeededRng::new(b"seed1");
        let mut rng2 = SeededRng::new(b"seed2");

        // Should produce different sequences
        let mut same = true;
        for _ in 0..10 {
            if rng1.next_u64() != rng2.next_u64() {
                same = false;
                break;
            }
        }
        assert!(!same);
    }

    #[test]
    fn test_random_permutation() {
        let mut rng = SeededRng::new(b"permutation_test");
        let perm = rng.random_permutation(256);

        // Check it's a valid permutation
        assert_eq!(perm.len(), 256);
        let mut seen = [false; 256];
        for &p in &perm {
            assert!(!seen[p as usize], "Duplicate value in permutation");
            seen[p as usize] = true;
        }
    }

    #[test]
    fn test_generate_tables() {
        let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                   0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let seed = b"test_build_seed";

        let tables = generate_tables(&key, seed);

        // Verify tables are not all zeros
        let mut has_nonzero_tybox = false;
        for round in 0..9 {
            for pos in 0..16 {
                for x in 0..256 {
                    if tables.tybox[round][pos][x] != 0 {
                        has_nonzero_tybox = true;
                        break;
                    }
                }
            }
        }
        assert!(has_nonzero_tybox, "Ty-boxes should have non-zero values");

        // Verify memory size
        assert!(tables.memory_size() > 500_000);
    }

    #[test]
    fn test_generate_tables_lite() {
        let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                   0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let seed = b"test_build_seed";

        let tables = generate_tables_lite(&key, seed);

        // Verify tables are populated
        let mut has_nonzero = false;
        for round in 0..AES_ROUNDS {
            for pos in 0..16 {
                for x in 0..256 {
                    if tables.tbox[round][pos][x] != 0 {
                        has_nonzero = true;
                        break;
                    }
                }
            }
        }
        assert!(has_nonzero, "T-boxes should have non-zero values");

        // Verify smaller memory footprint
        assert!(tables.memory_size() < 50_000);
    }

    #[test]
    fn test_tables_deterministic() {
        let key = [0x00; 16];
        let seed = b"deterministic_test";

        let tables1 = generate_tables(&key, seed);
        let tables2 = generate_tables(&key, seed);

        // Same seed should produce same tables
        for round in 0..9 {
            for pos in 0..16 {
                for x in 0..256 {
                    assert_eq!(
                        tables1.tybox[round][pos][x],
                        tables2.tybox[round][pos][x],
                        "Tables should be deterministic"
                    );
                }
            }
        }
    }
}
