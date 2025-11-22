<!--
🤖 AI-RULEZ :: GENERATED FILE — DO NOT EDIT DIRECTLY
Project: html-to-markdown
Generated: 2025-11-22 09:36:48
Source of truth: ai-rulez.yaml
Target file: GEMINI.md
Content summary: rules=10, sections=7, agents=0

UPDATE WORKFLOW
1. Modify ai-rulez.yaml
2. Run `ai-rulez generate` to refresh generated files
3. Commit regenerated outputs together with the config changes

AI ASSISTANT SAFEGUARDS
- Treat ai-rulez.yaml as the canonical configuration
- Never overwrite GEMINI.md manually; regenerate instead
- Surface changes as patches to ai-rulez.yaml (include doc/test updates)

Need help? /capability-plan or https://github.com/Goldziher/ai-rulez
-->

# html-to-markdown

High-performance HTML→Markdown converter with Rust core and polyglot bindings (Python, TypeScript, Ruby, PHP). Focus on performance, accuracy, and language parity.

Version: 2.0.0

## Governance

- Source of truth: ai-rulez.yaml
- Generated output: GEMINI.md
- Update workflow:
  1. Edit the source configuration above.
  2. Run ai-rulez generate to refresh generated files.
  3. Commit the regenerated files alongside the configuration change.
- AI assistants must propose edits to the source configuration, not this file.

## Rules

### Code Quality with Prek
**Priority:** medium

**Prek pre-commit hooks · No bare commits without linting/formatting**

- Use prek (NOT pre-commit) for pre-commit hooks
- Configuration: .prek-config.yaml (hooks: ruff, clippy, cargo fmt, mypy, rubocop, etc.)
- Install: prek install && prek install --hook-type commit-msg
- Run: prek run --all-files to check everything before commit
- Rust: cargo fmt --all, clippy with -D warnings
- Python: ruff check, ruff format, mypy --strict
- Ruby: rubocop --config ./.rubocop.yml, rbs validate, steep check
- PHP: phpstan analyse, phpcs/phpcbf
- TypeScript: biome check, biome format
- Never: skip hooks (--no-verify); enforce quality gates


### Continuous Integration & Coverage
**Priority:** medium

**GitHub Actions · Multi-platform · Coverage enforcement**

- Main CI: .github/workflows/ci.yaml runs Rust, Python, Ruby, Node, TypeScript, WASM
- Linting & formatting: ruff, clippy, rubocop, biome, phpstan, golangci-lint
- Test matrix: Python (3.10, 3.12, 3.14-dev), PHP (8.2+), Rust (stable 1.75+)
- OS matrix: Linux (amd64, arm64), macOS, Windows (where applicable)
- Artifacts: Rust coverage → rust-coverage.lcov, Python → coverage.lcov
- Quality gates: zero warnings, tests pass, coverage thresholds (Rust 95%, others 80%)
- Wheel builds: separate test-wheels.yaml for PyPI distribution testing
- Version-gated: tag-based releases trigger multi-platform builds


### Dual Testing Strategy - Core + Bindings
**Priority:** medium

**Core logic: Rust tests (95%) · Bindings: Language-specific tests (80%+)**

- Rust core: crates/html-to-markdown tests → 95% coverage (cargo-llvm-cov)
- Python binding: packages/python/tests → 80%+ coverage (pytest-cov)
- TypeScript binding: packages/typescript/tests → 80%+ coverage (vitest)
- Ruby binding: packages/ruby/spec → 80%+ coverage (rspec)
- PHP binding: packages/php/tests → 80%+ coverage (phpunit)
- Go FFI: packages/go/htmltomarkdown (black-box tests with cgo bindings)
- Java FFI: packages/java (JNI tests with cargo build -p html-to-markdown-ffi)
- C#/.NET: packages/csharp (P/Invoke tests with html-to-markdown-ffi)
- Fixture-driven: JSON/YAML fixtures in examples/fixtures/ with schemas
- Parametrized tests: use language-native parametrization (@dataProvider, @ParameterizedTest, etc.)
- Real HTML samples: use actual HTML from fixtures/ not mocks
- Never: Rust tests that mock the entire binding layer; test bindings in their native language


### PHP 8.2+ - ext-php-rs Extension + Composer Package
**Priority:** medium

**PHP 8.2+ · ext-php-rs extension · PHPStan level 9 · PSR-12 · PHPUnit · 80%+ coverage**

