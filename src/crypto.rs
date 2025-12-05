//! Cryptographic operations for bytecode encryption
//!
//! Uses AES-256-GCM for authenticated encryption and HMAC-SHA256 for key derivation.

use crate::error::{VmError, VmResult};
use aes_gcm::{
    aead::Aead,
    Aes256Gcm, Nonce, KeyInit,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// XOR-decode bytes at runtime (obfuscates string constants)
#[inline(always)]
const fn xor_decode<const N: usize>(encoded: &[u8; N], key: u8) -> [u8; N] {
    let mut result = [0u8; N];
    let mut i = 0;
    while i < N {
        result[i] = encoded[i] ^ key;
        i += 1;
    }
    result
}

// Obfuscated domain strings (XOR key = 0x5A)
// "anticheat-vm-key-v1" XOR 0x5A
const KEY_DOMAIN_ENC: [u8; 19] = [
    0x3b, 0x34, 0x2e, 0x33, 0x39, 0x32, 0x3f, 0x3b,
    0x2e, 0x77, 0x2c, 0x37, 0x77, 0x31, 0x3f, 0x23,
    0x77, 0x2c, 0x6b
];
// "anticheat-vm-nonce-v1" XOR 0x5A
const NONCE_DOMAIN_ENC: [u8; 21] = [
    0x3b, 0x34, 0x2e, 0x33, 0x39, 0x32, 0x3f, 0x3b,
    0x2e, 0x77, 0x2c, 0x37, 0x77, 0x34, 0x35, 0x34,
    0x39, 0x3f, 0x77, 0x2c, 0x6b
];
// "anticheat-vm-build-id-v1" XOR 0x5A
const BUILDID_DOMAIN_ENC: [u8; 24] = [
    0x3b, 0x34, 0x2e, 0x33, 0x39, 0x32, 0x3f, 0x3b,
    0x2e, 0x77, 0x2c, 0x37, 0x77, 0x38, 0x2f, 0x33,
    0x36, 0x3e, 0x77, 0x33, 0x3e, 0x77, 0x2c, 0x6b
];

/// AES-256 key size in bytes
pub const KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes
pub const NONCE_SIZE: usize = 12;

/// AES-GCM tag size in bytes
pub const TAG_SIZE: usize = 16;

/// HMAC-SHA256 type alias
type HmacSha256 = Hmac<Sha256>;

/// Derive encryption key from build seed using HMAC-SHA256
///
/// This creates a unique key per build, making each build's encryption different.
pub fn derive_key(build_seed: &[u8; 32], context: &[u8]) -> [u8; KEY_SIZE] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(build_seed)
        .expect("HMAC can take any size key");
    mac.update(context);
    // Use obfuscated domain string
    let domain = xor_decode(&KEY_DOMAIN_ENC, 0x5A);
    mac.update(&domain);

    let result = mac.finalize();
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&result.into_bytes()[..KEY_SIZE]);
    key
}

/// Derive nonce from counter and build seed
///
/// Uses HMAC to create unpredictable nonces while maintaining uniqueness.
/// Counter ensures we never reuse a nonce with the same key.
pub fn derive_nonce(build_seed: &[u8; 32], counter: u64) -> [u8; NONCE_SIZE] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(build_seed)
        .expect("HMAC can take any size key");
    mac.update(&counter.to_le_bytes());
    // Use obfuscated domain string
    let domain = xor_decode(&NONCE_DOMAIN_ENC, 0x5A);
    mac.update(&domain);

    let result = mac.finalize();
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.copy_from_slice(&result.into_bytes()[..NONCE_SIZE]);
    nonce
}

/// Encrypt bytecode using AES-256-GCM
///
/// Returns (ciphertext, tag)
pub fn encrypt_bytecode(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
) -> VmResult<(Vec<u8>, [u8; TAG_SIZE])> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| VmError::DecryptionFailed)?;

    let nonce_obj = Nonce::from_slice(nonce);

    // The aes-gcm crate appends the tag to the ciphertext
    let ciphertext = cipher
        .encrypt(nonce_obj, plaintext)
        .map_err(|_| VmError::DecryptionFailed)?;

    // Extract tag from the end of ciphertext
    if ciphertext.len() < TAG_SIZE {
        return Err(VmError::DecryptionFailed);
    }

    let tag_start = ciphertext.len() - TAG_SIZE;
    let mut tag = [0u8; TAG_SIZE];
    tag.copy_from_slice(&ciphertext[tag_start..]);

    let encrypted_data = ciphertext[..tag_start].to_vec();

    Ok((encrypted_data, tag))
}

/// Decrypt bytecode using AES-256-GCM
pub fn decrypt_bytecode(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
    tag: &[u8; TAG_SIZE],
) -> VmResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| VmError::DecryptionFailed)?;

    let nonce_obj = Nonce::from_slice(nonce);

    // Reconstruct ciphertext with tag appended (as expected by aes-gcm)
    let mut full_ciphertext = Vec::with_capacity(ciphertext.len() + TAG_SIZE);
    full_ciphertext.extend_from_slice(ciphertext);
    full_ciphertext.extend_from_slice(tag);

    let plaintext = cipher
        .decrypt(nonce_obj, full_ciphertext.as_slice())
        .map_err(|_| VmError::DecryptionFailed)?;

    Ok(plaintext)
}

