#![deny(clippy::all)]

#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DocumentMetadata as RustDocumentMetadata, ExtendedMetadata as RustExtendedMetadata,
    HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata, LinkMetadata as RustLinkMetadata,
    MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData,
};
use html_to_markdown_rs::safety::guard_panic;
mod profiling;
#[cfg(feature = "visitor")]
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext as RustNodeContext, VisitResult as RustVisitResult};
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions as RustConversionOptions, ConversionOptionsUpdate,
    DEFAULT_INLINE_IMAGE_LIMIT, HeadingStyle, HighlightStyle, InlineImageConfig as RustInlineImageConfig,
    InlineImageConfigUpdate, ListIndentType, NewlineStyle, PreprocessingOptions as RustPreprocessingOptions,
    PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{collections::HashMap, str};

fn to_js_error(err: ConversionError) -> Error {
    let message = match &err {
        ConversionError::Panic(msg) => format!("html-to-markdown panic during conversion: {msg}"),
        other => other.to_string(),
    };

    Error::new(Status::GenericFailure, message)
}

fn parse_options_json(options_json: Option<String>) -> Result<Option<RustConversionOptions>> {
    let Some(json) = options_json else {
        return Ok(None);
    };

    if json.trim().is_empty() {
        return Ok(None);
    }

    let options = html_to_markdown_rs::conversion_options_from_json(&json).map_err(to_js_error)?;
    Ok(Some(options))
}

fn parse_inline_image_config_json(config_json: Option<String>) -> Result<RustInlineImageConfig> {
    let Some(json) = config_json else {
        return Ok(RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    };

    if json.trim().is_empty() {
        return Ok(RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    }

    html_to_markdown_rs::inline_image_config_from_json(&json).map_err(to_js_error)
}

#[cfg(feature = "metadata")]
fn parse_metadata_config_json(config_json: Option<String>) -> Result<RustMetadataConfig> {
    let Some(json) = config_json else {
        return Ok(RustMetadataConfig::default());
    };

    if json.trim().is_empty() {
        return Ok(RustMetadataConfig::default());
    }

    html_to_markdown_rs::metadata_config_from_json(&json).map_err(to_js_error)
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
            JsHeadingStyle::Underlined => HeadingStyle::Underlined,
            JsHeadingStyle::Atx => HeadingStyle::Atx,
            JsHeadingStyle::AtxClosed => HeadingStyle::AtxClosed,
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
            JsListIndentType::Spaces => ListIndentType::Spaces,
            JsListIndentType::Tabs => ListIndentType::Tabs,
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
            JsWhitespaceMode::Normalized => WhitespaceMode::Normalized,
            JsWhitespaceMode::Strict => WhitespaceMode::Strict,
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
            JsNewlineStyle::Spaces => NewlineStyle::Spaces,
            JsNewlineStyle::Backslash => NewlineStyle::Backslash,
        }
    }
}

/// Code block style
#[napi(string_enum)]
pub enum JsCodeBlockStyle {
    /// Indented code blocks (4 spaces) - CommonMark default
    Indented,
    /// Fenced code blocks with backticks (```)
    Backticks,
    /// Fenced code blocks with tildes (~~~)
    Tildes,
}

impl From<JsCodeBlockStyle> for CodeBlockStyle {
    fn from(val: JsCodeBlockStyle) -> Self {
        match val {
            JsCodeBlockStyle::Indented => CodeBlockStyle::Indented,
            JsCodeBlockStyle::Backticks => CodeBlockStyle::Backticks,
            JsCodeBlockStyle::Tildes => CodeBlockStyle::Tildes,
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
            JsHighlightStyle::DoubleEqual => HighlightStyle::DoubleEqual,
            JsHighlightStyle::Html => HighlightStyle::Html,
            JsHighlightStyle::Bold => HighlightStyle::Bold,
            JsHighlightStyle::None => HighlightStyle::None,
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
            JsPreprocessingPreset::Minimal => PreprocessingPreset::Minimal,
            JsPreprocessingPreset::Standard => PreprocessingPreset::Standard,
            JsPreprocessingPreset::Aggressive => PreprocessingPreset::Aggressive,
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
        let mut opts = RustPreprocessingOptions::default();
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
        }
    }
}

