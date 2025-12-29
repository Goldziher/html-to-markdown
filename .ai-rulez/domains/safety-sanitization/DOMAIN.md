# Safety & Sanitization Domain

## Purpose

The Safety & Sanitization domain protects the html-to-markdown conversion pipeline from malicious or malformed input, ensuring that converted Markdown output cannot be exploited for XSS attacks, code injection, or data exfiltration. This domain implements multi-layer defense through HTML validation, XSS prevention via the ammonia library, binary data detection, URL sanitization, attribute filtering, and runtime safety constraints. The goal is to ensure safe conversion of untrusted HTML while maintaining legitimate content structure.

## Key Responsibilities

### 1. Input Validation & Binary Detection

- **Binary Data Detection**:
  - Detect magic number prefixes (gzip, zstd, ZIP, PDF)
  - Identify UTF-16 and UTF-32 encoded content (high null byte ratios)
  - Scan first 8KB for binary indicators
  - Calculate control character ratio to detect binary corruption
  - Reject non-UTF-8 encoded input with clear error messages

- **Binary Detection Heuristics**:
  - **Magic Numbers**: Check for gzip (0x1F8B), zstd (0x28B52FFD), ZIP, PDF signatures
  - **Null Bytes**: UTF-16 detection via even/odd null byte distribution (>20% threshold)
  - **Control Characters**: Reject if >30% of sample is control chars (except tab/newline)
  - **Valid UTF-8**: Verify byte sequences match UTF-8 encoding rules

- **Encoding Handling**:
  - Detect character encoding (UTF-8, Latin-1, CP-1252, etc.)
  - Convert to UTF-8 before parsing
  - Preserve encoding declaration if present
  - Fallback to UTF-8 if encoding detection fails

- **Size Validation**:
  - Enforce maximum document size (configurable, default 50MB)
  - Enforce maximum nesting depth (configurable, default 256)
  - Prevent unbounded memory allocation

### 2. HTML Validation & Sanitization (Ammonia)

- **Ammonia Integration**:
  - Use ammonia crate for production-grade HTML sanitization
  - Apply before parsing if configured
  - Remove dangerous elements and attributes
  - Whitelist safe HTML elements and attributes

