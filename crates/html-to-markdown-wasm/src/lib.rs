use html_to_markdown_rs::safety::guard_panic;
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle,
    ListIndentType, NewlineStyle, PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset,
    WhitespaceMode,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "js-bindings")]
use wasm_bindgen::{JsCast, prelude::*};
#[cfg(feature = "js-bindings")]
mod inline_images;
#[cfg(feature = "js-bindings")]
pub use inline_images::{WasmHtmlExtraction, WasmInlineImage, WasmInlineImageConfig, WasmInlineImageWarning};

#[cfg(feature = "js-bindings")]
fn to_js_error(err: ConversionError) -> JsValue {
    let message = match &err {
        ConversionError::Panic(msg) => format!("html-to-markdown panic during conversion: {msg}"),
        other => other.to_string(),
    };
    JsValue::from_str(&message)
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

impl From<WasmPreprocessingOptions> for RustPreprocessingOptions {
    fn from(val: WasmPreprocessingOptions) -> Self {
        RustPreprocessingOptions {
            enabled: val.enabled,
            preset: val.preset.map(Into::into).unwrap_or(PreprocessingPreset::Standard),
            remove_navigation: val.remove_navigation,
            remove_forms: val.remove_forms,
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

impl From<WasmConversionOptions> for RustConversionOptions {
    fn from(val: WasmConversionOptions) -> Self {
        let mut opts = RustConversionOptions::default();

        if let Some(heading_style) = val.heading_style {
            opts.heading_style = heading_style.into();
        }
        if let Some(list_indent_type) = val.list_indent_type {
            opts.list_indent_type = list_indent_type.into();
        }
        if let Some(list_indent_width) = val.list_indent_width {
            opts.list_indent_width = list_indent_width;
        }
        if let Some(bullets) = val.bullets {
            opts.bullets = bullets;
        }
        if let Some(strong_em_symbol) = val.strong_em_symbol {
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
            opts.wrap_width = wrap_width;
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

    let wasm_options: WasmConversionOptions = serde_wasm_bindgen::from_value(options)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse options: {}", e)))?;
    Ok(Some(wasm_options.into()))
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

    guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertBytes)]
pub fn convert_bytes(html: js_sys::Uint8Array, options: JsValue) -> Result<String, JsValue> {
    let html = bytes_to_string(html)?;
    let rust_options = parse_wasm_options(options)?;
    guard_panic(|| html_to_markdown_rs::convert(&html, rust_options)).map_err(to_js_error)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = createConversionOptionsHandle)]
pub fn create_conversion_options_handle(options: JsValue) -> Result<WasmConversionOptionsHandle, JsValue> {
    WasmConversionOptionsHandle::new(options)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertWithOptionsHandle)]
pub fn convert_with_options_handle(html: String, handle: &WasmConversionOptionsHandle) -> Result<String, JsValue> {
    guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
}

#[cfg(feature = "js-bindings")]
#[wasm_bindgen(js_name = convertBytesWithOptionsHandle)]
pub fn convert_bytes_with_options_handle(
    html: js_sys::Uint8Array,
    handle: &WasmConversionOptionsHandle,
) -> Result<String, JsValue> {
    let html = bytes_to_string(html)?;
    guard_panic(|| html_to_markdown_rs::convert(&html, Some(handle.inner.clone()))).map_err(to_js_error)
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
        .unwrap_or_else(|| html_to_markdown_rs::InlineImageConfig::new(5 * 1024 * 1024));

    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, rust_config))
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

#[cfg(feature = "wasmtime-testing")]
mod wasmtime_runtime {
    use super::*;
    use core::{mem, slice, str};
    use std::cell::RefCell;

    thread_local! {
        static RESULT_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
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
        if json.trim().is_empty() {
            return None;
        }
        let wasm_options: WasmConversionOptions = serde_json::from_str(&json).expect("options JSON must be valid");
        Some(wasm_options.into())
    }

    fn convert_internal(html_ptr: u32, html_len: u32, options: Option<RustConversionOptions>) -> u32 {
        let html = read_utf8(html_ptr, html_len);
        match guard_panic(|| html_to_markdown_rs::convert(&html, options)) {
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
        let mut options = html_to_markdown_rs::ConversionOptions::default();
        options.heading_style = HeadingStyle::Underlined;
        options.wrap = true;
        options.wrap_width = 12;
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
}