impl From<JsConversionOptions> for RustConversionOptions {
    fn from(val: JsConversionOptions) -> Self {
        RustConversionOptions::from(ConversionOptionsUpdate::from(val))
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
        Self {
            max_decoded_size_bytes: val.max_decoded_size_bytes.map(|b| {
                let (lossless, value, _negative) = b.get_u64();
                if !lossless {
                    // Value doesn't fit in u64, use maximum u64 value as safe fallback
                    u64::MAX
                } else {
                    value
                }
            }),
            filename_prefix: val.filename_prefix,
            capture_svg: val.capture_svg,
            infer_dimensions: val.infer_dimensions,
        }
    }
}

impl From<JsInlineImageConfig> for RustInlineImageConfig {
    fn from(val: JsInlineImageConfig) -> Self {
        let mut cfg = RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT);
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
    /// Source type (img_data_uri or svg_element)
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
        RustMetadataConfig::from(update)
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

/// Structured data (JSON-LD, Microdata, RDFa)
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
            level: h.level as u32,
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
// ============================================================================
// Visitor Pattern Support
// ============================================================================

#[cfg(feature = "visitor")]
#[napi(object)]
pub struct JsNodeContext {
    pub node_type: String,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub depth: u32,
    pub index_in_parent: u32,
    pub parent_tag: Option<String>,
    pub is_inline: bool,
}

#[cfg(feature = "visitor")]
#[napi(object)]
pub struct JsVisitResult {
    #[napi(js_name = "type")]
    pub result_type: String,
    pub output: Option<String>,
}

/// NAPI-RS Visitor Bridge Implementation
///
/// # Architecture Notes
///
/// The sync/async mismatch between the Rust `HtmlVisitor` trait (which is inherently
/// synchronous, returning `VisitResult` not `Future<VisitResult>`) and JavaScript's
/// callback model creates a fundamental challenge:
///
/// 1. **Rust Core (Synchronous)**: The HtmlVisitor trait requires synchronous visitor
///    methods that return `VisitResult` immediately. This is essential for the HTML
///    parsing loop in the Rust core.
///
/// 2. **JS Callbacks**: Calling JS functions through NAPI requires access to the V8
///    `Env` (JavaScript environment), which is only available in the native function
///    context. Within a sync visitor method, we don't have this context.
///
/// 3. **NAPI-RS Limitations**: ThreadsafeFunction would require async/await, but we're
///    in a synchronous context. There's no built-in way to block on a V8 call from
///    within native code during conversion.
///
/// # Current Implementation Strategy
///
/// Rather than an incomplete implementation, we provide a documented placeholder that:
/// - Accepts visitor objects at the JavaScript boundary
/// - Validates that callback methods exist
/// - Reserves the capability for future enhancement when NAPI-RS or V8 provides
///   better blocking mechanisms
///
/// # Recommended Alternatives for Users
///
/// For visitor pattern support in Node.js, consider:
/// 1. Use the Python binding (supports full async visitor via asyncio)
/// 2. Use the Ruby binding (supports full visitor pattern)
/// 3. Implement visitor logic in JavaScript after conversion instead of during
/// 4. Process the conversion result through post-conversion filtering/transformation
///
#[cfg(feature = "visitor")]
#[derive(Debug, Clone)]
struct JsVisitorBridge {
    // Placeholder to indicate visitor was initialized with callbacks
    // Full callback invocation requires architectural changes
    #[allow(dead_code)]
    has_callbacks: bool,
}

#[cfg(feature = "visitor")]
unsafe impl Send for JsVisitorBridge {}

#[cfg(feature = "visitor")]
impl std::panic::RefUnwindSafe for JsVisitorBridge {}

#[cfg(feature = "visitor")]
impl JsVisitorBridge {
    fn new(_env: Env, visitor_obj: Option<Object>) -> Self {
        let has_callbacks = if let Some(obj) = visitor_obj {
            // Just check if the object exists and has some callback properties
            // Full implementation would require passing Env through visitor context
            let callback_names = [
                "visitElementStart",
                "visitElementEnd",
                "visitText",
                "visitLink",
                "visitImage",
                "visitHeading",
                "visitCodeBlock",
                "visitCodeInline",
            ];

            callback_names
                .iter()
                .any(|name| obj.get_named_property::<Object>(name).is_ok())
        } else {
            false
        };

        JsVisitorBridge { has_callbacks }
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
}

#[cfg(feature = "visitor")]
impl HtmlVisitor for JsVisitorBridge {
    fn visit_element_start(&mut self, _ctx: &RustNodeContext) -> RustVisitResult {
        // Visitor callbacks are not yet implemented in Node.js binding.
        // This is due to the architectural mismatch between:
        // - Synchronous Rust visitor trait (returns VisitResult)
        // - NAPI-RS which requires V8 Env context to call JS functions
        // - Sync/async boundary that can't be crossed from sync context
        //
        // See VISITOR_IMPLEMENTATION_NOTES above for alternatives.
        RustVisitResult::Continue
    }

