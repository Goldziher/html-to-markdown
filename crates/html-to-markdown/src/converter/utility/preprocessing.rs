//! HTML preprocessing and normalization.
//!
//! Functions for preprocessing HTML before conversion, including script/style stripping,
//! tag repair, and malformed HTML handling.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{
    eq_ascii_case_insensitive, eq_ascii_insensitive, find_closing_tag, find_closing_tag_bytes, find_tag_end,
    is_json_ld_script_open_tag, matches_end_tag_start, matches_tag_start, preprocess_html, strip_script_and_style_tags,
};
