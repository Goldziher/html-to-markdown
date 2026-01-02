______________________________________________________________________

## priority: high

# Rust Module Organization & Public API Design

## Module Structure Principles

**Modules are organizational units, not visibility boundaries**. Use `pub` / `pub(crate)` / `pub(super)` to control visibility explicitly.

1. **Root crate module**: `src/lib.rs` or `src/main.rs` re-exports public items
1. **Feature-specific modules**: `src/parsing/`, `src/conversion/`, `src/utils/` organize by domain
1. **Internal vs. public**: Mark private modules with `pub(crate)` if they're internal infrastructure
1. **Re-exports**: Use `pub use` in `lib.rs` to define the public API surface

## Module Hierarchy Example

```
src/
├── lib.rs                    # Root: re-exports public API
├── parsing/                  # Feature module
│   ├── mod.rs               # Module interface, re-exports
│   ├── html_parser.rs       # Internal implementation
│   ├── sanitizer.rs         # Internal implementation
│   └── error.rs             # Domain-specific errors
├── conversion/              # Feature module
│   ├── mod.rs               # Module interface
│   ├── node_visitor.rs      # Implementation
│   └── formatters/          # Submodule
│       ├── mod.rs
│       ├── markdown.rs
│       └── text.rs
├── error.rs                 # Crate-wide error types
└── config.rs                # Configuration
```

## Public API Design

**Golden Rule**: Your `src/lib.rs` should read like a user guide.

````rust
//! html-to-markdown: Convert HTML to Markdown with safety & performance
//!
//! # Quick Start
//! ```
//! use html_to_markdown::HtmlConverter;
//! let converter = HtmlConverter::new();
//! let markdown = converter.convert("<h1>Hello</h1>")?;
//! ```

// Public types
pub use crate::conversion::{HtmlConverter, ConversionConfig};
pub use crate::parsing::ParseError;
pub use crate::error::Error;

// Public type aliases for convenience
pub type Result<T> = std::result::Result<T, Error>;

// Don't re-export implementation details
// pub use crate::parsing::html_parser;  // BAD
````

## Pub/Private Boundaries

**Public** (`pub`): Part of stable API contract

- Main types: `HtmlConverter`, `ConversionConfig`
- Error types: `ParseError`, `ConversionError`
- Common utility functions
- Config builders

**Pub(crate)** (`pub(crate)`): Internal infrastructure, not for external users

```rust
pub(crate) struct InternalParser { ... }
pub(crate) fn internal_helper() { ... }
```

**Private** (no visibility keyword): Never exposed

```rust
fn internal_detail() { ... }
struct InternalState { ... }
```

## Feature Gates

Use Cargo features to conditionally expose API surface:

```toml
[features]
default = ["parsing"]
parsing = []
async-runtime = ["tokio"]
ffi = ["libffi"]

[[example]]
name = "async_convert"
required-features = ["async-runtime"]
```

```rust
// Conditional re-export
#[cfg(feature = "async-runtime")]
pub use crate::async_convert::AsyncConverter;

#[cfg(feature = "ffi")]
pub use crate::ffi::ExportedAPI;
```

## Module Re-exports Pattern

**Module interface** (`src/parsing/mod.rs`):

```rust
//! HTML parsing module with robust error handling

mod html_parser;
mod sanitizer;
pub mod error;

pub use self::html_parser::{Parser, ParseConfig};
pub use self::sanitizer::Sanitizer;

// Internal utilities not re-exported
use self::html_parser::HtmlToken;
```

**Root re-export** (`src/lib.rs`):

```rust
pub use crate::parsing::{Parser, ParseConfig};
pub use crate::parsing::error::{ParseError, SyntaxError};
pub use crate::conversion::HtmlConverter;

// Don't expose internal modules
// Bad: pub mod parsing;
```

## Cargo Public API Validation

Use `cargo-public-api` to track breaking changes:

```bash
# Generate baseline
cargo public-api --baseline > baseline.txt

# Check for breaking changes in PRs
cargo public-api --diff baseline.txt

# Track additions
cargo public-api > current.txt
```

Add to CI:

```yaml
- name: Check public API
  run: |
    cargo install cargo-public-api
    cargo public-api --diff baseline.txt
```

## Anti-Patterns to Avoid

1. **Exposing implementation details**:

   ```rust
   // BAD: Leaks internal parser state
   pub use crate::parsing::internal::ParserState;

   // GOOD: Expose stable interface
   pub fn parse(input: &str) -> Result<Document> { ... }
   ```

1. **Deeply nested modules in public API**:

   ```rust
   // BAD: Public `mod parsing { mod html_parser { ... } }`
   // Users write: html_to_markdown::parsing::html_parser::Parser

   // GOOD: Flatten in re-export
   // Users write: html_to_markdown::Parser
   ```

1. **Mixed public/private in single module**:

   ```rust
   // BAD: Confusing what's stable
   pub fn stable_api() { ... }
   pub fn internal_detail() { ... }

   // GOOD: Separate concerns
   pub mod api { pub fn stable() { ... } }
   mod internal { pub(crate) fn detail() { ... } }
   ```

1. **Not documenting API stability**:

   ```rust
   // BAD: Users don't know if this might change
   pub struct Config { ... }

   // GOOD: Clear stability guarantees
   /// Configuration for HTML conversion (part of stable 1.x API)
   pub struct Config { ... }
   ```

## Documentation Best Practices

- **Module docs**: Explain purpose and common patterns
- **Type docs**: Document public invariants
- **Example code**: Show real usage patterns
- **Error docs**: Document error conditions

````rust
/// Converts HTML to Markdown
///
/// # Examples
/// ```
/// use html_to_markdown::HtmlConverter;
/// let converter = HtmlConverter::new();
/// let md = converter.convert("<h1>Title</h1><p>Content</p>")?;
/// assert!(md.contains("# Title"));
/// ```
///
/// # Errors
/// Returns `ParseError` if HTML is malformed beyond recovery.
pub fn convert(html: &str) -> Result<String> { ... }
````

## Stability Markers

Document API stability intentions:

```rust
// Stable in 1.x API
/// Configuration builder (stable API)
pub struct ConfigBuilder { ... }

// Experimental, may change
/// Experimental: May change in minor versions
#[doc(alias = "deprecated")]
pub struct ExperimentalFeature { ... }

// Deprecated
/// Deprecated: Use `convert_v2()` instead
#[deprecated(since = "0.5.0", note = "use convert_v2")]
pub fn convert_old(input: &str) -> Result<String> { ... }
```

## Cross-references to Related Skills

- **workspace-dependency-management**: Coordinating public API across crates
- **error-handling-strategy**: Designing error types as part of public API
- **testing-philosophy-coverage**: Testing public API surface
