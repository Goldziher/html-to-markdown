//! Element handlers extracted from the main conversion pipeline.
//!
//! This module contains handler functions for specific HTML elements,
//! allowing the main `walk_node` function to delegate to specialized handlers.
//!
//! Each handler takes the standard set of parameters:
//! - `node_handle`: Reference to the DOM node
//! - `tag`: The HTML tag being processed
//! - `parser`: The DOM parser
//! - `output`: The output string buffer
//! - `options`: Conversion options
//! - `ctx`: Conversion context
//! - `depth`: Current tree depth
//! - `dom_ctx`: DOM context cache

pub mod blockquote;
pub mod code_block;
pub mod graphic;
pub mod image;
pub mod link;
mod srcset;

pub use blockquote::handle_blockquote;
pub use code_block::handle_pre;
pub use graphic::handle_graphic;
pub use image::handle_img;
pub use link::open_link;
