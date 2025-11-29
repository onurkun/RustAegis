//! Region-based Integrity Checking
//!
//! Verifies bytecode hasn't been tampered with at runtime.
//! Bytecode is divided into regions, each with a precomputed hash.
//!
//! ## How it works
//!
//! ```text
//! Compile-time:
//! ┌────────┬────────┬────────┬────────┐
//! │Region 0│Region 1│Region 2│Region 3│  ← Bytecode
//! └────────┴────────┴────────┴────────┘
//!     ↓         ↓         ↓         ↓
//!   Hash 0   Hash 1   Hash 2   Hash 3    ← Computed & embedded
//!
//! Runtime:
//! 1. Recompute hashes for each region
//! 2. Compare with embedded hashes
//! 3. Mismatch → Tampering detected!
//! ```

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::build_config::{FNV_BASIS_64, FNV_PRIME_64};

/// Default region size (64 bytes)
/// Smaller = more granular detection, larger = less overhead
pub const DEFAULT_REGION_SIZE: usize = 64;

/// Maximum number of regions to check
pub const MAX_REGIONS: usize = 256;

/// Region integrity information
#[derive(Debug, Clone, Copy)]
pub struct RegionInfo {
    /// Start offset in bytecode
    pub start: u32,
    /// End offset in bytecode (exclusive)
    pub end: u32,
    /// Precomputed FNV-1a hash of the region
    pub hash: u64,
}

/// Integrity table embedded in protected functions
#[derive(Debug, Clone)]
pub struct IntegrityTable {
    /// Region information
    pub regions: Vec<RegionInfo>,
    /// Overall bytecode hash (for quick full check)
    pub full_hash: u64,
    /// Region size used
    pub region_size: usize,
}

impl IntegrityTable {
    /// Create integrity table for bytecode
    pub fn new(bytecode: &[u8], region_size: usize) -> Self {
        let mut regions = Vec::new();
        let mut offset = 0;

        while offset < bytecode.len() && regions.len() < MAX_REGIONS {
            let end = (offset + region_size).min(bytecode.len());
            let region_data = &bytecode[offset..end];
            let hash = fnv1a_hash(region_data);

            regions.push(RegionInfo {
                start: offset as u32,
                end: end as u32,
                hash,
            });

            offset = end;
        }

        let full_hash = fnv1a_hash(bytecode);

        IntegrityTable {
            regions,
            full_hash,
            region_size,
        }
    }

    /// Verify bytecode integrity
    /// Returns Ok(()) if valid, Err with details if tampered
    pub fn verify(&self, bytecode: &[u8]) -> Result<(), IntegrityError> {
        // Quick full hash check first
        let computed_full = fnv1a_hash(bytecode);
        if computed_full != self.full_hash {
            // Full hash mismatch - find which region(s) were tampered
            for (idx, region) in self.regions.iter().enumerate() {
                let start = region.start as usize;
                let end = region.end as usize;

                if end > bytecode.len() {
                    return Err(IntegrityError::SizeMismatch {
                        expected: end,
                        actual: bytecode.len(),
                    });
                }

                let region_data = &bytecode[start..end];
                let computed_hash = fnv1a_hash(region_data);

                if computed_hash != region.hash {
                    return Err(IntegrityError::RegionTampered {
                        region_index: idx,
                        start,
                        end,
                        expected_hash: region.hash,
                        actual_hash: computed_hash,
                    });
                }
            }

            // Regions all match but full doesn't? Shouldn't happen, but report
            return Err(IntegrityError::HashMismatch {
                expected: self.full_hash,
                actual: computed_full,
            });
        }

        Ok(())
    }

    /// Quick verify - only checks full hash (faster)
    #[inline]
    pub fn verify_quick(&self, bytecode: &[u8]) -> bool {
        fnv1a_hash(bytecode) == self.full_hash
    }

    /// Verify specific region only (for incremental checking)
    #[inline]
    pub fn verify_region(&self, bytecode: &[u8], region_index: usize) -> bool {
        if region_index >= self.regions.len() {
            return false;
        }

        let region = &self.regions[region_index];
        let start = region.start as usize;
        let end = region.end as usize;

        if end > bytecode.len() {
            return false;
        }

        let region_data = &bytecode[start..end];
        fnv1a_hash(region_data) == region.hash
    }
}

