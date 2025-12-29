# Edge Case Handling in html-to-markdown

## Overview

The html-to-markdown converter handles numerous edge cases and adversarial inputs with robust validation, error recovery, and fallback mechanisms. This skill documents binary detection, encoding issues, malformed HTML recovery, and other robustness strategies.

## Binary File Detection

### Multi-Layer Binary Detection Strategy

The converter implements multiple detection techniques to prevent binary data from being processed as HTML.

#### Layer 1: Magic Prefix Detection

Located in `/crates/html-to-markdown/src/lib.rs` (lines 50-57, 111-118):

```rust
const BINARY_MAGIC_PREFIXES: &[(&[u8], &str)] = &[
    (b"\x1F\x8B", "gzip-compressed data"),      // gzip
    (b"\x28\xB5\x2F\xFD", "zstd-compressed data"), // zstd
    (b"PK\x03\x04", "zip archive"),             // ZIP
    (b"PK\x05\x06", "zip archive"),
    (b"PK\x07\x08", "zip archive"),
    (b"%PDF-", "PDF data"),                      // PDF
];

fn detect_binary_magic(bytes: &[u8]) -> Option<&'static str> {
    for (prefix, label) in BINARY_MAGIC_PREFIXES {
        if bytes.starts_with(prefix) {
            return Some(*label);
        }
    }
    None
}
```

**Detection Flow:**
```
Input bytes
    |
    +-- Check for known magic signatures (first 4-8 bytes)
    |
    +-- Match found? Return error immediately
    |
    +-- No match? Continue to next detection layer
```

**Example:**
```rust
// Gzip-compressed file
let html = b"\x1F\x8B\x08\x00...gzipped content...".to_vec();
let result = convert(&String::from_utf8_lossy(&html), None);
// Error: "binary data detected (gzip-compressed data)"
```

#### Layer 2: UTF-16 Encoding Detection

Located in `/crates/html-to-markdown/src/lib.rs` (lines 120-147):

**BOM (Byte Order Mark) Detection:**
```rust
// UTF-16LE BOM
if bytes.starts_with(b"\xFF\xFE") {
    return Some("UTF-16LE BOM");
}

// UTF-16BE BOM
if bytes.starts_with(b"\xFE\xFF") {
    return Some("UTF-16BE BOM");
}
```

**Heuristic UTF-16 Detection (without BOM):**
```rust
const BINARY_UTF16_NULL_RATIO: f64 = 0.2;  // 20% null bytes

// Count null bytes in sample
let nul_ratio = nul_count as f64 / sample_len as f64;
if nul_ratio < BINARY_UTF16_NULL_RATIO {
    return None;  // Not enough nulls for UTF-16
}

// UTF-16 has consistent null pattern (even or odd positions)
let dominant_ratio = (even_nul_count.max(odd_nul_count) as f64) / nul_count as f64;
if dominant_ratio >= 0.9 {
    return Some("UTF-16 data without BOM");
}
```

**Examples:**

```
UTF-16LE without BOM: <\0h\0t\0m\0l\0>\0
- Nulls at even indices (2, 4, 6, 8, 10, 12)
- Detected as UTF-16 heuristic

UTF-16BE without BOM: \0<\0h\0t\0m\0l\0>
- Nulls at odd indices (1, 3, 5, 7, 9, 11)
- Detected as UTF-16 heuristic
```

#### Layer 3: Control Character Detection

Located in `/crates/html-to-markdown/src/lib.rs` (lines 71-106):

```rust
const BINARY_SCAN_LIMIT: usize = 8192;      // Scan first 8KB
const BINARY_CONTROL_RATIO: f64 = 0.3;      // 30% threshold

fn validate_input(html: &str) -> Result<()> {
    let bytes = html.as_bytes();
    if bytes.is_empty() {
        return Ok(());
    }

    // Check magic prefixes first
    if let Some(label) = detect_binary_magic(bytes) {
        return Err(ConversionError::InvalidInput(format!(
            "binary data detected ({label}); decode/decompress to UTF-8 HTML first"
        )));
    }

    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    let mut control_count = 0usize;

    for &byte in bytes[..sample_len].iter() {
        let is_control = (byte < 0x09) || (0x0E..0x20).contains(&byte);
        if is_control {
            control_count += 1;
        }
    }

    let control_ratio = control_count as f64 / sample_len as f64;
    if control_ratio > BINARY_CONTROL_RATIO {
        return Err(ConversionError::InvalidInput(
            "binary data detected (excess control bytes)".to_string(),
        ));
    }

    Ok(())
}
```

**Control Character Ranges:**
- `0x00-0x08`: NUL, SOH, STX, ETX, EOT, ENQ, ACK, BEL, BS
- `0x0E-0x1F`: Shift Out through Unit Separator (except TAB 0x09, LF 0x0A, CR 0x0D)
- Threshold: If > 30% are control characters in sample, reject as binary

