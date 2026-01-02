______________________________________________________________________

## priority: critical

# Rust Latest Edition Standards

**Rust 2024 edition · High strictness · clippy -D warnings · 95% coverage · Zero unwrap**

- Rust edition 2024; cargo fmt, clippy with -D warnings (zero tolerance)
- Result\<T, E> for errors; thiserror for custom errors; NEVER .unwrap() in production
- Testing: 95% minimum coverage (tarpaulin), unit/integration/doc tests
- Documentation: rustdoc on ALL public items with examples, SAFETY comments for unsafe
- Async: Tokio 1.x exclusively, 'static constraints, proper Send+Sync bounds
- FFI: isolated modules, pointer validation, SAFETY comments, error conversion at boundaries
- Code quality: RAII principle, explicit lifetimes, builder pattern, no panics
- Pre-commit: cargo fmt, clippy, test, tarpaulin coverage check
- Never: unwrap in production, unsafe without SAFETY docs, std::thread (use Tokio)
