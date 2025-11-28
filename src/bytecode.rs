//! Encrypted bytecode format with header
//!
//! ## Bytecode Format
//!
//! ```text
//! +------------------+------------------+------------------+
//! | Magic (4 bytes)  | Version (2)      | Flags (2)        |
//! +------------------+------------------+------------------+
//! | Build ID (8)     | Timestamp (8)    |
//! +------------------+------------------+------------------+
//! | Nonce (12)       | Tag (16)         |
//! +------------------+------------------+
//! | Encrypted Code Length (4)          |
//! +------------------------------------+
//! | Encrypted Bytecode (variable)      |
//! +------------------------------------+
//! | Original Hash (8) - inside encrypted payload
//! +------------------------------------+
//! ```

use crate::error::{VmError, VmResult};
use crate::build_config;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Magic bytes for bytecode identification (randomized per build)
pub use build_config::MAGIC;

/// Current bytecode format version
pub const FORMAT_VERSION: u16 = 1;

/// Bytecode header flags
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytecodeFlags {
    /// No special flags
    None = 0,
    /// Code is encrypted
    Encrypted = 1 << 0,
    /// Includes integrity hash
    HasIntegrity = 1 << 1,
    /// Includes timing checks
    HasTimingChecks = 1 << 2,
    /// Paranoid mode (all protections)
    Paranoid = 1 << 3,
}

/// Protection level for bytecode generation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectionLevel {
    /// No obfuscation, useful for debugging
    Debug,
    /// Basic encryption only
    Low,
    /// Encryption + integrity checks
    Medium,
    /// Encryption + integrity + polymorphism
    High,
    /// All protections including timing checks
    Paranoid,
}

impl ProtectionLevel {
    pub fn to_flags(self) -> u16 {
        match self {
            ProtectionLevel::Debug => 0,
            ProtectionLevel::Low => BytecodeFlags::Encrypted as u16,
            ProtectionLevel::Medium => {
                BytecodeFlags::Encrypted as u16 | BytecodeFlags::HasIntegrity as u16
            }
            ProtectionLevel::High => {
                BytecodeFlags::Encrypted as u16
                    | BytecodeFlags::HasIntegrity as u16
            }
            ProtectionLevel::Paranoid => {
                BytecodeFlags::Encrypted as u16
                    | BytecodeFlags::HasIntegrity as u16
                    | BytecodeFlags::HasTimingChecks as u16
                    | BytecodeFlags::Paranoid as u16
            }
        }
    }
}

/// Bytecode header structure
#[derive(Clone, Debug)]
pub struct BytecodeHeader {
    /// Magic bytes (must be MAGIC)
    pub magic: [u8; 4],
    /// Format version
    pub version: u16,
    /// Protection flags
    pub flags: u16,
    /// Unique build identifier
    pub build_id: u64,
    /// Build timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// AES-GCM nonce (12 bytes)
    pub nonce: [u8; 12],
    /// AES-GCM authentication tag (16 bytes)
    pub tag: [u8; 16],
    /// Length of encrypted bytecode
    pub code_len: u32,
}

impl BytecodeHeader {
    /// Header size in bytes
    pub const SIZE: usize = 4 + 2 + 2 + 8 + 8 + 12 + 16 + 4; // 56 bytes

    /// Create a new header
    pub fn new(build_id: u64, timestamp: u64, flags: u16) -> Self {
        Self {
            magic: MAGIC,
            version: FORMAT_VERSION,
            flags,
            build_id,
            timestamp,
            nonce: [0u8; 12],
            tag: [0u8; 16],
            code_len: 0,
        }
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        let mut offset = 0;

        // Magic
        buf[offset..offset + 4].copy_from_slice(&self.magic);
        offset += 4;

        // Version
        buf[offset..offset + 2].copy_from_slice(&self.version.to_le_bytes());
        offset += 2;

        // Flags
        buf[offset..offset + 2].copy_from_slice(&self.flags.to_le_bytes());
        offset += 2;

        // Build ID
        buf[offset..offset + 8].copy_from_slice(&self.build_id.to_le_bytes());
        offset += 8;

        // Timestamp
        buf[offset..offset + 8].copy_from_slice(&self.timestamp.to_le_bytes());
        offset += 8;

        // Nonce
        buf[offset..offset + 12].copy_from_slice(&self.nonce);
        offset += 12;

        // Tag
        buf[offset..offset + 16].copy_from_slice(&self.tag);
        offset += 16;

        // Code length
        buf[offset..offset + 4].copy_from_slice(&self.code_len.to_le_bytes());

        buf
    }

