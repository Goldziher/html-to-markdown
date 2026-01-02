# Build System Architecture

## Overview

The build system orchestrates compilation, testing, and artifact generation across the polyglot ecosystem using a task-based approach. A single source of truth (Cargo.toml) drives version consistency across all 10 language bindings.

## Task-Based Orchestration (Taskfile.yaml)

### High-Level Task Hierarchy

```yaml
# Taskfile.yaml - Root task definitions
version: '3'

vars:
  RUST_VERSION: 1.75.0
  MSRV: 1.70.0

tasks:
  # Build tasks
  build:
    desc: Build all artifacts
    cmds:
      - task: build:core
      - task: build:bindings

  build:core:
    desc: Build Rust core library
    dir: src
    cmds:
      - cargo build --release

  build:bindings:
    desc: Build all language bindings
    cmds:
      - task: build:bindings:python
      - task: build:bindings:typescript
      - task: build:bindings:ruby
      - task: build:bindings:java
      - task: build:bindings:go
      - task: build:bindings:csharp
      - task: build:bindings:php
      - task: build:bindings:elixir
      - task: build:bindings:wasm

  build:bindings:python:
    desc: Build Python binding
    dir: bindings/python
    cmds:
      - cargo build --release --target-dir ../../target
      - maturin develop

  build:bindings:typescript:
    desc: Build TypeScript binding
    dir: bindings/typescript
    cmds:
      - cargo build --release --target-dir ../../target
      - npm run build

  # Test tasks
  test:
    desc: Run all tests
    cmds:
      - task: test:core
      - task: test:bindings
      - task: test:integration
      - task: test:coverage

  test:core:
    desc: Test Rust core (95% coverage required)
    dir: src
    cmds:
      - cargo test --all-features
      - cargo test --doc
      - cargo tarpaulin --out Html --output-dir ../../coverage

  test:bindings:
    desc: Test all language bindings
    cmds:
      - task: test:bindings:python
      - task: test:bindings:typescript

  test:bindings:python:
    desc: Test Python binding (80% coverage required)
    dir: bindings/python
    cmds:
      - pytest --cov=mylib --cov-report=html

  test:bindings:typescript:
    desc: Test TypeScript binding (80% coverage required)
    dir: bindings/typescript
    cmds:
      - npm test -- --coverage

  test:integration:
    desc: Integration tests across languages
    cmds:
      - cargo test --test integration_tests --all-features

  test:coverage:
    desc: Generate coverage reports
    cmds:
      - cargo tarpaulin --out Html --output-dir coverage
      - echo "Coverage report: coverage/index.html"

  # Benchmark tasks
  bench:
    desc: Run benchmarks
    cmds:
      - task: bench:core
      - task: bench:bindings

  bench:core:
    desc: Benchmark Rust core
    dir: src
    cmds:
      - cargo bench --all-features

  bench:bindings:
    desc: Benchmark binding overhead
    cmds:
      - task: bench:bindings:python
      - task: bench:bindings:typescript

  bench:bindings:python:
    desc: Benchmark Python binding
    dir: bindings/python
    cmds:
      - python -m pytest tests/bench --benchmark-cli

  # Release tasks
  release:
    desc: Publish release to all registries
    cmds:
      - task: release:prepare
      - task: release:publish:crates
      - task: release:publish:pypi
      - task: release:publish:npm
      - task: release:publish:rubygems
      - task: release:publish:maven
      - task: release:publish:docker

  release:prepare:
    desc: Prepare release (bump versions, changelog)
    cmds:
      - echo "Run: cargo release --execute"
      - echo "Update CHANGELOG.md"
      - echo "Create git tag"

  release:publish:crates:
    desc: Publish to crates.io
    dir: src
    cmds:
      - cargo publish --allow-dirty

  release:publish:pypi:
    desc: Publish to PyPI
    dir: bindings/python
    cmds:
      - maturin publish -b pyo3

  release:publish:npm:
    desc: Publish to npm
    dir: bindings/typescript
    cmds:
      - npm publish

  # Cleanup tasks
  clean:
    desc: Clean build artifacts
    cmds:
      - cargo clean
      - rm -rf target/

  clean:bindings:
    desc: Clean binding artifacts
    cmds:
      - task: clean:bindings:python
      - task: clean:bindings:typescript

  clean:bindings:python:
    dir: bindings/python
    cmds:
      - rm -rf dist/ build/ *.egg-info .tox

  clean:bindings:typescript:
    dir: bindings/typescript
    cmds:
      - rm -rf dist/ node_modules/

  # Development tasks
  dev:
    desc: Setup development environment
    cmds:
      - task: dev:install
      - task: dev:hooks

  dev:install:
    desc: Install development dependencies
    cmds:
      - rustup toolchain install {{.RUST_VERSION}}
      - rustup component add rustfmt clippy
      - cargo install cargo-tarpaulin
      - cargo install cargo-deny

  dev:hooks:
    desc: Install git hooks
    cmds:
      - echo "Installing pre-commit hooks"
      - cp scripts/hooks/pre-commit .git/hooks/pre-commit
      - chmod +x .git/hooks/pre-commit

  # Quality tasks
  lint:
    desc: Run all linters
    cmds:
      - task: lint:rust
      - task: lint:bindings

  lint:rust:
    desc: Lint Rust code
    cmds:
      - cargo fmt --check
      - cargo clippy --all-targets --all-features -- -D warnings

  lint:bindings:
    desc: Lint binding code
    cmds:
      - task: lint:bindings:python
      - task: lint:bindings:typescript

  lint:bindings:python:
    desc: Lint Python binding
    dir: bindings/python
    cmds:
      - black --check .
      - flake8 .
      - mypy . --strict

  lint:bindings:typescript:
    desc: Lint TypeScript binding
    dir: bindings/typescript
    cmds:
      - npm run lint
      - npm run type-check

  fmt:
    desc: Format all code
    cmds:
      - cargo fmt --all
      - task: fmt:bindings

  fmt:bindings:
    desc: Format binding code
    cmds:
      - task: fmt:bindings:python
      - task: fmt:bindings:typescript

  fmt:bindings:python:
    dir: bindings/python
    cmds:
      - black .

  fmt:bindings:typescript:
    dir: bindings/typescript
    cmds:
      - npm run format

  # CI/CD integration
  ci:
    desc: Run full CI pipeline
    cmds:
      - task: lint
      - task: build
      - task: test
      - task: bench

  ci:pr:
    desc: Run PR validation pipeline
    cmds:
      - task: lint
      - task: build:core
      - task: test:core
```

