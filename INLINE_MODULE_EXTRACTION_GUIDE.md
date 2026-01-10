# Inline Element Handler Extraction Guide

## Quick Reference: Extraction Lines

### Source File
`crates/html-to-markdown/src/converter.rs` (walk_node function starting at line 3334)

### Extraction Map

| Module | Elements | Start | End | Lines | Priority |
|--------|----------|-------|-----|-------|----------|
| **inline/emphasis.rs** | `<strong>`, `<b>`, `<em>`, `<i>` | 3874, 3993 | 3991, 4105 | 231 | HIGH |
| **inline/link.rs** | `<a>` | 4107 | 4375 | 269 | HIGH |
| **inline/code.rs** | `<code>`, `<kbd>`, `<samp>` | 5092, 5197 | 5124, 5379 | 216 | MEDIUM |
| **inline/semantic.rs** | `<mark>`, `<del>`, `<s>`, `<ins>`, `<u>`, `<small>`, `<sub>`, `<sup>` | 4746-5090 | 5090 | 345 | LOW-MEDIUM |
| **inline/ruby.rs** | `<ruby>`, `<rb>`, `<rt>`, `<rp>`, `<rtc>` | 7321-7546 | 7553 | 243 | HIGH |

---

## Helper Functions to Extract/Move

These functions are currently in converter.rs and need to be accessible to inline handlers:

### 1. Text Processing Helpers

**Location: Line 97**
```rust
fn chomp_inline(text: &str) -> (&str, &str, &str)
```
- **Purpose:** Split text into (prefix, suffix, trimmed) parts
- **Used by:** emphasis, code, semantic modules
- **Note:** Can be moved to `inline/helpers.rs` or kept in converter

**Location: Line 875**
```rust
fn escape_link_label(text: &str) -> String
```
- **Purpose:** Escape brackets in link labels
- **Used by:** link module
- **Scope:** Public for inline module

**Location: Line 1564**
```rust
fn normalize_link_label(label: &str) -> String
```
- **Purpose:** Normalize whitespace and newlines in link labels
- **Used by:** link module

### 2. Link-Specific Helpers

**Location: Line 914**
```rust
fn append_markdown_link(
    output: &mut String,
    label: &str,
    href: &str,
    title: Option<&str>,
    raw_text: &str,
    options: &ConversionOptions,
)
```
- **Purpose:** Format markdown link syntax
- **Used by:** link module
- **Note:** Could be moved to link module or helpers

**Location: Line 1512**
```rust
fn collect_link_label_text(
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> (String, Vec<tl::NodeHandle>, bool)
```
- **Purpose:** Recursively collect text from link children
- **Used by:** link module
- **Note:** Links with block-level children

**Location: Line 979**
```rust
fn find_single_heading_child(node_handle: tl::NodeHandle, parser: &tl::Parser) -> Option<(usize, tl::NodeHandle)>
```
- **Purpose:** Detect if link contains only one heading
- **Used by:** link module

**Location: Line 1018**
```rust
fn push_heading(output: &mut String, ctx: &Context, options: &ConversionOptions, level: usize, text: &str)
```
- **Purpose:** Format heading output (for links containing headings)
- **Used by:** link module
- **Note:** Complex heading formatting logic

### 3. Utility Helpers

**Location: Line 3311**
```rust
fn append_inline_suffix(
    output: &mut String,
    suffix: &str,
    has_core_content: bool,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
)
```
- **Purpose:** Append spacing/line break suffix intelligently
- **Used by:** emphasis, code, semantic modules

**Location: Line 130**
```rust
fn trim_trailing_whitespace(output: &mut String)
```
- **Purpose:** Remove trailing spaces/tabs
- **Used by:** Multiple modules

**Location: Line 162**
```rust
fn truncate_at_char_boundary(value: &mut String, max_len: usize)
```
- **Purpose:** Safe UTF-8 truncation
- **Used by:** link module (512 char limit)

---

## Module Files To Create

### 1. crates/html-to-markdown/src/converter/inline/mod.rs

