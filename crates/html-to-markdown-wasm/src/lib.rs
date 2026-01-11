#![allow(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]
#![deny(clippy::correctness, clippy::suspicious)]
#![warn(clippy::all)]
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(feature = "js-bindings")]
use html_to_markdown_rs::DEFAULT_INLINE_IMAGE_LIMIT;
#[cfg(feature = "metadata")]
use html_to_markdown_rs::DEFAULT_MAX_STRUCTURED_DATA_SIZE;
#[cfg(all(feature = "js-bindings", feature = "metadata"))]
use html_to_markdown_rs::MetadataConfigUpdate;
#[cfg(any(feature = "js-bindings", feature = "wasmtime-testing"))]
use html_to_markdown_rs::safety::guard_panic;
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions as RustConversionOptions, ConversionOptionsUpdate, HeadingStyle, HighlightStyle,
    ListIndentType, NewlineStyle, PreprocessingOptionsUpdate, PreprocessingPreset, WhitespaceMode,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "js-bindings")]
use html_to_markdown_rs::ConversionError;
#[cfg(feature = "js-bindings")]
use wasm_bindgen::{JsCast, prelude::*};
#[cfg(feature = "js-bindings")]
mod inline_images;
#[cfg(feature = "js-bindings")]
pub use inline_images::{WasmHtmlExtraction, WasmInlineImage, WasmInlineImageConfig, WasmInlineImageWarning};

#[cfg(feature = "js-bindings")]
fn to_js_error(err: ConversionError) -> JsValue {
    JsValue::from_str(&html_to_markdown_bindings_common::error::error_message(&err))
}

/// Initialize panic hook for better error messages in the browser
#[cfg(feature = "js-bindings")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Heading style options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmHeadingStyle {
    /// Underlined style (=== for h1, --- for h2)
    Underlined,
    /// ATX style (# for h1, ## for h2, etc.)
    Atx,
    /// ATX closed style (# title #)
    AtxClosed,
}

impl From<WasmHeadingStyle> for HeadingStyle {
    fn from(val: WasmHeadingStyle) -> Self {
        match val {
            WasmHeadingStyle::Underlined => HeadingStyle::Underlined,
            WasmHeadingStyle::Atx => HeadingStyle::Atx,
            WasmHeadingStyle::AtxClosed => HeadingStyle::AtxClosed,
        }
    }
}

/// List indentation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmListIndentType {
    Spaces,
    Tabs,
}

impl From<WasmListIndentType> for ListIndentType {
    fn from(val: WasmListIndentType) -> Self {
        match val {
            WasmListIndentType::Spaces => ListIndentType::Spaces,
            WasmListIndentType::Tabs => ListIndentType::Tabs,
        }
    }
}

/// Whitespace handling mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmWhitespaceMode {
    Normalized,
    Strict,
}

impl From<WasmWhitespaceMode> for WhitespaceMode {
    fn from(val: WasmWhitespaceMode) -> Self {
        match val {
            WasmWhitespaceMode::Normalized => WhitespaceMode::Normalized,
            WasmWhitespaceMode::Strict => WhitespaceMode::Strict,
        }
    }
}

/// Newline style
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmNewlineStyle {
    /// Two spaces at end of line
    Spaces,
    /// Backslash at end of line
    Backslash,
}

impl From<WasmNewlineStyle> for NewlineStyle {
    fn from(val: WasmNewlineStyle) -> Self {
        match val {
            WasmNewlineStyle::Spaces => NewlineStyle::Spaces,
            WasmNewlineStyle::Backslash => NewlineStyle::Backslash,
        }
    }
}

/// Code block style
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmCodeBlockStyle {
    /// Indented code blocks (4 spaces) - CommonMark default
    Indented,
    /// Fenced code blocks with backticks (```)
    Backticks,
    /// Fenced code blocks with tildes (~~~)
    Tildes,
}

