use html_to_markdown_rs::{
    CodeBlockStyle, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle,
    InlineImageConfig as RustInlineImageConfig, InlineImageFormat, InlineImageSource, ListIndentType, NewlineStyle,
    PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

type PyInlineExtraction = PyResult<(String, Vec<Py<PyAny>>, Vec<Py<PyAny>>)>;

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

/// Python wrapper for inline image extraction configuration
#[pyclass]
#[derive(Clone)]
struct InlineImageConfig {
    #[pyo3(get, set)]
    max_decoded_size_bytes: u64,
    #[pyo3(get, set)]
    filename_prefix: Option<String>,
    #[pyo3(get, set)]
    capture_svg: bool,
    #[pyo3(get, set)]
    infer_dimensions: bool,
}

#[pymethods]
impl InlineImageConfig {
    #[new]
    #[pyo3(signature = (
        max_decoded_size_bytes=5 * 1024 * 1024,
        filename_prefix=None,
        capture_svg=true,
        infer_dimensions=false
    ))]
    fn new(
        max_decoded_size_bytes: u64,
        filename_prefix: Option<String>,
        capture_svg: bool,
        infer_dimensions: bool,
    ) -> Self {
        Self {
            max_decoded_size_bytes,
            filename_prefix,
            capture_svg,
            infer_dimensions,
        }
    }
}

impl InlineImageConfig {
    fn to_rust(&self) -> RustInlineImageConfig {
        let mut cfg = RustInlineImageConfig::new(self.max_decoded_size_bytes);
        cfg.filename_prefix = self.filename_prefix.clone();
        cfg.capture_svg = self.capture_svg;
        cfg.infer_dimensions = self.infer_dimensions;
        cfg
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
    hocr_spatial_tables: bool,
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
    preprocessing: PreprocessingOptions,
    #[pyo3(get, set)]
    debug: bool,
    #[pyo3(get, set)]
    strip_tags: Vec<String>,
    #[pyo3(get, set)]
    preserve_tags: Vec<String>,
    #[pyo3(get, set)]
    encoding: String,
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
        hocr_spatial_tables=true,
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
        preprocessing=None,
        debug=false,
        strip_tags=Vec::new(),
        preserve_tags=Vec::new(),
        encoding="utf-8".to_string()
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
        hocr_spatial_tables: bool,
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
        preprocessing: Option<PreprocessingOptions>,
        debug: bool,
        strip_tags: Vec<String>,
        preserve_tags: Vec<String>,
        encoding: String,
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
            hocr_spatial_tables,
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
            preprocessing: preprocessing
                .unwrap_or_else(|| PreprocessingOptions::new(false, "standard".to_string(), true, true)),
            debug,
            strip_tags,
            preserve_tags,
            encoding,
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
            hocr_spatial_tables: self.hocr_spatial_tables,
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
            preprocessing: self.preprocessing.to_rust(),
            encoding: self.encoding.clone(),
            debug: self.debug,
            strip_tags: self.strip_tags.clone(),
            preserve_tags: self.preserve_tags.clone(),
        }
    }
}

#[pyclass(name = "ConversionOptionsHandle")]
#[derive(Clone)]
struct ConversionOptionsHandle {
    inner: RustConversionOptions,
}

impl ConversionOptionsHandle {
    fn new_with_options(options: Option<ConversionOptions>) -> Self {
        let inner = options.map(|opts| opts.to_rust()).unwrap_or_default();
        Self { inner }
    }
}

#[pymethods]
impl ConversionOptionsHandle {
    #[new]
    #[pyo3(signature = (options=None))]
    fn py_new(options: Option<ConversionOptions>) -> Self {
        ConversionOptionsHandle::new_with_options(options)
    }
}

/// Convert HTML to Markdown.
///
/// Args:
///     html: HTML string to convert
///     options: Optional conversion configuration
///
/// Returns:
///     Markdown string
///
/// Raises:
///     ValueError: Invalid HTML or configuration
///
/// Example:
///     ```ignore
///     from html_to_markdown import convert, ConversionOptions
///
///     html = "<h1>Hello</h1><p>World</p>"
///     markdown = convert(html)
///
///     # With options
///     options = ConversionOptions(heading_style="atx")
///     markdown = convert(html, options)
///     ```
#[pyfunction]
#[pyo3(signature = (html, options=None))]
fn convert(html: &str, options: Option<ConversionOptions>) -> PyResult<String> {
    let rust_options = options.map(|opts| opts.to_rust());
    html_to_markdown_rs::convert(html, rust_options).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pyfunction]
#[pyo3(signature = (html, handle))]
fn convert_with_options_handle(html: &str, handle: &ConversionOptionsHandle) -> PyResult<String> {
    html_to_markdown_rs::convert(html, Some(handle.inner.clone()))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pyfunction]
#[pyo3(signature = (options=None))]
fn create_options_handle(options: Option<ConversionOptions>) -> ConversionOptionsHandle {
    ConversionOptionsHandle::new_with_options(options)
}

fn inline_image_format_to_str(format: &InlineImageFormat) -> String {
    match format {
        InlineImageFormat::Png => "png".to_string(),
        InlineImageFormat::Jpeg => "jpeg".to_string(),
        InlineImageFormat::Gif => "gif".to_string(),
        InlineImageFormat::Bmp => "bmp".to_string(),
        InlineImageFormat::Webp => "webp".to_string(),
        InlineImageFormat::Svg => "svg".to_string(),
        InlineImageFormat::Other(other) => other.clone(),
    }
}

