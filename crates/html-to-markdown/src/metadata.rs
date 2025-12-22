//! Metadata extraction for HTML to Markdown conversion.
//!
//! This module provides comprehensive, type-safe metadata extraction during HTML-to-Markdown
//! conversion, enabling content analysis, SEO optimization, and document indexing workflows.
//! Metadata includes:
//! - **Document metadata**: Title, description, author, language, canonical URL, Open Graph, Twitter Card
//! - **Headers**: Heading elements (h1-h6) with hierarchy, IDs, and positions
//! - **Links**: Hyperlinks with type classification (anchor, internal, external, email, phone)
//! - **Images**: Image elements with source, alt text, dimensions, and type (data URI, external, etc.)
//! - **Structured data**: JSON-LD, Microdata, and RDFa blocks
//!
//! The implementation follows a single-pass collector pattern for zero-overhead extraction
//! when metadata features are disabled.
//!
//! # Architecture
//!
//! Metadata extraction uses the [`MetadataCollector`] pattern (similar to [`InlineImageCollector`]):
//! - **Single-pass collection**: Metadata is gathered during the primary tree traversal without additional passes
//! - **Zero overhead when disabled**: Entire module can be compiled out via feature flags
//! - **Configurable granularity**: Use [`MetadataConfig`] to select which metadata types to extract
//! - **Type-safe APIs**: All metadata types are enum-based with exhaustive matching
//! - **Memory-bounded**: Size limits prevent memory exhaustion from adversarial documents
//! - **Pre-allocated buffers**: Typical documents (32 headers, 64 links, 16 images) handled efficiently
//!
//! # Type Overview
//!
//! ## Enumerations
//!
//! - [`TextDirection`]: Document directionality (LTR, RTL, Auto)
//! - [`LinkType`]: Link classification (Anchor, Internal, External, Email, Phone, Other)
//! - [`ImageType`]: Image source type (DataUri, External, Relative, InlineSvg)
//! - [`StructuredDataType`]: Structured data format (JsonLd, Microdata, RDFa)
//!
//! ## Structures
//!
//! - [`DocumentMetadata`]: Head-level metadata with maps for Open Graph and Twitter Card
//! - [`HeaderMetadata`]: Heading element with level (1-6), text, ID, hierarchy depth, and position
//! - [`LinkMetadata`]: Hyperlink with href, text, title, type, rel attributes, and custom attributes
//! - [`ImageMetadata`]: Image element with src, alt, title, dimensions, type, and attributes
//! - [`StructuredData`]: Structured data block with type and raw JSON
//! - [`MetadataConfig`]: Configuration controlling extraction granularity and size limits
//! - [`ExtendedMetadata`]: Top-level result containing all extracted metadata
//!
//! # Examples
//!
//! ## Basic Usage with convert_with_metadata
//!
//! ```ignore
//! use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
//!
//! let html = r#"
//!   <html lang="en">
//!     <head>
//!       <title>My Article</title>
//!       <meta name="description" content="An interesting read">
//!     </head>
//!     <body>
//!       <h1 id="main">Title</h1>
//!       <a href="https://example.com">External Link</a>
//!       <img src="photo.jpg" alt="A photo">
//!     </body>
//!   </html>
//! "#;
//!
//! let config = MetadataConfig::default();
//! let (markdown, metadata) = convert_with_metadata(html, None, config)?;
//!
//! // Access document metadata
//! assert_eq!(metadata.document.title, Some("My Article".to_string()));
//! assert_eq!(metadata.document.language, Some("en".to_string()));
//!
//! // Access headers
//! assert_eq!(metadata.headers.len(), 1);
//! assert_eq!(metadata.headers[0].level, 1);
//! assert_eq!(metadata.headers[0].id, Some("main".to_string()));
//!
//! // Access links
//! assert_eq!(metadata.links.len(), 1);
//! assert_eq!(metadata.links[0].link_type, LinkType::External);
//!
//! // Access images
//! assert_eq!(metadata.images.len(), 1);
//! assert_eq!(metadata.images[0].image_type, ImageType::Relative);
//! # Ok::<(), html_to_markdown_rs::ConversionError>(())
//! ```
//!
//! ## Selective Extraction
//!
//! ```ignore
//! use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
//!
//! let config = MetadataConfig {
//!     extract_headers: true,
//!     extract_links: true,
//!     extract_images: false,  // Skip images
//!     extract_structured_data: false,  // Skip structured data
//!     max_structured_data_size: 0,
//! };
//!
//! let (markdown, metadata) = convert_with_metadata(html, None, config)?;
//! assert_eq!(metadata.images.len(), 0);  // Images not extracted
//! # Ok::<(), html_to_markdown_rs::ConversionError>(())
//! ```
//!
//! ## Analyzing Link Types
//!
//! ```ignore
//! use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
//! use html_to_markdown_rs::metadata::LinkType;
//!
//! let (_markdown, metadata) = convert_with_metadata(html, None, MetadataConfig::default())?;
//!
//! for link in &metadata.links {
//!     match link.link_type {
//!         LinkType::External => println!("External: {}", link.href),
//!         LinkType::Internal => println!("Internal: {}", link.href),
//!         LinkType::Anchor => println!("Anchor: {}", link.href),
//!         LinkType::Email => println!("Email: {}", link.href),
//!         _ => {}
//!     }
//! }
//! # Ok::<(), html_to_markdown_rs::ConversionError>(())
//! ```
//!
//! # Serialization
//!
//! All types in this module support serialization via `serde` when the `metadata` feature is enabled.
//! This enables easy export to JSON, YAML, or other formats:
//!
//! ```ignore
//! use html_to_markdown_rs::{convert_with_metadata, MetadataConfig};
//!
//! let (_markdown, metadata) = convert_with_metadata(html, None, MetadataConfig::default())?;
//! let json = serde_json::to_string_pretty(&metadata)?;
//! println!("{}", json);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Text directionality of document content.
///
/// Corresponds to the HTML `dir` attribute and `bdi` element directionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub enum TextDirection {
    /// Left-to-right text flow (default for Latin scripts)
    #[cfg_attr(feature = "metadata", serde(rename = "ltr"))]
    LeftToRight,
    /// Right-to-left text flow (Hebrew, Arabic, Urdu, etc.)
    #[cfg_attr(feature = "metadata", serde(rename = "rtl"))]
    RightToLeft,
    /// Automatic directionality detection
    #[cfg_attr(feature = "metadata", serde(rename = "auto"))]
    Auto,
}