## Cross-Language Build Coordination

### Dependency Version Source of Truth

```toml
# src/Cargo.toml - Authoritative version definitions
[package]
name = "mylib"
version = "0.5.0"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

All bindings reference this version:

```python
# bindings/python/Cargo.toml
[dependencies]
mylib = { path = "../../src", version = "0.5.0" }
tokio = { version = "1.35", features = ["full"] }
pyo3 = { version = "0.20", features = ["extension-module"] }
```

```toml
# bindings/typescript/Cargo.toml
[dependencies]
mylib = { path = "../../src", version = "0.5.0" }
tokio = { version = "1.35", features = ["full"] }
napi = { version = "2.14", features = ["tokio_rt"] }
```

### Coordinated Build Profiles

```toml
# Cargo.toml - Build profiles synchronized across workspace

[profile.dev]
opt-level = 0
debug = true
debug-assertions = true
overflow-checks = true
lto = false
panic = 'unwind'
incremental = true

[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = "thin"
panic = 'abort'
codegen-units = 16

[profile.bench]
inherits = "release"
debug = true
debug-assertions = false

[profile.ci]
inherits = "release"
lto = "fat"
codegen-units = 1
```

## Build Profiles (dev, ci, release)

### Development Profile

```bash
task build:core          # Unoptimized, full debug symbols
task test:core           # Run core tests locally
task lint                # Check code style
```

**Characteristics:**

- Fast compile time (incremental)
- Full debug symbols
- Debug assertions enabled
- Optimized for iteration

### CI Profile

```bash
task ci                  # Full CI pipeline
```

**Pipeline:**

1. **Lint**: Code style and safety checks
1. **Build**: With optimizations (release profile)
1. **Test**: Full test suite (95% core, 80% bindings)
1. **Bench**: Performance regression detection
1. **Coverage**: Upload to coverage service

**Configuration:**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: task ci

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: task test:coverage
      - uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
```

### Release Profile

```bash
task release             # Full release process
```

**Steps:**

1. **Version Bump**: Update Cargo.toml versions
1. **Changelog**: Generate from git log
1. **Build**: Release-optimized builds
1. **Tag**: Create git tag
1. **Publish**: To all registries simultaneously

**Registry Publishing:**

```bash
# Rust crates registry
cargo publish --token $CARGO_REGISTRY_TOKEN

# Python PyPI
maturin publish --token $PYPI_TOKEN

# Node npm registry
npm publish --access public

# Ruby Rubygems
gem push mylib-*.gem --api-key $RUBYGEMS_API_KEY

# Java Maven Central
mvn deploy -Dorg.slf4j.simpleLogger.defaultLogLevel=warn
```

## Artifact Generation and Caching

### Build Artifact Structure

```
target/
├── release/
│   ├── libmylib.a          # Static lib (Unix)
│   ├── libmylib.so         # Shared lib (Linux)
│   ├── libmylib.dylib      # Shared lib (macOS)
│   ├── mylib.dll           # DLL (Windows)
│   └── mylib.lib           # Import lib (Windows)
├── wheels/
│   ├── mylib-0.5.0-cp312-cp312-linux_x86_64.whl
│   ├── mylib-0.5.0-cp312-cp312-macosx_10_9_x86_64.whl
│   └── mylib-0.5.0-cp312-cp312-win_amd64.whl
├── npm/
│   └── mylib-0.5.0.tgz
└── coverage/
    ├── index.html
    └── cobertura.xml
```

### Caching Strategy

```yaml
# .github/workflows/cache.yml
name: Build Cache

jobs:
  cache:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Cache Rust dependencies
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: |
            src
            bindings/*

      # Cache pip dependencies
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          cache: 'pip'

      # Cache npm dependencies
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: 'bindings/typescript/package-lock.json'
```

### Build Matrix Strategy

```yaml
# Matrix builds for multiple configurations
jobs:
  test:
    strategy:
      matrix:
        rust: ['1.70.0', 'stable', 'nightly']  # MSRV + stable + nightly
        os: [ubuntu-latest, macos-latest, windows-latest]
        python: ['3.8', '3.9', '3.10', '3.11', '3.12']

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python }}
      - run: task build
      - run: task test
```

## MSRV (Minimum Supported Rust Version)

### MSRV Policy

- **Current MSRV**: 1.70.0 (defined in Cargo.toml)
- **Deprecation**: Versions older than 6 months lose support
- **Testing**: CI tests against MSRV and latest stable

```toml
# Cargo.toml
[package]
rust-version = "1.70.0"
```

```bash
# Test MSRV locally
rustup install 1.70.0
rustup target add x86_64-unknown-linux-gnu --toolchain 1.70.0
cargo +1.70.0 test --all-features
```

### Stable Feature Detection

```rust
// Use #[cfg(feature = "...")] for newer features
#[cfg(feature = "async_closure")]
pub fn use_async_closure() { /* ... */ }

// Graceful degradation for MSRV
#[cfg(not(feature = "nightly"))]
pub fn fallback_implementation() { /* ... */ }
```

## Cross-References

- **Rust Core Design**: See [02-rust-core-design.md](02-rust-core-design.md)
- **Binding Patterns**: See [03-binding-patterns.md](03-binding-patterns.md)
- **Testing Strategy**: See [05-testing-strategy.md](05-testing-strategy.md)
- **Performance Patterns**: See [06-performance-patterns.md](06-performance-patterns.md)
- **Security Model**: See [07-security-model.md](07-security-model.md)
- **Dependency Management**: See [08-dependency-management.md](08-dependency-management.md)

## Implementation Checklist

- [ ] Taskfile.yaml covers all build scenarios
- [ ] MSRV tested in CI for all platforms
- [ ] Build cache configured for faster CI
- [ ] Artifact caching reduces redundant builds
- [ ] Release process fully automated
- [ ] All registries receive simultaneous updates
- [ ] Build matrix covers MSRV + latest stable
- [ ] Dependency versions centralized in src/Cargo.toml
