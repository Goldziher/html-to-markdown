//! Media element handlers for HTML-to-Markdown conversion.
//!
//! This module provides specialized handling for various media elements:
//! - **Image**: img tags with inline data URI and metadata collection
//! - **Graphic**: Custom graphic elements with multiple source attributes
//! - **SVG**: SVG and MathML elements with serialization and base64 encoding
//! - **Embedded**: iframe, video, audio, and source elements

pub mod embedded;
pub mod graphic;
pub mod image;
pub mod svg;

#[cfg(feature = "inline-images")]
pub use image::handle_inline_data_image;
pub use image::heading_allows_inline_images;

pub use graphic::{extract_graphic_alt, extract_graphic_src, should_skip_graphic_attr};

#[cfg(feature = "inline-images")]
pub use svg::{encode_svg_to_data_uri, handle_inline_svg};
pub use svg::{serialize_element, serialize_node};

pub use embedded::{extract_media_src, find_source_src, is_source_element, should_output_media_link};
