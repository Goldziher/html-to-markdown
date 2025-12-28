#[cfg(feature = "metadata")]
use html_to_markdown_rs::metadata::{
    DEFAULT_MAX_STRUCTURED_DATA_SIZE, DocumentMetadata as RustDocumentMetadata,
    ExtendedMetadata as RustExtendedMetadata, HeaderMetadata as RustHeaderMetadata, ImageMetadata as RustImageMetadata,
    LinkMetadata as RustLinkMetadata, MetadataConfig as RustMetadataConfig, StructuredData as RustStructuredData,
    TextDirection as RustTextDirection,
};
use html_to_markdown_rs::safety::guard_panic;
mod profiling;
#[cfg(feature = "visitor")]
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{
    CodeBlockStyle, ConversionError, ConversionOptions as RustConversionOptions, HeadingStyle, HighlightStyle,
    ListIndentType, NewlineStyle, PreprocessingOptions as RustPreprocessingOptions, PreprocessingPreset,
    WhitespaceMode,
};
#[cfg(feature = "inline-images")]
use html_to_markdown_rs::{DEFAULT_INLINE_IMAGE_LIMIT, InlineImageConfig as RustInlineImageConfig};
#[cfg(feature = "async-visitor")]
use once_cell::sync::OnceCell;
use pyo3::prelude::*;
#[cfg(feature = "inline-images")]
use pyo3::types::PyBytes;
#[cfg(any(feature = "inline-images", feature = "metadata"))]
use pyo3::types::PyDict;
#[cfg(feature = "metadata")]
use pyo3::types::{PyList, PyTuple};
#[cfg(feature = "async-visitor")]
use pyo3_async_runtimes::TaskLocals;
#[cfg(feature = "visitor")]
use std::cell::RefCell;
use std::panic::UnwindSafe;
use std::path::PathBuf;
#[cfg(feature = "visitor")]
use std::rc::Rc;
#[cfg(feature = "async-visitor")]
use std::sync::Arc;
#[cfg(feature = "async-visitor")]
use std::sync::Mutex;

fn to_py_err(err: ConversionError) -> PyErr {
    match err {
        ConversionError::Panic(message) => {
            pyo3::exceptions::PyRuntimeError::new_err(format!("html-to-markdown panic during conversion: {message}"))
        }
        other => pyo3::exceptions::PyValueError::new_err(other.to_string()),
    }
}

fn run_with_guard_and_profile<F, T>(f: F) -> html_to_markdown_rs::Result<T>
where
    F: FnMut() -> html_to_markdown_rs::Result<T> + UnwindSafe,
{
    guard_panic(|| profiling::maybe_profile(f))
}

#[cfg(feature = "async-visitor")]
pub static PYTHON_TASK_LOCALS: OnceCell<TaskLocals> = OnceCell::new();

#[cfg(feature = "async-visitor")]
fn init_python_event_loop(_py: Python) -> PyResult<()> {
    if PYTHON_TASK_LOCALS.get().is_some() {
        return Ok(());
    }

    let (tx, rx) = std::sync::mpsc::channel::<PyResult<TaskLocals>>();

    std::thread::spawn(move || {
        let result = Python::attach(|py| -> PyResult<()> {
            let asyncio = py.import("asyncio")?;
            let event_loop = asyncio.call_method0("new_event_loop")?;
            asyncio.call_method1("set_event_loop", (event_loop.clone(),))?;

            let locals = TaskLocals::new(event_loop.clone()).copy_context(py)?;
            let _ = tx.send(Ok(locals));

            event_loop.call_method0("run_forever")?;
            Ok(())
        });

        if let Err(err) = result {
            let _ = tx.send(Err(err));
        }
    });

    let task_locals = rx
        .recv()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to init async event loop"))??;

    PYTHON_TASK_LOCALS
        .set(task_locals)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Python async context already initialized"))?;

    Ok(())
}

fn parse_options_json(options_json: Option<&str>) -> PyResult<Option<RustConversionOptions>> {
    let Some(json) = options_json else {
        return Ok(None);
    };

    if json.trim().is_empty() {
        return Ok(None);
    }

    let options = html_to_markdown_rs::conversion_options_from_json(json).map_err(to_py_err)?;
    Ok(Some(options))
}

#[cfg(feature = "inline-images")]
fn parse_inline_image_config_json(config_json: Option<&str>) -> PyResult<RustInlineImageConfig> {
    let Some(json) = config_json else {
        return Ok(RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    };

    if json.trim().is_empty() {
        return Ok(RustInlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT));
    }

    html_to_markdown_rs::inline_image_config_from_json(json).map_err(to_py_err)
}

#[cfg(feature = "metadata")]
fn parse_metadata_config_json(config_json: Option<&str>) -> PyResult<RustMetadataConfig> {
    let Some(json) = config_json else {
        return Ok(RustMetadataConfig::default());
    };

    if json.trim().is_empty() {
        return Ok(RustMetadataConfig::default());
    }

    html_to_markdown_rs::metadata_config_from_json(json).map_err(to_py_err)
}

#[pyfunction]
fn start_profiling(output_path: &str, frequency: Option<i32>) -> PyResult<()> {
    let path = PathBuf::from(output_path);
    let freq = frequency.unwrap_or(1000);
    profiling::start(path, freq).map_err(to_py_err)?;
    Ok(())
}

#[pyfunction]
fn stop_profiling() -> PyResult<()> {
    profiling::stop().map_err(to_py_err)?;
    Ok(())
}

#[cfg(feature = "inline-images")]
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
#[cfg(feature = "inline-images")]
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

#[cfg(feature = "inline-images")]
#[pymethods]
impl InlineImageConfig {
    #[new]
    #[pyo3(signature = (
        max_decoded_size_bytes=DEFAULT_INLINE_IMAGE_LIMIT,
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

#[cfg(feature = "inline-images")]
impl InlineImageConfig {
    fn to_rust(&self) -> RustInlineImageConfig {
        let mut cfg = RustInlineImageConfig::new(self.max_decoded_size_bytes);
        cfg.filename_prefix = self.filename_prefix.clone();
        cfg.capture_svg = self.capture_svg;
        cfg.infer_dimensions = self.infer_dimensions;
        cfg
    }
}

/// Python wrapper for metadata extraction configuration
#[cfg(feature = "metadata")]
#[pyclass]
#[derive(Clone)]
struct MetadataConfig {
    #[pyo3(get, set)]
    extract_document: bool,
    #[pyo3(get, set)]
    extract_headers: bool,
    #[pyo3(get, set)]
    extract_links: bool,
    #[pyo3(get, set)]
    extract_images: bool,
    #[pyo3(get, set)]
    extract_structured_data: bool,
    #[pyo3(get, set)]
    max_structured_data_size: usize,
}

#[cfg(feature = "metadata")]
#[pymethods]
impl MetadataConfig {
    #[new]
    #[pyo3(signature = (
        extract_document=true,
        extract_headers=true,
        extract_links=true,
        extract_images=true,
        extract_structured_data=true,
        max_structured_data_size=DEFAULT_MAX_STRUCTURED_DATA_SIZE
    ))]
    fn new(
        extract_document: bool,
        extract_headers: bool,
        extract_links: bool,
        extract_images: bool,
        extract_structured_data: bool,
        max_structured_data_size: usize,
    ) -> Self {
        Self {
            extract_document,
            extract_headers,
            extract_links,
            extract_images,
            extract_structured_data,
            max_structured_data_size,
        }
    }
}

