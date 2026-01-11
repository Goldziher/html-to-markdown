//! Handlers for HTML5 figure elements.
//!
//! Processes figure-related semantic elements:
//! - `<figure>` - Self-contained illustration, diagram, photo, code listing, etc.
//! - `<figcaption>` - Caption or legend for a figure
//!
//! Figure elements group an image (or other media) with its associated caption.
//! The Markdown output preserves this relationship through content organization.

// Note: Context and DomContext are defined in converter.rs
// walk_node is also defined there and must be called via the parent module

/// Handles the `<figure>` element.
///
/// A figure element contains content (typically images) and optionally a figcaption.
/// The handler collects all content and cleans up extra line breaks.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is collected, line breaks normalized, and wrapped with blank lines
/// - **Image normalization**: Removes extra spaces before `![` to improve Markdown formatting
///
/// # Implementation Details
///
/// The handler performs the following on the collected content:
/// 1. Normalizes newline + image sequences: `\n![` → `![`
/// 2. Normalizes space + image sequences: ` ![` → `![`
/// 3. Trims the final content and wraps it with blank lines
pub fn handle_figure(
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

        // Ensure spacing before the figure
        if !output.is_empty() && !output.ends_with("\n\n") {
            output.push_str("\n\n");
        }

        // Collect content in a separate buffer
        let mut figure_content = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, &mut figure_content, options, ctx, depth, dom_ctx);
            }
        }

        // Normalize image syntax
        figure_content = figure_content.replace("\n![", "![");
        figure_content = figure_content.replace(" ![", "![");

        // Trim and output
        let trimmed = figure_content.trim_matches(|c| c == '\n' || c == ' ' || c == '\t');
        if !trimmed.is_empty() {
            output.push_str(trimmed);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            if !output.ends_with("\n\n") {
                output.push('\n');
            }
        }
    }
}

/// Handles the `<figcaption>` element.
///
/// A figcaption element contains text that describes or supplements the figure.
/// It is rendered as emphasized (italic) text to distinguish it from regular content.
///
/// # Behavior
///
/// - Content is collected and trimmed
/// - Non-empty content is wrapped in `*text*` (emphasis) markers
/// - Proper spacing is maintained around the caption
///
/// # Implementation Details
///
/// The handler:
/// 1. Collects and processes all children
/// 2. Checks for existing output and adds spacing as needed
/// 3. Wraps content in emphasis markers: `*caption*`
/// 4. Ensures proper blank-line spacing after the caption
pub fn handle_figcaption(
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
        let mut text = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
            }
        }

        let text = text.trim();
        if !text.is_empty() {
            // Add spacing before caption if needed
            if !output.is_empty() {
                if output.ends_with("```\n") {
                    output.push('\n');
                } else {
                    // Trim trailing whitespace and ensure single blank line
                    while output.ends_with(' ') || output.ends_with('\t') {
                        output.pop();
                    }
                    if output.ends_with('\n') && !output.ends_with("\n\n") {
                        output.push('\n');
                    } else if !output.ends_with('\n') {
                        output.push_str("\n\n");
                    }
                }
            }

            // Output caption as emphasized text
            output.push('*');
            output.push_str(text);
            output.push_str("*\n\n");
        }
    }
}

/// Dispatcher for figure-related elements.
///
/// Routes `<figure>` and `<figcaption>` elements to their respective handlers.
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
        "figure" => handle_figure(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "figcaption" => handle_figcaption(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        _ => {}
    }
}
