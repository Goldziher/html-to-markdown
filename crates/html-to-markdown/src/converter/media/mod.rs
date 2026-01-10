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
pub(crate) use image::handle_inline_data_image;

#[cfg(feature = "inline-images")]
pub(crate) use svg::handle_inline_svg;
