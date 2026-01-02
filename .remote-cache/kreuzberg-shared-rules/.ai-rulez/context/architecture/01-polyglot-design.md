# Polyglot Design Architecture

## Overview

This document defines the architectural principles for the polyglot Rust library, establishing the foundation for safe, efficient, and consistent cross-language bindings across 10 programming languages. The design prioritizes safety, performance, and developer experience through a Rust-core model where all business logic resides in a single compiled artifact.

## FFI Boundaries and Safety Guarantees

### Safety Model

The FFI (Foreign Function Interface) boundary is the critical security perimeter of our architecture. All binding layers implement strict safety guarantees:

```
┌─────────────────────────────────────────┐
│      Host Language Runtime               │
│    (Python, Node, Ruby, etc.)            │
└────────────────────┬────────────────────┘
                     │
           ┌─────────▼─────────┐
           │  Language Binding  │
           │  (PyO3, NAPI, etc)│
           │  - Type conversion │
           │  - Error bridging  │
           │  - Memory safety   │
           └─────────┬──────────┘
                     │
           ┌─────────▼─────────────────┐
           │  #[no_std] FFI Boundary  │
           │  - Checked allocations   │
           │  - Explicit conversions  │
           │  - SAFETY comments      │
           └─────────┬────────────────┘
                     │
           ┌─────────▼────────────────┐
           │   Safe Rust Core         │
           │   (95%+ Safe Rust)       │
           │   - Data structures      │
           │   - Business logic       │
           │   - Algorithms           │
           └──────────────────────────┘
```

### Unsafe Boundary Rules

1. **All unsafe code is in binding layers or FFI wrappers**

   - Core library is 95%+ safe Rust
   - Unsafe blocks require `// SAFETY:` comments (see 07-security-model.md)
   - Each unsafe operation must be justified and documented

1. **Memory Safety Guarantees**

   - No memory leaks across FFI boundary
   - No use-after-free vulnerabilities
   - No buffer overflows in type conversions
   - Proper cleanup in all error paths

1. **Thread Safety**

   - All bindings maintain Send/Sync bounds
   - Concurrent access is serialized or protected with Arc\<Mutex<T>>
   - Language-specific concurrency models are wrapped appropriately

### Error Boundary

Errors crossing the FFI boundary must be:

- Serialized in a language-neutral format
- Deterministic and reproducible
- Include stack traces and context
- Never expose internal Rust panics

## Language Selection Rationale

### The 10 Languages

```yaml
Tier 1 (Production Ready):
  - Python: PyO3 → 35% developer population
  - TypeScript/Node: NAPI-RS → JavaScript ecosystem dominance
  - Rust: Native → Core library

Tier 2 (Mature Bindings):
  - Ruby: Magnus → Web development integration
  - Java: FFM API → Enterprise adoption
  - Go: cgo → System programming alignment

Tier 3 (Emerging):
  - C#: P/Invoke → .NET ecosystem
  - PHP: ext-php-rs → Web hosting ubiquity
  - Elixir: Rustler → Functional programming niche
  - WebAssembly: wasm-bindgen → Browser/Deno support
```

### Selection Criteria

Each language was chosen based on:

1. **Developer Population**: Target languages with 2M+ developers (Python, JS)
1. **Use Case Alignment**: Domains where Rust provides clear value (systems, performance)
1. **Binding Maturity**: Stable, well-maintained FFI crates available
1. **Community Support**: Active issue resolution and security updates
1. **Performance Requirements**: Expected FFI overhead acceptable for use case

## Binding Strategy: Thin Wrappers, Rust Core as Truth

### The Rust Core as Single Source of Truth

```
┌─────────────────────────────────────┐
│     Canonical Rust Core             │
│  - API surface definition           │
│  - All business logic               │
│  - Version as authoritative         │
│  - Cargo.toml drives dependency     │
│    versions for all bindings        │
└────────────┬────────────────────────┘
             │
    ┌────────┼────────┬────────┬──────────┐
    │        │        │        │          │
    ▼        ▼        ▼        ▼          ▼
  Python  TypeScript  Ruby   Java   WebAssembly
  (PyO3)  (NAPI-RS) (Magnus)(FFM) (wasm-bind)
```

### Thin Wrapper Principle

Bindings are intentionally thin to minimize:

- Maintenance burden (avoid duplicate logic)
- Version skew (single source of truth)
- Bug surface (less code = fewer bugs)
- Testing complexity (delegate to core tests)

