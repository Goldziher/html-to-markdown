______________________________________________________________________

## priority: medium

# Rust Core Architecture

## Core Library Crate

- **Library crate**: crates/core-library/ implements the core domain logic
- **Domain focus**: Clear separation of concerns with focused business logic
- **Error handling**: thiserror for ergonomic custom errors
- **Processing pipeline**: Parse input → Transform data → Validate result → Format output
- **Performance**: zero-copy where possible, streaming for large data structures
- **Dependencies**: Leverage the Rust ecosystem (serde, tracing, thiserror, etc.)

## Integration Points

- **PyO3 bindings**: crates/core-library-py exports Rust API to Python
- **NAPI-RS bindings**: crates/core-library-node for Node.js/Bun
- **Magnus bindings**: Ruby gem uses Magnus for clean FFI
- **ext-php-rs bindings**: crates/core-library-php for PHP extension
- **WASM**: crates/core-library-wasm for browser/Wasmtime
- **FFI library**: crates/core-library-ffi for C-compatible exports (Go, Java, C#)

## Testing Strategy

- Doc tests on public types with realistic domain examples
- Unit tests per module (core components, utilities, validators)
- Integration tests with actual data fixtures in crates/core-library/tests/
- Benchmarks in benches/ with criterium for performance-critical paths
- Coverage: cargo-llvm-cov with 95% threshold
