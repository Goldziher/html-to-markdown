# Metadata Extraction for html-to-markdown

## Overview

The html-to-markdown library provides comprehensive, single-pass metadata extraction during HTML-to-Markdown conversion. This enables content analysis, SEO optimization, document indexing, and structured data processing without extra parsing passes.

## Architecture: MetadataCollector Pattern

### Single-Pass Collection

Metadata extraction uses the same `MetadataCollector` pattern as inline image collection:

```rust
// From lib.rs line 445
let metadata_collector = Rc::new(RefCell::new(metadata::MetadataCollector::new(metadata_cfg)));

// Passed to converter during tree walk
let markdown = converter::convert_html_with_metadata(
    normalized_html.as_ref(),
    &options,
    Rc::clone(&metadata_collector)
)?;

// After conversion, recover and return metadata
let metadata = metadata_collector.finish();
```

**Key Benefits:**
- **Zero overhead when disabled**: Entire module compilable out via feature flags
- **Single tree traversal**: No separate metadata extraction pass
- **Memory efficient**: Pre-allocated buffers (typical: 32 headers, 64 links, 16 images)
- **Configurable granularity**: Extract only needed metadata types

## Metadata Configuration

### MetadataConfig Structure

Located in `/crates/html-to-markdown/src/metadata.rs`:

```rust
pub struct MetadataConfig {
    pub extract_document: bool,        // <head> meta tags, title, etc.
    pub extract_headers: bool,         // h1-h6 with hierarchy
    pub extract_links: bool,           // All hyperlinks with classification
    pub extract_images: bool,          // All images with dimensions
    pub extract_structured_data: bool, // JSON-LD, Microdata, RDFa
    pub max_structured_data_size: usize, // Prevent memory exhaustion
}
```

### Default Behavior

```rust
impl Default for MetadataConfig {
    fn default() -> Self {
        MetadataConfig {
            extract_document: true,
            extract_headers: true,
            extract_links: true,
            extract_images: true,
            extract_structured_data: true,
            max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE, // typically 10MB
        }
    }
}
```

### Selective Extraction

Only extract specific metadata types:

```rust
let config = MetadataConfig {
    extract_document: true,
    extract_headers: true,
    extract_links: false,      // Skip links
    extract_images: false,     // Skip images
    extract_structured_data: false,
    max_structured_data_size: 0,
};

let (markdown, metadata) = convert_with_metadata(html, None, config)?;
// metadata.links will be empty (not extracted)
```

## Document Metadata Extraction

### Extracted Fields

**Document metadata** collects head-level information:

```rust
pub struct DocumentMetadata {
    pub title: Option<String>,           // <title> element
    pub description: Option<String>,     // meta[name="description"]
    pub author: Option<String>,          // meta[name="author"]
    pub language: Option<String>,        // html[lang] or meta[http-equiv]
    pub charset: Option<String>,         // meta[charset] or meta[http-equiv]
    pub canonical_url: Option<String>,   // <link rel="canonical">
    pub viewport: Option<String>,        // meta[name="viewport"]
    pub text_direction: TextDirection,   // html[dir] or meta property
    pub open_graph: BTreeMap<String, String>, // og:* properties
    pub twitter_card: BTreeMap<String, String>, // twitter:* properties
    pub other_meta: BTreeMap<String, String>, // Remaining meta tags
}
```

### Text Direction

```rust
pub enum TextDirection {
    Ltr,     // <html dir="ltr">
    Rtl,     // <html dir="rtl">
    Auto,    // Default or <html dir="auto">
}
```

### HTML Example

```html
<html lang="en" dir="ltr">
<head>
    <title>My Article</title>
    <meta name="description" content="Article about HTML conversion">
    <meta name="author" content="John Doe">
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="canonical" href="https://example.com/article">

    <!-- Open Graph -->
    <meta property="og:title" content="Article Title">
    <meta property="og:description" content="Description for sharing">
    <meta property="og:image" content="https://example.com/image.jpg">
    <meta property="og:type" content="article">
    <meta property="og:url" content="https://example.com/article">

    <!-- Twitter Card -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="Article Title">
    <meta name="twitter:description" content="Description">
    <meta name="twitter:image" content="https://example.com/image.jpg">
</head>
</html>
```

