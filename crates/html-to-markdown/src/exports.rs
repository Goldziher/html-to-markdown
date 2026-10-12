//! Public API re-exports from submodules.
//!
//! This module centralizes all public type and function exports,
//! making the crate's public interface clear and organized.

pub use crate::error::{ConversionError, Result};

#[cfg(feature = "inline-images")]
pub use crate::inline_images::{
    DEFAULT_INLINE_IMAGE_LIMIT, HtmlExtraction, InlineImage, InlineImageConfig, InlineImageConfigUpdate,
    InlineImageFormat, InlineImageSource, InlineImageWarning,
};

#[cfg(feature = "metadata")]
pub use crate::metadata::{
    DEFAULT_MAX_STRUCTURED_DATA_SIZE, DocumentMetadata, HeaderMetadata, HtmlMetadata, ImageMetadata, ImageType,
    LinkMetadata, LinkType, MetadataConfig, MetadataConfigUpdate, StructuredData, StructuredDataType, TextDirection,
};

pub use crate::options::{
    CodeBlockStyle, ConversionOptions, ConversionOptionsBuilder, ConversionOptionsUpdate, DEFAULT_WASM_MAX_INPUT_SIZE,
    HeadingStyle, HiddenContent, HighlightStyle, InlineDataMedia, LinkStyle, ListIndentType, NewlineStyle,
    OutputFormat, PreprocessingOptions, PreprocessingOptionsUpdate, PreprocessingPreset, TierStrategy, UrlEscapeStyle,
    WhitespaceMode,
};