- **Element Filtering**:
  - **Always Remove**: `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, `<applet>`, `<meta>`, `<link>`, `<base>`, `<form>`
  - **Conditional Removal**: Event handlers, javascript: URLs, data: URLs
  - **Preserve Content**: Remove element tags but keep text content
  - **Allow Safe Elements**: `<p>`, `<div>`, `<h1-h6>`, `<a>`, `<img>`, `<ul>`, `<ol>`, `<li>`, `<table>`, `<code>`, `<pre>`, etc.

- **Attribute Filtering**:
  - **Remove Event Handlers**: onclick, onload, onerror, onmouseover, etc.
  - **Sanitize URLs**: Validate href and src attributes
  - **Block Protocols**: Remove javascript:, data:, vbscript:, file: schemes
  - **Whitelist Safe Attributes**: id, class, title, alt, src, href, style (limited)
  - **URL Validation**: Check protocol whitelist (http, https, mailto, ftp)

- **Style Attribute Handling**:
  - Remove dangerous CSS (expression, behavior, -moz-binding, etc.)
  - Disable CSS import, javascript: URLs in CSS
  - Preserve safe styles (color, font-size, text-align, etc.)
  - Optional: Strip all styles if configured

- **Ammonia Configuration**:
  ```rust
  let mut cleaner = ammonia::Builder::default();
  cleaner.tags(SAFE_TAGS);           // Whitelist elements
  cleaner.generic_attributes(SAFE_ATTRS); // Whitelist attributes
  cleaner.url_relative_mode(UrlRelativeMode::PassThrough);
  cleaner.link_rel(Some("noopener noreferrer".to_string()));
  let clean_html = cleaner.clean(untrusted_html).to_string();
  ```

### 3. XSS Prevention

- **Attack Vector Mitigation**:
  - **Stored XSS**: Sanitize on input via ammonia
  - **Reflected XSS**: Don't trust user-provided HTML attributes
  - **DOM XSS**: Markdown output cannot execute scripts
  - **CSS Injection**: Strip or validate CSS content

- **Dangerous HTML Patterns**:
  - `<img src=x onerror=alert(1)>` → Sanitized to `<img src="x" alt="">`
  - `<a href="javascript:void(0)">` → Converted to plain text link
  - `<style>body { background: url('javascript:...') }</style>` → Removed
  - `<svg onload=alert(1)>` → SVG removed or content extracted
  - Data URIs with scripts → Stripped or marked unsafe

- **Markdown-Level Security**:
  - Markdown output cannot contain executable content (inherently safe)
  - Links in Markdown use URL escaping
  - Code blocks preserve literal text (no interpretation)
  - Prevent Markdown escaping bypass via double-escaping

### 4. URL Sanitization

- **URL Validation Pipeline**:
  1. Extract URL from href/src attribute
  2. Parse URL components (scheme, host, path, query, fragment)
  3. Validate scheme (whitelist: http, https, mailto, ftp, file, /)
  4. Check for dangerous patterns (javascript:, data:, vbscript:)
  5. Validate host (no localhost resolution, domain validation optional)
  6. Return safe URL or fallback

- **Protocol Whitelist**:
  - **Safe**: http, https, mailto, ftp, tel, sms, geo, magnet
  - **Dangerous**: javascript, data, vbscript, file (when untrusted)
  - **Relative**: /, ./, ../ (relative path resolution)

- **URL Edge Cases**:
  - URL-encoded payload: `%6a%61%76%61%73%63%72%69%70%74%3a%61%6c%65%72%74%28%31%29` → Decoded and validated
  - Case-insensitive schemes: `JaVaScRiPt:` → Normalized and rejected
  - Null bytes: `javascript\x00:alert(1)` → Detected and rejected
  - Unicode tricks: `jаvascript:` (Cyrillic 'a') → Detected if configured

- **Implementation**:
  ```rust
  pub fn sanitize_url(url: &str, whitelist: &[&str]) -> Option<String> {
      if url.is_empty() {
          return None;
      }

      let trimmed = url.trim().to_lowercase();

      // Check for dangerous schemes
      for scheme in DANGEROUS_SCHEMES {
          if trimmed.starts_with(&format!("{}:", scheme)) {
              return None;
          }
      }

      // Check whitelist
      for safe_scheme in whitelist {
          if url.starts_with(safe_scheme) || url.starts_with(&format!("{}:", safe_scheme)) {
              return Some(url.to_string());
          }
      }

      // Relative URLs
      if url.starts_with('/') || url.starts_with('./') || url.starts_with('#') {
          return Some(url.to_string());
      }

      None
  }
  ```

### 5. Attribute Sanitization

- **Attribute Whitelist**:
  - **Global**: id, class, title, lang, dir, data-*
  - **Links**: href, target (with rel="noopener noreferrer"), type
  - **Images**: src, alt, width, height, loading
  - **Tables**: colspan, rowspan, align, border
  - **Forms**: name, value, type, required, disabled, placeholder
  - **Forbidden**: onclick, onload, onerror, onmouseover, etc.

- **Attribute Value Validation**:
  - **URLs** (href, src): Validate with sanitize_url()
  - **Classes**: Whitelist safe class names or strip
  - **IDs**: Alphanumeric + dash/underscore or strip
  - **Data Attributes**: Allow key-value pairs (no scripts)
  - **Style**: Inline CSS sanitization (see Style Handling)

- **Dangerous Attributes Removal**:
  ```rust
  const DANGEROUS_ATTRIBUTES: &[&str] = &[
      "on*",           // All event handlers
      "style",         // If style stripping enabled
      "action",        // Form action URLs
      "formaction",    // Button form action
      "srcset",        // Responsive image URLs
      "srcdoc",        // Inline HTML in iframe
      "data",          // Embedded data
      "codebase",      // Java applet codebase
      "usemap",        // Client-side image map
  ];
  ```

### 6. SVG & Vector Graphic Handling

- **SVG Security Risks**:
  - `<script>` tags within SVG
  - Event handlers (onload, onclick, etc.)
  - External references (`xlink:href` to malicious URLs)
  - Animation with malicious behavior
  - Embedded JavaScript in animation timing

- **SVG Sanitization**:
  - Remove all `<script>` and `<style>` within SVG
  - Remove event handler attributes
  - Validate xlink:href and other URL attributes
  - Preserve safe presentational attributes
  - Extract text content as fallback for Markdown

- **SVG Conversion Strategy**:
  - **Text Extraction**: Use SVG text content if available
  - **Alt Text**: Fall back to aria-label or title attribute
  - **Alt Notation**: Use `[SVG: description]` notation
  - **Removal Option**: Strip SVG entirely if configured for strict safety

### 7. Runtime Safety Constraints

- **Stack Overflow Prevention**:
  - Enforce maximum nesting depth (default 256 levels)
  - Track nesting as tree is traversed
  - Return error before stack exhaust
  - Prevent pathological HTML like: `<div><div>...</div></div>` (1000 levels)

- **Memory Bounds**:
  - Enforce maximum document size (default 50MB)
  - Reject documents exceeding limit with clear error
  - Enforce maximum output size (configurable, default 100MB)

- **Timeout Protection**:
  - Optionally implement timeout for parsing/conversion
  - Return partial result if timeout exceeded
  - Prevent ReDoS (Regular Expression Denial of Service) in sanitization

- **Error Handling**:
  - Graceful degradation: Unsafe content removed, safe content preserved
  - Clear error messages: Indicate what was removed and why
  - Logging: Track sanitization events for security audit

### 8. Metadata & Structured Data Validation

- **Structured Data (JSON-LD, Schema.org)**:
  - **Risk**: JSON-LD can contain XSS payloads
  - **Strategy**: Remove structured data entirely by default
  - **Optional**: Validate and whitelist specific fields if needed

- **Meta Tags**:
  - Remove meta tags entirely (potential for CSRF, referrer leakage)
  - Preserve only safe OG/Twitter card metadata if configured

- **Comment Handling**:
  - Strip HTML comments by default (may contain sensitive info)
  - Optional: Preserve comments if configured

## Core Components

### Input Validator (`lib.rs`)

```rust
pub fn validate_input(html: &str) -> Result<()> {
    let bytes = html.as_bytes();

    // Check for binary magic numbers
    if detect_binary_magic(bytes).is_some() {
        return Err(ConversionError::InvalidInput("binary data detected".into()));
    }

    // Check for high control character ratio
    if detect_binary_heuristic(bytes) {
        return Err(ConversionError::InvalidInput("binary data detected".into()));
    }

    // Check size limits
    if bytes.len() > MAX_DOCUMENT_SIZE {
        return Err(ConversionError::InvalidInput(format!(
            "document size {} exceeds limit {}",
            bytes.len(),
            MAX_DOCUMENT_SIZE
        )));
    }

    Ok(())
}

