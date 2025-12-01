//! Heap Operation Handlers
//!
//! HEAP_ALLOC, HEAP_FREE, HEAP_LOAD*, HEAP_STORE*, HEAP_SIZE

use crate::error::VmResult;
use crate::state::VmState;

/// HEAP_ALLOC: Allocate memory on heap
/// Stack: [size] -> [address]
pub fn handle_heap_alloc(state: &mut VmState) -> VmResult<()> {
    let size = state.pop()? as usize;
    let addr = state.heap_alloc(size)?;
    state.push(addr as u64)
}

/// HEAP_FREE: Free heap memory and return it to free list
/// Stack: [address] -> []
///
/// The address must be a valid user address returned by HEAP_ALLOC.
/// The block will be added to the free list and may be reused by
/// future allocations. Adjacent free blocks are merged automatically.
pub fn handle_heap_free(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    state.heap_free(addr)
}

/// HEAP_LOAD8: Read u8 from heap
/// Stack: [address] -> [value]
pub fn handle_heap_load8(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u8(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD16: Read u16 from heap (little-endian)
/// Stack: [address] -> [value]
pub fn handle_heap_load16(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u16(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD32: Read u32 from heap (little-endian)
/// Stack: [address] -> [value]
pub fn handle_heap_load32(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u32(addr)? as u64;
    state.push(value)
}

/// HEAP_LOAD64: Read u64 from heap (little-endian)
/// Stack: [address] -> [value]
pub fn handle_heap_load64(state: &mut VmState) -> VmResult<()> {
    let addr = state.pop()? as usize;
    let value = state.heap_read_u64(addr)?;
    state.push(value)
}

/// HEAP_STORE8: Write u8 to heap
/// Stack: [address, value] -> []
pub fn handle_heap_store8(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u8;
    let addr = state.pop()? as usize;
    state.heap_write_u8(addr, value)
}

/// HEAP_STORE16: Write u16 to heap (little-endian)
/// Stack: [address, value] -> []
pub fn handle_heap_store16(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u16;
    let addr = state.pop()? as usize;
    state.heap_write_u16(addr, value)
}

/// HEAP_STORE32: Write u32 to heap (little-endian)
/// Stack: [address, value] -> []
pub fn handle_heap_store32(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()? as u32;
    let addr = state.pop()? as usize;
    state.heap_write_u32(addr, value)
}

/// HEAP_STORE64: Write u64 to heap (little-endian)
/// Stack: [address, value] -> []
pub fn handle_heap_store64(state: &mut VmState) -> VmResult<()> {
    let value = state.pop()?;
    let addr = state.pop()? as usize;
    state.heap_write_u64(addr, value)
}

/// HEAP_SIZE: Get current heap pointer (bytes used)
/// Stack: [] -> [heap_ptr]
pub fn handle_heap_size(state: &mut VmState) -> VmResult<()> {
    let size = state.heap_size() as u64;
    state.push(size)
}
