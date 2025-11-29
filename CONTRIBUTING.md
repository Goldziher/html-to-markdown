# Contributing to html-to-markdown

## Prerequisites

### Core Development

- **Python** 3.10+
- **Rust** 1.80+ (stable)
- **uv** - Python package manager ([install](https://docs.astral.sh/uv/))
- **Task** - Task runner ([install](https://taskfile.dev/))
- **prek** - Pre-commit hooks (`uv tool install prek`)

### JavaScript/TypeScript Development (Optional)

- **Node.js** 18+
- **pnpm** 8+ - Fast package manager ([install](https://pnpm.io/installation))
- **wasm-pack** - For WASM builds (`cargo install wasm-pack`)

## Quick Setup

```bash
# Clone repository
git clone https://github.com/Goldziher/html-to-markdown.git
cd html-to-markdown

# Setup environment (installs deps, builds Rust, installs hooks)
task setup
```

This will:

1. Install Python dependencies with `uv sync`
1. Build Rust extension with maturin
1. Install prek hooks for commit linting and code quality

## Development Workflow

### Running Tests

#### Python & Rust

```bash
# Python tests
task test:python

# Rust tests
task test:rust

# All Python + Rust tests
task test

# With coverage
task cov:all
```

#### JavaScript/TypeScript

```bash
# Install dependencies first
pnpm install

# All JavaScript tests
pnpm test

# Specific packages
pnpm run test:node      # NAPI-RS bindings
pnpm run test:wasm      # WebAssembly bindings
pnpm run test:ts        # TypeScript package

# Watch mode
cd packages/typescript
pnpm test:watch

# With coverage
cd packages/typescript
pnpm test -- --coverage
```

### Code Quality

#### Python & Rust

```bash
# Format code (Rust + Python)
task format

# Run all linters
task lint

# Build Rust components
task build
```

#### JavaScript/TypeScript

```bash
# Type checking
pnpm run typecheck

# Build all packages
pnpm run build

# Build specific targets
pnpm run build:node     # NAPI-RS native bindings
pnpm run build:wasm     # WebAssembly (all 3 targets)
pnpm run build:ts       # TypeScript wrapper

# Clean build artifacts
pnpm run clean
```

### Benchmarking

```bash
# Quick benchmarks
task bench

# All benchmarks
task bench:all
```

## Project Structure

This is a **monorepo** containing multiple language bindings and distributions:

```text
html-to-markdown/
├── pnpm-workspace.yaml         # pnpm workspace configuration
├── package.json                # Root workspace scripts
│
├── crates/                     # Rust crates
│   ├── html-to-markdown/       # Core library (astral-tl parser)
│   ├── html-to-markdown-cli/   # Rust CLI binary
│   ├── html-to-markdown-node/  # NAPI-RS bindings for Node.js (~691k ops/sec)
│   ├── html-to-markdown-wasm/  # wasm-bindgen for browsers (~229k ops/sec)
│   └── html-to-markdown-py/    # PyO3 bindings powering the Python package
│
├── packages/                   # Releasable packages
│   ├── python/                 # PyPI package (html_to_markdown)
│   │   ├── html_to_markdown/   # Python sources
│   │   └── tests/             # Python integration + unit tests
│   ├── typescript/             # TypeScript package with CLI (npm)
│   └── ruby/                   # Ruby gem sources/specs (RubyGems)
│
└── scripts/                    # Helper scripts (wheel prep, gem prep, demo)
```

### Package Distribution

| Package                  | Registry  | Description                 |
| ------------------------ | --------- | --------------------------- |
| `html-to-markdown-rs`    | crates.io | Core Rust library           |
| `html-to-markdown`       | PyPI      | Python package              |
| `html-to-markdown`       | npm       | TypeScript package with CLI |
| `html-to-markdown`       | RubyGems  | Ruby gem (Magnus bindings)  |
| `html-to-markdown-node` | npm       | Native Node.js bindings     |
| `html-to-markdown-wasm` | npm       | WebAssembly bindings        |

## Making Changes

### Rust Core Changes

1. Edit code in `crates/html-to-markdown/src/`
1. Run Rust tests: `task test:rust` or `cargo test`
1. Rebuild bindings:
    - Python: `task build`
    - Node.js: `cd crates/html-to-markdown-node && pnpm run build`
    - WASM: `cd crates/html-to-markdown-wasm && pnpm run build:all`
1. Run integration tests: `task test:python` or `pnpm test`

### Python API Changes

1. Edit code in `packages/python/html_to_markdown/`
1. Update type stubs in `_rust.pyi` if needed
1. Run tests: `task test:python`

### JavaScript/TypeScript Changes

#### Node.js Bindings (`crates/html-to-markdown-node`)

1. Edit Rust code in `src/lib.rs`
1. Rebuild: `pnpm run build` (generates TypeScript types automatically)
1. Test: `pnpm test` or `cargo test`

#### WASM Bindings (`crates/html-to-markdown-wasm`)

1. Edit Rust code in `src/lib.rs`
1. Rebuild: `pnpm run build:all` (builds for bundler, nodejs, and web)
1. Test: `pnpm test` or `cargo test`

#### TypeScript Package with CLI (`packages/typescript`)

1. Edit code in `src/` (library entrypoints + CLI)
1. Build: `pnpm run build` (runs Node binding build + TypeScript emit)
1. Lint: `pnpm run lint`
1. Test: `pnpm test` or `pnpm test:watch`
1. Test CLI locally: `node dist/cli.js input.html`

#### Ruby Gem (`packages/ruby`)

1. Edit Ruby sources in `lib/` and specs in `spec/`
1. Build native extension: `bundle exec rake compile`
1. Run specs: `bundle exec rake spec`

### Adding Tests

- **Rust tests**: Add to `crates/*/src/lib.rs` or `crates/*/tests/`
- **Python tests**: Add to `packages/python/tests/` following pytest patterns
- **TypeScript tests**: Add to `packages/typescript/tests/` using vitest
- **Ruby specs**: Add to `packages/ruby/spec/`
- **Integration tests**: Add to appropriate test directory

## Testing

### Test Without Releasing

To test wheels and binaries without creating a release:

```bash
# Test wheel building manually
gh workflow run "Test Wheel Building"

# Or manually build locally
pip install cibuildwheel
cibuildwheel --output-dir wheelhouse

# Test CLI binary locally
cargo build --release --package html-to-markdown-cli
./target/release/html-to-markdown --version
```

### CI Workflows

- **ci-*.yaml**: Kreuzberg-style, path-filtered workflows (rust, python, node, wasm, ruby, php, go, java, elixir, smoke, validate)
- **test-wheels.yaml**: Builds and tests wheels (manual or on Rust/config changes)
- All workflows must pass before merging

## Commit Guidelines

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
feat: add new feature
fix: fix bug
docs: update documentation
refactor: refactor code
test: add tests
```

Prek enforces this automatically via commitlint hook.

## Code Quality Standards

### Python

- **Formatting**: ruff (120 char line length)
- **Linting**: ruff with ALL rules enabled (see pyproject.toml for ignores)
- **Type checking**: mypy in strict mode

### Rust

- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy` with `-D warnings`
- **Style**: Follow standard Rust conventions
- **Tests**: Required for all public APIs

### TypeScript

- **Formatting**: Prettier via tsup
- **Type checking**: TypeScript 5.6+ in strict mode
- **Linting**: ESLint (when configured)
- **Tests**: vitest with coverage reporting
- **Style**: 2-space indentation, trailing commas

All Python/Rust checks run automatically via prek on commit.

## Pull Requests

1. Fork the repository
1. Create a feature branch (`git checkout -b feat/amazing-feature`)
1. Make your changes
1. Run `task test` and `task lint`
1. Commit with conventional commit format
1. Push and create a pull request

## Release Process (Maintainers Only)

### Pre-release Checklist

1. **Update versions** in:

    - `Cargo.toml` (workspace.package.version)
    - `packages/*/package.json`
    - `crates/html-to-markdown-node/package.json`
    - `crates/html-to-markdown-wasm/package.json`

    ```toml
    # Cargo.toml
    [workspace.package]
    version = "2.4.2"
    ```

    ```json
    // package.json files
    "version": "2.4.2"
    ```

1. Update `CHANGELOG.md` with changes

1. Run full test suite:

    ```bash
    task test           # Python + Rust
    pnpm test          # JavaScript/TypeScript
    ```

1. Build and verify all targets:

    ```bash
    task build:cli && ./target/release/html-to-markdown --version
    pnpm run build     # All JS/TS packages
    ```

1. Commit changes: `git commit -m "chore: bump version to 2.4.2"`

### Creating a Release

1. **Create and push tag**:

    ```bash
    git tag -a v2.4.2 -m "Release v2.4.2"
    git push origin v2.4.2
    ```

1. **Automated workflows trigger**:

    - `release.yml` - GitHub release with CLI binaries
    - `release-homebrew.yml` - Updates Homebrew formula
    - `publish-cargo.yml` - Publishes to crates.io
    - `release.yaml` - Publishes Python to PyPI
    - Manual npm publish required (see below)

1. **Publish npm packages** (manual):

    ```bash
    # Login to npm (once)
    npm login

    # Publish main TypeScript package (includes CLI)
    cd packages/typescript
    pnpm publish

    # Publish native bindings (with pre-built binaries)
    cd ../../crates/html-to-markdown-node
    pnpm run build
    pnpm publish

    # Publish WASM
    cd ../html-to-markdown-wasm
    pnpm run build:all
    pnpm publish
    ```

1. **Required secrets** (already configured):

    - `CARGO_TOKEN` - From <https://crates.io/settings/tokens>
    - `HOMEBREW_TOKEN` - GitHub token with `repo` scope
    - `PYPI_TOKEN` - PyPI trusted publishing
    - `NPM_TOKEN` - From <https://www.npmjs.com/settings/tokens> (automation token)

### Post-release Verification

Verify all distributions are published:

- **Rust**: <https://crates.io/crates/html-to-markdown-rs>
- **Python**: <https://pypi.org/project/html-to-markdown/>
- **npm (main)**: <https://www.npmjs.com/package/html-to-markdown>
- **npm (node)**: <https://www.npmjs.com/package/html-to-markdown-node>
- **npm (wasm)**: <https://www.npmjs.com/package/html-to-markdown-wasm>
- **Homebrew**: <https://github.com/Goldziher/homebrew-tap>
- **GitHub**: <https://github.com/Goldziher/html-to-markdown/releases>

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/Goldziher/html-to-markdown/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Goldziher/html-to-markdown/discussions)
- **Discord**: [Kreuzberg Community](https://discord.gg/pXxagNK2zN)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
