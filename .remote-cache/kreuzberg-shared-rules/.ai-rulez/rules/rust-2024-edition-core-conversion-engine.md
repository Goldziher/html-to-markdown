______________________________________________________________________

## priority: medium

# Rust 2024 Edition - Core Conversion Engine

**Rust 2024 edition · html5ever + ammonia · clippy -D warnings · 95% coverage**

- Rust 2024; cargo fmt, clippy with -D warnings (zero tolerance)
- Result\<T, E> for errors; thiserror for custom errors; NEVER .unwrap() in production
- Testing: 95% minimum coverage (cargo-llvm-cov), unit/integration/doc tests in crates/
- Documentation: rustdoc on ALL public items with examples, SAFETY comments for unsafe
- Async: Tokio 1.x exclusively, 'static constraints, proper Send+Sync bounds
- Core libraries: html5ever (parsing), ammonia (sanitization), regex, encoding_rs
- Pre-commit: cargo fmt, clippy, test, coverage check
- Never: unwrap in production, unsafe without SAFETY docs, panics in library code
- Use Sonnet 4.5 for architectural decisions on the Rust core