**Extracted Result:**
```rust
DocumentMetadata {
    title: Some("My Article"),
    description: Some("Article about HTML conversion"),
    author: Some("John Doe"),
    language: Some("en"),
    charset: Some("utf-8"),
    canonical_url: Some("https://example.com/article"),
    viewport: Some("width=device-width, initial-scale=1"),
    text_direction: TextDirection::Ltr,
    open_graph: {
        "title" => "Article Title",
        "description" => "Description for sharing",
        "image" => "https://example.com/image.jpg",
        "type" => "article",
        "url" => "https://example.com/article",
    },
    twitter_card: {
        "card" => "summary_large_image",
        "title" => "Article Title",
        "description" => "Description",
        "image" => "https://example.com/image.jpg",
    },
    other_meta: { /* remaining meta tags */ },
}
```

## Headers Extraction

### HeaderMetadata Structure

```rust
pub struct HeaderMetadata {
    pub level: u8,              // 1-6 (h1-h6)
    pub text: String,           // Extracted text content
    pub id: Option<String>,     // id attribute
    pub hierarchy_depth: u8,    // Nesting depth (0 = top-level)
    pub position: usize,        // Document order position
}
```

### Hierarchy and Nesting

Headers are extracted with context about document structure:

```html
<h1>Main Title</h1>          <!-- hierarchy_depth: 0 -->
<p>Introduction</p>
<h2 id="section-1">Section 1</h2>  <!-- hierarchy_depth: 1 -->
<p>Content</p>
<h3>Subsection 1.1</h3>      <!-- hierarchy_depth: 2 -->
<h3>Subsection 1.2</h3>      <!-- hierarchy_depth: 2 -->
<h2>Section 2</h2>           <!-- hierarchy_depth: 1 -->
```

**Extracted Headers:**
```rust
vec![
    HeaderMetadata { level: 1, text: "Main Title", id: None, hierarchy_depth: 0, position: 0 },
    HeaderMetadata { level: 2, text: "Section 1", id: Some("section-1"), hierarchy_depth: 1, position: 2 },
    HeaderMetadata { level: 3, text: "Subsection 1.1", id: None, hierarchy_depth: 2, position: 4 },
    HeaderMetadata { level: 3, text: "Subsection 1.2", id: None, hierarchy_depth: 2, position: 5 },
    HeaderMetadata { level: 2, text: "Section 2", id: None, hierarchy_depth: 1, position: 6 },
]
```

### Use Cases

- **Table of contents generation**: Build TOC from headers and IDs
- **Document outline**: Create hierarchical structure
- **SEO analysis**: Verify H1 presence, hierarchy correctness
- **Navigation**: Generate internal anchor links

## Links Extraction

### LinkType Classification

```rust
pub enum LinkType {
    Anchor,      // #section-id (internal fragment)
    Internal,    // /page, ../other, /path/to/page
    External,    // https://example.com
    Email,       // mailto:user@example.com
    Phone,       // tel:+1234567890
    Other,       // Unknown schemes (ftp, data, etc.)
}
```

### LinkMetadata Structure

```rust
pub struct LinkMetadata {
    pub href: String,                      // Full href attribute
    pub text: String,                      // Link display text
    pub title: Option<String>,             // title attribute
    pub link_type: LinkType,               // Classification
    pub rel_attributes: Vec<String>,       // rel attribute values
    pub custom_attributes: BTreeMap<String, String>, // data-*, aria-*, etc.
    pub is_external: bool,                 // Convenience flag
}
```

### Classification Logic

```
href="#intro"                 → LinkType::Anchor
href="/page"                  → LinkType::Internal
href="../sibling"             → LinkType::Internal
href="relative/path"          → LinkType::Internal
href="https://example.com"    → LinkType::External
href="http://example.com"     → LinkType::External
href="mailto:user@example.com" → LinkType::Email
href="tel:+1234567890"        → LinkType::Phone
href="ftp://server.com"       → LinkType::Other
href="javascript:void(0)"     → LinkType::Other
```

### HTML Example

```html
<body>
    <!-- Anchor link -->
    <a href="#main-section" title="Jump to main">Main</a>

    <!-- Internal links -->
    <a href="/about">About Us</a>
    <a href="../other-page" rel="internal">Other</a>

    <!-- External links -->
    <a href="https://google.com" rel="external nofollow">Google</a>

    <!-- Email -->
    <a href="mailto:support@example.com">Contact</a>

    <!-- Phone -->
    <a href="tel:+1-555-1234">Call us</a>

    <!-- Data attributes -->
    <a href="/product" data-id="123" data-category="electronics">Product</a>
</body>
```