    /// Parse header from bytes
    pub fn from_bytes(data: &[u8]) -> VmResult<Self> {
        if data.len() < Self::SIZE {
            return Err(VmError::InvalidBytecode);
        }

        let mut offset = 0;

        // Magic
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[offset..offset + 4]);
        if magic != MAGIC {
            return Err(VmError::InvalidBytecode);
        }
        offset += 4;

        // Version
        let version = u16::from_le_bytes([data[offset], data[offset + 1]]);
        if version > FORMAT_VERSION {
            return Err(VmError::InvalidBytecode);
        }
        offset += 2;

        // Flags
        let flags = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Build ID
        let build_id = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Timestamp
        let timestamp = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Nonce
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&data[offset..offset + 12]);
        offset += 12;

        // Tag
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&data[offset..offset + 16]);
        offset += 16;

        // Code length
        let code_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        Ok(Self {
            magic,
            version,
            flags,
            build_id,
            timestamp,
            nonce,
            tag,
            code_len,
        })
    }

    /// Check if bytecode is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.flags & BytecodeFlags::Encrypted as u16 != 0
    }

    /// Check if bytecode has integrity checks
    pub fn has_integrity(&self) -> bool {
        self.flags & BytecodeFlags::HasIntegrity as u16 != 0
    }

    /// Check if bytecode has timing checks
    pub fn has_timing_checks(&self) -> bool {
        self.flags & BytecodeFlags::HasTimingChecks as u16 != 0
    }

    /// Check if paranoid mode is enabled
    pub fn is_paranoid(&self) -> bool {
        self.flags & BytecodeFlags::Paranoid as u16 != 0
    }
}

/// Complete bytecode package (header + encrypted code)
#[derive(Clone, Debug)]
pub struct BytecodePackage {
    /// Header with metadata
    pub header: BytecodeHeader,
    /// Encrypted bytecode (or plaintext if debug mode)
    pub code: Vec<u8>,
}

impl BytecodePackage {
    /// Create a new package with plaintext code (debug mode)
    pub fn new_plaintext(code: Vec<u8>, build_id: u64) -> Self {
        let mut header = BytecodeHeader::new(build_id, 0, 0);
        header.code_len = code.len() as u32;
        Self { header, code }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BytecodeHeader::SIZE + self.code.len());
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.code);
        buf
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> VmResult<Self> {
        let header = BytecodeHeader::from_bytes(data)?;
        let code_start = BytecodeHeader::SIZE;
        let code_end = code_start + header.code_len as usize;

        if data.len() < code_end {
            return Err(VmError::InvalidBytecode);
        }

        let code = data[code_start..code_end].to_vec();
        Ok(Self { header, code })
    }
}

/// Build information for watermarking
#[derive(Clone, Debug)]
pub struct BuildInfo {
    /// Unique build identifier (derived from HMAC)
    pub build_id: u64,
    /// Build timestamp
    pub timestamp: u64,
    /// Git commit hash (first 8 bytes)
    pub git_commit: u64,
    /// Protection level used
    pub protection_level: ProtectionLevel,
}

impl BuildInfo {
    /// Create new build info
    pub fn new(build_id: u64, protection_level: ProtectionLevel) -> Self {
        Self {
            build_id,
            timestamp: 0, // Will be set at compile time
            git_commit: 0,
            protection_level,
        }
    }
}