**Examples:**
```
HTML with 35% control chars: REJECTED
"Hello\x00\x01\x02World\x03\x04\x05..."
     ^^^^^^         ^^^^^^^ = 6/8 sample = 75% > 30%

Text with \r\n newlines: ACCEPTED
"Hello\r\nWorld\r\n" = 2 CR, 2 LF within normal ranges (0x0D, 0x0A allowed)
```

### Detection Performance

- **Magic prefix**: O(1), first 8 bytes checked
- **Full sample scan**: O(8192) = O(1) constant
- **Total**: < 1ms for all checks
- **Position**: Happens before any HTML parsing

### Error Messages

All binary detection errors return `ConversionError::InvalidInput`:

```rust
pub enum ConversionError {
    InvalidInput(String),  // Binary, encoding, or malformed
    ParseError(String),
    ConfigError(String),
    Panic(String),
    Other(String),
}
```

**Example Error Flow:**
```rust
let html = read_file("archive.zip");  // Binary ZIP file
match convert(&html, None) {
    Err(ConversionError::InvalidInput(msg)) => {
        eprintln!("Cannot convert: {}", msg);
        // Error: "binary data detected (zip archive); decode/decompress to UTF-8 HTML first"
    }
    _ => {}
}
```

## UTF-16 Encoding Issues

### Why UTF-16 Causes Problems

1. **Two bytes per character minimum**: ASCII "a" = `\x61\x00` in UTF-16LE or `\x00\x61` in UTF-16BE
2. **HTML requires UTF-8 or explicit charset**: HTTP headers or `<meta charset>` specify UTF-8
3. **Common mistake**: Saving HTML from Windows Notepad in UTF-16 without BOM
4. **JavaScript/web incompatibility**: Browsers expect UTF-8 HTML

### Conversion Strategy for UTF-16

Since detection happens before parsing:

```rust
// User receives error
Err("binary data detected (UTF-16LE BOM); decode to UTF-8 HTML first")

// User must decode first:
let utf16_html = fs::read("utf16.html").unwrap();  // Vec<u8>
let utf8_html = String::from_utf16_le(&utf16_html)
    .expect("decode UTF-16LE to UTF-8")
    .into_string();
let markdown = convert(&utf8_html, None)?;
```

### Python Example (Common Case)

```python
# Common mistake: file saved as UTF-16
with open('document.html', 'rb') as f:
    raw_bytes = f.read()

# Detect and convert
try:
    # Try UTF-16LE (Windows)
    html = raw_bytes.decode('utf-16-le')
except UnicodeDecodeError:
    # Try UTF-16BE (Mac)
    html = raw_bytes.decode('utf-16-be')

# Now convert
markdown = html_to_markdown.convert(html)
```

## Malformed HTML Recovery

### Parser Fallback Strategy

The converter uses two parsers with different robustness levels:

**astral-tl (fast, primary):**
- Works well for well-formed HTML
- Fails gracefully on malformed markup
- Used by default in `/crates/html-to-markdown/src/converter.rs`

**html5ever (robust, fallback):**
- Full HTML5 spec compliance
- Automatic error recovery
- Tag closure, namespace handling
- Used when astral-tl insufficient (feature-gated)

### Malformed HTML Examples

#### Example 1: Unclosed Tags

```html
<!-- Input -->
<div>
  <p>Paragraph 1
  <p>Paragraph 2
</div>

<!-- astral-tl behavior: Treats as flat text flow -->
<!-- html5ever behavior: Auto-closes <p>, handles nesting -->
```

**Output:**
```markdown
Paragraph 1

Paragraph 2
```

#### Example 2: Mismatched Nesting

```html
<!-- Input: Divs inside table cells (invalid structure) -->
<table>
  <tr>
    <td>
      <div>Content in div</div>
    </td>
</table>
<!-- Missing closing </tr>, </td> -->

<!-- astral-tl: Best-effort recovery -->
<!-- html5ever: Proper reconstruction of table structure -->
```

#### Example 3: Invalid Characters in Attributes

```html
<!-- Input: Unescaped quotes in attribute values -->
<a href="page?a=1&b=2">Link</a>

<!-- astral-tl: Attribute parsing stops at first "
<a href="page?a=1">
<!-- html5ever: Proper entity decoding (&amp; → &) -->
<a href="page?a=1&amp;b=2">
```

### Recovery Mechanisms

#### 1. Whitespace Normalization

Before parsing, normalize line endings:

```rust
fn normalize_line_endings(html: &str) -> Cow<'_, str> {
    if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    }
}
```

**Input:** `<p>Line1\r\nLine2\r<br></p>` (mixed CRLF, CR, LF)
**Output:** `<p>Line1\nLine2\n<br></p>` (normalized to LF)

