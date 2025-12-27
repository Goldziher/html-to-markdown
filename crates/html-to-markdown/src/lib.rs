//! High-performance HTML to Markdown converter.
//!
//! Built with html5ever for fast, memory-efficient HTML parsing.
//!
//! ## Optional inline image extraction
//!
//! Enable the `inline-images` Cargo feature to collect embedded data URI images and inline SVG
//! assets alongside the produced Markdown.

use std::borrow::Cow;

pub mod converter;
pub mod error;
pub mod hocr;
#[cfg(feature = "inline-images")]
mod inline_images;
#[cfg(feature = "metadata")]
pub mod metadata;
pub mod options;
pub mod safety;
pub mod text;
#[cfg(feature = "visitor")]
pub mod visitor;
#[cfg(feature = "visitor")]
pub mod visitor_helpers;
#[cfg(feature = "async-visitor")]
pub use visitor_helpers::AsyncVisitorHandle;
pub mod wrapper;

pub use error::{ConversionError, Result};
#[cfg(feature = "inline-images")]
pub use inline_images::{
    DEFAULT_INLINE_IMAGE_LIMIT, HtmlExtraction, InlineImage, InlineImageConfig, InlineImageConfigUpdate,
    InlineImageFormat, InlineImageSource, InlineImageWarning,
};
#[cfg(feature = "metadata")]
pub use metadata::{
    DEFAULT_MAX_STRUCTURED_DATA_SIZE, DocumentMetadata, ExtendedMetadata, HeaderMetadata, ImageMetadata, ImageType,
    LinkMetadata, LinkType, MetadataConfig, MetadataConfigUpdate, StructuredData, StructuredDataType, TextDirection,
};
pub use options::{
    CodeBlockStyle, ConversionOptions, ConversionOptionsUpdate, HeadingStyle, HighlightStyle, ListIndentType,
    NewlineStyle, PreprocessingOptions, PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
};

const BINARY_SCAN_LIMIT: usize = 8192;
const BINARY_CONTROL_RATIO: f64 = 0.3;
const BINARY_UTF16_NULL_RATIO: f64 = 0.2;

const BINARY_MAGIC_PREFIXES: &[(&[u8], &str)] = &[
    (b"\x1F\x8B", "gzip-compressed data"),
    (b"\x28\xB5\x2F\xFD", "zstd-compressed data"),
    (b"PK\x03\x04", "zip archive"),
    (b"PK\x05\x06", "zip archive"),
    (b"PK\x07\x08", "zip archive"),
    (b"%PDF-", "PDF data"),
];

fn validate_input(html: &str) -> Result<()> {
    let bytes = html.as_bytes();
    if bytes.is_empty() {
        return Ok(());
    }

    if let Some(label) = detect_binary_magic(bytes) {
        return Err(ConversionError::InvalidInput(format!(
            "binary data detected ({label}); decode/decompress to UTF-8 HTML first"
        )));
    }

    let sample_len = bytes.len().min(BINARY_SCAN_LIMIT);
    let mut control_count = 0usize;
    let mut nul_count = 0usize;
    let mut even_nul_count = 0usize;
    let mut odd_nul_count = 0usize;

    for (idx, &byte) in bytes[..sample_len].iter().enumerate() {
        if byte == 0 {
            nul_count += 1;
            if idx % 2 == 0 {
                even_nul_count += 1;
            } else {
                odd_nul_count += 1;
            }
        }
        let is_control = (byte < 0x09) || (0x0E..0x20).contains(&byte);
        if is_control {
            control_count += 1;
        }
    }

    if nul_count > 0 {
        if let Some(label) = detect_utf16_hint(bytes, sample_len, nul_count, even_nul_count, odd_nul_count) {
            return Err(ConversionError::InvalidInput(format!(
                "binary data detected ({label}); decode to UTF-8 HTML first"
            )));
        }
        return Err(ConversionError::InvalidInput("binary data detected".to_string()));
    }

    let control_ratio = control_count as f64 / sample_len as f64;
    if control_ratio > BINARY_CONTROL_RATIO {
        return Err(ConversionError::InvalidInput(
            "binary data detected (excess control bytes)".to_string(),
        ));
    }

    Ok(())
}

fn detect_binary_magic(bytes: &[u8]) -> Option<&'static str> {
    for (prefix, label) in BINARY_MAGIC_PREFIXES {
        if bytes.starts_with(prefix) {
            return Some(*label);
        }
    }
    None
}

