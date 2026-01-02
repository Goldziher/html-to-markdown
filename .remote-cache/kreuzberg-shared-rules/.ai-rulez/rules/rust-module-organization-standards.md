______________________________________________________________________

## priority: high

# Rust Module Organization Standards

**Module structure · Public API surface · Re-export patterns · Feature gates · cargo-public-api verification**

## Module Structure Requirements

### lib.rs Organization

- **Single entry point**: All public APIs exposed through `crates/{name}/src/lib.rs`
- **Flat re-export strategy**: Public modules re-exported at root, not nested in use statements
  ```rust
  // crates/html-to-markdown/src/lib.rs
  pub use crate::converter::{HtmlToMarkdown, ConversionOptions};
  pub use crate::error::ConversionError;

  mod converter;
  mod error;
  mod sanitizer;
  ```
- **Three-tier module structure**:
  1. **Public tier** (re-exported from lib.rs): Stable, versioned APIs
  1. **Semi-private tier** (pub within crate, not re-exported): Internal-facing utilities
  1. **Private tier** (private)\*\*: Internal implementation details

### Submodule Organization

- **One type per file** for public types: `src/converter.rs` contains `HtmlToMarkdown` only
- **Utility modules grouped**: `src/utils/mod.rs` for multiple small types (< 100 lines each)
- **Feature-gated modules**: Conditional compilation in `src/lib.rs` with cfg attributes
  ```rust
  #[cfg(feature = "serde")]
  pub mod serde;

  mod parser;      // always compiled
  mod sanitizer;
  ```
- **Tests co-located**: Unit tests in same file as implementation; integration tests in `tests/` directory
- **Crate structure limit**: Max 8 top-level modules per crate (combine smaller ones into utils)

### Clear Module Boundaries

- **No circular dependencies**: Check with `cargo depgraph` for cycles
- **Dependency direction**: Leaf modules (utilities) → Core modules → Root re-exports
- **Export policies**: Document in module rustdoc what is public API vs internal
  ```rust
  /// Public API for HTML to Markdown conversion.
  ///
  /// # Stability
  /// This module is part of the stable API; breaking changes require semver major version bump.
  pub mod converter {
      // ...
  }
  ```

## Public API Surface Design

### Stable API Contract

- **Semantic versioning**: Public APIs follow semver; private APIs can change freely
- **Marked stability**: Add rustdoc comment to all public items:
  ```rust
  /// Convert HTML to Markdown.
  ///
  /// # Stability
  /// Stable in 1.x; signature frozen until 2.0.
  pub fn html_to_markdown(html: &str) -> Result<String> {
  ```
- **Deprecation path**: Deprecated functions marked with `#[deprecated]` + rustdoc explanation
  ```rust
  #[deprecated(since = "1.2.0", note = "use `html_to_markdown_v2` instead")]
  pub fn html_to_markdown_old(html: &str) -> String {
  ```

### Minimal Public API

- **Principle of least privilege**: Only export types/functions required for user-facing operations
- **Hide internals**: Mark internal types as `pub(crate)` aggressively
- **Use builder pattern**: For complex configurations, avoid huge public structs
  ```rust
  pub struct ConversionOptions {
      allow_html: bool,      // public field only if truly needed
  }

  impl ConversionOptions {
      pub fn new() -> Self { /* ... */ }
      pub fn with_html_allowed(mut self, allow: bool) -> Self { /* ... */ }
  }
  ```

### Error Types

- **Single error type per domain**: `pub enum ConversionError { ... }` not scattered errors
- **Exhaustive error variants**: Use `#[non_exhaustive]` to allow future additions safely
  ```rust
  #[non_exhaustive]
  pub enum ConversionError {
      InvalidHtml(String),
      UnsupportedFeature,
      #[doc(hidden)]
      __NonExhaustive,
  }
  ```
- **Error documentation**: Every variant documented with examples and recovery strategies

## Re-Export Patterns

### Root-Level Re-Exports

- **All public items re-exported in lib.rs**: Users import from crate root, not submodules
  ```rust
  // GOOD
  use html_to_markdown::{HtmlToMarkdown, ConversionError};

  // AVOID
  use html_to_markdown::converter::HtmlToMarkdown;
  ```
- **Organize re-exports by category**:
  ```rust
  // Core conversion API
  pub use crate::converter::{HtmlToMarkdown, ConversionOptions};

  // Errors
  pub use crate::error::{ConversionError, ParseError};

  // Feature-specific
  #[cfg(feature = "serde")]
  pub use crate::serde_support::{serialize, deserialize};
  ```