fn inline_image_source_to_str(source: &InlineImageSource) -> &'static str {
    match source {
        InlineImageSource::ImgDataUri => "img_data_uri",
        InlineImageSource::SvgElement => "svg_element",
    }
}

fn inline_image_to_py<'py>(py: Python<'py>, image: html_to_markdown_rs::InlineImage) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("data", PyBytes::new(py, &image.data))?;
    dict.set_item("format", inline_image_format_to_str(&image.format))?;

    match image.filename {
        Some(filename) => dict.set_item("filename", filename)?,
        None => dict.set_item("filename", py.None())?,
    }

    match image.description {
        Some(description) => dict.set_item("description", description)?,
        None => dict.set_item("description", py.None())?,
    }

    if let Some((width, height)) = image.dimensions {
        dict.set_item("dimensions", (width, height))?;
    } else {
        dict.set_item("dimensions", py.None())?;
    }

    dict.set_item("source", inline_image_source_to_str(&image.source))?;

    let attrs = PyDict::new(py);
    for (key, value) in image.attributes {
        attrs.set_item(key, value)?;
    }
    dict.set_item("attributes", attrs)?;

    Ok(dict.into())
}

fn warning_to_py<'py>(py: Python<'py>, warning: html_to_markdown_rs::InlineImageWarning) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("index", warning.index)?;
    dict.set_item("message", warning.message)?;
    Ok(dict.into())
}

/// Convert HTML to Markdown with inline image extraction.
///
/// Extracts embedded images (data URIs and inline SVG) during conversion.
///
/// Args:
///     html: HTML string to convert
///     options: Optional conversion configuration
///     image_config: Optional image extraction configuration
///
/// Returns:
///     Tuple of (markdown: str, images: List[dict], warnings: List[dict])
///
/// Raises:
///     ValueError: Invalid HTML or configuration
///
/// Example:
///     ```ignore
///     from html_to_markdown import convert_with_inline_images, InlineImageConfig
///
///     html = '<img src="data:image/png;base64,..." alt="Logo">'
///     config = InlineImageConfig(max_decoded_size_bytes=1024*1024)
///     markdown, images, warnings = convert_with_inline_images(html, image_config=config)
///
///     print(f"Found {len(images)} images")
///     for img in images:
///         print(f"Format: {img['format']}, Size: {len(img['data'])} bytes")
///     ```
#[pyfunction]
#[pyo3(signature = (html, options=None, image_config=None))]
fn convert_with_inline_images<'py>(
    py: Python<'py>,
    html: &str,
    options: Option<ConversionOptions>,
    image_config: Option<InlineImageConfig>,
) -> PyInlineExtraction {
    let rust_options = options.map(|opts| opts.to_rust());
    let cfg = image_config.unwrap_or_else(|| InlineImageConfig::new(5 * 1024 * 1024, None, true, false));
    let extraction = html_to_markdown_rs::convert_with_inline_images(html, rust_options, cfg.to_rust())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let images = extraction
        .inline_images
        .into_iter()
        .map(|image| inline_image_to_py(py, image))
        .collect::<PyResult<Vec<_>>>()?;

    let warnings = extraction
        .warnings
        .into_iter()
        .map(|warning| warning_to_py(py, warning))
        .collect::<PyResult<Vec<_>>>()?;

    Ok((extraction.markdown, images, warnings))
}

/// Python bindings for html-to-markdown
#[pymodule]
fn _html_to_markdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_with_options_handle, m)?)?;
    m.add_function(wrap_pyfunction!(create_options_handle, m)?)?;
    m.add_class::<ConversionOptions>()?;
    m.add_class::<PreprocessingOptions>()?;
    m.add_class::<ConversionOptionsHandle>()?;
    m.add_function(wrap_pyfunction!(convert_with_inline_images, m)?)?;
    m.add_class::<InlineImageConfig>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_returns_markdown() {
        let html = "<h1>Hello</h1>";
        let result = convert(html, None).expect("conversion succeeds");
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_conversion_options_defaults() {
        let opts = ConversionOptions::new(
            "underlined".to_string(),
            "spaces".to_string(),
            4,
            "*+-".to_string(),
            '*',
            false,
            false,
            false,
            false,
            "".to_string(),
            true,
            false,
            false,
            true,
            "double-equal".to_string(),
            true,
            "normalized".to_string(),
            false,
            false,
            80,
            false,
            "".to_string(),
            "".to_string(),
            "spaces".to_string(),
            "indented".to_string(),
            Vec::new(),
            None,
            false,
            Vec::new(),
            Vec::new(),
            "utf-8".to_string(),
        );
        let rust_opts = opts.to_rust();
        assert_eq!(rust_opts.list_indent_width, 4);
        assert_eq!(rust_opts.wrap_width, 80);
    }

    #[test]
    fn test_preprocessing_options_conversion() {
        let preprocessing = PreprocessingOptions::new(true, "aggressive".to_string(), true, false);
        let rust_preprocessing = preprocessing.to_rust();
        assert!(rust_preprocessing.enabled);
        assert!(matches!(
            rust_preprocessing.preset,
            html_to_markdown_rs::PreprocessingPreset::Aggressive
        ));
        assert!(rust_preprocessing.remove_navigation);
        assert!(!rust_preprocessing.remove_forms);
    }
}
