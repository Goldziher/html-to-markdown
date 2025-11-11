//! High-performance HTML to Markdown converter.
//!
//! Built with html5ever for fast, memory-efficient HTML parsing.
//!
//! ## Optional inline image extraction
//!
//! Enable the `inline-images` Cargo feature to collect embedded data URI images and inline SVG
//! assets alongside the produced Markdown.

use std::borrow::Cow;

pub mod converter;
pub mod error;
pub mod hocr;
#[cfg(feature = "inline-images")]
mod inline_images;
pub mod options;
pub mod text;
pub mod wrapper;

pub use error::{ConversionError, Result};
#[cfg(feature = "inline-images")]
pub use inline_images::{
    HtmlExtraction, InlineImage, InlineImageConfig, InlineImageFormat, InlineImageSource, InlineImageWarning,
};
pub use options::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, ListIndentType, NewlineStyle,
    PreprocessingOptions, PreprocessingPreset, WhitespaceMode,
};

/// Convert HTML to Markdown.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (defaults to ConversionOptions::default())
///
/// # Example
///
/// ```
/// use html_to_markdown_rs::{convert, ConversionOptions};
///
/// let html = "<h1>Hello World</h1>";
/// let markdown = convert(html, None).unwrap();
/// assert!(markdown.contains("Hello World"));
/// ```
pub fn convert(html: &str, options: Option<ConversionOptions>) -> Result<String> {
    let options = options.unwrap_or_default();

    let normalized_html = if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    };

    let markdown = converter::convert_html(normalized_html.as_ref(), &options)?;

    if options.wrap {
        Ok(wrapper::wrap_markdown(&markdown, &options))
    } else {
        Ok(markdown)
    }
}

#[cfg(feature = "inline-images")]
/// Convert HTML to Markdown while collecting inline image assets (requires the `inline-images` feature).
///
/// Extracts inline image data URIs and inline `<svg>` elements alongside Markdown conversion.
///
/// # Arguments
///
/// * `html` - The HTML string to convert
/// * `options` - Optional conversion options (defaults to ConversionOptions::default())
/// * `image_cfg` - Configuration controlling inline image extraction
pub fn convert_with_inline_images(
    html: &str,
    options: Option<ConversionOptions>,
    image_cfg: InlineImageConfig,
) -> Result<HtmlExtraction> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let options = options.unwrap_or_default();

    let normalized_html = if html.contains('\r') {
        Cow::Owned(html.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(html)
    };

    let collector = Rc::new(RefCell::new(inline_images::InlineImageCollector::new(image_cfg)?));

    let markdown =
        converter::convert_html_with_inline_collector(normalized_html.as_ref(), &options, Rc::clone(&collector))?;

    let markdown = if options.wrap {
        wrapper::wrap_markdown(&markdown, &options)
    } else {
        markdown
    };

    let collector = Rc::try_unwrap(collector)
        .map_err(|_| ConversionError::Other("failed to recover inline image state".to_string()))?
        .into_inner();
    let (inline_images, warnings) = collector.finish();

    Ok(HtmlExtraction {
        markdown,
        inline_images,
        warnings,
    })
}
