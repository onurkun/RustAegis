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

/// Initialize whitebox tables from build-time embedded data
///
/// **SECURITY**: The AES key was used ONLY during compilation.
/// It does NOT exist anywhere at runtime - true white-box security!
/// Tables are reconstructed from entropy pool + deltas.
///
/// # Example
/// ```ignore
/// use aegis_vm::whitebox;
///
/// // Initialize tables at startup (no key involved!)
/// let tables = whitebox::init_tables();
///
/// // Use for encryption
/// let mut block = [0u8; 16];
/// whitebox::whitebox_encrypt(&mut block, &tables);
/// ```
pub fn init_tables() -> WhiteboxTables {
    use crate::build_config::whitebox_config::{
        reconstruct_tbox, reconstruct_tybox, reconstruct_xor_tables,
        reconstruct_mbl, reconstruct_tbox_last
    };

    // Reconstruct tables from entropy pool + deltas
    // NO KEY INVOLVED - key was only used at build-time!
    WhiteboxTables {
        tbox: reconstruct_tbox(),
        tybox: reconstruct_tybox(),
        xor_tables: reconstruct_xor_tables(),
        mbl: reconstruct_mbl(),
        tbox_last: reconstruct_tbox_last(),
        input_encoding: None,
        output_encoding_inv: None,
    }
}