fn detect_utf16_hint(
    bytes: &[u8],
    sample_len: usize,
    nul_count: usize,
    even_nul_count: usize,
    odd_nul_count: usize,
) -> Option<&'static str> {
    if bytes.len() >= 2 {
        if bytes.starts_with(b"\xFF\xFE") {
            return Some("UTF-16LE BOM");
        }
        if bytes.starts_with(b"\xFE\xFF") {
            return Some("UTF-16BE BOM");
        }
    }

    let nul_ratio = nul_count as f64 / sample_len as f64;
    if nul_ratio < BINARY_UTF16_NULL_RATIO {
        return None;
    }

    let dominant_ratio = (even_nul_count.max(odd_nul_count) as f64) / nul_count as f64;
    if dominant_ratio >= 0.9 {
        Some("UTF-16 data without BOM")
    } else {
        None
    }
}

fn normalize_line_endings(html: &str) -> Cow<'_, str> {
    if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    }
}

fn fast_text_only(html: &str, options: &ConversionOptions) -> Option<String> {
    if html.contains('<') {
        return None;
    }

    let mut decoded = text::decode_html_entities_cow(html);
    if options.strip_newlines && (decoded.contains('\n') || decoded.contains('\r')) {
        decoded = Cow::Owned(decoded.replace(&['\r', '\n'][..], " "));
    }
    let trimmed = decoded.trim_end_matches('\n');
    if trimmed.is_empty() {
        return Some(String::new());
    }

    let normalized = if options.whitespace_mode == WhitespaceMode::Normalized {
        text::normalize_whitespace_cow(trimmed)
    } else {
        Cow::Borrowed(trimmed)
    };

    let escaped =
        if options.escape_misc || options.escape_asterisks || options.escape_underscores || options.escape_ascii {
            text::escape(
                normalized.as_ref(),
                options.escape_misc,
                options.escape_asterisks,
                options.escape_underscores,
                options.escape_ascii,
            )
        } else {
            normalized.into_owned()
        };

    let mut output = String::with_capacity(escaped.len() + 1);
    output.push_str(&escaped);
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
    output.push('\n');
    Some(output)
}

#[cfg(any(feature = "serde", feature = "metadata"))]
fn parse_json<T: serde::de::DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json).map_err(|err| ConversionError::ConfigError(err.to_string()))
}

#[cfg(any(feature = "serde", feature = "metadata"))]
pub fn conversion_options_from_json(json: &str) -> Result<ConversionOptions> {
    let update: ConversionOptionsUpdate = parse_json(json)?;
    Ok(ConversionOptions::from(update))
}

#[cfg(any(feature = "serde", feature = "metadata"))]
pub fn conversion_options_update_from_json(json: &str) -> Result<ConversionOptionsUpdate> {
    parse_json(json)
}

#[cfg(all(feature = "inline-images", any(feature = "serde", feature = "metadata")))]
pub fn inline_image_config_from_json(json: &str) -> Result<InlineImageConfig> {
    let update: InlineImageConfigUpdate = parse_json(json)?;
    Ok(InlineImageConfig::from_update(update))
}

#[cfg(all(feature = "metadata", any(feature = "serde", feature = "metadata")))]
pub fn metadata_config_from_json(json: &str) -> Result<MetadataConfig> {
    let update: MetadataConfigUpdate = parse_json(json)?;
    Ok(MetadataConfig::from(update))
}

/// Convert HTML to Markdown.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (defaults to ConversionOptions::default())
///
/// # Example
///
/// ```
/// use html_to_markdown_rs::{convert, ConversionOptions};
///
/// let html = "<h1>Hello World</h1>";
/// let markdown = convert(html, None).unwrap();
/// assert!(markdown.contains("Hello World"));
/// ```
pub fn convert(html: &str, options: Option<ConversionOptions>) -> Result<String> {
    validate_input(html)?;
    let options = options.unwrap_or_default();

    let normalized_html = normalize_line_endings(html);

    if !options.wrap {
        if let Some(markdown) = fast_text_only(normalized_html.as_ref(), &options) {
            return Ok(markdown);
        }
    }

    let markdown = converter::convert_html(normalized_html.as_ref(), &options)?;

    if options.wrap {
        Ok(wrapper::wrap_markdown(&markdown, &options))
    } else {
        Ok(markdown)
    }
}