impl From<WasmCodeBlockStyle> for CodeBlockStyle {
    fn from(val: WasmCodeBlockStyle) -> Self {
        match val {
            WasmCodeBlockStyle::Indented => CodeBlockStyle::Indented,
            WasmCodeBlockStyle::Backticks => CodeBlockStyle::Backticks,
            WasmCodeBlockStyle::Tildes => CodeBlockStyle::Tildes,
        }
    }
}

/// Highlight style for `<mark>` elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmHighlightStyle {
    /// ==text==
    DoubleEqual,
    /// <mark>text</mark>
    Html,
    /// **text**
    Bold,
    /// Plain text (no formatting)
    None,
}

impl From<WasmHighlightStyle> for HighlightStyle {
    fn from(val: WasmHighlightStyle) -> Self {
        match val {
            WasmHighlightStyle::DoubleEqual => HighlightStyle::DoubleEqual,
            WasmHighlightStyle::Html => HighlightStyle::Html,
            WasmHighlightStyle::Bold => HighlightStyle::Bold,
            WasmHighlightStyle::None => HighlightStyle::None,
        }
    }
}

/// Preprocessing preset levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WasmPreprocessingPreset {
    Minimal,
    Standard,
    Aggressive,
}

impl From<WasmPreprocessingPreset> for PreprocessingPreset {
    fn from(val: WasmPreprocessingPreset) -> Self {
        match val {
            WasmPreprocessingPreset::Minimal => PreprocessingPreset::Minimal,
            WasmPreprocessingPreset::Standard => PreprocessingPreset::Standard,
            WasmPreprocessingPreset::Aggressive => PreprocessingPreset::Aggressive,
        }
    }
}

/// HTML preprocessing options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmPreprocessingOptions {
    /// Enable preprocessing
    #[serde(default)]
    pub enabled: bool,
    /// Preprocessing preset
    #[serde(default)]
    pub preset: Option<WasmPreprocessingPreset>,
    /// Remove navigation elements
    #[serde(default = "default_true")]
    pub remove_navigation: bool,
    /// Remove form elements
    #[serde(default = "default_true")]
    pub remove_forms: bool,
}

fn default_true() -> bool {
    true
}

impl From<WasmPreprocessingOptions> for PreprocessingOptionsUpdate {
    fn from(val: WasmPreprocessingOptions) -> Self {
        Self {
            enabled: Some(val.enabled),
            preset: val.preset.map(Into::into),
            remove_navigation: Some(val.remove_navigation),
            remove_forms: Some(val.remove_forms),
        }
    }
}

/// Main conversion options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmConversionOptions {
    /// Heading style
    pub heading_style: Option<WasmHeadingStyle>,
    /// List indentation type
    pub list_indent_type: Option<WasmListIndentType>,
    /// List indentation width (spaces)
    pub list_indent_width: Option<usize>,
    /// Bullet characters for unordered lists
    pub bullets: Option<String>,
    /// Symbol for strong/emphasis (* or _)
    pub strong_em_symbol: Option<char>,
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
    pub highlight_style: Option<WasmHighlightStyle>,
    /// Extract metadata from HTML
    pub extract_metadata: Option<bool>,
    /// Whitespace handling mode
    pub whitespace_mode: Option<WasmWhitespaceMode>,
    /// Strip newlines from HTML before processing
    pub strip_newlines: Option<bool>,
    /// Enable text wrapping
    pub wrap: Option<bool>,
    /// Text wrap width
    pub wrap_width: Option<usize>,
    /// Treat block elements as inline
    pub convert_as_inline: Option<bool>,
    /// Subscript symbol
    pub sub_symbol: Option<String>,
    /// Superscript symbol
    pub sup_symbol: Option<String>,
    /// Newline style
    pub newline_style: Option<WasmNewlineStyle>,
    /// Code block style
    pub code_block_style: Option<WasmCodeBlockStyle>,
    /// Elements where images should remain as markdown
    pub keep_inline_images_in: Option<Vec<String>>,
    /// Skip images during conversion
    pub skip_images: Option<bool>,
    /// Preprocessing options
    pub preprocessing: Option<WasmPreprocessingOptions>,
    /// Source encoding (informational)
    pub encoding: Option<String>,
    /// Enable debug mode with diagnostic warnings
    pub debug: Option<bool>,
    /// List of HTML tags to strip
    pub strip_tags: Option<Vec<String>>,
    /// List of HTML tags to preserve as-is in the output
    pub preserve_tags: Option<Vec<String>>,
}