impl std::fmt::Display for TextDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftToRight => write!(f, "ltr"),
            Self::RightToLeft => write!(f, "rtl"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl TextDirection {
    /// Parse a text direction from string value.
    ///
    /// # Arguments
    ///
    /// * `s` - Direction string ("ltr", "rtl", or "auto")
    ///
    /// # Returns
    ///
    /// `Some(TextDirection)` if valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use html_to_markdown_rs::metadata::TextDirection;
    /// assert_eq!(TextDirection::parse("ltr"), Some(TextDirection::LeftToRight));
    /// assert_eq!(TextDirection::parse("rtl"), Some(TextDirection::RightToLeft));
    /// assert_eq!(TextDirection::parse("auto"), Some(TextDirection::Auto));
    /// assert_eq!(TextDirection::parse("invalid"), None);
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("ltr") {
            return Some(Self::LeftToRight);
        }
        if s.eq_ignore_ascii_case("rtl") {
            return Some(Self::RightToLeft);
        }
        if s.eq_ignore_ascii_case("auto") {
            return Some(Self::Auto);
        }
        None
    }
}

/// Link classification based on href value and document context.
///
/// Used to categorize links during extraction for filtering and analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "metadata", serde(rename_all = "snake_case"))]
pub enum LinkType {
    /// Anchor link within same document (href starts with #)
    Anchor,
    /// Internal link within same domain
    Internal,
    /// External link to different domain
    External,
    /// Email link (mailto:)
    Email,
    /// Phone link (tel:)
    Phone,
    /// Other protocol or unclassifiable
    Other,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anchor => write!(f, "anchor"),
            Self::Internal => write!(f, "internal"),
            Self::External => write!(f, "external"),
            Self::Email => write!(f, "email"),
            Self::Phone => write!(f, "phone"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Image source classification for proper handling and processing.
///
/// Determines whether an image is embedded (data URI), inline SVG, external, or relative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "metadata", serde(rename_all = "snake_case"))]
pub enum ImageType {
    /// Data URI embedded image (base64 or other encoding)
    DataUri,
    /// Inline SVG element
    InlineSvg,
    /// External image URL (http/https)
    External,
    /// Relative image path
    Relative,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataUri => write!(f, "data_uri"),
            Self::InlineSvg => write!(f, "inline_svg"),
            Self::External => write!(f, "external"),
            Self::Relative => write!(f, "relative"),
        }
    }
}

/// Structured data format type.
///
/// Identifies the schema/format used for structured data markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "metadata", serde(rename_all = "snake_case"))]
pub enum StructuredDataType {
    /// JSON-LD (JSON for Linking Data) script blocks
    #[cfg_attr(feature = "metadata", serde(rename = "json_ld"))]
    JsonLd,
    /// HTML5 Microdata attributes (itemscope, itemtype, itemprop)
    Microdata,
    /// RDF in Attributes (RDFa) markup
    #[cfg_attr(feature = "metadata", serde(rename = "rdfa"))]
    RDFa,
}

impl std::fmt::Display for StructuredDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonLd => write!(f, "json_ld"),
            Self::Microdata => write!(f, "microdata"),
            Self::RDFa => write!(f, "rdfa"),
        }
    }
}

/// Document-level metadata extracted from `<head>` and top-level elements.
///
/// Contains all metadata typically used by search engines, social media platforms,
/// and browsers for document indexing and presentation.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::DocumentMetadata;
/// let doc = DocumentMetadata {
///     title: Some("My Article".to_string()),
///     description: Some("A great article about Rust".to_string()),
///     keywords: vec!["rust".to_string(), "programming".to_string()],
///     ..Default::default()
/// };
///
/// assert_eq!(doc.title, Some("My Article".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentMetadata {
    /// Document title from `<title>` tag
    pub title: Option<String>,

    /// Document description from `<meta name="description">` tag
    pub description: Option<String>,

    /// Document keywords from `<meta name="keywords">` tag, split on commas
    pub keywords: Vec<String>,

    /// Document author from `<meta name="author">` tag
    pub author: Option<String>,

    /// Canonical URL from `<link rel="canonical">` tag
    pub canonical_url: Option<String>,

    /// Base URL from `<base href="">` tag for resolving relative URLs
    pub base_href: Option<String>,

    /// Document language from `lang` attribute
    pub language: Option<String>,

    /// Document text direction from `dir` attribute
    pub text_direction: Option<TextDirection>,

    /// Open Graph metadata (og:* properties) for social media
    /// Keys like "title", "description", "image", "url", etc.
    pub open_graph: BTreeMap<String, String>,

    /// Twitter Card metadata (twitter:* properties)
    /// Keys like "card", "site", "creator", "title", "description", "image", etc.
    pub twitter_card: BTreeMap<String, String>,

    /// Additional meta tags not covered by specific fields
    /// Keys are meta name/property attributes, values are content
    pub meta_tags: BTreeMap<String, String>,
}

