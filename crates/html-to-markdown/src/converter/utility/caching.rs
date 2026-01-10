//! Performance caching utilities.
//!
//! Caching mechanisms for expensive operations during conversion, including
//! DOM context building and cache capacity management.
//!
//! These functions are re-exported from the main converter module for organizational purposes.

#![allow(unused_imports)]

pub use crate::converter::{build_dom_context, record_node_hierarchy, text_cache_capacity_for_input};
