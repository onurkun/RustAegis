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
pub mod mutation;
pub mod dispatch;

// Re-export all handlers for convenience
pub use stack::*;
pub use register::*;
pub use control::*;
pub use special::*;
pub use convert::*;
pub use memory::*;
pub use heap::*;
pub use native::*;
pub use exec::*;
pub use vector::*;
pub use string::*;

// =============================================================================
// Arithmetic Handlers - Conditionally use mutated versions
// =============================================================================

// Non-mutated arithmetic handlers (always from arithmetic module)
pub use arithmetic::{
    handle_shl, handle_shr, handle_rol, handle_ror,
    handle_div, handle_mod, handle_idiv, handle_imod,
};

// Mutated arithmetic handlers - use build-time generated versions
#[cfg(feature = "handler_mutation")]
pub use mutation::{
    mutated_add as handle_add,
    mutated_sub as handle_sub,
    mutated_mul as handle_mul,
    mutated_xor as handle_xor,
    mutated_and as handle_and,
    mutated_or as handle_or,
    mutated_not as handle_not,
    mutated_inc as handle_inc,
    mutated_dec as handle_dec,
};

// Original arithmetic handlers (when mutation disabled)
#[cfg(not(feature = "handler_mutation"))]
pub use arithmetic::{
    handle_add, handle_sub, handle_mul,
    handle_xor, handle_and, handle_or,
    handle_not, handle_inc, handle_dec,
};
