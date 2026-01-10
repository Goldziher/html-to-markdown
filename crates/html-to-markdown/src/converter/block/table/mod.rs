//! Table element handler for HTML to Markdown conversion.
//!
//! This module provides specialized handling for table elements including:
//! - Table structure detection and scanning (TableScan)
//! - Row and cell conversion to Markdown table format
//! - Cell content processing with colspan/rowspan support
//! - Layout table detection (tables used for visual layout)
//! - Integration with the visitor pattern for custom table handling
//!
//! Tables are converted to Markdown pipe-delimited format with header separators.
//! Layout tables (tables without proper semantic headers) may be converted to lists
//! instead of tables for better readability.

pub mod builder;
pub mod cell;
pub mod scanner;

// Re-export types from parent module for submodule access
pub use super::super::{Context, DomContext};

// Re-export for use in converter.rs
pub(crate) use builder::handle_table;

/// Dispatches table element handling to the main convert_table function.
///
/// # Usage in converter.rs
/// ```ignore
/// if "table" == tag_name {
///     crate::converter::block::table::handle_table(
///         node_handle,
///         parser,
///         output,
///         options,
///         ctx,
///         dom_ctx,
///         depth,
///     );
///     return;
/// }
/// ```
pub fn dispatch_table_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::Context,
    depth: usize,
    dom_ctx: &super::super::DomContext,
) -> bool {
    match tag_name {
        "table" => {
            builder::handle_table(node_handle, parser, output, options, ctx, dom_ctx, depth);
            true
        }
        _ => false,
    }
}
