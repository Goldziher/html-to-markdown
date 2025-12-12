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
pub mod wrapper;

pub use error::{ConversionError, Result};
#[cfg(feature = "inline-images")]
pub use inline_images::{
    HtmlExtraction, InlineImage, InlineImageConfig, InlineImageFormat, InlineImageSource, InlineImageWarning,
};
#[cfg(feature = "metadata")]
pub use metadata::{
    DEFAULT_MAX_STRUCTURED_DATA_SIZE, DocumentMetadata, ExtendedMetadata, HeaderMetadata, ImageMetadata, ImageType,
    LinkMetadata, LinkType, MetadataConfig, StructuredData, StructuredDataType, TextDirection,
};
pub use options::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle,
    PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};

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
    let options = options.unwrap_or_default();

    let normalized_html = if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    };

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

    let options = options.unwrap_or_default();

    let normalized_html = if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    };

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

    let options = options.unwrap_or_default();

    let normalized_html = if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    };

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
        assert_eq!(metadata.document.language, Some("en".to_string()));
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
