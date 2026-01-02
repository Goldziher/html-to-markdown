______________________________________________________________________

## priority: critical

# Common Task Commands

**Setup & Installation**:

- `task setup`: Install all dependencies (Rust, Python, Node, Go, Java, Ruby, etc.)
- `task setup-pre-commit`: Configure pre-commit hooks

**Build Commands**:

- `task build`: Build all (respects BUILD_PROFILE)
- `task build:all`: Build all languages
- `task build:all:dev`: Build all in debug mode
- `task build:all:release`: Build all in release mode
- `task rust:build`: Build Rust core (respects BUILD_PROFILE)
- `task rust:build:dev`: Build Rust in debug mode
- `task rust:build:release`: Build Rust in release mode
- `task python:build`: Build Python bindings (maturin)
- `task node:build`: Build TypeScript/Node bindings (NAPI-RS)
- `task go:build`: Build Go bindings
- `task java:build`: Build Java bindings (Maven)
- `task ruby:build`: Compile Ruby native extensions
- `task csharp:build`: Build C# bindings
- `task wasm:build`: Build WebAssembly bindings

**Test Commands**:

- `task test`: Run tests (respects BUILD_PROFILE)
- `task test:all`: Run all tests across all languages
- `task test:all:fast`: Run fast tests (skip slow integration tests)
- `task rust:test`: Run Rust tests
- `task python:test`: Run Python tests (pytest)
- `task node:test`: Run TypeScript tests (vitest)
- `task go:test`: Run Go tests
- `task java:test`: Run Java tests (Maven)
- `task ruby:test`: Run Ruby tests (RSpec)
- `task e2e`: Run E2E tests
- `task e2e:all`: Run all E2E tests across all languages

**Linting & Formatting**:

- `task lint`: Lint current project
- `task lint:all`: Lint all languages
- `task lint:check`: CI linting (for GitHub Actions, fails on issues)
- `task format`: Format code (auto-fixes)
- `task format:check`: Check formatting (fails if needs formatting)
- `task rust:fmt`: Format Rust (cargo fmt)
- `task rust:clippy`: Lint Rust (cargo clippy)
- `task python:lint`: Lint Python (ruff, mypy)
- `task python:format`: Format Python (ruff, black)
- `task node:lint`: Lint TypeScript (biome)
- `task node:format`: Format TypeScript (biome)

**Utilities**:

- `task clean`: Clean build artifacts
- `task version:sync`: Sync version from Cargo.toml to all manifests (package.json, pyproject.toml, pom.xml, go.mod, Gemfile, etc.)
- `task pre-commit`: Run pre-commit hooks manually
- `task pdfium:install`: Download and install PDFium library
- `task smoke`: Run smoke tests (quick validation)
