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
//! - **[text]**: Text processing and escaping (via crate::text module)
//!
//! # Public Types
//!
//! The main context types used across the conversion pipeline:
//!
//! - **[Context]**: Stateful conversion context tracking (e.g., list nesting, code blocks, in_heading)
//! - **[DomContext]**: DOM relationship cache for efficient tree navigation
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
//! ```ignore
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
//! - **Context & DomContext**: Public types for external module coordination
//! - **Dispatch functions**: Public for main walk_node caller
//! - **Individual handlers**: Typically pub for direct access if needed
//! - **Internal utilities**: pub(crate) or pub(super) for module-internal use
//!
//! # Feature Support
//!
//! - Inline image extraction (`inline-images` feature)
//! - Metadata collection (`metadata` feature)
//! - Custom visitor callbacks (`visitor` feature)
//! - Async visitor support (`async-visitor` feature)
//!
//! # Example Integration
//!
//! Once `converter.rs` is refactored to use `converter/main.rs`, the walk_node function
//! will use dispatch functions like:
//!
//! ```ignore
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

pub mod block;
pub mod context;
pub mod form;
pub mod inline;
pub mod list;
pub mod main;
pub mod media;
pub mod semantic;
pub mod text;
pub mod utility;

// Import and re-export public types and functions from the main module
pub use self::main::{Context, DomContext, convert_html};

#[cfg(feature = "visitor")]
pub use self::main::convert_html_with_visitor;

// Import the tree walker and utility functions from main
pub(crate) use self::main::{convert_html_impl, trim_trailing_whitespace, walk_node};

// Re-export helper functions from utility modules (migrated from converter_legacy)
pub(crate) use crate::converter::utility::content::{chomp_inline, get_text_content, normalized_tag_name};
#[allow(unused_imports)]
pub(crate) use crate::converter::utility::serialization::{serialize_node, serialize_node_to_html};

// Helper functions migrated to utility modules
pub(crate) use crate::converter::utility::siblings::append_inline_suffix;

// Caching functions migrated to utility/caching

// Content functions migrated to utility/content

// Heading functions migrated to block/heading
pub(crate) use crate::converter::block::heading::find_single_heading_child;

// Link functions migrated to inline/link

// Re-export dispatch functions for routing elements to handlers
pub use block::dispatch_block_handler;
pub use form::dispatch_form_handler;
pub use inline::dispatch_inline_handler;
pub use list::dispatch_list_handler;
pub use semantic::dispatch_semantic_handler;
// Media module doesn't have a dispatcher - it exports utility functions

// Re-export utility submodules for public access to their types
pub use utility::{attributes, caching, content, preprocessing, serialization, siblings};

// Block and inline handlers are internal - only dispatchers are exposed
// Individual handlers are pub(crate) and not meant to be part of the public API

// Re-export media utilities for internal use (crate-private)

// Re-export list utilities for internal use (crate-private)
pub(crate) use list::continuation_indent_string;

// Semantic and form handlers are also internal (pub(crate))
