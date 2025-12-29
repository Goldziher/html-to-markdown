# Whitespace Handling in html-to-markdown

## Overview

Whitespace handling in html-to-markdown is a critical aspect of conversion fidelity. The library provides multiple modes to control how multiple newlines, indentation, spaces, and tabs are handled during HTML-to-Markdown conversion.

## Core Philosophy

**Principle:** "Preserve exactly as it appears in HTML source"

By default, html-to-markdown does NOT apply HTML5's automatic whitespace collapsing rules. Instead:
- Text nodes retain original spacing (multiple spaces, tabs preserved)
- Newlines in text content preserved exactly
- Full control delegated to user via options

This differs from browsers, which normalize whitespace according to CSS/display rules.

## Whitespace Mode Configuration

Located in `/crates/html-to-markdown/src/options.rs`:

### WhitespaceMode Enum

```rust
pub enum WhitespaceMode {
    #[default]
    Normalized,  // Collapse multiple spaces to single; normalize newlines
    Strict,      // Preserve every space and newline exactly
}
```

### How Modes Differ

#### Normalized Mode (Default)

```rust
let options = ConversionOptions {
    whitespace_mode: WhitespaceMode::Normalized,
    ..Default::default()
};

let html = "<p>Hello    world\n\n\nwith    spaces</p>";
// Multiple spaces → single space
// Multiple newlines → single newline
let markdown = convert(html, Some(options))?;
→ "Hello world with spaces\n"
```

**Rules:**
1. Multiple consecutive spaces → single space
2. Sequences of newlines/tabs → single newline
3. Leading/trailing whitespace per element → trimmed
4. Indentation within code blocks → preserved

**Use Cases:**
- Web content (where visual spacing matters, not literal spaces)
- Content management systems (where source formatting is inconsistent)
- User-generated content (where extra spaces are usually accidents)

#### Strict Mode

```rust
let options = ConversionOptions {
    whitespace_mode: WhitespaceMode::Strict,
    ..Default::default()
};

let html = "<p>Hello    world\n\n\nwith    spaces</p>";
// Every space and newline preserved exactly
let markdown = convert(html, Some(options))?;
→ "Hello    world\n\n\nwith    spaces\n"
```

**Rules:**
1. Every space preserved (even if sequential)
2. Every newline preserved (even if multiple)
3. Tabs kept as-is
4. No trimming of whitespace

**Use Cases:**
- Poetry and verse (where line breaks matter)
- ASCII art and diagrams
- Code documentation (preserve source formatting)
- Preformatted content (with semantic meaning)

## Preprocessing Options

Located in `/crates/html-to-markdown/src/options.rs`:

### PreprocessingOptions Structure

```rust
pub struct PreprocessingOptions {
    pub strip_newlines: bool,              // Remove \n from text content
    pub collapse_whitespace: bool,         // Multiple spaces → single
    pub remove_empty_elements: bool,       // Skip empty <p>, <div>, etc.
    pub trim_text_nodes: bool,             // Strip leading/trailing whitespace
    pub normalize_unicode: bool,           // NFC normalization
    pub remove_comments: bool,             // Strip HTML comments
    pub remove_empty_paragraphs: bool,     // Skip <p></p>
}

pub enum PreprocessingPreset {
    Minimal,      // Least processing
    #[default]
    Standard,     // Balanced
    Aggressive,   // Maximum cleanup
}
```

### Preset Definitions

**Minimal:**
- Minimal preprocessing
- Preserve source structure
- Use for critical content needing exact fidelity

**Standard (Default):**
- Balanced processing
- Remove obvious junk (empty paragraphs, HTML comments)
- Reasonable space collapsing
- Good for general web content

**Aggressive:**
- Heavy preprocessing
- Remove all empty elements
- Unicode normalization
- Maximum space collapsing
- Use for content analysis/indexing

### Configuration Example

```rust
let options = ConversionOptions {
    preprocessing: PreprocessingOptions {
        strip_newlines: false,
        collapse_whitespace: true,
        remove_empty_elements: true,
        trim_text_nodes: true,
        normalize_unicode: false,
        remove_comments: true,
        remove_empty_paragraphs: true,
    },
    whitespace_mode: WhitespaceMode::Normalized,
    ..Default::default()
};
```

