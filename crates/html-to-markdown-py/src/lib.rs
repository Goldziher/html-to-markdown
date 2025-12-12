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
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
#[cfg(feature = "metadata")]
use pyo3::types::{PyList, PyTuple};

fn to_py_err(err: ConversionError) -> PyErr {
    match err {
        ConversionError::Panic(message) => {
            pyo3::exceptions::PyRuntimeError::new_err(format!("html-to-markdown panic during conversion: {message}"))
        }
        other => pyo3::exceptions::PyValueError::new_err(other.to_string()),
    }
}

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
    guard_panic(|| html_to_markdown_rs::convert(html, rust_options)).map_err(to_py_err)
}

#[pyfunction]
#[pyo3(signature = (html, handle))]
fn convert_with_options_handle(html: &str, handle: &ConversionOptionsHandle) -> PyResult<String> {
    guard_panic(|| html_to_markdown_rs::convert(html, Some(handle.inner.clone()))).map_err(to_py_err)
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
    let extraction = guard_panic(|| html_to_markdown_rs::convert_with_inline_images(html, rust_options, cfg.to_rust()))
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
        Some(RustTextDirection::LeftToRight) => pyo3::types::PyString::new(py, "ltr").into(),
        Some(RustTextDirection::RightToLeft) => pyo3::types::PyString::new(py, "rtl").into(),
        Some(RustTextDirection::Auto) => pyo3::types::PyString::new(py, "auto").into(),
        None => py.None(),
    }
}

#[cfg(feature = "metadata")]
fn link_type_to_str(link_type: &RustLinkType) -> &'static str {
    match link_type {
        RustLinkType::Anchor => "anchor",
        RustLinkType::Internal => "internal",
        RustLinkType::External => "external",
        RustLinkType::Email => "email",
        RustLinkType::Phone => "phone",
        RustLinkType::Other => "other",
    }
}

#[cfg(feature = "metadata")]
fn image_type_to_str(image_type: &RustImageType) -> &'static str {
    match image_type {
        RustImageType::DataUri => "data_uri",
        RustImageType::InlineSvg => "inline_svg",
        RustImageType::External => "external",
        RustImageType::Relative => "relative",
    }
}

#[cfg(feature = "metadata")]
fn structured_data_type_to_str(data_type: &RustStructuredDataType) -> &'static str {
    match data_type {
        RustStructuredDataType::JsonLd => "json_ld",
        RustStructuredDataType::Microdata => "microdata",
        RustStructuredDataType::RDFa => "rdfa",
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
        dict.set_item("link_type", link_type_to_str(&link.link_type))?;
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
                let tuple = PyTuple::new(py, &[width, height])?;
                tuple.into()
            }
            None => py.None(),
        };
        dict.set_item("dimensions", dims)?;

        dict.set_item("image_type", image_type_to_str(&image.image_type))?;
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
        dict.set_item("data_type", structured_data_type_to_str(&item.data_type))?;
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
    let rust_options = options.map(|opts| opts.to_rust());
    let cfg = metadata_config
        .unwrap_or_else(|| MetadataConfig::new(true, true, true, true, true, DEFAULT_MAX_STRUCTURED_DATA_SIZE));
    let result = guard_panic(|| html_to_markdown_rs::convert_with_metadata(html, rust_options, cfg.to_rust()))
        .map_err(to_py_err)?;

    let (markdown, metadata) = result;
    let metadata_dict = extended_metadata_to_py(py, metadata)?;
    Ok((markdown, metadata_dict))
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
    #[cfg(feature = "metadata")]
    {
        m.add_function(wrap_pyfunction!(convert_with_metadata, m)?)?;
        m.add_class::<MetadataConfig>()?;
    }
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
