______________________________________________________________________

## priority: critical

# Workspace Dependency Management Policy

**Cargo.toml versioning · Workspace inheritance · MSRV consistency · Path vs published · Lock file discipline · Security audits**

## Cargo.toml as Single Source of Truth

### Workspace Root Configuration

- **Workspace definition**: `Cargo.toml` at repository root defines entire workspace:
  ```toml
  [workspace]
  members = [
      "crates/html-to-markdown",
      "crates/ffi",
      "packages/python",
      # ... all members
  ]
  resolver = "2"
  ```
- **Version inheritance**: All member crates inherit version from workspace root:
  ```toml
  # Cargo.toml (root)
  [workspace]
  members = ["crates/html-to-markdown", "crates/ffi"]

  [workspace.package]
  version = "1.2.3"
  authors = ["Team"]
  edition = "2024"

  # crates/html-to-markdown/Cargo.toml
  [package]
  name = "html-to-markdown"
  version.workspace = true
  authors.workspace = true
  edition.workspace = true
  ```
- **Single version source**: Update version in workspace root only; propagates to all members
- **No scattered versions**: All crates use workspace version; never hardcode per-crate versions
- **Dependency coordination**: `scripts/sync_versions.py` propagates workspace version to non-Rust packages (Python, PHP, Ruby, Node.js)

### Version Management Process

1. **Increment workspace version** in `Cargo.toml` (root)
1. **Run sync_versions.py** to update language-specific versions:
   ```bash
   python scripts/sync_versions.py 1.2.3
   ```
1. **Commit Cargo.toml + version sync outputs** atomically
1. **Tag release** on git: `git tag v1.2.3`
1. **Verify all lock files updated** before pushing

## Workspace Inheritance Patterns

### Shared Dependencies

- **Define common dependencies in workspace root**:
  ```toml
  [workspace.dependencies]
  tokio = { version = "1.35", features = ["full"] }
  serde = { version = "1.0", features = ["derive"] }
  anyhow = "1.0"
  ```
- **Members inherit shared dependencies**:
  ```toml
  # crates/html-to-markdown/Cargo.toml
  [dependencies]
  tokio = { workspace = true }
  serde = { workspace = true }
  ```
- **Consistency requirement**: No member crate pins different version than workspace definition

### Feature Inheritance

- **Workspace defines feature sets**:
  ```toml
  [workspace.dependencies]
  hyper = { version = "1.0", features = ["full"] }

  # Crates using workspace features get them all
  [dependencies]
  hyper = { workspace = true }
  ```
- **Per-crate feature overrides**: Only allowed if documented and approved by **dependency-management-coordinator**
- **Document overrides**: Add comment explaining why crate deviates from workspace norm

### Internal Workspace Dependencies

- **Use relative paths for workspace members**:
  ```toml
  [dependencies]
  html-to-markdown = { path = "../html-to-markdown", version = "1.2" }
  ```
- **Always include version requirement** even with path dependency; version must match workspace
- **Path dependencies never published**: Path deps are workspace-internal only; remove before publishing

## MSRV Policy (Minimum Supported Rust Version)

### MSRV Definition

- **Workspace MSRV**: Defined in root `Cargo.toml`:
  ```toml
  [package]
  rust-version = "1.70"  # All crates must be compatible
  ```
- **MSRV is binding**: No member crate can use features from newer Rust versions
- **MSRV testing**: CI runs `cargo +1.70 build --workspace` on minimum version
- **Document MSRV**: All crates include MSRV in crate-level rustdoc

### MSRV Maintenance

- **Bump MSRV carefully**: 6-month lead time before raising minimum version
- **Announce MSRV changes**: Include in release notes if not patch release
- **Patch releases**: Can only raise MSRV in minor/major releases (never patch)
- **Dependency alignment**: All dependencies must support declared MSRV

### MSRV-Safe Dependencies

- **Verify MSRV compatibility**:
  ```bash
  cargo update
  cargo +1.70 check --workspace
  ```
- **Dependencies older than MSRV target version**: Fail CI if any dependency drops support below MSRV
- **Document exceptional cases**: If dependency requires higher MSRV, mark as optional/feature-gated

## Path Dependencies vs Published Crates

### Path Dependency Strategy

- **Workspace members**: Use relative paths only
  ```toml
  [dependencies]
  html-to-markdown-core = { path = "../html-to-markdown-core" }
  ```
- **Crates depending on path members**: Must also specify workspace version
- **Path dependency rules**:
  - Only for workspace members
  - Version must match workspace definition
  - Never publish crate with path dependencies to crates.io

### Published Crate Dependencies

- **External crates**: Use published versions from crates.io
  ```toml
  [dependencies]
  tokio = "1.35"
  serde = "1.0"
  ```
