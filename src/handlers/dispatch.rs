//! Indirect Threading Dispatch Table
//!
//! This module provides the function pointer table for opcode dispatch.
//! Instead of a switch-case pattern, opcodes are dispatched via:
//!   HANDLERS[opcode](state, registry)
//!
//! Benefits:
//! - Eliminates recognizable switch-case pattern in binary
//! - Table order is shuffled per-build (via OPCODE_ENCODE)
//! - Makes static analysis significantly harder

use crate::error::{VmError, VmResult};
use crate::native::NativeRegistry;
use crate::state::VmState;

/// Unified handler function type
/// All handlers take (state, registry) even if they don't use registry
pub type Handler = fn(&mut VmState, &NativeRegistry) -> VmResult<()>;

// ============================================================================
// Wrapper functions for handlers that don't need registry
// ============================================================================
// These convert `fn(state) -> Result` to `fn(state, _registry) -> Result`
// The compiler will inline these, so there's no performance penalty

// Stack handlers
#[inline(always)]
pub fn w_push_imm(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_push_imm(s)
}
#[inline(always)]
pub fn w_push_imm8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_push_imm8(s)
}
#[inline(always)]
pub fn w_push_imm16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_push_imm16(s)
}
#[inline(always)]
pub fn w_push_imm32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_push_imm32(s)
}
#[inline(always)]
pub fn w_push_reg(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_push_reg(s)
}
#[inline(always)]
pub fn w_pop_reg(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_pop_reg(s)
}
#[inline(always)]
pub fn w_dup(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_dup(s)
}
#[inline(always)]
pub fn w_swap(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_swap(s)
}
#[inline(always)]
pub fn w_drop(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_drop(s)
}

// Register handlers
#[inline(always)]
pub fn w_mov_imm(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_mov_imm(s)
}
#[inline(always)]
pub fn w_mov_reg(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_mov_reg(s)
}
#[inline(always)]
pub fn w_load_mem(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_load_mem(s)
}
#[inline(always)]
pub fn w_store_mem(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_store_mem(s)
}

// Arithmetic handlers
#[inline(always)]
pub fn w_add(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_add(s)
}
#[inline(always)]
pub fn w_sub(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_sub(s)
}
#[inline(always)]
pub fn w_mul(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_mul(s)
}
#[inline(always)]
pub fn w_xor(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_xor(s)
}
#[inline(always)]
pub fn w_and(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_and(s)
}
#[inline(always)]
pub fn w_or(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_or(s)
}
#[inline(always)]
pub fn w_shl(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_shl(s)
}
#[inline(always)]
pub fn w_shr(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_shr(s)
}
#[inline(always)]
pub fn w_not(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_not(s)
}
#[inline(always)]
pub fn w_rol(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_rol(s)
}
#[inline(always)]
pub fn w_ror(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_ror(s)
}
#[inline(always)]
pub fn w_inc(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_inc(s)
}
#[inline(always)]
pub fn w_dec(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_dec(s)
}
#[inline(always)]
pub fn w_div(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_div(s)
}
#[inline(always)]
pub fn w_mod(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_mod(s)
}
#[inline(always)]
pub fn w_idiv(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_idiv(s)
}
#[inline(always)]
pub fn w_imod(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_imod(s)
}

// Control handlers
#[inline(always)]
pub fn w_cmp(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_cmp(s)
}
#[inline(always)]
pub fn w_jmp(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jmp(s)
}
#[inline(always)]
pub fn w_jz(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jz(s)
}
#[inline(always)]
pub fn w_jnz(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jnz(s)
}
#[inline(always)]
pub fn w_jgt(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jgt(s)
}
#[inline(always)]
pub fn w_jlt(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jlt(s)
}
#[inline(always)]
pub fn w_jge(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jge(s)
}
#[inline(always)]
pub fn w_jle(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_jle(s)
}
#[inline(always)]
pub fn w_call(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_call(s)
}
#[inline(always)]
pub fn w_ret(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_ret(s)
}

// Special handlers
#[inline(always)]
pub fn w_nop(_: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    Ok(())
}
#[inline(always)]
pub fn w_nop_n(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_nop_n(s)
}
#[inline(always)]
pub fn w_opaque_true(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_opaque_true(s)
}
#[inline(always)]
pub fn w_opaque_false(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_opaque_false(s)
}
#[inline(always)]
pub fn w_hash_check(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_hash_check(s)
}
#[inline(always)]
pub fn w_timing_check(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_timing_check(s)
}

// Convert handlers
#[inline(always)]
pub fn w_sext8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_sext8(s)
}
#[inline(always)]
pub fn w_sext16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_sext16(s)
}
#[inline(always)]
pub fn w_sext32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_sext32(s)
}
#[inline(always)]
pub fn w_trunc8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_trunc8(s)
}
#[inline(always)]
pub fn w_trunc16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_trunc16(s)
}
#[inline(always)]
pub fn w_trunc32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_trunc32(s)
}

