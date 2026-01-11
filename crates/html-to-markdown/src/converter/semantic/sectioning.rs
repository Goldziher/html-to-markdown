//! Handlers for HTML5 sectioning elements.
//!
//! Processes semantic sectioning elements:
//! - `<article>` - Independent, self-contained content
//! - `<section>` - Generic grouping of thematic content
//! - `<nav>` - Navigation links (typically rendered inline or in sidebars)
//! - `<aside>` - Peripheral content (sidebars, callouts)
//! - `<header>` - Introductory content (page headers)
//! - `<footer>` - End content (page footers)
//! - `<main>` - Primary content area
//!
//! All these elements are treated as block-level containers.
//! Their content is extracted and formatted with proper spacing.

// Note: Context and DomContext are defined in converter.rs
// walk_node is also defined there and must be called via the parent module

/// Handles sectioning elements (article, section, nav, aside, header, footer, main).
///
/// Sectioning elements are rendered as block-level containers. When in inline
/// conversion mode, their content is rendered inline without block spacing.
/// Otherwise, content is wrapped with proper blank lines to separate from other blocks.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline; block spacing is skipped
/// - **Block mode**: Content is collected, trimmed, and formatted with blank lines
/// - **Empty content**: Empty sections are skipped entirely
///
/// # Implementation Note
///
/// Sectioning elements act as transparent containers—their presence doesn't
/// add any special formatting beyond structural grouping.
pub fn handle(
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
                    super::walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
            return;
        }

        // Collect content in a separate buffer
        let mut content = String::with_capacity(256);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
            }
        }

        // Skip if content is empty
        if content.trim().is_empty() {
            return;
        }

        // Add spacing before the content
        if !output.is_empty() && !output.ends_with("\n\n") {
            output.push_str("\n\n");
        }

        // Append the content
        output.push_str(&content);

        // Ensure proper spacing after the content
        if content.ends_with('\n') && !content.ends_with("\n\n") {
            output.push('\n');
        } else if !content.ends_with('\n') {
            output.push_str("\n\n");
        }
    }
}
