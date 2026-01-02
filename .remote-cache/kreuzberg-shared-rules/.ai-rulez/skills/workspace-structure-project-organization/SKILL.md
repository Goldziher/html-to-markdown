______________________________________________________________________

## priority: high

# Workspace Structure & Project Organization

**Rust workspace** (Cargo.toml): crates/{kreuzberg,kreuzberg-py,kreuzberg-node,kreuzberg-ffi,kreuzberg-cli}, packages/ruby/ext/kreuzberg_rb/native, tools/{benchmark-harness,e2e-generator}, e2e/{rust,go}.

**Language packages**: packages/{python,typescript,ruby,java,go} - thin wrappers around Rust core.

**E2E tests**: Auto-generated from fixtures/ via tools/e2e-generator. Located in e2e/{rust,python,typescript,ruby,java,go}.

**Benchmarking**: Rust harness in tools/benchmark-harness.

**Install**: `uv sync --all-extras --all-packages --all-groups`.

**Version sync**: `task sync-versions` (syncs from Cargo.toml to all manifests).
