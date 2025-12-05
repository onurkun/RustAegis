//! Vector Operations for the VM
//!
//! Implements dynamic array support with the following layout in heap:
//! ```text
//! Offset 0:  capacity  (u64) - maximum elements
//! Offset 8:  length    (u64) - current element count
//! Offset 16: elem_size (u64) - bytes per element (any positive value)
//! Offset 24: data[...] - actual element data
//! ```
//!
//! Supports Rust array expressions:
//! - [expr.array.array]: List form `[1, 2, 3]` via VEC_NEW + VEC_PUSH
//! - [expr.array.repeat]: Repeat form `[0; N]` via VEC_REPEAT
//! - [expr.array.index.array]: Indexing `arr[i]` via VEC_GET/VEC_SET

use crate::error::{VmError, VmResult};
use crate::state::VmState;

/// Vector header size in bytes (capacity + length + elem_size)
pub const VEC_HEADER_SIZE: usize = 24;

/// Offset of capacity field in vector header
const OFFSET_CAPACITY: usize = 0;
/// Offset of length field in vector header
const OFFSET_LENGTH: usize = 8;
/// Offset of elem_size field in vector header
const OFFSET_ELEM_SIZE: usize = 16;
/// Offset where data begins
const OFFSET_DATA: usize = 24;

// ============================================================================
// Helper functions for vector operations
// ============================================================================

/// Read vector capacity from heap
#[inline]
fn vec_get_capacity(state: &VmState, vec_addr: usize) -> VmResult<u64> {
    state.heap_read_u64(vec_addr + OFFSET_CAPACITY)
}

/// Read vector length from heap
#[inline]
fn vec_get_length(state: &VmState, vec_addr: usize) -> VmResult<u64> {
    state.heap_read_u64(vec_addr + OFFSET_LENGTH)
}

/// Read vector element size from heap
#[inline]
fn vec_get_elem_size(state: &VmState, vec_addr: usize) -> VmResult<u64> {
    state.heap_read_u64(vec_addr + OFFSET_ELEM_SIZE)
}

/// Write vector length to heap
#[inline]
fn vec_set_length(state: &mut VmState, vec_addr: usize, length: u64) -> VmResult<()> {
    state.heap_write_u64(vec_addr + OFFSET_LENGTH, length)
}

/// Calculate data offset for element at index
#[inline]
fn vec_data_offset(vec_addr: usize, index: u64, elem_size: u64) -> usize {
    vec_addr + OFFSET_DATA + (index as usize * elem_size as usize)
}

/// Read element from vector based on element size
/// - For 1, 2, 4, 8 bytes: returns the value directly
/// - For larger elements: returns the ADDRESS of the element in heap
///   (caller must use HEAP_LOAD operations to read struct fields)
fn vec_read_element(state: &VmState, vec_addr: usize, index: u64, elem_size: u64) -> VmResult<u64> {
    let offset = vec_data_offset(vec_addr, index, elem_size);
    match elem_size {
        1 => Ok(state.heap_read_u8(offset)? as u64),
        2 => Ok(state.heap_read_u16(offset)? as u64),
        4 => Ok(state.heap_read_u32(offset)? as u64),
        8 => state.heap_read_u64(offset),
        // For larger elements (structs), return the ADDRESS of the element
        // Caller uses HEAP_LOAD* operations to read individual fields
        _ => Ok(offset as u64),
    }
}

