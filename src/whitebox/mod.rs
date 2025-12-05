// White-Box AES Implementation (Chow et al. scheme)
// Build-time table generation, runtime lookup-based encryption

mod sbox;
mod tables;
mod generator;
mod cipher;

pub use tables::{WhiteboxTables, WhiteboxTablesLite, WHITEBOX_TABLE_SIZE};
pub use cipher::{whitebox_encrypt, whitebox_decrypt, whitebox_encrypt_lite};
pub use generator::{generate_tables, generate_tables_lite};

/// AES block size in bytes
pub const AES_BLOCK_SIZE: usize = 16;

/// Number of AES-128 rounds
pub const AES_ROUNDS: usize = 10;

/// Whitebox key size (AES-128)
pub const WB_KEY_SIZE: usize = 16;

/// Initialize whitebox tables using build-time derived key
/// This should be called once at startup (cold path only!)
///
/// # Example
/// ```ignore
/// use aegis_vm::whitebox;
///
/// // Initialize tables at startup
/// let tables = whitebox::init_tables();
///
/// // Use for encryption
/// let mut block = [0u8; 16];
/// whitebox::whitebox_encrypt(&mut block, &tables);
/// ```
pub fn init_tables() -> WhiteboxTables {
    use crate::build_config::whitebox_config::{WBC_KEY, WBC_TABLE_SEED};
    generate_tables(&WBC_KEY, &WBC_TABLE_SEED)
}

/// Initialize lightweight whitebox tables (~40KB instead of ~500KB)
/// Less secure but smaller footprint
pub fn init_tables_lite() -> WhiteboxTablesLite {
    use crate::build_config::whitebox_config::{WBC_KEY, WBC_TABLE_SEED};
    generate_tables_lite(&WBC_KEY, &WBC_TABLE_SEED)
}

/// Encrypt a block using build-time derived key (convenience function)
/// Creates tables on-demand - for repeated use, prefer init_tables() once
pub fn encrypt_block(block: &mut [u8; AES_BLOCK_SIZE]) {
    let tables = init_tables();
    whitebox_encrypt(block, &tables);
}

/// Encrypt multiple blocks using the same tables
/// More efficient than calling encrypt_block repeatedly
pub fn encrypt_blocks(blocks: &mut [[u8; AES_BLOCK_SIZE]]) {
    let tables = init_tables();
    for block in blocks.iter_mut() {
        whitebox_encrypt(block, &tables);
    }
}

// ============================================================================
// WBC-Based Key Derivation (for hiding key generation in tables)
// ============================================================================

/// Derive a cryptographic key using WBC
///
/// This hides the key derivation process in the whitebox tables,
/// making it much harder to reverse engineer than plain HMAC.
///
/// # Arguments
/// * `domain` - Domain separation string (e.g., "bytecode-key", "smc-key")
///
/// # Returns
/// A 32-byte derived key
pub fn derive_key(domain: &[u8]) -> [u8; 32] {
    let tables = init_tables();
    derive_key_with_tables(domain, &tables)
}

/// Derive key using pre-initialized tables (more efficient for multiple derivations)
pub fn derive_key_with_tables(domain: &[u8], tables: &WhiteboxTables) -> [u8; 32] {
    // Create two 16-byte blocks from domain
    // Block 1: domain hash (first 16 bytes)
    // Block 2: domain hash (last 16 bytes, different)
    let mut block1 = [0u8; AES_BLOCK_SIZE];
    let mut block2 = [0u8; AES_BLOCK_SIZE];

    // Simple domain hashing (FNV-1a style, split into two blocks)
    let mut hash1 = 0xcbf29ce484222325u64;
    let mut hash2 = 0x84222325cbf29ce4u64;

    for &byte in domain {
        hash1 ^= byte as u64;
        hash1 = hash1.wrapping_mul(0x100000001b3);
        hash2 = hash2.wrapping_mul(0x100000001b3);
        hash2 ^= byte as u64;
    }

    // Fill blocks with hash values
    block1[0..8].copy_from_slice(&hash1.to_le_bytes());
    block1[8..16].copy_from_slice(&hash2.to_le_bytes());

    // Second block uses rotated/modified hash
    let hash3 = hash1.rotate_left(13) ^ hash2;
    let hash4 = hash2.rotate_right(17) ^ hash1;
    block2[0..8].copy_from_slice(&hash3.to_le_bytes());
    block2[8..16].copy_from_slice(&hash4.to_le_bytes());

    // Encrypt both blocks through WBC
    whitebox_encrypt(&mut block1, tables);
    whitebox_encrypt(&mut block2, tables);

    // Combine into 32-byte key
    let mut key = [0u8; 32];
    key[0..16].copy_from_slice(&block1);
    key[16..32].copy_from_slice(&block2);

    key
}

