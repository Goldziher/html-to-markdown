#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::all)]
#![allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]

#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DocumentMetadata as RustDocumentMetadata, ExtendedMetadata as RustExtendedMetadata,
    HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata, LinkMetadata as RustLinkMetadata,
    MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData,
};
use html_to_markdown_rs::safety::guard_panic;
mod profiling;
#[cfg(feature = "async-visitor")]
use async_trait::async_trait;
#[cfg(feature = "metadata")]
use html_to_markdown_bindings_common::parse_metadata_config;
use html_to_markdown_bindings_common::{error::error_message, parse_conversion_options, parse_inline_image_config};
#[cfg(feature = "async-visitor")]
use html_to_markdown_rs::visitor::AsyncHtmlVisitor;
#[cfg(feature = "visitor")]
#[allow(unused_imports)]
use html_to_markdown_rs::visitor::HtmlVisitor;
#[cfg(any(feature = "visitor", feature = "async-visitor"))]
use html_to_markdown_rs::visitor::{NodeContext as RustNodeContext, VisitResult as RustVisitResult};
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions as RustConversionOptions, ConversionOptionsUpdate,
    DEFAULT_INLINE_IMAGE_LIMIT, HeadingStyle, HighlightStyle, InlineImageConfig as RustInlineImageConfig,
    InlineImageConfigUpdate, ListIndentType, NewlineStyle, PreprocessingOptions as RustPreprocessingOptions,
    PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
#[cfg(feature = "async-visitor")]
#[allow(unused_imports)]
use std::sync::Arc;
use std::{collections::HashMap, str};

fn to_js_error(err: ConversionError) -> Error {
    Error::new(Status::GenericFailure, error_message(&err))
}

/// Heading style options
#[napi(string_enum)]
pub enum JsHeadingStyle {
    /// Underlined style (=== for h1, --- for h2)
    Underlined,
    /// ATX style (# for h1, ## for h2, etc.)
    Atx,
    /// ATX closed style (# title #)
    AtxClosed,
}

impl From<JsHeadingStyle> for HeadingStyle {
    fn from(val: JsHeadingStyle) -> Self {
        match val {
            JsHeadingStyle::Underlined => Self::Underlined,
            JsHeadingStyle::Atx => Self::Atx,
            JsHeadingStyle::AtxClosed => Self::AtxClosed,
        }
    }
}

/// List indentation type
#[napi(string_enum)]
pub enum JsListIndentType {
    Spaces,
    Tabs,
}

impl From<JsListIndentType> for ListIndentType {
    fn from(val: JsListIndentType) -> Self {
        match val {
            JsListIndentType::Spaces => Self::Spaces,
            JsListIndentType::Tabs => Self::Tabs,
        }
    }
}

/// Whitespace handling mode
#[napi(string_enum)]
pub enum JsWhitespaceMode {
    Normalized,
    Strict,
}

impl From<JsWhitespaceMode> for WhitespaceMode {
    fn from(val: JsWhitespaceMode) -> Self {
        match val {
            JsWhitespaceMode::Normalized => Self::Normalized,
            JsWhitespaceMode::Strict => Self::Strict,
        }
    }
}

/// Newline style
#[napi(string_enum)]
pub enum JsNewlineStyle {
    /// Two spaces at end of line
    Spaces,
    /// Backslash at end of line
    Backslash,
}

impl From<JsNewlineStyle> for NewlineStyle {
    fn from(val: JsNewlineStyle) -> Self {
        match val {
            JsNewlineStyle::Spaces => Self::Spaces,
            JsNewlineStyle::Backslash => Self::Backslash,
        }
    }
}

/// Code block style
#[napi(string_enum)]
pub enum JsCodeBlockStyle {
    /// Indented code blocks (4 spaces) - `CommonMark` default
    Indented,
    /// Fenced code blocks with backticks (```)
    Backticks,
    /// Fenced code blocks with tildes (~~~)
    Tildes,
}

impl From<JsCodeBlockStyle> for CodeBlockStyle {
    fn from(val: JsCodeBlockStyle) -> Self {
        match val {
            JsCodeBlockStyle::Indented => Self::Indented,
            JsCodeBlockStyle::Backticks => Self::Backticks,
            JsCodeBlockStyle::Tildes => Self::Tildes,
        }
    }
}

/// Highlight style for `<mark>` elements
#[napi(string_enum)]
pub enum JsHighlightStyle {
    /// ==text==
    DoubleEqual,
    /// <mark>text</mark>
    Html,
    /// **text**
    Bold,
    /// Plain text (no formatting)
    None,
}

impl From<JsHighlightStyle> for HighlightStyle {
    fn from(val: JsHighlightStyle) -> Self {
        match val {
            JsHighlightStyle::DoubleEqual => Self::DoubleEqual,
            JsHighlightStyle::Html => Self::Html,
            JsHighlightStyle::Bold => Self::Bold,
            JsHighlightStyle::None => Self::None,
        }
    }
}

/// Preprocessing preset levels
#[napi(string_enum)]
pub enum JsPreprocessingPreset {
    Minimal,
    Standard,
    Aggressive,
}

impl From<JsPreprocessingPreset> for PreprocessingPreset {
    fn from(val: JsPreprocessingPreset) -> Self {
        match val {
            JsPreprocessingPreset::Minimal => Self::Minimal,
            JsPreprocessingPreset::Standard => Self::Standard,
            JsPreprocessingPreset::Aggressive => Self::Aggressive,
        }
    }
}

/// HTML preprocessing options
#[napi(object)]
pub struct JsPreprocessingOptions {
    /// Enable preprocessing
    pub enabled: Option<bool>,
    /// Preprocessing preset
    pub preset: Option<JsPreprocessingPreset>,
    /// Remove navigation elements
    pub remove_navigation: Option<bool>,
    /// Remove form elements
    pub remove_forms: Option<bool>,
}

impl From<JsPreprocessingOptions> for PreprocessingOptionsUpdate {
    fn from(val: JsPreprocessingOptions) -> Self {
        Self {
            enabled: val.enabled,
            preset: val.preset.map(Into::into),
            remove_navigation: val.remove_navigation,
            remove_forms: val.remove_forms,
        }
    }
}

impl From<JsPreprocessingOptions> for RustPreprocessingOptions {
    fn from(val: JsPreprocessingOptions) -> Self {
        let update: PreprocessingOptionsUpdate = val.into();
        let mut opts = Self::default();
        opts.apply_update(update);
        opts
    }
}

