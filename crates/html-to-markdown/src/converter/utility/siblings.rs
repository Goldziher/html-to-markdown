//! Sibling node navigation and handling.
//!
//! Utilities for working with sibling nodes in the DOM tree, including navigation functions
//! and inline/block element detection for whitespace handling.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{
    append_inline_suffix, get_next_sibling_tag, get_previous_sibling_tag, next_sibling_is_inline_tag,
    next_sibling_is_whitespace_text, previous_sibling_is_inline_tag,
};
