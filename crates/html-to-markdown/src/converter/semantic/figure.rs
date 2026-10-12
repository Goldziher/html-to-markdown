//! Handlers for HTML5 figure elements.
//!
//! Processes figure-related semantic elements:
//! - `<figure>` - Self-contained illustration, diagram, photo, code listing, etc.
//! - `<figcaption>` - Caption or legend for a figure
//!
//! Figure elements group an image (or other media) with its associated caption.
//! The Markdown output preserves this relationship through content organization.

#[cfg(feature = "visitor")]
use std::borrow::Cow;

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
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_figure(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        if handler.ctx.convert_as_inline {
            super::walk_tag_children(tag, parser, output, handler);
            return;
        }

        #[cfg(feature = "visitor")]
        if visit_figure_start(tag, node_handle, parser, output, handler) {
            return;
        }

        let figure_start = output.len();

        let mut figure_content = collect_figure_content(tag, parser, handler);

        // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
        // ~keep it in `walk_node`'s pre-block-dispatch strip, since the figure's content is
        // ~keep simply finished here — so this closes its own trailing run the same way
        // ~keep `paragraph.rs` closes its own (issue #464 follow-up). `trim_matches` below
        // ~keep does not treat `\` as trimmable, so it cannot clean this up on its own.
        crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer(
            &mut figure_content,
            handler.options.newline_style,
        );

        figure_content = figure_content.replace("\n![", "![");
        figure_content = figure_content.replace(" ![", "![");

        let trimmed = figure_content.trim_matches(|c| c == '\n' || c == ' ' || c == '\t');
        if !trimmed.is_empty() {
            crate::converter::block::div::push_block(output, handler.options, handler.ctx, trimmed);
        }

        #[cfg(feature = "visitor")]
        visit_figure_end(tag, node_handle, parser, output, figure_start, handler);
    }
}

fn collect_figure_content(tag: &tl::HTMLTag<'_>, parser: &tl::Parser, handler: super::HandlerContext<'_>) -> String {
    let mut content = String::new();
    super::walk_tag_children(tag, parser, &mut content, handler);
    content
}

#[cfg(feature = "visitor")]
fn visit_figure_start(
    tag: &tl::HTMLTag<'_>,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = handler.ctx.visitor.as_ref() else {
        return false;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Figure,
        Cow::Borrowed("figure"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_figure_start(&node_ctx);
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            render_custom_figure(&node_ctx, node_handle, parser, output, &custom, handler.ctx);
            true
        }
        VisitResult::PreserveHtml => {
            output.push_str(&crate::converter::utility::serialization::serialize_node(
                node_handle,
                parser,
            ));
            true
        }
        VisitResult::Error(error) => {
            record_visitor_error(handler.ctx, error);
            true
        }
    }
}

#[cfg(feature = "visitor")]
fn render_custom_figure(
    node_context: &crate::visitor::NodeContext<'_>,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    custom: &str,
    ctx: &super::Context,
) {
    use crate::visitor::VisitResult;

    let start = output.len();
    if !output.is_empty() && !output.ends_with("\n\n") {
        output.push_str("\n\n");
    }
    output.push_str(custom);
    let safe_start = crate::converter::utility::content::floor_char_boundary(output, start.min(output.len()));
    let result = ctx
        .visitor
        .as_ref()
        .expect("visitor available")
        .lock()
        .expect("visitor mutex poisoned")
        .visit_figure_end(node_context, &output[safe_start..]);
    match result {
        VisitResult::Continue => ensure_figure_trailing_blank_line(output),
        VisitResult::Custom(replacement) => replace_figure_output(output, safe_start, &replacement),
        VisitResult::Skip => output.truncate(safe_start),
        VisitResult::PreserveHtml => replace_figure_output(
            output,
            safe_start,
            &crate::converter::utility::serialization::serialize_node(node_handle, parser),
        ),
        VisitResult::Error(error) => record_visitor_error(ctx, error),
    }
}

