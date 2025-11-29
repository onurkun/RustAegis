# aegis_vm WASM Test

This is a complete working example of using `aegis_vm` with WebAssembly.

## Prerequisites

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (for testing)
cargo install wasm-pack
```

## Running Tests

```bash
# Run tests with Node.js
wasm-pack test --node

# Run tests in headless browser
wasm-pack test --headless --firefox
# or
wasm-pack test --headless --chrome
```

## Building for Production

```bash
# Build optimized WASM
wasm-pack build --target web --release

# Or for Node.js
wasm-pack build --target nodejs --release
```

## Project Structure

```
wasm_test/
├── Cargo.toml          # Dependencies
├── src/
│   └── lib.rs          # VM-protected functions + WASM exports
└── README.md
```

## Usage Pattern

The key pattern is to separate VM-protected implementation from WASM exports:

```rust
use aegis_vm::vm_protect;
use wasm_bindgen::prelude::*;

// VM-protected implementation (not exported)
#[vm_protect(level = "debug")]
fn my_secret_impl(x: u64) -> u64 {
    x ^ 0xDEADBEEF
}

// WASM export wrapper
#[wasm_bindgen]
pub fn my_secret(x: u64) -> u64 {
    my_secret_impl(x)
}
```

## Protection Levels

- `debug` - Plaintext bytecode, fastest, for development
- `standard` - Encrypted bytecode (default)
- `paranoid` - Maximum protection

For WASM, start with `debug` level during development, then switch to `standard` for production.

## Cargo.toml Example

```toml
[package]
name = "my_wasm_app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
aegis_vm = { version = "0.1", default-features = false }
wasm-bindgen = "0.2"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

Note: Use `default-features = false` to disable `std` for proper WASM support.
