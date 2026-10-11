//! HTML to Markdown conversion engine with modular architecture.
//!
//! This module provides the complete conversion pipeline for transforming HTML documents
//! into Markdown format. It follows a modular, type-safe design where HTML element handling
//! is organized by semantic category (block, inline, list, table, etc.) with dispatch functions
//! routing elements to their specialized handlers.
//!
//! # Module Organization
//!
//! The converter module is organized into semantic categories:
//!
//! - **[block]**: Block-level elements (headings, paragraphs, blockquotes, preformatted text, tables)
//! - **[inline]**: Inline formatting (emphasis, links, code, semantic formatting)
//! - **[list]**: List structures (ordered, unordered, definition lists)
//! - **[table]**: Accessible via `block::table` submodule
//! - **[media]**: Media elements (images, video, audio, embedded content, SVG)
//! - **[semantic]**: Semantic HTML5 elements (sectioning, figures, interactive elements)
//! - **[form]**: Form elements (inputs, selects, buttons, fieldsets)
//! - **[utility]**: Helper functions (DOM traversal, caching, serialization, attributes)
//! - **[text]**: Text processing and escaping (via `crate::text` module)
//!
//! # Public Types
//!
//! The main context types used across the conversion pipeline:
//!
//! - **[Context]**: Stateful conversion context tracking (e.g., list nesting, code blocks, `in_heading`)
//! - **[`DomContext`]**: DOM relationship cache for efficient tree navigation
//!
//! # Conversion Flow
//!
//! The conversion process follows these steps:
//!
//! 1. **Parse HTML**: Input HTML is parsed into a DOM tree using the astral-tl parser
//! 2. **Walk Tree**: Recursive tree walk starting from the root document node
//! 3. **Dispatch**: Each element is dispatched to its handler based on tag name
//! 4. **Convert**: Handler transforms the element to Markdown representation
//! 5. **Post-process**: Text escaping and whitespace normalization
//!
//! # Handler Pattern
//!
//! Each submodule (block, inline, list, etc.) follows a consistent pattern:
//!
//! ```text
//! // Module declares handlers for specific element types
//! pub fn dispatch_<category>_handler(
//!     tag_name: &str,
//!     node_handle: &NodeHandle,
//!     parser: &Parser,
//!     output: &mut String,
//!     options: &ConversionOptions,
//!     ctx: &Context,
//!     depth: usize,
//!     dom_ctx: &DomContext,
//! ) -> bool {
//!     // Route to appropriate handler, return true if handled
//! }
//! ```
//!
//! # Visibility Rules
//!
//! - **Context & `DomContext`**: Public types for external module coordination
//! - **Dispatch functions**: Public for main `walk_node` caller
//! - **Individual handlers**: Typically pub for direct access if needed
//! - **Internal utilities**: pub(crate) or pub(super) for module-internal use
//!
//! # Feature Support
//!
//! - Inline image extraction (`inline-images` feature)
//! - Metadata collection (`metadata` feature)
//! - Custom visitor callbacks (`visitor` feature)
//!
//! # Example Integration
//!
//! Once `converter.rs` is refactored to use `converter/main.rs`, the `walk_node` function
//! will use dispatch functions like:
//!
//! ```text
//! use crate::converter::{block, inline, list, media, semantic, form};
//!
//! fn walk_node(...) {
//!     // Try each dispatcher in order
//!     if block::dispatch_block_handler(&tag, ...) { return; }
//!     if inline::dispatch_inline_handler(&tag, ...) { return; }
//!     if list::dispatch_list_handler(&tag, ...) { return; }
//!     if media::dispatch_media_handler(&tag, ...) { return; }
//!     if semantic::dispatch_semantic_handler(&tag, ...) { return; }
//!     if form::dispatch_form_handler(&tag, ...) { return; }
//!     // Default handling for unrecognized tags
//! }
//! ```

mod anchor_origin;
pub mod block;
mod code_scan;
pub mod context;
pub mod dom_context;
pub mod form;
pub mod format;
pub mod handlers;
pub mod head_metadata;
pub mod inline;
pub mod list;
pub mod main;
mod main_dispatch;
mod main_helpers;
pub mod media;
mod metadata;
#[cfg(all(test, feature = "metadata"))]
mod metadata_preferences_tests;
pub mod plain_text;
pub mod preprocessing_helpers;
pub mod prescan;
pub mod reference_collector;
pub mod semantic;
mod structure_capture;
pub mod text;
mod text_node;
pub mod tier1;
pub mod url_resolve;
pub mod utility;

#[cfg(feature = "visitor")]
pub mod visitor_hooks;

pub use self::context::Context;
pub use self::dom_context::DomContext;

pub use self::main::{convert_html_impl, walk_node};
pub use self::main_helpers::document_head;
pub use self::main_helpers::{emit_table_cell_break, strip_trailing_backslash_breaks, trim_trailing_whitespace};
pub use self::main_helpers::{repair_head_content_before_explicit_head, repair_limit_warning};

pub use crate::converter::utility::content::{
    chomp_inline, get_text_content, merge_adjacent_emphasis, normalized_tag_name,
};
// ~keep reason: serialize_node and serialize_node_to_html are only used when the visitor
// ~keep or inline-images features are active; re-exported here so call sites don't need
// ~keep to know the internal utility path.
#[allow(unused_imports)]
pub use crate::converter::utility::serialization::{serialize_node, serialize_node_to_html};

pub use crate::converter::utility::siblings::append_inline_suffix;
