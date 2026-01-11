//! Shared utilities for html-to-markdown language bindings.
//!
//! This crate provides common functionality used across all language bindings
//! (Python, Node.js, Ruby, PHP, WebAssembly) to reduce code duplication and
//! ensure consistent behavior.
//!
//! # Modules
//!
//! - [`error`] - Error mapping patterns for different binding frameworks
//! - [`json`] - JSON parsing helpers for configuration objects
//! - [`enums`] - Shared enum definitions with serde support
//! - [`metadata`] - Metadata conversion utilities (when `metadata` feature enabled)
//! - [`inline_images`] - Inline image conversion utilities (when `inline-images` feature enabled)

#![allow(clippy::module_name_repetitions)]

pub mod enums;
pub mod error;
#[cfg(feature = "inline-images")]
pub mod inline_images;
pub mod json;
#[cfg(feature = "metadata")]
pub mod metadata;

// Re-export commonly used types
pub use enums::{
    CodeBlockStyleWrapper, HeadingStyleWrapper, HighlightStyleWrapper, ListIndentTypeWrapper, NewlineStyleWrapper,
    PreprocessingPresetWrapper, WhitespaceModeWrapper,
};
pub use error::BindingError;
pub use json::parse_conversion_options;
#[cfg(feature = "inline-images")]
pub use json::parse_inline_image_config;
#[cfg(feature = "metadata")]
pub use json::parse_metadata_config;

#[cfg(feature = "metadata")]
pub use metadata::{
    DocumentMetadataIntermediate, ExtendedMetadataIntermediate, HeaderMetadataIntermediate, ImageMetadataIntermediate,
    LinkMetadataIntermediate, StructuredDataIntermediate,
};

#[cfg(feature = "inline-images")]
pub use inline_images::InlineImageIntermediate;