/// Derive the bytecode encryption key using WBC
///
/// This replaces the plain HMAC-based key derivation with WBC-hidden derivation.
pub fn derive_bytecode_key() -> [u8; 32] {
    derive_key(b"aegis-bytecode-encryption-v1")
}

/// Derive the SMC (Self-Modifying Code) key using WBC
pub fn derive_smc_key() -> [u8; 32] {
    derive_key(b"aegis-smc-key-v1")
}

/// Derive a nonce using WBC
///
/// Uses counter to ensure unique nonces for each encryption.
pub fn derive_nonce(counter: u64) -> [u8; 12] {
    let tables = init_tables();

    // Create block from counter
    let mut block = [0u8; AES_BLOCK_SIZE];
    block[0..8].copy_from_slice(&counter.to_le_bytes());
    block[8..16].copy_from_slice(b"wbc-nonce");

    whitebox_encrypt(&mut block, &tables);

    // Take first 12 bytes as nonce
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&block[0..12]);
    nonce
}

/// WBC-protected crypto context
///
/// Uses whitebox tables to derive keys instead of plain HMAC.
/// This makes key extraction significantly harder.
pub struct WhiteboxCryptoContext {
    /// WBC tables (generated once at startup)
    tables: WhiteboxTables,
    /// Derived bytecode encryption key
    bytecode_key: [u8; 32],
    /// Derived SMC key
    smc_key: [u8; 32],
    /// Nonce counter
    nonce_counter: u64,
}

impl WhiteboxCryptoContext {
    /// Create new WBC crypto context
    ///
    /// Generates tables and derives keys at startup (cold path).
    pub fn new() -> Self {
        let tables = init_tables();
        let bytecode_key = derive_key_with_tables(b"aegis-bytecode-encryption-v1", &tables);
        let smc_key = derive_key_with_tables(b"aegis-smc-key-v1", &tables);

        Self {
            tables,
            bytecode_key,
            smc_key,
            nonce_counter: 0,
        }
    }

    /// Get the bytecode encryption key
    pub fn bytecode_key(&self) -> &[u8; 32] {
        &self.bytecode_key
    }

    /// Get the SMC key
    pub fn smc_key(&self) -> &[u8; 32] {
        &self.smc_key
    }

    /// Derive a unique nonce for encryption
    pub fn next_nonce(&mut self) -> [u8; 12] {
        let mut block = [0u8; AES_BLOCK_SIZE];
        block[0..8].copy_from_slice(&self.nonce_counter.to_le_bytes());
        block[8..16].copy_from_slice(b"wbc-nonce");

        self.nonce_counter += 1;

        whitebox_encrypt(&mut block, &self.tables);

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&block[0..12]);
        nonce
    }

    /// Derive a custom key for any domain
    pub fn derive_custom_key(&self, domain: &[u8]) -> [u8; 32] {
        derive_key_with_tables(domain, &self.tables)
    }

    /// Encrypt a single block using WBC (for key obfuscation, not bulk data)
    pub fn wbc_encrypt(&self, block: &mut [u8; AES_BLOCK_SIZE]) {
        whitebox_encrypt(block, &self.tables);
    }
}
