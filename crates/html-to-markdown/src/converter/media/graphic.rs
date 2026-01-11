//! Graphic element handling (custom graphic elements with alternative source attributes).

use std::borrow::Cow;
use tl::HTMLTag;

/// Handle custom graphic elements with multiple source attribute options.
///
/// The graphic element is a custom XML element that supports multiple source attributes:
/// - `url` (primary)
/// - `href` (secondary)
/// - `xlink:href` (SVG standard)
/// - `src` (fallback)
///
/// This is commonly used in publishing formats like EPUB.
pub(crate) fn extract_graphic_src<'a>(tag: &'a HTMLTag<'a>) -> Cow<'a, str> {
    tag.attributes()
        .get("url")
        .flatten()
        .or_else(|| tag.attributes().get("href").flatten())
        .or_else(|| tag.attributes().get("xlink:href").flatten())
        .or_else(|| tag.attributes().get("src").flatten())
        .map_or_else(|| Cow::Borrowed(""), |v| v.as_utf8_str())
}

/// Extract alt text from graphic element with fallback to filename.
pub(crate) fn extract_graphic_alt<'a>(tag: &'a HTMLTag<'a>) -> Cow<'a, str> {
    tag.attributes()
        .get("alt")
        .flatten()
        .map(|v| v.as_utf8_str())
        .or_else(|| tag.attributes().get("filename").flatten().map(|v| v.as_utf8_str()))
        .unwrap_or_else(|| Cow::Borrowed(""))
}

/// Get source attributes to skip during metadata collection.
///
/// These attributes are handled specially and should not be included
/// in the generic attributes map.
pub(crate) fn should_skip_graphic_attr(key_str: &str) -> bool {
    matches!(key_str, "url" | "href" | "xlink:href" | "src")
}
