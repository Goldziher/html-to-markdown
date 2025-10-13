use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle, ListIndentType,
    NewlineStyle, PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod inline_images;
pub use inline_images::{WasmHtmlExtraction, WasmInlineImage, WasmInlineImageConfig, WasmInlineImageWarning};

/// Initialize panic hook for better error messages in the browser
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
}

impl From<WasmConversionOptions> for RustConversionOptions {
    fn from(val: WasmConversionOptions) -> Self {
        let defaults = RustConversionOptions::default();
        RustConversionOptions {
            heading_style: val.heading_style.map(Into::into).unwrap_or(defaults.heading_style),
            list_indent_type: val
                .list_indent_type
                .map(Into::into)
                .unwrap_or(defaults.list_indent_type),
            list_indent_width: val.list_indent_width.unwrap_or(defaults.list_indent_width),
            bullets: val.bullets.unwrap_or(defaults.bullets),
            strong_em_symbol: val.strong_em_symbol.unwrap_or(defaults.strong_em_symbol),
            escape_asterisks: val.escape_asterisks.unwrap_or(defaults.escape_asterisks),
            escape_underscores: val.escape_underscores.unwrap_or(defaults.escape_underscores),
            escape_misc: val.escape_misc.unwrap_or(defaults.escape_misc),
            escape_ascii: val.escape_ascii.unwrap_or(defaults.escape_ascii),
            code_language: val.code_language.unwrap_or(defaults.code_language),
            autolinks: val.autolinks.unwrap_or(defaults.autolinks),
            default_title: val.default_title.unwrap_or(defaults.default_title),
            br_in_tables: val.br_in_tables.unwrap_or(defaults.br_in_tables),
            hocr_spatial_tables: val.hocr_spatial_tables.unwrap_or(defaults.hocr_spatial_tables),
            highlight_style: val.highlight_style.map(Into::into).unwrap_or(defaults.highlight_style),
            extract_metadata: val.extract_metadata.unwrap_or(defaults.extract_metadata),
            whitespace_mode: val.whitespace_mode.map(Into::into).unwrap_or(defaults.whitespace_mode),
            strip_newlines: val.strip_newlines.unwrap_or(defaults.strip_newlines),
            wrap: val.wrap.unwrap_or(defaults.wrap),
            wrap_width: val.wrap_width.unwrap_or(defaults.wrap_width),
            convert_as_inline: val.convert_as_inline.unwrap_or(defaults.convert_as_inline),
            sub_symbol: val.sub_symbol.unwrap_or(defaults.sub_symbol),
            sup_symbol: val.sup_symbol.unwrap_or(defaults.sup_symbol),
            newline_style: val.newline_style.map(Into::into).unwrap_or(defaults.newline_style),
            code_block_style: val
                .code_block_style
                .map(Into::into)
                .unwrap_or(defaults.code_block_style),
            keep_inline_images_in: val.keep_inline_images_in.unwrap_or(defaults.keep_inline_images_in),
            preprocessing: val.preprocessing.map(Into::into).unwrap_or(defaults.preprocessing),
            encoding: val.encoding.unwrap_or(defaults.encoding),
            debug: val.debug.unwrap_or(defaults.debug),
            strip_tags: val.strip_tags.unwrap_or(defaults.strip_tags),
        }
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
/// import { convert } from '@html-to-markdown/wasm';
///
/// const html = '<h1>Hello World</h1>';
/// const markdown = convert(html);
/// console.log(markdown); // # Hello World
/// ```
#[wasm_bindgen]
pub fn convert(html: String, options: JsValue) -> Result<String, JsValue> {
    let rust_options = if options.is_undefined() || options.is_null() {
        None
    } else {
        let wasm_options: WasmConversionOptions = serde_wasm_bindgen::from_value(options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse options: {}", e)))?;
        Some(wasm_options.into())
    };

    html_to_markdown_rs::convert(&html, rust_options).map_err(|e| JsValue::from_str(&e.to_string()))
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
/// import { convertWithInlineImages, WasmInlineImageConfig } from '@html-to-markdown/wasm';
///
/// const html = '<img src="data:image/png;base64,..." alt="test">';
/// const config = new WasmInlineImageConfig(1024 * 1024);
/// config.inferDimensions = true;
///
/// const result = convertWithInlineImages(html, null, config);
/// console.log(result.markdown);
/// console.log(result.inlineImages.length);
/// ```
#[wasm_bindgen(js_name = convertWithInlineImages)]
pub fn convert_with_inline_images(
    html: String,
    options: JsValue,
    image_config: Option<WasmInlineImageConfig>,
) -> Result<WasmHtmlExtraction, JsValue> {
    let rust_options = if options.is_undefined() || options.is_null() {
        None
    } else {
        let wasm_options: WasmConversionOptions = serde_wasm_bindgen::from_value(options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse options: {}", e)))?;
        Some(wasm_options.into())
    };

    let rust_config = image_config
        .map(Into::into)
        .unwrap_or_else(|| html_to_markdown_rs::InlineImageConfig::new(5 * 1024 * 1024));

    let extraction = html_to_markdown_rs::convert_with_inline_images(&html, rust_options, rust_config)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(extraction.into())
}

#[cfg(test)]
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
        };

        let js_options = serde_wasm_bindgen::to_value(&options).unwrap();
        let result = convert(html, js_options);
        assert!(result.is_ok());
    }
}