## Newline Style Configuration

Located in `/crates/html-to-markdown/src/options.rs`:

### NewlineStyle Enum

Controls how `<br>` tags are rendered:

```rust
pub enum NewlineStyle {
    #[default]
    Spaces,      // Two spaces at end of line
    Backslash,   // Backslash at end of line
}
```

### Rendering Differences

#### Spaces Style (Default)

```markdown
Line 1
Line 2
```

**Generated:**
```
Line 1  \n
Line 2\n
```

**Markdown parsers recognize:** Two spaces before newline as hard line break

**HTML Input:**
```html
<p>Line 1<br>Line 2</p>
```

**Markdown Output:**
```markdown
Line 1
Line 2
```

#### Backslash Style

```markdown
Line 1\
Line 2
```

**Generated:**
```
Line 1\\n
Line 2\n
```

**Markdown parsers recognize:** Backslash before newline as hard line break

**HTML Input:**
```html
<p>Line 1<br>Line 2</p>
```

**Markdown Output:**
```markdown
Line 1\
Line 2
```

### Which Style to Use?

| Style | Pros | Cons | Use Case |
|-------|------|------|----------|
| Spaces | Standard, widely supported, visual | Invisible, can be lost in editing | Default choice |
| Backslash | Visible, explicit, CommonMark spec | Less common support | Standards-strict, visibility preferred |

### Configuration

```rust
let options = ConversionOptions {
    newline_style: NewlineStyle::Backslash,  // Override default
    ..Default::default()
};

let html = "<p>A<br>B<br>C</p>";
let markdown = convert(html, Some(options))?;
// With Backslash: "A\\\nB\\\nC\n"
// With Spaces:    "A  \nB  \nC\n"
```

## List Indentation Configuration

Located in `/crates/html-to-markdown/src/options.rs`:

### ListIndentType Enum

Controls indentation for nested lists:

```rust
pub enum ListIndentType {
    #[default]
    Spaces,  // 2 or 4 spaces per level
    Tabs,    // One tab per level
}
```

### Visual Comparison

#### Spaces Indentation (Default)

```markdown
- Item 1
  - Nested 1.1
    - Deeply nested 1.1.1
- Item 2
  - Nested 2.1
```

**Generated:**
```
- Item 1\n
  - Nested 1.1\n
    - Deeply nested 1.1.1\n
- Item 2\n
  - Nested 2.1\n
```

**Characteristics:**
- 2 spaces per nesting level (configurable)
- Clear visual hierarchy
- Works with all Markdown parsers
- Visible in most editors

#### Tabs Indentation

```markdown
- Item 1
	- Nested 1.1
		- Deeply nested 1.1.1
- Item 2
	- Nested 2.1
```

**Generated:**
```
- Item 1\n
\t- Nested 1.1\n
\t\t- Deeply nested 1.1.1\n
- Item 2\n
\t- Nested 2.1\n
```

**Characteristics:**
- One `\t` per nesting level
- Compact representation
- Variable visual width (editor-dependent)
- Some parsers require specific tab stops

### Configuration

```rust
let options = ConversionOptions {
    list_indent_type: ListIndentType::Tabs,
    ..Default::default()
};

let html = "<ul><li>A<ul><li>B</li></ul></li></ul>";
let markdown = convert(html, Some(options))?;
// With Tabs: "- A\n\t- B\n"
// With Spaces: "- A\n  - B\n"
```

## Text Normalization

Located in `/crates/html-to-markdown/src/text.rs`:

### Whitespace Normalization Function

```rust
pub fn normalize_whitespace_cow(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed("");
    }

    // Check if normalization needed
    let needs_norm = text
        .split_whitespace()
        .count() != text.split_whitespace().count();

    if !needs_norm && !text.starts_with(' ') && !text.ends_with(' ') {
        return Cow::Borrowed(text);  // No-op if already normalized
    }

    let words = text.split_whitespace().collect::<Vec<_>>();
    let result = words.join(" ");
    Cow::Owned(result)
}
```