#[cfg(feature = "metadata")]
impl MetadataConfig {
    fn to_rust(&self) -> RustMetadataConfig {
        RustMetadataConfig {
            extract_document: self.extract_document,
            extract_headers: self.extract_headers,
            extract_links: self.extract_links,
            extract_images: self.extract_images,
            extract_structured_data: self.extract_structured_data,
            max_structured_data_size: self.max_structured_data_size,
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
            heading_style: HeadingStyle::parse(self.heading_style.as_str()),
            list_indent_type: ListIndentType::parse(self.list_indent_type.as_str()),
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
            highlight_style: HighlightStyle::parse(self.highlight_style.as_str()),
            extract_metadata: self.extract_metadata,
            whitespace_mode: WhitespaceMode::parse(self.whitespace_mode.as_str()),
            strip_newlines: self.strip_newlines,
            wrap: self.wrap,
            wrap_width: self.wrap_width,
            convert_as_inline: self.convert_as_inline,
            sub_symbol: self.sub_symbol.clone(),
            sup_symbol: self.sup_symbol.clone(),
            newline_style: NewlineStyle::parse(self.newline_style.as_str()),
            code_block_style: CodeBlockStyle::parse(self.code_block_style.as_str()),
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

    fn new_with_rust(options: RustConversionOptions) -> Self {
        Self { inner: options }
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
fn convert(py: Python<'_>, html: &str, options: Option<ConversionOptions>) -> PyResult<String> {
    let html = html.to_owned();
    let rust_options = options.map(|opts| opts.to_rust());
    py.detach(move || run_with_guard_and_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
        .map_err(to_py_err)
}

#[pyfunction]
#[pyo3(signature = (html, options_json=None))]
fn convert_json(py: Python<'_>, html: &str, options_json: Option<&str>) -> PyResult<String> {
    let html = html.to_owned();
    let rust_options = parse_options_json(options_json)?;
    py.detach(move || run_with_guard_and_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
        .map_err(to_py_err)
}

#[pyfunction]
#[pyo3(signature = (html, handle))]
fn convert_with_options_handle(py: Python<'_>, html: &str, handle: &ConversionOptionsHandle) -> PyResult<String> {
    let html = html.to_owned();
    let rust_options = handle.inner.clone();
    py.detach(move || run_with_guard_and_profile(|| html_to_markdown_rs::convert(&html, Some(rust_options.clone()))))
        .map_err(to_py_err)
}

#[pyfunction]
#[pyo3(signature = (options=None))]
fn create_options_handle(options: Option<ConversionOptions>) -> ConversionOptionsHandle {
    ConversionOptionsHandle::new_with_options(options)
}

#[pyfunction]
#[pyo3(signature = (options_json=None))]
fn create_options_handle_json(options_json: Option<&str>) -> PyResult<ConversionOptionsHandle> {
    let rust_options = parse_options_json(options_json)?.unwrap_or_default();
    Ok(ConversionOptionsHandle::new_with_rust(rust_options))
}

#[cfg(feature = "inline-images")]
fn inline_image_to_py<'py>(py: Python<'py>, image: html_to_markdown_rs::InlineImage) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("data", PyBytes::new(py, &image.data))?;
    dict.set_item("format", image.format.to_string())?;

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

    dict.set_item("source", image.source.to_string())?;

    let attrs = PyDict::new(py);
    for (key, value) in image.attributes {
        attrs.set_item(key, value)?;
    }
    dict.set_item("attributes", attrs)?;

    Ok(dict.into())
}

#[cfg(feature = "inline-images")]
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
#[cfg(feature = "inline-images")]
#[pyfunction]
#[pyo3(signature = (html, options=None, image_config=None))]
fn convert_with_inline_images<'py>(
    py: Python<'py>,
    html: &str,
    options: Option<ConversionOptions>,
    image_config: Option<InlineImageConfig>,
) -> PyInlineExtraction {
    let html = html.to_owned();
    let rust_options = options.map(|opts| opts.to_rust());
    let cfg = image_config.unwrap_or_else(|| InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT, None, true, false));
    let rust_cfg = cfg.to_rust();
    let extraction = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_inline_images(&html, rust_options.clone(), rust_cfg.clone())
            })
        })
        .map_err(to_py_err)?;

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

#[cfg(feature = "inline-images")]
#[pyfunction]
#[pyo3(signature = (html, options_json=None, image_config_json=None))]
fn convert_with_inline_images_json<'py>(
    py: Python<'py>,
    html: &str,
    options_json: Option<&str>,
    image_config_json: Option<&str>,
) -> PyInlineExtraction {
    let html = html.to_owned();
    let rust_options = parse_options_json(options_json)?;
    let rust_config = parse_inline_image_config_json(image_config_json)?;
    let extraction = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_inline_images(&html, rust_options.clone(), rust_config.clone())
            })
        })
        .map_err(to_py_err)?;

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

/// Convert HTML to Markdown with inline images using a pre-parsed options handle.
#[cfg(feature = "inline-images")]
#[pyfunction]
#[pyo3(signature = (html, handle, image_config=None))]
fn convert_with_inline_images_handle<'py>(
    py: Python<'py>,
    html: &str,
    handle: &ConversionOptionsHandle,
    image_config: Option<InlineImageConfig>,
) -> PyInlineExtraction {
    let html = html.to_owned();
    let rust_options = Some(handle.inner.clone());
    let cfg = image_config.unwrap_or_else(|| InlineImageConfig::new(DEFAULT_INLINE_IMAGE_LIMIT, None, true, false));
    let rust_cfg = cfg.to_rust();
    let extraction = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_inline_images(&html, rust_options.clone(), rust_cfg.clone())
            })
        })
        .map_err(to_py_err)?;

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

