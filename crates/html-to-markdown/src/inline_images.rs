#![allow(clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::unused_self)]
use std::collections::BTreeMap;

use crate::error::ConversionError;

/// Configuration for capturing inline images during conversion.
#[derive(Debug, Clone)]
pub struct InlineImageConfig {
    /// Maximum allowed decoded size in bytes; larger payloads are rejected.
    pub max_decoded_size_bytes: u64,
    /// Optional prefix for generated filenames (defaults to "`embedded_image`").
    pub filename_prefix: Option<String>,
    /// Whether to capture inline SVG elements (defaults to true).
    pub capture_svg: bool,
    /// Whether to decode raster images to infer dimensions (defaults to false).
    pub infer_dimensions: bool,
}

/// Default maximum size for inline image extraction (5 MB).
pub const DEFAULT_INLINE_IMAGE_LIMIT: u64 = 5 * 1024 * 1024;

/// Partial update for `InlineImageConfig`.
///
/// This struct uses `Option<T>` to represent optional fields that can be selectively updated.
/// Only specified fields (Some values) will override existing options; None values leave the
/// corresponding fields unchanged when applied via [`InlineImageConfig::apply_update`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(any(feature = "serde", feature = "metadata"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "serde", feature = "metadata"), serde(rename_all = "camelCase"))]
pub struct InlineImageConfigUpdate {
    /// Optional maximum decoded size override in bytes.
    pub max_decoded_size_bytes: Option<u64>,
    /// Optional filename prefix override for generated filenames.
    pub filename_prefix: Option<String>,
    /// Optional inline SVG capture enablement override.
    pub capture_svg: Option<bool>,
    /// Optional dimension inference override for raster images.
    pub infer_dimensions: Option<bool>,
}

impl InlineImageConfig {
    /// Create a new configuration with required maximum decoded size.
    #[must_use]
    pub const fn new(max_decoded_size_bytes: u64) -> Self {
        Self {
            max_decoded_size_bytes,
            filename_prefix: None,
            capture_svg: true,
            infer_dimensions: false,
        }
    }

    /// Apply a partial update to this inline image configuration.
    ///
    /// Any specified fields in the update will override the current values.
    /// Unspecified fields (None) are left unchanged.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial inline image options update with fields to override
    pub fn apply_update(&mut self, update: InlineImageConfigUpdate) {
        if let Some(max_decoded_size_bytes) = update.max_decoded_size_bytes {
            self.max_decoded_size_bytes = max_decoded_size_bytes;
        }
        if let Some(filename_prefix) = update.filename_prefix {
            self.filename_prefix = Some(filename_prefix);
        }
        if let Some(capture_svg) = update.capture_svg {
            self.capture_svg = capture_svg;
        }
        if let Some(infer_dimensions) = update.infer_dimensions {
            self.infer_dimensions = infer_dimensions;
        }
    }

    /// Create new inline image configuration from a partial update.
    ///
    /// Creates a new `InlineImageConfig` struct with defaults, then applies the update.
    /// Fields not specified in the update keep their default values.
    ///
    /// # Arguments
    ///
    /// * `update` - Partial inline image options update with fields to set
    ///
    /// # Returns
    ///
    /// New `InlineImageConfig` with specified updates applied to defaults
    #[must_use]
    pub fn from_update(update: InlineImageConfigUpdate) -> Self {
        let mut config = Self::new(DEFAULT_INLINE_IMAGE_LIMIT);
        config.apply_update(update);
        config
    }
}

/// Supported inline image formats derived from the MIME subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineImageFormat {
    /// PNG (Portable Network Graphics) raster image format.
    Png,
    /// JPEG (Joint Photographic Experts Group) raster image format.
    Jpeg,
    /// GIF (Graphics Interchange Format) raster image format.
    Gif,
    /// BMP (Bitmap) raster image format.
    Bmp,
    /// WebP modern raster image format.
    Webp,
    /// SVG (Scalable Vector Graphics) vector format.
    Svg,
    /// Custom or unrecognized image format; contains the MIME subtype.
    Other(String),
}

impl std::fmt::Display for InlineImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png => write!(f, "png"),
            Self::Jpeg => write!(f, "jpeg"),
            Self::Gif => write!(f, "gif"),
            Self::Bmp => write!(f, "bmp"),
            Self::Webp => write!(f, "webp"),
            Self::Svg => write!(f, "svg"),
            Self::Other(custom) => write!(f, "{custom}"),
        }
    }
}

/// Source of the inline image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineImageSource {
    /// Image sourced from an `<img>` tag's `data:` URI.
    ImgDataUri,
    /// Image sourced from an inline `<svg>` element.
    SvgElement,
}

impl std::fmt::Display for InlineImageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImgDataUri => write!(f, "img_data_uri"),
            Self::SvgElement => write!(f, "svg_element"),
        }
    }
}

/// Information about an extracted inline image.
#[derive(Debug, Clone)]
pub struct InlineImage {
    /// Raw image data as bytes (encoded in its original format).
    pub data: Vec<u8>,
    /// Detected or inferred image format.
    pub format: InlineImageFormat,
    /// Generated or extracted filename for the image.
    pub filename: Option<String>,
    /// Alt text or other descriptive metadata from the source HTML.
    pub description: Option<String>,
    /// Image dimensions in pixels (width, height); only present if inferred.
    pub dimensions: Option<(u32, u32)>,
    /// Where the image originated (data URI or SVG element).
    pub source: InlineImageSource,
    /// Additional HTML attributes from the source element.
    pub attributes: BTreeMap<String, String>,
}

/// Human-friendly warning emitted during inline image extraction.
#[derive(Debug, Clone)]
pub struct InlineImageWarning {
    /// The index of the image that triggered the warning.
    pub index: usize,
    /// Descriptive message explaining the warning or issue.
    pub message: String,
}

/// Output of `convert_with_inline_images`.
#[derive(Debug, Clone)]
pub struct HtmlExtraction {
    /// Converted markdown output.
    pub markdown: String,
    /// Extracted inline images found in the HTML.
    pub inline_images: Vec<InlineImage>,
    /// Non-fatal warnings encountered during extraction.
    pub warnings: Vec<InlineImageWarning>,
}

/// Internal collector that maintains inline image state during traversal.
#[derive(Debug)]
pub struct InlineImageCollector {
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

    pub(crate) const fn capture_svg(&self) -> bool {
        self.config.capture_svg
    }

    pub(crate) const fn should_infer_dimensions(&self) -> bool {
        self.config.infer_dimensions
    }

    pub(crate) const fn max_decoded_size(&self) -> u64 {
        self.config.max_decoded_size_bytes
    }

    pub(crate) const fn next_index(&mut self) -> usize {
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
    pub(crate) const fn build_image(
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