impl From<WasmConversionOptions> for ConversionOptionsUpdate {
    fn from(val: WasmConversionOptions) -> Self {
        Self {
            heading_style: val.heading_style.map(Into::into),
            list_indent_type: val.list_indent_type.map(Into::into),
            list_indent_width: val.list_indent_width,
            bullets: val.bullets,
            strong_em_symbol: val.strong_em_symbol,
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
            wrap_width: val.wrap_width,
            convert_as_inline: val.convert_as_inline,
            sub_symbol: val.sub_symbol,
            sup_symbol: val.sup_symbol,
            newline_style: val.newline_style.map(Into::into),
            code_block_style: val.code_block_style.map(Into::into),
            keep_inline_images_in: val.keep_inline_images_in,
            skip_images: val.skip_images,
            preprocessing: val.preprocessing.map(Into::into),
            encoding: val.encoding,
            debug: val.debug,
            strip_tags: val.strip_tags,
            preserve_tags: val.preserve_tags,
        }
    }
}

impl From<WasmConversionOptions> for RustConversionOptions {
    fn from(val: WasmConversionOptions) -> Self {
        RustConversionOptions::from(ConversionOptionsUpdate::from(val))
    }
}

#[cfg(feature = "js-bindings")]
fn parse_wasm_options(options: JsValue) -> Result<Option<RustConversionOptions>, JsValue> {
    if options.is_undefined() || options.is_null() {
        return Ok(None);
    }

    if let Some(obj) = options.dyn_ref::<js_sys::Object>() {
        if js_sys::Object::keys(obj).length() == 0 {
            return Ok(None);
        }
    }

    let update: html_to_markdown_rs::ConversionOptionsUpdate = serde_wasm_bindgen::from_value(options)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse options: {}", e)))?;
    Ok(Some(update.into()))
}

#[cfg(feature = "js-bindings")]
fn bytes_to_string(bytes: js_sys::Uint8Array) -> Result<String, JsValue> {
    let mut buffer = vec![0u8; bytes.length() as usize];
    bytes.copy_to(&mut buffer);
    String::from_utf8(buffer).map_err(|e| JsValue::from_str(&format!("HTML must be valid UTF-8: {}", e)))
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen]
pub struct WasmConversionOptionsHandle {
    inner: RustConversionOptions,
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen]
impl WasmConversionOptionsHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Result<WasmConversionOptionsHandle, JsValue> {
        let inner = parse_wasm_options(options)?.unwrap_or_else(RustConversionOptions::default);
        Ok(Self { inner })
    }
}

/// Convert HTML to Markdown
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (as a JavaScript object)
///
/// # Example
///
/// ```javascript
/// import { convert } from 'html-to-markdown-wasm';
///
/// const html = '<h1>Hello World</h1>';
/// const markdown = convert(html);
/// console.log(markdown); // # Hello World
/// ```
#[cfg(feature = "js-bindings")]
#[wasm_bindgen]
pub fn convert(html: String, options: JsValue) -> Result<String, JsValue> {
    let rust_options = parse_wasm_options(options)?;

    #[cfg(feature = "visitor")]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
    }
    #[cfg(not(feature = "visitor"))]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
    }
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertBytes)]
pub fn convert_bytes(html: js_sys::Uint8Array, options: JsValue) -> Result<String, JsValue> {
    let html = bytes_to_string(html)?;
    let rust_options = parse_wasm_options(options)?;

    #[cfg(feature = "visitor")]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
    }
    #[cfg(not(feature = "visitor"))]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
    }
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = createConversionOptionsHandle)]
pub fn create_conversion_options_handle(options: JsValue) -> Result<WasmConversionOptionsHandle, JsValue> {
    WasmConversionOptionsHandle::new(options)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertWithOptionsHandle)]
