---
priority: medium
---

# Dual Testing Strategy - Core + Bindings

**Core logic: Rust tests (95%) · Bindings: Language-specific tests (80%+)**

- Rust core: crates/html-to-markdown tests → 95% coverage (cargo-llvm-cov)
- Python binding: packages/python/tests → 80%+ coverage (pytest-cov)
- TypeScript binding: packages/typescript/tests → 80%+ coverage (vitest)
- Ruby binding: packages/ruby/spec → 80%+ coverage (rspec)
- PHP binding: packages/php/tests → 80%+ coverage (phpunit)
- Go FFI: packages/go/v2/htmltomarkdown (black-box tests with cgo bindings)
- Java FFI: packages/java (JNI tests with cargo build -p html-to-markdown-ffi)
- C#/.NET: packages/csharp (P/Invoke tests with html-to-markdown-ffi)
- Fixture-driven: JSON/YAML fixtures in examples/fixtures/ with schemas
- Parametrized tests: use language-native parametrization (@dataProvider, @ParameterizedTest, etc.)
- Real HTML samples: use actual HTML from fixtures/ not mocks
- Never: Rust tests that mock the entire binding layer; test bindings in their native language
