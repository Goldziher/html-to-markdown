# Rust Dependencies Analysis

## HTML Parsing & Conversion

### lol-html

- **Purpose**: Streaming HTML parser/rewriter
- **License**: BSD-3-Clause (permissive)
- **Version**: Latest (2.6.0 as of July 2025)
- **URL**: <https://github.com/cloudflare/lol-html>
- **Key Features**:
  - Streaming HTML processing with minimal buffering
  - CSS selector-based element manipulation
  - ~2x faster than html5ever
  - Low memory footprint
  - Used by Cloudflare Workers
- **Use Case**: Core HTML parsing and element handling

### ammonia

- **Purpose**: HTML sanitization
- **License**: MIT OR Apache-2.0 (permissive)
- **Version**: 4.1.2
- **URL**: <https://github.com/rust-ammonia/ammonia>
- **Key Features**:
  - Whitelist-based HTML sanitization
  - Built on html5ever (browser-grade parsing)
  - XSS prevention
  - Customizable via Builder pattern
  - 15x faster than Python's bleach
- **Use Case**: HTML preprocessing/sanitization (replaces nh3/Python preprocessing)

## API Comparison

### ammonia API

```rust
// Simple usage
let clean_html = ammonia::clean("<script>alert('xss')</script><p>Safe</p>");
// Result: "<p>Safe</p>"

// Custom configuration
use ammonia::Builder;

let mut builder = Builder::default();
builder
    .add_tags(&["custom-tag"])
    .add_tag_attributes("a", &["data-custom"])
    .strip_comments(false);

let clean_html = builder.clean(dirty_html).to_string();
```

### Key ammonia Components

**Builder fields**:

- `tags: HashSet<&str>` - Allowed HTML tags
- `clean_content_tags: HashSet<&str>` - Tags whose content should be stripped (e.g., script, style)
- `tag_attributes: HashMap<&str, HashSet<&str>>` - Allowed attributes per tag
- `generic_attributes: HashSet<&str>` - Attributes allowed on all tags
- `url_schemes: HashSet<&str>` - Allowed URL schemes
- `strip_comments: bool` - Whether to strip HTML comments
- `allowed_classes: HashMap<&str, HashSet<&str>>` - Allowed class names per tag
- `link_rel: Option<&str>` - Default rel attribute for links (default: "noopener noreferrer")

**Default allowed tags**:
a, abbr, acronym, area, article, aside, b, bdi, bdo, blockquote, br, caption, center, cite, code, col, colgroup, data, dd, del, details, dfn, div, dl, dt, em, figcaption, figure, footer, h1-h6, header, hgroup, hr, i, img, ins, kbd, li, map, mark, nav, ol, p, pre, q, rp, rt, rtc, ruby, s, samp, small, span, strike, strong, sub, summary, sup, table, tbody, td, th, thead, time, tr, tt, u, ul, var, wbr

## Integration Plan

### Preprocessing Pipeline

**Current (Python v1)**:

```
HTML → nh3.clean() (Python→Rust bindings) → BeautifulSoup (Python) → Convert
```

**New (Rust v2)**:

```
HTML → ammonia::clean() (Rust) → lol-html streaming (Rust) → Convert
```

**Migration Note**:

- v1 uses nh3 (Python bindings to ammonia)
- v2 uses ammonia directly in Rust
- Eliminates Python→Rust boundary overhead
- Expected 2-3x additional improvement from removing bindings layer

### Configuration Mapping

Python `PreprocessingOptions` → ammonia `Builder`:

- `preset` levels → Different Builder configurations
- `remove_navigation` → Custom Builder with navigation tags stripped
- `remove_forms` → Builder without form tags

### Dependencies to Add

```toml
[workspace.dependencies]
lol-html = "2.6"
ammonia = "4.1"
```

## Performance Expectations

- **ammonia**: 15x faster than bleach (Python)
- **lol-html**: 2x faster than html5ever
- Combined: Expect 10-20x performance improvement over v1

## Notes

- ammonia uses html5ever internally (DOM-based)
- lol-html is streaming-based
- For preprocessing: Use ammonia (DOM is fine for sanitization)
- For conversion: Use lol-html (streaming is critical)
- Both are actively maintained and production-ready