pub fn convert_with_options_handle(html: String, handle: &WasmConversionOptionsHandle) -> Result<String, JsValue> {
    #[cfg(feature = "visitor")]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
    }
    #[cfg(not(feature = "visitor"))]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
    }
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertBytesWithOptionsHandle)]
pub fn convert_bytes_with_options_handle(
    html: js_sys::Uint8Array,
    handle: &WasmConversionOptionsHandle,
) -> Result<String, JsValue> {
    let html = bytes_to_string(html)?;

    #[cfg(feature = "visitor")]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
    }
    #[cfg(not(feature = "visitor"))]
    {
        guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
    }
}

/// Convert HTML to Markdown while collecting inline images
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (as a JavaScript object)
/// * `image_config` - Configuration for inline image extraction
///
/// # Example
///
/// ```javascript
/// import { convertWithInlineImages, WasmInlineImageConfig } from 'html-to-markdown-wasm';
///
/// const html = '<img src="data:image/png;base64,..." alt="test">';
/// const config = new WasmInlineImageConfig(1024 * 1024);
/// config.inferDimensions = true;
///
/// const result = convertWithInlineImages(html, null, config);
/// console.log(result.markdown);
/// console.log(result.inlineImages.length);
/// ```
#[cfg(feature = "js-bindings")]
fn convert_with_inline_images_internal(
    html: &str,
    options: JsValue,
    image_config: Option<WasmInlineImageConfig>,
) -> Result<WasmHtmlExtraction, JsValue> {
    let rust_options = parse_wasm_options(options)?;

    let rust_config = image_config
        .map(Into::into)
        .unwrap_or_else(|| html_to_markdown_rs::InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));

    let extraction = {
        #[cfg(feature = "visitor")]
        {
            guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config, None))
        }
        #[cfg(not(feature = "visitor"))]
        {
            guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config, None))
        }
    }
    .map_err(to_js_error)?;

    Ok(extraction.into())
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertWithInlineImages)]
pub fn convert_with_inline_images(
    html: String,
    options: JsValue,
    image_config: Option<WasmInlineImageConfig>,
) -> Result<WasmHtmlExtraction, JsValue> {
    convert_with_inline_images_internal(&html, options, image_config)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertBytesWithInlineImages)]
pub fn convert_bytes_with_inline_images(
    html: js_sys::Uint8Array,
    options: JsValue,
    image_config: Option<WasmInlineImageConfig>,
) -> Result<WasmHtmlExtraction, JsValue> {
    let html = bytes_to_string(html)?;
    convert_with_inline_images_internal(&html, options, image_config)
}

/// Metadata extraction configuration
#[cfg(all(feature = "js-bindings", feature = "metadata"))]
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMetadataConfig {
    extract_document: bool,
    extract_headers: bool,
    extract_links: bool,
    extract_images: bool,
    extract_structured_data: bool,
    max_structured_data_size: usize,
}

#[cfg(all(feature = "js-bindings", feature = "metadata"))]
#[wasm_bindgen]
impl WasmMetadataConfig {
    /// Create a new metadata configuration with defaults
    ///
    /// All extraction types enabled by default with 1MB structured data limit
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            extract_document: true,
            extract_headers: true,
            extract_links: true,
            extract_images: true,
            extract_structured_data: true,
            max_structured_data_size: DEFAULT_MAX_STRUCTURED_DATA_SIZE,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn extract_document(&self) -> bool {
        self.extract_document
    }

    #[wasm_bindgen(setter)]
    pub fn set_extract_document(&mut self, value: bool) {
        self.extract_document = value;
    }

