______________________________________________________________________

## priority: critical

# Workspace Dependency Management

## Cargo Workspace Fundamentals

A workspace coordinates multiple crates under unified configuration. This is **critical** for polyglot projects with core library + language bindings.

**workspace/Cargo.toml**:

```toml
[workspace]
members = [
    "crates/html-to-markdown",           # Core library
    "crates/html-to-markdown-py",        # PyO3 bindings
    "crates/html-to-markdown-node",      # NAPI-RS bindings
    "crates/html-to-markdown-rb",        # Magnus bindings
    "crates/html-to-markdown-php",       # PHP extension
    "crates/html-to-markdown-wasm",      # WebAssembly
    "crates/html-to-markdown-ffi",       # C FFI library
    "crates/html-to-markdown-cli",       # CLI binary
]

resolver = "2"  # Always use v2 for modern dependency resolution

[workspace.package]
version = "0.5.0"              # Single source of truth
authors = ["Team"]
edition = "2021"
rust-version = "1.70"
```

## Version Synchronization

**Golden Rule**: Core library and all bindings must have the same version number.

**Problem**: Manual version updates across 8+ Cargo.toml files leads to inconsistency.

**Solution**: Use workspace.package version inheritance + sync script.

**In each crate's Cargo.toml**:

```toml
[package]
name = "html-to-markdown-py"
version.workspace = true       # Inherit from workspace
authors.workspace = true
edition.workspace = true
rust-version.workspace = true
```

**Version sync script** (scripts/sync_versions.py):

```python
#!/usr/bin/env python3
import tomllib
import toml
from pathlib import Path

def sync_versions(workspace_root: Path, new_version: str):
    """Sync version across all crates in workspace"""
    workspace_toml = workspace_root / "Cargo.toml"

    # Update workspace version
    with open(workspace_toml, "rb") as f:
        data = tomllib.load(f)
    data["workspace"]["package"]["version"] = new_version
    with open(workspace_toml, "w") as f:
        toml.dump(data, f)

    # Update all crates (non-workspace members)
    for crate_dir in (workspace_root / "crates").iterdir():
        if crate_dir.is_dir():
            crate_toml = crate_dir / "Cargo.toml"
            if crate_toml.exists():
                with open(crate_toml, "rb") as f:
                    data = tomllib.load(f)
                if "version" in data.get("package", {}):
                    data["package"]["version"] = new_version
                with open(crate_toml, "w") as f:
                    toml.dump(data, f)

    print(f"Synced all crates to version {new_version}")

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: sync_versions.py <new_version>")
        sys.exit(1)
    sync_versions(Path.cwd(), sys.argv[1])
```

**Usage**:

```bash
./scripts/sync_versions.py 0.6.0
cargo update -w  # Update workspace lockfile
git add -A && git commit -m "chore: bump version to 0.6.0"
```

## Path Dependencies for Core Library

Binding crates depend on core via path dependency:

```toml
# crates/html-to-markdown-py/Cargo.toml
[dependencies]
html-to-markdown = { path = "../html-to-markdown", version = "0.5.0" }
pyo3 = { version = "0.20", features = ["extension-module"] }
```

**Why version constraint + path?**

- Path ensures local development uses local code
- Version constraint ensures semver is respected if/when published to crates.io

## MSRV Policy (Minimum Supported Rust Version)

**Define in workspace.package**:

```toml
[workspace.package]
rust-version = "1.70"
```

**Update workflow**:

1. Update `rust-version` in workspace Cargo.toml
1. Add CI check to test MSRV
1. Run: `cargo +1.70 test` to verify

**CI workflow for MSRV**:

```yaml
- name: Test MSRV
  run: |
    rustup install 1.70
    cargo +1.70 test --all-features
```

## Dependency Constraints Best Practices

**Be explicit with version ranges**:

```toml
# BAD: Too permissive
pyo3 = "*"
tokio = "1"

# GOOD: Explicit ranges
pyo3 = "0.20"          # Patch updates OK
tokio = "1.35"         # Patch updates OK (1.35.x)
thiserror = "1.0"      # Conservative

# Exact versions for unstable features
napi = "= 2.13.0"
```

## Shared Dependencies

Prevent duplicate dependency trees by centralizing versions:

```toml
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
```

**Each crate imports from workspace**:

```toml
[dependencies]
tokio = { workspace = true, features = ["rt-multi-thread"] }
serde = { workspace = true }
```

## Lock File Strategy

**Commit Cargo.lock** for reproducible builds:

```bash
git add Cargo.lock
git commit -m "chore: update lockfile"
```

This ensures:

- CI builds are deterministic
- Language bindings get same underlying Rust code
- Security updates are tracked

## Workspace Member Discovery

**Common pitfall**: Forgetting to add crate to workspace members.

**Verify workspace integrity**:

```bash
cargo metadata --format-version 1 | jq '.workspace_members'
```

Should list all crates. If missing:

```toml
[workspace]
members = [
    "crates/html-to-markdown",
    "crates/html-to-markdown-py",
    # ... add missing member here
]
```

## Cross-Crate Dependencies

**Problem**: Circular dependencies between crates in workspace.

**Solution**: Clearly defined dependency graph.

```
html-to-markdown (core library)
├── html-to-markdown-py (depends on core)
├── html-to-markdown-node (depends on core)
├── html-to-markdown-rb (depends on core)
├── html-to-markdown-ffi (depends on core)
└── html-to-markdown-cli (depends on core)
```

**Bad structure** (avoid):

```
- html-to-markdown depends on html-to-markdown-py
- html-to-markdown-py depends on html-to-markdown
# Circular!
```

## Building Specific Members

```bash
# Build single crate
cargo build -p html-to-markdown-py

# Build all members
cargo build --all

# Build all but exclude certain platform bindings
cargo build --all --exclude html-to-markdown-wasm

# Test single member
cargo test -p html-to-markdown
```

## Feature Gate Coordination

**Workspace-level features for conditional compilation**:

```toml
[workspace.dependencies]
tokio = { version = "1.35", optional = true, features = ["full"] }

# In core library Cargo.toml
[package]
features = ["default"]

[features]
default = ["sync"]
async-runtime = ["tokio", "dep:tokio"]
ffi = []
```

**Binding crates enable needed features**:

```toml
# html-to-markdown-py/Cargo.toml
[dependencies]
html-to-markdown = { path = "../html-to-markdown", version = "0.5.0", features = ["sync"] }
```

## Example: Real Workspace with Version Sync

**Initial setup**:

```bash
cargo new --lib crates/html-to-markdown
cargo new --lib crates/html-to-markdown-py
cargo new --lib crates/html-to-markdown-node
# ... etc

# Create workspace at root
echo '[workspace]
members = ["crates/*"]
resolver = "2"
[workspace.package]
version = "0.5.0"
' > Cargo.toml
```

**Sync versions for 0.6.0 release**:

```bash
./scripts/sync_versions.py 0.6.0
cargo test --all          # Verify all members still work
cargo build --release --all
git add -A && git commit -m "chore: bump version to 0.6.0"
```

## Anti-Patterns to Avoid

1. **Mismatched versions across crates**:

   ```toml
   # BAD: Different versions
   # html-to-markdown/Cargo.toml: version = "0.5.0"
   # html-to-markdown-py/Cargo.toml: version = "0.4.9"

   # GOOD: Use workspace.package inheritance
   version.workspace = true
   ```

1. **Circular dependencies**:

   ```toml
   # BAD: Core depends on binding
   # html-to-markdown/Cargo.toml:
   html-to-markdown-py = { path = "../html-to-markdown-py" }

   # GOOD: Only bindings depend on core
   ```

1. **Uncommitted Cargo.lock**:

   ```bash
   # BAD: Cargo.lock in .gitignore

   # GOOD: Commit for reproducibility
   git add Cargo.lock
   ```

1. **Too many nested workspaces**:

   ```toml
   # BAD: Nested workspaces confuse resolution
   # /Cargo.toml: workspace with members
   # /crates/sub/Cargo.toml: another workspace!

   # GOOD: Single root workspace
   ```

## Cross-references to Related Skills

- **rust-module-organization-public-api-design**: APIs exposed across workspace members
- **binding-crate-architecture-patterns**: Structure of binding crates within workspace
- **build-profiles**: Coordinating build modes across workspace
- **release-and-deployment-processes**: Version bumping workflow