#[cfg(feature = "metadata")]
fn opt_string_to_py<'py>(py: Python<'py>, opt: Option<String>) -> PyResult<Py<PyAny>> {
    match opt {
        Some(val) => {
            let str_obj = pyo3::types::PyString::new(py, &val);
            Ok(str_obj.into())
        }
        None => Ok(py.None()),
    }
}

#[cfg(feature = "metadata")]
fn btreemap_to_py<'py>(py: Python<'py>, map: std::collections::BTreeMap<String, String>) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    for (k, v) in map {
        dict.set_item(k, v)?;
    }
    Ok(dict.into())
}

#[cfg(feature = "metadata")]
fn text_direction_to_str<'py>(py: Python<'py>, text_direction: Option<RustTextDirection>) -> Py<PyAny> {
    match text_direction {
        Some(direction) => pyo3::types::PyString::new(py, &direction.to_string())
            .as_any()
            .to_owned()
            .into(),
        None => py.None(),
    }
}

#[cfg(feature = "metadata")]
fn document_metadata_to_py<'py>(py: Python<'py>, doc: RustDocumentMetadata) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);

    dict.set_item("title", opt_string_to_py(py, doc.title)?)?;
    dict.set_item("description", opt_string_to_py(py, doc.description)?)?;
    dict.set_item("keywords", doc.keywords)?;
    dict.set_item("author", opt_string_to_py(py, doc.author)?)?;
    dict.set_item("canonical_url", opt_string_to_py(py, doc.canonical_url)?)?;
    dict.set_item("base_href", opt_string_to_py(py, doc.base_href)?)?;
    dict.set_item("language", opt_string_to_py(py, doc.language)?)?;
    dict.set_item("text_direction", text_direction_to_str(py, doc.text_direction))?;
    dict.set_item("open_graph", btreemap_to_py(py, doc.open_graph)?)?;
    dict.set_item("twitter_card", btreemap_to_py(py, doc.twitter_card)?)?;
    dict.set_item("meta_tags", btreemap_to_py(py, doc.meta_tags)?)?;

    Ok(dict.into())
}

#[cfg(feature = "metadata")]
fn headers_to_py<'py>(py: Python<'py>, headers: Vec<RustHeaderMetadata>) -> PyResult<Py<PyAny>> {
    let list = PyList::empty(py);
    for header in headers {
        let dict = PyDict::new(py);
        dict.set_item("level", header.level)?;
        dict.set_item("text", header.text)?;
        dict.set_item("id", opt_string_to_py(py, header.id)?)?;
        dict.set_item("depth", header.depth)?;
        dict.set_item("html_offset", header.html_offset)?;
        list.append(dict)?;
    }
    Ok(list.into())
}

#[cfg(feature = "metadata")]
fn links_to_py<'py>(py: Python<'py>, links: Vec<RustLinkMetadata>) -> PyResult<Py<PyAny>> {
    let list = PyList::empty(py);
    for link in links {
        let dict = PyDict::new(py);
        dict.set_item("href", link.href)?;
        dict.set_item("text", link.text)?;
        dict.set_item("title", opt_string_to_py(py, link.title)?)?;
        dict.set_item("link_type", link.link_type.to_string())?;
        dict.set_item("rel", link.rel)?;
        dict.set_item("attributes", btreemap_to_py(py, link.attributes)?)?;
        list.append(dict)?;
    }
    Ok(list.into())
}

#[cfg(feature = "metadata")]
fn images_to_py<'py>(py: Python<'py>, images: Vec<RustImageMetadata>) -> PyResult<Py<PyAny>> {
    let list = PyList::empty(py);
    for image in images {
        let dict = PyDict::new(py);
        dict.set_item("src", image.src)?;
        dict.set_item("alt", opt_string_to_py(py, image.alt)?)?;
        dict.set_item("title", opt_string_to_py(py, image.title)?)?;

        let dims = match image.dimensions {
            Some((width, height)) => {
                let tuple = PyTuple::new(py, [width, height])?;
                tuple.into()
            }
            None => py.None(),
        };
        dict.set_item("dimensions", dims)?;

        dict.set_item("image_type", image.image_type.to_string())?;
        dict.set_item("attributes", btreemap_to_py(py, image.attributes)?)?;
        list.append(dict)?;
    }
    Ok(list.into())
}

#[cfg(feature = "metadata")]
fn structured_data_to_py<'py>(py: Python<'py>, data: Vec<RustStructuredData>) -> PyResult<Py<PyAny>> {
    let list = PyList::empty(py);
    for item in data {
        let dict = PyDict::new(py);
        dict.set_item("data_type", item.data_type.to_string())?;
        dict.set_item("raw_json", item.raw_json)?;
        dict.set_item("schema_type", opt_string_to_py(py, item.schema_type)?)?;
        list.append(dict)?;
    }
    Ok(list.into())
}

#[cfg(feature = "metadata")]
fn extended_metadata_to_py<'py>(py: Python<'py>, metadata: RustExtendedMetadata) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("document", document_metadata_to_py(py, metadata.document)?)?;
    dict.set_item("headers", headers_to_py(py, metadata.headers)?)?;
    dict.set_item("links", links_to_py(py, metadata.links)?)?;
    dict.set_item("images", images_to_py(py, metadata.images)?)?;
    dict.set_item("structured_data", structured_data_to_py(py, metadata.structured_data)?)?;
    Ok(dict.into())
}