    fn visit_element_end(&mut self, _ctx: &RustNodeContext, _output: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_text(&mut self, _ctx: &RustNodeContext, _text: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_link(
        &mut self,
        _ctx: &RustNodeContext,
        _href: &str,
        _text: &str,
        _title: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_image(&mut self, _ctx: &RustNodeContext, _src: &str, _alt: &str, _title: Option<&str>) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_heading(
        &mut self,
        _ctx: &RustNodeContext,
        _level: u32,
        _text: &str,
        _id: Option<&str>,
    ) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_code_block(&mut self, _ctx: &RustNodeContext, _lang: Option<&str>, _code: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }

    fn visit_code_inline(&mut self, _ctx: &RustNodeContext, _code: &str) -> RustVisitResult {
        RustVisitResult::Continue
    }
}

/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
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
pub fn convert(html: String, options: Option<JsConversionOptions>) -> Result<String> {
    let rust_options = options.map(Into::into);
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
        .map_err(to_js_error)
}

/// Convert HTML to Markdown with a custom visitor object (experimental/limited).
///
/// # Status
///
/// **IMPORTANT**: Visitor callbacks are not currently implemented in the Node.js binding.
/// This API accepts visitor objects but does not invoke their callback methods during
/// conversion. It exists as a placeholder for future implementation.
///
/// # Why Callbacks Aren't Implemented
///
/// The Node.js binding faces an architectural limitation:
/// - The Rust `HtmlVisitor` trait is synchronous (methods return `VisitResult`, not `Future`)
/// - NAPI-RS requires access to the V8 environment (`Env`) to call JavaScript functions
/// - This `Env` is only available in the NAPI function context, not within sync visitor methods
/// - Calling JS functions from deep within the Rust conversion loop would require either:
///   - A blocking V8 call (not supported by NAPI-RS)
///   - Refactoring the Rust core to be async (not feasible)
///   - Complex inter-thread communication (performance impact)
///
/// # Alternatives for Node.js Users
///
/// Instead of using visitor callbacks, consider:
/// 1. **Python binding**: Supports full async visitor pattern via `asyncio`
/// 2. **Ruby binding**: Supports full synchronous visitor pattern
/// 3. **Post-conversion processing**: Transform the markdown result in JavaScript after conversion
/// 4. **Preprocessing**: Manipulate the HTML before calling `convert()`
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
/// * `visitor` - Visitor object (accepted but not invoked; reserved for future use)
///
/// # Example
///
/// ```javascript
/// const { convertWithVisitor } = require('html-to-markdown-node');
///
/// const html = '<h1>Hello World</h1>';
/// const visitor = {
///   visitElementStart: () => { /* not called */ },
///   visitElementEnd: () => { /* not called */ },
/// };
///
/// const markdown = convertWithVisitor(html, undefined, visitor);
/// // Returns: # Hello World
/// // (visitor callbacks are currently ignored)
/// ```
#[cfg(feature = "visitor")]
#[napi(js_name = "convertWithVisitor")]
pub fn convert_with_visitor(
    env: Env,
    html: String,
    options: Option<JsConversionOptions>,
    visitor: Object,
) -> Result<String> {
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    let rust_options = options.map(Into::into);
    let bridge = JsVisitorBridge::new(env, Some(visitor));
    let visitor_rc = std::rc::Rc::new(std::cell::RefCell::new(bridge));

    // Use catch_unwind with AssertUnwindSafe instead of guard_panic
    let result = catch_unwind(AssertUnwindSafe(|| {
        profiling::maybe_profile(|| html_to_markdown_rs::convert_with_visitor(&html, rust_options, Some(visitor_rc)))
    }));

    match result {
        Ok(conversion_result) => conversion_result.map_err(to_js_error),
        Err(_) => Err(to_js_error(ConversionError::Panic(
            "Panic during conversion".to_string(),
        ))),
    }
}

#[napi(js_name = "convertJson")]
pub fn convert_json(html: String, options_json: Option<String>) -> Result<String> {
    let rust_options = parse_options_json(options_json)?;
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
    str::from_utf8(html.as_ref())
        .map_err(|e| Error::new(Status::InvalidArg, format!("HTML must be valid UTF-8: {}", e)))
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
    let rust_options = parse_options_json(options_json)?;
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(html, rust_options.clone())))
        .map_err(to_js_error)
}

/// Create a reusable ConversionOptions handle.
#[napi]
pub fn create_conversion_options_handle(options: Option<JsConversionOptions>) -> External<RustConversionOptions> {
    External::new(options.map(Into::into).unwrap_or_default())
}

#[napi(js_name = "createConversionOptionsHandleJson")]
pub fn create_conversion_options_handle_json(options_json: Option<String>) -> Result<External<RustConversionOptions>> {
    let rust_options = parse_options_json(options_json)?;
    Ok(External::new(rust_options.unwrap_or_default()))
}

/// Create a reusable MetadataConfig handle.
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
    let rust_config = parse_metadata_config_json(metadata_config_json)?;
    Ok(External::new(rust_config))
}

/// Convert HTML using a previously-created ConversionOptions handle.
#[napi]
pub fn convert_with_options_handle(html: String, options: &External<RustConversionOptions>) -> Result<String> {
    guard_panic(|| profiling::maybe_profile(|| html_to_markdown_rs::convert(&html, Some((**options).clone()))))
        .map_err(to_js_error)
}

/// Convert HTML Buffer data using a previously-created ConversionOptions handle.
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
    let rust_config = image_config
        .map(Into::into)
        .unwrap_or_else(|| RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));

    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config))
        .map_err(to_js_error)?;

    Ok(build_js_extraction(extraction))
}

