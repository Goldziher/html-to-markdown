//! Attribute handling and extraction utilities.
//!
//! Functions for working with element attributes, semantic detection, and hOCR document detection.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{
    attribute_contains_any, attribute_matches_any, element_has_navigation_hint, has_semantic_content_ancestor,
    is_hocr_document, may_be_hocr, tag_has_main_semantics,
};
