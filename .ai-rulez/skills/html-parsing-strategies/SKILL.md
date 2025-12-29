# HTML Parsing Strategies for html-to-markdown

## Overview

The html-to-markdown project uses two complementary HTML parsers to handle different conversion scenarios and performance requirements:

- **astral-tl** (primary, lightweight): Fast, incremental parser via the `tl` crate
- **html5ever** (robust fallback): Spec-compliant HTML5 parser for edge cases and malformed HTML

## Parser Trade-offs and Selection

### astral-tl (tl crate)

**Strengths:**
- Ultra-fast parsing performance with minimal memory overhead
- Ideal for streaming/large document processing
- Lightweight footprint in binary distributions
- Fast DOM traversal without full tree reconstruction
- Direct integration in core conversion pipeline

**Weaknesses:**
- Less tolerant of severely malformed HTML
- May require pre-processing for certain edge cases
- Limited HTML5 spec compliance for recovery strategies
- No automatic tag closure or namespace handling

**Use Cases:**
- Well-formed HTML from web servers
- Templated HTML with consistent structure
- High-throughput conversion pipelines
- Resource-constrained environments (WASM, mobile)

**Implementation Location:**
- `/crates/html-to-markdown/src/converter.rs` - Primary conversion logic
- Dependency: `tl.workspace = true` in Cargo.toml

### html5ever (html5ever + markup5ever_rcdom)

**Strengths:**
- Full HTML5 spec compliance with automatic error recovery
- Handles severely malformed HTML gracefully
- Proper tag closure and namespacing (SVG, MathML)
- Robust against adversarial or malicious input
- Comprehensive entity decoding

**Weaknesses:**
- Higher memory footprint due to full tree construction
- Slower parsing for well-formed documents
- Larger compiled binary size
- Not used by default in fast path

**Use Cases:**
- Untrusted/adversarial HTML input
- Legacy or malformed markup from old systems
- Documents with embedded XML/SVG content
- Scenarios where correctness > performance

**Implementation Location:**
- Dependency: `html5ever.workspace = true` and `markup5ever_rcdom.workspace = true`
- Feature-gated or used in specific edge case handlers

## DOM Traversal Patterns

### tl-based Traversal

The `tl` parser provides a lightweight DOM interface:

```rust
// Example from converter.rs
// Using tl parser for fast sequential traversal
for node in dom.iter() {
    match node.kind() {
        NodeKind::Tag(tag) => {
            // Handle element
            let tag_name = tag.name();
            let attrs = tag.attributes();
            // Process children recursively
        }
        NodeKind::Text(text) => {
            // Handle text node
            let content = text.as_bytes();
        }
        _ => {}
    }
}
```

**Characteristics:**
- Single-pass traversal in document order
- Minimal allocations during walk
- Reference-based access (no deep cloning)
- Position-aware (useful for table cells, list items)

### html5ever-based Traversal

The `markup5ever_rcdom` provides a full DOM tree:

```rust
// html5ever creates a reference-counted DOM tree
// Traversal via RcDom with Handle references
// Recursive descent through child_nodes()
// Full attribute access via attributes()
```

**Characteristics:**
- Complete parent/child/sibling relationships
- Full attribute parsing with namespaces
- Post-processing and recovery applied
- Higher memory cost but more robust

## Edge Cases and Parser Selection

### When to Use tl (Fast Path)

1. **Plain text input (no `<` characters)**
   - Detected in `fast_text_only()` before any parsing
   - Optimized for text-only documents

2. **Well-formed HTML**
   - Standard web pages, CMS output, templated HTML
   - Default path in `converter.rs`

3. **UTF-8 validated input**
   - HTML already proven to be valid UTF-8
   - No encoding detection needed

### When to Use html5ever (Robust Path)

1. **Binary detection triggered**
   - UTF-16 encoding detected
   - Gzip/PDF/ZIP signatures found
   - Excess control characters
   - Rejection in `validate_input()` in `lib.rs`