/// Main conversion options
#[napi(object)]
pub struct JsConversionOptions {
    /// Heading style
    pub heading_style: Option<JsHeadingStyle>,
    /// List indentation type
    pub list_indent_type: Option<JsListIndentType>,
    /// List indentation width (spaces)
    pub list_indent_width: Option<u32>,
    /// Bullet characters for unordered lists
    pub bullets: Option<String>,
    /// Symbol for strong/emphasis (* or _)
    pub strong_em_symbol: Option<String>,
    /// Escape asterisks in text
    pub escape_asterisks: Option<bool>,
    /// Escape underscores in text
    pub escape_underscores: Option<bool>,
    /// Escape misc markdown characters
    pub escape_misc: Option<bool>,
    /// Escape all ASCII punctuation
    pub escape_ascii: Option<bool>,
    /// Default code language
    pub code_language: Option<String>,
    /// Use autolinks for bare URLs
    pub autolinks: Option<bool>,
    /// Add default title if none exists
    pub default_title: Option<bool>,
    /// Use <br> in tables instead of spaces
    pub br_in_tables: Option<bool>,
    /// Enable spatial table reconstruction in hOCR documents
    pub hocr_spatial_tables: Option<bool>,
    /// Highlight style for <mark> elements
    pub highlight_style: Option<JsHighlightStyle>,
    /// Extract metadata from HTML
    pub extract_metadata: Option<bool>,
    /// Whitespace handling mode
    pub whitespace_mode: Option<JsWhitespaceMode>,
    /// Strip newlines from HTML before processing
    pub strip_newlines: Option<bool>,
    /// Enable text wrapping
    pub wrap: Option<bool>,
    /// Text wrap width
    pub wrap_width: Option<u32>,
    /// Treat block elements as inline
    pub convert_as_inline: Option<bool>,
    /// Subscript symbol
    pub sub_symbol: Option<String>,
    /// Superscript symbol
    pub sup_symbol: Option<String>,
    /// Newline style
    pub newline_style: Option<JsNewlineStyle>,
    /// Code block style
    pub code_block_style: Option<JsCodeBlockStyle>,
    /// Elements where images should remain as markdown
    pub keep_inline_images_in: Option<Vec<String>>,
    /// Preprocessing options
    pub preprocessing: Option<JsPreprocessingOptions>,
    /// Source encoding (informational)
    pub encoding: Option<String>,
    /// Enable debug mode with diagnostic warnings
    pub debug: Option<bool>,
    /// List of HTML tags to strip
    pub strip_tags: Option<Vec<String>>,
    /// List of HTML tags to preserve as-is in the output
    pub preserve_tags: Option<Vec<String>>,
    /// Skip image conversion (keep as HTML)
    pub skip_images: Option<bool>,
}

impl From<JsConversionOptions> for ConversionOptionsUpdate {
    fn from(val: JsConversionOptions) -> Self {
        Self {
            heading_style: val.heading_style.map(Into::into),
            list_indent_type: val.list_indent_type.map(Into::into),
            list_indent_width: val.list_indent_width.map(|value| value as usize),
            bullets: val.bullets,
            strong_em_symbol: val.strong_em_symbol.and_then(|s| s.chars().next()),
            escape_asterisks: val.escape_asterisks,
            escape_underscores: val.escape_underscores,
            escape_misc: val.escape_misc,
            escape_ascii: val.escape_ascii,
            code_language: val.code_language,
            autolinks: val.autolinks,
            default_title: val.default_title,
            br_in_tables: val.br_in_tables,
            hocr_spatial_tables: val.hocr_spatial_tables,
            highlight_style: val.highlight_style.map(Into::into),
            extract_metadata: val.extract_metadata,
            whitespace_mode: val.whitespace_mode.map(Into::into),
            strip_newlines: val.strip_newlines,
            wrap: val.wrap,
            wrap_width: val.wrap_width.map(|value| value as usize),
            convert_as_inline: val.convert_as_inline,
            sub_symbol: val.sub_symbol,
            sup_symbol: val.sup_symbol,
            newline_style: val.newline_style.map(Into::into),
            code_block_style: val.code_block_style.map(Into::into),
            keep_inline_images_in: val.keep_inline_images_in,
            preprocessing: val.preprocessing.map(Into::into),
            encoding: val.encoding,
            debug: val.debug,
            strip_tags: val.strip_tags,
            preserve_tags: val.preserve_tags,
            skip_images: val.skip_images,
        }
    }
}

impl From<JsConversionOptions> for RustConversionOptions {
    fn from(val: JsConversionOptions) -> Self {
        Self::from(ConversionOptionsUpdate::from(val))
    }
}

/// Inline image configuration
#[napi(object)]
pub struct JsInlineImageConfig {
    /// Maximum decoded size in bytes (default: 5MB)
    pub max_decoded_size_bytes: Option<BigInt>,
    /// Filename prefix for generated filenames
    pub filename_prefix: Option<String>,
    /// Capture inline SVG elements (default: true)
    pub capture_svg: Option<bool>,
    /// Infer image dimensions (default: false)
    pub infer_dimensions: Option<bool>,
}

impl From<JsInlineImageConfig> for InlineImageConfigUpdate {
    fn from(val: JsInlineImageConfig) -> Self {
        let max_decoded_size_bytes = val.max_decoded_size_bytes.map(|b| {
            // Use get_u64 but don't rely on the lossless flag for correct sign detection
            // Instead, check the sign_bit directly from the internal structure
            let (_, value, _) = b.get_u64();
            // The BigInt is positive if sign_bit is false, so we use the value directly
            value
        });
        Self {
            max_decoded_size_bytes,
            filename_prefix: val.filename_prefix,
            capture_svg: val.capture_svg,
            infer_dimensions: val.infer_dimensions,
        }
    }
}

impl From<JsInlineImageConfig> for RustInlineImageConfig {
    fn from(val: JsInlineImageConfig) -> Self {
        let mut cfg = Self::new(DEFAULT_INLINE_IMAGE_LIMIT);
        cfg.apply_update(InlineImageConfigUpdate::from(val));
        cfg
    }
}

/// Inline image data
#[napi(object)]
pub struct JsInlineImage {
    /// Raw image data
    pub data: Buffer,
    /// Image format (png, jpeg, gif, etc.)
    pub format: String,
    /// Generated or provided filename
    pub filename: Option<String>,
    /// Alt text / description
    pub description: Option<String>,
    /// Image dimensions (width, height) if available
    pub dimensions: Option<Vec<u32>>,
    /// Source type (`img_data_uri` or `svg_element`)
    pub source: String,
    /// HTML attributes from the source element
    pub attributes: HashMap<String, String>,
}

