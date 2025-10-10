//! # html-to-markdown
//!
//! A modern, high-performance library for converting HTML to Markdown.
//! Built with html5ever for fast, memory-efficient HTML parsing.

pub mod converter;
pub mod error;
pub mod hocr;
pub mod options;
pub mod sanitizer;
pub mod text;
pub mod wrapper;

pub use error::{ConversionError, Result};
pub use options::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle, ParsingOptions,
    PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};

/// Convert HTML to Markdown.
///
/// This function takes HTML input and converts it to Markdown using the provided options.
/// If no options are provided, default options will be used.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options
///
/// # Example
///
/// ```
/// use html_to_markdown::{convert, ConversionOptions};
///
/// let html = "<h1>Hello World</h1>";
/// let markdown = convert(html, None).unwrap();
/// assert!(markdown.contains("Hello World"));
/// ```
pub fn convert(html: &str, options: Option<ConversionOptions>) -> Result<String> {
    let options = options.unwrap_or_default();

    let normalized_html = html.replace("\r\n", "\n").replace('\r', "\n");

    let clean_html = if options.preprocessing.enabled {
        sanitizer::sanitize(&normalized_html, &options.preprocessing)?
    } else {
        normalized_html
    };

    let markdown = converter::convert_html(&clean_html, &options)?;

    if options.wrap {
        Ok(wrapper::wrap_markdown(&markdown, &options))
    } else {
        Ok(markdown)
    }
}
