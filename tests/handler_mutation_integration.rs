//! Handler Mutation Integration Tests
//!
//! These tests verify that mutated handlers are correctly integrated
//! into the VM execution pipeline via the module alias system.
//!
//! When `handler_mutation` feature is enabled (default):
//!   handlers::handle_add → mutation::mutated_add
//!
//! When disabled:
//!   handlers::handle_add → arithmetic::handle_add

use aegis_vm::execute;
// Use shuffled opcodes from build config
use aegis_vm::build_config::opcodes::{stack, arithmetic, control, exec};

// =============================================================================
// Basic Arithmetic Integration Tests
// =============================================================================

#[test]
fn test_vm_add_through_execute() {
    // Tests that ADD opcode goes through the (possibly mutated) handler
    let code = vec![
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 20,
        arithmetic::ADD,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 30);
}

#[test]
fn test_vm_sub_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 50,
        stack::PUSH_IMM8, 20,
        arithmetic::SUB,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 30);
}

#[test]
fn test_vm_mul_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 6,
        stack::PUSH_IMM8, 7,
        arithmetic::MUL,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 42);
}

#[test]
fn test_vm_xor_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 0b1100,
        stack::PUSH_IMM8, 0b1010,
        arithmetic::XOR,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0b0110);
}

#[test]
fn test_vm_and_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 0b1100,
        stack::PUSH_IMM8, 0b1010,
        arithmetic::AND,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0b1000);
}

#[test]
fn test_vm_or_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 0b1100,
        stack::PUSH_IMM8, 0b1010,
        arithmetic::OR,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0b1110);
}

#[test]
fn test_vm_not_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 0,
        arithmetic::NOT,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), u64::MAX);
}

#[test]
fn test_vm_inc_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 41,
        arithmetic::INC,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 42);
}

#[test]
fn test_vm_dec_through_execute() {
    let code = vec![
        stack::PUSH_IMM8, 43,
        arithmetic::DEC,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 42);
}

// =============================================================================
// Complex Expression Tests (Multiple Mutated Operations)
// =============================================================================

#[test]
fn test_vm_complex_arithmetic_expression() {
    // ((10 + 5) * 3) - 15 = 30
    let code = vec![
        stack::PUSH_IMM8, 10,
        stack::PUSH_IMM8, 5,
        arithmetic::ADD,      // 15
        stack::PUSH_IMM8, 3,
        arithmetic::MUL,      // 45
        stack::PUSH_IMM8, 15,
        arithmetic::SUB,      // 30
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 30);
}

#[test]
fn test_vm_bitwise_chain() {
    // (0xFF AND 0x0F) XOR 0x05 = 0x0A
    let code = vec![
        stack::PUSH_IMM8, 0xFF,
        stack::PUSH_IMM8, 0x0F,
        arithmetic::AND,      // 0x0F
        stack::PUSH_IMM8, 0x05,
        arithmetic::XOR,      // 0x0A
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0x0A);
}

#[test]
fn test_vm_inc_dec_chain() {
    // 10 -> INC -> INC -> DEC -> INC = 12
    let code = vec![
        stack::PUSH_IMM8, 10,
        arithmetic::INC,
        arithmetic::INC,
        arithmetic::DEC,
        arithmetic::INC,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 12);
}

#[test]
fn test_vm_not_double_inversion() {
    // NOT(NOT(42)) = 42
    let code = vec![
        stack::PUSH_IMM8, 42,
        arithmetic::NOT,
        arithmetic::NOT,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 42);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_vm_add_overflow() {
    // MAX + 1 = 0 (wrapping)
    let code = vec![
        stack::PUSH_IMM,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // u64::MAX
        stack::PUSH_IMM8, 1,
        arithmetic::ADD,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0);
}

#[test]
fn test_vm_sub_underflow() {
    // 0 - 1 = MAX (wrapping)
    let code = vec![
        stack::PUSH_IMM8, 0,
        stack::PUSH_IMM8, 1,
        arithmetic::SUB,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), u64::MAX);
}

#[test]
fn test_vm_xor_self_zero() {
    // x XOR x = 0
    let code = vec![
        stack::PUSH_IMM8, 123,
        stack::DUP,
        arithmetic::XOR,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0);
}

#[test]
fn test_vm_and_with_zero() {
    // x AND 0 = 0
    let code = vec![
        stack::PUSH_IMM8, 255,
        stack::PUSH_IMM8, 0,
        arithmetic::AND,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 0);
}