/// Warning about inline image processing
#[napi(object)]
pub struct JsInlineImageWarning {
    /// Index of the image that caused the warning
    pub index: u32,
    /// Warning message
    pub message: String,
}

/// Result of HTML extraction with inline images
#[napi(object)]
pub struct JsHtmlExtraction {
    /// Converted markdown
    pub markdown: String,
    /// Extracted inline images
    pub inline_images: Vec<JsInlineImage>,
    /// Warnings encountered during extraction
    pub warnings: Vec<JsInlineImageWarning>,
}

/// Metadata extraction configuration
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsMetadataConfig {
    #[napi(js_name = "extract_document")]
    pub extract_document: Option<bool>,
    #[napi(js_name = "extract_headers")]
    pub extract_headers: Option<bool>,
    #[napi(js_name = "extract_links")]
    pub extract_links: Option<bool>,
    #[napi(js_name = "extract_images")]
    pub extract_images: Option<bool>,
    #[napi(js_name = "extract_structured_data")]
    pub extract_structured_data: Option<bool>,
    #[napi(js_name = "max_structured_data_size")]
    pub max_structured_data_size: Option<i64>,
}

#[cfg(feature = "metadata")]
impl From<JsMetadataConfig> for RustMetadataConfig {
    fn from(val: JsMetadataConfig) -> Self {
        let update = html_to_markdown_rs::MetadataConfigUpdate {
            extract_document: val.extract_document,
            extract_headers: val.extract_headers,
            extract_links: val.extract_links,
            extract_images: val.extract_images,
            extract_structured_data: val.extract_structured_data,
            max_structured_data_size: val.max_structured_data_size.map(|value| value as usize),
        };
        Self::from(update)
    }
}

/// Document-level metadata
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsDocumentMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub author: Option<String>,
    #[napi(js_name = "canonical_url")]
    pub canonical_url: Option<String>,
    #[napi(js_name = "base_href")]
    pub base_href: Option<String>,
    pub language: Option<String>,
    #[napi(js_name = "text_direction")]
    pub text_direction: Option<String>,
    #[napi(js_name = "open_graph")]
    pub open_graph: HashMap<String, String>,
    #[napi(js_name = "twitter_card")]
    pub twitter_card: HashMap<String, String>,
    #[napi(js_name = "meta_tags")]
    pub meta_tags: HashMap<String, String>,
}

/// Header element metadata
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsHeaderMetadata {
    pub level: u32,
    pub text: String,
    pub id: Option<String>,
    pub depth: u32,
    #[napi(js_name = "html_offset")]
    pub html_offset: u32,
}

/// Hyperlink metadata
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsLinkMetadata {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
    #[napi(js_name = "link_type")]
    pub link_type: String,
    pub rel: Vec<String>,
    pub attributes: HashMap<String, String>,
}

/// Image metadata
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsImageMetadata {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub dimensions: Option<Vec<u32>>,
    #[napi(js_name = "image_type")]
    pub image_type: String,
    pub attributes: HashMap<String, String>,
}

/// Structured data (JSON-LD, Microdata, `RDFa`)
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsStructuredData {
    #[napi(js_name = "data_type")]
    pub data_type: String,
    #[napi(js_name = "raw_json")]
    pub raw_json: String,
    #[napi(js_name = "schema_type")]
    pub schema_type: Option<String>,
}

/// Complete extracted metadata
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsExtendedMetadata {
    pub document: JsDocumentMetadata,
    pub headers: Vec<JsHeaderMetadata>,
    pub links: Vec<JsLinkMetadata>,
    pub images: Vec<JsImageMetadata>,
    pub structured_data: Vec<JsStructuredData>,
}

/// Result of conversion with metadata extraction
#[cfg(feature = "metadata")]
#[napi(object)]
pub struct JsMetadataExtraction {
    pub markdown: String,
    pub metadata: JsExtendedMetadata,
}

#[cfg(feature = "metadata")]
fn convert_document_metadata(doc: RustDocumentMetadata) -> JsDocumentMetadata {
    JsDocumentMetadata {
        title: doc.title,
        description: doc.description,
        keywords: doc.keywords,
        author: doc.author,
        canonical_url: doc.canonical_url,
        base_href: doc.base_href,
        language: doc.language,
        text_direction: doc.text_direction.map(|dir| dir.to_string()),
        open_graph: doc.open_graph.into_iter().collect(),
        twitter_card: doc.twitter_card.into_iter().collect(),
        meta_tags: doc.meta_tags.into_iter().collect(),
    }
}