fn detect_binary_magic(bytes: &[u8]) -> Option<&'static str> {
    const PREFIXES: &[(&[u8], &str)] = &[
        (b"\x1F\x8B", "gzip"),
        (b"\x28\xB5\x2F\xFD", "zstd"),
        (b"PK\x03\x04", "zip"),
        (b"%PDF-", "pdf"),
    ];

    for (prefix, label) in PREFIXES {
        if bytes.starts_with(prefix) {
            return Some(label);
        }
    }

    None
}

fn detect_binary_heuristic(bytes: &[u8]) -> bool {
    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    let sample = &bytes[..sample_len];

    let mut control_count = 0;
    let mut nul_count = 0;

    for &byte in sample {
        if byte == 0 {
            nul_count += 1;
        }
        let is_control = (byte < 0x09) || (0x0E..0x20).contains(&byte);
        if is_control {
            control_count += 1;
        }
    }

    let control_ratio = control_count as f64 / sample_len as f64;
    let nul_ratio = nul_count as f64 / sample_len as f64;

    control_ratio > BINARY_CONTROL_RATIO || nul_ratio > BINARY_UTF16_NULL_RATIO
}
```

### HTML Sanitizer (Ammonia Integration)

```rust
pub fn sanitize_html(html: &str, config: &SafetyConfig) -> Result<String> {
    if !config.sanitize_html {
        return Ok(html.to_string());
    }

    let mut cleaner = ammonia::Builder::default();
    cleaner.tags(config.allowed_tags.clone());
    cleaner.generic_attributes(config.allowed_attributes.clone());
    cleaner.link_rel(Some("noopener noreferrer".to_string()));
    cleaner.url_relative_mode(ammonia::UrlRelativeMode::PassThrough);

    if let Some(schemes) = &config.allowed_url_schemes {
        cleaner.url_schemes(schemes.clone());
    }

    Ok(cleaner.clean(html).to_string())
}
```

### URL Sanitizer

```rust
pub fn sanitize_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }

    let trimmed = url.trim();
    let lowercased = trimmed.to_lowercase();

    // Reject javascript: and similar
    if is_dangerous_scheme(&lowercased) {
        return None;
    }

    // Allow relative URLs
    if trimmed.starts_with('/') || trimmed.starts_with('#') || trimmed.starts_with("./") {
        return Some(trimmed.to_string());
    }

    // Allow whitelisted schemes
    for scheme in ALLOWED_SCHEMES {
        if lowercased.starts_with(&format!("{}:", scheme)) {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn is_dangerous_scheme(url: &str) -> bool {
    const DANGEROUS: &[&str] = &["javascript:", "data:", "vbscript:", "file:"];
    dangerous.iter().any(|scheme| url.starts_with(scheme))
}
```

### Safety Configuration

```rust
pub struct SafetyConfig {
    pub sanitize_html: bool,
    pub max_document_size: usize,
    pub max_nesting_depth: usize,
    pub allowed_tags: Vec<&'static str>,
    pub allowed_attributes: Vec<&'static str>,
    pub allowed_url_schemes: Option<Vec<&'static str>>,
    pub strip_svg: bool,
    pub strip_comments: bool,
    pub strict_mode: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            sanitize_html: true,
            max_document_size: 50 * 1024 * 1024,  // 50MB
            max_nesting_depth: 256,
            allowed_tags: vec!["p", "div", "h1", "h2", "h3", "h4", "h5", "h6",
                              "ul", "ol", "li", "table", "tr", "td", "th",
                              "a", "img", "strong", "em", "code", "pre"],
            allowed_attributes: vec!["id", "class", "title", "alt", "href", "src"],
            allowed_url_schemes: Some(vec!["http", "https", "mailto", "ftp"]),
            strip_svg: false,
            strip_comments: true,
            strict_mode: false,
        }
    }
}
```

## Integration with html-to-markdown Conversion

### Safety Pipeline

```
Untrusted HTML Input
    ↓
