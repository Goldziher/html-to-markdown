______________________________________________________________________

## priority: critical

# Polyglot FFI Stability Guarantee

**C API stability (semver) · ABI compatibility · cbindgen automation · FFI versioning**

## C API Stability Policy

### Semantic Versioning for C Headers

- **MAJOR version** (breaking C API changes): struct layout changes, function signature modifications, removal of public types
- **MINOR version** (backward-compatible additions): new functions, new optional struct fields (only at end), new enums
- **PATCH version** (bug fixes): internal implementations, documentation clarifications, non-breaking optimizations
- All C header files are versioned: `html_to_markdown.h` includes version constant `HTML_TO_MARKDOWN_VERSION_MAJOR`, `_MINOR`, `_PATCH`
- C API changes MUST be announced 2 major releases in advance (deprecation period)
- Document breaking changes in CHANGELOG.md with migration guides for each language binding

### Header Generation Automation

- **cbindgen** is the single source of truth for C bindings: `crates/ffi/cbindgen.toml` controls output
- Headers are auto-generated from Rust source: `cargo build -p html-to-markdown-ffi` generates `target/include/html_to_markdown.h`
- Never manually edit generated `.h` files; changes go in `crates/ffi/src/lib.rs` exclusively
- CI step: Verify generated headers match committed versions; fail if drift detected
- Header locations:
  - Development: `target/include/html_to_markdown.h`
  - Distributed: `crates/ffi/include/html_to_markdown.h` (committed for reproducibility)

## ABI Compatibility Guarantees

### Stable ABI Commitment

- All exported C functions use `#[no_mangle]` with extern "C" in Rust: `pub extern "C" fn html_to_markdown(...)`
- Function signatures are frozen at MAJOR.MINOR boundaries; additions only as new functions
- Struct layouts are fixed: no reordering fields, no changing field types within a MAJOR version
- Struct size and alignment must remain stable (use `#[repr(C)]` exclusively)
- Enums use explicit `repr` (e.g., `#[repr(u32)]`) for cross-platform consistency
- All public types documented in `/// FFI STABLE: ...` rustdoc comments with version they were introduced

### Cross-Platform ABI

- Test ABI compatibility on x86_64-linux-gnu, x86_64-pc-windows-msvc, aarch64-apple-darwin
- Pointer size validation: use `size_t` (not `usize` in C) for all size parameters
- Integer types: prefer `int32_t`, `uint64_t` etc. (from `<stdint.h>`) over platform-dependent types
- FFI tests compile C code against distributed headers on all three platforms in CI
- Never ship headers with inline implementations; use function pointers for callbacks

## FFI Versioning Strategy

### Version Constants and Checks

- C headers define three versioning constants:
  ```c
  #define HTML_TO_MARKDOWN_VERSION_MAJOR 1
  #define HTML_TO_MARKDOWN_VERSION_MINOR 2
  #define HTML_TO_MARKDOWN_VERSION_PATCH 0
  ```
- Runtime version query function: `const char* html_to_markdown_version(void)` returns "1.2.0"
- All FFI bindings call version check at initialization: `assert_version_compatible(major, minor)`
- Binding code refuses to link if major version mismatch detected

### FFI Package Versions

- `crates/ffi` Cargo.toml version is the authoritative FFI version
- `packages/{python,ruby,php,node,java,csharp}` binding versions track `crates/ffi` major.minor (patch versions independent)
- Example: html-to-markdown-python 1.2.x binds to html-to-markdown-ffi 1.2.y (any patch)
- Binding breaking changes (e.g., API redesign) bump minor version within FFI major version

## Breaking Change Procedures

### Deprecation & Migration Path

1. **Announce (MAJOR - 1)**: Document breaking change in RFC issue, provide migration guide
1. **Deprecate (MAJOR)**: Add `_deprecated` suffix to old function, mark with `#[deprecated]` in Rust
   ```c
   void html_to_markdown_convert_deprecated(const HtmlInput* input, HtmlOutput* output);
   ```
1. **Support dual API (MAJOR)**: Both old and new functions work simultaneously
1. **Remove (MAJOR + 1)**: Delete deprecated function; announce removal in docs
1. **Communicate**: Email all FFI binding maintainers 6 weeks before removal

### Breaking Change Categories

- **Struct field reordering**: Not allowed; add new struct instead
- **Function parameter count change**: Must create new function (`html_to_markdown_v2_...`)
- **Enum value removal**: Add deprecation comment, create new enum if needed
- **Memory ownership change**: Document clearly in migration guide (who owns allocated memory)
- **Callback signature change**: New callback type required (e.g., `OnProgressV2Callback`)

## Agent Coordination

- **ffi-maintenance-engineer** agent owns `crates/ffi/` and all `.h` header stability
- **ffi-maintenance-engineer** reviews all C API changes before commit
- **ffi-maintenance-engineer** maintains FFI version matrix and binding compatibility docs
- Other agents coordinate with **ffi-maintenance-engineer** for breaking changes requiring 2-release lead time

## Anti-Patterns

- Never: Export Rust types directly (Result, Vec); wrap in opaque C structs
- Never: Use platform-specific integer types in C API (use stdint.h types exclusively)
- Never: Inline implementations in public C headers; always opaque pointers to Rust data
- Never: Forget deprecation period; removing functions with < 2 release lead time breaks users
- Never: Change struct field order, type, or layout within a major version
- Never: Manually edit generated cbindgen output; regenerate and commit