### Glob Re-Exports (Restricted)

- **Never use glob re-exports** (`pub use module::*`) in lib.rs; explicit is better
- **Justified only in prelude modules**:
  ```rust
  // src/prelude.rs
  pub use crate::{HtmlToMarkdown, ConversionOptions, ConversionError};
  // Users: use html_to_markdown::prelude::*
  ```
- **Prelude re-exported from lib.rs**:
  ```rust
  pub mod prelude {
      pub use crate::{HtmlToMarkdown, ConversionOptions};
  }
  ```

### Version-Specific Re-Exports

- **No version-specific public modules**: Feature gates used instead for variants
  ```rust
  // AVOID: pub mod v1 { ... }; pub mod v2 { ... }

  // GOOD: Single type with feature flags
  pub struct Options { ... }
  #[cfg(feature = "experimental-v2-api")]
  pub struct OptionsV2 { ... }
  ```

## Feature Gate Organization

### Feature Definition & Hygiene

- **Declare all features in Cargo.toml** with descriptions:
  ```toml
  [features]
  default = ["html5ever"]
  html5ever = ["dep:html5ever"]
  serde = ["dep:serde"]
  experimental-v2-api = []
  ```
- **No undocumented features**: Every feature must have a comment explaining purpose
- **Feature combinations**: Document incompatible feature combinations

### Conditional Module Compilation

- **Features control modules, not implementations**:
  ```rust
  // BAD: Scattered #[cfg] throughout code
  if cfg!(feature = "serde") { /* ... */ }

  // GOOD: Entire module gated
  #[cfg(feature = "serde")]
  pub mod serde_support;
  ```
- **Feature re-exports at root**:
  ```rust
  #[cfg(feature = "serde")]
  pub use crate::serde_support::*;
  ```

### Default Features

- **Lean default**: Default feature set should provide core functionality
- **Optional dependencies**: Use `dep:` syntax in Cargo.toml:
  ```toml
  html5ever = ["dep:html5ever"]  # Empty feature, enables dependency
  ```
- **Document defaults**: Clearly state what default features enable

## Cargo Public API Verification

### CI Integration

- **Install cargo-public-api**: `cargo install cargo-public-api` in CI
- **CI check**: `cargo public-api --features default` in each crate's CI
- **Fail on breaking changes**: CI blocks any public API removals/modifications without version bump

### API Baseline Management

- **Commit baseline**: `crates/{name}/api.txt` contains previous public API baseline
  ```
  # crates/html-to-markdown/api.txt
  pub use std::result::Result;
  pub struct html_to_markdown::HtmlToMarkdown { ... }
  pub fn html_to_markdown::HtmlToMarkdown::new(...) -> ...
  ```
- **Generate baseline**: `cargo public-api --features default > crates/{name}/api.txt`
- **Diff on changes**: `cargo public-api --features default --diff` shows changes
- **Review requirement**: Any changes to api.txt require explicit approval

### Breaking Change Detection

- **Removed items**: Fail CI if public type/function removed without deprecation
- **Signature changes**: Detect function parameter/return type modifications
- **Trait implementation changes**: Catch removal of trait derives or implementations
- **Error handling**: All breaking changes require RFC discussion + 2-release deprecation window

## Documentation Requirements

- **Module rustdoc**: Every module (public or `pub(crate)`) has top-level rustdoc comment
- **Public item examples**: All public types/functions include `# Examples` section
- **Safety comments**: Unsafe blocks documented with `// SAFETY: ...`
- **Version information**: Public APIs include `# Stability` section with version introduced and semver commitments

## Agent Coordination

- **rust-core-engineer** owns all module organization and public API design decisions
- **rust-core-engineer** reviews all changes to `lib.rs` and module structure
- **rust-core-engineer** maintains `api.txt` baselines and approves public API changes
- Breaking API changes require **rust-core-engineer** RFC + 2-release lead time

## Anti-Patterns

- Never: Use `pub use module::*` in lib.rs (glob re-exports of entire modules)
- Never: Expose internal implementation types (Vec, HashMap) in public API; use newtypes
- Never: Create version-specific modules (v1, v2); use feature gates instead
- Never: Add public items without examples and stability documentation
- Never: Modify public API signature without understanding impact on bindings
- Never: Skip cargo-public-api checks; breaking changes must be intentional
- Never: Leave deprecated items without removal timeline (max 2 major versions)