/// Compute HMAC-SHA256 for integrity verification
pub fn compute_hmac(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .expect("HMAC can take any size key");
    mac.update(data);

    let result = mac.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result.into_bytes());
    output
}

/// Verify HMAC-SHA256 in constant time
pub fn verify_hmac(key: &[u8], data: &[u8], expected: &[u8; 32]) -> bool {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .expect("HMAC can take any size key");
    mac.update(data);

    mac.verify_slice(expected).is_ok()
}

/// Derive build ID from seed (first 8 bytes of HMAC)
pub fn derive_build_id(build_seed: &[u8; 32]) -> u64 {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(build_seed)
        .expect("HMAC can take any size key");
    // Use obfuscated domain string
    let domain = xor_decode(&BUILDID_DOMAIN_ENC, 0x5A);
    mac.update(&domain);

    let result = mac.finalize();
    let bytes = result.into_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Cryptographic context for VM operations
pub struct CryptoContext {
    /// Derived encryption key
    key: [u8; KEY_SIZE],
    /// Build seed (for nonce derivation)
    build_seed: [u8; 32],
    /// Nonce counter (must be incremented for each encryption)
    nonce_counter: u64,
    /// Build ID derived from seed
    pub build_id: u64,
    /// WBC crypto context (when whitebox feature is enabled)
    #[cfg(feature = "whitebox")]
    wbc_context: Option<crate::whitebox::WhiteboxCryptoContext>,
}

impl CryptoContext {
    /// Create new crypto context from build seed
    ///
    /// Bytecode encryption always uses HMAC-based key derivation for compatibility
    /// with compile-time encrypted bytecode. When `whitebox` feature is enabled,
    /// WBC is used for SMC key and other runtime secrets.
    pub fn new(build_seed: [u8; 32]) -> Self {
        // Bytecode key is ALWAYS derived via HMAC for backward compatibility
        // (proc-macro encrypts at compile time using HMAC)
        let key = derive_key(&build_seed, b"bytecode-encryption");
        let build_id = derive_build_id(&build_seed);

        #[cfg(feature = "whitebox")]
        {
            // Initialize WBC for SMC key and other runtime secrets
            let wbc = crate::whitebox::WhiteboxCryptoContext::new();

            Self {
                key,
                build_seed,
                nonce_counter: 0,
                build_id,
                wbc_context: Some(wbc),
            }
        }

        #[cfg(not(feature = "whitebox"))]
        {
            Self {
                key,
                build_seed,
                nonce_counter: 0,
                build_id,
            }
        }
    }

    /// Create new crypto context with WBC explicitly enabled
    #[cfg(feature = "whitebox")]
    pub fn new_with_wbc() -> Self {
        let build_seed = crate::build_config::get_build_seed();
        Self::new(build_seed)
    }

    /// Get the SMC key (derived from WBC when enabled)
    ///
    /// This is where WBC provides protection - the SMC key derivation
    /// is hidden inside whitebox tables, making key extraction much harder.
    #[cfg(feature = "whitebox")]
    pub fn smc_key(&self) -> [u8; KEY_SIZE] {
        if let Some(ref wbc) = self.wbc_context {
            *wbc.smc_key()
        } else {
            derive_key(&self.build_seed, b"smc-encryption")
        }
    }

    /// Get the SMC key (standard derivation when WBC disabled)
    #[cfg(not(feature = "whitebox"))]
    pub fn smc_key(&self) -> [u8; KEY_SIZE] {
        derive_key(&self.build_seed, b"smc-encryption")
    }

    /// Encrypt bytecode and return package data
    pub fn encrypt(&mut self, plaintext: &[u8]) -> VmResult<(Vec<u8>, [u8; NONCE_SIZE], [u8; TAG_SIZE])> {
        let nonce = derive_nonce(&self.build_seed, self.nonce_counter);
        self.nonce_counter += 1;

        let (ciphertext, tag) = encrypt_bytecode(&self.key, &nonce, plaintext)?;
        Ok((ciphertext, nonce, tag))
    }

    /// Decrypt bytecode
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; NONCE_SIZE], tag: &[u8; TAG_SIZE]) -> VmResult<Vec<u8>> {
        decrypt_bytecode(&self.key, nonce, ciphertext, tag)
    }

    /// Derive a WBC-protected key for runtime secrets
    ///
    /// When WBC is enabled, this uses whitebox tables for key derivation.
    /// Use this for runtime secrets (not bytecode which needs compile-time compatibility).
    #[cfg(feature = "whitebox")]
    pub fn derive_wbc_key(&self, domain: &[u8]) -> [u8; KEY_SIZE] {
        if let Some(ref wbc) = self.wbc_context {
            wbc.derive_custom_key(domain)
        } else {
            derive_key(&self.build_seed, domain)
        }
    }

    /// Derive a custom key for a specific purpose (HMAC-based)
    pub fn derive_custom_key(&self, domain: &[u8]) -> [u8; KEY_SIZE] {
        derive_key(&self.build_seed, domain)
    }

    /// Get access to WBC context for advanced usage
    #[cfg(feature = "whitebox")]
    pub fn wbc(&self) -> Option<&crate::whitebox::WhiteboxCryptoContext> {
        self.wbc_context.as_ref()
    }
}
