//! Utility module: helper functions for common operations.
//!
//! This module contains utility functions used across conversion logic
//! including sibling handling, content extraction, serialization, preprocessing,
//! caching, and attribute processing.
//!
//! These functions are re-exported from the main converter module to provide
//! organized access to utility functions by category.

pub mod attributes;
pub mod caching;
pub mod content;
pub mod preprocessing;
pub mod serialization;
pub mod siblings;

// Re-export commonly used functions at module level for convenience
pub use siblings::{
    append_inline_suffix, get_next_sibling_tag, get_previous_sibling_tag, next_sibling_is_inline_tag,
    next_sibling_is_whitespace_text, previous_sibling_is_inline_tag,
};

pub use content::{collect_link_label_text, get_text_content, is_empty_inline_element, normalize_link_label};

pub use serialization::{serialize_element, serialize_node, serialize_node_to_html, serialize_tag_to_html};

pub use preprocessing::{
    eq_ascii_case_insensitive, eq_ascii_insensitive, find_closing_tag, find_closing_tag_bytes, find_tag_end,
    is_json_ld_script_open_tag, matches_end_tag_start, matches_tag_start, preprocess_html, strip_script_and_style_tags,
};

pub use caching::{build_dom_context, record_node_hierarchy, text_cache_capacity_for_input};

pub use attributes::{
    attribute_contains_any, attribute_matches_any, element_has_navigation_hint, has_semantic_content_ancestor,
    is_hocr_document, may_be_hocr, tag_has_main_semantics,
};