#[test]
fn test_vm_or_with_zero() {
    // x OR 0 = x
    let code = vec![
        stack::PUSH_IMM8, 42,
        stack::PUSH_IMM8, 0,
        arithmetic::OR,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 42);
}

// =============================================================================
// Loop with Mutated Operations
// =============================================================================

#[test]
fn test_vm_sum_loop_with_mutated_add() {
    // Sum 1 to 5 using a loop: 1+2+3+4+5 = 15
    let code = vec![
        // R0 = counter (starts at 1)
        // R1 = sum (starts at 0)
        stack::PUSH_IMM8, 1,
        stack::POP_REG, 0,    // R0 = 1
        stack::PUSH_IMM8, 0,
        stack::POP_REG, 1,    // R1 = 0

        // loop_start (offset 8):
        stack::PUSH_REG, 1,   // push sum
        stack::PUSH_REG, 0,   // push counter
        arithmetic::ADD,       // sum + counter (MUTATED!)
        stack::POP_REG, 1,    // R1 = new sum

        // counter++
        stack::PUSH_REG, 0,
        arithmetic::INC,       // (MUTATED!)
        stack::POP_REG, 0,

        // if counter <= 5, jump to loop_start
        stack::PUSH_REG, 0,
        stack::PUSH_IMM8, 6,
        control::CMP,
        control::JLT,
        (-20i16 as u16) as u8, ((-20i16 as u16) >> 8) as u8, // relative jump back to offset 8

        // return sum
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 15);
}

#[test]
fn test_vm_factorial_with_mutated_mul() {
    // 5! = 120 using loop with MUL
    let code = vec![
        // R0 = counter (starts at 5)
        // R1 = result (starts at 1)
        stack::PUSH_IMM8, 5,
        stack::POP_REG, 0,    // R0 = 5
        stack::PUSH_IMM8, 1,
        stack::POP_REG, 1,    // R1 = 1

        // loop_start (offset 8):
        stack::PUSH_REG, 1,   // push result
        stack::PUSH_REG, 0,   // push counter
        arithmetic::MUL,       // result * counter (MUTATED!)
        stack::POP_REG, 1,    // R1 = new result

        // counter--
        stack::PUSH_REG, 0,
        arithmetic::DEC,       // (MUTATED!)
        stack::POP_REG, 0,

        // if counter > 0, jump to loop_start
        stack::PUSH_REG, 0,
        stack::PUSH_IMM8, 0,
        control::CMP,
        control::JGT,
        (-20i16 as u16) as u8, ((-20i16 as u16) >> 8) as u8, // relative jump back to offset 8

        // return result
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 120);
}

// =============================================================================
// Fibonacci with Multiple Mutated Operations
// =============================================================================

#[test]
fn test_vm_fibonacci_with_mutated_handlers() {
    // Fib(10) = 55
    // Uses ADD extensively - perfect test for mutated handler
    let code = vec![
        // R0 = n (input)
        // R1 = fib(n-1)
        // R2 = fib(n-2)
        // R3 = counter

        stack::PUSH_IMM8, 10,
        stack::POP_REG, 0,     // R0 = 10 (we want fib(10))

        stack::PUSH_IMM8, 1,
        stack::POP_REG, 1,     // R1 = 1 (fib(1))

        stack::PUSH_IMM8, 0,
        stack::POP_REG, 2,     // R2 = 0 (fib(0))

        stack::PUSH_IMM8, 2,
        stack::POP_REG, 3,     // R3 = 2 (counter starts at 2)

        // loop_start (offset 16):
        // temp = R1 + R2
        stack::PUSH_REG, 1,
        stack::PUSH_REG, 2,
        arithmetic::ADD,        // fib(n-1) + fib(n-2) (MUTATED!)

        // R2 = R1 (shift down)
        stack::PUSH_REG, 1,
        stack::POP_REG, 2,

        // R1 = temp (pop the sum)
        stack::POP_REG, 1,

        // counter++
        stack::PUSH_REG, 3,
        arithmetic::INC,        // (MUTATED!)
        stack::POP_REG, 3,

        // if counter <= n, continue loop
        stack::PUSH_REG, 3,
        stack::PUSH_REG, 0,
        control::CMP,
        control::JLE,
        (-24i16 as u16) as u8, ((-24i16 as u16) >> 8) as u8, // relative jump back to offset 16

        // return R1 (fib(n))
        stack::PUSH_REG, 1,
        exec::HALT,
    ];
    assert_eq!(execute(&code, &[]).unwrap(), 55);
}
