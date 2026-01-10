//! Form element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for HTML form elements:
//! - Container elements (form, fieldset, legend, label)
//! - Input elements (input, textarea, select, option, optgroup, button)
//! - Measurement elements (progress, meter, output, datalist)
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
//! The main dispatcher function `dispatch_form_handler` routes tags to
//! their appropriate handlers and returns a boolean indicating success.

pub mod elements;

// Re-export types from parent module for submodule access
pub(crate) use super::walk_node;
pub use super::{Context, DomContext};

// Re-export handler function for direct use
pub use elements::handle as handle_form_elements;

// Re-exports are done via the dispatch function parameter types

/// Dispatches form element handling to the appropriate handler.
///
/// This function routes form-related HTML elements to their specialized handlers
/// based on tag name. It is designed to be called from the main `walk_node`
/// function in `converter.rs`.
///
/// # Routing Table
///
/// The following tag routes are supported:
/// - **Containers**: form, fieldset, legend, label
/// - **Inputs**: input, textarea, select, option, optgroup, button
/// - **Measurements**: progress, meter, output, datalist
///
/// # Returns
///
/// Returns `true` if the tag was successfully handled by a form handler,
/// `false` if the tag is not a form element and requires other handling.
///
/// # Example
///
/// ```ignore
/// if dispatch_form_handler(tag_name, &node_handle, &parser, output, options, ctx, depth, dom_ctx) {
///     // Tag was handled
/// } else {
///     // Continue with other handlers
/// }
/// ```
pub fn dispatch_form_handler(
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
        // Form containers and metadata
        "form" | "fieldset" | "legend" | "label" | "input" | "textarea" | "select" | "option" | "optgroup"
        | "button" | "progress" | "meter" | "output" | "datalist" => {
            handle_form_elements(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        _ => false,
    }
}