/// Header element metadata with hierarchy tracking.
///
/// Captures heading elements (h1-h6) with their text content, identifiers,
/// and position in the document structure.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::HeaderMetadata;
/// let header = HeaderMetadata {
///     level: 1,
///     text: "Main Title".to_string(),
///     id: Some("main-title".to_string()),
///     depth: 0,
///     html_offset: 145,
/// };
///
/// assert_eq!(header.level, 1);
/// assert!(header.is_valid());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderMetadata {
    /// Header level: 1 (h1) through 6 (h6)
    pub level: u8,

    /// Normalized text content of the header
    pub text: String,

    /// HTML id attribute if present
    pub id: Option<String>,

    /// Document tree depth at the header element
    pub depth: usize,

    /// Byte offset in original HTML document
    pub html_offset: usize,
}

impl HeaderMetadata {
    /// Validate that the header level is within valid range (1-6).
    ///
    /// # Returns
    ///
    /// `true` if level is 1-6, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use html_to_markdown_rs::metadata::HeaderMetadata;
    /// let valid = HeaderMetadata {
    ///     level: 3,
    ///     text: "Title".to_string(),
    ///     id: None,
    ///     depth: 2,
    ///     html_offset: 100,
    /// };
    /// assert!(valid.is_valid());
    ///
    /// let invalid = HeaderMetadata {
    ///     level: 7,  // Invalid
    ///     text: "Title".to_string(),
    ///     id: None,
    ///     depth: 2,
    ///     html_offset: 100,
    /// };
    /// assert!(!invalid.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.level >= 1 && self.level <= 6
    }
}

/// Hyperlink metadata with categorization and attributes.
///
/// Represents `<a>` elements with parsed href values, text content, and link type classification.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::{LinkMetadata, LinkType};
/// let link = LinkMetadata {
///     href: "https://example.com".to_string(),
///     text: "Example".to_string(),
///     title: Some("Visit Example".to_string()),
///     link_type: LinkType::External,
///     rel: vec!["nofollow".to_string()],
///     attributes: Default::default(),
/// };
///
/// assert_eq!(link.link_type, LinkType::External);
/// assert_eq!(link.text, "Example");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkMetadata {
    /// The href URL value
    pub href: String,

    /// Link text content (normalized, concatenated if mixed with elements)
    pub text: String,

    /// Optional title attribute (often shown as tooltip)
    pub title: Option<String>,

    /// Link type classification
    pub link_type: LinkType,

    /// Rel attribute values (e.g., "nofollow", "stylesheet", "canonical")
    pub rel: Vec<String>,

    /// Additional HTML attributes
    pub attributes: BTreeMap<String, String>,
}

impl LinkMetadata {
    /// Classify a link based on href value.
    ///
    /// # Arguments
    ///
    /// * `href` - The href attribute value
    ///
    /// # Returns
    ///
    /// Appropriate [`LinkType`] based on protocol and content.
    ///
    /// # Examples
    ///
    /// ```
    /// # use html_to_markdown_rs::metadata::{LinkMetadata, LinkType};
    /// assert_eq!(LinkMetadata::classify_link("#section"), LinkType::Anchor);
    /// assert_eq!(LinkMetadata::classify_link("mailto:test@example.com"), LinkType::Email);
    /// assert_eq!(LinkMetadata::classify_link("tel:+1234567890"), LinkType::Phone);
    /// assert_eq!(LinkMetadata::classify_link("https://example.com"), LinkType::External);
    /// ```
    pub fn classify_link(href: &str) -> LinkType {
        if href.starts_with('#') {
            LinkType::Anchor
        } else if href.starts_with("mailto:") {
            LinkType::Email
        } else if href.starts_with("tel:") {
            LinkType::Phone
        } else if href.starts_with("http://") || href.starts_with("https://") {
            LinkType::External
        } else if href.starts_with('/') || href.starts_with("../") || href.starts_with("./") {
            LinkType::Internal
        } else {
            LinkType::Other
        }
    }
}

/// Image metadata with source and dimensions.
///
/// Captures `<img>` elements and inline `<svg>` elements with metadata
/// for image analysis and optimization.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::{ImageMetadata, ImageType};
/// let img = ImageMetadata {
///     src: "https://example.com/image.jpg".to_string(),
///     alt: Some("An example image".to_string()),
///     title: Some("Example".to_string()),
///     dimensions: Some((800, 600)),
///     image_type: ImageType::External,
///     attributes: Default::default(),
/// };
///
/// assert_eq!(img.image_type, ImageType::External);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageMetadata {
    /// Image source (URL, data URI, or SVG content identifier)
    pub src: String,

    /// Alternative text from alt attribute (for accessibility)
    pub alt: Option<String>,

    /// Title attribute (often shown as tooltip)
    pub title: Option<String>,

    /// Image dimensions as (width, height) if available
    pub dimensions: Option<(u32, u32)>,

    /// Image type classification
    pub image_type: ImageType,

    /// Additional HTML attributes
    pub attributes: BTreeMap<String, String>,
}

/// Structured data block (JSON-LD, Microdata, or RDFa).
///
/// Represents machine-readable structured data found in the document.
/// JSON-LD blocks are collected as raw JSON strings for flexibility.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::{StructuredData, StructuredDataType};
/// let schema = StructuredData {
///     data_type: StructuredDataType::JsonLd,
///     raw_json: r#"{"@context":"https://schema.org","@type":"Article"}"#.to_string(),
///     schema_type: Some("Article".to_string()),
/// };
///
/// assert_eq!(schema.data_type, StructuredDataType::JsonLd);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct StructuredData {
    /// Type of structured data (JSON-LD, Microdata, RDFa)
    pub data_type: StructuredDataType,

    /// Raw JSON string (for JSON-LD) or serialized representation
    pub raw_json: String,

    /// Schema type if detectable (e.g., "Article", "Event", "Product")
    pub schema_type: Option<String>,
}

