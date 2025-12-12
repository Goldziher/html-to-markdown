#![deny(clippy::all)]

#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DEFAULT_MAX_STRUCTURED_DATA_SIZE, DocumentMetadata as RustDocumentMetadata,
    ExtendedMetadata as RustExtendedMetadata, HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata,
    ImageType as RustImageType, LinkMetadata as RustLinkMetadata, LinkType as RustLinkType,
    MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData,
    StructuredDataType as RustStructuredDataType, TextDirection as RustTextDirection,
};
use html_to_markdown_rs::safety::guard_panic;
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle,
    InlineImageConfig as RustInlineImageConfig, InlineImageFormat, InlineImageSource, ListIndentType, NewlineStyle,
    PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset, WhitespaceMode,
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

impl From<JsPreprocessingOptions> for RustPreprocessingOptions {
    fn from(val: JsPreprocessingOptions) -> Self {
        RustPreprocessingOptions {
            enabled: val.enabled.unwrap_or(false),
            preset: val.preset.map(Into::into).unwrap_or(PreprocessingPreset::Standard),
            remove_navigation: val.remove_navigation.unwrap_or(true),
            remove_forms: val.remove_forms.unwrap_or(true),
        }
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

impl From<JsConversionOptions> for RustConversionOptions {
    fn from(val: JsConversionOptions) -> Self {
        let mut opts = RustConversionOptions::default();

        if let Some(heading_style) = val.heading_style {
            opts.heading_style = heading_style.into();
        }
        if let Some(list_indent_type) = val.list_indent_type {
            opts.list_indent_type = list_indent_type.into();
        }
        if let Some(list_indent_width) = val.list_indent_width {
            opts.list_indent_width = list_indent_width as usize;
        }
        if let Some(bullets) = val.bullets {
            opts.bullets = bullets;
        }
        if let Some(strong_em_symbol) = val.strong_em_symbol.and_then(|s| s.chars().next()) {
            opts.strong_em_symbol = strong_em_symbol;
        }
        if let Some(escape_asterisks) = val.escape_asterisks {
            opts.escape_asterisks = escape_asterisks;
        }
        if let Some(escape_underscores) = val.escape_underscores {
            opts.escape_underscores = escape_underscores;
        }
        if let Some(escape_misc) = val.escape_misc {
            opts.escape_misc = escape_misc;
        }
        if let Some(escape_ascii) = val.escape_ascii {
            opts.escape_ascii = escape_ascii;
        }
        if let Some(code_language) = val.code_language {
            opts.code_language = code_language;
        }
        if let Some(autolinks) = val.autolinks {
            opts.autolinks = autolinks;
        }
        if let Some(default_title) = val.default_title {
            opts.default_title = default_title;
        }
        if let Some(br_in_tables) = val.br_in_tables {
            opts.br_in_tables = br_in_tables;
        }
        if let Some(hocr_spatial_tables) = val.hocr_spatial_tables {
            opts.hocr_spatial_tables = hocr_spatial_tables;
        }
        if let Some(highlight_style) = val.highlight_style {
            opts.highlight_style = highlight_style.into();
        }
        if let Some(extract_metadata) = val.extract_metadata {
            opts.extract_metadata = extract_metadata;
        }
        if let Some(whitespace_mode) = val.whitespace_mode {
            opts.whitespace_mode = whitespace_mode.into();
        }
        if let Some(strip_newlines) = val.strip_newlines {
            opts.strip_newlines = strip_newlines;
        }
        if let Some(wrap) = val.wrap {
            opts.wrap = wrap;
        }
        if let Some(wrap_width) = val.wrap_width {
            opts.wrap_width = wrap_width as usize;
        }
        if let Some(convert_as_inline) = val.convert_as_inline {
            opts.convert_as_inline = convert_as_inline;
        }
        if let Some(sub_symbol) = val.sub_symbol {
            opts.sub_symbol = sub_symbol;
        }
        if let Some(sup_symbol) = val.sup_symbol {
            opts.sup_symbol = sup_symbol;
        }
        if let Some(newline_style) = val.newline_style {
            opts.newline_style = newline_style.into();
        }
        if let Some(code_block_style) = val.code_block_style {
            opts.code_block_style = code_block_style.into();
        }
        if let Some(keep_inline_images_in) = val.keep_inline_images_in {
            opts.keep_inline_images_in = keep_inline_images_in;
        }
        if let Some(preprocessing) = val.preprocessing {
            opts.preprocessing = preprocessing.into();
        }
        if let Some(encoding) = val.encoding {
            opts.encoding = encoding;
        }
        if let Some(debug) = val.debug {
            opts.debug = debug;
        }
        if let Some(strip_tags) = val.strip_tags {
            opts.strip_tags = strip_tags;
        }
        if let Some(preserve_tags) = val.preserve_tags {
            opts.preserve_tags = preserve_tags;
        }

        opts
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

impl From<JsInlineImageConfig> for RustInlineImageConfig {
    fn from(val: JsInlineImageConfig) -> Self {
        let max_size = val
            .max_decoded_size_bytes
            .map(|b| b.get_u64().1)
            .unwrap_or(5 * 1024 * 1024);

        let mut cfg = RustInlineImageConfig::new(max_size);
        cfg.filename_prefix = val.filename_prefix;
        cfg.capture_svg = val.capture_svg.unwrap_or(true);
        cfg.infer_dimensions = val.infer_dimensions.unwrap_or(false);
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

fn format_to_string(format: &InlineImageFormat) -> String {
    match format {
        InlineImageFormat::Png => "png".to_string(),
        InlineImageFormat::Jpeg => "jpeg".to_string(),
        InlineImageFormat::Gif => "gif".to_string(),
        InlineImageFormat::Bmp => "bmp".to_string(),
        InlineImageFormat::Webp => "webp".to_string(),
        InlineImageFormat::Svg => "svg".to_string(),
        InlineImageFormat::Other(s) => s.clone(),
    }
}

fn source_to_string(source: &InlineImageSource) -> String {
    match source {
        InlineImageSource::ImgDataUri => "img_data_uri".to_string(),
        InlineImageSource::SvgElement => "svg_element".to_string(),
    }
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
        RustMetadataConfig {
            extract_document: val.extract_document.unwrap_or(true),
            extract_headers: val.extract_headers.unwrap_or(true),
            extract_links: val.extract_links.unwrap_or(true),
            extract_images: val.extract_images.unwrap_or(true),
            extract_structured_data: val.extract_structured_data.unwrap_or(true),
            max_structured_data_size: val
                .max_structured_data_size
                .unwrap_or(DEFAULT_MAX_STRUCTURED_DATA_SIZE as i64) as usize,
        }
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
fn text_direction_to_string(direction: Option<RustTextDirection>) -> Option<String> {
    direction.map(|d| d.to_string())
}

#[cfg(feature = "metadata")]
fn link_type_to_string(link_type: &RustLinkType) -> String {
    link_type.to_string()
}

#[cfg(feature = "metadata")]
fn image_type_to_string(image_type: &RustImageType) -> String {
    image_type.to_string()
}

#[cfg(feature = "metadata")]
fn structured_data_type_to_string(data_type: &RustStructuredDataType) -> String {
    data_type.to_string()
}

#[cfg(feature = "metadata")]
fn convert_document_metadata(doc: RustDocumentMetadata) -> JsDocumentMetadata {
    let mut title = None;
    let mut description = None;
    let mut author = None;
    let mut canonical_url = None;
    let mut base_href = None;
    let mut keywords = Vec::new();
    let mut open_graph = HashMap::new();
    let mut twitter_card = HashMap::new();
    let mut meta_tags = HashMap::new();

    for (raw_key, value) in doc.meta_tags.iter() {
        let mut key = raw_key.to_lowercase();
        let value = value.clone();

        if let Some(stripped) = key.strip_prefix("meta-") {
            key = stripped.to_string();
        }

        if key.contains(':') {
            key = key.replace(':', "-");
        }

        match key.as_str() {
            "title" => title = Some(value.clone()),
            "description" => description = Some(value.clone()),
            "author" => author = Some(value.clone()),
            "canonical" => canonical_url = Some(value.clone()),
            "base" | "base-href" => base_href = Some(value.clone()),
            "keywords" => {
                keywords = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            k if k.starts_with("og-") => {
                let og_key = k.trim_start_matches("og-").replace('-', "_");
                open_graph.insert(og_key, value.clone());
            }
            k if k.starts_with("twitter-") => {
                let tw_key = k.trim_start_matches("twitter-").replace('-', "_");
                twitter_card.insert(tw_key, value.clone());
            }
            _ => {
                meta_tags.insert(key, value.clone());
            }
        }
    }

    JsDocumentMetadata {
        title: doc.title.or(title),
        description: doc.description.or(description),
        keywords: if doc.keywords.is_empty() {
            keywords
        } else {
            doc.keywords
        },
        author: doc.author.or(author),
        canonical_url: doc.canonical_url.or(canonical_url),
        base_href: doc.base_href.or(base_href),
        language: doc.language,
        text_direction: text_direction_to_string(doc.text_direction),
        open_graph: if doc.open_graph.is_empty() {
            open_graph
        } else {
            doc.open_graph.into_iter().collect()
        },
        twitter_card: if doc.twitter_card.is_empty() {
            twitter_card
        } else {
            doc.twitter_card.into_iter().collect()
        },
        meta_tags: if doc.meta_tags.is_empty() {
            meta_tags
        } else {
            doc.meta_tags.into_iter().collect()
        },
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
            link_type: link_type_to_string(&l.link_type),
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
            image_type: image_type_to_string(&i.image_type),
            attributes: i.attributes.into_iter().collect(),
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn convert_structured_data(data: Vec<RustStructuredData>) -> Vec<JsStructuredData> {
    data.into_iter()
        .map(|d| JsStructuredData {
            data_type: structured_data_type_to_string(&d.data_type),
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
    guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
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
    guard_panic(|| html_to_markdown_rs::convert(html, rust_options)).map_err(to_js_error)
}

/// Create a reusable ConversionOptions handle.
#[napi]
pub fn create_conversion_options_handle(options: Option<JsConversionOptions>) -> External<RustConversionOptions> {
    External::new(options.map(Into::into).unwrap_or_default())
}

/// Convert HTML using a previously-created ConversionOptions handle.
#[napi]
pub fn convert_with_options_handle(html: String, options: &External<RustConversionOptions>) -> Result<String> {
    guard_panic(|| html_to_markdown_rs::convert(&html, Some((**options).clone()))).map_err(to_js_error)
}

/// Convert HTML Buffer data using a previously-created ConversionOptions handle.
#[napi(js_name = "convertBufferWithOptionsHandle")]
pub fn convert_buffer_with_options_handle(html: Buffer, options: &External<RustConversionOptions>) -> Result<String> {
    let html = buffer_to_str(&html)?;
    guard_panic(|| html_to_markdown_rs::convert(html, Some((**options).clone()))).map_err(to_js_error)
}

fn convert_inline_images_impl(
    html: &str,
    options: Option<JsConversionOptions>,
    image_config: Option<JsInlineImageConfig>,
) -> Result<JsHtmlExtraction> {
    let rust_options = options.map(Into::into);
    let rust_config = image_config
        .map(Into::into)
        .unwrap_or_else(|| RustInlineImageConfig::new(5 * 1024 * 1024));

    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config))
        .map_err(to_js_error)?;

    let inline_images = extraction
        .inline_images
        .into_iter()
        .map(|img| JsInlineImage {
            data: img.data.into(),
            format: format_to_string(&img.format),
            filename: img.filename,
            description: img.description,
            dimensions: img.dimensions.map(|(w, h)| vec![w, h]),
            source: source_to_string(&img.source),
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

    Ok(JsHtmlExtraction {
        markdown: extraction.markdown,
        inline_images,
        warnings,
    })
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
