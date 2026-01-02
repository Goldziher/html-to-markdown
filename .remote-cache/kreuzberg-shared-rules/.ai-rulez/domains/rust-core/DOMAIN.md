# Rust Core Library Domain

## Purpose

The Rust core library domain houses the primary extraction and conversion logic. Kreuzberg's Rust core (crates/kreuzberg) is a standalone library that serves as the single source of truth for all document processing, text extraction, and transformation capabilities across all language bindings.

## Scope and Responsibilities

- Design and implement core extraction engine (HTML parsing, text extraction, sanitization)
- Develop high-performance async pipelines using Tokio for document processing
- Implement plugin system architecture (DocumentExtractor, OcrBackend, PostProcessor, Validator traits)
- Maintain strict Rust 2024 edition standards with zero clippy warnings and 95% test coverage
- Ensure error handling through Result\<T, KreuzbergError> with full SAFETY documentation
- Provide language-agnostic Rust APIs that language bindings wrap without duplication
- Manage all core modules: api, cache, chunking, core, extraction, extractors, image, keywords, language_detection, mcp, ocr, pdf, plugins, stopwords, text, utils
- Optimize performance through SIMD, streaming, zero-copy patterns, and memory management

## Referenced Agents

- **rust-core-engineer**: PRIMARY Rust engineer responsible for all core library development. Edition 2024, Tokio async expertise, plugin system design, 95% coverage enforcement.

## Referenced Skills

- **rust-latest-edition-standards**: Rust 2024 with clippy -D warnings, Result-based error handling, 95% coverage minimum, SAFETY comments for unsafe blocks
- **rust-kreuzberg-specific-conventions**: PascalCase types, snake_case functions, let-chains, gen blocks, Arc/Mutex patterns, Tokio-only async
- **rust-core-arch**: Core conversion pipeline architecture, html5ever/ammonia integration, streaming for large files, integration points with all binding layers
- **pyo3-performance-patterns**: Async callback patterns (pyo3_async_runtimes), GIL handling strategies for Python integration
- **error-handling-strategy**: KreuzbergError propagation, OSError/RuntimeError bubbling, exception hierarchy design

## Referenced Rules

- **rust-2024-edition-core-conversion-engine**: html5ever + ammonia, Tokio 1.x, thiserror custom errors, 95% coverage via cargo-llvm-cov, zero .unwrap() in production

## Interaction Points

- **Provides to**: All language-bindings domain (FFI exposed through PyO3, NAPI-RS, Magnus, etc.)
- **Depends on**: Core principles governance from organizational domain
- **Coordinates with**: quality-verification domain for coverage/testing standards, ffi-bindings for FFI boundary design

## Critical Files This Domain Manages

- `crates/kreuzberg/src/` (core library implementation with all module subdirectories)
- `crates/kreuzberg/Cargo.toml` (workspace root package configuration)
- `crates/kreuzberg/tests/` (integration test fixtures and harnesses)
- `crates/kreuzberg/benches/` (performance benchmarks with criterium)
