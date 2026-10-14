//! Semantic HTML5 element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for semantic HTML5 elements:
//! - Sectioning elements (article, section, nav, aside, header, footer, main)
//! - Figure elements (figure, figcaption)
//! - Interactive elements (details, summary)
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
pub mod definition_list;
pub mod figure;
pub mod sectioning;
pub mod summary;

pub use super::walk_node;
pub use super::{Context, DomContext};
pub use crate::converter::block::container::HandlerContext;

pub use attributes::handle as handle_attributes;
pub use attributes::{CollectedQuoteEnd, close_collected_q, close_q, open_q};
pub use definition_list::handle as handle_definition_list;
pub use figure::handle as handle_figure;
pub use sectioning::handle as handle_sectioning;
pub use summary::handle as handle_summary;

pub fn walk_tag_children(tag: &tl::HTMLTag<'_>, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.ctx,
                handler.depth + 1,
                handler.dom_ctx,
            ),
        );
    }
}

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
/// - **Summary**: details, summary
/// - **Definition List**: hgroup, dl, dt, dd, menu
/// - **Attributes**: cite, q, abbr, dfn, time, data
///
/// # Returns
///
/// Returns `true` if the tag was successfully handled by a semantic handler,
/// `false` if the tag is not a semantic element and requires other handling.
///
/// # Example
///
/// ```text
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
    handler: HandlerContext<'_>,
) -> bool {
    match tag_name {
        "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
            handle_sectioning(tag_name, node_handle, parser, output, handler);
            true
        }
        "figure" | "figcaption" => {
            handle_figure(tag_name, node_handle, parser, output, handler);
            true
        }
        // ~keep Summary and interactive elements
        "details" | "summary" => {
            handle_summary(tag_name, node_handle, parser, output, handler);
            true
        }
        "hgroup" | "dl" | "dt" | "dd" | "menu" => {
            handle_definition_list(tag_name, node_handle, parser, output, handler);
            true
        }
        "cite" | "q" | "abbr" | "dfn" | "time" | "data" => {
            handle_attributes(tag_name, node_handle, parser, output, handler);
            true
        }
        _ => false,
    }
}