/// Default maximum size for structured data extraction (1 MB)
pub const DEFAULT_MAX_STRUCTURED_DATA_SIZE: usize = 1_000_000;

/// Configuration for metadata extraction granularity.
///
/// Controls which metadata types are extracted and size limits for safety.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::MetadataConfig;
/// let config = MetadataConfig {
///     extract_document: true,
///     extract_headers: true,
///     extract_links: true,
///     extract_images: true,
///     extract_structured_data: true,
///     max_structured_data_size: 1_000_000,
/// };
///
/// assert!(config.extract_headers);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataConfig {
    /// Extract document-level metadata (title, description, author, etc.)
    pub extract_document: bool,

    /// Extract h1-h6 header elements and their hierarchy
    pub extract_headers: bool,

    /// Extract anchor (a) elements as links with type classification
    pub extract_links: bool,

    /// Extract image elements and data URIs
    pub extract_images: bool,

    /// Extract structured data (JSON-LD, Microdata, RDFa)
    pub extract_structured_data: bool,

    /// Maximum total size of structured data to collect (bytes)
    /// Prevents memory exhaustion on malformed or adversarial documents
    pub max_structured_data_size: usize,
}

/// Partial update for MetadataConfig.
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct MetadataConfigUpdate {
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(alias = "extract_document"))]
    pub extract_document: Option<bool>,
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(alias = "extract_headers"))]
    pub extract_headers: Option<bool>,
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(alias = "extract_links"))]
    pub extract_links: Option<bool>,
    #[cfg_attr(any(feature = "serde", feature = "metadata"), serde(alias = "extract_images"))]
    pub extract_images: Option<bool>,
    #[cfg_attr(
        any(feature = "serde", feature = "metadata"),
        serde(alias = "extract_structured_data")
    )]
    pub extract_structured_data: Option<bool>,
    #[cfg_attr(
        any(feature = "serde", feature = "metadata"),
        serde(alias = "max_structured_data_size")
    )]
    pub max_structured_data_size: Option<usize>,
}

impl Default for MetadataConfig {
    /// Create default metadata configuration.
    ///
    /// Defaults to extracting all metadata types with 1MB limit on structured data.
    fn default() -> Self {
        Self {
            extract_document: true,
            extract_headers: true,
            extract_links: true,
            extract_images: true,
            extract_structured_data: true,
            max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
        }
    }
}

impl MetadataConfig {
    pub fn any_enabled(&self) -> bool {
        self.extract_document
            || self.extract_headers
            || self.extract_links
            || self.extract_images
            || self.extract_structured_data
    }

    pub fn apply_update(&mut self, update: MetadataConfigUpdate) {
        if let Some(extract_document) = update.extract_document {
            self.extract_document = extract_document;
        }
        if let Some(extract_headers) = update.extract_headers {
            self.extract_headers = extract_headers;
        }
        if let Some(extract_links) = update.extract_links {
            self.extract_links = extract_links;
        }
        if let Some(extract_images) = update.extract_images {
            self.extract_images = extract_images;
        }
        if let Some(extract_structured_data) = update.extract_structured_data {
            self.extract_structured_data = extract_structured_data;
        }
        if let Some(max_structured_data_size) = update.max_structured_data_size {
            self.max_structured_data_size = max_structured_data_size;
        }
    }

    pub fn from_update(update: MetadataConfigUpdate) -> Self {
        let mut config = Self::default();
        config.apply_update(update);
        config
    }
}

impl From<MetadataConfigUpdate> for MetadataConfig {
    fn from(update: MetadataConfigUpdate) -> Self {
        Self::from_update(update)
    }
}

/// Comprehensive metadata extraction result from HTML document.
///
/// Contains all extracted metadata types in a single structure,
/// suitable for serialization and transmission across language boundaries.
///
/// # Examples
///
/// ```
/// # use html_to_markdown_rs::metadata::ExtendedMetadata;
/// let metadata = ExtendedMetadata {
///     document: Default::default(),
///     headers: Vec::new(),
///     links: Vec::new(),
///     images: Vec::new(),
///     structured_data: Vec::new(),
/// };
///
/// assert!(metadata.headers.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "metadata", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendedMetadata {
    /// Document-level metadata (title, description, canonical, etc.)
    pub document: DocumentMetadata,

    /// Extracted header elements with hierarchy
    pub headers: Vec<HeaderMetadata>,

    /// Extracted hyperlinks with type classification
    pub links: Vec<LinkMetadata>,

    /// Extracted images with source and dimensions
    pub images: Vec<ImageMetadata>,

    /// Extracted structured data blocks
    pub structured_data: Vec<StructuredData>,
}

