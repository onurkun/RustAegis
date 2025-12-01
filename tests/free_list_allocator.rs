//! Free-List Allocator Tests
//!
//! Tests to verify that HEAP_FREE actually works and memory is reused.
//! Before free-list: loops would cause memory to grow unboundedly.
//! After free-list: freed memory is recycled.

use aegis_vm::engine::execute;
use aegis_vm::error::VmError;
use aegis_vm::build_config::opcodes::{stack, heap, exec, arithmetic};

/// Test: Allocate, free, allocate again - should reuse same address
#[test]
fn test_free_and_reuse_same_address() {
    let code = [
        // First allocation (8 bytes)
        stack::PUSH_IMM8, 8,
        heap::HEAP_ALLOC,           // addr1 on stack

        // Save addr1 to compare later
        stack::DUP,                 // [addr1, addr1]

        // Free it
        heap::HEAP_FREE,            // [addr1] -> [] but addr1 still on stack from DUP

        // Allocate same size again
        stack::PUSH_IMM8, 8,
        heap::HEAP_ALLOC,           // addr2 on stack

        // Compare: addr1 == addr2 means reuse worked
        arithmetic::SUB,            // addr2 - addr1
        exec::HALT,
    ];

    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 0, "Second allocation should reuse freed block (same address)");
}

/// Test: Multiple alloc/free cycles don't grow heap
#[test]
fn test_repeated_alloc_free_no_growth() {
    let mut code = Vec::new();

    // Get initial heap size
    code.extend_from_slice(&[heap::HEAP_SIZE]);  // initial_size on stack

    // Do 100 alloc/free cycles
    for _ in 0..100 {
        code.extend_from_slice(&[
            stack::PUSH_IMM8, 64,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,
        ]);
    }

    // Get final heap size
    code.extend_from_slice(&[heap::HEAP_SIZE]);  // [initial, final]

    // Calculate growth: final - initial
    // SUB pops a, b and pushes b-a, so we need SWAP first
    code.extend_from_slice(&[stack::SWAP, arithmetic::SUB, exec::HALT]);

    let result = execute(&code, &[]).unwrap();
    // First allocation creates block, subsequent ones reuse it
    // Growth should be just ONE block: header(8) + data(64) = 72
    assert_eq!(result, 72, "100 alloc/free cycles should only grow by one block (72 bytes)");
}

/// Test: Without free, heap grows linearly
#[test]
fn test_without_free_heap_grows() {
    let mut code = Vec::new();

    // Allocate 10 times WITHOUT freeing
    for _ in 0..10 {
        code.extend_from_slice(&[
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
        ]);
    }

    code.extend_from_slice(&[heap::HEAP_SIZE, exec::HALT]);

    let result = execute(&code, &[]).unwrap();
    // 10 * (8 header + 8 data) = 10 * 16 = 160
    assert_eq!(result, 160, "10 allocations without free should use 160 bytes");
}

/// Test: With free, heap stays constant (reuse)
#[test]
fn test_with_free_heap_reuses() {
    let mut code = Vec::new();

    // Allocate and free 10 times
    for _ in 0..10 {
        code.extend_from_slice(&[
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,
        ]);
    }

    code.extend_from_slice(&[heap::HEAP_SIZE, exec::HALT]);

    let result = execute(&code, &[]).unwrap();
    // Only ONE block ever allocated: header(8) + data(8) = 16
    assert_eq!(result, 16, "10 alloc/free cycles should only use 16 bytes total");
}

/// Test: Free list merges adjacent blocks
#[test]
fn test_merge_adjacent_blocks() {
    let code = [
        // Allocate 3 adjacent blocks
        stack::PUSH_IMM8, 32,
        heap::HEAP_ALLOC,           // block1
        stack::PUSH_IMM8, 32,
        heap::HEAP_ALLOC,           // block2
        stack::PUSH_IMM8, 32,
        heap::HEAP_ALLOC,           // block3

        // Stack: [block1, block2, block3]
        // Free block3
        heap::HEAP_FREE,            // [block1, block2]
        // Free block2
        heap::HEAP_FREE,            // [block1]
        // Free block1
        heap::HEAP_FREE,            // []

        // Now allocate a BIG block - should fit in merged space
        // 3 * (8+32) = 120 bytes were freed, merged into one block
        // Allocate 100 bytes: needs 8+104=112 < 120
        stack::PUSH_IMM8, 100,
        heap::HEAP_ALLOC,
        stack::DROP,

        // If merge worked, heap size should still be 120 (no new allocation)
        heap::HEAP_SIZE,
        exec::HALT,
    ];

    let result = execute(&code, &[]).unwrap();
    // Original 3 blocks: 3 * 40 = 120
    // After free+merge: one free block of 120
    // New alloc of 100 (needs 112): fits in merged block
    assert_eq!(result, 120, "Merged free blocks should accommodate larger allocation");
}

/// Test: Loop that would explode without free-list
/// This simulates string concatenation in a loop
#[test]
fn test_loop_simulation_with_free() {
    let mut code = Vec::new();

    // Simulate: for i in 0..50 { temp = alloc(); free(temp); }
    // Without free-list: 50 * 24 = 1200 bytes
    // With free-list: 24 bytes (reused)

    for _ in 0..50 {
        code.extend_from_slice(&[
            stack::PUSH_IMM8, 16,    // "string" allocation
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,         // free after use
        ]);
    }

    code.extend_from_slice(&[heap::HEAP_SIZE, exec::HALT]);

    let result = execute(&code, &[]).unwrap();
    // header(8) + data(16) = 24
    assert_eq!(result, 24, "50 iterations with free should only use 24 bytes");
}