#### 2. Fast Text Path

For text-only inputs (no `<` character), skip HTML parsing entirely:

```rust
fn fast_text_only(html: &str, options: &ConversionOptions) -> Option<String> {
    if html.contains('<') {
        return None;  // Has HTML, can't use fast path
    }

    // Apply text processing directly
    let decoded = text::decode_html_entities_cow(html);
    let normalized = if options.whitespace_mode == WhitespaceMode::Normalized {
        text::normalize_whitespace_cow(&decoded)
    } else {
        Cow::Borrowed(&decoded)
    };

    Some(normalized.into_owned() + "\n")
}
```

**Benefit:** Plain text documents converted without parsing overhead

#### 3. Entity Decoding

Handle HTML entities even with malformed markup:

```rust
// From text.rs
pub fn decode_html_entities_cow(text: &str) -> Cow<'_, str> {
    // Handles:
    // &amp; → &
    // &#123; → {
    // &#x1F; → (Unicode char)
    // &nbsp; → (non-breaking space)
    // ... 100+ HTML5 entities
}
```

**Examples:**
```
&lt;script&gt;  → <script>
&quot;test&quot; → "test"
&#8364;         → (Euro symbol)
&#x1F600;       → (Emoji)
```

#### 4. Attribute Error Recovery

When parser encounters malformed attributes, graceful degradation:

```html
<!-- Input -->
<img src="image.jpg" alt=Unquoted width=100 data-x='mixed quote>

<!-- Outcome: Some attributes lost, image still converts -->
![Unquoted](image.jpg)
```

## Panic Recovery

Located in `/crates/html-to-markdown/src/error.rs`:

```rust
pub enum ConversionError {
    Panic(String),  // Catches panics in conversion pipeline
}
```

### Panic Catching Strategy

Bindings wrap conversion in catch-panic:

```rust
// From Python bindings
#[pyfunction]
fn convert(html: String, options: Option<PyObject>) -> PyResult<String> {
    let result = std::panic::catch_unwind(|| {
        html_to_markdown_rs::convert(&html, None)
    });

    match result {
        Ok(Ok(md)) => Ok(md),
        Ok(Err(e)) => Err(PyErr::new::<PyException, _>(e.to_string())),
        Err(_) => Err(PyErr::new::<PyException, _>("panic in conversion")),
    }
}
```

### Safe Error Paths

Never unwrap in conversion code:

```rust
// WRONG - Can panic
let tag_name = element.name().expect("tag name");

// CORRECT - Returns Result
match element.name() {
    Some(name) => { /* use name */ },
    None => { /* handle missing name gracefully */ },
}
```

## Empty/Null Input Handling

### Validation Rules

```rust
fn validate_input(html: &str) -> Result<()> {
    let bytes = html.as_bytes();
    if bytes.is_empty() {
        return Ok(());  // Empty string is valid
    }

    // Continue with other checks
    // ...
}
```

### Output for Empty Input

```rust
convert("", None)     → Ok("\n")           // Single newline
convert("   ", None)  → Ok("\n")           // Whitespace stripped
convert(null, None)   → Error (from binding)  // Python None → error
```

## Large Document Handling

### Memory Constraints

The converter handles large documents efficiently:

**Tested sizes:**
- Typical document: 1-10 MB HTML → < 100 MB peak memory
- Large document: 50+ MB HTML → proportional memory growth
- Maximum: Limited by system RAM, no inherent limit

### Streaming Considerations

For extremely large documents (> 100 MB), consider:

1. **Streaming parsing** (future enhancement):
   - Process HTML in chunks
   - Emit markdown incrementally

2. **Document splitting**:
   - Break HTML by `<div>` sections
   - Convert each part separately
   - Concatenate results

3. **Chunked metadata**:
   - Extract only document metadata (< 1KB)
   - Skip headers/links for size savings

### Memory Optimization

```rust
// Use pre-allocated buffers
let mut output = String::with_capacity(html.len());

// LRU cache for common patterns
static PATTERN_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())));

// Metadata size-bounded
config.max_structured_data_size = 10 * 1024 * 1024;  // 10 MB limit
```

## Malformed Table Recovery

### Invalid Table Structures

```html
<!-- Missing tbody, thead -->
<table>
  <tr><td>Cell 1</td></tr>
</table>

<!-- Cells spanning beyond row width -->
<table>
  <tr>
    <td colspan="5">Wide cell</td>
    <td>Normal</td>
  </tr>
</table>

<!-- Nested tables (complex) -->
<table>
  <tr>
    <td>
      <table><tr><td>Inner</td></tr></table>
    </td>
  </tr>
</table>
```

### Conversion Behavior

**Single-cell table:**
```html
<table><tr><td>Data</td></tr></table>
→ | Data |
  |------|
```

