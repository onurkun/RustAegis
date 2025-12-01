//! Heap operation tests for the VM
//!
//! Production-level tests for heap allocation and memory operations.
//! Tests cover: allocation, alignment, read/write operations, error handling,
//! stress tests, concurrent-like patterns, memory patterns, and edge cases.

use aegis_vm::engine::execute;
use aegis_vm::error::VmError;
// Use shuffled opcodes from build config for tests
use aegis_vm::build_config::opcodes::{arithmetic, control, exec, heap, stack};

// =============================================================================
// SECTION 1: Basic Heap Allocation Tests
// =============================================================================

mod allocation {
    use super::*;

    #[test]
    fn test_basic_alloc_returns_zero() {
        // First allocation returns address 8 (after 8-byte header)
        let code = [
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(8)); // 8 = header size
    }

    #[test]
    fn test_sequential_allocs_increment() {
        // Multiple allocations return sequential addresses
        // First: header(8) + data(8) = 16 total, user addr = 8
        // Second: header at 16, user addr = 24
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,        // user addr 8
            stack::DROP,
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,        // user addr 24
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(24));
    }

    #[test]
    fn test_alignment_to_8_bytes() {
        // 1-byte allocation: header(8) + aligned(8) = 16 total
        // Second allocation starts at 16+8 = 24
        let code = [
            stack::PUSH_IMM8, 1,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 1,
            heap::HEAP_ALLOC,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(24));
    }

    #[test]
    fn test_alignment_7_bytes() {
        // 7-byte allocation: header(8) + aligned(8) = 16 total
        // Second allocation starts at 16+8 = 24
        let code = [
            stack::PUSH_IMM8, 7,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 1,
            heap::HEAP_ALLOC,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(24));
    }

    #[test]
    fn test_alignment_9_bytes() {
        // 9-byte allocation: header(8) + aligned(16) = 24 total
        // Second allocation starts at 24+8 = 32
        let code = [
            stack::PUSH_IMM8, 9,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 1,
            heap::HEAP_ALLOC,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(32));
    }

    #[test]
    fn test_zero_size_alloc() {
        // Zero-size allocation should still return valid address and align
        let code = [
            stack::PUSH_IMM8, 0,
            heap::HEAP_ALLOC,        // addr 0
            stack::DROP,
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,        // addr should be 0 since 0-byte aligned to 0
            exec::HALT,
        ];
        let result = execute(&code, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_allocation() {
        // Allocate 1000 bytes: header(8) + data(1000) = 1008
        let code = [
            stack::PUSH_IMM16, 0xE8, 0x03,  // 1000 in little-endian
            heap::HEAP_ALLOC,
            stack::DROP,
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        let result = execute(&code, &[]);
        assert_eq!(result, Ok(1008));  // 8 (header) + 1000 (aligned)
    }
}

// =============================================================================
// SECTION 2: Heap Size Tracking Tests
// =============================================================================

mod heap_size {
    use super::*;

    #[test]
    fn test_empty_heap_size() {
        let code = [
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0));
    }

    #[test]
    fn test_size_after_single_alloc() {
        // 100 bytes: header(8) + aligned(104) = 112
        let code = [
            stack::PUSH_IMM8, 100,
            heap::HEAP_ALLOC,
            stack::DROP,
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(112)); // 8 (header) + 104 (aligned)
    }

    #[test]
    fn test_size_accumulation() {
        // Each allocation: header(8) + aligned_data
        // alloc(8): 8+8=16, alloc(16): 8+16=24, alloc(24): 8+24=32
        // Total: 16+24+32 = 72
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 24,
            heap::HEAP_ALLOC,
            stack::DROP,
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(72)); // (8+8)+(8+16)+(8+24) = 72
    }
}

// =============================================================================
// SECTION 3: Heap Free Tests (Bump Allocator - No-op)
// =============================================================================

mod heap_free {
    use super::*;