**Algorithm:**
1. Split on whitespace (matches `\s+`)
2. Collect non-empty tokens
3. Join with single space

**Examples:**
```
"hello    world"      → "hello world"
"  leading/trailing  " → "leading/trailing"
"multiple\n\nlines"   → "multiple lines"
"text\twith\ttabs"    → "text with tabs"
```

### Entity Decoding and Normalization

Text processing flow:

```
Raw HTML text
    |
    +-- Decode HTML entities: &amp; → &
    |
    +-- Normalize whitespace (if mode = Normalized)
    |
    +-- Trim leading/trailing spaces
    |
    +-- Escape special Markdown characters
    |
    +-- Output
```

**Example:**
```
Input HTML:  "<p>&nbsp;&nbsp;Hello&nbsp;&nbsp;&nbsp;world&nbsp;</p>"
After decode: "  Hello   world "
After normalize: "Hello world"
After escape (misc): "Hello world"
Final: "Hello world\n"
```

## Wrapping Configuration

Located in `/crates/html-to-markdown/src/wrapper.rs`:

### Wrap Options

```rust
pub struct ConversionOptions {
    pub wrap: bool,              // Enable wrapping
    pub wrap_width: usize,       // Line width (default: 80)
    pub wrap_preserve_words: bool, // Don't break mid-word
}
```

### Wrapping Behavior

**Without wrapping (default):**
```rust
let options = ConversionOptions {
    wrap: false,
    ..Default::default()
};

let markdown = "This is a very long line that would normally wrap at 80 characters if wrapping was enabled but since it's disabled it stays on one line.\n";
```

**With wrapping at 80 characters:**
```rust
let options = ConversionOptions {
    wrap: true,
    wrap_width: 80,
    ..Default::default()
};

let markdown = "This is a very long line that would normally wrap at 80\ncharacters if wrapping was enabled but since it's\ndisabled it stays on one line.\n";
```

### Word Boundary Preservation

```rust
pub struct ConversionOptions {
    pub wrap_preserve_words: bool,  // true = don't break words
}
```

**With word preservation (true):**
```
Line with a very_long_word_that_exceeds_wrap_width...
→ "Line with a\nvery_long_word_that_exceeds_wrap_width..."
```

**Without word preservation (false):**
```
Line with a very_long_word_that_exceeds_wrap_width...
→ "Line with a\nvery_long_word_that_exce\neds_wrap_width..."
```

## Practical Whitespace Examples

### Example 1: Poetry with Preserved Spacing

```html
<pre>
     The     forest     is     dark
          and full      of    secrets
               sleeping   in   moonlight
</pre>
```

**Conversion with Strict mode:**
```rust
let options = ConversionOptions {
    whitespace_mode: WhitespaceMode::Strict,
    code_block_style: CodeBlockStyle::Indented,
    ..Default::default()
};

let markdown = convert(html, Some(options))?;
```

**Output preserves exact spacing:**
```markdown
     The     forest     is     dark
          and full      of    secrets
               sleeping   in   moonlight
```

### Example 2: Web Content Cleanup

```html
<p>
    Welcome   to    our
    site!   We    have   great
    content    for    you.
</p>
```

**Conversion with Normalized mode (default):**
```rust
let markdown = convert(html, None)?;  // Uses default options
```

**Output (spaces collapsed):**
```markdown
Welcome to our site! We have great content for you.
```

### Example 3: Code with Tab Indentation

```html
<ul>
<li>Feature 1
<ul>
<li>Sub-feature 1.1</li>
<li>Sub-feature 1.2</li>
</ul>
</li>
<li>Feature 2</li>
</ul>
```

**With Tab indentation:**
```rust
let options = ConversionOptions {
    list_indent_type: ListIndentType::Tabs,
    ..Default::default()
};

let markdown = convert(html, Some(options))?;
```

**Output:**
```markdown
- Feature 1
	- Sub-feature 1.1
	- Sub-feature 1.2
- Feature 2
```

### Example 4: Wrapped Long Paragraphs

```html
<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
```

**Without wrapping:**
```rust
let markdown = convert(html, None)?;
```

