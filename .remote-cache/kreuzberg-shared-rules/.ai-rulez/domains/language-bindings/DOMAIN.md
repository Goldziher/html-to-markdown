# Language-Specific Bindings Domain

## Purpose

The language-bindings domain provides idiomatic APIs for Rust core across ten language ecosystems. Each binding acts as a thin wrapper exposing the Rust core through language-native patterns, conventions, and tooling while maintaining behavioral consistency and feature parity.

## Scope and Responsibilities

- Develop PyO3 Python bindings (crates/\*-py) with Python 3.10+ idioms and type stubs
- Implement NAPI-RS TypeScript bindings (crates/\*-node) with strictest type checking
- Create Magnus Ruby bindings (packages/ruby) with RBS type definitions and Steep type checking
- Build ext-php-rs PHP extension (packages/php-ext) with modern PHP 8.2+ syntax
- Integrate Java FFI bindings (packages/java) with JVM-native error handling
- Implement Go bindings (packages/go/v\*) with idiomatic goroutine patterns
- Create C# bindings (packages/csharp) with P/Invoke or NativeAOT
- Develop Elixir bindings (packages/elixir) with Rustler NIF
- Build WebAssembly bindings (crates/\*-wasm, packages/wasm) with wasm-bindgen
- Ensure language-idiomatic error handling and exception hierarchies per language
- Maintain 80%+ test coverage for all bindings in language-native test frameworks
- Provide comprehensive type definitions (Python stubs, TypeScript .d.ts, RBS, etc.)

## Referenced Agents

- **python-bindings-engineer**: PyO3 FFI expert for crates/\*-py and packages/python. Python-specific implementations, pyo3_async_runtimes for callbacks.
- **typescript-bindings-engineer**: NAPI-RS and TypeScript SDK development. Strictest TS flags, Biome linting, no any/object types.
- **ruby-bindings-engineer**: Magnus native bindings with RBS type definitions. Steep type checking, 80%+ RSpec coverage.
- **java-bindings-engineer**: JNI and FFI integration for Java 25+ with Foreign Function & Memory API.
- **go-bindings-engineer**: Go cgo wrapper design for idiomatic Go integration with channels/goroutines.

## Referenced Skills

- **python-kreuzberg-bindings**: PyO3 binding patterns, type stubs (\_rust.pyi), pyo3_async_runtimes for async callbacks
- **typescript-kreuzberg-bindings**: NAPI-RS FFI, strictest type flags, path aliases, Biome formatting
- **ruby-kreuzberg-bindings**: Magnus native bindings, RBS parallel definitions, Steep type checking
- **java-kreuzberg-bindings**: JNI wrapper generation, Java 25 FFM API integration
- **go-kreuzberg-bindings**: cgo wrapper design, idiomatic Go patterns, error type conversion
- **python-modern-performance-standards**: async/await patterns, dataclass optimization, import ordering
- **ruby-32-with-rbs-steep**: RBS file structure, Steep configuration, union and optional types
- **typescript-strictest-standards**: strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes flags
- **java-25-with-ffm-api**: Modern Java binding patterns for FFM API
- **go-125-standards**: Go 1.25+ standards, module organization
- **elixir-kreuzberg-bindings-rustler-nif**: Rustler NIF patterns, ExUnit testing
- **polyglot-bindings**: Universal patterns across all binding frameworks

## Referenced Rules

- **python-310-pyo3-binding-wrappers**: Target Python 3.10+, PyO3 minimal wrappers, type stubs, mypy --strict, 80%+ coverage
- **typescript-5x-napi-rs-bindings-cli**: TypeScript 5.x, NAPI-RS native bindings, strictest flags, vitest 80%+ coverage
- **ruby-32-magnus-native-bindings-with-rbs**: Ruby 3.2+, Magnus bindings, RBS type definitions, Steep checking
- **php-82-ext-php-rs-extension-composer-package**: PHP 8.2+, ext-php-rs, Composer package distribution

## Interaction Points

- **Receives from**: rust-core domain (FFI exposed through binding frameworks), ffi-bindings domain (Java/Go/C# use FFI layer)
- **Provides to**: documentation domain (bindings provide language-specific APIs to document), quality-verification (binding tests), build-distribution (package publishing)
- **Coordinates with**: organizational domain for language parity standards, quality-verification for integration testing

## Critical Files This Domain Manages

- `crates/*-py/src/lib.rs` (PyO3 bindings entry point)
- `packages/python/*/` (Python wrapper library)
- `crates/*-node/src/lib.rs` (NAPI-RS bindings)
- `packages/typescript/src/` (TypeScript SDK wrapper)
- `packages/ruby/lib/` (Magnus Ruby bindings wrapper)
- `packages/ruby/sig/` (RBS type definitions)
- `packages/java/src/main/java/` (JNI/FFM wrapper code)
- `packages/go/v*/` (Go binding wrapper)
- `packages/php-ext/` (PHP extension source)
- `packages/csharp/` (C# bindings)
- `packages/elixir/` (Elixir bindings)
- `packages/wasm/` or `crates/*-wasm/` (WebAssembly bindings)
