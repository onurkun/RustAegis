//! String Operations for the VM
//!
//! Strings are implemented as Vec<u8> with elem_size=1.
//! Uses the same heap layout as vectors:
//! ```text
//! Offset 0:  capacity  (u64) - maximum bytes
//! Offset 8:  length    (u64) - current byte count
//! Offset 16: elem_size (u64) - always 1 for strings
//! Offset 24: data[...] - UTF-8 byte data
//! ```

use crate::error::{VmError, VmResult};
use crate::state::VmState;
use super::vector::VEC_HEADER_SIZE;

/// Offset of capacity field
const OFFSET_CAPACITY: usize = 0;
/// Offset of length field
const OFFSET_LENGTH: usize = 8;
/// Offset of elem_size field
const OFFSET_ELEM_SIZE: usize = 16;
/// Offset where string data begins
const OFFSET_DATA: usize = 24;

// ============================================================================
// Helper functions for string operations
// ============================================================================

/// Read string length from heap
#[inline]
fn str_get_length(state: &VmState, str_addr: usize) -> VmResult<u64> {
    state.heap_read_u64(str_addr + OFFSET_LENGTH)
}

/// Read string capacity from heap
#[inline]
fn str_get_capacity(state: &VmState, str_addr: usize) -> VmResult<u64> {
    state.heap_read_u64(str_addr + OFFSET_CAPACITY)
}

/// Write string length to heap
#[inline]
fn str_set_length(state: &mut VmState, str_addr: usize, length: u64) -> VmResult<()> {
    state.heap_write_u64(str_addr + OFFSET_LENGTH, length)
}

/// Read byte at index
#[inline]
fn str_read_byte(state: &VmState, str_addr: usize, index: u64) -> VmResult<u8> {
    state.heap_read_u8(str_addr + OFFSET_DATA + index as usize)
}

/// Write byte at index
#[inline]
fn str_write_byte(state: &mut VmState, str_addr: usize, index: u64, value: u8) -> VmResult<()> {
    state.heap_write_u8(str_addr + OFFSET_DATA + index as usize, value)
}

// ============================================================================
// String Operation Handlers
// ============================================================================

/// STR_NEW: Create new string with capacity
/// Stack: [capacity] -> [str_addr]
pub fn handle_str_new(state: &mut VmState) -> VmResult<()> {
    let capacity = state.pop()?;

    // Calculate total size: header + capacity bytes
    let total_size = VEC_HEADER_SIZE as u64 + capacity;

    // Allocate on heap
    let str_addr = state.heap_alloc(total_size as usize)? as usize;

    // Initialize header (elem_size = 1 for strings)
    state.heap_write_u64(str_addr + OFFSET_CAPACITY, capacity)?;
    state.heap_write_u64(str_addr + OFFSET_LENGTH, 0)?;
    state.heap_write_u64(str_addr + OFFSET_ELEM_SIZE, 1)?;

    state.push(str_addr as u64)
}

/// STR_LEN: Get string length (byte count)
/// Stack: [str_addr] -> [length]
pub fn handle_str_len(state: &mut VmState) -> VmResult<()> {
    let str_addr = state.pop()? as usize;
    let length = str_get_length(state, str_addr)?;
    state.push(length)
}

/// STR_PUSH: Push byte to string
/// Stack: [str_addr, byte] -> []
pub fn handle_str_push(state: &mut VmState) -> VmResult<()> {
    let byte = state.pop()? as u8;
    let str_addr = state.pop()? as usize;

    let length = str_get_length(state, str_addr)?;
    let capacity = str_get_capacity(state, str_addr)?;

    // Check capacity
    if length >= capacity {
        return Err(VmError::HeapOutOfMemory);
    }

    // Write byte and increment length
    str_write_byte(state, str_addr, length, byte)?;
    str_set_length(state, str_addr, length + 1)
}

/// STR_GET: Get byte at index
/// Stack: [str_addr, index] -> [byte]
pub fn handle_str_get(state: &mut VmState) -> VmResult<()> {
    let index = state.pop()?;
    let str_addr = state.pop()? as usize;

    let length = str_get_length(state, str_addr)?;

    // Bounds check
    if index >= length {
        return Err(VmError::HeapOutOfBounds);
    }

    let byte = str_read_byte(state, str_addr, index)?;
    state.push(byte as u64)
}