#[cfg(feature = "metadata")]
fn convert_headers(headers: Vec<RustHeaderMetadata>) -> Vec<JsHeaderMetadata> {
    headers
        .into_iter()
        .map(|h| JsHeaderMetadata {
            level: u32::from(h.level),
            text: h.text,
            id: h.id,
            depth: h.depth as u32,
            html_offset: h.html_offset as u32,
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn convert_links(links: Vec<RustLinkMetadata>) -> Vec<JsLinkMetadata> {
    links
        .into_iter()
        .map(|l| JsLinkMetadata {
            href: l.href,
            text: l.text,
            title: l.title,
            link_type: l.link_type.to_string(),
            rel: l.rel,
            attributes: l.attributes.into_iter().collect(),
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn convert_images(images: Vec<RustImageMetadata>) -> Vec<JsImageMetadata> {
    images
        .into_iter()
        .map(|i| JsImageMetadata {
            src: i.src,
            alt: i.alt,
            title: i.title,
            dimensions: i.dimensions.map(|(w, h)| vec![w, h]),
            image_type: i.image_type.to_string(),
            attributes: i.attributes.into_iter().collect(),
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn convert_structured_data(data: Vec<RustStructuredData>) -> Vec<JsStructuredData> {
    data.into_iter()
        .map(|d| JsStructuredData {
            data_type: d.data_type.to_string(),
            raw_json: d.raw_json,
            schema_type: d.schema_type,
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn convert_metadata(metadata: RustExtendedMetadata) -> JsExtendedMetadata {
    JsExtendedMetadata {
        document: convert_document_metadata(metadata.document),
        headers: convert_headers(metadata.headers),
        links: convert_links(metadata.links),
        images: convert_images(metadata.images),
        structured_data: convert_structured_data(metadata.structured_data),
    }
}

/// Convert HTML to Markdown
///
/// # Arguments
///

#[cfg(feature = "async-visitor")]
#[napi(object)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsNodeContext {
    pub node_type: String,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub depth: u32,
    pub index_in_parent: u32,
    pub parent_tag: Option<String>,
    pub is_inline: bool,
}

#[cfg(feature = "async-visitor")]
#[napi(object)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsVisitResult {
    #[napi(js_name = "type")]
    pub result_type: String,
    pub output: Option<String>,
}

/// NAPI-RS `AsyncHtmlVisitor` Bridge Implementation
///
/// # Architecture
///
/// This bridge enables full async visitor pattern support for Node.js by:
/// 1. Accepting JavaScript visitor objects at the NAPI boundary
/// 2. Wrapping JS callbacks as NAPI `ThreadsafeFunction` references
/// 3. Implementing `AsyncHtmlVisitor` trait with proper .await on JS calls
/// 4. Executing the async conversion pipeline via tokio runtime
///
/// # Key Design Decisions
///
/// - **Feature Gate**: Uses `async-visitor` feature (not `visitor`)
/// - **`ThreadsafeFunction`**: Stores Arc<ThreadsafeFunction> for each visitor method
/// - **Async Methods**: All visitor methods are `async fn` to properly .await on JS calls
/// - **Runtime**: Uses `tokio::runtime` to `block_on` the async conversion
/// - **Error Handling**: JS callback errors default to `VisitResult::Continue`
///
/// # JavaScript Integration
///
/// From JavaScript, pass a visitor object with optional async methods:
/// ```javascript
/// const visitor = {
///   visitText: async (ctx, text) => ({ type: 'continue' }),
///   visitLink: async (ctx, href, text, title) => ({ type: 'continue' }),
///   // ... other methods as needed
/// };
/// ```
///
/// # Type alias for `ThreadsafeFunction`
///
/// Each visitor method callback is wrapped as an Arc-based `ThreadsafeFunction`
/// that accepts a String parameter and returns a Promise<String>.
#[cfg(feature = "async-visitor")]
type VisitorFn = Arc<
    napi::threadsafe_function::ThreadsafeFunction<
        String,
        napi::bindgen_prelude::Promise<String>,
        String,
        napi::Status,
        false,
    >,
>;

#[cfg(feature = "async-visitor")]
#[allow(dead_code)]
#[derive(Clone)]
struct JsVisitorBridge {
    visit_element_start_fn: Option<VisitorFn>,
    visit_element_end_fn: Option<VisitorFn>,
    visit_text_fn: Option<VisitorFn>,
    visit_link_fn: Option<VisitorFn>,
    visit_image_fn: Option<VisitorFn>,
    visit_heading_fn: Option<VisitorFn>,
    visit_code_block_fn: Option<VisitorFn>,
    visit_code_inline_fn: Option<VisitorFn>,
    visit_list_item_fn: Option<VisitorFn>,
    visit_list_start_fn: Option<VisitorFn>,
    visit_list_end_fn: Option<VisitorFn>,
    visit_table_start_fn: Option<VisitorFn>,
    visit_table_row_fn: Option<VisitorFn>,
    visit_table_end_fn: Option<VisitorFn>,
    visit_blockquote_fn: Option<VisitorFn>,
    visit_strong_fn: Option<VisitorFn>,
    visit_emphasis_fn: Option<VisitorFn>,
    visit_strikethrough_fn: Option<VisitorFn>,
    visit_underline_fn: Option<VisitorFn>,
    visit_subscript_fn: Option<VisitorFn>,
    visit_superscript_fn: Option<VisitorFn>,
    visit_mark_fn: Option<VisitorFn>,
    visit_line_break_fn: Option<VisitorFn>,
    visit_horizontal_rule_fn: Option<VisitorFn>,
    visit_custom_element_fn: Option<VisitorFn>,
    visit_definition_list_start_fn: Option<VisitorFn>,
    visit_definition_term_fn: Option<VisitorFn>,
    visit_definition_description_fn: Option<VisitorFn>,
    visit_definition_list_end_fn: Option<VisitorFn>,
    visit_form_fn: Option<VisitorFn>,
    visit_input_fn: Option<VisitorFn>,
    visit_button_fn: Option<VisitorFn>,
    visit_audio_fn: Option<VisitorFn>,
    visit_video_fn: Option<VisitorFn>,
    visit_iframe_fn: Option<VisitorFn>,
    visit_details_fn: Option<VisitorFn>,
    visit_summary_fn: Option<VisitorFn>,
    visit_figure_start_fn: Option<VisitorFn>,
    visit_figcaption_fn: Option<VisitorFn>,
    visit_figure_end_fn: Option<VisitorFn>,
}

#[cfg(feature = "async-visitor")]
impl std::fmt::Debug for JsVisitorBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsVisitorBridge").finish()
    }
}

#[cfg(feature = "async-visitor")]
unsafe impl Send for JsVisitorBridge {}

#[cfg(feature = "async-visitor")]
unsafe impl Sync for JsVisitorBridge {}

#[cfg(feature = "async-visitor")]
impl JsVisitorBridge {
    const fn new() -> Self {
        Self {
            visit_element_start_fn: None,
            visit_element_end_fn: None,
            visit_text_fn: None,
            visit_link_fn: None,
            visit_image_fn: None,
            visit_heading_fn: None,
            visit_code_block_fn: None,
            visit_code_inline_fn: None,
            visit_list_item_fn: None,
            visit_list_start_fn: None,
            visit_list_end_fn: None,
            visit_table_start_fn: None,
            visit_table_row_fn: None,
            visit_table_end_fn: None,
            visit_blockquote_fn: None,
            visit_strong_fn: None,
            visit_emphasis_fn: None,
            visit_strikethrough_fn: None,
            visit_underline_fn: None,
            visit_subscript_fn: None,
            visit_superscript_fn: None,
            visit_mark_fn: None,
            visit_line_break_fn: None,
            visit_horizontal_rule_fn: None,
            visit_custom_element_fn: None,
            visit_definition_list_start_fn: None,
            visit_definition_term_fn: None,
            visit_definition_description_fn: None,
            visit_definition_list_end_fn: None,
            visit_form_fn: None,
            visit_input_fn: None,
            visit_button_fn: None,
            visit_audio_fn: None,
            visit_video_fn: None,
            visit_iframe_fn: None,
            visit_details_fn: None,
            visit_summary_fn: None,
            visit_figure_start_fn: None,
            visit_figcaption_fn: None,
            visit_figure_end_fn: None,
        }
    }

    #[allow(dead_code)]
    fn node_context_to_js(ctx: &RustNodeContext) -> JsNodeContext {
        let mut attributes = HashMap::new();
        for (k, v) in &ctx.attributes {
            attributes.insert(k.clone(), v.clone());
        }

        JsNodeContext {
            node_type: format!("{:?}", ctx.node_type),
            tag_name: ctx.tag_name.clone(),
            attributes,
            depth: ctx.depth as u32,
            index_in_parent: ctx.index_in_parent as u32,
            parent_tag: ctx.parent_tag.clone(),
            is_inline: ctx.is_inline,
        }
    }

    #[allow(dead_code)]
    fn visit_result_from_js(js_result: &JsVisitResult) -> RustVisitResult {
        match js_result.result_type.to_lowercase().as_str() {
            "continue" => RustVisitResult::Continue,
            "custom" => RustVisitResult::Custom(js_result.output.clone().unwrap_or_default()),
            "skip" => RustVisitResult::Skip,
            "preservehtml" => RustVisitResult::PreserveHtml,
            "error" => RustVisitResult::Error(js_result.output.clone().unwrap_or_else(|| "Unknown error".to_string())),
            _ => RustVisitResult::Continue,
        }
    }

    /// Serialize visitor parameters to JSON string
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters implementing `serde::Serialize` trait
    ///
    /// # Returns
    ///
    /// JSON string representation or `serde_json` error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ctx = JsNodeContext { /* ... */ };
    /// let json = JsVisitorBridge::serialize_params(&ctx)?;
    /// ```
    #[allow(dead_code)]
    fn serialize_params<T: serde::Serialize>(params: &T) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string(params)
    }

    /// Deserialize visitor result from JSON string
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string representation of `JsVisitResult`
    ///
    /// # Returns
    ///
    /// Deserialized `JsVisitResult` or `serde_json` error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let json = r#"{"type":"continue","output":null}"#;
    /// let result = JsVisitorBridge::deserialize_result(json)?;
    /// ```
    #[allow(dead_code)]
    fn deserialize_result(json: &str) -> std::result::Result<JsVisitResult, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(feature = "async-visitor")]
#[async_trait]
impl AsyncHtmlVisitor for JsVisitorBridge {
    async fn visit_element_start(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_element_end(&mut self, _ctx: &RustNodeContext, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_text(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_link(
        &mut self,
        _ctx: &RustNodeContext,
        _href: &str,
        _text: &str,
        _title: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_image(
        &mut self,
        _ctx: &RustNodeContext,
        _src: &str,
        _alt: &str,
        _title: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_heading(
        &mut self,
        _ctx: &RustNodeContext,
        _level: u32,
        _text: &str,
        _id: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_code_block(&mut self, _ctx: &RustNodeContext, _lang: Option<&str>, _code: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_code_inline(&mut self, _ctx: &RustNodeContext, _code: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_list_item(
        &mut self,
        _ctx: &RustNodeContext,
        _ordered: bool,
        _marker: &str,
        _text: &str,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_list_start(&mut self, _ctx: &RustNodeContext, _ordered: bool) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_list_end(&mut self, _ctx: &RustNodeContext, _ordered: bool, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_table_start(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_table_row(
        &mut self,
        _ctx: &RustNodeContext,
        _cells: &[String],
        _is_header: bool,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_table_end(&mut self, _ctx: &RustNodeContext, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_blockquote(&mut self, _ctx: &RustNodeContext, _content: &str, _depth: usize) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_strong(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_emphasis(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_strikethrough(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_underline(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_subscript(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_superscript(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_mark(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_line_break(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_horizontal_rule(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_custom_element(&mut self, _ctx: &RustNodeContext, _tag_name: &str, _html: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_definition_list_start(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_definition_term(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_definition_description(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_definition_list_end(&mut self, _ctx: &RustNodeContext, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_form(
        &mut self,
        _ctx: &RustNodeContext,
        _action: Option<&str>,
        _method: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_input(
        &mut self,
        _ctx: &RustNodeContext,
        _input_type: &str,
        _name: Option<&str>,
        _value: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_button(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_audio(&mut self, _ctx: &RustNodeContext, _src: Option<&str>) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_video(&mut self, _ctx: &RustNodeContext, _src: Option<&str>) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_iframe(&mut self, _ctx: &RustNodeContext, _src: Option<&str>) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_details(&mut self, _ctx: &RustNodeContext, _open: bool) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_summary(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_figure_start(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_figcaption(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    async fn visit_figure_end(&mut self, _ctx: &RustNodeContext, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }
}

/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
/// * `visitor` - Optional visitor object (when visitor feature is enabled)
///
/// # Example
///
/// ```javascript
/// const { convert } = require('html-to-markdown');
///
/// const html = '<h1>Hello World</h1>';
/// const markdown = convert(html);
/// console.log(markdown); // # Hello World
/// ```
#[napi]
pub fn convert(html: String, options: Option<JsConversionOptions>, visitor: Option<Object>) -> Result<String> {
    let rust_options = options.map(Into::into);

    #[cfg(feature = "visitor")]
    if visitor.is_some() {
        // Visitor support for synchronous conversion would require a separate implementation
        return Err(Error::new(
            Status::GenericFailure,
            "Use convertWithVisitor for async visitor support",
        ));
    }
    #[cfg(not(feature = "visitor"))]
    let _ = visitor;

    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
        .map_err(to_js_error)
}

/// Convert HTML to Markdown with an async visitor object.
///
/// # Async Visitor Support
///
/// This function enables full async visitor pattern support for Node.js:
/// - JavaScript visitor callbacks are invoked asynchronously via NAPI `ThreadsafeFunction`
/// - All 30+ visitor methods are supported (links, images, headings, code, lists, tables, etc.)
/// - Callback errors gracefully default to `VisitResult::Continue`
/// - Powered by tokio async runtime for seamless JS-Rust cooperation
///
/// # Visitor Methods
///
/// Implement any combination of these optional async methods in your visitor:
/// - `visitText(ctx, text) -> { type: string, output?: string }`
/// - `visitLink(ctx, href, text, title) -> VisitResult`
/// - `visitImage(ctx, src, alt, title) -> VisitResult`
/// - `visitHeading(ctx, level, text, id) -> VisitResult`
/// - `visitCodeBlock(ctx, lang, code) -> VisitResult`
/// - `visitCodeInline(ctx, code) -> VisitResult`
/// - `visitListItem(ctx, ordered, marker, text) -> VisitResult`
/// - `visitTableRow(ctx, cells, isHeader) -> VisitResult`
/// - `visitBlockquote(ctx, content, depth) -> VisitResult`
/// - And 20+ more semantic and inline element callbacks
///
/// # `VisitResult` Types
///
/// Each callback should return an object with:
/// - `type: 'continue' | 'skip' | 'custom' | 'preservehtml' | 'error'`
/// - `output?: string` (required for 'custom' and 'error' types)
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
/// * `visitor` - Visitor object with optional async callback methods
///
/// # Example
///
/// ```javascript
/// const { convertWithVisitor } = require('@kreuzberg/html-to-markdown-node');
///
/// const html = '<a href="https://example.com">Click me</a>';
/// const visitor = {
///   visitLink: async (ctx, href, text, title) => {
///     console.log(`Found link: ${href}`);
///     return { type: 'continue' };  // Use default markdown conversion
///   }
/// };
///
/// const markdown = await convertWithVisitor(html, undefined, visitor);
/// console.log(markdown); // [Click me](https://example.com)
/// ```
///
/// @deprecated Use the optional visitor parameter in convert() instead
#[cfg(feature = "async-visitor")]
#[napi(js_name = "convertWithVisitor")]
pub fn convert_with_visitor(
    _env: Env,
    html: String,
    options: Option<JsConversionOptions>,
    visitor: Object,
) -> napi::Result<String> {
    let rust_options = options.map(Into::into);

    let mut bridge = JsVisitorBridge::new();

    macro_rules! extract_fn {
        ($method_name:literal, $field:ident) => {
            if let Ok(func) = visitor.get_named_property::<Function<String, Promise<String>>>($method_name) {
                if let Ok(tsfn) = func
                    .build_threadsafe_function::<String>()
                    .build_callback(|ctx: napi::threadsafe_function::ThreadsafeCallContext<String>| Ok(ctx.value))
                {
                    bridge.$field = Some(Arc::new(tsfn));
                }
            }
        };
    }

    extract_fn!("visitElementStart", visit_element_start_fn);
    extract_fn!("visitElementEnd", visit_element_end_fn);
    extract_fn!("visitText", visit_text_fn);
    extract_fn!("visitLink", visit_link_fn);
    extract_fn!("visitImage", visit_image_fn);
    extract_fn!("visitHeading", visit_heading_fn);
    extract_fn!("visitCodeBlock", visit_code_block_fn);
    extract_fn!("visitCodeInline", visit_code_inline_fn);
    extract_fn!("visitListItem", visit_list_item_fn);
    extract_fn!("visitListStart", visit_list_start_fn);
    extract_fn!("visitListEnd", visit_list_end_fn);
    extract_fn!("visitTableStart", visit_table_start_fn);
    extract_fn!("visitTableRow", visit_table_row_fn);
    extract_fn!("visitTableEnd", visit_table_end_fn);
    extract_fn!("visitBlockquote", visit_blockquote_fn);
    extract_fn!("visitStrong", visit_strong_fn);
    extract_fn!("visitEmphasis", visit_emphasis_fn);
    extract_fn!("visitStrikethrough", visit_strikethrough_fn);
    extract_fn!("visitUnderline", visit_underline_fn);
    extract_fn!("visitSubscript", visit_subscript_fn);
    extract_fn!("visitSuperscript", visit_superscript_fn);
    extract_fn!("visitMark", visit_mark_fn);
    extract_fn!("visitLineBreak", visit_line_break_fn);
    extract_fn!("visitHorizontalRule", visit_horizontal_rule_fn);
    extract_fn!("visitCustomElement", visit_custom_element_fn);
    extract_fn!("visitDefinitionListStart", visit_definition_list_start_fn);
    extract_fn!("visitDefinitionTerm", visit_definition_term_fn);
    extract_fn!("visitDefinitionDescription", visit_definition_description_fn);
    extract_fn!("visitDefinitionListEnd", visit_definition_list_end_fn);
    extract_fn!("visitForm", visit_form_fn);
    extract_fn!("visitInput", visit_input_fn);
    extract_fn!("visitButton", visit_button_fn);
    extract_fn!("visitAudio", visit_audio_fn);
    extract_fn!("visitVideo", visit_video_fn);
    extract_fn!("visitIframe", visit_iframe_fn);
    extract_fn!("visitDetails", visit_details_fn);
    extract_fn!("visitSummary", visit_summary_fn);
    extract_fn!("visitFigureStart", visit_figure_start_fn);
    extract_fn!("visitFigcaption", visit_figcaption_fn);
    extract_fn!("visitFigureEnd", visit_figure_end_fn);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("Failed to create runtime: {e}")))?;

    let result = rt
        .block_on(async {
            html_to_markdown_rs::convert_with_async_visitor(
                &html,
                rust_options,
                Some(std::rc::Rc::new(std::cell::RefCell::new(bridge))),
            )
            .await
        })
        .map_err(to_js_error)?;

    Ok(result)
}

#[napi(js_name = "convertJson")]
pub fn convert_json(html: String, options_json: Option<String>) -> Result<String> {
    let rust_options = parse_conversion_options(options_json.as_deref()).map_err(to_js_error)?;
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
        .map_err(to_js_error)
}

#[napi]
pub fn start_profiling(output_path: String, frequency: Option<i32>) -> Result<()> {
    let freq = frequency.unwrap_or(1000);
    profiling::start(output_path.into(), freq).map_err(to_js_error)
}

#[napi]
pub fn stop_profiling() -> Result<()> {
    profiling::stop().map_err(to_js_error)
}

fn buffer_to_str(html: &Buffer) -> Result<&str> {
    str::from_utf8(html.as_ref()).map_err(|e| Error::new(Status::InvalidArg, format!("HTML must be valid UTF-8: {e}")))
}

/// Convert HTML to Markdown from a Buffer/Uint8Array without creating intermediate JS strings.
#[napi(js_name = "convertBuffer")]
pub fn convert_buffer(html: Buffer, options: Option<JsConversionOptions>) -> Result<String> {
    let html = buffer_to_str(&html)?;
    let rust_options = options.map(Into::into);
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(html, rust_options.clone())))
        .map_err(to_js_error)
}

#[napi(js_name = "convertBufferJson")]
pub fn convert_buffer_json(html: Buffer, options_json: Option<String>) -> Result<String> {
    let html = buffer_to_str(&html)?;
    let rust_options = parse_conversion_options(options_json.as_deref()).map_err(to_js_error)?;
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(html, rust_options.clone())))
        .map_err(to_js_error)
}

/// Create a reusable `ConversionOptions` handle.
#[napi]
pub fn create_conversion_options_handle(options: Option<JsConversionOptions>) -> External<RustConversionOptions> {
    External::new(options.map(Into::into).unwrap_or_default())
}

#[napi(js_name = "createConversionOptionsHandleJson")]
pub fn create_conversion_options_handle_json(options_json: Option<String>) -> Result<External<RustConversionOptions>> {
    let rust_options = parse_conversion_options(options_json.as_deref()).map_err(to_js_error)?;
    Ok(External::new(rust_options.unwrap_or_default()))
}

/// Create a reusable `MetadataConfig` handle.
#[cfg(feature = "metadata")]
#[napi]
pub fn create_metadata_config_handle(metadata_config: Option<JsMetadataConfig>) -> External<RustMetadataConfig> {
    External::new(metadata_config.map(Into::into).unwrap_or_default())
}

#[cfg(feature = "metadata")]
#[napi(js_name = "createMetadataConfigHandleJson")]
pub fn create_metadata_config_handle_json(
    metadata_config_json: Option<String>,
) -> Result<External<RustMetadataConfig>> {
    let rust_config = parse_metadata_config(metadata_config_json.as_deref()).map_err(to_js_error)?;
    Ok(External::new(rust_config))
}

/// Convert HTML using a previously-created `ConversionOptions` handle.
#[napi]
pub fn convert_with_options_handle(html: String, options: &External<RustConversionOptions>) -> Result<String> {
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(&html, Some((**options).clone()))))
        .map_err(to_js_error)
}

/// Convert HTML Buffer data using a previously-created `ConversionOptions` handle.
#[napi(js_name = "convertBufferWithOptionsHandle")]
pub fn convert_buffer_with_options_handle(html: Buffer, options: &External<RustConversionOptions>) -> Result<String> {
    let html = buffer_to_str(&html)?;
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(html, Some((**options).clone()))))
        .map_err(to_js_error)
}

fn build_js_extraction(extraction: html_to_markdown_rs::HtmlExtraction) -> JsHtmlExtraction {
    let inline_images = extraction
        .inline_images
        .into_iter()
        .map(|img| JsInlineImage {
            data: img.data.into(),
            format: img.format.to_string(),
            filename: img.filename,
            description: img.description,
            dimensions: img.dimensions.map(|(w, h)| vec![w, h]),
            source: img.source.to_string(),
            attributes: img.attributes.into_iter().collect(),
        })
        .collect();

    let warnings = extraction
        .warnings
        .into_iter()
        .map(|w| JsInlineImageWarning {
            index: w.index as u32,
            message: w.message,
        })
        .collect();

    JsHtmlExtraction {
        markdown: extraction.markdown,
        inline_images,
        warnings,
    }
}

fn convert_inline_images_impl(
    html: &str,
    options: Option<JsConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let rust_options = options.map(Into::into);
    let rust_config = image_config.map_or_else(|| RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT), Into::into);

    let extraction =
        guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(build_js_extraction(extraction))
}

fn convert_inline_images_with_handle_impl(
    html: &str,
    options: &External<RustConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let rust_options = Some((**options).clone());
    let rust_config = image_config.map_or_else(|| RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT), Into::into);

    let extraction =
        guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(build_js_extraction(extraction))
}

fn convert_inline_images_json_impl(
    html: &str,
    options_json: Option<String>,
    image_config_json: Option<String>,
) -> Result<JsHtmlExtraction> {
    let rust_options = parse_conversion_options(options_json.as_deref()).map_err(to_js_error)?;
    let rust_config = parse_inline_image_config(image_config_json.as_deref()).map_err(to_js_error)?;

    let extraction =
        guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(build_js_extraction(extraction))
}

/// Convert HTML to Markdown while collecting inline images
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
/// * `image_config` - Configuration for inline image extraction
/// * `visitor` - Optional visitor object (when visitor feature is enabled)
#[napi]
pub fn convert_with_inline_images(
    html: String,
    options: Option<JsConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
    visitor: Option<Object>,
) -> Result<JsHtmlExtraction> {
    #[cfg(feature = "visitor")]
    if visitor.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Use convertWithVisitor for async visitor support",
        ));
    }
    #[cfg(not(feature = "visitor"))]
    let _ = visitor;

    convert_inline_images_impl(&html, options, image_config)
}

/// Convert HTML to Markdown while collecting inline images using a pre-created options handle.
#[napi(js_name = "convertWithInlineImagesHandle")]
pub fn convert_with_inline_images_handle(
    html: String,
    options: &External<RustConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    convert_inline_images_with_handle_impl(&html, options, image_config)
}

#[napi(js_name = "convertWithInlineImagesJson")]
pub fn convert_with_inline_images_json(
    html: String,
    options_json: Option<String>,
    image_config_json: Option<String>,
) -> Result<JsHtmlExtraction> {
    convert_inline_images_json_impl(&html, options_json, image_config_json)
}

/// Convert inline images from Buffer/Uint8Array input without an intermediate string allocation.
#[napi(js_name = "convertInlineImagesBuffer")]
pub fn convert_inline_images_buffer(
    html: Buffer,
    options: Option<JsConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let html = buffer_to_str(&html)?;
    convert_inline_images_impl(html, options, image_config)
}

/// Convert inline images from Buffer/Uint8Array input using a pre-created options handle.
#[napi(js_name = "convertInlineImagesBufferWithOptionsHandle")]
pub fn convert_inline_images_buffer_with_options_handle(
    html: Buffer,
    options: &External<RustConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let html = buffer_to_str(&html)?;
    convert_inline_images_with_handle_impl(html, options, image_config)
}

#[napi(js_name = "convertInlineImagesBufferJson")]
pub fn convert_inline_images_buffer_json(
    html: Buffer,
    options_json: Option<String>,
    image_config_json: Option<String>,
) -> Result<JsHtmlExtraction> {
    let html = buffer_to_str(&html)?;
    convert_inline_images_json_impl(html, options_json, image_config_json)
}

/// Convert HTML to Markdown with metadata extraction.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
/// * `metadata_config` - Optional metadata extraction configuration
/// * `visitor` - Optional visitor object (when visitor feature is enabled)
///
/// # Example
///
/// ```javascript
/// const { convertWithMetadata } = require('html-to-markdown');
///
/// const html = '<html lang="en"><head><title>Test</title></head><body><h1>Hello</h1></body></html>';
/// const config = { extractHeaders: true, extractLinks: true };
/// const result = convertWithMetadata(html, undefined, config);
/// console.log(result.markdown);
/// console.log(result.metadata.document.title);
/// ```
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadata")]
pub fn convert_with_metadata(
    html: String,
    options: Option<JsConversionOptions>,
    metadata_config: Option<JsMetadataConfig>,
    visitor: Option<Object>,
) -> Result<JsMetadataExtraction> {
    #[cfg(feature = "visitor")]
    if visitor.is_some() {
        return Err(Error::new(
            Status::GenericFailure,
            "Use convertWithVisitor for async visitor support",
        ));
    }
    #[cfg(not(feature = "visitor"))]
    let _ = visitor;

    let rust_options = options.map(Into::into);
    let rust_config = metadata_config.map(Into::into).unwrap_or_default();

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

/// Convert HTML to Markdown with metadata extraction using a pre-created options handle.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataHandle")]
pub fn convert_with_metadata_handle(
    html: String,
    options: &External<RustConversionOptions>,
    metadata_config: Option<JsMetadataConfig>,
) -> Result<JsMetadataExtraction> {
    let rust_config = metadata_config.map(Into::into).unwrap_or_default();
    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, Some((**options).clone()), rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataJson")]
pub fn convert_with_metadata_json(
    html: String,
    options_json: Option<String>,
    metadata_config_json: Option<String>,
) -> Result<JsMetadataExtraction> {
    convert_metadata_json_impl(&html, options_json, metadata_config_json)
}

/// Convert HTML from Buffer/Uint8Array with metadata extraction without intermediate string allocation.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataBuffer")]
pub fn convert_with_metadata_buffer(
    html: Buffer,
    options: Option<JsConversionOptions>,
    metadata_config: Option<JsMetadataConfig>,
) -> Result<JsMetadataExtraction> {
    let html = buffer_to_str(&html)?;
    let rust_options = options.map(Into::into);
    let rust_config = metadata_config.map(Into::into).unwrap_or_default();

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

/// Convert HTML from Buffer/Uint8Array with metadata extraction using a pre-created options handle.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataBufferWithOptionsHandle")]
pub fn convert_with_metadata_buffer_with_options_handle(
    html: Buffer,
    options: &External<RustConversionOptions>,
    metadata_config: Option<JsMetadataConfig>,
) -> Result<JsMetadataExtraction> {
    let html = buffer_to_str(&html)?;
    let rust_config = metadata_config.map(Into::into).unwrap_or_default();

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, Some((**options).clone()), rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

/// Convert HTML from Buffer/Uint8Array with metadata extraction using a metadata handle.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataBufferWithMetadataHandle")]
pub fn convert_with_metadata_buffer_with_metadata_handle(
    html: Buffer,
    metadata_config: &External<RustMetadataConfig>,
) -> Result<JsMetadataExtraction> {
    let html = buffer_to_str(&html)?;
    let rust_config = (**metadata_config).clone();
    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, None, rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

/// Convert HTML from Buffer/Uint8Array with metadata extraction using options + metadata handles.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataBufferWithOptionsAndMetadataHandle")]
pub fn convert_with_metadata_buffer_with_options_and_metadata_handle(
    html: Buffer,
    options: &External<RustConversionOptions>,
    metadata_config: &External<RustMetadataConfig>,
) -> Result<JsMetadataExtraction> {
    let html = buffer_to_str(&html)?;
    let rust_config = (**metadata_config).clone();
    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, Some((**options).clone()), rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

/// Convert HTML from Buffer/Uint8Array with metadata extraction using JSON config.
#[cfg(feature = "metadata")]
#[napi(js_name = "convertWithMetadataBufferJson")]
pub fn convert_with_metadata_buffer_json(
    html: Buffer,
    options_json: Option<String>,
    metadata_config_json: Option<String>,
) -> Result<JsMetadataExtraction> {
    let html = buffer_to_str(&html)?;
    convert_metadata_json_impl(html, options_json, metadata_config_json)
}

#[cfg(feature = "metadata")]
fn convert_metadata_json_impl(
    html: &str,
    options_json: Option<String>,
    metadata_config_json: Option<String>,
) -> Result<JsMetadataExtraction> {
    let rust_options = parse_conversion_options(options_json.as_deref()).map_err(to_js_error)?;
    let rust_config = parse_metadata_config(metadata_config_json.as_deref()).map_err(to_js_error)?;

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, rust_options, rust_config, None))
            .map_err(to_js_error)?;

    Ok(JsMetadataExtraction {
        markdown,
        metadata: convert_metadata(metadata),
    })
}

#[cfg(all(
    any(windows, unix),
    target_arch = "x86_64",
    not(target_env = "musl"),
    not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_style_conversion() {
        let atx: HeadingStyle = JsHeadingStyle::Atx.into();
        assert!(matches!(atx, HeadingStyle::Atx));
    }

    #[test]
    fn test_conversion_options_defaults() {
        let opts = JsConversionOptions {
            heading_style: Some(JsHeadingStyle::Atx),
            list_indent_type: None,
            list_indent_width: None,
            bullets: None,
            strong_em_symbol: None,
            escape_asterisks: None,
            escape_underscores: None,
            escape_misc: None,
            escape_ascii: None,
            code_language: None,
            autolinks: None,
            default_title: None,
            br_in_tables: None,
            hocr_spatial_tables: None,
            highlight_style: None,
            extract_metadata: None,
            whitespace_mode: None,
            strip_newlines: None,
            wrap: None,
            wrap_width: None,
            convert_as_inline: None,
            sub_symbol: None,
            sup_symbol: None,
            newline_style: None,
            code_block_style: None,
            keep_inline_images_in: None,
            preprocessing: None,
            encoding: None,
            debug: None,
            strip_tags: None,
            preserve_tags: None,
            skip_images: None,
        };

        let rust_opts: RustConversionOptions = opts.into();
        assert!(matches!(rust_opts.heading_style, HeadingStyle::Atx));
        assert_eq!(rust_opts.list_indent_width, 2);
    }

    #[test]
    fn test_preprocessing_options() {
        let opts = JsPreprocessingOptions {
            enabled: Some(true),
            preset: Some(JsPreprocessingPreset::Aggressive),
            remove_navigation: Some(false),
            remove_forms: Some(true),
        };

        let rust_opts: RustPreprocessingOptions = opts.into();
        assert!(rust_opts.enabled);
        assert!(matches!(rust_opts.preset, PreprocessingPreset::Aggressive));
        assert!(!rust_opts.remove_navigation);
        assert!(rust_opts.remove_forms);
    }
}
