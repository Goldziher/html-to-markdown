______________________________________________________________________

## priority: high

# Kreuzberg Architecture Overview

**Welcome to Kreuzberg** - a polyglot document processing library with a Rust core and language bindings for Python, TypeScript, Ruby, PHP, Java, Go, C#, Elixir, and WebAssembly.

This document is your entry point to understanding the entire architecture. For detailed information, navigate through the table of contents below or consult the specialized documentation links throughout.

______________________________________________________________________

## Table of Contents

### Architecture Documentation

- [Core Architecture](#core-architecture)
- [Domain Organization](#domain-organization)
- [Polyglot Design Pattern](#polyglot-design-pattern)
- [Key Agents & Responsibilities](#key-agents--responsibilities)

### Quick Navigation

- [Quick Start for Developers](#quick-start-for-developers)
- [For Consuming Projects](#for-consuming-projects)
- [Detailed Architecture Resources](#detailed-architecture-resources)
- [Decision Records](#decision-records)

______________________________________________________________________

## Core Architecture

### Rust-First, Language-Agnostic Foundation

Kreuzberg is built on a **Rust-first architecture** principle: all extraction logic, transformation algorithms, and complex computations live in the Rust core library (`crates/kreuzberg`). This single source of truth ensures consistency, performance, and maintainability across all language ecosystems.

**The Architecture Pattern:**

```
┌─────────────────────────────────────────────────────────────┐
│                    Consuming Applications                     │
│         (Python, TypeScript, Ruby, Java, PHP, Go, etc.)      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  Language-Specific Bindings Layer             │
│  PyO3 | NAPI-RS | Magnus | ext-php-rs | JNI/FFI | cgo | etc. │
│        (Thin wrappers exposing language-idiomatic APIs)      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    FFI/Interop Layer (Optional)               │
│         (C-compatible interfaces for some languages)          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                     Rust Core Library                         │
│                    (crates/kreuzberg)                         │
│  • Document extraction and parsing                            │
│  • Text processing and sanitization                           │
│  • Plugin system (extensible backends)                        │
│  • High-performance async pipelines (Tokio)                   │
│  • SIMD optimization and zero-copy patterns                   │
└─────────────────────────────────────────────────────────────┘
```

### Core Modules

The Rust core is organized into focused modules managed by the `rust-core-engineer`:

- **api**: Public API surface and main entry points
- **cache**: Caching strategies for document processing
- **chunking**: Text chunking and segmentation algorithms
- **core**: Core extraction pipeline
- **extraction**: Document and content extraction logic
- **extractors**: Concrete extractor implementations
- **image**: Image processing and OCR integration points
- **keywords**: Keyword extraction algorithms
- **language_detection**: Language detection capabilities
- **mcp**: Model context protocol integration
- **ocr**: OCR backend abstraction and integration
- **pdf**: PDF-specific extraction logic
- **plugins**: Plugin system traits and infrastructure
- **stopwords**: Stopword management
- **text**: Text processing utilities
- **utils**: General utility functions

______________________________________________________________________

## Domain Organization

Kreuzberg uses a **domain-driven architecture** to organize agents, skills, and rules across functional areas:

### Eight Core Domains

1. **rust-core** - Core library implementation

   - Rust 2024 edition, Tokio async, plugin system
   - Primary agent: `rust-core-engineer`

1. **ffi-bindings** - C-compatible foreign function interfaces

   - Pointer marshaling, memory safety, cross-platform distribution
   - Languages: Java, Go, C#
   - Managed by: `rust-core-engineer` and language binding engineers

1. **language-bindings** - Language-specific wrapper libraries

   - PyO3 (Python), NAPI-RS (TypeScript), Magnus (Ruby), ext-php-rs (PHP), JNI/FFI (Java), cgo (Go), P/Invoke (C#), Rustler (Elixir), wasm-bindgen (WebAssembly)
   - 6 language binding engineer agents (python, typescript, ruby, java, go, php)
   - Primary coordinator: `polyglot-architect`

1. **build-distribution** - Build orchestration and packaging

   - Cross-platform compilation, artifact publishing, dependency management
   - Agents: `dependency-management-coordinator`, `release-versioning-coordinator`

1. **quality-verification** - Testing, CI/CD, code review

   - Test strategies, coverage enforcement, automated reviews
   - Agents: `test-automation-engineer`, `code-reviewer`, `performance-profiling-specialist`

1. **documentation** - API reference, tutorials, guides

   - Polyglot API consistency, language parity
   - Agents: `docs-writer`, `api-doc-writer`, `tutorial-writer`

1. **devops-infrastructure** - CI/CD workflows, deployment pipelines

   - GitHub Actions, multi-platform matrix testing
   - Agent: `devops-infrastructure-engineer`

1. **organizational** - Governance, standards, principles

   - Architecture decisions, coding standards, security policies
   - Coordinated by: `polyglot-architect`

______________________________________________________________________

## Polyglot Design Pattern

### Language Parity Through Thin Wrappers

Each language binding is designed as a **thin wrapper** that:

1. **Exposes the Rust core** through language-native FFI frameworks
1. **Adds language-idiomatic APIs** (not duplicating business logic)
1. **Maintains behavioral consistency** with the Rust reference implementation
1. **Provides type safety** appropriate to each language (type stubs, generics, type definitions)
1. **Handles exceptions** with language-appropriate error hierarchies
1. **Achieves 80%+ test coverage** using language-native test frameworks

### Supported Language Ecosystems

| Language | Framework | Package Manager | Test Framework | Type System |
|----------|-----------|-----------------|---|---|
| Python | PyO3 | PyPI | pytest | Type stubs (.pyi) |
| TypeScript | NAPI-RS | npm | vitest | TypeScript (strictest mode) |
| Ruby | Magnus | RubyGems | RSpec | RBS + Steep |
| PHP | ext-php-rs | Composer | PHPUnit | PHP type hints |
| Java | JNI/FFI | Maven | JUnit | Java generics + FFM API |
| Go | cgo | Go modules | Go testing | Interface-based |
| C# | P/Invoke | NuGet | xUnit | C# generics |
| Elixir | Rustler NIF | Hex | ExUnit | Typespecs |
| WebAssembly | wasm-bindgen | npm | JavaScript | TypeScript types |

### API Parity Requirement

The `polyglot-architect` ensures all language bindings expose equivalent APIs. For example, if the Rust core has `extract()`, `chunk()`, and `validate()` methods, all language bindings must expose equivalent functions with language-idiomatic names and signatures.

______________________________________________________________________

## Key Agents & Responsibilities

### Strategic Leadership

**polyglot-architect** (Sonnet model)

- System design for multi-language FFI integration
- Architecture Decision Records (ADRs) for major changes
- Language-binding coordination and feature parity enforcement
- Cross-platform compatibility decisions (Linux, macOS, Windows)
- Dependency management across Cargo, npm, PyPI, Maven, Go modules
- Plugin system and extensibility patterns

### Core Implementation

**rust-core-engineer** (Haiku model)

- All Rust core library development (PRIMARY ROLE)
- Rust 2024 edition standards, zero clippy warnings
- Tokio async patterns and performance optimization
- Plugin system architecture (DocumentExtractor, OcrBackend, PostProcessor, Validator traits)
- 95% test coverage enforcement
- SAFETY documentation for all unsafe code
- New features implemented in Rust first, then exposed through bindings

### Language Binding Engineers

**python-bindings-engineer** - PyO3 expert for crates/*-py and packages/python
**typescript-bindings-engineer** - NAPI-RS for crates/*-node and packages/typescript
**ruby-bindings-engineer** - Magnus for packages/ruby with RBS type definitions
**java-bindings-engineer** - JNI/FFI integration with Java 25 Foreign Function & Memory API
**go-bindings-engineer** - cgo wrappers with idiomatic Go patterns
**php-bindings-engineer** - ext-php-rs for packages/php-ext

### Specialized Engineers

- **ffi-maintenance-engineer** - C-compatible interfaces, pointer safety, cross-platform builds
- **test-automation-engineer** - Test strategies, CI/CD implementation, coverage enforcement
- **performance-profiling-specialist** - Benchmarking, optimization identification
- **security-auditing-specialist** - Vulnerability scanning, dependency auditing
- **devops-infrastructure-engineer** - CI/CD pipeline management, deployment automation
- **release-versioning-coordinator** - Version synchronization across all packages
- **dependency-management-coordinator** - Workspace and cross-ecosystem dependencies

### Documentation & Tutorials

- **docs-writer** - API documentation and user guides
- **api-doc-writer** - Language-specific API references
- **tutorial-writer** - Getting started guides and examples
- **code-reviewer** - Code quality and consistency reviews

______________________________________________________________________

## Quick Start for Developers

### Setting Up Your Development Environment

1. **Clone the repository:**

   ```bash
   git clone <repository-url>
   cd ai-rulez
   ```

1. **Install Rust toolchain** (Rust 2024 edition required):

   ```bash
   rustup toolchain install nightly
   rustup override set nightly
   ```

1. **Build the Rust core:**

   ```bash
   cargo build --release
   ```

1. **Run Rust tests** (95% coverage expected):

   ```bash
   cargo test
   cargo tarpaulin --out Html  # coverage report
   ```

1. **For specific language binding development:**

   - **Python:** `maturin develop` (builds PyO3 bindings)
   - **TypeScript:** `npm install && npm test` (in packages/typescript)
   - **Ruby:** `bundle install && bundle exec rspec` (in packages/ruby)
   - **Java:** `gradle build` (in packages/java)
   - **Go:** `go test ./...` (in packages/go)

### Key Development Principles

1. **Rust is the source of truth** - All business logic lives in `crates/kreuzberg/src/`
1. **Test first, implement second** - Minimum 95% coverage for Rust core, 80%+ for bindings
1. **Bindings are thin wrappers** - No duplicate logic in language-specific code
1. **Document every public API** - Use `///` doc comments with examples
1. **No .unwrap() in production** - Always use `Result<T, KreuzbergError>`
1. **Type safety throughout** - Leverage each language's type system to the fullest

### Common Development Tasks

| Task | Command |
|------|---------|
| Format code | `cargo fmt` (Rust) |
| Lint code | `cargo clippy -- -D warnings` (Rust) |
| Run all tests | `cargo test --all` (Rust core + all bindings) |
| Generate documentation | `cargo doc --open` (Rust) |
| Run benchmarks | `cargo bench -p kreuzberg` (Rust) |
| Create release build | `cargo build --release` (with platform-specific targets) |

______________________________________________________________________

## For Consuming Projects

### Using Kreuzberg in Your Project

Kreuzberg provides consistent APIs across all language ecosystems. Choose the binding that matches your project's language:

### Python

```python
from kreuzberg import extract, chunk, validate

# Extract from HTML
result = extract("https://example.com")

# Chunk text for processing
chunks = chunk(result.text, max_size=1024)

# Validate content
is_valid = validate(result)
```

### TypeScript

```typescript
import { extract, chunk, validate } from 'kreuzberg';

const result = await extract('https://example.com');
const chunks = await chunk(result.text, { maxSize: 1024 });
const isValid = await validate(result);
```

### Ruby

```ruby
require 'kreuzberg'

result = Kreuzberg.extract('https://example.com')
chunks = Kreuzberg.chunk(result.text, max_size: 1024)
is_valid = Kreuzberg.validate(result)
```

### Guarantee of Stability

All Kreuzberg bindings follow semantic versioning and maintain API stability:

- **Patch versions** (x.y.Z) - Bug fixes, no API changes
- **Minor versions** (x.Y.z) - New features, backward compatible
- **Major versions** (X.y.z) - Breaking changes, documented migrations

The `polyglot-architect` and language binding engineers ensure that API changes are coordinated across all language ecosystems simultaneously, preventing accidental divergence.

### Feature Parity

All language bindings expose equivalent feature sets. If a feature exists in Python, it exists in TypeScript, Ruby, and all other supported languages with idiomatic APIs.

Check the latest release notes to understand which features are available in your version.

______________________________________________________________________

## Detailed Architecture Resources

### Within This Directory

The `.ai-rulez/context/` directory contains detailed architecture documentation:

- **`architecture/`** - Detailed domain documentation (when created):

  - Domain-specific architecture patterns
  - Technology stack decisions
  - Integration points between domains

- **`decision-records/`** - Architecture Decision Records (ADRs) (when created):

  - Major architectural decisions and trade-offs
  - RFCs for significant features
  - Migration guides for breaking changes

### Related Documentation

- **Agents** - `.ai-rulez/agents/` - Detailed role descriptions and expertise areas
- **Domains** - `.ai-rulez/domains/` - Domain-specific purposes, responsibilities, and interaction points
- **Skills** - `.ai-rulez/skills/` - Technical implementation guidelines for specific domains
- **Rules** - `.ai-rulez/rules/` - Coding standards and requirements per language/framework

### Key Agents to Learn From

1. Start with **polyglot-architect** for system-wide design patterns
1. Understand **rust-core-engineer** for Rust core conventions
1. Review specific binding engineer documentation for your language
1. Check **ffi-maintenance-engineer** if working on cross-language integration

______________________________________________________________________

## Decision Records

Architecture Decision Records (ADRs) document major design decisions, trade-offs, and rationales. These will be maintained in `.ai-rulez/context/decision-records/` as the project evolves.

When a significant architectural decision is made, an ADR should be created to:

1. Document the decision and its context
1. Explain alternatives considered and why they were rejected
1. Record the rationale for the chosen approach
1. Capture any dependencies or migration considerations

______________________________________________________________________

## Directory Structure

### Rust Core Layout

```
crates/
├── kreuzberg/                 # Main Rust core library
│   ├── src/
│   │   ├── api.rs             # Public API
│   │   ├── cache.rs           # Caching strategies
│   │   ├── chunking.rs        # Text chunking
│   │   ├── core.rs            # Core pipeline
│   │   ├── extraction.rs      # Extraction logic
│   │   ├── extractors.rs      # Concrete extractors
│   │   ├── image.rs           # Image processing
│   │   ├── keywords.rs        # Keyword extraction
│   │   ├── language_detection.rs
│   │   ├── mcp.rs             # MCP integration
│   │   ├── ocr.rs             # OCR backends
│   │   ├── pdf.rs             # PDF processing
│   │   ├── plugins.rs         # Plugin system
│   │   ├── stopwords.rs       # Stopwords
│   │   ├── text.rs            # Text utilities
│   │   └── utils.rs           # General utils
│   ├── tests/                 # Integration tests
│   └── benches/               # Performance benchmarks
├── kreuzberg-ffi/             # C-compatible FFI layer
├── kreuzberg-py/              # PyO3 Python bindings
├── kreuzberg-node/            # NAPI-RS TypeScript bindings
└── kreuzberg-wasm/            # WebAssembly bindings
```

### Language Binding Packages

```
packages/
├── python/                    # Python wrapper library
├── typescript/                # TypeScript SDK wrapper
├── ruby/                      # Ruby binding wrapper + RBS types
├── java/                      # JNI/FFI Java wrapper
├── go/v1/                     # Go module wrapper
├── php-ext/                   # PHP extension source
├── csharp/                    # C# bindings
├── elixir/                    # Elixir binding
└── wasm/                      # WebAssembly package
```

______________________________________________________________________

## Key Principles

### Architectural Principles

1. **Rust-First** - All extraction logic in Rust core; language bindings are thin wrappers
1. **Feature Parity** - All language bindings expose equivalent APIs
1. **Type Safety** - Leverage each language's type system (stubs, generics, type definitions)
1. **Test Coverage** - 95% for Rust core, 80%+ for language bindings
1. **Performance** - SIMD, streaming, zero-copy patterns where applicable
1. **Extensibility** - Plugin system for custom extractors, OCR backends, post-processors

### Development Principles

1. **Dependency Injection** - Better testability and flexibility
1. **Interface-Based Design** - Loose coupling between components
1. **Clear Separation of Concerns** - Business logic separate from infrastructure
1. **No Unsafe in Production** - All unsafe Rust code must have SAFETY documentation
1. **Documentation Driven** - All public APIs have doc comments with examples
1. **Semantic Versioning** - Clear versioning across all language ecosystems

______________________________________________________________________

## Getting Help

- Check the agent documentation (`.ai-rulez/agents/`) for role-specific questions
- Review domain documentation (`.ai-rulez/domains/`) for architectural guidance
- Consult skills documentation (`.ai-rulez/skills/`) for implementation patterns
- Read rules documentation (`.ai-rulez/rules/`) for language-specific standards
- File an issue or discussion for cross-cutting concerns

Last updated: 2025-12-28
