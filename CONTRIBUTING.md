# Contributing to html-to-markdown

## Prerequisites

- **Python** 3.10+
- **Rust** 1.75+ (stable)
- **uv** - Python package manager ([install](https://docs.astral.sh/uv/))
- **Task** - Task runner ([install](https://taskfile.dev/))
- **prek** - Pre-commit hooks (`uv tool install prek`)

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

```bash
# Python tests
task test:python

# Rust tests
task test:rust

# All tests
task test

# With coverage
task cov:all
```

### Code Quality

```bash
# Format code (Rust + Python)
task fmt

# Run all linters
task lint

# Build Rust components
task build
```

### Benchmarking

```bash
# Quick benchmarks
task bench

# All benchmarks
task bench:all
```

## Project Structure

```text
html-to-markdown/
├── crates/
│   ├── html-to-markdown/       # Core Rust library (html5ever + ammonia)
│   ├── html-to-markdown-py/    # Python bindings (PyO3)
│   └── html-to-markdown-cli/   # Native CLI binary
├── html_to_markdown/
│   ├── api.py                  # V2 Python API
│   ├── options.py              # Configuration dataclasses
│   ├── v1_compat.py           # V1 compatibility layer
│   ├── cli_proxy.py           # CLI argument translation
│   └── _rust.pyi              # Type stubs
└── tests/                      # 700+ tests
```

## Making Changes

### Rust Core Changes

1. Edit code in `crates/html-to-markdown/src/`
1. Run Rust tests: `task test:rust`
1. Rebuild Python bindings: `task build`
1. Run Python integration tests: `task test:python`

### Python API Changes

1. Edit code in `html_to_markdown/`
1. Update type stubs in `_rust.pyi` if needed
1. Run tests: `task test:python`

### Adding Tests

- **Rust tests**: Add to `crates/html-to-markdown/src/` (inline) or `crates/html-to-markdown/tests/`
- **Python tests**: Add to `tests/` following existing patterns

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

- **ci.yaml**: Runs on every PR and push to main (tests, validation, coverage)
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

All checks run automatically via prek on commit.

## Pull Requests

1. Fork the repository
1. Create a feature branch (`git checkout -b feat/amazing-feature`)
1. Make your changes
1. Run `task test` and `task lint`
1. Commit with conventional commit format
1. Push and create a pull request

## Release Process (Maintainers Only)

### Pre-release Checklist

1. Update version in `Cargo.toml`:

    ```toml
    [workspace.package]
    version = "2.1.0"
    ```

1. Update `CHANGELOG.md` with changes

1. Run full test suite: `task test`

1. Build CLI locally: `task build:cli && ./target/release/html-to-markdown --version`

1. Commit changes: `git commit -m "chore: bump version to 2.1.0"`

### Creating a Release

1. **Create and push tag**:

    ```bash
    git tag -a v2.1.0 -m "Release v2.1.0"
    git push origin v2.1.0
    ```

1. **Automated workflows trigger**:

    - `release.yml` - Creates GitHub release with CLI binaries
    - `release-homebrew.yml` - Updates Homebrew tap formula
    - `publish-cargo.yml` - Publishes to crates.io
    - `release.yaml` - Publishes Python package to PyPI

1. **Required secrets** (already configured):

    - `CARGO_TOKEN` - From <https://crates.io/settings/tokens>
    - `HOMEBREW_TOKEN` - GitHub token with `repo` scope
    - `PYPI_TOKEN` - Configured via PyPI trusted publishing

### Post-release Verification

- Cargo: <https://crates.io/crates/html-to-markdown>
- PyPI: <https://pypi.org/project/html-to-markdown/>
- Homebrew: <https://github.com/Goldziher/homebrew-tap>
- GitHub: <https://github.com/Goldziher/html-to-markdown/releases>

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/Goldziher/html-to-markdown/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Goldziher/html-to-markdown/discussions)
- **Discord**: [Kreuzberg Community](https://discord.gg/pXxagNK2zN)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
