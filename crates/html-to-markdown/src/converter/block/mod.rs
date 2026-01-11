//! Block element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for block-level HTML elements:
//! - Headings (h1-h6)
//! - Paragraphs (p)
//! - Blockquotes (blockquote)
//! - Preformatted code (pre)
//! - Tables (table, thead, tbody, tfoot, tr, th, td)
//!
//! These handlers are designed to be extracted from the main `converter.rs`
//! file and integrated once the converter module is refactored.
//!
//! **Note on Current Integration:**
//! This module cannot currently be fully integrated into converter.rs due to
//! Rust's module system rules (cannot have both converter.rs and converter/mod.rs).
//! Once converter.rs is refactored to use converter/main.rs or similar pattern,
//! these handlers should be exposed through converter/mod.rs and used in the
//! main walk_node function via the dispatch_block_handler function below.

pub mod blockquote;
pub mod heading;
pub mod paragraph;
pub mod preformatted;
pub mod table;

// Re-export types from parent module for submodule access
pub use super::{Context, DomContext};

// Re-export for internal use by dispatcher (crate-private)

/// Dispatches block element handling to the appropriate handler.
///
/// This function is designed to be called from the main walk_node function
/// in converter.rs once the module is refactored. It returns `true` if the
/// element was handled, `false` otherwise.
///
/// # Usage in converter.rs
/// ```ignore
/// if crate::converter::block::dispatch_block_handler(
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
pub fn dispatch_block_handler(
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
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            heading::handle(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "p" => {
            paragraph::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "blockquote" => {
            blockquote::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "pre" => {
            preformatted::handle_pre(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        "table" => {
            table::handle_table(node_handle, parser, output, options, ctx, dom_ctx, depth);
            true
        }
        _ => false,
    }
}
