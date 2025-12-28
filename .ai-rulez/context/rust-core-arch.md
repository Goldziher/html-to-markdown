---
priority: medium
---

# Rust Core Architecture

## Core Conversion Engine
- **Library crate**: crates/html-to-markdown/ implements HTML→Markdown conversion
- **Parser**: html5ever for robust HTML5 parsing
- **Sanitizer**: ammonia for XSS prevention and safe HTML handling
- **Error handling**: thiserror for ergonomic custom errors
- **Conversion pipeline**: Parse → Walk tree → Convert nodes → Format output
- **Performance**: zero-copy where possible, streaming for large documents

## Integration Points
- **PyO3 bindings**: crates/html-to-markdown-py exports Rust API to Python
- **NAPI-RS bindings**: crates/html-to-markdown-node for Node.js/Bun
- **Magnus bindings**: Ruby gem uses Magnus for clean FFI
- **ext-php-rs bindings**: crates/html-to-markdown-php for PHP extension
- **WASM**: crates/html-to-markdown-wasm for browser/Wasmtime
- **FFI library**: crates/html-to-markdown-ffi for C-compatible exports (Go, Java, C#)

## Testing Strategy
- Doc tests on public types with realistic HTML examples
- Unit tests per module (parser, sanitizer, converters)
- Integration tests with actual HTML fixtures in crates/html-to-markdown/tests/
- Benchmarks in benches/ with criterium
- Coverage: cargo-llvm-cov with 95% threshold