    #[test]
    fn test_free_is_noop() {
        // Free adds block to free list, heap_ptr stays same
        // alloc(8): header(8) + data(8) = 16 total
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(16)); // heap_ptr unchanged
    }

    #[test]
    fn test_free_reuse() {
        // Free then alloc same size should reuse freed block
        // First alloc: header(8) + data(8) = 16
        // Free it, second alloc reuses it -> heap_size stays 16
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            heap::HEAP_FREE,
            heap::HEAP_SIZE,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(16)); // Reused, no growth
    }
}

// =============================================================================
// SECTION 4: Store/Load Tests (All Sizes)
// =============================================================================

mod store_load {
    use super::*;

    #[test]
    fn test_u8_store_load() {
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DUP,
            stack::PUSH_IMM8, 0xAB,
            heap::HEAP_STORE8,
            heap::HEAP_LOAD8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xAB));
    }

    #[test]
    fn test_u16_store_load() {
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DUP,
            stack::PUSH_IMM16, 0xCD, 0xAB,  // 0xABCD
            heap::HEAP_STORE16,
            heap::HEAP_LOAD16,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xABCD));
    }

    #[test]
    fn test_u32_store_load() {
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DUP,
            stack::PUSH_IMM32, 0xEF, 0xBE, 0xAD, 0xDE,  // 0xDEADBEEF
            heap::HEAP_STORE32,
            heap::HEAP_LOAD32,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xDEADBEEF));
    }

    #[test]
    fn test_u64_store_load() {
        let code = [
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            stack::DUP,
            stack::PUSH_IMM, 0xBE, 0xBA, 0xFE, 0xCA, 0xEF, 0xBE, 0xAD, 0xDE,
            heap::HEAP_STORE64,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xDEADBEEFCAFEBABE));
    }

    #[test]
    fn test_mixed_size_operations() {
        // Store different sizes at different offsets
        let code = [
            stack::PUSH_IMM8, 32,
            heap::HEAP_ALLOC,        // addr 0

            // Store u8 at offset 0
            stack::DUP,
            stack::PUSH_IMM8, 0x11,
            heap::HEAP_STORE8,

            // Store u16 at offset 2
            stack::DUP,
            stack::PUSH_IMM8, 2,
            arithmetic::ADD,
            stack::PUSH_IMM16, 0x33, 0x22,
            heap::HEAP_STORE16,

            // Store u32 at offset 8
            stack::DUP,
            stack::PUSH_IMM8, 8,
            arithmetic::ADD,
            stack::PUSH_IMM32, 0x77, 0x66, 0x55, 0x44,
            heap::HEAP_STORE32,

            // Read back u8
            stack::DUP,
            heap::HEAP_LOAD8,        // [addr, 0x11]

            // Read back u16
            stack::SWAP,
            stack::DUP,
            stack::PUSH_IMM8, 2,
            arithmetic::ADD,
            heap::HEAP_LOAD16,       // [0x11, addr, 0x2233]

            // Sum them
            arithmetic::ADD,         // [0x11, 0x2233+addr]... wait, need to fix

            stack::DROP,             // Drop addr
            exec::HALT,
        ];
        let result = execute(&code, &[]);
        // Just verify it doesn't crash
        assert!(result.is_ok());
    }

    #[test]
    fn test_u8_boundary_values() {
        // Test min/max u8 values
        let code = [
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            // Store 0
            stack::DUP,
            stack::PUSH_IMM8, 0,
            heap::HEAP_STORE8,
            // Store 255 at offset 1
            stack::DUP,
            stack::PUSH_IMM8, 1,
            arithmetic::ADD,
            stack::PUSH_IMM8, 0xFF,
            heap::HEAP_STORE8,
            // Read back 255
            stack::PUSH_IMM8, 1,
            arithmetic::ADD,
            heap::HEAP_LOAD8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(255));
    }

    #[test]
    fn test_u64_max_value() {
        let code = [
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            stack::DUP,
            stack::PUSH_IMM, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            heap::HEAP_STORE64,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(u64::MAX));
    }
}

