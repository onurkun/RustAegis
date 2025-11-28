//! Native Function Registry
//!
//! Allows VM bytecode to call native Rust functions.
//!
//! # Example
//!
//! ```rust
//! use aegis_vm::native::{NativeRegistry, NativeFunction};
//!
//! let mut registry = NativeRegistry::new();
//!
//! // Register a simple function
//! registry.register(0, |_args| 42);
//!
//! // Register a function that uses arguments
//! registry.register(1, |args| {
//!     if args.is_empty() { return 0; }
//!     args[0] * 2
//! });
//!
//! // Call from VM
//! assert_eq!(registry.call(0, &[]).unwrap(), 42);
//! assert_eq!(registry.call(1, &[21]).unwrap(), 42);
//! ```

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

use crate::error::{VmError, VmResult};

/// Maximum number of native functions that can be registered
pub const MAX_NATIVE_FUNCTIONS: usize = 256;

/// Maximum number of arguments a native function can receive
pub const MAX_NATIVE_ARGS: usize = 8;

/// Native function signature
/// Takes a slice of u64 arguments, returns a u64 result
pub type NativeFunction = Box<dyn Fn(&[u64]) -> u64 + Send + Sync>;

/// Native function registry
///
/// Stores registered native functions that can be called from VM bytecode.
/// Functions are identified by a u8 ID (0-255).
pub struct NativeRegistry {
    /// Registered functions (None = not registered)
    functions: Vec<Option<NativeFunction>>,
}

impl Default for NativeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        let mut functions = Vec::with_capacity(MAX_NATIVE_FUNCTIONS);
        for _ in 0..MAX_NATIVE_FUNCTIONS {
            functions.push(None);
        }
        Self { functions }
    }

    /// Register a native function with the given ID
    ///
    /// # Arguments
    /// * `id` - Function ID (0-255)
    /// * `func` - The function to register
    ///
    /// # Returns
    /// * `Ok(())` if registered successfully
    /// * `Err` if ID is already registered
    pub fn register<F>(&mut self, id: u8, func: F) -> VmResult<()>
    where
        F: Fn(&[u64]) -> u64 + Send + Sync + 'static,
    {
        let idx = id as usize;
        if self.functions[idx].is_some() {
            return Err(VmError::NativeFunctionAlreadyRegistered);
        }
        self.functions[idx] = Some(Box::new(func));
        Ok(())
    }

    /// Register a native function, replacing any existing one
    pub fn register_replace<F>(&mut self, id: u8, func: F)
    where
        F: Fn(&[u64]) -> u64 + Send + Sync + 'static,
    {
        let idx = id as usize;
        self.functions[idx] = Some(Box::new(func));
    }

    /// Unregister a native function
    pub fn unregister(&mut self, id: u8) {
        let idx = id as usize;
        self.functions[idx] = None;
    }

    /// Call a native function by ID
    ///
    /// # Arguments
    /// * `id` - Function ID
    /// * `args` - Arguments to pass to the function
    ///
    /// # Returns
    /// * `Ok(result)` - The function's return value
    /// * `Err(NativeFunctionNotFound)` - If no function is registered with this ID
    pub fn call(&self, id: u8, args: &[u64]) -> VmResult<u64> {
        let idx = id as usize;
        match &self.functions[idx] {
            Some(func) => Ok(func(args)),
            None => Err(VmError::NativeFunctionNotFound),
        }
    }

    /// Check if a function is registered
    pub fn is_registered(&self, id: u8) -> bool {
        self.functions[id as usize].is_some()
    }

    /// Get the number of registered functions
    pub fn count(&self) -> usize {
        self.functions.iter().filter(|f| f.is_some()).count()
    }

    /// Clear all registered functions
    pub fn clear(&mut self) {
        for func in &mut self.functions {
            *func = None;
        }
    }
}

/// Standard native function IDs
///
/// These are predefined IDs for common anticheat operations.
/// Custom functions should use IDs >= 128.
/// Note: These IDs are shuffled per-build for anti-analysis
pub mod standard_ids {
    pub use crate::build_config::native_ids::*;
}

/// Builder pattern for creating a NativeRegistry with common functions
pub struct NativeRegistryBuilder {
    registry: NativeRegistry,
}

impl Default for NativeRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: NativeRegistry::new(),
        }
    }

    /// Register a function
    pub fn with_function<F>(mut self, id: u8, func: F) -> Self
    where
        F: Fn(&[u64]) -> u64 + Send + Sync + 'static,
    {
        let _ = self.registry.register(id, func);
        self
    }

    /// Add timestamp function
    pub fn with_timestamp(self) -> Self {
        self.with_function(standard_ids::GET_TIMESTAMP, |_| {
            // In production, this would use actual timestamp
            // For now, return a placeholder
            #[cfg(feature = "std")]
            {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0)
            }
            #[cfg(not(feature = "std"))]
            {
                0
            }
        })
    }

    /// Add FNV-1a hash function (randomized constants per build)
    pub fn with_hash(self) -> Self {
        self.with_function(standard_ids::HASH_FNV1A, |args| {
            // Hash all arguments together
            let mut hash = crate::build_config::FNV_BASIS_64;
            for &arg in args {
                for byte in arg.to_le_bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(crate::build_config::FNV_PRIME_64);
                }
            }
            hash
        })
    }

    /// Build the registry
    pub fn build(self) -> NativeRegistry {
        self.registry
    }
}