#[cfg(feature = "visitor")]
fn visit_figure_end(
    tag: &tl::HTMLTag<'_>,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    figure_start: usize,
    handler: super::HandlerContext<'_>,
) {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = handler.ctx.visitor.as_ref() else {
        return;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Figure,
        Cow::Borrowed("figure"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let safe_start = crate::converter::utility::content::floor_char_boundary(output, figure_start.min(output.len()));
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_figure_end(&node_ctx, &output[safe_start..]);
    match result {
        VisitResult::Continue => {}
        VisitResult::Skip => output.truncate(safe_start),
        VisitResult::Custom(custom) => replace_figure_output(output, safe_start, &custom),
        VisitResult::PreserveHtml => replace_figure_output(
            output,
            safe_start,
            &crate::converter::utility::serialization::serialize_node(node_handle, parser),
        ),
        VisitResult::Error(error) => record_visitor_error(handler.ctx, error),
    }
}

#[cfg(feature = "visitor")]
fn ensure_figure_trailing_blank_line(output: &mut String) {
    if !output.ends_with('\n') {
        output.push_str("\n\n");
    } else if !output.ends_with("\n\n") {
        output.push('\n');
    }
}

#[cfg(feature = "visitor")]
fn replace_figure_output(output: &mut String, start: usize, replacement: &str) {
    output.truncate(start);
    output.push_str(replacement);
}

#[cfg(feature = "visitor")]
fn record_visitor_error(ctx: &super::Context, error: String) {
    if ctx.visitor_error.borrow().is_none() {
        *ctx.visitor_error.borrow_mut() = Some(error);
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
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_figcaption(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: super::HandlerContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut text = String::new();
        let children = tag.children();
        {
            let caption_ctx = super::Context {
                text_in_markers: true,
                ..handler.ctx.clone()
            };
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut text,
                    crate::converter::block::container::HandlerContext::new(
                        handler.options,
                        &caption_ctx,
                        handler.depth + 1,
                        handler.dom_ctx,
                    ),
                );
            }
        }

        // ~keep A trailing <br> run with no following sibling has no next dispatch to catch
        // ~keep it in `walk_node`'s pre-block-dispatch strip, since the figcaption's content
        // ~keep is simply finished here — so this closes its own trailing run the same way
        // ~keep `paragraph.rs` closes its own (issue #464 follow-up). Left uncaught, the
        // ~keep stray `\` below ends up between the text and the closing `*`, escaping the
        // ~keep emphasis delimiter instead of merely surviving as a visible character.
        crate::converter::main_helpers::strip_trailing_backslash_breaks_from_fresh_buffer(
            &mut text,
            handler.options.newline_style,
        );

        let text = text.trim().to_owned();
        if text.is_empty() {
            return;
        }

        #[cfg(feature = "visitor")]
        if visit_figcaption(tag, node_handle, parser, output, &text, handler) {
            return;
        }

        separate_caption(output);

        output.push(handler.options.strong_em_symbol);
        output.push_str(&text);
        output.push(handler.options.strong_em_symbol);
        output.push_str("\n\n");
    }
}

#[cfg(feature = "visitor")]
fn visit_figcaption(
    tag: &tl::HTMLTag<'_>,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    text: &str,
    handler: super::HandlerContext<'_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(visitor_handle) = handler.ctx.visitor.as_ref() else {
        return false;
    };
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Figcaption,
        Cow::Borrowed("figcaption"),
        tag,
        handler.depth,
        handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_figcaption(&node_ctx, text);
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            separate_caption(output);
            output.push_str(&custom);
            if !custom.ends_with('\n') {
                output.push_str("\n\n");
            }
            true
        }
        VisitResult::PreserveHtml => {
            output.push_str(&crate::converter::utility::serialization::serialize_node(
                node_handle,
                parser,
            ));
            true
        }
        VisitResult::Error(error) => {
            record_visitor_error(handler.ctx, error);
            true
        }
    }
}

fn separate_caption(output: &mut String) {
    if output.is_empty() {
        return;
    }
    if output.ends_with("```\n") {
        output.push('\n');
        return;
    }
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
    if output.ends_with('\n') && !output.ends_with("\n\n") {
        output.push('\n');
    } else if !output.ends_with('\n') {
        output.push_str("\n\n");
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
    handler: super::HandlerContext<'_>,
) {
    match tag_name {
        "figure" => handle_figure(tag_name, node_handle, parser, output, handler),
        "figcaption" => handle_figcaption(tag_name, node_handle, parser, output, handler),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn figure_caption_separated_from_image() {
        let html = r#"<figure><img src="photo.jpg" alt="Photo"><figcaption>A nice photo</figcaption></figure>"#;
        let result = crate::convert(html, None).unwrap();
        let content = result.content.unwrap_or_default();
        assert!(
            content.contains("![Photo](photo.jpg)"),
            "image should be present: {content}"
        );
        assert!(content.contains("A nice photo"), "caption should be present: {content}");
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        let img_line = lines.iter().position(|l| l.contains("![")).unwrap_or(999);
        let cap_line = lines.iter().position(|l| l.contains("A nice photo")).unwrap_or(999);
        assert!(
            cap_line > img_line,
            "caption should be on a separate line after image, lines: {lines:?}"
        );
    }
}