    #[wasm_bindgen(getter)]
    pub fn extract_headers(&self) -> bool {
        self.extract_headers
    }

    #[wasm_bindgen(setter)]
    pub fn set_extract_headers(&mut self, value: bool) {
        self.extract_headers = value;
    }

    #[wasm_bindgen(getter)]
    pub fn extract_links(&self) -> bool {
        self.extract_links
    }

    #[wasm_bindgen(setter)]
    pub fn set_extract_links(&mut self, value: bool) {
        self.extract_links = value;
    }

    #[wasm_bindgen(getter)]
    pub fn extract_images(&self) -> bool {
        self.extract_images
    }

    #[wasm_bindgen(setter)]
    pub fn set_extract_images(&mut self, value: bool) {
        self.extract_images = value;
    }

    #[wasm_bindgen(getter)]
    pub fn extract_structured_data(&self) -> bool {
        self.extract_structured_data
    }

    #[wasm_bindgen(setter)]
    pub fn set_extract_structured_data(&mut self, value: bool) {
        self.extract_structured_data = value;
    }

    #[wasm_bindgen(getter)]
    pub fn max_structured_data_size(&self) -> usize {
        self.max_structured_data_size
    }

    #[wasm_bindgen(setter)]
    pub fn set_max_structured_data_size(&mut self, value: usize) {
        self.max_structured_data_size = value;
    }
}

#[cfg(all(feature = "js-bindings", feature = "metadata"))]
impl Default for WasmMetadataConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(feature = "js-bindings", feature = "metadata"))]
impl From<WasmMetadataConfig> for html_to_markdown_rs::MetadataConfig {
    fn from(cfg: WasmMetadataConfig) -> Self {
        let update = MetadataConfigUpdate {
            extract_document: Some(cfg.extract_document),
            extract_headers: Some(cfg.extract_headers),
            extract_links: Some(cfg.extract_links),
            extract_images: Some(cfg.extract_images),
            extract_structured_data: Some(cfg.extract_structured_data),
            max_structured_data_size: Some(cfg.max_structured_data_size),
        };
        html_to_markdown_rs::MetadataConfig::from(update)
    }
}

/// Convert HTML to Markdown with metadata extraction
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (as a JavaScript object)
/// * `metadata_config` - Metadata extraction configuration
///
/// # Returns
///
/// JavaScript object with `markdown` (string) and `metadata` (object) fields
///
/// # Example
///
/// ```javascript
/// import { convertWithMetadata, WasmMetadataConfig } from 'html-to-markdown-wasm';
///
/// const html = '<h1>Hello World</h1><a href="https://example.com">Link</a>';
/// const config = new WasmMetadataConfig();
/// config.extractHeaders = true;
/// config.extractLinks = true;
///
/// const result = convertWithMetadata(html, null, config);
/// console.log(result.markdown); // # Hello World\n\n[Link](https://example.com)
/// console.log(result.metadata.headers); // [{ level: 1, text: "Hello World", ... }]
/// console.log(result.metadata.links); // [{ href: "https://example.com", text: "Link", ... }]
/// ```
#[cfg(all(feature = "js-bindings", feature = "metadata"))]
#[wasm_bindgen(js_name = convertWithMetadata)]
pub fn convert_with_metadata(
    html: String,
    options: JsValue,
    metadata_config: Option<WasmMetadataConfig>,
) -> Result<JsValue, JsValue> {
    let rust_options = parse_wasm_options(options)?;
    let rust_metadata_config = metadata_config.map(Into::into).unwrap_or_default();

    let (markdown, metadata) = {
        #[cfg(feature = "visitor")]
        {
            guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, rust_options, rust_metadata_config, None))
        }
        #[cfg(not(feature = "visitor"))]
        {
            guard_panic(|| html_to_markdown_rs::convert_with_metadata(&html, rust_options, rust_metadata_config, None))
        }
    }
    .map_err(to_js_error)?;

    let metadata_js = serde_wasm_bindgen::to_value(&metadata).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &JsValue::from_str("markdown"), &JsValue::from_str(&markdown))
        .map_err(|_| JsValue::from_str("failed to set markdown property"))?;
    js_sys::Reflect::set(&result, &JsValue::from_str("metadata"), &metadata_js)
        .map_err(|_| JsValue::from_str("failed to set metadata property"))?;

    Ok(result.into())
}

