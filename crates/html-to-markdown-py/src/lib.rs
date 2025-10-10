use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle, ListIndentType,
    NewlineStyle, ParsingOptions as RustParsingOptions, PreprocessingOptions as RustPreprocessingOptions,
    PreprocessingPreset, WhitespaceMode,
};
use pyo3::prelude::*;

/// Python wrapper for PreprocessingOptions
#[pyclass]
#[derive(Clone)]
struct PreprocessingOptions {
    #[pyo3(get, set)]
    enabled: bool,
    #[pyo3(get, set)]
    preset: String,
    #[pyo3(get, set)]
    remove_navigation: bool,
    #[pyo3(get, set)]
    remove_forms: bool,
}

#[pymethods]
impl PreprocessingOptions {
    #[new]
    #[pyo3(signature = (enabled=false, preset="standard".to_string(), remove_navigation=true, remove_forms=true))]
    fn new(enabled: bool, preset: String, remove_navigation: bool, remove_forms: bool) -> Self {
        Self {
            enabled,
            preset,
            remove_navigation,
            remove_forms,
        }
    }
}

impl PreprocessingOptions {
    /// Convert to Rust PreprocessingOptions
    fn to_rust(&self) -> RustPreprocessingOptions {
        RustPreprocessingOptions {
            enabled: self.enabled,
            preset: match self.preset.as_str() {
                "minimal" => PreprocessingPreset::Minimal,
                "aggressive" => PreprocessingPreset::Aggressive,
                _ => PreprocessingPreset::Standard,
            },
            remove_navigation: self.remove_navigation,
            remove_forms: self.remove_forms,
        }
    }
}

/// Python wrapper for ParsingOptions
#[pyclass]
#[derive(Clone)]
struct ParsingOptions {
    #[pyo3(get, set)]
    encoding: String,
    #[pyo3(get, set)]
    parser: Option<String>,
}

#[pymethods]
impl ParsingOptions {
    #[new]
    #[pyo3(signature = (encoding="utf-8".to_string(), parser=None))]
    fn new(encoding: String, parser: Option<String>) -> Self {
        Self { encoding, parser }
    }
}

impl ParsingOptions {
    /// Convert to Rust ParsingOptions
    fn to_rust(&self) -> RustParsingOptions {
        RustParsingOptions {
            encoding: self.encoding.clone(),
            parser: self.parser.clone(),
        }
    }
}

/// Python wrapper for ConversionOptions
#[pyclass]
#[derive(Clone)]
struct ConversionOptions {
    #[pyo3(get, set)]
    heading_style: String,
    #[pyo3(get, set)]
    list_indent_type: String,
    #[pyo3(get, set)]
    list_indent_width: usize,
    #[pyo3(get, set)]
    bullets: String,
    #[pyo3(get, set)]
    strong_em_symbol: char,
    #[pyo3(get, set)]
    escape_asterisks: bool,
    #[pyo3(get, set)]
    escape_underscores: bool,
    #[pyo3(get, set)]
    escape_misc: bool,
    #[pyo3(get, set)]
    escape_ascii: bool,
    #[pyo3(get, set)]
    code_language: String,
    #[pyo3(get, set)]
    autolinks: bool,
    #[pyo3(get, set)]
    default_title: bool,
    #[pyo3(get, set)]
    br_in_tables: bool,
    #[pyo3(get, set)]
    highlight_style: String,
    #[pyo3(get, set)]
    extract_metadata: bool,
    #[pyo3(get, set)]
    whitespace_mode: String,
    #[pyo3(get, set)]
    strip_newlines: bool,
    #[pyo3(get, set)]
    wrap: bool,
    #[pyo3(get, set)]
    wrap_width: usize,
    #[pyo3(get, set)]
    convert_as_inline: bool,
    #[pyo3(get, set)]
    sub_symbol: String,
    #[pyo3(get, set)]
    sup_symbol: String,
    #[pyo3(get, set)]
    newline_style: String,
    #[pyo3(get, set)]
    code_block_style: String,
    #[pyo3(get, set)]
    keep_inline_images_in: Vec<String>,
    #[pyo3(get, set)]
    hocr_extract_tables: bool,
    #[pyo3(get, set)]
    hocr_table_column_threshold: u32,
    #[pyo3(get, set)]
    hocr_table_row_threshold_ratio: f64,
    #[pyo3(get, set)]
    preprocessing: PreprocessingOptions,
    #[pyo3(get, set)]
    parsing: ParsingOptions,
    #[pyo3(get, set)]
    debug: bool,
    #[pyo3(get, set)]
    strip_tags: Vec<String>,
}