/// Write element to vector based on element size
/// - For 1, 2, 4, 8 bytes: writes the value directly
/// - For larger elements: `value` is a SOURCE ADDRESS in heap,
///   copies `elem_size` bytes from source to target slot
fn vec_write_element(state: &mut VmState, vec_addr: usize, index: u64, elem_size: u64, value: u64) -> VmResult<()> {
    let offset = vec_data_offset(vec_addr, index, elem_size);
    match elem_size {
        1 => state.heap_write_u8(offset, value as u8),
        2 => state.heap_write_u16(offset, value as u16),
        4 => state.heap_write_u32(offset, value as u32),
        8 => state.heap_write_u64(offset, value),
        // For larger elements (structs), `value` is source address
        // Copy elem_size bytes from source to target slot
        _ => {
            let src_addr = value as usize;
            let elem_size_usize = elem_size as usize;
            // Read source bytes
            let src_bytes = state.heap_read_bytes(src_addr, elem_size_usize)?;
            // Copy to a temporary buffer (to avoid borrow issues)
            let buffer = src_bytes.to_vec();
            // Write to target offset
            state.heap_write_bytes(offset, &buffer)
        }
    }
}

// ============================================================================
// Vector Operation Handlers
// ============================================================================

/// VEC_NEW: Create new vector with capacity
/// Stack: [capacity, elem_size] -> [vec_addr]
///
/// elem_size can be any positive value (1, 2, 4, 8 for primitives, larger for structs)
pub fn handle_vec_new(state: &mut VmState) -> VmResult<()> {
    let elem_size = state.pop()?;
    let capacity = state.pop()?;

    // Validate element size (must be > 0)
    if elem_size == 0 {
        return Err(VmError::HeapOutOfBounds);
    }

    // Calculate total size: header + (capacity * elem_size)
    let data_size = capacity.checked_mul(elem_size)
        .ok_or(VmError::HeapOutOfMemory)?;
    let total_size = VEC_HEADER_SIZE as u64 + data_size;

    // Allocate on heap
    let vec_addr = state.heap_alloc(total_size as usize)? as usize;

    // Initialize header
    state.heap_write_u64(vec_addr + OFFSET_CAPACITY, capacity)?;
    state.heap_write_u64(vec_addr + OFFSET_LENGTH, 0)?;
    state.heap_write_u64(vec_addr + OFFSET_ELEM_SIZE, elem_size)?;

    // Return vector address
    state.push(vec_addr as u64)
}

/// VEC_LEN: Get vector length
/// Stack: [vec_addr] -> [length]
pub fn handle_vec_len(state: &mut VmState) -> VmResult<()> {
    let vec_addr = state.pop()? as usize;
    let length = vec_get_length(state, vec_addr)?;
    state.push(length)
}

/// VEC_CAP: Get vector capacity
/// Stack: [vec_addr] -> [capacity]
pub fn handle_vec_cap(state: &mut VmState) -> VmResult<()> {
    let vec_addr = state.pop()? as usize;
    let capacity = vec_get_capacity(state, vec_addr)?;
    state.push(capacity)
}

/// VEC_PUSH: Push element to vector
/// Stack: [vec_addr, value] -> []
///
/// Note: Current implementation doesn't auto-grow. Use VEC_RESERVE first if needed.
pub fn handle_vec_push(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let vec_addr = state.pop()? as usize;

    let length = vec_get_length(state, vec_addr)?;
    let capacity = vec_get_capacity(state, vec_addr)?;
    let elem_size = vec_get_elem_size(state, vec_addr)?;

    // Check capacity
    if length >= capacity {
        return Err(VmError::HeapOutOfMemory); // Vector full
    }

    // Write element at current length position
    vec_write_element(state, vec_addr, length, elem_size, value)?;

    // Increment length
    vec_set_length(state, vec_addr, length + 1)
}

/// VEC_POP: Pop element from vector
/// Stack: [vec_addr] -> [value]
pub fn handle_vec_pop(state: &mut VmState) -> VmResult<()> {
    let vec_addr = state.pop()? as usize;

    let length = vec_get_length(state, vec_addr)?;
    let elem_size = vec_get_elem_size(state, vec_addr)?;

    // Check if empty
    if length == 0 {
        return Err(VmError::StackUnderflow); // Vector empty
    }

    // Decrement length first
    let new_length = length - 1;
    vec_set_length(state, vec_addr, new_length)?;

    // Read and return element
    let value = vec_read_element(state, vec_addr, new_length, elem_size)?;
    state.push(value)
}