// Memory handlers
#[inline(always)]
pub fn w_load8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_load8(s)
}
#[inline(always)]
pub fn w_load16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_load16(s)
}
#[inline(always)]
pub fn w_load32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_load32(s)
}
#[inline(always)]
pub fn w_load64(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_load64(s)
}
#[inline(always)]
pub fn w_store8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_store8(s)
}
#[inline(always)]
pub fn w_store16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_store16(s)
}
#[inline(always)]
pub fn w_store32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_store32(s)
}
#[inline(always)]
pub fn w_store64(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_store64(s)
}

// Heap handlers
#[inline(always)]
pub fn w_heap_alloc(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_alloc(s)
}
#[inline(always)]
pub fn w_heap_free(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_free(s)
}
#[inline(always)]
pub fn w_heap_load8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_load8(s)
}
#[inline(always)]
pub fn w_heap_load16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_load16(s)
}
#[inline(always)]
pub fn w_heap_load32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_load32(s)
}
#[inline(always)]
pub fn w_heap_load64(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_load64(s)
}
#[inline(always)]
pub fn w_heap_store8(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_store8(s)
}
#[inline(always)]
pub fn w_heap_store16(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_store16(s)
}
#[inline(always)]
pub fn w_heap_store32(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_store32(s)
}
#[inline(always)]
pub fn w_heap_store64(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_store64(s)
}
#[inline(always)]
pub fn w_heap_size(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_heap_size(s)
}

// Vector handlers
#[inline(always)]
pub fn w_vec_new(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_new(s)
}
#[inline(always)]
pub fn w_vec_len(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_len(s)
}
#[inline(always)]
pub fn w_vec_cap(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_cap(s)
}
#[inline(always)]
pub fn w_vec_push(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_push(s)
}
#[inline(always)]
pub fn w_vec_pop(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_pop(s)
}
#[inline(always)]
pub fn w_vec_get(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_get(s)
}
#[inline(always)]
pub fn w_vec_set(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_set(s)
}
#[inline(always)]
pub fn w_vec_repeat(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_repeat(s)
}
#[inline(always)]
pub fn w_vec_clear(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_clear(s)
}
#[inline(always)]
pub fn w_vec_reserve(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_vec_reserve(s)
}

// String handlers
#[inline(always)]
pub fn w_str_new(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_new(s)
}
#[inline(always)]
pub fn w_str_len(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_len(s)
}
#[inline(always)]
pub fn w_str_push(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_push(s)
}
#[inline(always)]
pub fn w_str_get(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_get(s)
}
#[inline(always)]
pub fn w_str_set(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_set(s)
}
#[inline(always)]
pub fn w_str_cmp(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_cmp(s)
}
#[inline(always)]
pub fn w_str_eq(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_eq(s)
}
#[inline(always)]
pub fn w_str_hash(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_hash(s)
}
#[inline(always)]
pub fn w_str_concat(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_str_concat(s)
}

// Native handlers (handle_native_call already takes registry)
#[inline(always)]
pub fn w_native_read(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_native_read(s)
}
#[inline(always)]
pub fn w_native_write(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_native_write(s)
}
#[inline(always)]
pub fn w_input_len(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_input_len(s)
}

// Exec handlers
#[inline(always)]
pub fn w_halt(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_halt(s)
}
#[inline(always)]
pub fn w_halt_err(s: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    super::handle_halt_err(s)
}

/// Handler for invalid/unknown opcodes
#[inline(always)]
pub fn w_invalid(_: &mut VmState, _: &NativeRegistry) -> VmResult<()> {
    Err(VmError::InvalidOpcode)
}

// ============================================================================
// Handler table indexed by BASE opcode (after decoding via OPCODE_DECODE)
// ============================================================================

