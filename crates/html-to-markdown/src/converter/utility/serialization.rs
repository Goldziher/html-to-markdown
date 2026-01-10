//! Output serialization and formatting.
//!
//! Utilities for serializing HTML elements back to string format, used for preserving
//! original HTML for elements like SVG, math, and custom elements.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{serialize_element, serialize_node, serialize_node_to_html, serialize_tag_to_html};
