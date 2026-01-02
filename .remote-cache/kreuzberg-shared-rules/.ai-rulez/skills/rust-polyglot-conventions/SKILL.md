______________________________________________________________________

## priority: critical

# Rust Polyglot Conventions

**Edition 2024**: let-chains, gen blocks, if/match guards. **Naming**: PascalCase (types), snake_case (fns/vars/modules), SCREAMING_SNAKE_CASE (consts).

**Error handling**: Result\<T, Error>, never .unwrap() in production, use `?`, IO errors bubble up properly, SAFETY comments for unsafe code, handle lock poisoning.

**Async**: Tokio throughout, #[tokio::main]/#[tokio::test], provide \_sync wrappers, never std::thread::sleep.

**Memory**: Arc for shared ownership, Mutex/RwLock for interior mutability, streaming for large data, RAII patterns.

**Performance**: ahash for HashMap, lazy_static/once_cell, SIMD where appropriate, zero-copy (&str/&[u8]).

**Plugins**: Traits for extensibility, Arc<dyn Trait> storage, Send+Sync requirements, registry pattern.

**Zero clippy warnings** (cargo clippy -- -D warnings).

**Core structure**: Modular organization with trait-based extensibility. Plugin flow: Input→Registry→Handler→Pipeline→Output.