**Colspan expansion:**
```html
<table>
  <tr><td colspan="3">Wide</td></tr>
  <tr><td>A</td><td>B</td><td>C</td></tr>
</table>
→ | Wide | | |
  |------|---|---|
  | A | B | C |
```

**Nested tables:**
Converts outer table, treats inner table as cell content:
```html
<table><tr><td>Cell with [nested table] inside</td></tr></table>
→ Outer table with nested table markdown in cell
```

## Quote and Escape Edge Cases

### Special Character Escaping

Based on `ConversionOptions`:

```rust
pub struct ConversionOptions {
    pub escape_misc: bool,           // \ & < ` [ > ~ # = + | -
    pub escape_asterisks: bool,      // *
    pub escape_underscores: bool,    // _
    pub escape_ascii: bool,          // All ASCII punctuation
}
```

### Problematic Patterns

```
Input: "Price: $10 & shipping"
escape_misc=true  → "Price: $10 \& shipping"

Input: "*not bold*"
escape_asterisks=true → "\*not bold\*"

Input: "Text_with_underscore"
escape_underscores=true → "Text\_with\_underscore"

Input: "1. First point"
No explicit option, but auto-escaped to "1\\. First point" for ordered list safety
```

## Performance Edge Cases

### Slow Cases

1. **Deep nesting** (h1 inside 50 nested divs)
   - O(depth) memory for context stack
   - ~1ms per 10 levels

2. **Large attribute maps**
   - Elements with 50+ attributes
   - O(n) linear lookup in visitor pattern

3. **Regex escaping**
   - Many special characters in text
   - Multiple regex replacements sequential

### Optimization Advice

```rust
// Cache compiled regex patterns (already done in text.rs)
static ESCAPE_MISC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([\\&<`\[\]>~#=+|\-])").unwrap());

// Use iterators instead of allocating vectors
for byte in bytes.iter() {
    // Process in-place
}

// Limit depth checks in visitor
if ctx.depth > 100 {
    return VisitResult::Skip;  // Avoid stack exhaustion
}
```

## Error Types and Recovery

### ConversionError Variants

```rust
pub enum ConversionError {
    InvalidInput(String),    // Binary, encoding, empty (recoverable)
    ParseError(String),      // HTML parsing failed (unrecoverable)
    ConfigError(String),     // Invalid config (unrecoverable)
    Panic(String),          // Unexpected panic (unrecoverable)
    Other(String),          // Misc errors (mixed)
}
```

### Recovery Strategy by Error Type

| Error | Cause | Recovery |
|-------|-------|----------|
| InvalidInput | UTF-16, gzip, control chars | User must fix input, then retry |
| ParseError | HTML unparseable | Fall back to html5ever or give up |
| ConfigError | Bad options | Fix config, retry |
| Panic | Code bug | Report as issue, workaround TBD |

## Testing Edge Cases

**Location:** `/crates/html-to-markdown/src/lib.rs` (lines 722-765)

```rust
#[test]
fn test_binary_input_rejected() {
    let html = "PDF\0DATA";
    let result = convert(html, None);
    assert!(matches!(result, Err(ConversionError::InvalidInput(_))));
}

#[test]
fn test_binary_magic_rejected() {
    let html = String::from_utf8_lossy(b"\x1F\x8B\x08\x00gzip").to_string();
    let result = convert(&html, None);
    assert!(matches!(result, Err(ConversionError::InvalidInput(_))));
}

#[test]
fn test_utf16_hint_rejected() {
    let html = String::from_utf8_lossy(b"\xFF\xFE<\0h\0t\0m\0l\0>\0").to_string();
    let result = convert(&html, None);
    assert!(matches!(result, Err(ConversionError::InvalidInput(_))));
}

#[test]
fn test_plain_text_allowed() {
    let result = convert("Just text", None).unwrap();
    assert!(result.contains("Just text"));
}
```

## Quick Reference: Edge Cases

| Input Type | Detection | Outcome | User Action |
|-----------|-----------|---------|------------|
| Empty string | Whitespace check | `Ok("\n")` | None needed |
| Plain text (no HTML) | `fast_text_only()` | Fast path OK | None |
| UTF-16 BOM | Magic prefix | Error | Decode to UTF-8 first |
| UTF-16 heuristic | Null byte pattern | Error | Decode to UTF-8 first |
| Gzip/ZIP | Magic prefix | Error | Decompress first |
| PDF | Magic prefix | Error | Extract text first |
| 35% control chars | Ratio > 0.3 | Error | Check encoding |
| Malformed HTML | astral-tl best-effort | Best-effort conversion | Check for quality |
| Deep nesting | Depth tracking | Slowdown (acceptable) | Consider chunking |
| Large document | Memory allocation | Proportional memory | Monitor RAM |