**Extracted Links:**
```rust
vec![
    LinkMetadata {
        href: "#main-section",
        text: "Main",
        title: Some("Jump to main"),
        link_type: LinkType::Anchor,
        rel_attributes: vec![],
        custom_attributes: {},
        is_external: false,
    },
    LinkMetadata {
        href: "/about",
        text: "About Us",
        title: None,
        link_type: LinkType::Internal,
        rel_attributes: vec![],
        custom_attributes: {},
        is_external: false,
    },
    LinkMetadata {
        href: "https://google.com",
        text: "Google",
        title: None,
        link_type: LinkType::External,
        rel_attributes: vec!["external", "nofollow"],
        custom_attributes: {},
        is_external: true,
    },
    LinkMetadata {
        href: "mailto:support@example.com",
        text: "Contact",
        title: None,
        link_type: LinkType::Email,
        rel_attributes: vec![],
        custom_attributes: {},
        is_external: false,
    },
    LinkMetadata {
        href: "tel:+1-555-1234",
        text: "Call us",
        title: None,
        link_type: LinkType::Phone,
        rel_attributes: vec![],
        custom_attributes: {},
        is_external: false,
    },
    LinkMetadata {
        href: "/product",
        text: "Product",
        title: None,
        link_type: LinkType::Internal,
        rel_attributes: vec![],
        custom_attributes: {
            "data-id" => "123",
            "data-category" => "electronics",
        },
        is_external: false,
    },
]
```

### Use Cases

- **Link auditing**: Find broken links, external vs internal
- **SEO analysis**: Check nofollow, link density
- **Content organization**: Identify internal link structure
- **Contact extraction**: Find email and phone links
- **Metadata preservation**: Retain data attributes for custom processing

## Images Extraction

### ImageType Classification

```rust
pub enum ImageType {
    DataUri,     // data:image/png;base64,...
    External,    // https://example.com/image.jpg
    Relative,    // /images/photo.jpg or ../img/pic.png
    InlineSvg,   // <svg>...</svg> embedded (if inline-images feature)
}
```

### ImageMetadata Structure

```rust
pub struct ImageMetadata {
    pub src: String,                       // Image source URL or data URI
    pub alt: Option<String>,               // alt attribute text
    pub title: Option<String>,             // title attribute
    pub image_type: ImageType,             // Classification
    pub width: Option<u32>,                // width attribute (pixels)
    pub height: Option<u32>,               // height attribute (pixels)
    pub custom_attributes: BTreeMap<String, String>, // data-*, aria-*, etc.
}
```

### HTML Example

```html
<body>
    <!-- External image -->
    <img src="https://example.com/photo.jpg" alt="A photo" title="Photo title" width="800" height="600">

    <!-- Relative path -->
    <img src="/images/icon.png" alt="Icon">

    <!-- Data URI (base64-encoded PNG) -->
    <img src="data:image/png;base64,iVBORw0KGgoAAAANS..." alt="Embedded">

    <!-- With metadata -->
    <img src="product.jpg" alt="Product" data-id="456" data-category="gadgets">
</body>
```

**Extracted Images:**
```rust
vec![
    ImageMetadata {
        src: "https://example.com/photo.jpg",
        alt: Some("A photo"),
        title: Some("Photo title"),
        image_type: ImageType::External,
        width: Some(800),
        height: Some(600),
        custom_attributes: {},
    },
    ImageMetadata {
        src: "/images/icon.png",
        alt: Some("Icon"),
        title: None,
        image_type: ImageType::Relative,
        width: None,
        height: None,
        custom_attributes: {},
    },
    ImageMetadata {
        src: "data:image/png;base64,iVBORw0KGgoAAAANS...",
        alt: Some("Embedded"),
        title: None,
        image_type: ImageType::DataUri,
        width: None,
        height: None,
        custom_attributes: {},
    },
    ImageMetadata {
        src: "product.jpg",
        alt: Some("Product"),
        title: None,
        image_type: ImageType::Relative,
        width: None,
        height: None,
        custom_attributes: {
            "data-id" => "456",
            "data-category" => "gadgets",
        },
    },
]
```

### Use Cases

- **Image inventory**: Catalog all images by type and source
- **Alt text audit**: Find missing alt text for accessibility
- **Thumbnail extraction**: Collect small images for previews
- **Data URI detection**: Find embedded images for optimization
- **Dimension analysis**: Track image sizes for responsive design

## Structured Data Extraction

### StructuredDataType

```rust
pub enum StructuredDataType {
    JsonLd,      // <script type="application/ld+json">
    Microdata,   // itemscope, itemtype, itemprop attributes
    RDFa,        // vocab, property, typeof attributes
}
```

### StructuredData Structure

```rust
pub struct StructuredData {
    pub data_type: StructuredDataType,
    pub raw_data: String,  // Original JSON, HTML, or RDF markup
}
```

### JSON-LD Example

```html
<script type="application/ld+json">
{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "My Article",
    "author": {
        "@type": "Person",
        "name": "John Doe"
    },
    "datePublished": "2025-12-29",
    "articleBody": "Content here..."
}
</script>
```

