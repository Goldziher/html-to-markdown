use html_to_markdown_rs::{InlineImageConfig as RustInlineImageConfig, InlineImageFormat, InlineImageSource};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Inline image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct WasmInlineImageConfig {
    max_decoded_size_bytes: u64,
    filename_prefix: Option<String>,
    capture_svg: bool,
    infer_dimensions: bool,
}

#[wasm_bindgen]
impl WasmInlineImageConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(max_decoded_size_bytes: Option<f64>) -> Self {
        Self {
            max_decoded_size_bytes: max_decoded_size_bytes.map(|n| n as u64).unwrap_or(5 * 1024 * 1024),
            filename_prefix: None,
            capture_svg: true,
            infer_dimensions: false,
        }
    }

    #[wasm_bindgen(setter, js_name = "filenamePrefix")]
    pub fn set_filename_prefix(&mut self, prefix: Option<String>) {
        self.filename_prefix = prefix;
    }

    #[wasm_bindgen(setter, js_name = "captureSvg")]
    pub fn set_capture_svg(&mut self, capture: bool) {
        self.capture_svg = capture;
    }

    #[wasm_bindgen(setter, js_name = "inferDimensions")]
    pub fn set_infer_dimensions(&mut self, infer: bool) {
        self.infer_dimensions = infer;
    }
}

impl From<WasmInlineImageConfig> for RustInlineImageConfig {
    fn from(val: WasmInlineImageConfig) -> Self {
        let mut cfg = RustInlineImageConfig::new(val.max_decoded_size_bytes);
        cfg.filename_prefix = val.filename_prefix;
        cfg.capture_svg = val.capture_svg;
        cfg.infer_dimensions = val.infer_dimensions;
        cfg
    }
}

/// Inline image data
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmInlineImage {
    data: Vec<u8>,
    format: String,
    filename: Option<String>,
    description: Option<String>,
    dimensions: Option<(u32, u32)>,
    source: String,
    attributes: HashMap<String, String>,
}

#[wasm_bindgen]
impl WasmInlineImage {
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Uint8Array {
        Uint8Array::from(&self.data[..])
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn filename(&self) -> Option<String> {
        self.filename.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> Option<Vec<u32>> {
        self.dimensions.map(|(w, h)| vec![w, h])
    }

    #[wasm_bindgen(getter)]
    pub fn source(&self) -> String {
        self.source.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn attributes(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.attributes).unwrap_or(JsValue::NULL)
    }
}

impl WasmInlineImage {
    pub fn from_rust(img: html_to_markdown_rs::InlineImage) -> Self {
        Self {
            data: img.data,
            format: format_to_string(&img.format),
            filename: img.filename,
            description: img.description,
            dimensions: img.dimensions,
            source: source_to_string(&img.source),
            attributes: img.attributes.into_iter().collect(),
        }
    }
}

/// Warning about inline image processing
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmInlineImageWarning {
    index: usize,
    message: String,
}

#[wasm_bindgen]
impl WasmInlineImageWarning {
    #[wasm_bindgen(getter)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<html_to_markdown_rs::InlineImageWarning> for WasmInlineImageWarning {
    fn from(val: html_to_markdown_rs::InlineImageWarning) -> Self {
        Self {
            index: val.index,
            message: val.message,
        }
    }
}

/// Result of HTML extraction with inline images
#[wasm_bindgen]
pub struct WasmHtmlExtraction {
    markdown: String,
    inline_images: Vec<WasmInlineImage>,
    warnings: Vec<WasmInlineImageWarning>,
}

#[wasm_bindgen]
impl WasmHtmlExtraction {
    #[wasm_bindgen(getter)]
    pub fn markdown(&self) -> String {
        self.markdown.clone()
    }

    #[wasm_bindgen(getter, js_name = "inlineImages")]
    pub fn inline_images(&self) -> Vec<WasmInlineImage> {
        self.inline_images.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<WasmInlineImageWarning> {
        self.warnings.clone()
    }
}

impl From<html_to_markdown_rs::HtmlExtraction> for WasmHtmlExtraction {
    fn from(val: html_to_markdown_rs::HtmlExtraction) -> Self {
        Self {
            markdown: val.markdown,
            inline_images: val.inline_images.into_iter().map(WasmInlineImage::from_rust).collect(),
            warnings: val.warnings.into_iter().map(Into::into).collect(),
        }
    }
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
