//! Handlers for semantic inline elements with attributes.
//!
//! Processes semantic inline elements that often carry semantic meaning through attributes:
//! - `<cite>` - Citation/reference to a source
//! - `<q>` - Inline quotation
//! - `<abbr>` - Abbreviation with optional title explanation
//! - `<dfn>` - Definition of a term
//! - `<time>` - Machine-readable date/time
//! - `<data>` - Machine-readable data value
//!
//! These elements carry semantic meaning in HTML5 but often have minimal
//! visual distinction in rendered Markdown. Some are formatted with emphasis
//! or have their attributes included in the output.

use crate::converter::utility::content::chomp_inline;

/// Handles the `<dfn>` element.
///
/// A dfn element marks a term that is being defined. The content represents
/// the term, and its definition would typically appear in surrounding context.
/// It is rendered as emphasized (italic) text.
///
/// # Behavior
///
/// - Content is collected from children
/// - Non-empty content is wrapped with the configured emphasis symbol (default: `*`)
/// - Inline suffix handling is applied (e.g., footnote references)
pub fn handle_dfn(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        handler.ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        let (prefix, suffix, trimmed) = chomp_inline(&content);
        if !trimmed.is_empty() {
            output.push_str(prefix);
            output.push(handler.options.strong_em_symbol);
            output.push_str(trimmed);
            output.push(handler.options.strong_em_symbol);
            append_inline_suffix(
                output,
                suffix,
                !trimmed.is_empty(),
                node_handle,
                parser,
                handler.dom_ctx,
            );
        }
    }
}

/// Handles the `<abbr>` element.
///
/// An abbr element marks an abbreviation or acronym. The `title` attribute
/// provides the expansion of the abbreviation, which is appended in parentheses
/// if present.
///
/// # Behavior
///
/// - Content is collected from children
/// - Non-empty content is output as-is
/// - If `title` attribute exists, it is appended in parentheses: `abbr (title)`
///
/// # Example
///
/// ```html
/// <abbr title="HyperText Markup Language">HTML</abbr>
/// ```
///
/// Produces: `HTML (HyperText Markup Language)`
pub fn handle_abbr(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        handler.ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            output.push_str(trimmed);

            if let Some(title) = crate::converter::utility::attributes::decoded_attribute(tag, "title") {
                let trimmed_title = title.trim();
                if !trimmed_title.is_empty() {
                    output.push_str(" (");
                    output.push_str(trimmed_title);
                    output.push(')');
                }
            }
        }
    }
}

/// Handles the `<time>` and `<data>` elements.
///
/// Time and data elements contain machine-readable content in their attributes
/// and human-readable content in their text. For Markdown purposes, we output
/// only the human-readable text content, as Markdown doesn't have a way to
/// preserve machine-readable metadata.
///
/// # Behavior
///
/// - Content is extracted from children and output as-is
/// - Attributes (datetime, value) are not rendered in Markdown output
pub fn handle_time_data(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
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
    }
}

/// Handles the `<cite>` element.
///
/// A cite element marks the title of a cited work (book, article, website, etc.).
/// It is rendered as emphasized (italic) text in block mode, or as plain text in inline mode.
///
/// # Behavior
///
/// - **Block mode**: Content is wrapped with emphasis markers (default: `*`)
/// - **Inline mode**: Content is output as-is without formatting
pub fn handle_cite(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        handler.ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            if handler.ctx.convert_as_inline {
                output.push_str(trimmed);
            } else {
                output.push('*');
                output.push_str(trimmed);
                output.push('*');
            }
        }
    }
}

/// Handles the `<q>` element.
///
/// A q element marks an inline quotation. In Markdown, it is rendered as
/// quoted text enclosed in double quotes. Backslashes and quotes within
/// the content are escaped.
///
/// # Behavior
///
/// - **Block mode**: Content is wrapped in escaped double quotes: `"content"`
/// - **Inline mode**: Content is output as-is without quotes
///
/// # Escaping
///
/// Internal backslashes and double quotes are escaped:
/// - `\` → `\\`
/// - `"` → `\"`
pub fn handle_q(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        if handle_collected_q(tag, parser, output, handler) {
            return;
        }
        let mut content = String::with_capacity(32);
        let children = tag.children();
        {
            let q_ctx = super::Context {
                text_in_markers: true,
                ..handler.ctx.clone()
            };
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        &q_ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            // ~keep issue #481: `<q>` wraps in quotes, which a whitespace-only body has
            // ~keep nothing to wrap -- but the body is still the word separator the source
            // ~keep wrote, and dropping it outright joined the words either side together.
            if !content.is_empty() {
                crate::converter::inline::wrapped::emit_whitespace_only_inline_body(&content, output);
            }
            return;
        }
        output.push('"');
        output.push_str(trimmed);
        output.push('"');
    }
}

fn handle_collected_q(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) -> bool {
    let Some(links) = handler.ctx.inline_code_links.as_ref() else {
        return false;
    };
    let link_count = links.borrow().len();
    let start = output.len();
    output.push('"');
    super::walk_tag_children(tag, parser, output, handler);
    if links.borrow().len() == link_count {
        let body = output[start + 1..].to_string();
        output.truncate(start);
        if body.trim().is_empty() {
            crate::converter::inline::wrapped::emit_whitespace_only_inline_body(&body, output);
        } else {
            output.push('"');
            output.push_str(body.trim());
            output.push('"');
        }
    } else {
        // ~keep Quoted links were collected in this buffer; preserve their offsets.
        output.push('"');
    }
    true
}

/// Dispatcher for semantic inline attribute elements.
///
/// Routes `<cite>`, `<q>`, `<abbr>`, `<dfn>`, `<time>`, and `<data>` elements
/// to their respective handlers.
pub fn handle(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    match tag_name {
        "dfn" => handle_dfn(tag_name, node_handle, parser, output, handler),
        "abbr" => handle_abbr(tag_name, node_handle, parser, output, handler),
        "time" | "data" => handle_time_data(tag_name, node_handle, parser, output, handler),
        "cite" => handle_cite(tag_name, node_handle, parser, output, handler),
        "q" => handle_q(tag_name, node_handle, parser, output, handler),
        _ => {}
    }
}

/// Appends inline suffix to the output.
///
/// This is a placeholder for integrating with other inline formatting systems
/// (e.g., footnote references). For now, it simply outputs the suffix.
fn append_inline_suffix(
    output: &mut String,
    suffix: &str,
    _is_nonempty: bool,
    _node_handle: &tl::NodeHandle,
    _parser: &tl::Parser,
    _dom_ctx: &super::DomContext,
) {
    output.push_str(suffix);
}