/// Convert HTML bytes to Markdown with metadata extraction
///
/// # Arguments
///
/// * `html` - The HTML bytes to convert
/// * `options` - Optional conversion options (as a JavaScript object)
/// * `metadata_config` - Metadata extraction configuration
///
/// # Returns
///
/// JavaScript object with `markdown` (string) and `metadata` (object) fields
#[cfg(all(feature = "js-bindings", feature = "metadata"))]
#[wasm_bindgen(js_name = convertBytesWithMetadata)]
pub fn convert_bytes_with_metadata(
    html: js_sys::Uint8Array,
    options: JsValue,
    metadata_config: Option<WasmMetadataConfig>,
) -> Result<JsValue, JsValue> {
    let html = bytes_to_string(html)?;
    convert_with_metadata(html, options, metadata_config)
}

#[cfg(feature = "wasmtime-testing")]
mod wasmtime_runtime {
    use super::*;
    use core::{mem, slice, str};
    use std::cell::RefCell;

    thread_local! {
        static RESULT_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    fn write_result(bytes: &[u8]) -> u32 {
        RESULT_BUFFER.with(|buf| {
            let mut buffer = buf.borrow_mut();
            buffer.clear();
            buffer.extend_from_slice(bytes);
            buffer.len() as u32
        })
    }

    fn read_utf8(ptr: u32, len: u32) -> String {
        let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };
        str::from_utf8(slice).expect("input must be valid UTF-8").to_owned()
    }

    fn parse_options(ptr: u32, len: u32) -> Option<RustConversionOptions> {
        if len == 0 {
            return None;
        }
        let json = read_utf8(ptr, len);
        html_to_markdown_bindings_common::json::parse_conversion_options(Some(&json))
            .expect("options JSON must be valid")
    }

