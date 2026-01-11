//! Text processing module for HTML to Markdown conversion.
//!
//! This module provides utilities for normalizing, escaping, and processing text content
//! extracted from HTML documents during the conversion to Markdown format.

mod escaping;
mod normalization;
mod processing;

pub use escaping::{escape_link_label, escape_malformed_angle_brackets};
pub use normalization::{
    chomp_inline, normalize_heading_text, trim_line_end_whitespace, trim_trailing_whitespace, truncate_at_char_boundary,
};
pub use processing::dedent_code_block;