validate_input() [Binary detection, encoding check]
    ↓
sanitize_html() [Ammonia: remove dangerous elements/attrs]
    ↓
Validated HTML
    ↓
parse_html() [Safe to parse, no exploit risk]
    ↓
walk_tree() [Traverse sanitized DOM]
    ↓
visit_element() [Apply converters]
    ├─ sanitize_url(href, src) [Safe URLs only]
    ├─ escape_markdown(text) [No injection]
    └─ validate_attributes() [Whitelist attrs]
    ↓
Safe Markdown Output
```

### Configuration Integration

```rust
pub struct ConversionOptions {
    pub safety: SafetyConfig,
    pub sanitize_urls: bool,
    pub escape_html_entities: bool,
    pub strip_dangerous_elements: bool,
}
```

## Dependencies & Relationships

### Upstream Dependencies

- **ammonia**: Production-grade HTML sanitization
- **url**: URL parsing and validation
- **regex**: Pattern matching for XSS detection
- **encoding_rs**: Character encoding detection

### Downstream Dependencies

- **HTML Parsing Domain**: Operates on validated, sanitized HTML
- **Conversion Algorithms**: Only processes whitelisted elements/attributes
- **Output Formatting**: Markdown escaping prevents output-level XSS

## Security Characteristics

### Attack Mitigation

- **Stored XSS**: Ammonia sanitization removes executable code
- **Reflected XSS**: Markdown output inherently safe (no script execution)
- **DOM XSS**: Not applicable (static conversion, no JavaScript)
- **CSS Injection**: Style attributes sanitized or removed
- **URL-based XSS**: URL sanitization with scheme whitelist
- **SVG XSS**: SVG scripts and handlers removed or element stripped

### Threat Model

**Attacker Goal**: Execute JavaScript in user's browser via converted Markdown

**Attack Vector**: Malicious HTML input containing:
- `<script>` tags
- Event handler attributes (onclick, onerror, etc.)
- javascript: URLs in links/image sources
- Data URIs with encoded scripts
- SVG with embedded scripts

**Defense**:
1. Ammonia sanitization removes dangerous elements/attributes
2. URL sanitization blocks javascript: and data: schemes
3. Markdown output cannot execute scripts
4. Size/nesting limits prevent DoS

### Assumptions

- Converted Markdown is displayed in a safe context (not reinterpreted as HTML)
- Markdown renderer does not enable JavaScript or unsafe HTML
- Input size/complexity is bounded by configuration

## Testing & Validation

### Security Testing

- **XSS Payloads**: Test OWASP XSS Filter Evasion Cheat Sheet patterns
- **URL Injection**: Test javascript:, data:, vbscript: scheme handling
- **Encoding Bypasses**: Test URL decoding, Unicode tricks
- **Binary Detection**: Test gzip, PDF, ZIP, UTF-16 detection
- **Size Limits**: Test enforcement of max document size/nesting depth

### Sanitization Verification

- **Element Removal**: Verify dangerous elements stripped
- **Attribute Removal**: Verify event handlers removed
- **Content Preservation**: Verify text content retained
- **URL Validation**: Verify safe URLs preserved, dangerous URLs blocked
- **Roundtrip**: Verify sanitized HTML → Markdown conversion works

### Regression Tests

- **Safe Content**: Legitimate HTML not affected by sanitization
- **Performance**: Sanitization doesn't significantly slow conversion
- **Output Quality**: Converted Markdown remains readable and accurate

## Future Enhancements

- Machine learning-based XSS detection
- CSP (Content Security Policy) header generation
- Structured data validation (JSON-LD whitelisting)
- Content Security Policy integration
- Security audit logging and reporting
- Performance optimization of ammonia sanitization
- Custom content filters via plugin system
- Configurable threat levels (strict, moderate, permissive)