/// Convert HTML to Markdown with comprehensive metadata extraction.
///
/// Performs HTML-to-Markdown conversion while simultaneously extracting structured metadata
/// including document properties, headers, links, images, and structured data in a single pass.
/// Ideal for content analysis, SEO workflows, and document indexing.
///
/// Args:
///     html (str): HTML string to convert. Line endings are normalized (CRLF -> LF).
///     options (ConversionOptions, optional): Conversion configuration controlling output format.
///         Defaults to standard conversion options if None. Controls:
///         - heading_style: "atx", "atx_closed", or "underlined"
///         - list_indent_type: "spaces" or "tabs"
///         - wrap: Enable text wrapping at specified width
///         - And many other formatting options
///     metadata_config (MetadataConfig, optional): Configuration for metadata extraction.
///         Defaults to extracting all metadata types if None. Configure with:
///         - extract_headers: bool - Extract h1-h6 heading elements
///         - extract_links: bool - Extract hyperlinks with type classification
///         - extract_images: bool - Extract image elements
///         - extract_structured_data: bool - Extract JSON-LD/Microdata/RDFa
///         - max_structured_data_size: int - Size limit for structured data (bytes)
///
/// Returns:
///     tuple[str, dict]: A tuple of (markdown_string, metadata_dict) where:
///
///     markdown_string: str
///         The converted Markdown output
///
///     metadata_dict: dict with keys:
///         - document: dict containing:
///             - title: str | None - Document title from <title> tag
///             - description: str | None - From <meta name="description">
///             - keywords: list[str] - Keywords from <meta name="keywords">
///             - author: str | None - Author from <meta name="author">
///             - language: str | None - Language from lang attribute
///             - text_direction: str | None - Text direction ("ltr", "rtl", "auto")
///             - canonical_url: str | None - Canonical URL from <link rel="canonical">
///             - base_href: str | None - Base URL from <base href="">
///             - open_graph: dict[str, str] - Open Graph properties (og:*)
///             - twitter_card: dict[str, str] - Twitter Card properties (twitter:*)
///             - meta_tags: dict[str, str] - Other meta tags
///
///         - headers: list[dict] containing:
///             - level: int - Header level (1-6)
///             - text: str - Header text content
///             - id: str | None - HTML id attribute
///             - depth: int - Nesting depth in document tree
///             - html_offset: int - Byte offset in original HTML
///
///         - links: list[dict] containing:
///             - href: str - Link URL
///             - text: str - Link text content
///             - title: str | None - Link title attribute
///             - link_type: str - Type: "anchor", "internal", "external", "email", "phone", "other"
///             - rel: list[str] - Rel attribute values
///             - attributes: dict[str, str] - Additional HTML attributes
///
///         - images: list[dict] containing:
///             - src: str - Image source (URL or data URI)
///             - alt: str | None - Alt text for accessibility
///             - title: str | None - Title attribute
///             - dimensions: tuple[int, int] | None - (width, height) if available
///             - image_type: str - Type: "data_uri", "external", "relative", "inline_svg"
///             - attributes: dict[str, str] - Additional HTML attributes
///
///         - structured_data: list[dict] containing:
///             - data_type: str - Type: "json_ld", "microdata", or "rdfa"
///             - raw_json: str - Raw JSON string content
///             - schema_type: str | None - Schema type (e.g., "Article", "Event")
///
/// Raises:
///     ValueError: If HTML parsing fails or configuration is invalid
///     RuntimeError: If a panic occurs during conversion
///
/// Examples:
///
///     Basic usage - extract all metadata:
///
///     ```ignore
///     from html_to_markdown import convert_with_metadata, MetadataConfig
///
///     html = '''
///     <html lang="en">
///         <head>
///             <title>My Blog Post</title>
///             <meta name="description" content="A great article">
///         </head>
///         <body>
///             <h1 id="intro">Introduction</h1>
///             <p>Read more at <a href="https://example.com">our site</a></p>
///             <img src="photo.jpg" alt="Beautiful landscape">
///         </body>
///     </html>
///     '''
///
///     markdown, metadata = convert_with_metadata(html)
///
///     print(f"Title: {metadata['document']['title']}")
///     # Output: Title: My Blog Post
///
///     print(f"Language: {metadata['document']['language']}")
///     # Output: Language: en
///
///     print(f"Headers found: {len(metadata['headers'])}")
///     # Output: Headers found: 1
///
///     for header in metadata['headers']:
///         print(f"  - {header['text']} (level {header['level']})")
///     # Output:   - Introduction (level 1)
///
///     print(f"External links: {len([l for l in metadata['links'] if l['link_type'] == 'external'])}")
///     # Output: External links: 1
///
///     for img in metadata['images']:
///         print(f"Image: {img['alt']} ({img['src']})")
///     # Output: Image: Beautiful landscape (photo.jpg)
///     ```
///
///     Selective metadata extraction - headers and links only:
///
///     ```ignore
///     from html_to_markdown import convert_with_metadata, MetadataConfig
///
///     config = MetadataConfig(
///         extract_headers=True,
///         extract_links=True,
///         extract_images=False,  # Skip image extraction
///         extract_structured_data=False  # Skip structured data
///     )
///
///     markdown, metadata = convert_with_metadata(html, metadata_config=config)
///
///     assert len(metadata['images']) == 0  # Images not extracted
///     assert len(metadata['headers']) > 0  # Headers extracted
///     ```
///
///     With custom conversion options:
///
///     ```ignore
///     from html_to_markdown import convert_with_metadata, ConversionOptions, MetadataConfig
///
///     options = ConversionOptions(
///         heading_style="atx",  # Use # H1, ## H2 style
///         wrap=True,
///         wrap_width=80
///     )
///
///     config = MetadataConfig(extract_headers=True)
///
///     markdown, metadata = convert_with_metadata(html, options=options, metadata_config=config)
///     # Markdown uses ATX-style headings and is wrapped at 80 chars
///     ```
///
/// See Also:
///     - convert: Simple HTML to Markdown conversion without metadata
///     - convert_with_inline_images: Extract inline images alongside conversion
///     - ConversionOptions: Conversion configuration class
///     - MetadataConfig: Metadata extraction configuration class
#[cfg(feature = "metadata")]
#[pyfunction]
#[pyo3(signature = (html, options=None, metadata_config=None))]
fn convert_with_metadata<'py>(
    py: Python<'py>,
    html: &str,
    options: Option<ConversionOptions>,
    metadata_config: Option<MetadataConfig>,
) -> PyResult<(String, Py<PyAny>)> {
    let html = html.to_owned();
    let rust_options = options.map(|opts| opts.to_rust());
    let cfg = metadata_config
        .unwrap_or_else(|| MetadataConfig::new(true, true, true, true, true, DEFAULT_MAX_STRUCTURED_DATA_SIZE));
    let rust_cfg = cfg.to_rust();
    let result = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_metadata(&html, rust_options.clone(), rust_cfg.clone())
            })
        })
        .map_err(to_py_err)?;

    let (markdown, metadata) = result;
    let metadata_dict = extended_metadata_to_py(py, metadata)?;
    Ok((markdown, metadata_dict))
}

#[cfg(feature = "metadata")]
#[pyfunction]
#[pyo3(signature = (html, options_json=None, metadata_config_json=None))]
fn convert_with_metadata_json(
    py: Python<'_>,
    html: &str,
    options_json: Option<&str>,
    metadata_config_json: Option<&str>,
) -> PyResult<(String, Py<PyAny>)> {
    let html = html.to_owned();
    let rust_options = parse_options_json(options_json)?;
    let rust_cfg = parse_metadata_config_json(metadata_config_json)?;

    let result = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_metadata(&html, rust_options.clone(), rust_cfg.clone())
            })
        })
        .map_err(to_py_err)?;

    let (markdown, metadata) = result;
    let metadata_dict = extended_metadata_to_py(py, metadata)?;
    Ok((markdown, metadata_dict))
}