**Output (single line):**
```markdown
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
```

**With wrapping at 60 chars:**
```rust
let options = ConversionOptions {
    wrap: true,
    wrap_width: 60,
    ..Default::default()
};

let markdown = convert(html, Some(options))?;
```

**Output (multiple lines):**
```markdown
Lorem ipsum dolor sit amet, consectetur adipiscing
elit, sed do eiusmod tempor incididunt ut labore et
dolore magna aliqua.
```

## Performance Impact of Whitespace Options

### Benchmarking Results

From `/tools/benchmark-harness/`:

| Option | Impact | Notes |
|--------|--------|-------|
| Strict mode | +0% | No extra processing |
| Normalized mode | +2-3% | Single regex pass |
| Wrapping (80 chars) | +5-10% | Line-by-line processing |
| Unicode normalization | +1-2% | Optional feature |
| Comment removal | +1% | Single pass removal |

### Optimization Strategies

1. **Use Strict mode for large documents** if whitespace preservation needed
   - Avoids normalization regex pass
   - Better memory efficiency

2. **Disable wrapping** if not needed
   - Saves line-splitting overhead
   - Useful for programmatic processing

3. **Batch preprocessing** before conversion
   - Remove comments before conversion
   - Strip empty elements in preprocessing step

## Implementation Location

**Core Files:**

- `/crates/html-to-markdown/src/options.rs` - All whitespace option definitions
  - `WhitespaceMode` enum (lines 42-57)
  - `NewlineStyle` enum (lines 59-76)
  - `ListIndentType` enum (lines 25-40)
  - `PreprocessingOptions` struct
  - `PreprocessingPreset` enum

- `/crates/html-to-markdown/src/text.rs` - Text processing
  - `normalize_whitespace_cow()` function
  - `decode_html_entities_cow()` function
  - `escape()` function (whitespace-aware)

- `/crates/html-to-markdown/src/converter.rs` - Element conversion
  - Integration with WhitespaceMode throughout
  - Text node handling in `convert_text()`
  - Newline style in `convert_br()`
  - List indentation in `convert_list()`

- `/crates/html-to-markdown/src/wrapper.rs` - Line wrapping
  - `wrap_markdown()` function
  - Word boundary detection

- `/crates/html-to-markdown/src/lib.rs` - Preprocessing
  - `normalize_line_endings()` function (lines 149-155)
  - `fast_text_only()` function (lines 157-197)

## Quick Reference: Configuration Cheat Sheet

```rust
// Default options (sensible for most web content)
ConversionOptions::default()
→ Normalized whitespace, Spaces newlines, Space indentation, No wrapping

// Poetry/ASCII art (preserve exact spacing)
ConversionOptions {
    whitespace_mode: WhitespaceMode::Strict,
    ..Default::default()
}

// Readable web conversion with line wrapping
ConversionOptions {
    wrap: true,
    wrap_width: 80,
    whitespace_mode: WhitespaceMode::Normalized,
    ..Default::default()
}

// Tab-indented nested lists
ConversionOptions {
    list_indent_type: ListIndentType::Tabs,
    ..Default::default()
}

// Strict CommonMark compliance
ConversionOptions {
    newline_style: NewlineStyle::Backslash,
    whitespace_mode: WhitespaceMode::Normalized,
    ..Default::default()
}

// Aggressive cleanup for content indexing
ConversionOptions {
    preprocessing: PreprocessingOptions {
        collapse_whitespace: true,
        remove_comments: true,
        remove_empty_elements: true,
        ..Default::default()
    },
    whitespace_mode: WhitespaceMode::Normalized,
    ..Default::default()
}
```

## Testing Whitespace Handling

Located throughout `/crates/html-to-markdown/tests/`:

```bash
# Run whitespace-specific tests
task rust:test -- --exact "test_whitespace"
task rust:test -- --exact "test_normalize"
task rust:test -- --exact "test_wrap"
```

**Test patterns:**
- Multiple spaces collapsing
- Newline normalization
- Tab handling in lists
- Wrapping at boundaries
- Preserve mode behavior
- Entity + whitespace combinations
