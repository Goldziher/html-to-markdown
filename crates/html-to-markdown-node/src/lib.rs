#![deny(clippy::all)]

use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle,
    InlineImageConfig as RustInlineImageConfig, InlineImageFormat, InlineImageSource, ListIndentType, NewlineStyle,
    PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::{collections::HashMap, str};

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
    html_to_markdown_rs::convert(&html, rust_options).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
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
    html_to_markdown_rs::convert(html, rust_options).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}

/// Create a reusable ConversionOptions handle.
#[napi]
pub fn create_conversion_options_handle(options: Option<JsConversionOptions>) -> External<RustConversionOptions> {
    External::new(options.map(Into::into).unwrap_or_default())
}

/// Convert HTML using a previously-created ConversionOptions handle.
#[napi]
pub fn convert_with_options_handle(html: String, options: &External<RustConversionOptions>) -> Result<String> {
    html_to_markdown_rs::convert(&html, Some((**options).clone()))
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}

/// Convert HTML Buffer data using a previously-created ConversionOptions handle.
#[napi(js_name = "convertBufferWithOptionsHandle")]
pub fn convert_buffer_with_options_handle(html: Buffer, options: &External<RustConversionOptions>) -> Result<String> {
    let html = buffer_to_str(&html)?;
    html_to_markdown_rs::convert(html, Some((**options).clone()))
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
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

    let extraction = html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

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
        assert_eq!(rust_opts.list_indent_width, 2); // default
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