**Extracted:**
```rust
StructuredData {
    data_type: StructuredDataType::JsonLd,
    raw_data: r#"{"@context":"https://schema.org","@type":"Article",...}"#,
}
```

### Microdata Example

```html
<div itemscope itemtype="https://schema.org/Article">
    <h1 itemprop="headline">Article Title</h1>
    <p itemprop="articleBody">Content...</p>
    <span itemprop="author" itemscope itemtype="https://schema.org/Person">
        <span itemprop="name">John Doe</span>
    </span>
</div>
```

**Extracted:** Raw HTML fragment preserved with attributes

### RDFa Example

```html
<div vocab="https://schema.org/" typeof="Article">
    <h1 property="headline">Article Title</h1>
    <p property="articleBody">Content...</p>
    <span property="author" typeof="Person">
        <span property="name">John Doe</span>
    </span>
</div>
```

### Use Cases

- **Schema.org parsing**: Extract structured data for rich snippets
- **SEO metadata**: Validate schema.org compliance
- **Feed generation**: Use article schema for RSS/JSON feeds
- **Knowledge graphs**: Populate semantic web data

## Complete Example: convert_with_metadata

From `/crates/html-to-markdown/src/lib.rs` (lines 428-462):

```rust
pub fn convert_with_metadata(
    html: &str,
    options: Option<ConversionOptions>,
    metadata_cfg: MetadataConfig,
) -> Result<(String, ExtendedMetadata)> {
    // Validate input
    validate_input(html)?;
    let options = options.unwrap_or_default();

    // Early return if no extraction requested
    if !metadata_cfg.any_enabled() {
        let markdown = convert(html, Some(options))?;
        return Ok((markdown, ExtendedMetadata::default()));
    }

    // Normalize line endings
    let normalized_html = normalize_line_endings(html);

    // Create collector for single-pass gathering
    let metadata_collector = Rc::new(RefCell::new(
        metadata::MetadataCollector::new(metadata_cfg)
    ));

    // Convert with metadata collection
    let markdown = converter::convert_html_with_metadata(
        normalized_html.as_ref(),
        &options,
        Rc::clone(&metadata_collector)
    )?;

    // Apply wrapping if configured
    let markdown = if options.wrap {
        wrapper::wrap_markdown(&markdown, &options)
    } else {
        markdown
    };

    // Recover metadata from collector
    let metadata_collector = Rc::try_unwrap(metadata_collector)
        .map_err(|_| ConversionError::Other("failed to recover metadata state".to_string()))?
        .into_inner();
    let metadata = metadata_collector.finish();

    // Return both markdown and metadata
    Ok((markdown, metadata))
}
```

## Performance Characteristics

**Benchmarking:**
- Single-pass collection adds < 5% overhead to conversion
- Memory: Typical document (50 headers, 100 links, 20 images) < 50KB overhead
- Large documents (1000+ links): Pre-allocated buffers grow as needed

**Memory Safety:**
- `max_structured_data_size` prevents DoS from huge JSON-LD blocks
- Recursive metadata collection depth-limited
- No unbounded allocations

## Implementation Location

**Core Files:**
- `/crates/html-to-markdown/src/metadata.rs` - All metadata types and collector
- `/crates/html-to-markdown/src/lib.rs` - `convert_with_metadata()` public API (lines 310-462)
- `/crates/html-to-markdown/src/converter.rs` - Integration with conversion pipeline

**Testing:**
- `/crates/html-to-markdown/src/lib.rs` - Tests starting at line 604

## API Pattern Consistency

Metadata extraction follows the same pattern as inline images:

```rust
// Image extraction (inline-images feature)
pub fn convert_with_inline_images(
    html: &str,
    options: Option<ConversionOptions>,
    image_cfg: InlineImageConfig,
) -> Result<HtmlExtraction> { ... }

// Metadata extraction (metadata feature)
pub fn convert_with_metadata(
    html: &str,
    options: Option<ConversionOptions>,
    metadata_cfg: MetadataConfig,
) -> Result<(String, ExtendedMetadata)> { ... }

// Visitor pattern (visitor feature)
pub fn convert_with_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<visitor::VisitorHandle>,
) -> Result<String> { ... }
```

## Quick Integration Guide

1. Enable `metadata` feature in Cargo.toml
2. Import: `use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};`
3. Call: `let (md, meta) = convert_with_metadata(html, None, MetadataConfig::default())?;`
4. Access: `meta.document.title`, `meta.headers`, `meta.links`, `meta.images`
5. Optional: Filter metadata types via `MetadataConfig` to reduce overhead