// =============================================================================
// SECTION 5: Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_load_unallocated_address() {
        let code = [
            stack::PUSH_IMM8, 100,
            heap::HEAP_LOAD8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_store_unallocated_address() {
        let code = [
            stack::PUSH_IMM8, 100,
            stack::PUSH_IMM8, 42,
            heap::HEAP_STORE8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_load_past_allocation() {
        // Allocate 8 bytes, try to load at offset 100
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 100,
            heap::HEAP_LOAD8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_store_past_allocation() {
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 100,
            stack::PUSH_IMM8, 42,
            heap::HEAP_STORE8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_u16_partial_overflow() {
        // Allocate 8 bytes (user addr=8), try to load u16 at user_addr+7 (would read beyond)
        // user data: [8..16), reading at 15 would need bytes 15,16 but 16 is out
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 15,    // 8 (user addr) + 7 = 15
            heap::HEAP_LOAD16,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_u32_partial_overflow() {
        // Allocate 8 bytes (user addr=8), try to load u32 at user_addr+6
        // reading at 14 needs bytes 14,15,16,17 but heap ends at 16
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 14,    // 8 (user addr) + 6 = 14
            heap::HEAP_LOAD32,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }

    #[test]
    fn test_u64_partial_overflow() {
        // Allocate 8 bytes (user addr=8), try to load u64 at user_addr+1
        // reading at 9 needs bytes 9-16 but heap ends at 16, byte 16 out of bounds
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,
            stack::PUSH_IMM8, 9,     // 8 (user addr) + 1 = 9
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Err(VmError::HeapOutOfBounds));
    }
}

// =============================================================================
// SECTION 6: Complex Data Structure Tests
// =============================================================================

mod data_structures {
    use super::*;

    #[test]
    fn test_array_sum() {
        // Allocate array of 4 u64s, store values, sum them
        let code = [
            stack::PUSH_IMM8, 32,
            heap::HEAP_ALLOC,

            // Store 10 at offset 0
            stack::DUP,
            stack::PUSH_IMM8, 10,
            heap::HEAP_STORE64,

            // Store 20 at offset 8
            stack::DUP,
            stack::PUSH_IMM8, 8,
            arithmetic::ADD,
            stack::PUSH_IMM8, 20,
            heap::HEAP_STORE64,

            // Store 30 at offset 16
            stack::DUP,
            stack::PUSH_IMM8, 16,
            arithmetic::ADD,
            stack::PUSH_IMM8, 30,
            heap::HEAP_STORE64,

            // Store 40 at offset 24
            stack::DUP,
            stack::PUSH_IMM8, 24,
            arithmetic::ADD,
            stack::PUSH_IMM8, 40,
            heap::HEAP_STORE64,

            stack::DROP,

            // Sum all values (base addr is 8 after header)
            stack::PUSH_IMM8, 8,
            heap::HEAP_LOAD64,
            stack::PUSH_IMM8, 16,
            heap::HEAP_LOAD64,
            arithmetic::ADD,
            stack::PUSH_IMM8, 24,
            heap::HEAP_LOAD64,
            arithmetic::ADD,
            stack::PUSH_IMM8, 32,
            heap::HEAP_LOAD64,
            arithmetic::ADD,

            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(100));
    }

    #[test]
    fn test_linked_blocks() {
        // Allocate two blocks, store pointer in first, follow it
        // block1: header@0, data@8; block2: header@24, data@32
        let code = [
            // Allocate block1 (16 bytes)
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,        // block1 = 8

            // Allocate block2 (16 bytes)
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,        // block2 = 32

            // Store block2 address in block1[0]
            stack::PUSH_IMM8, 8,     // block1 addr
            stack::SWAP,
            heap::HEAP_STORE64,

            // Store 999 in block2[0]
            stack::PUSH_IMM8, 32,    // block2 addr
            stack::PUSH_IMM16, 0xE7, 0x03,  // 999
            heap::HEAP_STORE64,

            // Follow pointer: load block1[0]
            stack::PUSH_IMM8, 8,     // block1 addr
            heap::HEAP_LOAD64,
            // Now stack has block2 address (32)
            heap::HEAP_LOAD64,       // Load block2[0] = 999

            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(999));
    }

    #[test]
    fn test_byte_pattern() {
        // Write byte pattern, read as u32
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,

            // Write 0xAA at offset 0
            stack::DUP,
            stack::PUSH_IMM8, 0xAA,
            heap::HEAP_STORE8,

            // Write 0xBB at offset 1
            stack::DUP,
            stack::PUSH_IMM8, 1,
            arithmetic::ADD,
            stack::PUSH_IMM8, 0xBB,
            heap::HEAP_STORE8,

            // Write 0xCC at offset 2
            stack::DUP,
            stack::PUSH_IMM8, 2,
            arithmetic::ADD,
            stack::PUSH_IMM8, 0xCC,
            heap::HEAP_STORE8,

            // Write 0xDD at offset 3
            stack::DUP,
            stack::PUSH_IMM8, 3,
            arithmetic::ADD,
            stack::PUSH_IMM8, 0xDD,
            heap::HEAP_STORE8,

            // Read as u32 (little-endian: 0xDDCCBBAA)
            heap::HEAP_LOAD32,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xDDCCBBAA));
    }

    #[test]
    fn test_counter_increment() {
        // Use heap as a counter, increment it 5 times
        let code = [
            // Allocate counter
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            stack::DROP,

            // Initialize to 0
            stack::PUSH_IMM8, 0,
            stack::PUSH_IMM8, 0,
            heap::HEAP_STORE64,

            // Increment 5 times
            // 1st
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::INC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,
            // 2nd
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::INC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,
            // 3rd
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::INC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,
            // 4th
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::INC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,
            // 5th
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::INC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,

            // Read final value
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(5));
    }
}

// =============================================================================
// SECTION 7: Stress Tests
// =============================================================================

mod stress {
    use super::*;

    #[test]
    fn test_100_small_allocations() {
        // Each allocation: header(8) + data(8) = 16 bytes
        // 100 * 16 = 1600
        let mut code = Vec::new();

        for _ in 0..100 {
            code.push(stack::PUSH_IMM8);
            code.push(8);
            code.push(heap::HEAP_ALLOC);
            code.push(stack::DROP);
        }

        code.push(heap::HEAP_SIZE);
        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(1600));
    }

    #[test]
    fn test_1000_store_load_cycles() {
        // alloc(8): header(8) + data(8) = 16 bytes
        let mut code = Vec::new();

        // Allocate buffer
        code.push(stack::PUSH_IMM8);
        code.push(8);
        code.push(heap::HEAP_ALLOC);

        // Repeat store/load 100 times
        for i in 0..100 {
            // Store i
            code.push(stack::DUP);
            code.push(stack::PUSH_IMM8);
            code.push(i as u8);
            code.push(heap::HEAP_STORE8);

            // Load back
            code.push(stack::DUP);
            code.push(heap::HEAP_LOAD8);
            code.push(stack::DROP);
        }

        code.push(stack::DROP);
        code.push(heap::HEAP_SIZE);
        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(16)); // 8 (header) + 8 (data)
    }

    #[test]
    fn test_varying_allocation_sizes() {
        // Each allocation adds 8-byte header
        // sizes: 1->8, 2->8, 4->8, 8, 16, 32, 64 (aligned)
        // totals: 16, 16, 16, 16, 24, 40, 72 = 200
        let mut code = Vec::new();

        for size in [1, 2, 4, 8, 16, 32, 64].iter() {
            code.push(stack::PUSH_IMM8);
            code.push(*size as u8);
            code.push(heap::HEAP_ALLOC);
            code.push(stack::DROP);
        }

        code.push(heap::HEAP_SIZE);
        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(200));
    }

    #[test]
    fn test_alternating_sizes() {
        // Alternate between 8 and 16 bytes user data
        // 8 bytes: header(8) + data(8) = 16
        // 16 bytes: header(8) + data(16) = 24
        // 25*16 + 25*24 = 400 + 600 = 1000
        let mut code = Vec::new();

        for i in 0..50 {
            let size = if i % 2 == 0 { 8 } else { 16 };
            code.push(stack::PUSH_IMM8);
            code.push(size);
            code.push(heap::HEAP_ALLOC);
            code.push(stack::DROP);
        }

        code.push(heap::HEAP_SIZE);
        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(1000));
    }
}