/// Convert HTML to Markdown with metadata using a pre-parsed options handle.
#[cfg(feature = "metadata")]
#[pyfunction]
#[pyo3(signature = (html, handle, metadata_config=None))]
fn convert_with_metadata_handle<'py>(
    py: Python<'py>,
    html: &str,
    handle: &ConversionOptionsHandle,
    metadata_config: Option<MetadataConfig>,
) -> PyResult<(String, Py<PyAny>)> {
    let html = html.to_owned();
    let rust_options = Some(handle.inner.clone());
    let cfg = metadata_config
        .unwrap_or_else(|| MetadataConfig::new(true, true, true, true, true, DEFAULT_MAX_STRUCTURED_DATA_SIZE));
    let rust_cfg = cfg.to_rust();
    let result = py
        .detach(move || {
            run_with_guard_and_profile(|| {
                html_to_markdown_rs::convert_with_metadata(&html, rust_options.clone(), rust_cfg.clone())
            })
        })
        .map_err(to_py_err)?;

    let (markdown, metadata) = result;
    let metadata_dict = extended_metadata_to_py(py, metadata)?;
    Ok((markdown, metadata_dict))
}

#[cfg(feature = "visitor")]
mod visitor_support {
    use super::*;

    /// PyO3 wrapper around a Python visitor object.
    ///
    /// This struct bridges Python callbacks to the Rust HtmlVisitor trait.
    /// It holds a reference to a Python object and calls its methods dynamically.
    #[derive(Debug)]
    pub struct PyVisitorBridge {
        pub visitor: Py<PyAny>,
    }

    impl PyVisitorBridge {
        /// Create a new bridge wrapping a Python visitor object.
        pub fn new(visitor: Py<PyAny>) -> Self {
            Self { visitor }
        }