/// STR_SET: Set byte at index
/// Stack: [str_addr, index, byte] -> []
pub fn handle_str_set(state: &mut VmState) -> VmResult<()> {
    let byte = state.pop()? as u8;
    let index = state.pop()?;
    let str_addr = state.pop()? as usize;

    let length = str_get_length(state, str_addr)?;

    // Bounds check
    if index >= length {
        return Err(VmError::HeapOutOfBounds);
    }

    str_write_byte(state, str_addr, index, byte)
}

/// STR_CMP: Compare two strings lexicographically
/// Stack: [str1_addr, str2_addr] -> [result]
/// Returns: 0 if equal, -1 (as u64::MAX) if str1 < str2, 1 if str1 > str2
pub fn handle_str_cmp(state: &mut VmState) -> VmResult<()> {
    let str2_addr = state.pop()? as usize;
    let str1_addr = state.pop()? as usize;

    let len1 = str_get_length(state, str1_addr)?;
    let len2 = str_get_length(state, str2_addr)?;

    let min_len = len1.min(len2);

    // Compare byte by byte
    for i in 0..min_len {
        let b1 = str_read_byte(state, str1_addr, i)?;
        let b2 = str_read_byte(state, str2_addr, i)?;

        if b1 < b2 {
            return state.push(u64::MAX); // -1 as unsigned
        } else if b1 > b2 {
            return state.push(1);
        }
    }

    // All compared bytes equal, compare lengths
    let result = if len1 < len2 {
        u64::MAX // -1
    } else if len1 > len2 {
        1
    } else {
        0 // Equal
    };

    state.push(result)
}

/// STR_EQ: Check string equality
/// Stack: [str1_addr, str2_addr] -> [0/1]
pub fn handle_str_eq(state: &mut VmState) -> VmResult<()> {
    let str2_addr = state.pop()? as usize;
    let str1_addr = state.pop()? as usize;

    let len1 = str_get_length(state, str1_addr)?;
    let len2 = str_get_length(state, str2_addr)?;

    // Quick check: different lengths mean not equal
    if len1 != len2 {
        return state.push(0);
    }

    // Compare byte by byte
    for i in 0..len1 {
        let b1 = str_read_byte(state, str1_addr, i)?;
        let b2 = str_read_byte(state, str2_addr, i)?;

        if b1 != b2 {
            return state.push(0);
        }
    }

    state.push(1) // Equal
}

/// STR_HASH: Hash string using FNV-1a
/// Stack: [str_addr] -> [hash]
pub fn handle_str_hash(state: &mut VmState) -> VmResult<()> {
    let str_addr = state.pop()? as usize;
    let length = str_get_length(state, str_addr)?;

    // FNV-1a 64-bit hash (using build-time randomized constants)
    let mut hash = crate::build_config::FNV_BASIS_64;
    let prime = crate::build_config::FNV_PRIME_64;

    for i in 0..length {
        let byte = str_read_byte(state, str_addr, i)?;
        hash ^= byte as u64;
        hash = hash.wrapping_mul(prime);
    }

    state.push(hash)
}

/// STR_CONCAT: Concatenate two strings into new string
/// Stack: [str1_addr, str2_addr] -> [new_str_addr]
pub fn handle_str_concat(state: &mut VmState) -> VmResult<()> {
    let str2_addr = state.pop()? as usize;
    let str1_addr = state.pop()? as usize;

    let len1 = str_get_length(state, str1_addr)?;
    let len2 = str_get_length(state, str2_addr)?;

    let new_len = len1.checked_add(len2)
        .ok_or(VmError::HeapOutOfMemory)?;

    // Allocate new string
    let total_size = VEC_HEADER_SIZE as u64 + new_len;
    let new_addr = state.heap_alloc(total_size as usize)? as usize;

    // Initialize header
    state.heap_write_u64(new_addr + OFFSET_CAPACITY, new_len)?;
    state.heap_write_u64(new_addr + OFFSET_LENGTH, new_len)?;
    state.heap_write_u64(new_addr + OFFSET_ELEM_SIZE, 1)?;

    // Copy str1
    for i in 0..len1 {
        let byte = str_read_byte(state, str1_addr, i)?;
        str_write_byte(state, new_addr, i, byte)?;
    }

    // Copy str2
    for i in 0..len2 {
        let byte = str_read_byte(state, str2_addr, i)?;
        str_write_byte(state, new_addr, len1 + i, byte)?;
    }

    state.push(new_addr as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_constants() {
        assert_eq!(VEC_HEADER_SIZE, 24);
        assert_eq!(OFFSET_DATA, 24);
    }
}