/// Handler table indexed by BASE opcode
/// Indexed as: HANDLER_TABLE[OPCODE_DECODE[shuffled_opcode]]
/// Note: Using `const` instead of `static` for no_std/WASM compatibility
pub const HANDLER_TABLE: [Handler; 256] = {
    let mut table: [Handler; 256] = [w_invalid; 256];

    // Stack (0x01-0x09)
    table[0x01] = w_push_imm;
    table[0x02] = w_push_imm8;
    table[0x03] = w_push_reg;
    table[0x04] = w_pop_reg;
    table[0x05] = w_dup;
    table[0x06] = w_swap;
    table[0x07] = w_drop;
    table[0x08] = w_push_imm16;
    table[0x09] = w_push_imm32;

    // Register (0x10-0x13)
    table[0x10] = w_mov_imm;
    table[0x11] = w_mov_reg;
    table[0x12] = w_load_mem;
    table[0x13] = w_store_mem;

    // Arithmetic (0x20-0x2C, 0x46-0x49)
    table[0x20] = w_add;
    table[0x21] = w_sub;
    table[0x22] = w_mul;
    table[0x23] = w_xor;
    table[0x24] = w_and;
    table[0x25] = w_or;
    table[0x26] = w_shl;
    table[0x27] = w_shr;
    table[0x28] = w_not;
    table[0x29] = w_rol;
    table[0x2A] = w_ror;
    table[0x2B] = w_inc;
    table[0x2C] = w_dec;
    table[0x46] = w_div;
    table[0x47] = w_mod;
    table[0x48] = w_idiv;
    table[0x49] = w_imod;

    // Control (0x30-0x39)
    table[0x30] = w_cmp;
    table[0x31] = w_jmp;
    table[0x32] = w_jz;
    table[0x33] = w_jnz;
    table[0x34] = w_jgt;
    table[0x35] = w_jlt;
    table[0x36] = w_jge;
    table[0x37] = w_jle;
    table[0x38] = w_call;
    table[0x39] = w_ret;

    // Special (0x40-0x45)
    table[0x40] = w_nop;
    table[0x41] = w_nop_n;
    table[0x42] = w_opaque_true;
    table[0x43] = w_opaque_false;
    table[0x44] = w_hash_check;
    table[0x45] = w_timing_check;

    // Convert (0x50-0x55)
    table[0x50] = w_sext8;
    table[0x51] = w_sext16;
    table[0x52] = w_sext32;
    table[0x53] = w_trunc8;
    table[0x54] = w_trunc16;
    table[0x55] = w_trunc32;

    // Memory (0x60-0x67)
    table[0x60] = w_load8;
    table[0x61] = w_load16;
    table[0x62] = w_load32;
    table[0x63] = w_load64;
    table[0x64] = w_store8;
    table[0x65] = w_store16;
    table[0x66] = w_store32;
    table[0x67] = w_store64;

    // Heap (0x70-0x7A)
    table[0x70] = w_heap_alloc;
    table[0x71] = w_heap_free;
    table[0x72] = w_heap_load8;
    table[0x73] = w_heap_load16;
    table[0x74] = w_heap_load32;
    table[0x75] = w_heap_load64;
    table[0x76] = w_heap_store8;
    table[0x77] = w_heap_store16;
    table[0x78] = w_heap_store32;
    table[0x79] = w_heap_store64;
    table[0x7A] = w_heap_size;

    // Vector (0x80-0x89)
    table[0x80] = w_vec_new;
    table[0x81] = w_vec_len;
    table[0x82] = w_vec_cap;
    table[0x83] = w_vec_push;
    table[0x84] = w_vec_pop;
    table[0x85] = w_vec_get;
    table[0x86] = w_vec_set;
    table[0x87] = w_vec_repeat;
    table[0x88] = w_vec_clear;
    table[0x89] = w_vec_reserve;

    // String (0x90-0x98)
    table[0x90] = w_str_new;
    table[0x91] = w_str_len;
    table[0x92] = w_str_push;
    table[0x93] = w_str_get;
    table[0x94] = w_str_set;
    table[0x95] = w_str_cmp;
    table[0x96] = w_str_eq;
    table[0x97] = w_str_hash;
    table[0x98] = w_str_concat;

    // Native (0xF0-0xF3)
    table[0xF0] = super::handle_native_call; // Already takes registry
    table[0xF1] = w_native_read;
    table[0xF2] = w_native_write;
    table[0xF3] = w_input_len;

    // Exec (0xFE-0xFF)
    table[0xFE] = w_halt_err;
    table[0xFF] = w_halt;

    table
};

/// Dispatch an opcode using the indirect handler table
///
/// This replaces the switch-case pattern in binary:
/// ```text
/// // Old (visible pattern):
/// cmp x8, #0x01
/// b.eq handler_push_imm
/// cmp x8, #0x02
/// b.eq handler_push_imm8
/// ...
///
/// // New (indirect call):
/// ldr x9, [HANDLER_TABLE, x8, lsl #3]
/// blr x9
/// ```
#[inline(always)]
pub fn dispatch_indirect(
    state: &mut VmState,
    opcode: u8,
    registry: &NativeRegistry
) -> VmResult<()> {
    use crate::build_config::OPCODE_DECODE;

    // Decode shuffled opcode to base opcode
    let base_opcode = OPCODE_DECODE[opcode as usize];

    // Call handler via function pointer (no switch-case pattern)
    HANDLER_TABLE[base_opcode as usize](state, registry)
}