fn convert_inline_images_with_handle_impl(
    html: &str,
    options: &External<RustConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let rust_options = Some((**options).clone());
    let rust_config = image_config
        .map(Into::into)
        .unwrap_or_else(|| RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));

    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config))
        .map_err(to_js_error)?;

    Ok(build_js_extraction(extraction))
}

fn convert_inline_images_json_impl(
    html: &str,
    options_json: Option<String>,
    image_config_json: Option<String>,
) -> Result<JsHtmlExtraction> {
    let rust_options = parse_options_json(options_json)?;
    let rust_config = parse_inline_image_config_json(image_config_json)?;

    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config))
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
#[napi]
pub fn convert_with_inline_images(
    html: String,
    options: Option<JsConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
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
) -> Result<JsMetadataExtraction> {
    let rust_options = options.map(Into::into);
    let rust_config = metadata_config.map(Into::into).unwrap_or_default();

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, rust_options, rust_config))
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
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, Some((**options).clone()), rust_config))
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
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, rust_options, rust_config))
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
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, Some((**options).clone()), rust_config))
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
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, None, rust_config)).map_err(to_js_error)?;

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
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, Some((**options).clone()), rust_config))
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
    let rust_options = parse_options_json(options_json)?;
    let rust_config = parse_metadata_config_json(metadata_config_json)?;

    let (markdown, metadata) =
        guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, rust_options, rust_config))
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