/// Test: Interleaved alloc/free pattern
#[test]
fn test_interleaved_alloc_free() {
    let code = [
        // Alloc A
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // A

        // Alloc B
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // B

        // Free A (not B)
        stack::SWAP,
        heap::HEAP_FREE,            // stack: [B]

        // Alloc C - should reuse A's space
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // C (should be at A's address)

        // Free B
        stack::SWAP,
        heap::HEAP_FREE,            // stack: [C]

        // Free C
        heap::HEAP_FREE,

        // Final heap size: 2 blocks were allocated max
        // 2 * (8+16) = 48
        heap::HEAP_SIZE,
        exec::HALT,
    ];

    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 48, "Interleaved pattern should use max 2 blocks");
}

/// Test: Double-free protection
/// Freeing the same block twice must return DoubleFree error
#[test]
fn test_double_free_protection() {
    let code = [
        // Allocate a block
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // addr on stack

        // Free it once
        stack::DUP,
        heap::HEAP_FREE,            // OK

        // Try to free it again - should fail!
        heap::HEAP_FREE,            // DoubleFree!

        exec::HALT,
    ];

    let result = execute(&code, &[]);
    assert_eq!(result, Err(VmError::DoubleFree), "Double-free must be detected");
}

/// Test: Double-free after reallocation (use-after-free scenario)
#[test]
fn test_double_free_after_realloc() {
    let code = [
        // Allocate block A
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // addr_a

        // Save addr_a
        stack::DUP,                 // [addr_a, addr_a]

        // Free block A
        heap::HEAP_FREE,            // [addr_a]

        // Allocate block B (reuses A's space)
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // [addr_a, addr_b] where addr_b == addr_a

        // Drop addr_b
        stack::DROP,                // [addr_a]

        // Try to free using old addr_a pointer (block is now allocated as B)
        // This should succeed because block is allocated (as B)
        heap::HEAP_FREE,            // OK (frees B)

        // Try to free again - double free!
        stack::PUSH_IMM8, 8,        // same user addr (8)
        heap::HEAP_FREE,            // DoubleFree!

        exec::HALT,
    ];

    let result = execute(&code, &[]);
    assert_eq!(result, Err(VmError::DoubleFree), "Double-free after realloc must be detected");
}

/// Test: Normal free-alloc-free cycle works
#[test]
fn test_free_alloc_free_cycle() {
    let code = [
        // Allocate
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,

        // Free
        heap::HEAP_FREE,

        // Allocate again (reuses)
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,

        // Free again - should work (different allocation)
        heap::HEAP_FREE,

        // Return heap size
        heap::HEAP_SIZE,
        exec::HALT,
    ];

    let result = execute(&code, &[]).unwrap();
    // Only one block was ever in use: header(8) + data(16) = 24
    assert_eq!(result, 24, "Free-alloc-free cycle should work normally");
}

/// Test: Write to reused block (critical bugfix test)
/// This verifies that heap_write uses heap.len() not heap_ptr
#[test]
fn test_write_to_reused_block() {
    let code = [
        // Allocate first block
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // addr1 (=8)

        // Write value 42 to it
        stack::DUP,
        stack::PUSH_IMM8, 42,
        heap::HEAP_STORE8,          // store 42 at addr1

        // Allocate second block (pushes heap_ptr forward)
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // addr2 (=32)
        stack::DROP,

        // Free first block (addr1 goes to free list)
        heap::HEAP_FREE,            // free addr1

        // Allocate same size - should reuse addr1
        stack::PUSH_IMM8, 16,
        heap::HEAP_ALLOC,           // addr3 = addr1 (reused!)

        // Write value 99 to reused block
        // This MUST work - it's writing to addr < heap_ptr but in valid heap.len()
        stack::DUP,
        stack::PUSH_IMM8, 99,
        heap::HEAP_STORE8,          // store 99 at addr3 (=addr1)

        // Read back to verify
        heap::HEAP_LOAD8,
        exec::HALT,
    ];

    let result = execute(&code, &[]).unwrap();
    assert_eq!(result, 99, "Write to reused block must succeed");
}

/// Comparison test: Show memory savings
#[test]
fn test_memory_savings_comparison() {
    // WITHOUT free
    let mut code_no_free = Vec::new();
    for _ in 0..20 {
        code_no_free.extend_from_slice(&[
            stack::PUSH_IMM8, 32,
            heap::HEAP_ALLOC,
            stack::DROP,    // Just drop, don't free
        ]);
    }
    code_no_free.extend_from_slice(&[heap::HEAP_SIZE, exec::HALT]);
    let size_no_free = execute(&code_no_free, &[]).unwrap();

    // WITH free
    let mut code_with_free = Vec::new();
    for _ in 0..20 {
        code_with_free.extend_from_slice(&[
            stack::PUSH_IMM8, 32,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE, // Free after use
        ]);
    }
    code_with_free.extend_from_slice(&[heap::HEAP_SIZE, exec::HALT]);
    let size_with_free = execute(&code_with_free, &[]).unwrap();

    // Without free: 20 * (8+32) = 800 bytes
    assert_eq!(size_no_free, 800);

    // With free: 1 * (8+32) = 40 bytes
    assert_eq!(size_with_free, 40);

    // Memory savings: 95%!
    let savings = 100 - (size_with_free * 100 / size_no_free);
    assert_eq!(savings, 95, "Free-list should save 95% memory");
}