**Binding Layer Responsibilities:**

- Type mapping (Rust ↔ Host language)
- Error conversion (Rust Result → Host exceptions)
- Memory management (reference counting, GC integration)
- Concurrency adaptation (Tokio → language runtimes)

**Core Library Responsibilities:**

- All algorithms and business logic
- Data structure implementations
- Configuration and defaults
- Performance-critical operations

### Example: Cross-Language Type Mapping

```rust
// Rust Core (src/lib.rs)
pub struct Config {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub enable_compression: bool,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validation logic
    }
}
```

```python
# Python Binding (bindings/python/src/lib.rs)
#[pyclass]
pub struct Config {
    inner: crate::Config,
}

#[pymethods]
impl Config {
    #[new]
    fn new(timeout_ms: u64, max_retries: u32, enable_compression: bool) -> Self {
        Config {
            inner: crate::Config {
                timeout_ms,
                max_retries,
                enable_compression,
            }
        }
    }

    fn validate(&self) -> PyResult<()> {
        self.inner.validate().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(e.to_string())
        })
    }
}
```

```typescript
// TypeScript Binding (bindings/typescript/src/lib.rs)
#[napi]
pub struct Config {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub enable_compression: bool,
}

#[napi]
impl Config {
    #[napi(constructor)]
    pub fn new(timeout_ms: u64, max_retries: u32, enable_compression: bool) -> Self {
        Config { timeout_ms, max_retries, enable_compression }
    }

    #[napi]
    pub fn validate(&self) -> Result<()> {
        crate::Config {
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            enable_compression: self.enable_compression,
        }.validate()
    }
}
```

## Cross-Language API Consistency

### Consistency Principles

1. **Naming Convention**: Same method names across all languages

   - `create_client()` in Rust → `create_client()` in Python/Ruby/Go
   - `from_config()` → `from_config()` (not `FromConfig()` in Python)
   - Exceptions: Language idioms override if unavoidable (C# → PascalCase)

1. **Function Signature Alignment**

   - Same parameters in same order
   - Same return types (mapped appropriately)
   - Same error conditions and messages
   - Same default values

1. **Behavioral Consistency**

   - Side effects identical across languages
   - Error messages identical
   - Performance characteristics within 10%
   - Async/sync handling parallel

### API Surface Definition

The canonical API is defined in Rust and replicated across bindings:

```rust
// src/client.rs - Authoritative API
pub struct Client { /* ... */ }

impl Client {
    pub fn new(config: Config) -> Result<Self> { /* ... */ }
    pub async fn fetch(&self, url: &str) -> Result<Data> { /* ... */ }
    pub fn set_timeout(&mut self, ms: u64) { /* ... */ }
}
```

This becomes:

```python
# Python
client = Client(config)  # from config or error
data = await client.fetch(url)
client.set_timeout(5000)
```

```typescript
// TypeScript
const client = new Client(config);
const data = await client.fetch(url);
client.setTimeout(5000);
```

### Validation Test Matrix

All bindings pass identical validation tests:

```yaml
Test Categories:
  API Parity:
    - Every method exists in every binding
    - Signatures match (parameter count, types)
    - Return types are correct
    - Errors match expected conditions

  Behavioral Parity:
    - Same operation produces same result
    - Error messages are identical
    - Edge cases handled consistently
    - Performance within tolerance

  Integration Tests:
    - Multi-language workflows
    - Shared memory scenarios
    - Concurrent access patterns
    - Error propagation
```

## Cross-References

- **Core Library Design**: See [02-rust-core-design.md](02-rust-core-design.md)
- **Binding Patterns**: See [03-binding-patterns.md](03-binding-patterns.md)
- **Security Model**: See [07-security-model.md](07-security-model.md)
- **Testing Strategy**: See [05-testing-strategy.md](05-testing-strategy.md)
- **Agent**: See `.ai-rulez/agents/architecture-advisor.yaml` for architectural decisions
- **Rule**: See `.ai-rulez/rules/polyglot-consistency.yaml` for enforcement rules

## Implementation Checklist

- [ ] All 10 binding layers implement thin wrapper pattern
- [ ] FFI boundary marked with `// SAFETY:` comments
- [ ] API consistency tests pass (see 05-testing-strategy.md)
- [ ] Memory safety audits completed (see 07-security-model.md)
- [ ] Cross-language integration tests implemented
- [ ] Performance benchmarks within 10% (see 06-performance-patterns.md)