- **Version specification**:
  - Use semantic versions: `"1.35"` (not `"1.35.0"`)
  - Allow patch updates: `"1.35"` allows `1.35.x`
  - Restrict major updates: Never use `"*"` or overly loose specs
  - Pre-release versions only for exceptional alpha/beta crates

### Pre-Release Dependencies

- **Never depend on pre-release versions** for stable releases:
  ```toml
  # BAD for stable release
  tokio = "1.35.0-rc.1"

  # GOOD when crate requires it
  html5ever = "0.27.0"  # Currently pre-release but stable enough
  ```
- **Track pre-release dependencies**: Document in RFC if required
- **Clear deprecation path**: Mark when moving off pre-release versions

## Lock File Management

### Cargo.lock Commitment

- **Commit Cargo.lock to repository**: Version control ensures reproducible builds
  ```bash
  git add Cargo.lock
  git commit -m "Update Cargo.lock"
  ```
- **Never .gitignore Cargo.lock**: All developers and CI use identical dependency versions
- **Lock file updates**: `cargo update` updates Cargo.lock for minor/patch versions

### Coordinated Lock File Updates

- **Regular updates**: `cargo update` runs in CI monthly for security patches
- **Dependency audit**: `cargo audit` run before updating lock files
- **Automated update process**:
  1. `cargo update` (respects semver constraints)
  1. `cargo audit` (verify no vulnerabilities)
  1. `cargo build --workspace` (test full build)
  1. Commit Cargo.lock atomically

### Lock File Conflicts

- **Never rebase lock files**: Always merge with three-way merge strategy
- **Conflict resolution**: Keep lock file from base branch, run `cargo update` to resolve
  ```bash
  git checkout --ours Cargo.lock
  cargo update --aggressive
  git add Cargo.lock
  ```

## Security Audit Requirements

### Cargo Audit Integration

- **CI: cargo-audit step**: Run before every merge to main
  ```bash
  cargo audit --deny warnings
  ```
- **Audit policy**: Zero tolerance for known vulnerabilities in dependency tree
- **Fail-fast**: Any crate with vulnerability blocks CI; must be fixed immediately

### Vulnerability Response

- **Minor vulnerabilities**: Update dependency in next patch release
- **Major vulnerabilities**: Emergency patch release with fix within 24 hours
- **Deprecated dependencies**: Plan migration to replacement; set removal timeline

### Dependency Audit Checklist

Before updating Cargo.lock:

- [ ] Run `cargo audit` and verify no vulnerabilities (exit code 0)
- [ ] Verify all dependencies have active maintenance
- [ ] Check MSRV compatibility for updated versions
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Review Cargo.lock diff: no unexpected sub-dependency changes

## Workspace Organization Best Practices

### Crate Arrangement

- **Core library**: `crates/html-to-markdown` - stable, published API
- **FFI library**: `crates/ffi` - C bindings, separate versioning track
- **Language bindings**: `packages/{python,ruby,php,node,java,csharp}` - non-Rust
- **CLI**: `packages/cli` or `crates/cli` - executable, semi-independent versioning
- **Examples**: `examples/` - runnable examples, not part of workspace members

### Dependency Minimization

- **Thin dependency tree**: Fewer dependencies = fewer vulnerabilities
- **Critical path only**: Only add dependencies absolutely necessary for core functionality
- **Evaluate alternatives**: Consider stdlib solutions before adding crate
- **Size audit**: `cargo tree --duplicates` run monthly to catch duplicate versions

## Agent Coordination

- **dependency-management-coordinator** owns all dependency decisions
- **dependency-management-coordinator** reviews all `Cargo.toml` changes before merge
- **dependency-management-coordinator** maintains MSRV and runs security audits
- **dependency-management-coordinator** coordinates version bumps across all language bindings
- **rust-core-engineer** defers to **dependency-management-coordinator** on dependency selections

## Anti-Patterns

- Never: Hardcode version in individual crate Cargo.toml (use workspace inheritance)
- Never: Use glob version specifiers (`"*"`) or overly loose constraints (`">= 1.0"`)
- Never: Add pre-release dependencies to stable crate (except documented exceptions)
- Never: Add path dependencies to published crates; remove before cargo publish
- Never: Create circular dependencies between crates (use cargo tree to verify)
- Never: Ignore cargo-audit warnings or skip security checks before releasing
- Never: Allow Cargo.lock conflicts to remain unresolved; always do three-way merge
- Never: Use different versions of same crate across workspace members (detect with `cargo tree --duplicates`)
- Never: Commit Cargo.lock updates without running full test suite
- Never: Change MSRV in patch releases; semver major/minor only
