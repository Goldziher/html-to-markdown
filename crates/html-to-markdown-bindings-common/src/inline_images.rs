//! Inline image conversion utilities for language bindings.
//!
//! Provides intermediate representations for inline image data that can
//! be easily converted to language-specific structures.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Intermediate representation for inline image data.
///
/// This structure is designed to be easily converted to language-specific
/// types like PyDict, JsObject, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineImageIntermediate {
    /// The image identifier/key
    pub key: String,
    /// Raw image data (as bytes)
    pub data: Vec<u8>,
    /// Image format (e.g., "png", "jpg", "svg")
    pub format: String,
    /// Image width in pixels (if known)
    pub width: Option<u32>,
    /// Image height in pixels (if known)
    pub height: Option<u32>,
    /// Additional attributes from the HTML
    pub attributes: BTreeMap<String, String>,
}

/// Intermediate representation for inline image warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineImageWarningIntermediate {
    /// The image source that caused the warning
    pub source: String,
    /// Warning message
    pub message: String,
}

/// Result type for inline image extraction.
///
/// Contains the markdown output, extracted images, and any warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineImageExtractionResult {
    /// The markdown output
    pub markdown: String,
    /// Extracted inline images
    pub images: Vec<InlineImageIntermediate>,
    /// Warnings encountered during extraction
    pub warnings: Vec<InlineImageWarningIntermediate>,
}
