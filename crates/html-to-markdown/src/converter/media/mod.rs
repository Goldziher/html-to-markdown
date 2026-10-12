//! Media element handlers for HTML-to-Markdown conversion.
//!
//! This module provides specialized handling for various media elements:
//! - **Image**: img tags with inline data URI and metadata collection
//! - **Graphic**: Custom graphic elements with multiple source attributes
//! - **SVG**: SVG and `MathML` elements with serialization and base64 encoding
//! - **Embedded**: iframe, video, audio, and source elements

pub mod embedded;
pub mod graphic;
pub mod image;
pub mod svg;

#[cfg(feature = "inline-images")]
pub use image::handle_inline_data_image;

use crate::options::InlineDataMedia;

#[derive(Clone, Copy)]
pub struct MediaContext<'a> {
    pub options: &'a crate::options::ConversionOptions,
    pub ctx: &'a super::Context,
    pub depth: usize,
    pub dom_ctx: &'a super::DomContext,
}

/// Whether `address` is a `data:` URL, which carries its content inline. The scheme matches in
/// any case, after leading whitespace, as a URL parser reads it.
pub fn is_inline_data(address: &str) -> bool {
    data_url_body(address).is_some()
}

/// The part of a `data:` URL after its scheme (`image/png;base64,...`), or `None` when `address`
/// is not one. Matches as [`is_inline_data`] does.
pub fn data_url_body(address: &str) -> Option<&str> {
    let address = address.trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control());
    address
        .get(..5)
        .filter(|scheme| scheme.eq_ignore_ascii_case("data:"))
        .map(|_| &address[5..])
}

/// What to write for an element whose chosen address is `address`: `choice` when the address is
/// a `data:` URL, otherwise [`InlineDataMedia::Keep`].
pub fn inline_data_treatment(choice: InlineDataMedia, address: &str) -> InlineDataMedia {
    if is_inline_data(address) {
        choice
    } else {
        InlineDataMedia::Keep
    }
}

/// The first of `addresses` to use: with any choice but [`InlineDataMedia::Keep`], the first that
/// is not a `data:` URL, else the first one.
pub fn first_address<T: AsRef<str>>(choice: InlineDataMedia, addresses: impl IntoIterator<Item = T>) -> Option<T> {
    let mut first = None;
    for address in addresses {
        if choice == InlineDataMedia::Keep || !is_inline_data(address.as_ref()) {
            return Some(address);
        }
        first.get_or_insert(address);
    }
    first
}

/// Dispatches media element handling to the appropriate handler.
///
/// This function routes media-related HTML elements to their specialized handlers
/// based on tag name. It is designed to be called from the main `walk_node`
/// function in `converter.rs`.
///
/// # Routing Table
///
/// The following tag routes are supported:
///
/// | Tag(s) | Handler | Description |
/// |--------|---------|-------------|
/// | `iframe` | embedded | Embedded content frames |
/// | `svg` | svg | SVG image elements |
/// | `math` | svg | `MathML` elements |
///
/// # Return Value
///
/// Returns `true` if the tag was recognized and handled, `false` otherwise.
pub fn dispatch_media_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: MediaContext<'_>,
) -> bool {
    let Some(node) = node_handle.get(parser) else {
        return false;
    };
    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return false,
    };

    match tag_name {
        "iframe" => {
            embedded::handle_iframe(node_handle, tag, parser, output, context);
            true
        }
        "svg" => {
            svg::handle_svg(node_handle, tag, parser, output, context);
            true
        }
        "math" => {
            svg::handle_math(node_handle, tag, parser, output, context);
            true
        }
        _ => false,
    }
}