/// Internal metadata collector for single-pass extraction.
///
/// Follows the [`InlineImageCollector`](crate::inline_images::InlineImageCollector) pattern
/// for efficient metadata extraction during tree traversal. Maintains state for:
/// - Document metadata from head elements
/// - Header hierarchy tracking
/// - Link accumulation
/// - Structured data collection
/// - Language and directionality attributes
///
/// # Architecture
///
/// The collector is designed to be:
/// - **Performant**: Pre-allocated collections, minimal cloning
/// - **Single-pass**: Collects during main tree walk without separate passes
/// - **Optional**: Zero overhead when disabled via feature flags
/// - **Type-safe**: Strict separation of collection and result types
///
/// # Internal State
///
/// - `head_metadata`: Raw metadata pairs from head element
/// - `headers`: Collected header elements
/// - `header_stack`: For tracking nesting depth
/// - `links`: Collected link elements
/// - `base_href`: Base URL for relative link resolution
/// - `json_ld`: JSON-LD script block contents
/// - `lang`: Document language
/// - `dir`: Document text direction
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct MetadataCollector {
    head_metadata: BTreeMap<String, String>,
    headers: Vec<HeaderMetadata>,
    header_stack: Vec<usize>,
    links: Vec<LinkMetadata>,
    images: Vec<ImageMetadata>,
    json_ld: Vec<String>,
    structured_data_size: usize,
    config: MetadataConfig,
    lang: Option<String>,
    dir: Option<String>,
}

#[allow(dead_code)]
impl MetadataCollector {
    /// Create a new metadata collector with configuration.
    ///
    /// Pre-allocates collections based on typical document sizes
    /// for efficient append operations during traversal.
    ///
    /// # Arguments
    ///
    /// * `config` - Extraction configuration specifying which types to collect
    ///
    /// # Returns
    ///
    /// A new collector ready for use during tree traversal.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = MetadataConfig::default();
    /// let collector = MetadataCollector::new(config);
    /// ```
    pub(crate) fn new(config: MetadataConfig) -> Self {
        Self {
            head_metadata: BTreeMap::new(),
            headers: Vec::with_capacity(32),
            header_stack: Vec::with_capacity(6),
            links: Vec::with_capacity(64),
            images: Vec::with_capacity(16),
            json_ld: Vec::with_capacity(4),
            structured_data_size: 0,
            config,
            lang: None,
            dir: None,
        }
    }

    /// Add a header element to the collection.
    ///
    /// Validates that level is in range 1-6 and tracks hierarchy via depth.
    ///
    /// # Arguments
    ///
    /// * `level` - Header level (1-6)
    /// * `text` - Normalized header text content
    /// * `id` - Optional HTML id attribute
    /// * `depth` - Current document nesting depth
    /// * `html_offset` - Byte offset in original HTML
    pub(crate) fn add_header(&mut self, level: u8, text: String, id: Option<String>, depth: usize, html_offset: usize) {
        if !self.config.extract_headers {
            return;
        }

        if !(1..=6).contains(&level) {
            return;
        }

        let header = HeaderMetadata {
            level,
            text,
            id,
            depth,
            html_offset,
        };

        self.headers.push(header);
    }

