//! Content extraction and manipulation utilities.
//!
//! Functions for extracting and processing element content, including text collection
//! and empty element detection.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{collect_link_label_text, get_text_content, is_empty_inline_element, normalize_link_label};