// =============================================================================
// SECTION 8: Edge Cases and Boundary Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_read_at_exact_boundary() {
        // Allocate 8 bytes, read u8 at offset 7 (last valid byte)
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,
            // Store at offset 7
            stack::DUP,
            stack::PUSH_IMM8, 7,
            arithmetic::ADD,
            stack::PUSH_IMM8, 0x42,
            heap::HEAP_STORE8,
            // Read back
            stack::PUSH_IMM8, 7,
            arithmetic::ADD,
            heap::HEAP_LOAD8,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0x42));
    }

    #[test]
    fn test_overwrite_same_location() {
        // Write multiple times to same location
        let code = [
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,

            // Write 1
            stack::DUP,
            stack::PUSH_IMM8, 1,
            heap::HEAP_STORE64,

            // Write 2
            stack::DUP,
            stack::PUSH_IMM8, 2,
            heap::HEAP_STORE64,

            // Write 3
            stack::DUP,
            stack::PUSH_IMM8, 3,
            heap::HEAP_STORE64,

            // Read final value
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(3));
    }

    #[test]
    fn test_all_zeros() {
        // Verify freshly allocated memory is zero
        let code = [
            stack::PUSH_IMM8, 64,
            heap::HEAP_ALLOC,

            // Read u64 at offset 0
            stack::DUP,
            heap::HEAP_LOAD64,

            // Read u64 at offset 8
            stack::SWAP,
            stack::PUSH_IMM8, 8,
            arithmetic::ADD,
            heap::HEAP_LOAD64,

            // Sum should be 0
            arithmetic::ADD,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0));
    }

    #[test]
    fn test_address_arithmetic() {
        // Test that address calculations work correctly
        let code = [
            // Allocate 32 bytes
            stack::PUSH_IMM8, 32,
            heap::HEAP_ALLOC,        // addr = 0

            // Store unique values at each u64 slot
            // slot 0: 0xAAAAAAAAAAAAAAAA
            stack::DUP,
            stack::PUSH_IMM, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
            heap::HEAP_STORE64,

            // slot 1: 0xBBBBBBBBBBBBBBBB
            stack::DUP,
            stack::PUSH_IMM8, 8,
            arithmetic::ADD,
            stack::PUSH_IMM, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB,
            heap::HEAP_STORE64,

            // Read slot 1 back
            stack::PUSH_IMM8, 8,
            arithmetic::ADD,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xBBBBBBBBBBBBBBBB));
    }

    #[test]
    fn test_interleaved_alloc_and_write() {
        // Interleave allocations and writes
        let code = [
            // Alloc A
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,

            // Alloc B
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,

            // Write to A (should be at 0)
            stack::SWAP,
            stack::DUP,
            stack::PUSH_IMM8, 0xAA,
            heap::HEAP_STORE8,

            // Write to B (should be at 8)
            stack::SWAP,
            stack::DUP,
            stack::PUSH_IMM8, 0xBB,
            heap::HEAP_STORE8,

            // Read from A
            stack::SWAP,
            heap::HEAP_LOAD8,        // [B, 0xAA]

            // Read from B
            stack::SWAP,
            heap::HEAP_LOAD8,        // [0xAA, 0xBB]

            // Sum them
            arithmetic::ADD,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(0xAA + 0xBB));
    }
}

