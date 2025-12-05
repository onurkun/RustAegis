// White-box AES table structures (Chow et al. scheme)

use super::{AES_ROUNDS, AES_BLOCK_SIZE};

/// Total size of whitebox tables in bytes (approximate)
/// T-boxes: 10 * 16 * 256 = 40KB
/// Ty-boxes: 9 * 16 * 256 * 4 = 147KB
/// XOR tables: 9 * 96 * 16 * 16 = 221KB
/// MBL tables: 9 * 16 * 256 * 4 = 147KB
/// Total: ~555KB
pub const WHITEBOX_TABLE_SIZE: usize = 555_000;

/// T-box: SubBytes + AddRoundKey combined
/// For each round r and byte position i, Tbox[r][i][x] = S(x ^ k[r][i])
/// Size: 10 rounds × 16 positions × 256 values = 40,960 bytes
pub type TBox = [[[u8; 256]; AES_BLOCK_SIZE]; AES_ROUNDS];

/// Ty-box: T-box with MixColumns embedded
/// Output is 32-bit because MixColumns mixes 4 bytes together
/// For rounds 0-8 (MixColumns applied)
/// Size: 9 rounds × 16 positions × 256 values × 4 bytes = 147,456 bytes
pub type TyBox = [[[u32; 256]; AES_BLOCK_SIZE]; 9];

/// XOR tables for nibble-based XOR operations
/// Each table maps (a, b) -> a ^ b where a, b are 4-bit nibbles
/// Used to hide XOR operations in table lookups
/// Size: 9 rounds × 96 tables × 16 × 16 = 221,184 bytes
pub type XorTable = [[[[u8; 16]; 16]; 96]; 9];

/// Mixing Bijection tables (MBL = inv(MB) × L)
/// Used to encode/decode intermediate values between rounds
/// Size: 9 rounds × 16 positions × 256 values × 4 bytes = 147,456 bytes
pub type MBLTable = [[[u32; 256]; AES_BLOCK_SIZE]; 9];

/// Complete whitebox AES tables
#[derive(Clone)]
pub struct WhiteboxTables {
    /// T-boxes for all rounds (including last round without MixColumns)
    pub tbox: Box<TBox>,

    /// Ty-boxes with MixColumns embedded (rounds 0-8)
    pub tybox: Box<TyBox>,

    /// XOR tables for hiding XOR operations
    pub xor_tables: Box<XorTable>,

    /// MBL tables (inverse mixing bijection × L encoding)
    pub mbl: Box<MBLTable>,

    /// Last round T-boxes (no MixColumns)
    pub tbox_last: [[u8; 256]; AES_BLOCK_SIZE],

    /// External input encoding (optional)
    pub input_encoding: Option<Box<[[u8; 256]; AES_BLOCK_SIZE]>>,

    /// External output encoding inverse (optional)
    pub output_encoding_inv: Option<Box<[[u8; 256]; AES_BLOCK_SIZE]>>,
}

impl Default for WhiteboxTables {
    fn default() -> Self {
        Self::new()
    }
}

impl WhiteboxTables {
    /// Create empty tables (to be filled by generator)
    pub fn new() -> Self {
        Self {
            tbox: Box::new([[[0u8; 256]; AES_BLOCK_SIZE]; AES_ROUNDS]),
            tybox: Box::new([[[0u32; 256]; AES_BLOCK_SIZE]; 9]),
            xor_tables: Box::new([[[[0u8; 16]; 16]; 96]; 9]),
            mbl: Box::new([[[0u32; 256]; AES_BLOCK_SIZE]; 9]),
            tbox_last: [[0u8; 256]; AES_BLOCK_SIZE],
            input_encoding: None,
            output_encoding_inv: None,
        }
    }

    /// Get approximate memory usage
    pub fn memory_size(&self) -> usize {
        let base = core::mem::size_of::<TBox>()
            + core::mem::size_of::<TyBox>()
            + core::mem::size_of::<XorTable>()
            + core::mem::size_of::<MBLTable>()
            + core::mem::size_of::<[[u8; 256]; AES_BLOCK_SIZE]>();

        let encoding_size = self.input_encoding.as_ref().map(|_| 4096).unwrap_or(0)
            + self.output_encoding_inv.as_ref().map(|_| 4096).unwrap_or(0);

        base + encoding_size
    }
}

/// Lightweight whitebox tables (T-boxes only, ~40KB)
/// Provides less protection but smaller footprint
#[derive(Clone)]
pub struct WhiteboxTablesLite {
    /// T-boxes for all rounds
    pub tbox: Box<TBox>,

    /// Last round T-boxes (no MixColumns)
    pub tbox_last: [[u8; 256]; AES_BLOCK_SIZE],
}

impl Default for WhiteboxTablesLite {
    fn default() -> Self {
        Self::new()
    }
}