    fn convert_internal(html_ptr: u32, html_len: u32, options: Option<RustConversionOptions>) -> u32 {
        let html = read_utf8(html_ptr, html_len);
        let result = {
            #[cfg(feature = "visitor")]
            {
                guard_panic(|| html_to_markdown_rs::convert(&html, options))
            }
            #[cfg(not(feature = "visitor"))]
            {
                guard_panic(|| html_to_markdown_rs::convert(&html, options))
            }
        };
        match result {
            Ok(markdown) => write_result(markdown.as_bytes()),
            Err(err) => write_result(format!("ERROR:{}", err).as_bytes()),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_alloc(len: u32) -> u32 {
        let mut buffer = vec![0u8; len as usize];
        let ptr = buffer.as_mut_ptr();
        mem::forget(buffer);
        ptr as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_dealloc(ptr: u32, len: u32) {
        unsafe {
            Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_result_ptr() -> u32 {
        RESULT_BUFFER.with(|buf| buf.borrow().as_ptr() as u32)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_convert(ptr: u32, len: u32) -> u32 {
        convert_internal(ptr, len, None)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_convert_with_options(
        html_ptr: u32,
        html_len: u32,
        options_ptr: u32,
        options_len: u32,
    ) -> u32 {
        let options = parse_options(options_ptr, options_len);
        convert_internal(html_ptr, html_len, options)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn htmd_convert_underlined(html_ptr: u32, html_len: u32) -> u32 {
        let options = html_to_markdown_rs::ConversionOptions {
            heading_style: HeadingStyle::Underlined,
            wrap: true,
            wrap_width: 12,
            ..Default::default()
        };
        convert_internal(html_ptr, html_len, Some(options))
    }
}

#[cfg(all(test, feature = "js-bindings"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_convert_basic() {
        let html = "<h1>Hello World</h1>".to_string();
        let result = convert(html, JsValue::UNDEFINED);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Hello World"));
    }

    #[wasm_bindgen_test]
    fn test_convert_with_options() {
        let html = "<h1>Hello</h1>".to_string();
        let options = WasmConversionOptions {
            heading_style: Some(WasmHeadingStyle::Atx),
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
            skip_images: None,
            preprocessing: None,
            encoding: None,
            debug: None,
            strip_tags: None,
            preserve_tags: None,
        };

        let js_options = serde_wasm_bindgen::to_value(&options).unwrap();
        let result = convert(html, js_options);
        assert!(result.is_ok());
    }

    #[cfg(feature = "metadata")]
    #[wasm_bindgen_test]
    fn test_metadata_config_new() {
        let config = WasmMetadataConfig::new();
        assert!(config.extract_headers());
        assert!(config.extract_links());
        assert!(config.extract_images());
        assert!(config.extract_structured_data());
        assert_eq!(config.max_structured_data_size(), DEFAULT_MAX_STRUCTURED_DATA_SIZE);
    }

    #[cfg(feature = "metadata")]
    #[wasm_bindgen_test]
    fn test_metadata_config_setters() {
        let mut config = WasmMetadataConfig::new();

        config.set_extract_headers(false);
        assert!(!config.extract_headers());

        config.set_extract_links(false);
        assert!(!config.extract_links());

        config.set_extract_images(false);
        assert!(!config.extract_images());

        config.set_extract_structured_data(false);
        assert!(!config.extract_structured_data());

        config.set_max_structured_data_size(500_000);
        assert_eq!(config.max_structured_data_size(), 500_000);
    }

    #[cfg(feature = "metadata")]
    #[wasm_bindgen_test]
    fn test_convert_with_metadata_basic() {
        let html = "<h1>Hello World</h1>".to_string();
        let config = WasmMetadataConfig::new();

        let result = convert_with_metadata(html, JsValue::UNDEFINED, Some(config));
        assert!(result.is_ok());

        let obj = result.unwrap();
        assert!(js_sys::Reflect::has(&obj, &JsValue::from_str("markdown")).unwrap());
        assert!(js_sys::Reflect::has(&obj, &JsValue::from_str("metadata")).unwrap());
    }

    #[cfg(feature = "metadata")]
    #[wasm_bindgen_test]
    fn test_convert_with_metadata_with_headers() {
        let html = r#"<html><head><title>Test</title></head><body><h1 id="main">Main Title</h1><h2>Subsection</h2></body></html>"#
            .to_string();
        let config = WasmMetadataConfig::new();

        let result = convert_with_metadata(html, JsValue::UNDEFINED, Some(config));
        assert!(result.is_ok());

        let obj = result.unwrap();
        let markdown = js_sys::Reflect::get(&obj, &JsValue::from_str("markdown")).unwrap();
        let markdown_str = markdown.as_string().unwrap();
        assert!(markdown_str.contains("Main Title"));
    }

    #[cfg(feature = "metadata")]
    #[wasm_bindgen_test]
    fn test_convert_bytes_with_metadata() {
        let html_bytes = vec![60, 104, 49, 62, 72, 101, 108, 108, 111, 60, 47, 104, 49, 62];
        let uint8 = js_sys::Uint8Array::from(&html_bytes[..]);
        let config = WasmMetadataConfig::new();

        let result = convert_bytes_with_metadata(uint8, JsValue::UNDEFINED, Some(config));
        assert!(result.is_ok());

        let obj = result.unwrap();
        assert!(js_sys::Reflect::has(&obj, &JsValue::from_str("markdown")).unwrap());
        assert!(js_sys::Reflect::has(&obj, &JsValue::from_str("metadata")).unwrap());
    }
}