#[cfg(feature = "inline-images")]
/// Convert HTML to Markdown while collecting inline image assets (requires the `inline-images` feature).
///
/// Extracts inline image data URIs and inline `<svg>` elements alongside Markdown conversion.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (defaults to ConversionOptions::default())
/// * `image_cfg` - Configuration controlling inline image extraction
pub fn convert_with_inline_images(
    html: &str,
    options: Option<ConversionOptions>,
    image_cfg: InlineImageConfig,
) -> Result<HtmlExtraction> {
    use std::cell::RefCell;
    use std::rc::Rc;

    validate_input(html)?;
    let options = options.unwrap_or_default();

    let normalized_html = normalize_line_endings(html);

    let collector = Rc::new(RefCell::new(inline_images::InlineImageCollector::new(image_cfg)?));

    let markdown =
        converter::convert_html_with_inline_collector(normalized_html.as_ref(), &options, Rc::clone(&collector))?;

    let markdown = if options.wrap {
        wrapper::wrap_markdown(&markdown, &options)
    } else {
        markdown
    };

    let collector = Rc::try_unwrap(collector)
        .map_err(|_| ConversionError::Other("failed to recover inline image state".to_string()))?
        .into_inner();
    let (inline_images, warnings) = collector.finish();

    Ok(HtmlExtraction {
        markdown,
        inline_images,
        warnings,
    })
}

#[cfg(feature = "metadata")]
/// Convert HTML to Markdown with comprehensive metadata extraction (requires the `metadata` feature).
///
/// Performs HTML-to-Markdown conversion while simultaneously extracting structured metadata in a
/// single pass for maximum efficiency. Ideal for content analysis, SEO optimization, and document
/// indexing workflows.
///
/// # Arguments
///
/// * `html` - The HTML string to convert. Will normalize line endings (CRLF → LF).
/// * `options` - Optional conversion configuration. Defaults to `ConversionOptions::default()` if `None`.
///   Controls heading style, list indentation, escape behavior, wrapping, and other output formatting.
/// * `metadata_cfg` - Configuration for metadata extraction granularity. Use `MetadataConfig::default()`
///   to extract all metadata types, or customize with selective extraction flags.
///
/// # Returns
///
/// On success, returns a tuple of:
/// - `String`: The converted Markdown output
/// - `ExtendedMetadata`: Comprehensive metadata containing:
///   - `document`: Title, description, author, language, Open Graph, Twitter Card, and other meta tags
///   - `headers`: All heading elements (h1-h6) with hierarchy and IDs
///   - `links`: Hyperlinks classified as anchor, internal, external, email, or phone
///   - `images`: Image elements with source, dimensions, and alt text
///   - `structured_data`: JSON-LD, Microdata, and RDFa blocks
///
/// # Errors
///
/// Returns `ConversionError` if:
/// - HTML parsing fails
/// - Invalid UTF-8 sequences encountered
/// - Internal panic during conversion (wrapped in `ConversionError::Panic`)
/// - Configuration size limits exceeded
///
/// # Performance Notes
///
/// - Single-pass collection: metadata extraction has minimal overhead
/// - Zero cost when metadata feature is disabled
/// - Pre-allocated buffers: typically handles 50+ headers, 100+ links, 20+ images efficiently
/// - Structured data size-limited to prevent memory exhaustion (configurable)
///
/// # Example: Basic Usage
///
/// ```ignore
/// use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
///
/// let html = r#"
///   <html lang="en">
///     <head><title>My Article</title></head>
///     <body>
///       <h1 id="intro">Introduction</h1>
///       <p>Welcome to <a href="https://example.com">our site</a></p>
///     </body>
///   </html>
/// "#;
///
/// let (markdown, metadata) = convert_with_metadata(html, None, MetadataConfig::default())?;
///
/// assert_eq!(metadata.document.title, Some("My Article".to_string()));
/// assert_eq!(metadata.document.language, Some("en".to_string()));
/// assert_eq!(metadata.headers[0].text, "Introduction");
/// assert_eq!(metadata.headers[0].id, Some("intro".to_string()));
/// assert_eq!(metadata.links.len(), 1);
/// # Ok::<(), html_to_markdown_rs::ConversionError>(())
/// ```
///
/// # Example: Selective Metadata Extraction
///
/// ```ignore
/// use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
///
/// let html = "<html><body><h1>Title</h1><a href='#anchor'>Link</a></body></html>";
///
/// // Extract only headers and document metadata, skip links/images
/// let config = MetadataConfig {
///     extract_headers: true,
///     extract_links: false,
///     extract_images: false,
///     extract_structured_data: false,
///     max_structured_data_size: 0,
/// };
///
/// let (markdown, metadata) = convert_with_metadata(html, None, config)?;
/// assert!(metadata.headers.len() > 0);
/// assert!(metadata.links.is_empty());  // Not extracted
/// # Ok::<(), html_to_markdown_rs::ConversionError>(())
/// ```
///
/// # Example: With Conversion Options and Metadata Config
///
/// ```ignore
/// use html_to_markdown_rs::{convert_with_metadata, ConversionOptions, MetadataConfig, HeadingStyle};
///
/// let html = "<html><head><title>Blog Post</title></head><body><h1>Hello</h1></body></html>";
///
/// let options = ConversionOptions {
///     heading_style: HeadingStyle::Atx,
///     wrap: true,
///     wrap_width: 80,
///     ..Default::default()
/// };
///
/// let metadata_cfg = MetadataConfig::default();
///
/// let (markdown, metadata) = convert_with_metadata(html, Some(options), metadata_cfg)?;
/// // Markdown will use ATX-style headings (# H1, ## H2, etc.)
/// // Wrapped at 80 characters
/// // All metadata extracted
/// # Ok::<(), html_to_markdown_rs::ConversionError>(())
/// ```
///
/// # See Also
///
/// - [`convert`] - Simple HTML to Markdown conversion without metadata
/// - [`convert_with_inline_images`] - Conversion with inline image extraction
/// - [`MetadataConfig`] - Configuration for metadata extraction
/// - [`ExtendedMetadata`] - Metadata structure documentation
/// - [`metadata`] module - Detailed type documentation for metadata components
pub fn convert_with_metadata(
    html: &str,
    options: Option<ConversionOptions>,
    metadata_cfg: MetadataConfig,
) -> Result<(String, ExtendedMetadata)> {
    use std::cell::RefCell;
    use std::rc::Rc;

    validate_input(html)?;
    let options = options.unwrap_or_default();
    if !metadata_cfg.any_enabled() {
        let markdown = convert(html, Some(options))?;
        return Ok((markdown, ExtendedMetadata::default()));
    }

    let normalized_html = normalize_line_endings(html);

    let metadata_collector = Rc::new(RefCell::new(metadata::MetadataCollector::new(metadata_cfg)));

    let markdown =
        converter::convert_html_with_metadata(normalized_html.as_ref(), &options, Rc::clone(&metadata_collector))?;

    let markdown = if options.wrap {
        wrapper::wrap_markdown(&markdown, &options)
    } else {
        markdown
    };

    let metadata_collector = Rc::try_unwrap(metadata_collector)
        .map_err(|_| ConversionError::Other("failed to recover metadata state".to_string()))?
        .into_inner();
    let metadata = metadata_collector.finish();

    Ok((markdown, metadata))
}

