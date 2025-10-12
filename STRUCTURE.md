# Repository Structure

This is a **monorepo** containing the html-to-markdown converter with multiple language bindings and distribution formats.

## Directory Organization

```text
html-to-markdown/
├── pnpm-workspace.yaml             # pnpm workspace configuration
├── package.json                    # Root workspace scripts
├── Cargo.toml                      # Rust workspace configuration
│
├── crates/                         # Rust crates
│   ├── html-to-markdown/           # Core conversion library (Rust)
│   ├── html-to-markdown-cli/       # Command-line interface
│   ├── html-to-markdown-node/      # NAPI-RS bindings for Node.js
│   └── html-to-markdown-wasm/      # wasm-bindgen for browsers
│
├── packages/                       # Releasable packages
│   ├── html-to-markdown-py/        # PyO3 Python bindings (PyPI)
│   └── html-to-markdown-ts/        # TypeScript package with CLI (npm)
│
├── html_to_markdown/               # Python API
├── examples/                       # Usage examples
├── tests/                          # Integration tests (Python)
└── scripts/                        # Build and release scripts
```

## Workspace Configuration

### pnpm Workspace

The repository uses pnpm workspaces for JavaScript/TypeScript packages. The `pnpm-workspace.yaml` configures:

- `crates/html-to-markdown-node` - Native bindings
- `crates/html-to-markdown-wasm` - WASM bindings
- `packages/*` - Releasable npm packages

### Cargo Workspace

Rust components are managed via Cargo workspace in root `Cargo.toml`:

- Core library
- CLI binary
- Node.js bindings crate
- WASM bindings crate
- Python bindings crate

## Components

### Core Library (`crates/html-to-markdown`)

- Pure Rust implementation
- HTML parsing with `tl` (faster than html5ever for this use case)
- Sanitization with `ammonia`
- Zero JavaScript dependencies

### CLI (`crates/html-to-markdown-cli`)

- Standalone binary
- Built with `clap`
- Cross-platform distribution

### Node.js Native (`crates/html-to-markdown-node`)

- **Technology**: NAPI-RS v3
- **Performance**: ~2x faster than WASM
- **Platform support**:
  - macOS (x64, ARM64)
  - Linux (x64 gnu/musl, ARM64 gnu/musl, ARMv7)
  - Windows (x64, ARM64)
- **Package**: `@html-to-markdown/node`

### WebAssembly (`crates/html-to-markdown-wasm`)

- **Technology**: wasm-bindgen
- **Targets**:
  - `bundler` (Webpack, Vite, etc.)
  - `nodejs` (Node.js/Deno)
  - `web` (Browser ESM)
- **Package**: `@html-to-markdown/wasm`

### Python Bindings (`packages/html-to-markdown-py`)

- **Technology**: PyO3 with abi3 (Python 3.10+)
- **Distribution**: PyPI via maturin
- **Package**: `html-to-markdown`

### TypeScript Package (`packages/html-to-markdown-ts`)

- **Smart fallback**: Native → WASM
- **Full TypeScript types**
- **Dual package**: CJS + ESM
- **Includes CLI**: `html2md` and `html-to-markdown` commands
- **Package**: `html-to-markdown`

## Build Tools

- **Rust**: `cargo` (1.80+)
- **Python**: `maturin` via `uv`
- **Node.js**: `@napi-rs/cli` for native, `wasm-pack` for WASM
- **TypeScript**: `tsup` for bundling
- **Task runner**: `task` (Taskfile.yaml)

## Performance Comparison

Based on typical workloads:

1. **Native Node.js** (NAPI-RS): ~691k ops/sec ⚡ **Fastest**
1. **Rust binary/Python**: ~500-600k ops/sec
1. **WASM**: ~229k ops/sec (still fast, universal)
1. **Pure JavaScript**: ~276k ops/sec

## Package Distribution

### npm Packages

- `html-to-markdown` - Main package (smart native/WASM)
- `@html-to-markdown/node` - Native bindings only
- `@html-to-markdown/wasm` - WASM only

### PyPI

- `html-to-markdown` - Python package

### Cargo

- `html-to-markdown-rs` - Core Rust library

### Standalone Binaries

- GitHub Releases (CLI for all platforms)
