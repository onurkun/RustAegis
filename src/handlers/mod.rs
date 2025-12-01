//! VM Opcode Handlers
//!
//! Organized by expression/operation category following Rust Reference 8.2

pub mod stack;
pub mod register;
pub mod arithmetic;
pub mod control;
pub mod special;
pub mod convert;
pub mod memory;
pub mod heap;
pub mod native;
pub mod exec;
pub mod vector;
pub mod string;

// Re-export all handlers for convenience
pub use stack::*;
pub use register::*;
pub use arithmetic::*;
pub use control::*;
pub use special::*;
pub use convert::*;
pub use memory::*;
pub use heap::*;
pub use native::*;
pub use exec::*;
pub use vector::*;
pub use string::*;