/// Convert HTML to Markdown with a custom visitor callback.
///
/// This function allows you to provide a visitor implementation that can inspect,
/// modify, or replace the default conversion behavior for any HTML element type.
///
/// # Arguments
///
/// * `html` - The HTML input to convert
/// * `options` - Optional conversion options (uses defaults if None)
/// * `visitor` - Mutable reference to visitor implementation for customization
///
/// # Example
///
/// ```ignore
/// use html_to_markdown_rs::convert_with_visitor;
/// use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
///
/// #[derive(Debug)]
/// struct CustomVisitor;
///
/// impl HtmlVisitor for CustomVisitor {
///     fn visit_code_block(
///         &mut self,
///         _ctx: &NodeContext,
///         code: &str,
///         language: Option<&str>,
///     ) -> VisitResult {
///         VisitResult::Custom(format!("```{}\n{}\n```", language.unwrap_or(""), code))
///     }
/// }
///
/// let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
/// let mut visitor = CustomVisitor;
/// let markdown = convert_with_visitor(html, None, &mut visitor).unwrap();
/// ```
#[cfg(feature = "visitor")]
pub fn convert_with_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<visitor::VisitorHandle>,
) -> Result<String> {
    validate_input(html)?;
    let options = options.unwrap_or_default();

    let normalized_html = normalize_line_endings(html);

    let markdown = converter::convert_html_with_visitor(normalized_html.as_ref(), &options, visitor)?;

    if options.wrap {
        Ok(wrapper::wrap_markdown(&markdown, &options))
    } else {
        Ok(markdown)
    }
}