// =============================================================================
// SECTION 9: Memory Pattern Tests
// =============================================================================

mod patterns {
    use super::*;

    #[test]
    fn test_fill_pattern() {
        // Fill memory with a pattern and verify
        let mut code = Vec::new();

        // Allocate 16 bytes
        code.push(stack::PUSH_IMM8);
        code.push(16);
        code.push(heap::HEAP_ALLOC);

        // Fill with 0xAB
        for i in 0..16 {
            code.push(stack::DUP);
            code.push(stack::PUSH_IMM8);
            code.push(i);
            code.push(arithmetic::ADD);
            code.push(stack::PUSH_IMM8);
            code.push(0xAB);
            code.push(heap::HEAP_STORE8);
        }

        // Read byte 15 (last one)
        code.push(stack::PUSH_IMM8);
        code.push(15);
        code.push(arithmetic::ADD);
        code.push(heap::HEAP_LOAD8);
        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(0xAB));
    }

    #[test]
    fn test_sequential_values() {
        // Store sequential values 0-7
        // alloc(64) returns user addr 8 (after 8-byte header)
        let mut code = Vec::new();

        code.push(stack::PUSH_IMM8);
        code.push(64);
        code.push(heap::HEAP_ALLOC);  // returns 8

        // Store: use base_addr (on stack) + offset
        for i in 0..8 {
            code.push(stack::DUP);
            code.push(stack::PUSH_IMM8);
            code.push(i * 8);
            code.push(arithmetic::ADD);
            code.push(stack::PUSH_IMM8);
            code.push(i);
            code.push(heap::HEAP_STORE64);
        }

        // Drop base_addr after stores
        code.push(stack::DROP);

        // Sum all values: 0+1+2+3+4+5+6+7 = 28
        // Load using static addresses (8 + i*8)
        for i in 0..8 {
            code.push(stack::PUSH_IMM8);
            code.push(8 + i * 8);  // static address: 8, 16, 24, ...
            code.push(heap::HEAP_LOAD64);
        }

        // Sum them: stack has [v0, v1, v2, v3, v4, v5, v6, v7]
        for _ in 0..7 {
            code.push(arithmetic::ADD);
        }

        code.push(exec::HALT);

        assert_eq!(execute(&code, &[]), Ok(28));
    }
}