- PHP 8.2+ with declare(strict_types=1); typed properties, union types, enums
- ext-php-rs extension crate (crates/html-to-markdown-php) compiled to Rust .so/.dll
- PIE packaging metadata in packages/php-ext for distribution
- Composer package in packages/php wraps the extension with typed interfaces
- PHPStan level 9; never suppress warnings
- PSR-12 code standards: phpcbf auto-fix, max 120 char lines, 4-space indent
- PHPUnit tests in packages/php/tests; ClassName → ClassNameTest, 80%+ coverage
- Composer: lock dependencies (composer.lock committed), ^version constraints
- Build flow: cargo build -p html-to-markdown-php --release → composer run test
- Never: business logic in PHP; all conversion logic lives in Rust
- Use Haiku 4.5 for PHP binding engineering and Composer issues


### Polyglot Build System & Distribution
**Priority:** medium

**Multi-language builds · Cargo + maturin + NAPI-RS + Magnus + ext-php-rs**

- Rust core: cargo build --workspace --release (excludes language binding crates in CI)
- Python: maturin via uv pip install -e packages/python
- TypeScript: pnpm run build in packages/typescript (after crates/html-to-markdown-node builds)
- Ruby: bundle exec rake compile then bundle exec rake package
- PHP: cargo build -p html-to-markdown-php --release; PIE metadata in packages/php-ext
- CLI binary: cargo build --release --package html-to-markdown-cli
- Version sync: Cargo.toml is source of truth; scripts/sync_versions.py propagates to all
- Lock files committed: Cargo.lock, pnpm-lock.yaml, Gemfile.lock, composer.lock
- Never: manual version bumps; use sync_versions.py


### Python 3.10+ - PyO3 Binding Wrappers
**Priority:** medium

**Python 3.10+ · PyO3 minimal wrappers · Type-safe · pytest · 80%+ coverage**

- Target Python 3.10+; match/case, union types (X | Y), structural pattern matching
- PyO3 bindings minimal wrappers: expose Rust API cleanly without logic duplication
- Type stubs in _rust.pyi provide type info for Rust bindings
- Full type hints: mypy --strict, no Any types, ParamSpec for decorators
- Testing: pytest in packages/python/tests; 80%+ coverage with pytest-cov
- Package distribution: PyPI via maturin (uv pip install -e packages/python)
- Never: business logic in Python wrappers; that belongs in Rust
- Use Haiku 4.5 for binding engineering and integration issues


### Ruby 3.2+ - Magnus Native Bindings with RBS
**Priority:** medium

**Ruby 3.2+ · Magnus native bindings · RBS type definitions · Steep · RSpec · 80%+ coverage**

- Ruby 3.2+ with .ruby-version file; rbenv for version management
- Magnus bindings expose Rust API cleanly; minimal Ruby wrapper logic
- RBS files in packages/ruby/sig/ parallel to source (lib/foo.rb → sig/foo.rbs)
- Steep for type checking; avoid Any types, use union and optional types explicitly
- RSpec in packages/ruby/spec/; describe/context/it blocks, 80%+ coverage
- Rubocop with auto-fix: line length ≤120, prefer &:method_name blocks
- Distribution: Ruby gem via bundle exec rake package
- Never: business logic duplication; binding code defers to Rust
- Use Haiku 4.5 for Ruby binding engineering and RBS issues


### Rust 2024 Edition - Core Conversion Engine
**Priority:** medium

**Rust 2024 edition · html5ever + ammonia · clippy -D warnings · 95% coverage**

- Rust 2024; cargo fmt, clippy with -D warnings (zero tolerance)
- Result<T, E> for errors; thiserror for custom errors; NEVER .unwrap() in production
- Testing: 95% minimum coverage (cargo-llvm-cov), unit/integration/doc tests in crates/
- Documentation: rustdoc on ALL public items with examples, SAFETY comments for unsafe
- Async: Tokio 1.x exclusively, 'static constraints, proper Send+Sync bounds
- Core libraries: html5ever (parsing), ammonia (sanitization), regex, encoding_rs
- Pre-commit: cargo fmt, clippy, test, coverage check
- Never: unwrap in production, unsafe without SAFETY docs, panics in library code
- Use Sonnet 4.5 for architectural decisions on the Rust core


### Task Automation & Workflow
**Priority:** medium

**Taskfile.yaml for all workflows · setup → build → test → lint**