#[cfg(feature = "async-visitor")]
/// Convert HTML to Markdown with an async visitor callback.
///
/// This async function allows you to provide an async visitor implementation that can inspect,
/// modify, or replace the default conversion behavior for any HTML element type.
///
/// This function is useful for:
/// - Python async functions (with `async def` and `asyncio`)
/// - TypeScript/JavaScript async functions (with `Promise`-based callbacks)
/// - Elixir processes (with message-passing async operations)
///
/// For synchronous languages (Ruby, PHP, Go, Java, C#), use `convert_with_visitor` instead.
///
/// # Note
///
/// The async visitor trait (`AsyncHtmlVisitor`) and async dispatch helpers are designed to be
/// consumed by language bindings (PyO3, NAPI-RS, Magnus, etc.) which can bridge async/await
/// semantics from their host languages. The conversion pipeline itself runs synchronously,
/// but visitor callbacks are defined as async to support languages with native async/await.
///
/// Binding implementations will be responsible for running async callbacks on appropriate
/// event loops (asyncio for Python, Promise chains for TypeScript, etc.).
///
/// # Arguments
///
/// * `html` - The HTML input to convert
/// * `options` - Optional conversion options (uses defaults if None)
/// * `visitor` - Optional async visitor implementing `AsyncHtmlVisitor` trait for customization
///
/// # Example (Rust-like async)
///
/// ```ignore
/// use html_to_markdown_rs::convert_with_async_visitor;
/// use html_to_markdown_rs::visitor::{AsyncHtmlVisitor, NodeContext, VisitResult};
/// use async_trait::async_trait;
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// #[derive(Debug)]
/// struct CustomAsyncVisitor;
///
/// #[async_trait]
/// impl AsyncHtmlVisitor for CustomAsyncVisitor {
///     async fn visit_code_block(
///         &mut self,
///         _ctx: &NodeContext,
///         code: &str,
///         language: Option<&str>,
///     ) -> VisitResult {
///         // Can perform async operations here (e.g., syntax highlighting via service)
///         VisitResult::Custom(format!("```{}\n{}\n```", language.unwrap_or(""), code))
///     }
/// }
///
/// let html = "<pre><code class=\"language-rust\">fn main() {}</code></pre>";
/// let visitor = Some(Rc::new(RefCell::new(CustomAsyncVisitor) as _));
/// let markdown = convert_with_async_visitor(html, None, visitor).await.unwrap();
/// ```
///
/// # Implementation Note
///
/// Currently, this function placeholder documents the async visitor interface for binding
/// implementations. The actual async dispatch integration will be implemented in binding
/// layers (Python, TypeScript, etc.) to properly bridge async/await semantics to the
/// synchronous Rust conversion core.
pub async fn convert_with_async_visitor(
    html: &str,
    options: Option<ConversionOptions>,
    _visitor: Option<visitor_helpers::AsyncVisitorHandle>,
) -> Result<String> {
    validate_input(html)?;
    let options = options.unwrap_or_default();

    let normalized_html = normalize_line_endings(html);

    // TODO: Implement async dispatch in conversion pipeline
    let markdown = converter::convert_html(normalized_html.as_ref(), &options)?;

    if options.wrap {
        Ok(wrapper::wrap_markdown(&markdown, &options))
    } else {
        Ok(markdown)
    }
}

#[cfg(all(test, feature = "metadata"))]
mod tests {
    use super::*;

