//! List processing module for converting HTML lists to Markdown.
//!
//! Handles ordered lists (ol), unordered lists (ul), list items (li),
//! and definition lists (dl, dt, dd). Supports nested lists, loose/tight
//! list detection, and proper indentation.
//!
//! **Note on Current Integration:**
//! This module cannot currently be fully integrated into converter.rs due to
//! Rust's module system rules (cannot have both converter.rs and converter/mod.rs).
//! Once converter.rs is refactored to use converter/main.rs or similar pattern,
//! these handlers should be exposed through converter/mod.rs and used in the
//! main walk_node function via the dispatch_list_handler function below.

pub mod definition;
pub mod ordered;
pub mod unordered;
pub mod utils;

// Re-export utilities for use once converter.rs is refactored
pub use utils::{
    add_list_continuation_indent, add_list_leading_separator, add_nested_list_trailing_separator,
    calculate_list_continuation_indent, calculate_list_nesting_depth, continuation_indent_string, is_list_item,
    is_loose_list, process_list_children,
};

/// Dispatches list element handling to the appropriate handler.
///
/// This function is designed to be called from the main walk_node function
/// in converter.rs once the module is refactored. It returns `true` if the
/// element was handled, `false` otherwise.
///
/// # Usage in converter.rs
/// ```ignore
/// if crate::converter::list::dispatch_list_handler(
///     &tag_name,
///     node_handle,
///     parser,
///     output,
///     options,
///     ctx,
///     depth,
///     dom_ctx,
/// ) {
///     return; // Element was handled
/// }
/// ```
pub(crate) fn dispatch_list_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::Context,
    depth: usize,
    dom_ctx: &crate::converter::DomContext,
) -> bool {
    match tag_name {
        "ul" => {
            unordered::handle_ul(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "ol" => {
            ordered::handle_ol(node_handle, parser, output, options, ctx, depth, dom_ctx);
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

use tl;
