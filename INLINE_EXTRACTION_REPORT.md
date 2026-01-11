# Inline Element Handler Code Extraction Report

## Overview

This report documents the extraction of inline element handlers from the `walk_node()` function in `crates/html-to-markdown/src/converter.rs`. The extraction identifies code locations, line counts, and element types for modularization.

## File Location Reference

**Source file:** `/Users/naamanhirschfeld/workspace/kreuzberg-dev/html-to-markdown/crates/html-to-markdown/src/converter.rs`

**Source function:** `walk_node()` (starts at line 3334)

---

## 1. inline/emphasis.rs - Strong, Bold, Emphasis, Italic

**Element types:** `<strong>`, `<b>`, `<em>`, `<i>`

### Locations in converter.rs:

#### Strong/Bold Handler (`"strong" | "b"`)
- **Line range:** 3874-3991 (118 lines)
- **Start marker:** `"strong" | "b" => {`
- **Features:**
  - Handles nested context (`in_strong` flag)
  - Processes child nodes recursively
  - Applies strong emphasis markers (`**` or configured symbol)
  - Visitor callback support (`visit_strong`)
  - Dual-compiled sections (feature-gated for visitor)
  - Helper functions: `chomp_inline()`, `append_inline_suffix()`
  - Special handling for code context (ctx.in_code)

#### Emphasis/Italic Handler (`"em" | "i"`)
- **Line range:** 3993-4105 (113 lines)
- **Start marker:** `"em" | "i" => {`
- **Features:**
  - Similar structure to strong handler
  - Single-symbol emphasis (`*` or configured)
  - Visitor callback support (`visit_emphasis`)
  - Special case: class detection for `.caret` (Bootstrap carets)
  - Code context handling

### Helper Functions Required:

```rust
// Already defined in converter.rs, needed in extraction:
fn chomp_inline(text: &str) -> (&str, &str, &str)  // Line 97
fn append_inline_suffix(...)                        // Line 3311
fn get_text_content(...)                            // Referenced
```

### Visitor Integration:
- `NodeType::Strong` → `visit_strong(&node_ctx, &text_content)`
- `NodeType::Em` → `visit_emphasis(&node_ctx, &text_content)`

---

## 2. inline/link.rs - Anchor Elements

**Element types:** `<a>` (anchor/hyperlinks)

### Location in converter.rs:

#### Link Handler (`"a"`)
- **Line range:** 4107-4375 (269 lines)
- **Start marker:** `"a" => {`
- **Features:**
  - Autolink detection (when link text matches href)
  - Heading detection (if link contains single heading child)
  - Block content handling (links with block children)
  - Link label normalization and truncation
  - Metadata collection (feature-gated)
  - Visitor callback support (`visit_link`)
  - Max link label length: 512 characters with ellipsis truncation

### Helper Functions Required:

```rust
// Defined in converter.rs:
fn escape_link_label(text: &str) -> String                    // Line 875
fn append_markdown_link(output, label, href, title, raw_text, options)  // Line 914
fn collect_link_label_text(children, parser, dom_ctx)         // Line 1512
fn normalize_link_label(label: &str) -> String                // Line 1564
fn find_single_heading_child(node_handle, parser)             // Line 979
fn get_text_content(node_handle, parser, dom_ctx)             // Reference needed
fn heading_allows_inline_images(...)                          // Reference needed
fn push_heading(output, ctx, options, level, text)            // Line 1018
fn truncate_at_char_boundary(value, max_len)                  // Line 162
```

### Metadata Tracking:
```rust
#[cfg(feature = "metadata")]
collector.borrow_mut().add_link(
    href.clone(),
    label.clone(),
    title.clone(),
    rel_attr,
    attributes_map,
);
```

### Visitor Integration:
- `NodeType::Link` → `visit_link(&node_ctx, &href, &label, title.as_deref())`

---

## 3. inline/code.rs - Inline Code Elements

**Element types:** `<code>`, `<kbd>`, `<samp>`

### Locations in converter.rs:

#### Code Handler (`"code"`)
- **Line range:** 5197-5379 (183 lines)
- **Start marker:** `"code" => {`
- **Features:**
  - Nested code context handling
  - Backtick escaping logic (handles `` ` `` inside code)
  - Delimiter space logic (when code starts/ends with backtick or space)
  - Dynamic backtick count calculation
  - Visitor callback support (`visit_code_inline`)

#### Kbd/Samp Handler (`"kbd" | "samp"`)
- **Line range:** 5092-5124 (33 lines)
- **Start marker:** `"kbd" | "samp" => {`
- **Features:**
  - Sets `in_code: true` context
  - Wraps content in single backticks
  - Whitespace normalization

### Code-Related Helper:
```rust
// Backtick counting for code blocks with embedded backticks
let max_consecutive = trimmed
    .chars()
    .fold((0, 0), |(max, current), c| {
        if c == '`' {
            let new_current = current + 1;
            (max.max(new_current), new_current)
        } else {
            (max, 0)
        }
    })
    .0;
let num = if max_consecutive == 1 { 2 } else { 1 };
```

### Visitor Integration:
- `NodeType::Code` → `visit_code_inline(&node_ctx, trimmed)`

---

## 4. inline/semantic.rs - Semantic HTML Elements

**Element types:** `<mark>`, `<del>`, `<s>`, `<ins>`, `<u>`, `<small>`, `<sub>`, `<sup>`

### Locations in converter.rs:

#### Mark Handler (`"mark"`)
- **Line range:** 4746-4802 (57 lines)
- **Start marker:** `"mark" => {`
- **Features:**
  - Style configuration (`HighlightStyle` enum)
  - Four rendering modes: `DoubleEqual`, `Html`, `Bold`, `None`
  - Respects `convert_as_inline` context

#### Del/Strikethrough Handler (`"del" | "s"`)
- **Line range:** 4804-4896 (93 lines)
- **Start marker:** `"del" | "s" => {`
- **Features:**
  - Strikethrough with `~~` markers
  - Code context handling
  - Visitor callback support (`visit_strikethrough`)

#### Ins/Underline Handler (`"ins"`)
- **Line range:** 4898-4975 (78 lines)
- **Start marker:** `"ins" => {`
- **Features:**
  - Uses `==` for underline representation
  - Visitor callback support (`visit_underline`)
  - Trims empty content

#### U Handler (`"u"`)
- **Line range:** 4977-5043 (67 lines)
- **Start marker:** `"u" => {`
- **Features:**
  - Visitor callback support
  - Fallback to children rendering if visitor continues

#### Small Handler (`"small"`)
- **Line range:** 5045-5052 (8 lines)
- **Start marker:** `"small" => {`
- **Features:**
  - Simplest handler - just pass through children

#### Sub Handler (`"sub"`)
- **Line range:** 5054-5071 (18 lines)
- **Start marker:** `"sub" => {`
- **Features:**
  - Configurable symbol (`options.sub_symbol`)
  - HTML tag wrapping support (e.g., `<sub>...</sub>`)
  - Closing tag generation via string replacement

#### Sup Handler (`"sup"`)
- **Line range:** 5073-5090 (18 lines)
- **Start marker:** `"sup" => {`
- **Features:**
  - Same pattern as sub handler
  - Uses `options.sup_symbol`

### Visitor Integration:
- `NodeType::Strikethrough` → `visit_strikethrough(&node_ctx, &text_content)`
- `NodeType::Underline` → `visit_underline(&node_ctx, &text_content)`

---

## 5. inline/ruby.rs - East Asian Ruby Annotations

**Element types:** `<ruby>`, `<rb>`, `<rt>`, `<rp>`, `<rtc>`

### Locations in converter.rs:

#### Ruby Handler (`"ruby"`)
- **Line range:** 7321-7500 (180 lines)
- **Start marker:** `"ruby" => {`
- **Features:**
  - Two rendering paths based on structure
  - Interleaved detection (`rb` followed by `rt`)
  - Base text + annotations collection
  - RTC (ruby text container) support
  - Complex sequence analysis

**Ruby Structure Detection:**
```rust
let tag_sequence: Vec<String> = // Extract child tag names
let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");
let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

// Path 1: Interleaved rb/rt pattern (simpler)
if is_interleaved && !has_rtc {
    // Process as base + immediate annotation pairs
}
// Path 2: Grouped structure with rb(s) followed by rt(s) and optional rtc
else {
    // Collect all bases, then all annotations
}
```

#### Rb Handler (`"rb"`)
- **Line range:** 7502-7511 (10 lines)
- **Start marker:** `"rb" => {`
- **Features:**
  - Base text extraction

#### Rt Handler (`"rt"`)
- **Line range:** 7513-7530 (18 lines)
- **Start marker:** `"rt" => {`
- **Features:**
  - Annotation wrapping in parentheses: `(annotation)`
  - Continuation detection (if output already ends with `(`)

#### Rp Handler (`"rp"`)
- **Line range:** 7532-7544 (13 lines)
- **Start marker:** `"rp" => {`
- **Features:**
  - Ruby parenthesis element (fallback text)
  - Only outputs non-empty content

#### Rtc Handler (`"rtc"`)
- **Line range:** 7546-7553 (8 lines)
- **Start marker:** `"rtc" => {`
- **Features:**
  - Ruby text container (CJK only)
  - Passes through children with same depth (no depth increment)

### Ruby Examples:

```html
<!-- Interleaved pattern -->
<ruby>
  <rb>base</rb><rt>annotation</rt>
  <rb>base2</rb><rt>annotation2</rt>
</ruby>
<!-- Output: baseannotation(annotation2)base2 -->

<!-- Grouped pattern -->
<ruby>
  <rb>base1</rb>
  <rb>base2</rb>
  <rt>annotation1</rt>
  <rt>annotation2</rt>
</ruby>
<!-- Output: base1base2annotation1annotation2 -->

<!-- With fallback parentheses -->
<ruby>
  base<rp>(</rp><rt>annotation</rt><rp>)</rp>
</ruby>
```

---

## Summary Statistics

| Handler | File | Elements | Start Line | End Line | Lines | Complexity |
|---------|------|----------|------------|----------|-------|------------|
| Emphasis | emphasis.rs | strong, b, em, i | 3874/3993 | 3991/4105 | 118+113 | High |
| Link | link.rs | a | 4107 | 4375 | 269 | High |
| Code | code.rs | code, kbd, samp | 5197/5092 | 5379/5124 | 183+33 | Medium |
| Semantic | semantic.rs | mark, del, s, ins, u, small, sub, sup | 4746-5090 | 4802-5090 | 345 | Low-Medium |
| Ruby | ruby.rs | ruby, rb, rt, rp, rtc | 7321-7546 | 7500-7553 | 243 | High |

**Total Lines:** ~1,184 lines across all inline handlers

---

## Architecture Notes

### Context Flags Used:
- `ctx.in_code` - Affects code element handling
- `ctx.in_ruby` - Affects text processing in ruby annotations
- `ctx.in_strong` - For nested formatting (strong within strong)
- `ctx.convert_as_inline` - Forces inline rendering
- `ctx.inline_depth` - Tracks nesting level
- `ctx.heading_allow_inline_images` - Affects links with heading children

### ConversionOptions Fields Used:
- `options.strong_em_symbol` - Symbol for strong/em (`*` or `_`)
- `options.sub_symbol` - Sub element symbol
- `options.sup_symbol` - Sup element symbol
- `options.highlight_style` - Mark rendering style
- `options.autolinks` - Enable autolink detection
- `options.default_title` - Add title to links matching text
- `options.escape_*` - Text escaping flags
- `options.escape_misc` - Escape pipe character in tables
- `options.escape_asterisks` - Escape `*` characters
- `options.escape_underscores` - Escape `_` characters
- `options.escape_ascii` - Escape non-ASCII

### DomContext Methods Used:
- `dom_ctx.parent_tag_name(node_id, parser)` - Get parent tag
- `dom_ctx.get_sibling_index(node_id)` - Get position in siblings
- `dom_ctx.tag_info(node_id, parser)` - Get cached tag info
- `dom_ctx.children_of(node_id)` - Get cached children list

### Visitor Callbacks (when feature enabled):
```rust
#[cfg(feature = "visitor")]
match visitor.visit_strong(&node_ctx, &text_content) { ... }
match visitor.visit_emphasis(&node_ctx, &text_content) { ... }
match visitor.visit_link(&node_ctx, &href, &label, title) { ... }
match visitor.visit_code_inline(&node_ctx, trimmed) { ... }
match visitor.visit_strikethrough(&node_ctx, &text_content) { ... }
match visitor.visit_underline(&node_ctx, &text_content) { ... }
```

---

## Helper Function Dependencies

All helper functions are defined at module scope in converter.rs and should be extracted to a `helpers.rs` submodule or kept accessible:

1. **Text Processing:**
   - `chomp_inline()` - Line 97
   - `normalize_whitespace()` / `normalize_whitespace_cow()` - external (text module)
   - `escape_link_label()` - Line 875
   - `normalize_link_label()` - Line 1564

2. **Link Processing:**
   - `append_markdown_link()` - Line 914
   - `collect_link_label_text()` - Line 1512
   - `find_single_heading_child()` - Line 979
   - `escape_link_label()` - Line 875

3. **Utility:**
   - `append_inline_suffix()` - Line 3311
   - `trim_trailing_whitespace()` - Line 130
   - `get_text_content()` - Referenced (external)
   - `truncate_at_char_boundary()` - Line 162
   - `heading_allows_inline_images()` - Referenced (external)

4. **Ruby-Specific:**
   - `normalized_tag_name()` - References external

---

## Integration Strategy

When creating `inline/mod.rs`, use the dispatcher pattern from `block/mod.rs`:

```rust
pub fn dispatch_inline_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::Context,
    depth: usize,
    dom_ctx: &crate::converter::DomContext,
) -> bool {
    match tag_name {
        "strong" | "b" => { emphasis::handle_strong(...); true },
        "em" | "i" => { emphasis::handle_em(...); true },
        "a" => { link::handle(...); true },
        "code" => { code::handle_code(...); true },
        "kbd" | "samp" => { code::handle_kbd_samp(...); true },
        "mark" => { semantic::handle_mark(...); true },
        "del" | "s" => { semantic::handle_strikethrough(...); true },
        "ins" => { semantic::handle_ins(...); true },
        "u" => { semantic::handle_u(...); true },
        "small" => { semantic::handle_small(...); true },
        "sub" => { semantic::handle_sub(...); true },
        "sup" => { semantic::handle_sup(...); true },
        "ruby" => { ruby::handle(...); true },
        "rb" => { ruby::handle_rb(...); true },
        "rt" => { ruby::handle_rt(...); true },
        "rp" => { ruby::handle_rp(...); true },
        "rtc" => { ruby::handle_rtc(...); true },
        _ => false,
    }
}
```

---

## Next Steps

1. Extract each handler section to its respective file
2. Create inline/mod.rs with dispatcher
3. Move helper functions or create helper submodule
4. Update walk_node() to call dispatcher for inline elements
5. Add comprehensive documentation with examples
6. Test each handler independently
7. Verify feature flags (visitor, metadata) still work correctly