    /// Add a link element to the collection.
    ///
    /// Classifies the link based on href value and stores with metadata.
    ///
    /// # Arguments
    ///
    /// * `href` - The href attribute value
    /// * `text` - Link text content
    /// * `title` - Optional title attribute
    /// * `rel` - Comma/space-separated rel attribute value
    /// * `attributes` - Additional attributes to capture (e.g., data-* or aria-* values)
    pub(crate) fn add_link(
        &mut self,
        href: String,
        text: String,
        title: Option<String>,
        rel: Option<String>,
        attributes: BTreeMap<String, String>,
    ) {
        if !self.config.extract_links {
            return;
        }

        let link_type = LinkMetadata::classify_link(&href);

        let rel_vec = rel
            .map(|r| r.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();

        let link = LinkMetadata {
            href,
            text,
            title,
            link_type,
            rel: rel_vec,
            attributes,
        };

        self.links.push(link);
    }

    /// Add an image element to the collection.
    ///
    /// # Arguments
    ///
    /// * `src` - Image source (URL or data URI)
    /// * `alt` - Optional alt text
    /// * `title` - Optional title attribute
    /// * `dimensions` - Optional (width, height) tuple
    pub(crate) fn add_image(
        &mut self,
        src: String,
        alt: Option<String>,
        title: Option<String>,
        dimensions: Option<(u32, u32)>,
        attributes: BTreeMap<String, String>,
    ) {
        if !self.config.extract_images {
            return;
        }

        let image_type = if src.starts_with("data:") {
            ImageType::DataUri
        } else if src.starts_with("http://") || src.starts_with("https://") {
            ImageType::External
        } else if src.starts_with('<') && src.contains("svg") {
            ImageType::InlineSvg
        } else {
            ImageType::Relative
        };

        let image = ImageMetadata {
            src,
            alt,
            title,
            dimensions,
            image_type,
            attributes,
        };

        self.images.push(image);
    }

    /// Add a JSON-LD structured data block.
    ///
    /// Accumulates JSON content with size validation against configured limits.
    ///
    /// # Arguments
    ///
    /// * `json_content` - Raw JSON string content
    pub(crate) fn add_json_ld(&mut self, json_content: String) {
        if !self.config.extract_structured_data {
            return;
        }

        let content_size = json_content.len();
        if content_size > self.config.max_structured_data_size {
            return;
        }
        if self.structured_data_size + content_size > self.config.max_structured_data_size {
            return;
        }

        self.structured_data_size += content_size;
        self.json_ld.push(json_content);
    }

    /// Set document head metadata from extracted head section.
    ///
    /// Merges metadata pairs from head elements (meta, title, link, etc.)
    /// into the collector's head metadata store.
    ///
    /// # Arguments
    ///
    /// * `metadata` - BTreeMap of metadata key-value pairs
    pub(crate) fn set_head_metadata(&mut self, metadata: BTreeMap<String, String>) {
        if !self.config.extract_document {
            return;
        }
        self.head_metadata.extend(metadata);
    }

    /// Set document language attribute.
    ///
    /// Usually from `lang` attribute on `<html>` or `<body>` tag.
    /// Only sets if not already set (first occurrence wins).
    ///
    /// # Arguments
    ///
    /// * `lang` - Language code (e.g., "en", "es", "fr")
    pub(crate) fn set_language(&mut self, lang: String) {
        if !self.config.extract_document {
            return;
        }
        if self.lang.is_none() {
            self.lang = Some(lang);
        }
    }

    /// Set document text direction attribute.
    ///
    /// Usually from `dir` attribute on `<html>` or `<body>` tag.
    /// Only sets if not already set (first occurrence wins).
    ///
    /// # Arguments
    ///
    /// * `dir` - Direction string ("ltr", "rtl", or "auto")
    pub(crate) fn set_text_direction(&mut self, dir: String) {
        if !self.config.extract_document {
            return;
        }
        if self.dir.is_none() {
            self.dir = Some(dir);
        }
    }

    pub(crate) fn wants_document(&self) -> bool {
        self.config.extract_document
    }

    pub(crate) fn wants_headers(&self) -> bool {
        self.config.extract_headers
    }

    pub(crate) fn wants_links(&self) -> bool {
        self.config.extract_links
    }

    pub(crate) fn wants_images(&self) -> bool {
        self.config.extract_images
    }

    pub(crate) fn wants_structured_data(&self) -> bool {
        self.config.extract_structured_data
    }

    /// Extract document metadata from collected head metadata.
    ///
    /// Parses head metadata into structured document metadata,
    /// handling special cases like Open Graph, Twitter Card, keywords, etc.
    #[allow(dead_code)]
    fn extract_document_metadata(
        head_metadata: BTreeMap<String, String>,
        lang: Option<String>,
        dir: Option<String>,
    ) -> DocumentMetadata {
        let mut doc = DocumentMetadata::default();

        for (raw_key, value) in head_metadata {
            let mut key = raw_key.as_str();
            let mut replaced_key: Option<String> = None;

            if let Some(stripped) = key.strip_prefix("meta-") {
                key = stripped;
            }

            if key.as_bytes().contains(&b':') {
                replaced_key = Some(key.replace(':', "-"));
                key = replaced_key.as_deref().unwrap_or(key);
            }

            match key {
                "title" => doc.title = Some(value),
                "description" => doc.description = Some(value),
                "author" => doc.author = Some(value),
                "canonical" => doc.canonical_url = Some(value),
                "base" | "base-href" => doc.base_href = Some(value),
                key if key.starts_with("og-") => {
                    let og_key = if key.as_bytes().contains(&b'-') {
                        key.trim_start_matches("og-").replace('-', "_")
                    } else {
                        key.trim_start_matches("og-").to_string()
                    };
                    doc.open_graph.insert(og_key, value);
                }
                key if key.starts_with("twitter-") => {
                    let tw_key = if key.as_bytes().contains(&b'-') {
                        key.trim_start_matches("twitter-").replace('-', "_")
                    } else {
                        key.trim_start_matches("twitter-").to_string()
                    };
                    doc.twitter_card.insert(tw_key, value);
                }
                "keywords" => {
                    doc.keywords = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {
                    let meta_key = if key.as_ptr() == raw_key.as_ptr() && key.len() == raw_key.len() {
                        raw_key
                    } else if let Some(replaced) = replaced_key {
                        replaced
                    } else {
                        key.to_string()
                    };
                    doc.meta_tags.insert(meta_key, value);
                }
            }
        }

        if let Some(lang) = lang {
            doc.language = Some(lang);
        }

        if let Some(dir) = dir {
            if let Some(parsed_dir) = TextDirection::parse(&dir) {
                doc.text_direction = Some(parsed_dir);
            }
        }

        doc
    }

    /// Extract structured data blocks into StructuredData items.
    #[allow(dead_code)]
    fn extract_structured_data(json_ld: Vec<String>) -> Vec<StructuredData> {
        let mut result = Vec::with_capacity(json_ld.len());

        for json_str in json_ld {
            let schema_type = Self::scan_schema_type(&json_str)
                .or_else(|| {
                    if json_str.contains("\"@type\"") {
                        serde_json::from_str::<serde_json::Value>(&json_str)
                            .ok()
                            .and_then(|v| v.get("@type").and_then(|t| t.as_str().map(|s| s.to_string())))
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    if !json_str.contains("\"@graph\"") {
                        return None;
                    }

                    let value = serde_json::from_str::<serde_json::Value>(&json_str).ok()?;
                    let graph = value.get("@graph")?;
                    let items = graph.as_array()?;
                    items
                        .iter()
                        .find_map(|item| item.get("@type").and_then(|t| t.as_str().map(|s| s.to_string())))
                });

            result.push(StructuredData {
                data_type: StructuredDataType::JsonLd,
                raw_json: json_str,
                schema_type,
            });
        }

        result
    }

    fn scan_schema_type(json_str: &str) -> Option<String> {
        let needle = "\"@type\"";
        let start = json_str.find(needle)? + needle.len();
        let bytes = json_str.as_bytes();
        let mut i = start;

        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            return None;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }

        if bytes[i] == b'[' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'"' {
                return None;
            }
        } else if bytes[i] != b'"' {
            return None;
        }

        let start_quote = i;
        i += 1;
        let mut escaped = false;
        while i < bytes.len() {
            let byte = bytes[i];
            if escaped {
                escaped = false;
                i += 1;
                continue;
            }
            if byte == b'\\' {
                escaped = true;
                i += 1;
                continue;
            }
            if byte == b'"' {
                let end_quote = i;
                let slice = &json_str[start_quote..=end_quote];
                return serde_json::from_str::<String>(slice).ok();
            }
            i += 1;
        }

        None
    }

    /// Finish collection and return all extracted metadata.
    ///
    /// Performs final processing, validation, and consolidation of all
    /// collected data into the [`ExtendedMetadata`] output structure.
    ///
    /// # Returns
    ///
    /// Complete [`ExtendedMetadata`] with all extracted information.
    #[allow(dead_code)]
    pub(crate) fn finish(self) -> ExtendedMetadata {
        let structured_data = Self::extract_structured_data(self.json_ld);
        let document = Self::extract_document_metadata(self.head_metadata, self.lang, self.dir);

        ExtendedMetadata {
            document,
            headers: self.headers,
            links: self.links,
            images: self.images,
            structured_data,
        }
    }

    /// Categorize links by type for analysis and filtering.
    ///
    /// Separates collected links into groups by [`LinkType`].
    /// This is an analysis helper method; actual categorization happens during add_link.
    ///
    /// # Returns
    ///
    /// BTreeMap with LinkType as key and Vec of matching LinkMetadata as value.
    #[allow(dead_code)]
    pub(crate) fn categorize_links(&self) -> BTreeMap<String, Vec<&LinkMetadata>> {
        let mut categorized: BTreeMap<String, Vec<&LinkMetadata>> = BTreeMap::new();

        for link in &self.links {
            let category = link.link_type.to_string();
            categorized.entry(category).or_default().push(link);
        }

        categorized
    }

    /// Count headers by level for structural analysis.
    ///
    /// Returns count of headers at each level (1-6).
    ///
    /// # Returns
    ///
    /// BTreeMap with level as string key and count as value.
    #[allow(dead_code)]
    pub(crate) fn header_counts(&self) -> BTreeMap<String, usize> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();

        for header in &self.headers {
            *counts.entry(header.level.to_string()).or_insert(0) += 1;
        }

        counts
    }
}