2. **Severely malformed markup**
   - Unclosed tags across many levels
   - Mismatched tag nesting (e.g., `<table><div>` structure)
   - SVG/MathML embedded without proper namespace handling

3. **Legacy or fuzzing inputs**
   - Old HTML from Netscape era
   - Intentionally malformed test cases
   - Documents with mixed encoding declarations

## Text Handling During Parsing

### Whitespace Preservation

The converter preserves whitespace exactly as parsed:

```rust
// From converter.rs
// All text nodes retain original spacing
// No HTML5 whitespace collapsing applied
// Raw text preservation mode (not normalized)
```

**Modes:**
- **Strict**: All whitespace (spaces, tabs, newlines) preserved exactly
- **Normalized**: Multiple spaces/newlines collapsed to single space (configurable)

### Entity Decoding

Both parsers decode HTML entities:
- `html-escape` crate for quick common entities
- Both parsers handle numeric (`&#123;`) and named (`&amp;`) entities
- Context-aware decoding in `text::decode_html_entities_cow()`

## Practical Selection Flow

```
Input HTML string
    |
    +-- validate_input() checks for binary/encoding issues
    |
    +-- FAIL? Reject with ConversionError::InvalidInput
    |
    +-- PASS? Proceed to conversion
    |
    +-- Is plain text (no '<')? Use fast_text_only()
    |
    +-- Otherwise: Use tl parser (default)
            |
            +-- Converts successfully? Return markdown
            |
            +-- Need html5ever for edge case?
                    (handled via feature gates or fallback)
```

## Performance Considerations

### Benchmark Context

The `/tools/benchmark-harness/` suite includes parser benchmarks:
- astral-tl performance: Sub-millisecond for typical documents
- html5ever fallback: Used sparingly for worst-case handling
- Memory profiling via `benchmark:harness:memory` task

### Optimization Strategies

1. **Fast text path**: Check `if html.contains('<')` first
2. **LRU caching**: Common patterns cached in converter
3. **Streaming possible**: tl parser supports incremental feeding
4. **Pre-processing**: Normalize line endings before parsing

## References to Codebase

- **Input validation**: `/crates/html-to-markdown/src/lib.rs` (lines 59-147)
  - `validate_input()` function
  - Binary magic prefix detection
  - UTF-16 hint detection

- **Fast text path**: `/crates/html-to-markdown/src/lib.rs` (lines 157-197)
  - `fast_text_only()` optimization

- **Converter logic**: `/crates/html-to-markdown/src/converter.rs`
  - Primary parsing and traversal
  - Element-to-markdown mapping

- **Configuration**: `/crates/html-to-markdown/src/options.rs`
  - `WhitespaceMode` enum (Strict vs Normalized)
  - `PreprocessingOptions` for input handling

## Implementation Guidelines

### Adding New Parser Support

1. Create feature gate in Cargo.toml
2. Wrap parser in abstraction layer (don't expose directly)
3. Add to `convert_html()` or create new function
4. Test with both well-formed and malformed inputs
5. Benchmark against baseline (astral-tl)

### Handling Parser-Specific Issues

- **Entity handling differences**: Test with numeric and named entities
- **Namespace handling**: Ensure SVG/MathML parsed correctly
- **Error recovery**: Verify malformed HTML produces sensible output
- **Memory constraints**: Profile with large documents (>10MB)

## Quick Decision Matrix

| Input Type | Parser | Reason |
|-----------|--------|--------|
| Plain text | None (fast_text_only) | No parsing needed |
| Standard web HTML | tl | Fast, sufficient correctness |
| Untrusted/fuzzing input | html5ever | Full spec compliance |
| UTF-16 detected | Error | Reject with validation error |
| Severely malformed | html5ever | Better error recovery |
| Large documents | tl | Better streaming potential |
| WASM target | tl | Smaller binary footprint |