/// Integrity check error
#[derive(Debug, Clone)]
pub enum IntegrityError {
    /// Full bytecode hash mismatch
    HashMismatch {
        expected: u64,
        actual: u64,
    },
    /// Specific region was tampered
    RegionTampered {
        region_index: usize,
        start: usize,
        end: usize,
        expected_hash: u64,
        actual_hash: u64,
    },
    /// Bytecode size doesn't match expected
    SizeMismatch {
        expected: usize,
        actual: usize,
    },
}

impl core::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IntegrityError::HashMismatch { expected, actual } => {
                write!(f, "Bytecode integrity check failed: hash mismatch (expected 0x{:016x}, got 0x{:016x})", expected, actual)
            }
            IntegrityError::RegionTampered { region_index, start, end, .. } => {
                write!(f, "Bytecode tampering detected in region {} (bytes {}..{})", region_index, start, end)
            }
            IntegrityError::SizeMismatch { expected, actual } => {
                write!(f, "Bytecode size mismatch (expected {}, got {})", expected, actual)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IntegrityError {}

/// FNV-1a hash using build-specific constants
/// This makes the hash function polymorphic per build
#[inline]
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash = FNV_BASIS_64;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    hash
}

/// Compute integrity hash for bytecode (convenience function)
#[inline]
pub fn compute_hash(bytecode: &[u8]) -> u64 {
    fnv1a_hash(bytecode)
}

/// Verify bytecode against expected hash (simple check)
#[inline]
pub fn verify_hash(bytecode: &[u8], expected: u64) -> bool {
    fnv1a_hash(bytecode) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a_hash_deterministic() {
        let data = b"hello world";
        let hash1 = fnv1a_hash(data);
        let hash2 = fnv1a_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fnv1a_hash_different_inputs() {
        let hash1 = fnv1a_hash(b"hello");
        let hash2 = fnv1a_hash(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_integrity_table_creation() {
        let bytecode = vec![0u8; 256];
        let table = IntegrityTable::new(&bytecode, 64);

        assert_eq!(table.regions.len(), 4); // 256 / 64 = 4 regions
        assert_eq!(table.region_size, 64);
    }

    #[test]
    fn test_integrity_verify_valid() {
        let bytecode = vec![0x42u8; 128];
        let table = IntegrityTable::new(&bytecode, 32);

        assert!(table.verify(&bytecode).is_ok());
        assert!(table.verify_quick(&bytecode));
    }

    #[test]
    fn test_integrity_verify_tampered() {
        let bytecode = vec![0x42u8; 128];
        let table = IntegrityTable::new(&bytecode, 32);

        // Tamper with bytecode
        let mut tampered = bytecode.clone();
        tampered[64] = 0xFF; // Modify byte in region 2

        assert!(table.verify(&tampered).is_err());
        assert!(!table.verify_quick(&tampered));

        // Check specific region
        assert!(table.verify_region(&tampered, 0)); // Region 0 OK
        assert!(table.verify_region(&tampered, 1)); // Region 1 OK
        assert!(!table.verify_region(&tampered, 2)); // Region 2 tampered
        assert!(table.verify_region(&tampered, 3)); // Region 3 OK
    }

    #[test]
    fn test_integrity_error_details() {
        let bytecode = vec![0x42u8; 128];
        let table = IntegrityTable::new(&bytecode, 32);

        let mut tampered = bytecode.clone();
        tampered[70] = 0xFF;

        let err = table.verify(&tampered).unwrap_err();
        match err {
            IntegrityError::RegionTampered { region_index, start, end, .. } => {
                assert_eq!(region_index, 2);
                assert_eq!(start, 64);
                assert_eq!(end, 96);
            }
            _ => panic!("Expected RegionTampered error"),
        }
    }

    #[test]
    fn test_small_bytecode() {
        let bytecode = vec![0x42u8; 10]; // Smaller than region size
        let table = IntegrityTable::new(&bytecode, 64);

        assert_eq!(table.regions.len(), 1);
        assert!(table.verify(&bytecode).is_ok());
    }

    #[test]
    fn test_empty_bytecode() {
        let bytecode: Vec<u8> = vec![];
        let table = IntegrityTable::new(&bytecode, 64);

        assert_eq!(table.regions.len(), 0);
        assert!(table.verify(&bytecode).is_ok());
    }
}
