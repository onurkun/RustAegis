# 🛡️ RustAegis

> **Next-Generation Virtualization & Obfuscation Framework for Rust**

[![Crates.io](https://img.shields.io/crates/v/aegis_vm.svg)](https://crates.io/crates/aegis_vm)
[![Documentation](https://docs.rs/aegis_vm/badge.svg)](https://docs.rs/aegis_vm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

RustAegis is a research-grade software protection system that compiles Rust code into custom, polymorphic virtual machine bytecode. It is designed to protect sensitive logic against reverse engineering and tampering by moving execution from the native CPU to a secure, randomized software interpreter.

## 🚀 Key Features

*   **Virtualization:** Converts Rust AST directly into a custom stack-based VM instruction set.
*   **Polymorphism:** The instruction set mapping (Opcode Table) is randomized for every build via a `.build_seed` artifact.
*   **Mixed Boolean-Arithmetic (MBA):** Transforms simple arithmetic (`+`, `-`, `^`) into complex, mathematically equivalent boolean expressions.
*   **Compile-Time Encryption:** Bytecode is encrypted with a unique key per build and decrypted only at runtime.
*   **Anti-Tamper:** Integrated integrity checks ensure the bytecode has not been modified.
*   **Junk Code Injection:** Inserts dead code and entropy-based instructions to break signature scanning.

## 📦 Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
aegis_vm = "0.1.4"
```

## 🛠️ Usage

Apply the `#[vm_protect]` attribute to any sensitive function you wish to virtualize.

```rust
use aegis_vm::vm_protect;

// Standard protection (Polymorphism + Encryption)
#[vm_protect]
fn check_password(input: u64) -> bool {
    input == 0xCAFEBABE
}

// Paranoid protection (Heavy MBA + Obfuscation)
// Use this for critical logic like key derivation.
#[vm_protect(level = "paranoid")]
fn derive_key(seed: u64) -> u64 {
    // All arithmetic here is transformed into complex boolean logic
    (seed ^ 0x1234) + 0xABCD
}
```

## ⚙️ Architecture & The `.build_seed`

RustAegis uses a split architecture:
1.  **Compiler (`vm-macro`):** Runs at compile time, generating encrypted bytecode.
2.  **Runtime (`vm`):** Runs inside your application, executing the bytecode.

### Synchronization via `.anticheat_build_seed`
To ensure the compiler uses the exact same encryption keys and opcode mapping that the runtime expects, the system generates a temporary artifact named `.anticheat_build_seed` in your project root during the build process.

*   **Local Development:** This happens automatically. If you encounter a "Build ID mismatch" error, simply run `cargo clean` to regenerate the seed.
*   **CI/CD:** The seed is unique to each build environment. Do **not** commit `.anticheat_build_seed` to version control if you want unique polymorphism for every deployment.
*   **Reproducible Builds:** If you need exactly the same VM bytecode across different machines, you can set the `ANTICHEAT_BUILD_KEY` environment variable. This overrides the random generation.

```bash
# For reproducible builds (same opcodes, same keys)
export ANTICHEAT_BUILD_KEY="my-secret-company-build-key"
cargo build --release
```

## 🔍 Analysis & Reverse Engineering

RustAegis significantly complicates static and dynamic analysis by flattening control flow and obfuscating data flow.

### Control Flow Flattening

The VM interpreter acts as a massive switch statement (dispatcher). The original control flow (if/else, loops) is flattened into data-driven jumps within the interpreter loop.

**Native CFG:**
Distinct blocks for `if`, `else`, and `return`, easily readable by decompilers.

![Native Control Flow](docs/images/check_license_native_02_control_flow.png)
*Figure 1: Native assembly of the license check function. Logic is linear and easy to follow.*

**VM CFG:**
A single "God Node" (the dispatcher) with edges pointing back to itself. The actual logic is hidden in the bytecode data, not the CPU instructions.

![VM Control Flow Graph](docs/images/check_license_vm_02_control_flow.png)
*Figure 2: The same function protected by the VM. The control flow is flattened into the VM's fetch-decode-execute loop.*

### Arithmetic Obfuscation (MBA)

Instead of a single `ADD` instruction, the analyst sees a randomized sequence of stack operations implementing mathematically equivalent formulas like:
`x + y = (x ^ y) + 2 * (x & y)` or `(x | y) + (x & y)`

![Arithmetic Obfuscation Graph](docs/images/01_arithmetic_flow_graph.png)
*Figure 3: Even a simple arithmetic function explodes into a complex graph due to MBA transformations and the VM dispatcher overhead.*

## ⚡ Performance & Constraints

Virtualization comes with a cost. RustAegis is designed for **security**, not speed.

*   **Performance:** Expect a 10x-100x slowdown compared to native code. This is standard for software-based virtualization.
*   **Usage:** Apply `#[vm_protect]` **only** to sensitive functions (license checks, key generation, encryption logic). Do **not** virtualize tight loops in performance-critical rendering or physics code.
*   **Supported Platforms:** Works on `x86_64`, `aarch64`, `wasm32`, and any platform supported by Rust `std` or `alloc` (no_std compatible).

## 📂 Examples

Check the `examples/` directory for complete test cases:

*   `01_arithmetic.rs`: Demonstrates MBA transformations.
*   `02_control_flow.rs`: Demonstrates if/else logic protection.
*   `03_loops.rs`: Demonstrates loop virtualization.
*   `04_wasm.rs`: Demonstrates WASM integration.
*   `wasm_test/`: Complete WASM test project with `wasm-pack`.

Run them with:
```bash
cargo run --example 01_arithmetic
cargo run --example 04_wasm

# For WASM tests
cd examples/wasm_test
wasm-pack test --node
```

## 🌐 WASM Support

RustAegis fully supports WebAssembly. To use with WASM:

### Setup
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (optional, for building/testing)
cargo install wasm-pack
```

### Cargo.toml Configuration
```toml
[dependencies]
aegis_vm = { version = "0.1.3", default-features = false }
wasm-bindgen = "0.2"
```

### Usage Pattern
Since `#[vm_protect]` and `#[wasm_bindgen]` cannot be combined directly, use a wrapper:

```rust
use aegis_vm::vm_protect;
use wasm_bindgen::prelude::*;

// VM-protected implementation
#[vm_protect(level = "debug")]
fn secret_impl(x: u64) -> u64 {
    x ^ 0xDEADBEEF
}

// WASM export wrapper
#[wasm_bindgen]
pub fn secret(x: u64) -> u64 {
    secret_impl(x)
}
```

### Building WASM
```bash
cd examples/wasm_test

# Build for web
wasm-pack build --target web --release

# Run tests with Node.js
wasm-pack test --node

# Run tests in headless browser
wasm-pack test --headless --firefox
```

The compiled `.wasm` file will be in `pkg/` directory.

## 📋 Changelog

### v0.1.3

**New Features:**
*   **ValueCryptor (VMProtect-style):** Constants are now encrypted at compile-time with a chain of 3-7 cryptographic operations (ADD, SUB, XOR, ROL, ROR, NOT, NEG). The decryption chain is emitted as bytecode, preventing constants from appearing in plaintext.
*   **Region-based Integrity Checking:** Bytecode is divided into 64-byte regions, each with a precomputed FNV-1a hash. Tampering is detected at load time with detailed region identification for Paranoid level.
*   **Integrity Hash Verification:** All encrypted bytecode now includes a full integrity hash verified after decryption.

**Protection Levels:**
| Level | ValueCryptor | Full Hash | Region Hash |
|-------|--------------|-----------|-------------|
| debug | No | No | No |
| standard | No | Yes | No |
| paranoid | Yes | Yes | Yes |

**Note on Runtime Integrity:**
The current integrity checking protects against **static patching** (modifications to bytecode on disk). Runtime memory patching detection (continuous integrity checks during execution) is intentionally not included in this version to avoid performance overhead. This may be added as an optional feature in future releases for users who require protection against debugger-based runtime patching.

**Improvements:**
*   332 tests passing (up from ~160)
*   Better compile-time hash computation using build-specific FNV constants
*   Cleaner separation between compile-time and runtime integrity modules

### v0.1.2

**New Features:**
*   **WASM/WebAssembly Support:** Full `no_std` compatibility for `wasm32-unknown-unknown` target
*   **WASM Example:** Added `examples/04_wasm.rs` and `examples/wasm_test/` project with `wasm-pack` integration
*   **Industry-Standard Obfuscation:** Added new substitution patterns to `substitution.rs`:
    *   `AddSubstitution` - Multiple arithmetic identity transformations for ADD
    *   `SubSubstitution` - Multiple arithmetic identity transformations for SUB
    *   `MulSubstitution` - Multiplication obfuscation patterns
    *   `XorSubstitution` - XOR identity transformations
    *   `DeadCodeInsertion` - Deterministic dead code injection
    *   `OpaquePredicate` - Always-true/always-false conditions
    *   `ComparisonSubstitution` - Comparison obfuscation
    *   `ControlFlowSubstitution` - Control flow helpers

**Bug Fixes:**
*   Fixed `std::hint::black_box` → `core::hint::black_box` in `build.rs` for `no_std` compatibility
*   Fixed `SystemTime` usage with proper `#[cfg(feature = "std")]` guards in `state.rs` and `native.rs`
*   Refactored `compiler.rs` to use centralized `Substitution` module instead of inline implementations

**Improvements:**
*   Deterministic dead code insertion using position-based entropy (no RNG dependency)
*   Better separation of concerns between compiler and substitution modules

### v0.1.1

*   Initial public release
*   Core VM engine with 60+ opcodes
*   MBA (Mixed Boolean-Arithmetic) transformations
*   Compile-time encryption with AES-256-GCM
*   Polymorphic opcode shuffling

## ⚠️ Disclaimer

This project is for **educational and research purposes only**. It is designed to demonstrate concepts in software protection, obfuscation, and compiler theory.

## 📄 License

MIT