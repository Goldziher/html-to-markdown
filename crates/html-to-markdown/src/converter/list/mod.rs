//! List element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handling for various list types:
//! - **Ordered lists**: `<ol>` with counter management and formatting options
//! - **Unordered lists**: `<ul>` with bullet cycling based on nesting depth
//! - **Definition lists**: `<dl>`, `<dt>`, `<dd>` elements
//! - **List utilities**: Indentation, loose/tight list detection, nesting depth calculation

pub mod definition;
pub mod ordered;
pub mod unordered;
pub mod utils;

// Re-export types from parent module for submodule access
pub use super::{Context, DomContext};

// Re-export utility function needed by table builder
pub(crate) use utils::continuation_indent_string;

/// Dispatches list element handling to the appropriate handler.
///
/// Returns `true` if the element was handled, `false` otherwise.
///
/// # Supported Elements
///
/// - `ol`: Ordered list - routed to `ordered::handle`
/// - `ul`: Unordered list - routed to `unordered::handle`
/// - `li`: List item - handled contextually by parent list handler
/// - `dl`: Definition list - routed to `definition::handle_dl`
/// - `dt`: Definition term - routed to `definition::handle_dt`
/// - `dd`: Definition description - routed to `definition::handle_dd`
pub fn dispatch_list_handler(
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
        "ol" => {
            ordered::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "ul" => {
            unordered::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "dl" => {
            definition::handle_dl(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "dt" => {
            definition::handle_dt(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "dd" => {
            definition::handle_dd(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        _ => false,
    }
}