    #[test]
    fn test_convert_with_metadata_full_workflow() {
        let html = "<html lang=\"en\" dir=\"ltr\"><head><title>Test Article</title></head><body><h1 id=\"main-title\">Main Title</h1><p>This is a paragraph with a <a href=\"https://example.com\">link</a>.</p><h2>Subsection</h2><p>Another paragraph with <a href=\"#main-title\">internal link</a>.</p><img src=\"https://example.com/image.jpg\" alt=\"Test image\" title=\"Image title\"></body></html>";

        let config = MetadataConfig {
            extract_document: true,
            extract_headers: true,
            extract_links: true,
            extract_images: true,
            extract_structured_data: true,
            max_structured_data_size: metadata::DEFAULT_MAX_STRUCTURED_DATA_SIZE,
        };

        let (markdown, metadata) = convert_with_metadata(html, None, config).expect("conversion should succeed");

        assert!(!markdown.is_empty());
        assert!(markdown.contains("Main Title"));
        assert!(markdown.contains("Subsection"));

        assert_eq!(metadata.document.language, Some("en".to_string()));

        assert_eq!(metadata.headers.len(), 2);
        assert_eq!(metadata.headers[0].level, 1);
        assert_eq!(metadata.headers[0].text, "Main Title");
        assert_eq!(metadata.headers[0].id, Some("main-title".to_string()));
        assert_eq!(metadata.headers[1].level, 2);
        assert_eq!(metadata.headers[1].text, "Subsection");

        assert!(metadata.links.len() >= 2);
        let external_link = metadata.links.iter().find(|l| l.link_type == LinkType::External);
        assert!(external_link.is_some());
        let anchor_link = metadata.links.iter().find(|l| l.link_type == LinkType::Anchor);
        assert!(anchor_link.is_some());

        assert_eq!(metadata.images.len(), 1);
        assert_eq!(metadata.images[0].alt, Some("Test image".to_string()));
        assert_eq!(metadata.images[0].title, Some("Image title".to_string()));
        assert_eq!(metadata.images[0].image_type, ImageType::External);
    }

    #[test]
    fn test_convert_with_metadata_document_fields() {
        let html = "<html lang=\"en\"><head><title>Test Article</title><meta name=\"description\" content=\"Desc\"><meta name=\"author\" content=\"Author\"><meta property=\"og:title\" content=\"OG Title\"><meta property=\"og:description\" content=\"OG Desc\"></head><body><h1>Heading</h1></body></html>";

        let (_markdown, metadata) =
            convert_with_metadata(html, None, MetadataConfig::default()).expect("conversion should succeed");

        assert_eq!(
            metadata.document.title,
            Some("Test Article".to_string()),
            "document: {:?}",
            metadata.document
        );
        assert_eq!(metadata.document.description, Some("Desc".to_string()));
        assert_eq!(metadata.document.author, Some("Author".to_string()));
        assert_eq!(metadata.document.language, Some("en".to_string()));
        assert_eq!(metadata.document.open_graph.get("title"), Some(&"OG Title".to_string()));
        assert_eq!(
            metadata.document.open_graph.get("description"),
            Some(&"OG Desc".to_string())
        );
    }

    #[test]
    fn test_convert_with_metadata_empty_config() {
        let html = "<html lang=\"en\"><head><title>Test</title></head><body><h1>Title</h1><a href=\"#\">Link</a></body></html>";

        let config = MetadataConfig {
            extract_document: false,
            extract_headers: false,
            extract_links: false,
            extract_images: false,
            extract_structured_data: false,
            max_structured_data_size: 0,
        };

        let (_markdown, metadata) = convert_with_metadata(html, None, config).expect("conversion should succeed");

        assert!(metadata.headers.is_empty());
        assert!(metadata.links.is_empty());
        assert!(metadata.images.is_empty());
        assert_eq!(metadata.document.language, None);
    }

    #[test]
    fn test_convert_with_metadata_data_uri_image() {
        let html = "<html><body><img src=\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==\" alt=\"Pixel\"></body></html>";

        let config = MetadataConfig::default();

        let (_markdown, metadata) = convert_with_metadata(html, None, config).expect("conversion should succeed");

        assert_eq!(metadata.images.len(), 1);
        assert_eq!(metadata.images[0].image_type, ImageType::DataUri);
        assert_eq!(metadata.images[0].alt, Some("Pixel".to_string()));
    }

    #[test]
    fn test_convert_with_metadata_relative_paths() {
        let html = r#"<html><body><a href="/page">Internal</a><a href="../other">Relative</a></body></html>"#;

        let config = MetadataConfig::default();

        let (_markdown, metadata) = convert_with_metadata(html, None, config).expect("conversion should succeed");

        let internal_links: Vec<_> = metadata
            .links
            .iter()
            .filter(|l| l.link_type == LinkType::Internal)
            .collect();
        assert_eq!(internal_links.len(), 2);
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

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

    #[test]
    fn test_plain_text_escaped_when_enabled() {
        let options = ConversionOptions {
            escape_asterisks: true,
            escape_underscores: true,
            ..ConversionOptions::default()
        };
        let result = convert("Text *asterisks* _underscores_", Some(options)).unwrap();
        assert!(result.contains(r"\*asterisks\*"));
        assert!(result.contains(r"\_underscores\_"));
    }
}