/// Handle to a metadata collector via reference-counted mutable cell.
///
/// Used internally for sharing collector state across the tree traversal.
/// Matches the pattern used for [`InlineImageCollector`](crate::inline_images::InlineImageCollector).
///
/// # Examples
///
/// ```ignore
/// let collector = MetadataCollector::new(MetadataConfig::default());
/// let handle = Rc::new(RefCell::new(collector));
///
/// // In tree walk, can be passed and borrowed
/// handle.borrow_mut().add_header(1, "Title".to_string(), None, 0, 100);
///
/// let metadata = handle.take().finish();
/// ```
#[allow(dead_code)]
pub(crate) type MetadataCollectorHandle = Rc<RefCell<MetadataCollector>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_direction_parse() {
        assert_eq!(TextDirection::parse("ltr"), Some(TextDirection::LeftToRight));
        assert_eq!(TextDirection::parse("rtl"), Some(TextDirection::RightToLeft));
        assert_eq!(TextDirection::parse("auto"), Some(TextDirection::Auto));
        assert_eq!(TextDirection::parse("invalid"), None);
        assert_eq!(TextDirection::parse("LTR"), Some(TextDirection::LeftToRight));
    }

    #[test]
    fn test_text_direction_display() {
        assert_eq!(TextDirection::LeftToRight.to_string(), "ltr");
        assert_eq!(TextDirection::RightToLeft.to_string(), "rtl");
        assert_eq!(TextDirection::Auto.to_string(), "auto");
    }

    #[test]
    fn test_link_classification() {
        assert_eq!(LinkMetadata::classify_link("#section"), LinkType::Anchor);
        assert_eq!(LinkMetadata::classify_link("mailto:test@example.com"), LinkType::Email);
        assert_eq!(LinkMetadata::classify_link("tel:+1234567890"), LinkType::Phone);
        assert_eq!(LinkMetadata::classify_link("https://example.com"), LinkType::External);
        assert_eq!(LinkMetadata::classify_link("http://example.com"), LinkType::External);
        assert_eq!(LinkMetadata::classify_link("/path/to/page"), LinkType::Internal);
        assert_eq!(LinkMetadata::classify_link("../relative"), LinkType::Internal);
        assert_eq!(LinkMetadata::classify_link("./same"), LinkType::Internal);
    }

    #[test]
    fn test_header_validation() {
        let valid = HeaderMetadata {
            level: 3,
            text: "Title".to_string(),
            id: None,
            depth: 2,
            html_offset: 100,
        };
        assert!(valid.is_valid());

        let invalid_high = HeaderMetadata {
            level: 7,
            text: "Title".to_string(),
            id: None,
            depth: 2,
            html_offset: 100,
        };
        assert!(!invalid_high.is_valid());

        let invalid_low = HeaderMetadata {
            level: 0,
            text: "Title".to_string(),
            id: None,
            depth: 2,
            html_offset: 100,
        };
        assert!(!invalid_low.is_valid());
    }

    #[test]
    fn test_metadata_collector_new() {
        let config = MetadataConfig::default();
        let collector = MetadataCollector::new(config);

        assert_eq!(collector.headers.capacity(), 32);
        assert_eq!(collector.links.capacity(), 64);
        assert_eq!(collector.images.capacity(), 16);
        assert_eq!(collector.json_ld.capacity(), 4);
    }

    #[test]
    fn test_metadata_collector_add_header() {
        let config = MetadataConfig::default();
        let mut collector = MetadataCollector::new(config);

        collector.add_header(1, "Title".to_string(), Some("title".to_string()), 0, 100);
        assert_eq!(collector.headers.len(), 1);

        let header = &collector.headers[0];
        assert_eq!(header.level, 1);
        assert_eq!(header.text, "Title");
        assert_eq!(header.id, Some("title".to_string()));

        collector.add_header(7, "Invalid".to_string(), None, 0, 200);
        assert_eq!(collector.headers.len(), 1);
    }

    #[test]
    fn test_metadata_collector_add_link() {
        let config = MetadataConfig::default();
        let mut collector = MetadataCollector::new(config);

        collector.add_link(
            "https://example.com".to_string(),
            "Example".to_string(),
            Some("Visit".to_string()),
            Some("nofollow external".to_string()),
            BTreeMap::from([("data-id".to_string(), "example".to_string())]),
        );

        assert_eq!(collector.links.len(), 1);

        let link = &collector.links[0];
        assert_eq!(link.href, "https://example.com");
        assert_eq!(link.text, "Example");
        assert_eq!(link.link_type, LinkType::External);
        assert_eq!(link.rel, vec!["nofollow", "external"]);
        assert_eq!(link.attributes.get("data-id"), Some(&"example".to_string()));
    }

    #[test]
    fn test_metadata_collector_respects_config() {
        let config = MetadataConfig {
            extract_document: false,
            extract_headers: false,
            extract_links: false,
            extract_images: false,
            extract_structured_data: false,
            max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
        };
        let mut collector = MetadataCollector::new(config);

        collector.add_header(1, "Title".to_string(), None, 0, 100);
        collector.add_link(
            "https://example.com".to_string(),
            "Link".to_string(),
            None,
            None,
            BTreeMap::new(),
        );
        collector.add_image(
            "https://example.com/img.jpg".to_string(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        collector.add_json_ld("{}".to_string());

        assert!(collector.headers.is_empty());
        assert!(collector.links.is_empty());
        assert!(collector.images.is_empty());
        assert!(collector.json_ld.is_empty());
    }

    #[test]
    fn test_metadata_collector_finish() {
        let config = MetadataConfig::default();
        let mut collector = MetadataCollector::new(config);

        collector.set_language("en".to_string());
        collector.add_header(1, "Main Title".to_string(), None, 0, 100);
        collector.add_link(
            "https://example.com".to_string(),
            "Example".to_string(),
            None,
            None,
            BTreeMap::new(),
        );

        let metadata = collector.finish();

        assert_eq!(metadata.document.language, Some("en".to_string()));
        assert_eq!(metadata.headers.len(), 1);
        assert_eq!(metadata.links.len(), 1);
    }

    #[test]
    fn test_document_metadata_default() {
        let doc = DocumentMetadata::default();

        assert!(doc.title.is_none());
        assert!(doc.description.is_none());
        assert!(doc.keywords.is_empty());
        assert!(doc.open_graph.is_empty());
        assert!(doc.twitter_card.is_empty());
        assert!(doc.meta_tags.is_empty());
    }

    #[test]
    fn test_metadata_config_default() {
        let config = MetadataConfig::default();

        assert!(config.extract_headers);
        assert!(config.extract_links);
        assert!(config.extract_images);
        assert!(config.extract_structured_data);
        assert_eq!(config.max_structured_data_size, DEFAULT_MAX_STRUCTURED_DATA_SIZE);
    }

    #[test]
    fn test_image_type_classification() {
        let data_uri = ImageMetadata {
            src: "data:image/png;base64,iVBORw0KG...".to_string(),
            alt: None,
            title: None,
            dimensions: None,
            image_type: ImageType::DataUri,
            attributes: BTreeMap::new(),
        };
        assert_eq!(data_uri.image_type, ImageType::DataUri);

        let external = ImageMetadata {
            src: "https://example.com/image.jpg".to_string(),
            alt: None,
            title: None,
            dimensions: None,
            image_type: ImageType::External,
            attributes: BTreeMap::new(),
        };
        assert_eq!(external.image_type, ImageType::External);
    }

    #[test]
    fn test_link_type_display() {
        assert_eq!(LinkType::Anchor.to_string(), "anchor");
        assert_eq!(LinkType::Internal.to_string(), "internal");
        assert_eq!(LinkType::External.to_string(), "external");
        assert_eq!(LinkType::Email.to_string(), "email");
        assert_eq!(LinkType::Phone.to_string(), "phone");
        assert_eq!(LinkType::Other.to_string(), "other");
    }

    #[test]
    fn test_structured_data_type_display() {
        assert_eq!(StructuredDataType::JsonLd.to_string(), "json_ld");
        assert_eq!(StructuredDataType::Microdata.to_string(), "microdata");
        assert_eq!(StructuredDataType::RDFa.to_string(), "rdfa");
    }

    #[test]
    fn test_categorize_links() {
        let config = MetadataConfig::default();
        let mut collector = MetadataCollector::new(config);

        collector.add_link("#anchor".to_string(), "Anchor".to_string(), None, None, BTreeMap::new());
        collector.add_link(
            "https://example.com".to_string(),
            "External".to_string(),
            None,
            None,
            BTreeMap::new(),
        );
        collector.add_link(
            "mailto:test@example.com".to_string(),
            "Email".to_string(),
            None,
            None,
            BTreeMap::new(),
        );

        let categorized = collector.categorize_links();

        assert_eq!(categorized.get("anchor").map(|v| v.len()), Some(1));
        assert_eq!(categorized.get("external").map(|v| v.len()), Some(1));
        assert_eq!(categorized.get("email").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_header_counts() {
        let config = MetadataConfig::default();
        let mut collector = MetadataCollector::new(config);

        collector.add_header(1, "H1".to_string(), None, 0, 100);
        collector.add_header(2, "H2".to_string(), None, 1, 200);
        collector.add_header(2, "H2b".to_string(), None, 1, 300);
        collector.add_header(3, "H3".to_string(), None, 2, 400);

        let counts = collector.header_counts();

        assert_eq!(counts.get("1").copied(), Some(1));
        assert_eq!(counts.get("2").copied(), Some(2));
        assert_eq!(counts.get("3").copied(), Some(1));
    }
}
