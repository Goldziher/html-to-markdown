//! Error types for HTML to Markdown conversion.

use thiserror::Error;

/// Result type for conversion operations.
pub type Result<T> = std::result::Result<T, ConversionError>;

/// Errors that can occur during HTML to Markdown conversion.
#[derive(Error, Debug)]
pub enum ConversionError {
    /// HTML parsing error
    #[error("HTML parsing error: {0}")]
    ParseError(String),

    /// HTML sanitization error
    #[error("Sanitization error: {0}")]
    SanitizationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Panic caught during conversion to prevent unwinding across FFI boundaries
    #[error("Internal panic: {0}")]
    Panic(String),

    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Visitor callback error
    #[cfg(feature = "visitor")]
    #[error("Visitor error: {0}")]
    Visitor(String),

    /// Generic conversion error
    #[error("Conversion error: {0}")]
    Other(String),
}