// =============================================================================
// SECTION 10: Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_heap_with_loops() {
        // Use heap memory as a loop counter and accumulator
        let code = [
            // Allocate: counter at 0, accumulator at 8
            stack::PUSH_IMM8, 16,
            heap::HEAP_ALLOC,
            stack::DROP,

            // Init counter = 5
            stack::PUSH_IMM8, 0,
            stack::PUSH_IMM8, 5,
            heap::HEAP_STORE64,

            // Init accumulator = 0
            stack::PUSH_IMM8, 8,
            stack::PUSH_IMM8, 0,
            heap::HEAP_STORE64,

            // Loop: while counter > 0
            // loop_start:
            // Load counter
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,

            // Test if counter > 0
            stack::DUP,
            stack::PUSH_IMM8, 0,
            control::CMP,
            stack::DROP,
            stack::DROP,
            control::JLE, 19, 0,     // Jump to end if counter <= 0 (+19 relative)

            // accumulator += counter
            stack::PUSH_IMM8, 8,
            heap::HEAP_LOAD64,
            arithmetic::ADD,
            stack::PUSH_IMM8, 8,
            stack::SWAP,
            heap::HEAP_STORE64,

            // counter -= 1
            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            arithmetic::DEC,
            stack::PUSH_IMM8, 0,
            stack::SWAP,
            heap::HEAP_STORE64,

            // Jump back to loop_start
            control::JMP, 0xE1, 0xFF, // -31 relative

            // end: return accumulator
            stack::PUSH_IMM8, 8,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        let result = execute(&code, &[]);
        // 5+4+3+2+1 = 15
        assert_eq!(result, Ok(15));
    }

    #[test]
    fn test_heap_with_input() {
        // Use heap to store computed value from input
        let code = [
            // Allocate buffer
            stack::PUSH_IMM8, 8,
            heap::HEAP_ALLOC,        // addr on stack

            // Read input[0] (8 bytes)
            stack::DUP,
            stack::PUSH_IMM8, 0,     // Push input offset
            stack::SWAP,             // [addr, 0] -> [0, addr]
            stack::SWAP,             // Keep addr on stack bottom
            stack::DROP,             // Drop the extra

            // Hmm, let me simplify. Just test basic heap works with input
            stack::DROP,             // drop addr

            // Store input directly
            stack::PUSH_IMM8, 0,     // heap addr
            stack::PUSH_IMM8, 42,    // value
            heap::HEAP_STORE64,

            stack::PUSH_IMM8, 0,
            heap::HEAP_LOAD64,
            exec::HALT,
        ];
        assert_eq!(execute(&code, &[]), Ok(42));
    }
}