- Taskfile.yaml primary interface for all development tasks
- task setup: install all language deps (Python, Ruby, PHP, JS, Go, Java, C#, Elixir)
- task build: compile Rust core + JavaScript bindings
- task test: run all language test suites (pytest, cargo test, rspec, vitest, phpunit)
- task lint: ruff + clippy + phpstan + rubocop + golangci-lint + more
- task format: ruff fix + cargo fmt + rubocop --autocorrect + biome + phpcbf
- task cov:all: generate Rust + Python coverage reports (lcov format)
- task bench: Rust benchmarks + PHP/Ruby binding harness via tools/runtime-bench
- Environment variables in Taskfile (RUST_LOG, RUBY_BIN, BUNDLER_VERSION)
- Never: manual commands instead of task tasks


### TypeScript 5.x - NAPI-RS Bindings & CLI
**Priority:** medium

**TypeScript 5.x · NAPI-RS native bindings · Strictest typing · vitest · 80%+ coverage**

- Enable ALL strict flags: strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes
- NAPI-RS bindings in crates/html-to-markdown-node, consumed by packages/typescript
- Ban any and object types; use unknown with guards, Record<string, unknown>
- Tests: .spec.ts next to source files (packages/typescript/tests); vitest, 80%+ coverage
- CLI wrapper with TypeScript commands; pnpm ≥10.17, pnpm-lock.yaml committed
- Biome for linting/formatting; import type for types, path aliases (@/lib/*)
- Never: any/object types, non-null assertions !, hardcoded CLI commands
- Use Haiku 4.5 for CLI engineering and TypeScript binding issues


## Sections

### Developer Quick Start
**Priority:** medium

## Prerequisites
- Rust 1.75+ (stable or nightly for WASM)
- Python 3.10+ with uv package manager
- Node.js 18+ with pnpm ≥10.17
- Ruby 3.2+ with rbenv
- PHP 8.2+ with Composer
- Go 1.25+ (optional, for Go binding)
- Java JDK 22+ (optional, for Java binding)
- .NET 8.0+ SDK (optional, for C# binding)
- Task (task runner)
- prek (pre-commit hook manager)

## Quick Setup
```bash
git clone https://github.com/Goldziher/html-to-markdown.git
cd html-to-markdown

# Install all dependencies
task setup

# Install pre-commit hooks
task pre-commit:install
```

## Running Tests
```bash
# All languages
task test

# Specific languages
task test:rust
task test:python
task test:ruby
task test:node
task test:ts
task test:js
task test:php
task test:go

# Coverage
task cov:rust
task cov:python
task cov:all
```

## Development Workflow
```bash
# Build everything
task build

# Format code
task format

# Lint everything
task lint

# Run benchmarks
task bench

# Update dependencies
task update
```

## Editing & Committing
1. Edit source files (Rust, Python, TypeScript, Ruby, PHP, etc.)
2. prek will auto-format on commit
3. If hooks reject, fix issues and retry git commit
4. Never use --no-verify; enforce code quality


### Git Commit Standards
**Priority:** medium

**Conventional Commits 1.0.0 · Pre-commit hooks enforce quality**

- Commit message format: feat/fix/docs/refactor/test/chore(scope): description
- Scopes: rust-core, py-binding, ts-binding, rb-binding, php-binding, build, ci, docs
- Example: fix(rust-core): handle nested lists in markdown output
- Example: feat(py-binding): expose sanitization options to Python API
- Example: test(ts-binding): add parametrized tests for edge cases
- Commits MUST pass prek hooks: fmt, lint, tests (at min)
- NEVER include AI signatures in commits; commits authored by humans
- NEVER force push to main/development; require PR reviews


### Model Routing & AI Guidance
**Priority:** medium

## When to Use Sonnet 4.5 vs Haiku 4.5

**Use Sonnet 4.5 (claude-sonnet-4-5-20250929) for:**
- Rust core architecture & design decisions
- Complex algorithm optimization
- Safety-critical code changes
- Cross-cutting refactoring
- Error handling strategy & thiserror design

**Use Haiku 4.5 (claude-haiku-4-5-20251001) for:**
- Python/TypeScript/Ruby/PHP binding engineering
- Test writing & fixture management
- CLI tool development
- Integration issues (build system, CI/CD)
- Dependency updates & version bumping

**Applies to all tasks in this project** unless explicitly overridden.


### Polyglot Binding Architecture
**Priority:** medium

## Language Binding Pattern
Each binding crate (Python, TypeScript, Ruby, PHP, Go, Java, C#) follows:
1. **Minimal wrapper layer**: Call Rust functions directly, no business logic
2. **Type translation**: Convert host language types ↔ Rust types
3. **Error mapping**: Rust errors → language-native exceptions
4. **Documentation**: Link bindings to Rust docs, add language-specific examples
5. **Testing**: Language-native test suite validating binding + integration

## Binding Crates (crates/)
- **html-to-markdown-py**: PyO3 bindings → packages/python distribution
- **html-to-markdown-node**: NAPI-RS bindings → packages/typescript npm package
- **html-to-markdown-rb**: Magnus bindings → packages/ruby gem (Ruby 3.2+)
- **html-to-markdown-php**: ext-php-rs extension → packages/php Composer package
- **html-to-markdown-wasm**: wasm-bindgen → browser + Wasmtime targets
- **html-to-markdown-ffi**: C-compatible FFI library → Go, Java, C# consumers
- **html-to-markdown-cli**: Standalone CLI using core library

## Distribution Packages (packages/)
- **python/**: PyPI package with Python wrappers + tests
- **typescript/**: npm package with TypeScript wrappers + CLI + tests
- **ruby/**: Ruby gem (RBS types in sig/, specs in spec/)
- **php/**: Composer package with PHP wrappers + PHPUnit tests
- **php-ext/**: PIE metadata for ext-php-rs distribution
- **go/**: Go module wrapping FFI library
- **java/**: Maven project wrapping FFI library with JNI
- **csharp/**: .NET project wrapping FFI library with P/Invoke


### Project Structure & Conventions
**Priority:** medium

```
html-to-markdown/
├── crates/
│   ├── html-to-markdown/           # Core Rust library (main logic)
│   ├── html-to-markdown-py/        # PyO3 bindings for Python
│   ├── html-to-markdown-node/      # NAPI-RS bindings for Node.js
│   ├── html-to-markdown-rb/        # Magnus bindings for Ruby
│   ├── html-to-markdown-php/       # ext-php-rs extension for PHP
│   ├── html-to-markdown-wasm/      # wasm-bindgen for WebAssembly
│   ├── html-to-markdown-ffi/       # C FFI library (Go, Java, C# wrapper)
│   ├── html-to-markdown-cli/       # Standalone CLI binary
│   └── Cargo.toml                  # Workspace manifest
│
├── packages/
│   ├── python/
│   │   ├── src/                    # Python package source
│   │   ├── tests/                  # pytest test suite
│   │   └── pyproject.toml
│   ├── typescript/
│   │   ├── src/                    # TypeScript source
│   │   ├── tests/                  # vitest test suite
│   │   └── package.json
│   ├── ruby/
│   │   ├── lib/                    # Ruby source
│   │   ├── sig/                    # RBS type definitions
│   │   ├── spec/                   # RSpec test suite
│   │   └── Gemfile
│   ├── php/
│   │   ├── src/                    # PHP source
│   │   ├── tests/                  # PHPUnit test suite
│   │   └── composer.json
│   ├── php-ext/                    # PIE packaging metadata
│   ├── go/                         # Go module wrapper
│   ├── java/                       # Maven Java project
│   └── csharp/                     # .NET C# project
│
├── examples/
│   ├── fixtures/                   # HTML test fixtures (JSON/YAML)
│   ├── node-smoke/                 # Node.js smoke tests
│   ├── ruby-smoke/                 # Ruby smoke tests
│   └── go-smoke/                   # Go smoke tests
│
├── tools/
│   ├── runtime-bench/              # PHP/Ruby binding benchmarks
│   └── ...
│
├── scripts/
│   ├── sync_versions.py            # Sync version across all manifests
│   ├── prepare_wheel.py            # Python wheel prep
│   ├── package_php_pie_source.sh   # PHP PIE packaging
│   └── preferred-rustc.sh          # WASM rustc selection
│
├── .github/workflows/
│   ├── ci.yaml                     # Main CI pipeline
│   └── test-wheels.yaml            # Python wheel testing
│
├── Taskfile.yaml                   # Task automation (PRIMARY DEV INTERFACE)
├── Cargo.toml                      # Cargo workspace
├── Cargo.lock                      # Lock file (committed)
├── pnpm-lock.yaml                  # pnpm lock file (committed)
├── .prek-config.yaml               # Prek pre-commit hooks
└── ai-rulez.yaml                   # This file (AI guidelines)
```


### Rust Core Architecture
**Priority:** medium

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


### Universal Anti-Patterns
**Priority:** medium

**Cross-language patterns to NEVER use:**
- Any type (Python, TypeScript, Rust unknown) without exhaustive matching
- Class-based tests (Python) – use function-based with pytest fixtures
- Unwrap/panic in production code (Rust) – use Result<T, E>
- Mocking internal services – use real objects/fixtures
- Manual dependency management – use lock files (Cargo.lock, pnpm-lock.yaml, etc.)
- Blocking I/O in async code (Python/TypeScript) – fully async paths
- Bare exception handlers – catch specific types only
- Magic numbers – extract to named constants
- Inheritance for code reuse – prefer composition
- Global state – dependency injection instead
- f-strings in logging – structured key=value logging
- Duplication across bindings – core logic ALWAYS in Rust


## MCP Servers

### ai-rulez
AI-Rulez MCP server for configuration management
- Transport: stdio
- Command: ai-rulez
- Args: mcp