```rust
//! Inline element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for inline HTML elements:
//! - Emphasis (strong, b, em, i)
//! - Links (a)
//! - Code (code, kbd, samp)
//! - Semantic elements (mark, del, s, ins, u, small, sub, sup)
//! - Ruby annotations (ruby, rb, rt, rp, rtc)

pub mod emphasis;
pub mod link;
pub mod code;
pub mod semantic;
pub mod ruby;

// Re-exports for convenience
pub use emphasis::{handle_strong, handle_emphasis};
pub use link::handle as handle_link;
pub use code::{handle_code, handle_kbd_samp};
pub use semantic::{handle_mark, handle_strikethrough, handle_ins, handle_u, handle_small, handle_sub, handle_sup};
pub use ruby::{handle_ruby, handle_rb, handle_rt, handle_rp, handle_rtc};

/// Dispatches inline element handling to the appropriate handler.
///
/// Returns `true` if the element was handled, `false` otherwise.
///
/// # Usage
/// ```ignore
/// if crate::converter::inline::dispatch_inline_handler(
///     &tag_name,
///     node_handle,
///     parser,
///     output,
///     options,
///     ctx,
///     depth,
///     dom_ctx,
/// ) {
///     return; // Element was handled
/// }
/// ```
pub fn dispatch_inline_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::converter::Context,
    depth: usize,
    dom_ctx: &super::super::converter::DomContext,
) -> bool {
    match tag_name {
        "strong" | "b" => {
            emphasis::handle_strong(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "em" | "i" => {
            emphasis::handle_emphasis(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "a" => {
            link::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "code" => {
            code::handle_code(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "kbd" | "samp" => {
            code::handle_kbd_samp(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "mark" => {
            semantic::handle_mark(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "del" | "s" => {
            semantic::handle_strikethrough(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "ins" => {
            semantic::handle_ins(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "u" => {
            semantic::handle_u(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "small" => {
            semantic::handle_small(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "sub" => {
            semantic::handle_sub(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "sup" => {
            semantic::handle_sup(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "ruby" => {
            ruby::handle_ruby(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "rb" => {
            ruby::handle_rb(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "rt" => {
            ruby::handle_rt(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "rp" => {
            ruby::handle_rp(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "rtc" => {
            ruby::handle_rtc(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        _ => false,
    }
}
```

### 2. crates/html-to-markdown/src/converter/inline/emphasis.rs

**Extract lines 3874-3991 and 3993-4105 from converter.rs**

Key refactoring needed:
- Create two functions: `pub fn handle_strong(...)` and `pub fn handle_emphasis(...)`
- Keep visitor code (feature-gated with `#[cfg(feature = "visitor")]`)
- Import/use helper functions: `chomp_inline()`, `append_inline_suffix()`, `get_text_content()`

### 3. crates/html-to-markdown/src/converter/inline/link.rs

**Extract lines 4107-4375 from converter.rs**

Key refactoring needed:
- Create function: `pub fn handle(node_handle, parser, output, options, ctx, depth, dom_ctx)`
- Export link helper functions or keep private
- Feature-gate metadata collection code
- Import: `escape_link_label()`, `append_markdown_link()`, `collect_link_label_text()`, etc.

### 4. crates/html-to-markdown/src/converter/inline/code.rs

**Extract lines 5092-5124 and 5197-5379 from converter.rs**

Key refactoring needed:
- Two functions: `pub fn handle_code(...)` and `pub fn handle_kbd_samp(...)`
- Complex backtick escaping logic (keep as-is)
- Feature-gate visitor code
- Import: `chomp_inline()`, `append_inline_suffix()`

### 5. crates/html-to-markdown/src/converter/inline/semantic.rs

**Extract lines 4746-5090 from converter.rs** (8 handlers combined)

Key refactoring needed:
- Separate functions for each element type: `handle_mark()`, `handle_strikethrough()`, etc.
- Feature-gate visitor callbacks
- Import: `chomp_inline()`, `append_inline_suffix()`, `get_text_content()`

### 6. crates/html-to-markdown/src/converter/inline/ruby.rs

**Extract lines 7321-7553 from converter.rs**

Key refactoring needed:
- Functions: `handle_ruby()`, `handle_rb()`, `handle_rt()`, `handle_rp()`, `handle_rtc()`
- Complex tag sequence analysis logic (keep as-is)
- Import: `normalized_tag_name()`

---

## Integration Steps

### Step 1: Update Module Declarations
In `crates/html-to-markdown/src/converter/mod.rs` (or create if needed):
```rust
pub mod inline;
pub use inline::dispatch_inline_handler;
```

### Step 2: Update walk_node() in converter.rs
Replace inline element handling code with dispatcher call:

**Before (lines 3874-7553):**
```rust
"strong" | "b" => {
    // 118 lines of code...
}
"em" | "i" => {
    // 113 lines of code...
}
// ... many more handlers ...
"ruby" => {
    // 180 lines of code...
}
```

**After:**
```rust
tl::Node::Tag(tag) => {
    // ... existing code ...
    if inline::dispatch_inline_handler(
        tag_name.as_ref(),
        node_handle,
        parser,
        output,
        options,
        ctx,
        depth,
        dom_ctx,
    ) {
        return; // Element was handled by inline dispatcher
    }

    // ... continue with block elements ...
```

### Step 3: Make Helper Functions Public
Change visibility of helper functions:
- `pub(crate) fn chomp_inline(...)`
- `pub(crate) fn escape_link_label(...)`
- `pub(crate) fn append_markdown_link(...)`
- etc.

Or create `converter/inline/helpers.rs` module with re-exports.

### Step 4: Testing
- Verify no functionality changes
- Run existing test suite
- Check feature flags (visitor, metadata)
- Validate all inline elements still render correctly

---

## Feature Flag Mapping

| Feature | Used In | Code |
|---------|---------|------|
| `visitor` | emphasis, link, code, semantic | `#[cfg(feature = "visitor")]` blocks |
| `metadata` | link | Link metadata collection |
| `inline-images` | (not in inline handlers) | References in context |

---

## Element Coverage

### Emphasis Module (4 elements)
- `<strong>` → Strong text
- `<b>` → Bold text
- `<em>` → Emphasized text
- `<i>` → Italic text

### Link Module (1 element)
- `<a>` → Hyperlinks with href, title, rel, etc.

### Code Module (3 elements)
- `<code>` → Inline code with backtick escaping
- `<kbd>` → Keyboard input
- `<samp>` → Sample output

### Semantic Module (8 elements)
- `<mark>` → Highlighted text (multiple styles)
- `<del>` → Deleted text (strikethrough)
- `<s>` → Strikethrough
- `<ins>` → Inserted text (underline)
- `<u>` → Underline
- `<small>` → Small text (pass-through)
- `<sub>` → Subscript
- `<sup>` → Superscript

### Ruby Module (5 elements)
- `<ruby>` → Ruby container
- `<rb>` → Ruby base text
- `<rt>` → Ruby text annotation
- `<rp>` → Ruby parenthesis (fallback)
- `<rtc>` → Ruby text container (CJK)

**Total: 21 HTML element handlers extracted**

---

## Notes for Implementation

1. **Preserve walk_node recursion:** All handlers call `walk_node()` recursively for child processing
2. **Context cloning:** Handlers clone context with modified flags (e.g., `in_code: true`)
3. **Output buffering:** Use intermediate `String` buffers for content collection
4. **Visitor pattern:** Each handler has optional visitor callback integration
5. **Metadata collection:** Links collect metadata when feature enabled
6. **Feature flags:** Maintain all conditional compilation blocks

---

## File Organization

```
crates/html-to-markdown/src/converter/
├── mod.rs          (new: declares inline module)
├── converter.rs    (modified: calls dispatcher)
├── block/
│   ├── mod.rs
│   ├── blockquote.rs
│   ├── heading.rs
│   ├── paragraph.rs
│   └── preformatted.rs
├── inline/         (NEW DIRECTORY)
│   ├── mod.rs      (dispatcher + exports)
│   ├── emphasis.rs (4 elements)
│   ├── link.rs     (1 element)
│   ├── code.rs     (3 elements)
│   ├── semantic.rs (8 elements)
│   ├── ruby.rs     (5 elements)
│   └── helpers.rs  (optional: shared utilities)
└── utility/
    ├── mod.rs
    ├── attributes.rs
    ├── caching.rs
    ├── content.rs
    ├── preprocessing.rs
    ├── serialization.rs
    └── siblings.rs
```

---

## Validation Checklist

- [ ] All 21 inline element handlers extracted
- [ ] Module structure mirrors block module pattern
- [ ] Dispatcher function created in inline/mod.rs
- [ ] Helper functions made accessible (public scope)
- [ ] Visitor feature gates preserved
- [ ] Metadata collection preserved (feature gate)
- [ ] walk_node() integration updated
- [ ] No functionality changes to output
- [ ] Test suite passes
- [ ] Feature flags work correctly
- [ ] No compilation warnings

---

## Reference Implementation

The block module (`converter/block/mod.rs`) serves as the reference pattern for this extraction.
Follow the same structure, dispatcher logic, and re-export pattern.
