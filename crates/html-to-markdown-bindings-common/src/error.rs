//! Error mapping patterns for language bindings.
//!
//! Provides a trait for mapping `ConversionError` from the core library
//! to language-specific error types.

use html_to_markdown_rs::ConversionError;

/// Trait for converting `ConversionError` to binding-specific error types.
///
/// Each language binding should implement this trait for their error type
/// (PyErr, napi::Error, magnus::Error, etc.).
///
/// # Example
///
/// ```ignore
/// impl BindingError for PyErr {
///     fn from_conversion_error(err: ConversionError) -> Self {
///         match err {
///             ConversionError::Panic(msg) => {
///                 pyo3::exceptions::PyRuntimeError::new_err(
///                     format!("html-to-markdown panic: {}", msg)
///                 )
///             }
///             other => pyo3::exceptions::PyValueError::new_err(other.to_string()),
///         }
///     }
/// }
/// ```
pub trait BindingError: Sized {
    /// Convert a `ConversionError` to the binding's error type.
    ///
    /// Typically maps:
    /// - `ConversionError::Panic` → Runtime/panic error
    /// - Other variants → Standard error with message
    fn from_conversion_error(err: ConversionError) -> Self;
}

/// Helper function to categorize conversion errors.
///
/// Returns `true` if the error is a panic/runtime error that should
/// be treated differently from validation errors.
#[must_use]
pub fn is_panic_error(err: &ConversionError) -> bool {
    matches!(err, ConversionError::Panic(_))
}

/// Extract error message from `ConversionError`.
#[must_use]
pub fn error_message(err: &ConversionError) -> String {
    match err {
        ConversionError::Panic(msg) => format!("html-to-markdown panic during conversion: {msg}"),
        other => other.to_string(),
    }
}