/// Initialize lightweight whitebox tables (~40KB instead of ~500KB)
/// Less secure but smaller footprint
///
/// **NOTE**: Lite tables still use runtime key derivation for now.
/// For maximum security, use full `init_tables()` instead.
pub fn init_tables_lite() -> WhiteboxTablesLite {
    use crate::build_config::whitebox_config::reconstruct_tbox;

    // For lite version, we only need tbox and tbox_last
    let full_tbox = reconstruct_tbox();

    // Extract tbox_last from round 9
    let mut tbox_last = [[0u8; 256]; AES_BLOCK_SIZE];
    for (pos, tbox_pos) in tbox_last.iter_mut().enumerate() {
        tbox_pos.copy_from_slice(&full_tbox[AES_ROUNDS - 1][pos]);
    }

    WhiteboxTablesLite {
        tbox: full_tbox,
        tbox_last,
    }
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

/// Derive a cryptographic key using WBC from pre-computed domain hash
///
/// This hides the key derivation process in the whitebox tables,
/// making it much harder to reverse engineer than plain HMAC.
///
/// **SECURITY**: Domain strings are hashed at BUILD TIME.
/// Runtime only sees 32 random-looking bytes, not the actual string!
///
/// # Arguments
/// * `domain_hash` - Pre-computed 32-byte hash of domain string (from build-time)
///
/// # Returns
/// A 32-byte derived key
pub fn derive_key_from_hash(domain_hash: &[u8; 32]) -> [u8; 32] {
    let tables = init_tables();
    derive_key_from_hash_with_tables(domain_hash, &tables)
}

/// Derive key from pre-computed hash using pre-initialized tables
///
/// **SECURITY**: This function takes a pre-computed FNV hash instead of a string.
/// The domain string never exists in the binary!
pub fn derive_key_from_hash_with_tables(domain_hash: &[u8; 32], tables: &WhiteboxTables) -> [u8; 32] {
    // domain_hash layout (computed at build-time):
    // [0..8]   = hash1 (FNV-1a)
    // [8..16]  = hash2 (FNV-1a variant)
    // [16..24] = hash3 (hash1.rotate_left(13) ^ hash2)
    // [24..32] = hash4 (hash2.rotate_right(17) ^ hash1)

    // Create two 16-byte blocks from pre-computed hash
    let mut block1 = [0u8; AES_BLOCK_SIZE];
    let mut block2 = [0u8; AES_BLOCK_SIZE];

    // Block 1: hash1 + hash2
    block1.copy_from_slice(&domain_hash[0..16]);

    // Block 2: hash3 + hash4
    block2.copy_from_slice(&domain_hash[16..32]);

    // Encrypt both blocks through WBC
    whitebox_encrypt(&mut block1, tables);
    whitebox_encrypt(&mut block2, tables);

    // Combine into 32-byte key
    let mut key = [0u8; 32];
    key[0..16].copy_from_slice(&block1);
    key[16..32].copy_from_slice(&block2);

    key
}

/// Derive key using pre-initialized tables (legacy, for custom domains)
///
/// **WARNING**: This function includes the domain string in the binary!
/// For built-in domains, use derive_bytecode_key() or derive_smc_key() instead.
#[deprecated(note = "Use derive_key_from_hash for security - domain strings are visible in binary")]
pub fn derive_key_with_tables(domain: &[u8], tables: &WhiteboxTables) -> [u8; 32] {
    // Simple domain hashing (FNV-1a style, split into two blocks)
    let mut hash1 = 0xcbf29ce484222325u64;
    let mut hash2 = 0x84222325cbf29ce4u64;

    for &byte in domain {
        hash1 ^= byte as u64;
        hash1 = hash1.wrapping_mul(0x100000001b3);
        hash2 = hash2.wrapping_mul(0x100000001b3);
        hash2 ^= byte as u64;
    }

    let hash3 = hash1.rotate_left(13) ^ hash2;
    let hash4 = hash2.rotate_right(17) ^ hash1;

    let mut domain_hash = [0u8; 32];
    domain_hash[0..8].copy_from_slice(&hash1.to_le_bytes());
    domain_hash[8..16].copy_from_slice(&hash2.to_le_bytes());
    domain_hash[16..24].copy_from_slice(&hash3.to_le_bytes());
    domain_hash[24..32].copy_from_slice(&hash4.to_le_bytes());

    derive_key_from_hash_with_tables(&domain_hash, tables)
}

/// Derive the bytecode encryption key using WBC
///
/// **SECURITY**: Uses build-time pre-computed domain hash.
/// The string "aegis-bytecode-encryption-v1" does NOT exist in the binary!
pub fn derive_bytecode_key() -> [u8; 32] {
    use crate::build_config::whitebox_config::get_bytecode_domain_hash;
    let domain_hash = get_bytecode_domain_hash();
    derive_key_from_hash(&domain_hash)
}

/// Derive the SMC (Self-Modifying Code) key using WBC
///
/// **SECURITY**: Uses build-time pre-computed domain hash.
/// The string "aegis-smc-key-v1" does NOT exist in the binary!
pub fn derive_smc_key() -> [u8; 32] {
    use crate::build_config::whitebox_config::get_smc_domain_hash;
    let domain_hash = get_smc_domain_hash();
    derive_key_from_hash(&domain_hash)
}

/// Derive a nonce using WBC
///
/// Uses counter to ensure unique nonces for each encryption.
///
/// **SECURITY**: Uses build-time pre-computed nonce domain hash.
/// The string "wbc-nonce" does NOT exist in the binary!
pub fn derive_nonce(counter: u64) -> [u8; 12] {
    use crate::build_config::whitebox_config::get_nonce_domain_hash;

    let tables = init_tables();
    let nonce_hash = get_nonce_domain_hash();

    // Create block from counter + pre-computed nonce domain hash
    let mut block = [0u8; AES_BLOCK_SIZE];
    block[0..8].copy_from_slice(&counter.to_le_bytes());
    // Use first 8 bytes of nonce domain hash instead of "wbc-nonce" string
    block[8..16].copy_from_slice(&nonce_hash[0..8]);

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

impl Default for WhiteboxCryptoContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WhiteboxCryptoContext {
    /// Create new WBC crypto context
    ///
    /// Generates tables and derives keys at startup (cold path).
    ///
    /// **SECURITY**: Uses build-time pre-computed domain hashes.
    /// No domain strings exist in the binary!
    pub fn new() -> Self {
        use crate::build_config::whitebox_config::{
            get_bytecode_domain_hash, get_smc_domain_hash
        };

        let tables = init_tables();

        // Use pre-computed domain hashes (strings don't exist in binary!)
        let bytecode_hash = get_bytecode_domain_hash();
        let smc_hash = get_smc_domain_hash();

        let bytecode_key = derive_key_from_hash_with_tables(&bytecode_hash, &tables);
        let smc_key = derive_key_from_hash_with_tables(&smc_hash, &tables);

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
    ///
    /// **SECURITY**: Uses build-time pre-computed nonce domain hash.
    pub fn next_nonce(&mut self) -> [u8; 12] {
        use crate::build_config::whitebox_config::get_nonce_domain_hash;

        let nonce_hash = get_nonce_domain_hash();

        let mut block = [0u8; AES_BLOCK_SIZE];
        block[0..8].copy_from_slice(&self.nonce_counter.to_le_bytes());
        // Use first 8 bytes of nonce domain hash instead of "wbc-nonce" string
        block[8..16].copy_from_slice(&nonce_hash[0..8]);

        self.nonce_counter += 1;

        whitebox_encrypt(&mut block, &self.tables);

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&block[0..12]);
        nonce
    }

    /// Derive a custom key for any domain
    #[allow(deprecated)]
    pub fn derive_custom_key(&self, domain: &[u8]) -> [u8; 32] {
        derive_key_with_tables(domain, &self.tables)
    }

    /// Encrypt a single block using WBC (for key obfuscation, not bulk data)
    pub fn wbc_encrypt(&self, block: &mut [u8; AES_BLOCK_SIZE]) {
        whitebox_encrypt(block, &self.tables);
    }
}
