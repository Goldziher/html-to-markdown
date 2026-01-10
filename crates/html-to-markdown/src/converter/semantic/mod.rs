//! Semantic HTML5 element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for semantic HTML5 elements:
//! - Sectioning elements (article, section, nav, aside, header, footer, main)
//! - Figure elements (figure, figcaption)
//! - Interactive elements (details, summary, dialog)
//! - Semantic inline attributes (cite, q, abbr, dfn, time, data)
//!
//! These handlers are designed to be extracted from the main `converter.rs`
//! file and integrated through the dispatcher function.
//!
//! **Integration Pattern:**
//! Each handler function takes the same signature:
//! - `tag_name: &str` - The HTML tag being processed
//! - `node_handle: &NodeHandle` - The DOM node handle
//! - `parser: &Parser` - The HTML parser reference
//! - `output: &mut String` - The output buffer to write to
//! - `options: &ConversionOptions` - Conversion configuration
//! - `ctx: &Context` - Processing context (state tracking)
//! - `depth: usize` - Current DOM tree depth
//! - `dom_ctx: &DomContext` - DOM context for tree relationships
//!
//! The main dispatcher function `dispatch_semantic_handler` routes tags to
//! their appropriate handlers and returns a boolean indicating success.

pub mod attributes;
pub mod figure;
pub mod sectioning;
pub mod summary;

// Re-export types from parent module for submodule access
pub(crate) use super::walk_node;
pub use super::{Context, DomContext};

// Re-export handler functions for direct use
pub use attributes::handle as handle_attributes;
pub use figure::handle as handle_figure;
pub use sectioning::handle as handle_sectioning;
pub use summary::handle as handle_summary;

// Re-exports are done via the dispatch function parameter types

/// Dispatches semantic element handling to the appropriate handler.
///
/// This function routes semantic HTML5 elements to their specialized handlers
/// based on tag name. It is designed to be called from the main `walk_node`
/// function in `converter.rs`.
///
/// # Routing Table
///
/// The following tag routes are supported:
/// - **Sectioning**: article, section, nav, aside, header, footer, main
/// - **Figure**: figure, figcaption
/// - **Summary**: details, summary, dialog
/// - **Attributes**: cite, q, abbr, dfn, time, data
///
/// # Returns
///
/// Returns `true` if the tag was successfully handled by a semantic handler,
/// `false` if the tag is not a semantic element and requires other handling.
///
/// # Example
///
/// ```ignore
/// if dispatch_semantic_handler(tag_name, &node_handle, &parser, output, options, ctx, depth, dom_ctx) {
///     // Tag was handled
/// } else {
///     // Continue with other handlers
/// }
/// ```
pub fn dispatch_semantic_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) -> bool {
    match tag_name {
        // Sectioning elements
        "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
            handle_sectioning(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Figure elements
        "figure" | "figcaption" => {
            handle_figure(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Summary and interactive elements
        "details" | "summary" | "dialog" => {
            handle_summary(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Semantic inline attributes
        "cite" | "q" | "abbr" | "dfn" | "time" | "data" => {
            handle_attributes(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        _ => false,
    }
}
