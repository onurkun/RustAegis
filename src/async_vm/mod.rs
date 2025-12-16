//! Async VM Engine (Experimental)
//!
//! This module provides an async/await based VM execution engine for
//! anti-analysis purposes. The Rust compiler transforms async functions
//! into complex state machines, making control flow analysis harder.
//!
//! # Features
//! - Custom micro-executor (no external dependencies)
//! - Configurable yield frequency via `yield_mask`
//! - `no_std` compatible
//! - Battery-friendly (no busy spin)
//!
//! # Usage
//! ```ignore
//! use aegis_vm::async_vm::execute_async;
//!
//! let result = execute_async(&bytecode, &input)?;
//! ```
//!
//! # Security Note
//! This adds obfuscation, not cryptographic security. A skilled reverse
//! engineer can still analyze the state machine given enough time.

mod executor;
mod yielder;
mod engine;

pub use executor::block_on;
pub use yielder::YieldNow;
pub use engine::{
    execute_async,
    execute_async_with_natives,
    execute_async_with_native_table,
    run_async,
    run_async_with_native_table,
};