impl WhiteboxTablesLite {
    pub fn new() -> Self {
        Self {
            tbox: Box::new([[[0u8; 256]; AES_BLOCK_SIZE]; AES_ROUNDS]),
            tbox_last: [[0u8; 256]; AES_BLOCK_SIZE],
        }
    }

    pub fn memory_size(&self) -> usize {
        core::mem::size_of::<TBox>()
            + core::mem::size_of::<[[u8; 256]; AES_BLOCK_SIZE]>()
    }
}

/// 8-bit random bijection (encoding function)
#[derive(Clone, Copy)]
pub struct Bijection8 {
    pub forward: [u8; 256],
    pub inverse: [u8; 256],
}

impl Bijection8 {
    /// Create identity bijection
    pub fn identity() -> Self {
        let mut forward = [0u8; 256];
        let mut inverse = [0u8; 256];
        for i in 0..256 {
            forward[i] = i as u8;
            inverse[i] = i as u8;
        }
        Self { forward, inverse }
    }

    /// Apply forward encoding
    #[inline]
    pub fn encode(&self, x: u8) -> u8 {
        self.forward[x as usize]
    }

    /// Apply inverse encoding (decode)
    #[inline]
    pub fn decode(&self, x: u8) -> u8 {
        self.inverse[x as usize]
    }
}

/// 4-bit random bijection (for nibble encoding)
#[derive(Clone, Copy)]
pub struct Bijection4 {
    pub forward: [u8; 16],
    pub inverse: [u8; 16],
}

impl Bijection4 {
    /// Create identity bijection
    pub fn identity() -> Self {
        let mut forward = [0u8; 16];
        let mut inverse = [0u8; 16];
        for i in 0..16 {
            forward[i] = i as u8;
            inverse[i] = i as u8;
        }
        Self { forward, inverse }
    }

    /// Apply forward encoding
    #[inline]
    pub fn encode(&self, x: u8) -> u8 {
        self.forward[(x & 0x0f) as usize]
    }

    /// Apply inverse encoding (decode)
    #[inline]
    pub fn decode(&self, x: u8) -> u8 {
        self.inverse[(x & 0x0f) as usize]
    }
}

/// 32x32 mixing bijection matrix
#[derive(Clone)]
pub struct MixingBijection32 {
    pub matrix: [[u8; 32]; 32],
    pub inverse: [[u8; 32]; 32],
}

impl Default for MixingBijection32 {
    fn default() -> Self {
        Self::identity()
    }
}

impl MixingBijection32 {
    /// Create identity mixing bijection
    pub fn identity() -> Self {
        let mut matrix = [[0u8; 32]; 32];
        let mut inverse = [[0u8; 32]; 32];
        for i in 0..32 {
            matrix[i][i] = 1;
            inverse[i][i] = 1;
        }
        Self { matrix, inverse }
    }

    /// Apply mixing bijection to 32-bit value
    pub fn apply(&self, input: u32) -> u32 {
        let mut result = 0u32;
        for i in 0..32 {
            let mut bit = 0u8;
            for j in 0..32 {
                if self.matrix[i][j] != 0 && (input >> j) & 1 != 0 {
                    bit ^= 1;
                }
            }
            result |= (bit as u32) << i;
        }
        result
    }

    /// Apply inverse mixing bijection
    pub fn apply_inverse(&self, input: u32) -> u32 {
        let mut result = 0u32;
        for i in 0..32 {
            let mut bit = 0u8;
            for j in 0..32 {
                if self.inverse[i][j] != 0 && (input >> j) & 1 != 0 {
                    bit ^= 1;
                }
            }
            result |= (bit as u32) << i;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bijection8_identity() {
        let bij = Bijection8::identity();
        for i in 0u8..=255 {
            assert_eq!(bij.encode(i), i);
            assert_eq!(bij.decode(i), i);
        }
    }

    #[test]
    fn test_bijection4_identity() {
        let bij = Bijection4::identity();
        for i in 0u8..16 {
            assert_eq!(bij.encode(i), i);
            assert_eq!(bij.decode(i), i);
        }
    }

    #[test]
    fn test_mixing_bijection_identity() {
        let mb = MixingBijection32::identity();
        assert_eq!(mb.apply(0x12345678), 0x12345678);
        assert_eq!(mb.apply_inverse(0xdeadbeef), 0xdeadbeef);
    }

    #[test]
    fn test_whitebox_tables_memory() {
        let tables = WhiteboxTables::new();
        let size = tables.memory_size();
        // Should be around 555KB
        assert!(size > 500_000);
        assert!(size < 600_000);
    }

    #[test]
    fn test_whitebox_tables_lite_memory() {
        let tables = WhiteboxTablesLite::new();
        let size = tables.memory_size();
        // Should be around 45KB
        assert!(size > 40_000);
        assert!(size < 50_000);
    }
}