#[pymethods]
impl ConversionOptions {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        heading_style="underlined".to_string(),
        list_indent_type="spaces".to_string(),
        list_indent_width=4,
        bullets="*+-".to_string(),
        strong_em_symbol='*',
        escape_asterisks=false,
        escape_underscores=false,
        escape_misc=false,
        escape_ascii=false,
        code_language="".to_string(),
        autolinks=true,
        default_title=false,
        br_in_tables=false,
        highlight_style="double-equal".to_string(),
        extract_metadata=true,
        whitespace_mode="normalized".to_string(),
        strip_newlines=false,
        wrap=false,
        wrap_width=80,
        convert_as_inline=false,
        sub_symbol="".to_string(),
        sup_symbol="".to_string(),
        newline_style="spaces".to_string(),
        code_block_style="indented".to_string(),
        keep_inline_images_in=Vec::new(),
        hocr_extract_tables=true,
        hocr_table_column_threshold=50,
        hocr_table_row_threshold_ratio=0.5,
        preprocessing=None,
        parsing=None,
        debug=false,
        strip_tags=Vec::new()
    ))]
    fn new(
        heading_style: String,
        list_indent_type: String,
        list_indent_width: usize,
        bullets: String,
        strong_em_symbol: char,
        escape_asterisks: bool,
        escape_underscores: bool,
        escape_misc: bool,
        escape_ascii: bool,
        code_language: String,
        autolinks: bool,
        default_title: bool,
        br_in_tables: bool,
        highlight_style: String,
        extract_metadata: bool,
        whitespace_mode: String,
        strip_newlines: bool,
        wrap: bool,
        wrap_width: usize,
        convert_as_inline: bool,
        sub_symbol: String,
        sup_symbol: String,
        newline_style: String,
        code_block_style: String,
        keep_inline_images_in: Vec<String>,
        hocr_extract_tables: bool,
        hocr_table_column_threshold: u32,
        hocr_table_row_threshold_ratio: f64,
        preprocessing: Option<PreprocessingOptions>,
        parsing: Option<ParsingOptions>,
        debug: bool,
        strip_tags: Vec<String>,
    ) -> Self {
        Self {
            heading_style,
            list_indent_type,
            list_indent_width,
            bullets,
            strong_em_symbol,
            escape_asterisks,
            escape_underscores,
            escape_misc,
            escape_ascii,
            code_language,
            autolinks,
            default_title,
            br_in_tables,
            highlight_style,
            extract_metadata,
            whitespace_mode,
            strip_newlines,
            wrap,
            wrap_width,
            convert_as_inline,
            sub_symbol,
            sup_symbol,
            newline_style,
            code_block_style,
            keep_inline_images_in,
            hocr_extract_tables,
            hocr_table_column_threshold,
            hocr_table_row_threshold_ratio,
            preprocessing: preprocessing
                .unwrap_or_else(|| PreprocessingOptions::new(false, "standard".to_string(), true, true)),
            parsing: parsing.unwrap_or_else(|| ParsingOptions::new("utf-8".to_string(), None)),
            debug,
            strip_tags,
        }
    }
}

impl ConversionOptions {
    /// Convert to Rust ConversionOptions
    fn to_rust(&self) -> RustConversionOptions {
        RustConversionOptions {
            heading_style: match self.heading_style.as_str() {
                "atx" => HeadingStyle::Atx,
                "atx_closed" => HeadingStyle::AtxClosed,
                _ => HeadingStyle::Underlined,
            },
            list_indent_type: match self.list_indent_type.as_str() {
                "tabs" => ListIndentType::Tabs,
                _ => ListIndentType::Spaces,
            },
            list_indent_width: self.list_indent_width,
            bullets: self.bullets.clone(),
            strong_em_symbol: self.strong_em_symbol,
            escape_asterisks: self.escape_asterisks,
            escape_underscores: self.escape_underscores,
            escape_misc: self.escape_misc,
            escape_ascii: self.escape_ascii,
            code_language: self.code_language.clone(),
            autolinks: self.autolinks,
            default_title: self.default_title,
            br_in_tables: self.br_in_tables,
            highlight_style: match self.highlight_style.as_str() {
                "double-equal" => HighlightStyle::DoubleEqual,
                "html" => HighlightStyle::Html,
                "bold" => HighlightStyle::Bold,
                _ => HighlightStyle::None,
            },
            extract_metadata: self.extract_metadata,
            whitespace_mode: match self.whitespace_mode.as_str() {
                "strict" => WhitespaceMode::Strict,
                _ => WhitespaceMode::Normalized,
            },
            strip_newlines: self.strip_newlines,
            wrap: self.wrap,
            wrap_width: self.wrap_width,
            convert_as_inline: self.convert_as_inline,
            sub_symbol: self.sub_symbol.clone(),
            sup_symbol: self.sup_symbol.clone(),
            newline_style: match self.newline_style.as_str() {
                "backslash" => NewlineStyle::Backslash,
                _ => NewlineStyle::Spaces,
            },
            code_block_style: match self.code_block_style.as_str() {
                "backticks" => CodeBlockStyle::Backticks,
                "tildes" => CodeBlockStyle::Tildes,
                _ => CodeBlockStyle::Indented,
            },
            keep_inline_images_in: self.keep_inline_images_in.clone(),
            hocr_extract_tables: self.hocr_extract_tables,
            hocr_table_column_threshold: self.hocr_table_column_threshold,
            hocr_table_row_threshold_ratio: self.hocr_table_row_threshold_ratio,
            preprocessing: self.preprocessing.to_rust(),
            parsing: self.parsing.to_rust(),
            debug: self.debug,
            strip_tags: self.strip_tags.clone(),
        }
    }
}

/// Convert HTML to Markdown
#[pyfunction]
#[pyo3(signature = (html, options=None))]
fn convert(html: &str, options: Option<ConversionOptions>) -> PyResult<String> {
    let rust_options = options.map(|opts| opts.to_rust());
    html_to_markdown_rs::convert(html, rust_options).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Python bindings for html-to-markdown
#[pymodule]
fn _html_to_markdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_class::<ConversionOptions>()?;
    m.add_class::<PreprocessingOptions>()?;
    m.add_class::<ParsingOptions>()?;
    Ok(())
}