/// VEC_GET: Get element at index (arr[i])
/// Stack: [vec_addr, index] -> [value]
///
/// Implements [expr.array.index.array] - bounds checked at runtime
pub fn handle_vec_get(state: &mut VmState) -> VmResult<()> {
    let index = state.pop()?;
    let vec_addr = state.pop()? as usize;

    let length = vec_get_length(state, vec_addr)?;
    let elem_size = vec_get_elem_size(state, vec_addr)?;

    // Bounds check [expr.array.index.const] at runtime
    if index >= length {
        return Err(VmError::HeapOutOfBounds); // Index out of bounds
    }

    let value = vec_read_element(state, vec_addr, index, elem_size)?;
    state.push(value)
}

/// VEC_SET: Set element at index (arr[i] = x)
/// Stack: [vec_addr, index, value] -> []
pub fn handle_vec_set(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let index = state.pop()?;
    let vec_addr = state.pop()? as usize;

    let length = vec_get_length(state, vec_addr)?;
    let elem_size = vec_get_elem_size(state, vec_addr)?;

    // Bounds check
    if index >= length {
        return Err(VmError::HeapOutOfBounds); // Index out of bounds
    }

    vec_write_element(state, vec_addr, index, elem_size, value)
}

/// VEC_REPEAT: Create vector with repeated value ([value; count])
/// Stack: [value, count, elem_size] -> [vec_addr]
///
/// Implements [expr.array.repeat] - creates array filled with copies of value
/// For elem_size > 8, only first 8 bytes are filled with value
pub fn handle_vec_repeat(state: &mut VmState) -> VmResult<()> {
    let elem_size = state.pop()?;
    let count = state.pop()?;
    let value = state.pop()?;

    // Validate element size (must be > 0)
    if elem_size == 0 {
        return Err(VmError::HeapOutOfBounds);
    }

    // Calculate total size
    let data_size = count.checked_mul(elem_size)
        .ok_or(VmError::HeapOutOfMemory)?;
    let total_size = VEC_HEADER_SIZE as u64 + data_size;

    // Allocate on heap
    let vec_addr = state.heap_alloc(total_size as usize)? as usize;

    // Initialize header
    state.heap_write_u64(vec_addr + OFFSET_CAPACITY, count)?;
    state.heap_write_u64(vec_addr + OFFSET_LENGTH, count)?; // Length = count (fully initialized)
    state.heap_write_u64(vec_addr + OFFSET_ELEM_SIZE, elem_size)?;

    // Fill with repeated value [expr.array.repeat-copy]
    for i in 0..count {
        vec_write_element(state, vec_addr, i, elem_size, value)?;
    }

    state.push(vec_addr as u64)
}

/// VEC_CLEAR: Clear vector (set length to 0)
/// Stack: [vec_addr] -> []
pub fn handle_vec_clear(state: &mut VmState) -> VmResult<()> {
    let vec_addr = state.pop()? as usize;
    vec_set_length(state, vec_addr, 0)
}

/// VEC_RESERVE: Reserve additional capacity (no-op in bump allocator)
/// Stack: [vec_addr, additional] -> []
///
/// Note: With bump allocator, we can't truly grow. This is a placeholder.
/// In production, you'd need a more sophisticated allocator.
pub fn handle_vec_reserve(state: &mut VmState) -> VmResult<()> {
    let _additional = state.pop()?;
    let _vec_addr = state.pop()? as usize;

    // With bump allocator, we can't resize in place.
    // This is a no-op - user should pre-allocate enough capacity.
    // A real implementation would allocate new memory and copy.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_header_constants() {
        assert_eq!(VEC_HEADER_SIZE, 24);
        assert_eq!(OFFSET_CAPACITY, 0);
        assert_eq!(OFFSET_LENGTH, 8);
        assert_eq!(OFFSET_ELEM_SIZE, 16);
        assert_eq!(OFFSET_DATA, 24);
    }
}
