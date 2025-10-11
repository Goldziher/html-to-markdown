use std::collections::BTreeMap;

use crate::error::ConversionError;

/// Configuration for capturing inline images during conversion.
#[derive(Debug, Clone)]
pub struct InlineImageConfig {
    /// Maximum allowed decoded size in bytes; larger payloads are rejected.
    pub max_decoded_size_bytes: u64,
    /// Optional prefix for generated filenames (defaults to "embedded_image").
    pub filename_prefix: Option<String>,
    /// Whether to capture inline SVG elements (defaults to true).
    pub capture_svg: bool,
    /// Whether to decode raster images to infer dimensions (defaults to false).
    pub infer_dimensions: bool,
}

impl InlineImageConfig {
    /// Create a new configuration with required maximum decoded size.
    pub fn new(max_decoded_size_bytes: u64) -> Self {
        Self {
            max_decoded_size_bytes,
            filename_prefix: None,
            capture_svg: true,
            infer_dimensions: false,
        }
    }
}

/// Supported inline image formats derived from the MIME subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Webp,
    Svg,
    Other(String),
}

/// Source of the inline image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineImageSource {
    ImgDataUri,
    SvgElement,
}

/// Information about an extracted inline image.
#[derive(Debug, Clone)]
pub struct InlineImage {
    pub data: Vec<u8>,
    pub format: InlineImageFormat,
    pub filename: Option<String>,
    pub description: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub source: InlineImageSource,
    pub attributes: BTreeMap<String, String>,
}

/// Human-friendly warning emitted during inline image extraction.
#[derive(Debug, Clone)]
pub struct InlineImageWarning {
    pub index: usize,
    pub message: String,
}

/// Output of `convert_with_inline_images`.
#[derive(Debug, Clone)]
pub struct HtmlExtraction {
    pub markdown: String,
    pub inline_images: Vec<InlineImage>,
    pub warnings: Vec<InlineImageWarning>,
}

/// Internal collector that maintains inline image state during traversal.
#[derive(Debug)]
pub(crate) struct InlineImageCollector {
    config: InlineImageConfig,
    prefix: String,
    next_index: usize,
    images: Vec<InlineImage>,
    warnings: Vec<InlineImageWarning>,
}

impl InlineImageCollector {
    pub(crate) fn new(config: InlineImageConfig) -> Result<Self, ConversionError> {
        if config.max_decoded_size_bytes == 0 {
            return Err(ConversionError::ConfigError(
                "inline image max_decoded_size_bytes must be greater than zero".to_string(),
            ));
        }

        let prefix = config
            .filename_prefix
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("embedded_image")
            .to_string();

        Ok(Self {
            config,
            prefix,
            next_index: 0,
            images: Vec::new(),
            warnings: Vec::new(),
        })
    }

    pub(crate) fn capture_svg(&self) -> bool {
        self.config.capture_svg
    }

    pub(crate) fn should_infer_dimensions(&self) -> bool {
        self.config.infer_dimensions
    }

    pub(crate) fn max_decoded_size(&self) -> u64 {
        self.config.max_decoded_size_bytes
    }

    pub(crate) fn next_index(&mut self) -> usize {
        self.next_index += 1;
        self.next_index
    }

    pub(crate) fn finalize_filename(&self, provided: Option<&str>, index: usize, format: &InlineImageFormat) -> String {
        if let Some(name) = provided {
            return name.to_string();
        }

        let extension = match format {
            InlineImageFormat::Png => "png",
            InlineImageFormat::Jpeg => "jpeg",
            InlineImageFormat::Gif => "gif",
            InlineImageFormat::Bmp => "bmp",
            InlineImageFormat::Webp => "webp",
            InlineImageFormat::Svg => "svg",
            // ~keep: Split on MIME type delimiters (+, ., ;) to extract base subtype
            // e.g., "svg+xml" -> "svg", "vnd.foo.bar" -> "vnd"
            InlineImageFormat::Other(custom) => custom
                .split(['+', '.', ';'])
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("bin"),
        };

        format!("{}_{}.{}", self.prefix, index, extension)
    }

    pub(crate) fn warn_skip(&mut self, index: usize, reason: impl Into<String>) {
        let message = format!("Skipped inline image {}: {}", index, reason.into());
        self.warnings.push(InlineImageWarning { index, message });
    }

    pub(crate) fn warn_info(&mut self, index: usize, reason: impl Into<String>) {
        let message = format!("Inline image {}: {}", index, reason.into());
        self.warnings.push(InlineImageWarning { index, message });
    }

    pub(crate) fn push_image(&mut self, index: usize, mut image: InlineImage) {
        if image.filename.is_none() {
            let derived = self.finalize_filename(None, index, &image.format);
            image.filename = Some(derived);
        }
        self.images.push(image);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_image(
        &self,
        data: Vec<u8>,
        format: InlineImageFormat,
        filename: Option<String>,
        description: Option<String>,
        dimensions: Option<(u32, u32)>,
        source: InlineImageSource,
        attributes: BTreeMap<String, String>,
    ) -> InlineImage {
        InlineImage {
            data,
            format,
            filename,
            description,
            dimensions,
            source,
            attributes,
        }
    }

    pub(crate) fn infer_dimensions(
        &mut self,
        index: usize,
        data: &[u8],
        format: &InlineImageFormat,
    ) -> Option<(u32, u32)> {
        if !self.should_infer_dimensions() {
            return None;
        }

        match format {
            InlineImageFormat::Svg | InlineImageFormat::Other(_) => return None,
            _ => {}
        }

        match image::load_from_memory(data) {
            Ok(img) => Some((img.width(), img.height())),
            Err(err) => {
                self.warn_info(
                    index,
                    format!("unable to decode raster data for dimension inference ({err})"),
                );
                None
            }
        }
    }

    pub(crate) fn finish(self) -> (Vec<InlineImage>, Vec<InlineImageWarning>) {
        (self.images, self.warnings)
    }
}
