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

// Re-export commonly used functions for convenience
