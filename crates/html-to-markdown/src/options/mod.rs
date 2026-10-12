//! Configuration options for HTML to Markdown conversion.
//!
//! This module provides comprehensive configuration options for customizing
//! HTML to Markdown conversion behavior, including output formatting, preprocessing,
//! and metadata extraction options.

pub mod conversion;
pub mod inline_image;
pub mod preprocessing;
pub mod validation;

pub use conversion::{
    ConversionOptions, ConversionOptionsBuilder, ConversionOptionsUpdate, DEFAULT_WASM_MAX_INPUT_SIZE, TierStrategy,
};
pub use preprocessing::{PreprocessingOptions, PreprocessingOptionsUpdate, PreprocessingPreset};
pub use validation::{
    CodeBlockStyle, HeadingStyle, HiddenContent, HighlightStyle, InlineDataMedia, LinkStyle, ListIndentType,
    NewlineStyle, OutputFormat, UrlEscapeStyle, WhitespaceMode,
};