        /// Convert a Python dictionary result to a VisitResult enum.
        fn result_from_dict(result_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<VisitResult> {
            let result_type: String = result_dict
                .get_item("type")?
                .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Visitor result dict must have 'type' key"))?
                .extract()?;

            match result_type.as_str() {
                "continue" => Ok(VisitResult::Continue),
                "skip" => Ok(VisitResult::Skip),
                "preserve_html" => Ok(VisitResult::PreserveHtml),
                "custom" => {
                    let output: String = result_dict
                        .get_item("output")?
                        .ok_or_else(|| {
                            pyo3::exceptions::PyTypeError::new_err("Visitor 'custom' result must have 'output' key")
                        })?
                        .extract()?;
                    Ok(VisitResult::Custom(output))
                }
                "error" => {
                    let message: String = result_dict
                        .get_item("message")?
                        .ok_or_else(|| {
                            pyo3::exceptions::PyTypeError::new_err("Visitor 'error' result must have 'message' key")
                        })?
                        .extract()?;
                    Ok(VisitResult::Error(message))
                }
                unknown => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown visitor result type: {}",
                    unknown
                ))),
            }
        }

        /// Convert NodeContext to a Python dictionary.
        fn context_to_dict<'a>(py: Python<'a>, ctx: &NodeContext) -> PyResult<Bound<'a, pyo3::types::PyDict>> {
            let dict = pyo3::types::PyDict::new(py);

            let node_type_str = format!("{:?}", ctx.node_type).to_lowercase();
            dict.set_item("node_type", node_type_str)?;

            dict.set_item("tag_name", &ctx.tag_name)?;

            let attrs_dict = pyo3::types::PyDict::new(py);
            for (k, v) in &ctx.attributes {
                attrs_dict.set_item(k, v)?;
            }
            dict.set_item("attributes", attrs_dict)?;

            dict.set_item("depth", ctx.depth)?;

            dict.set_item("index_in_parent", ctx.index_in_parent)?;

            match &ctx.parent_tag {
                Some(tag) => dict.set_item("parent_tag", tag)?,
                None => dict.set_item("parent_tag", py.None())?,
            }

            dict.set_item("is_inline", ctx.is_inline)?;

            Ok(dict)
        }

        /// Call a Python visitor method and convert the result.
        fn call_visitor_method(
            &self,
            py: Python<'_>,
            method_name: &str,
            args: &[Bound<'_, PyAny>],
        ) -> PyResult<VisitResult> {
            let visitor_bound = self.visitor.bind(py);
            let method = match visitor_bound.getattr(method_name) {
                Ok(m) => m,
                Err(_) => {
                    return Ok(VisitResult::Continue);
                }
            };

            let args_tuple = pyo3::types::PyTuple::new(py, args)?;

            let result = method.call(args_tuple, None)?;

            if result.is_none() {
                return Ok(VisitResult::Continue);
            }

            if result.hasattr("__await__")? {
                let locals = PYTHON_TASK_LOCALS.get().ok_or_else(|| {
                    pyo3::exceptions::PyRuntimeError::new_err("Async visitor event loop not initialized")
                })?;

                let fut = pyo3_async_runtimes::into_future_with_locals(locals, result)?;
                let py_result: Py<PyAny> = py.detach(|| pyo3_async_runtimes::tokio::get_runtime().block_on(fut))?;
                let result_dict: Bound<'_, pyo3::types::PyDict> = py_result.bind(py).extract()?;
                return Self::result_from_dict(&result_dict);
            }

            let result_dict: Bound<'_, pyo3::types::PyDict> = result.extract()?;

            Self::result_from_dict(&result_dict)
        }
    }

    impl HtmlVisitor for PyVisitorBridge {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_element_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method(py, "visit_element_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_text", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let href_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, href).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let title_py: Bound<'_, PyAny> = match title {
                    Some(t) => pyo3::types::PyString::new(py, t).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), href_py, text_py, title_py];
                self.call_visitor_method(py, "visit_link", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, src).as_any().clone();
                let alt_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, alt).as_any().clone();
                let title_py: Bound<'_, PyAny> = match title {
                    Some(t) => pyo3::types::PyString::new(py, t).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py, alt_py, title_py];
                self.call_visitor_method(py, "visit_image", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let level_py: Bound<'_, PyAny> = pyo3::types::PyInt::new(py, level).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let id_py: Bound<'_, PyAny> = match id {
                    Some(i) => pyo3::types::PyString::new(py, i).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), level_py, text_py, id_py];
                self.call_visitor_method(py, "visit_heading", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_code_block(&mut self, ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let lang_py: Bound<'_, PyAny> = match lang {
                    Some(l) => pyo3::types::PyString::new(py, l).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let code_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, code).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), lang_py, code_py];
                self.call_visitor_method(py, "visit_code_block", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_code_inline(&mut self, ctx: &NodeContext, code: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let code_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, code).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), code_py];
                self.call_visitor_method(py, "visit_code_inline", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_item(&mut self, ctx: &NodeContext, ordered: bool, marker: &str, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let marker_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, marker).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py, marker_py, text_py];
                self.call_visitor_method(py, "visit_list_item", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_start(&mut self, ctx: &NodeContext, ordered: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py];
                self.call_visitor_method(py, "visit_list_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_end(&mut self, ctx: &NodeContext, ordered: bool, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py, output_py];
                self.call_visitor_method(py, "visit_list_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_table_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let cells_py: Bound<'_, PyAny> = match pyo3::types::PyList::new(py, cells) {
                    Ok(list) => list.as_any().clone(),
                    Err(_) => return VisitResult::Continue,
                };
                let is_header_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, is_header).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), cells_py, is_header_py];
                self.call_visitor_method(py, "visit_table_row", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method(py, "visit_table_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_blockquote(&mut self, ctx: &NodeContext, content: &str, depth: usize) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let content_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, content).as_any().clone();
                let depth_py: Bound<'_, PyAny> = pyo3::types::PyInt::new(py, depth).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), content_py, depth_py];
                self.call_visitor_method(py, "visit_blockquote", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_strong", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_emphasis(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_emphasis", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_strikethrough", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_underline(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_underline", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_subscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_subscript", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_superscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_superscript", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_mark(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_mark", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_line_break(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_line_break", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_horizontal_rule(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_horizontal_rule", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_custom_element(&mut self, ctx: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let tag_name_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, tag_name).as_any().clone();
                let html_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, html).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), tag_name_py, html_py];
                self.call_visitor_method(py, "visit_custom_element", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_list_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_definition_list_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_term(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_definition_term", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_description(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_definition_description", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_list_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method(py, "visit_definition_list_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_form(&mut self, ctx: &NodeContext, action: Option<&str>, method: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let action_py: Bound<'_, PyAny> = match action {
                    Some(a) => pyo3::types::PyString::new(py, a).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let method_py: Bound<'_, PyAny> = match method {
                    Some(m) => pyo3::types::PyString::new(py, m).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), action_py, method_py];
                self.call_visitor_method(py, "visit_form", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_input(
            &mut self,
            ctx: &NodeContext,
            input_type: &str,
            name: Option<&str>,
            value: Option<&str>,
        ) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let input_type_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, input_type).as_any().clone();
                let name_py: Bound<'_, PyAny> = match name {
                    Some(n) => pyo3::types::PyString::new(py, n).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let value_py: Bound<'_, PyAny> = match value {
                    Some(v) => pyo3::types::PyString::new(py, v).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), input_type_py, name_py, value_py];
                self.call_visitor_method(py, "visit_input", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_button(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_button", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_audio(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method(py, "visit_audio", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_video(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method(py, "visit_video", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_iframe(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method(py, "visit_iframe", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_details(&mut self, ctx: &NodeContext, open: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let open_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, open).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), open_py];
                self.call_visitor_method(py, "visit_details", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_summary(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_summary", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figure_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method(py, "visit_figure_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figcaption(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method(py, "visit_figcaption", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figure_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method(py, "visit_figure_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }
    }

    #[cfg(feature = "async-visitor")]
    #[derive(Debug)]
    pub struct PyAsyncVisitorBridge {
        pub visitor: Arc<Mutex<Py<PyAny>>>,
    }

    #[cfg(feature = "async-visitor")]
    impl PyAsyncVisitorBridge {
        pub fn new(visitor: Py<PyAny>) -> Self {
            Self {
                visitor: Arc::new(Mutex::new(visitor)),
            }
        }

        /// Convert a Python dictionary result to a VisitResult enum.
        fn result_from_dict(result_dict: &Bound<'_, pyo3::types::PyDict>) -> PyResult<VisitResult> {
            let result_type: String = result_dict
                .get_item("type")?
                .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Visitor result dict must have 'type' key"))?
                .extract()?;

            match result_type.as_str() {
                "continue" => Ok(VisitResult::Continue),
                "skip" => Ok(VisitResult::Skip),
                "preserve_html" => Ok(VisitResult::PreserveHtml),
                "custom" => {
                    let output: String = result_dict
                        .get_item("output")?
                        .ok_or_else(|| {
                            pyo3::exceptions::PyTypeError::new_err("Visitor 'custom' result must have 'output' key")
                        })?
                        .extract()?;
                    Ok(VisitResult::Custom(output))
                }
                "error" => {
                    let message: String = result_dict
                        .get_item("message")?
                        .ok_or_else(|| {
                            pyo3::exceptions::PyTypeError::new_err("Visitor 'error' result must have 'message' key")
                        })?
                        .extract()?;
                    Ok(VisitResult::Error(message))
                }
                unknown => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown visitor result type: {}",
                    unknown
                ))),
            }
        }

        /// Convert NodeContext to a Python dictionary.
        fn context_to_dict<'a>(py: Python<'a>, ctx: &NodeContext) -> PyResult<Bound<'a, pyo3::types::PyDict>> {
            let dict = pyo3::types::PyDict::new(py);

            let node_type_str = format!("{:?}", ctx.node_type).to_lowercase();
            dict.set_item("node_type", node_type_str)?;

            dict.set_item("tag_name", &ctx.tag_name)?;

            let attrs_dict = pyo3::types::PyDict::new(py);
            for (k, v) in &ctx.attributes {
                attrs_dict.set_item(k, v)?;
            }
            dict.set_item("attributes", attrs_dict)?;

            dict.set_item("depth", ctx.depth)?;

            dict.set_item("index_in_parent", ctx.index_in_parent)?;

            match &ctx.parent_tag {
                Some(tag) => dict.set_item("parent_tag", tag)?,
                None => dict.set_item("parent_tag", py.None())?,
            }

            dict.set_item("is_inline", ctx.is_inline)?;

            Ok(dict)
        }

        /// Call a Python visitor method and convert the result (supports both sync and async).
        fn call_visitor_method_sync(
            &self,
            py: Python<'_>,
            method_name: &str,
            args: &[Bound<'_, PyAny>],
        ) -> PyResult<VisitResult> {
            let visitor_guard = self.visitor.lock().unwrap();
            let visitor_bound = visitor_guard.bind(py);
            let method = match visitor_bound.getattr(method_name) {
                Ok(m) => m,
                Err(_) => {
                    return Ok(VisitResult::Continue);
                }
            };

            let args_tuple = pyo3::types::PyTuple::new(py, args)?;

            let result = method.call(args_tuple, None)?;

            if result.is_none() {
                return Ok(VisitResult::Continue);
            }

            let result_dict: Bound<'_, pyo3::types::PyDict> = result.extract()?;

            Self::result_from_dict(&result_dict)
        }
    }

    #[cfg(feature = "async-visitor")]
    impl HtmlVisitor for PyAsyncVisitorBridge {
        fn visit_element_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_element_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_element_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method_sync(py, "visit_element_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_text(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_text", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_link(&mut self, ctx: &NodeContext, href: &str, text: &str, title: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let href_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, href).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let title_py: Bound<'_, PyAny> = match title {
                    Some(t) => pyo3::types::PyString::new(py, t).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), href_py, text_py, title_py];
                self.call_visitor_method_sync(py, "visit_link", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_image(&mut self, ctx: &NodeContext, src: &str, alt: &str, title: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, src).as_any().clone();
                let alt_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, alt).as_any().clone();
                let title_py: Bound<'_, PyAny> = match title {
                    Some(t) => pyo3::types::PyString::new(py, t).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py, alt_py, title_py];
                self.call_visitor_method_sync(py, "visit_image", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_heading(&mut self, ctx: &NodeContext, level: u32, text: &str, id: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let level_py: Bound<'_, PyAny> = pyo3::types::PyInt::new(py, level).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let id_py: Bound<'_, PyAny> = match id {
                    Some(i) => pyo3::types::PyString::new(py, i).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), level_py, text_py, id_py];
                self.call_visitor_method_sync(py, "visit_heading", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_code_block(&mut self, ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let lang_py: Bound<'_, PyAny> = match lang {
                    Some(l) => pyo3::types::PyString::new(py, l).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let code_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, code).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), lang_py, code_py];
                self.call_visitor_method_sync(py, "visit_code_block", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_code_inline(&mut self, ctx: &NodeContext, code: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let code_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, code).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), code_py];
                self.call_visitor_method_sync(py, "visit_code_inline", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_item(&mut self, ctx: &NodeContext, ordered: bool, marker: &str, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let marker_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, marker).as_any().clone();
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py, marker_py, text_py];
                self.call_visitor_method_sync(py, "visit_list_item", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_start(&mut self, ctx: &NodeContext, ordered: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py];
                self.call_visitor_method_sync(py, "visit_list_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_list_end(&mut self, ctx: &NodeContext, ordered: bool, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let ordered_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, ordered).as_any().clone();
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), ordered_py, output_py];
                self.call_visitor_method_sync(py, "visit_list_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_table_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_row(&mut self, ctx: &NodeContext, cells: &[String], is_header: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let cells_py: Bound<'_, PyAny> = match pyo3::types::PyList::new(py, cells) {
                    Ok(list) => list.as_any().clone(),
                    Err(_) => return VisitResult::Continue,
                };
                let is_header_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, is_header).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), cells_py, is_header_py];
                self.call_visitor_method_sync(py, "visit_table_row", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_table_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method_sync(py, "visit_table_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_blockquote(&mut self, ctx: &NodeContext, content: &str, depth: usize) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let content_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, content).as_any().clone();
                let depth_py: Bound<'_, PyAny> = pyo3::types::PyInt::new(py, depth).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), content_py, depth_py];
                self.call_visitor_method_sync(py, "visit_blockquote", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_strong(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_strong", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_emphasis(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_emphasis", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_strikethrough(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_strikethrough", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_underline(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_underline", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_subscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_subscript", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_superscript(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_superscript", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_mark(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_mark", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_line_break(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_line_break", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_horizontal_rule(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_horizontal_rule", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_custom_element(&mut self, ctx: &NodeContext, tag_name: &str, html: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let tag_name_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, tag_name).as_any().clone();
                let html_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, html).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), tag_name_py, html_py];
                self.call_visitor_method_sync(py, "visit_custom_element", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_list_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_definition_list_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_term(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_definition_term", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_description(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_definition_description", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_definition_list_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method_sync(py, "visit_definition_list_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_form(&mut self, ctx: &NodeContext, action: Option<&str>, method: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let action_py: Bound<'_, PyAny> = match action {
                    Some(a) => pyo3::types::PyString::new(py, a).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let method_py: Bound<'_, PyAny> = match method {
                    Some(m) => pyo3::types::PyString::new(py, m).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), action_py, method_py];
                self.call_visitor_method_sync(py, "visit_form", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_input(
            &mut self,
            ctx: &NodeContext,
            input_type: &str,
            name: Option<&str>,
            value: Option<&str>,
        ) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let input_type_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, input_type).as_any().clone();
                let name_py: Bound<'_, PyAny> = match name {
                    Some(n) => pyo3::types::PyString::new(py, n).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let value_py: Bound<'_, PyAny> = match value {
                    Some(v) => pyo3::types::PyString::new(py, v).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), input_type_py, name_py, value_py];
                self.call_visitor_method_sync(py, "visit_input", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_button(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_button", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_audio(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method_sync(py, "visit_audio", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_video(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method_sync(py, "visit_video", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_iframe(&mut self, ctx: &NodeContext, src: Option<&str>) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let src_py: Bound<'_, PyAny> = match src {
                    Some(s) => pyo3::types::PyString::new(py, s).as_any().clone(),
                    None => py.None().bind(py).clone(),
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), src_py];
                self.call_visitor_method_sync(py, "visit_iframe", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_details(&mut self, ctx: &NodeContext, open: bool) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let open_py: Bound<'_, PyAny> = pyo3::types::PyBool::new(py, open).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), open_py];
                self.call_visitor_method_sync(py, "visit_details", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_summary(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_summary", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figure_start(&mut self, ctx: &NodeContext) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone()];
                self.call_visitor_method_sync(py, "visit_figure_start", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figcaption(&mut self, ctx: &NodeContext, text: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let text_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, text).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), text_py];
                self.call_visitor_method_sync(py, "visit_figcaption", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }

        fn visit_figure_end(&mut self, ctx: &NodeContext, output: &str) -> VisitResult {
            Python::attach(|py| {
                let ctx_dict = match Self::context_to_dict(py, ctx) {
                    Ok(d) => d,
                    Err(_) => return VisitResult::Continue,
                };
                let output_py: Bound<'_, PyAny> = pyo3::types::PyString::new(py, output).as_any().clone();
                let args: Vec<Bound<'_, PyAny>> = vec![ctx_dict.as_any().clone(), output_py];
                self.call_visitor_method_sync(py, "visit_figure_end", &args)
                    .unwrap_or(VisitResult::Continue)
            })
        }
    }
}

/// Convert HTML to Markdown with a custom visitor.
#[cfg(feature = "visitor")]
#[pyfunction]
#[pyo3(signature = (html, options=None, visitor=None))]
fn convert_with_visitor(
    py: Python<'_>,
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<Py<PyAny>>,
) -> PyResult<String> {
    let html = html.to_owned();
    let rust_options = options.map(|opts| opts.to_rust());

    let Some(visitor_py) = visitor else {
        return py
            .detach(move || run_with_guard_and_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
            .map_err(to_py_err);
    };

    let bridge = visitor_support::PyVisitorBridge::new(visitor_py);
    let visitor_handle = std::sync::Arc::new(std::sync::Mutex::new(bridge));

    py.detach(move || {
        run_with_guard_and_profile(|| {
            let rc_visitor: Rc<RefCell<dyn HtmlVisitor>> = {
                Python::attach(|py| {
                    let guard = visitor_handle.lock().unwrap();
                    let bridge_copy = visitor_support::PyVisitorBridge::new(guard.visitor.clone_ref(py));
                    Rc::new(RefCell::new(bridge_copy)) as Rc<RefCell<dyn HtmlVisitor>>
                })
            };
            html_to_markdown_rs::convert_with_visitor(&html, rust_options.clone(), Some(rc_visitor))
        })
    })
    .map_err(to_py_err)
}

/// Convert HTML to Markdown with a custom visitor (async-compatible version).
///
/// This function provides async-compatible support for visitor methods using pyo3-async-runtimes
/// with proper event loop management. Supports both synchronous and asynchronous visitor methods.
///
/// The visitor object should define callback methods that return dictionaries:
/// - `def visit_text(ctx, text)`: Called for text nodes (can be async)
/// - `def visit_link(ctx, href, text, title)`: Called for links (can be async)
/// - And many others...
///
/// Each method should return a dict (or coroutine) with a 'type' key:
/// - `{"type": "continue"}` - Continue with default conversion
/// - `{"type": "skip"}` - Skip this element
/// - `{"type": "preserve_html"}` - Preserve original HTML
/// - `{"type": "custom", "output": "markdown"}` - Custom markdown output
/// - `{"type": "error", "message": "error"}` - Stop with error
///
/// Example:
///     ```ignore
///     from html_to_markdown import convert_with_async_visitor
///     import asyncio
///
///     class MyAsyncVisitor:
///         async def visit_text(self, ctx, text):
///             # Can perform async operations here
///             result = await some_async_operation()
///             return {"type": "continue"}
///
///     html = "<h1>Hello</h1><p>World</p>"
///     markdown = await convert_with_async_visitor(html, visitor=MyAsyncVisitor())
///     ```
#[cfg(feature = "async-visitor")]
#[pyfunction]
#[pyo3(signature = (html, options=None, visitor=None))]
fn convert_with_async_visitor(
    py: Python<'_>,
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<Py<PyAny>>,
) -> PyResult<String> {
    init_python_event_loop(py)?;

    let html = html.to_owned();
    let rust_options = options.map(|opts| opts.to_rust());

    let Some(visitor_py) = visitor else {
        return py
            .detach(move || run_with_guard_and_profile(|| html_to_markdown_rs::convert(&html, rust_options.clone())))
            .map_err(to_py_err);
    };

    let visitor_handle = std::sync::Arc::new(std::sync::Mutex::new(visitor_py));

    py.detach(move || {
        run_with_guard_and_profile(|| {
            let rc_visitor: Rc<RefCell<dyn HtmlVisitor>> = {
                Python::attach(|py| {
                    let guard = visitor_handle.lock().unwrap();
                    let bridge_copy = visitor_support::PyAsyncVisitorBridge::new(guard.clone_ref(py));
                    Rc::new(RefCell::new(bridge_copy)) as Rc<RefCell<dyn HtmlVisitor>>
                })
            };
            html_to_markdown_rs::convert_with_visitor(&html, rust_options.clone(), Some(rc_visitor))
        })
    })
    .map_err(to_py_err)
}

/// Fallback for when async-visitor feature is not enabled
#[cfg(all(feature = "visitor", not(feature = "async-visitor")))]
#[pyfunction]
#[pyo3(signature = (html, options=None, visitor=None))]
fn convert_with_async_visitor(
    py: Python<'_>,
    html: &str,
    options: Option<ConversionOptions>,
    visitor: Option<Py<PyAny>>,
) -> PyResult<String> {
    convert_with_visitor(py, html, options, visitor)
}

/// Python bindings for html-to-markdown
#[pymodule]
fn _html_to_markdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_json, m)?)?;
    m.add_function(wrap_pyfunction!(convert_with_options_handle, m)?)?;
    m.add_function(wrap_pyfunction!(create_options_handle, m)?)?;
    m.add_function(wrap_pyfunction!(create_options_handle_json, m)?)?;
    m.add_class::<ConversionOptions>()?;
    m.add_class::<PreprocessingOptions>()?;
    m.add_class::<ConversionOptionsHandle>()?;
    #[cfg(feature = "inline-images")]
    {
        m.add_function(wrap_pyfunction!(convert_with_inline_images, m)?)?;
        m.add_function(wrap_pyfunction!(convert_with_inline_images_json, m)?)?;
        m.add_function(wrap_pyfunction!(convert_with_inline_images_handle, m)?)?;
        m.add_class::<InlineImageConfig>()?;
    }
    #[cfg(feature = "metadata")]
    {
        m.add_function(wrap_pyfunction!(convert_with_metadata, m)?)?;
        m.add_function(wrap_pyfunction!(convert_with_metadata_json, m)?)?;
        m.add_function(wrap_pyfunction!(convert_with_metadata_handle, m)?)?;
        m.add_class::<MetadataConfig>()?;
    }
    #[cfg(feature = "visitor")]
    {
        m.add_function(wrap_pyfunction!(convert_with_visitor, m)?)?;
        m.add_function(wrap_pyfunction!(convert_with_async_visitor, m)?)?;
    }
    m.add_function(wrap_pyfunction!(start_profiling, m)?)?;
    m.add_function(wrap_pyfunction!(stop_profiling, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_returns_markdown() {
        Python::initialize();
        Python::attach(|py| -> PyResult<()> {
            let html = "<h1>Hello</h1>";
            let result = convert(py, html, None)?;
            assert!(result.contains("Hello"));
            Ok(())
        })
        .expect("conversion succeeds");
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
