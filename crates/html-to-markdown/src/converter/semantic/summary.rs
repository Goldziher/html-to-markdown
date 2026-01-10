//! Handlers for HTML5 interactive elements.
//!
//! Processes interactive disclosure and dialog semantic elements:
//! - `<details>` - Expandable/collapsible disclosure widget
//! - `<summary>` - Summary or caption for a details element
//! - `<dialog>` - Dialog box overlay widget
//!
//! These elements are treated as block-level content containers
//! with special formatting for the summary element.

// Note: Context and DomContext are defined in converter.rs
// walk_node is also defined there and must be called via the parent module
use super::walk_node;

/// Handles the `<details>` element.
///
/// A details element represents a disclosure widget that can be toggled
/// to show/hide additional content. In Markdown, it's rendered as a block
/// with all content visible.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is collected and wrapped with proper blank-line spacing
/// - **Empty content**: Skipped entirely
pub fn handle_details(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        // In inline context, just process children inline
        if ctx.convert_as_inline {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    super::walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                }
            }
            return;
        }

        // Collect content
        let mut content = String::with_capacity(256);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            // Add spacing before if needed
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }

            // Output content
            output.push_str(trimmed);
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<summary>` element.
///
/// A summary element contains a caption for a details element.
/// It is rendered as strong (bold) text to distinguish it from regular content.
///
/// # Behavior
///
/// - **Inline mode**: Content is rendered inline without emphasis
/// - **Block mode**: Content is wrapped in strong markers (e.g., `**text**`)
/// - Uses the configured strong/emphasis symbol from ConversionOptions
///
/// # Implementation Details
///
/// The handler:
/// 1. Creates a context with `in_strong: true` for nested formatting
/// 2. Collects content from all children
/// 3. Wraps non-empty content in strong markers (repeated twice per Markdown spec)
pub fn handle_summary(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::with_capacity(64);

        // Set strong context for nested content
        let mut summary_ctx = ctx.clone();
        if !ctx.convert_as_inline {
            summary_ctx.in_strong = true;
        }

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &summary_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            if ctx.convert_as_inline {
                // Inline mode: output without formatting
                output.push_str(trimmed);
            } else {
                // Block mode: output with strong markers
                let symbol = options.strong_em_symbol.to_string().repeat(2);
                output.push_str(&symbol);
                output.push_str(trimmed);
                output.push_str(&symbol);
                output.push_str("\n\n");
            }
        }
    }
}

/// Handles the `<dialog>` element.
///
/// A dialog element represents a modal dialog box. In Markdown, it's rendered
/// as a block container with content visible.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is processed and wrapped with proper blank lines
/// - Trailing whitespace is removed from collected content
///
/// # Implementation Details
///
/// The handler:
/// 1. Marks the position in output before processing children
/// 2. Processes all children in the normal context
/// 3. Removes trailing spaces and tabs from the output
/// 4. Ensures proper blank-line spacing after the dialog
pub fn handle_dialog(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        // In inline context, just process children inline
        if ctx.convert_as_inline {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    super::walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                }
            }
            return;
        }

        // Mark position before processing children
        let content_start = output.len();

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
            }
        }

        // Remove trailing whitespace from dialog content
        while output.len() > content_start && (output.ends_with(' ') || output.ends_with('\t')) {
            output.pop();
        }

        // Ensure proper spacing after dialog
        if output.len() > content_start && !output.ends_with("\n\n") {
            output.push_str("\n\n");
        }
    }
}

/// Dispatcher for interactive elements.
///
/// Routes `<details>`, `<summary>`, and `<dialog>` elements to their respective handlers.
pub fn handle(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    match tag_name {
        "details" => handle_details(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "summary" => handle_summary(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "dialog" => handle_dialog(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        _ => {}
    }
}
