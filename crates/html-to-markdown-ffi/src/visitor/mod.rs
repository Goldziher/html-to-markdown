//! Visitor pattern FFI bridge for C-compatible languages
//!
//! This module provides a C-compatible FFI interface to the visitor pattern
//! implemented in the core html-to-markdown library. It allows languages like
//! Go, Java, C#, and others to implement custom HTML element processing logic.

pub mod types;

#[cfg(feature = "visitor")]
pub mod registry;

#[cfg(feature = "visitor")]
pub use registry::*;
