//! Link element handler for HTML to Markdown conversion.
//!
//! Handles `<a>` elements including:
//! - Basic link markdown output `[text](href "title")`
//! - Autolinks when text matches href
//! - Links containing heading elements
//! - Complex link content with mixed block/inline elements
//! - Visitor callback integration
//! - Link metadata collection

#[cfg(feature = "metadata")]
use std::collections::BTreeMap;

use crate::converter::Context;
use crate::converter::block::heading::{find_single_heading_child, heading_allows_inline_images, push_heading};
use crate::converter::dom_context::DomContext;
use crate::converter::inline::HandlerContext;
use crate::converter::inline::link::{MarkdownLink, append_markdown_link_in_context, has_uri_scheme};
use crate::converter::main::walk_node;
use crate::converter::media::inline_data_treatment;
use crate::converter::utility::content::{
    collect_link_label_text, get_text_content, node_is_block_level, normalize_link_label, normalized_tag_name,
};
use crate::converter::utility::escaping::escape_link_label;
use crate::options::{ConversionOptions, InlineDataMedia};
use crate::text;
use std::borrow::Cow;

#[cfg(feature = "visitor")]
use crate::converter::utility::serialization::serialize_node;

fn indent_hard_break_continuations(label: &mut String, ctx: &Context, options: &ConversionOptions) {
    if !ctx.in_list_item || !label.contains('\n') {
        return;
    }
    let Some(indent) = crate::converter::list::utils::continuation_indent_string(ctx.list_indent_columns, options)
    else {
        return;
    };
    let original = std::mem::take(label);
    let mut lines = original.split('\n');
    let mut indented = lines.next().unwrap_or_default().to_string();
    for line in lines {
        indented.push('\n');
        if !line.is_empty() {
            // ~keep Link-label normalization collapses the content-column whitespace after a
            // ~keep hard break to one space. Restore the enclosing item's actual column so the
            // ~keep continuation cannot become a new block and split the link (issue #678).
            indented.push_str(&indent);
            indented.push_str(line.trim_start_matches([' ', '\t']));
        }
    }
    *label = indented;
}

/// Handle an `<a>` (link) element and convert to Markdown.
///
/// This handler processes link elements including:
/// - Extracting href and title attributes
/// - Detecting autolinks (where text equals href)
/// - Handling links that contain heading elements
/// - Processing complex link content (mixed block/inline)
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Collecting link metadata when the metadata feature is enabled
/// - Generating appropriate markdown link output
pub fn handle_link(tag: &tl::HTMLTag, mut handler: HandlerContext<'_>) {
    let Some(data) = LinkData::new(tag, &handler) else {
        walk_handles_to_output(tag.children().top().iter().copied(), &mut handler);
        return;
    };
    if handler.context.in_code {
        emit_code_link(tag, &data, &mut handler);
        return;
    }
    if emit_autolink(&data, &mut handler) || emit_heading_link(&data, &mut handler) {
        return;
    }

    handler.context.inline_data_replaced.set(false);
    let mut label = build_label(&data, &handler);
    apply_label_fallbacks(&data, &mut label, &handler);
    indent_hard_break_continuations(&mut label, handler.context, handler.options);
    let drop_link = label.is_empty()
        && handler.context.inline_data_replaced.get()
        && handler.options.inline_data_media == InlineDataMedia::DropElement;
    let emit_deferred = emit_link(tag, &data, &label, drop_link, &mut handler);
    #[cfg(feature = "metadata")]
    record_link_metadata(tag, &data, &label, handler.context);
    if data.emit_blocks_separately && emit_deferred {
        walk_handles_to_output(data.deferred.iter().copied(), &mut handler);
    }
}

fn emit_code_link(tag: &tl::HTMLTag<'_>, data: &LinkData<'_>, handler: &mut HandlerContext<'_>) {
    let context = Context {
        inline_code_links: None,
        ..handler.context.clone()
    };
    let mut raw = collect_code_link_text(data, &context, handler);
    if (handler.context.in_table_cell || handler.context.in_heading) && followed_by_break(handler) {
        raw.truncate(raw.trim_end().len());
    }
    if let Some(links) = handler.context.inline_code_links.as_ref().filter(|_| {
        !handler.context.in_code_block && !data.href.is_empty() && !data.href_addr_dropped && !raw.is_empty()
    }) {
        let mut label = String::new();
        crate::converter::handlers::code_block::emit_inline_code(
            &raw,
            &mut label,
            handler.options,
            handler.node_handle,
            handler.parser,
            handler.dom_context,
        );
        let mut markdown = String::new();
        let mut link_handler = HandlerContext::new((
            handler.node_handle,
            handler.parser,
            &mut markdown,
            handler.options,
            &context,
            handler.depth,
            handler.dom_context,
        ));
        emit_link(tag, data, &label, false, &mut link_handler);
        if !markdown.is_empty() {
            let start = handler.output.len();
            handler.output.push_str(&raw);
            links.borrow_mut().push(crate::converter::context::InlineCodeLink {
                range: start..handler.output.len(),
                markdown,
            });
        }
    } else if handler.context.inline_code_links.is_some() {
        handler.output.push_str(&raw);
    } else {
        emit_link(tag, data, &raw, false, handler);
    }
    #[cfg(feature = "metadata")]
    record_link_metadata(tag, data, raw.trim(), handler.context);
}

fn followed_by_break(handler: &HandlerContext<'_>) -> bool {
    let id = handler.node_handle.get_inner();
    let next = handler
        .dom_context
        .parent_of(id)
        .and_then(|parent| handler.dom_context.children_of(parent))
        .and_then(|siblings| {
            handler
                .dom_context
                .get_sibling_index(id)
                .and_then(|index| siblings.get(index + 1))
        });
    matches!(next.and_then(|handle| handle.get(handler.parser)), Some(tl::Node::Tag(tag)) if tag.name().as_utf8_str().eq_ignore_ascii_case("br"))
}

fn collect_code_link_text(data: &LinkData<'_>, context: &Context, handler: &HandlerContext<'_>) -> String {
    let mut raw = String::new();
    for child in &data.children {
        walk_node(
            child,
            handler.parser,
            &mut raw,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
    raw
}

struct LinkData<'a> {
    href: String,
    title: Option<Cow<'a, str>>,
    children: Vec<tl::NodeHandle>,
    inline_label: String,
    raw_text: String,
    inline_children: Vec<tl::NodeHandle>,
    deferred: Vec<tl::NodeHandle>,
    emit_blocks_separately: bool,
    href_addr_dropped: bool,
    link_allow_inline_images: bool,
    saw_block: bool,
    accessible_name: Option<Cow<'a, str>>,
    empty_span_content: bool,
}

impl<'a> LinkData<'a> {
    fn new(tag: &'a tl::HTMLTag<'a>, handler: &HandlerContext<'_>) -> Option<Self> {
        let href = tag
            .attributes()
            .get("href")
            .flatten()
            .map(|value| text::decode_attribute_value_cow(&value.as_utf8_str()).into_owned())
            .map(|href| {
                handler
                    .context
                    .resolve_link_url(&href, handler.node_handle, handler.parser, handler.dom_context)
                    .unwrap_or(href)
            })?;
        // ~keep Empty titles are absent because Markdown serializers drop `""` on reparse.
        let title =
            crate::converter::utility::attributes::decoded_attribute(tag, "title").filter(|value| !value.is_empty());
        let children = handler
            .dom_context
            .children_of(handler.node_handle.get_inner())
            .map_or_else(|| tag.children().top().iter().copied().collect(), ToOwned::to_owned);
        let (inline_label, _, saw_block) = collect_link_label_text(&children, handler.parser, handler.dom_context);
        let text_source = if saw_block {
            get_text_content(handler.node_handle, handler.parser, handler.dom_context)
        } else {
            inline_label.clone()
        };
        let raw_text = text::normalize_whitespace_cow(&text_source).trim().to_string();
        let (inline_children, deferred) = partition_link_children(&children, handler.parser, handler.dom_context);
        Some(Self {
            href_addr_dropped: matches!(
                inline_data_treatment(handler.options.inline_data_media, &href),
                InlineDataMedia::AltTextOnly | InlineDataMedia::DropElement
            ),
            emit_blocks_separately: should_defer_table_blocks(
                handler.context,
                &deferred,
                handler.parser,
                handler.dom_context,
            ),
            link_allow_inline_images: handler.context.keep_inline_images_in.contains("a"),
            accessible_name: crate::converter::utility::attributes::decoded_attribute(tag, "aria-label")
                .filter(|value| !value.trim().is_empty())
                .or_else(|| title.clone()),
            empty_span_content: !children.is_empty()
                && children
                    .iter()
                    .all(|child| empty_text_wrapper_child(child, handler.parser)),
            href,
            title,
            children,
            inline_label,
            raw_text,
            inline_children,
            deferred,
            saw_block,
        })
    }
}

// ~keep Empty presentational wrappers carry no textual or media fallback; preserve their destination
// ~keep without inserting URL words into document content (#771). Icon fonts and images
// ~keep still use the link-name/address fallback required by #774 and #775.
fn empty_text_wrapper_child(handle: &tl::NodeHandle, parser: &tl::Parser<'_>) -> bool {
    let mut pending = vec![*handle];
    while let Some(child) = pending.pop() {
        match child.get(parser) {
            Some(tl::Node::Raw(raw)) if text::decode_html_entities_cow(&raw.as_utf8_str()).trim().is_empty() => {}
            Some(tl::Node::Comment(_)) => {}
            Some(tl::Node::Tag(tag))
                if matches!(
                    normalized_tag_name(tag.name().as_utf8_str()).as_ref(),
                    "span"
                        | "b"
                        | "strong"
                        | "em"
                        | "mark"
                        | "del"
                        | "s"
                        | "strike"
                        | "ins"
                        | "u"
                        | "small"
                        | "sub"
                        | "sup"
                        | "code"
                        | "kbd"
                        | "samp"
                ) || plain_wrapper(tag) =>
            {
                pending.extend(tag.children().top().iter().copied());
            }
            _ => return false,
        }
    }
    true
}

// ~keep An `i` or a `div` with a class is an icon that a style sheet draws: #774 and the vote
// ~keep link of the #121 fixture name it by the link address. With no class it shows nothing,
// ~keep as an empty `span` does.
fn plain_wrapper(tag: &tl::HTMLTag<'_>) -> bool {
    tag.attributes().get("class").is_none()
        && matches!(normalized_tag_name(tag.name().as_utf8_str()).as_ref(), "i" | "div")
}

fn emit_autolink(data: &LinkData<'_>, handler: &mut HandlerContext<'_>) -> bool {
    // ~keep Deferred tables and dropped data addresses can never use the visible `<href>` form (#120, #490).
    let autolink = handler.options.autolinks
        && !handler.options.default_title
        && !data.emit_blocks_separately
        && !data.href.is_empty()
        && !data.href_addr_dropped
        && has_uri_scheme(&data.href)
        && (data.raw_text == data.href || (data.href.starts_with("mailto:") && data.raw_text == data.href[7..]));
    if !autolink {
        return false;
    }
    handler.output.push('<');
    if data.href.starts_with("mailto:") && data.raw_text == data.href[7..] {
        handler.output.push_str(&data.raw_text);
    } else {
        handler.output.push_str(&data.href);
    }
    handler.output.push('>');
    true
}

fn emit_heading_link(data: &LinkData<'_>, handler: &mut HandlerContext<'_>) -> bool {
    if data.href_addr_dropped {
        return false;
    }
    let Some((level, heading_handle)) = find_single_heading_child(*handler.node_handle, handler.parser) else {
        return false;
    };
    let Some(tl::Node::Tag(heading_tag)) = heading_handle.get(handler.parser) else {
        return false;
    };
    let heading_name = normalized_tag_name(heading_tag.name().as_utf8_str());
    let heading_context = Context {
        in_heading: true,
        convert_as_inline: true,
        heading_allow_inline_images: heading_allows_inline_images(
            &heading_name,
            &handler.context.keep_inline_images_in,
        ),
        ..handler.context.clone()
    };
    let mut heading_text = String::new();
    walk_node(
        &heading_handle,
        handler.parser,
        &mut heading_text,
        crate::converter::block::container::HandlerContext::new(
            handler.options,
            &heading_context,
            handler.depth + 1,
            handler.dom_context,
        ),
    );
    let heading_text = heading_text.trim();
    if heading_text.is_empty() {
        return false;
    }
    let mut link = String::new();
    append_link(
        &mut link,
        data,
        &escape_link_label(heading_text),
        &data.raw_text,
        handler.options,
        handler.context,
    );
    push_heading(handler.output, handler.context, handler.options, level, &link);
    true
}

fn build_label(data: &LinkData<'_>, handler: &HandlerContext<'_>) -> String {
    if data.emit_blocks_separately {
        return walk_label(&data.inline_children, false, data, handler);
    }
    if data.saw_block {
        let content = walk_label_content(&data.children, true, data, handler);
        return if content.trim().is_empty() {
            normalize_link_label(&data.inline_label)
        } else {
            normalize_link_label(&content)
        };
    }
    walk_label(&data.children, false, data, handler)
}

fn walk_label(
    children: &[tl::NodeHandle],
    convert_as_inline: bool,
    data: &LinkData<'_>,
    handler: &HandlerContext<'_>,
) -> String {
    normalize_link_label(&walk_label_content(children, convert_as_inline, data, handler))
}

fn walk_label_content(
    children: &[tl::NodeHandle],
    merge_child_spacing: bool,
    data: &LinkData<'_>,
    handler: &HandlerContext<'_>,
) -> String {
    let link_context = Context {
        inline_depth: handler.context.inline_depth + 1,
        in_link: true,
        convert_as_inline: handler.context.convert_as_inline || merge_child_spacing,
        link_allow_inline_images: data.link_allow_inline_images,
        ..handler.context.clone()
    };
    let mut content = String::new();
    if !merge_child_spacing {
        for child in children {
            walk_node(
                child,
                handler.parser,
                &mut content,
                crate::converter::block::container::HandlerContext::new(
                    handler.options,
                    &link_context,
                    handler.depth + 1,
                    handler.dom_context,
                ),
            );
        }
        return content;
    }
    for child in children {
        let mut child_output = String::new();
        walk_node(
            child,
            handler.parser,
            &mut child_output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                &link_context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
        if merge_child_spacing && needs_label_space(&content, &child_output) {
            content.push(' ');
        }
        content.push_str(&child_output);
    }
    content
}

fn needs_label_space(content: &str, child: &str) -> bool {
    !child.trim().is_empty()
        && !content.is_empty()
        && !content.chars().last().is_none_or(char::is_whitespace)
        && !child.chars().next().is_none_or(char::is_whitespace)
}

fn apply_label_fallbacks(data: &LinkData<'_>, label: &mut String, handler: &HandlerContext<'_>) {
    // ~keep Deferred table text must not be duplicated into the label (#490).
    if !data.emit_blocks_separately && label.is_empty() && !data.raw_text.is_empty() {
        *label = normalize_link_label(&data.raw_text);
    }
    let drop_link = label.is_empty()
        && handler.context.inline_data_replaced.get()
        && handler.options.inline_data_media == InlineDataMedia::DropElement;
    if label.is_empty() && !drop_link && !data.href_addr_dropped {
        if let Some(name) = data.accessible_name.as_deref() {
            *label = normalize_link_label(name);
        }
    }
    if label.is_empty()
        && !data.href.is_empty()
        && !data.children.is_empty()
        && !data.empty_span_content
        && !drop_link
        && !data.href_addr_dropped
    {
        *label = text::escape(
            &data.href,
            handler.options.escape_misc,
            handler.options.escape_asterisks,
            handler.options.escape_underscores,
            handler.options.escape_ascii,
        )
        .into_owned();
    }
    if label == "^" && data.href.starts_with('#') {
        *label = "↑".to_string();
    }
}

#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
fn emit_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData<'_>,
    label: &str,
    drop_link: bool,
    handler: &mut HandlerContext<'_>,
) -> bool {
    if drop_link {
        return true;
    }
    #[cfg(feature = "visitor")]
    if let Some(visitor) = handler.context.visitor.clone() {
        return visit_link(tag, data, label, &visitor, handler);
    }
    write_link(handler.output, data, label, handler.options, handler.context);
    true
}

fn write_link(output: &mut String, data: &LinkData<'_>, label: &str, options: &ConversionOptions, context: &Context) {
    if data.href_addr_dropped || data.href.is_empty() || context.in_code_block {
        output.push_str(label);
        return;
    }
    append_link(output, data, &escape_link_label(label), label, options, context);
}

fn append_link(
    output: &mut String,
    data: &LinkData<'_>,
    escaped_label: &str,
    raw_text: &str,
    options: &ConversionOptions,
    context: &Context,
) {
    append_markdown_link_in_context(
        output,
        &MarkdownLink {
            label: escaped_label,
            href: &data.href,
            title: data.title.as_deref(),
            raw_text,
        },
        options,
        context.reference_collector.as_ref(),
        context.in_table_cell,
    );
}

#[cfg(feature = "visitor")]
fn visit_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData<'_>,
    label: &str,
    visitor: &crate::visitor::VisitorHandle,
    handler: &mut HandlerContext<'_>,
) -> bool {
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        NodeType::Link,
        Cow::Borrowed("a"),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        true,
    );
    let result = visitor.lock().expect("visitor mutex poisoned").visit_link(
        &node_context,
        &data.href,
        label,
        data.title.as_deref(),
    );
    match result {
        VisitResult::Continue => write_link(handler.output, data, label, handler.options, handler.context),
        VisitResult::Custom(custom) => {
            if let Some(collector) = handler.context.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(Some(&custom));
            }
            handler.output.push_str(&custom);
        }
        VisitResult::Skip => {
            if let Some(collector) = handler.context.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(None);
            }
            return false;
        }
        VisitResult::Error(error) => {
            if let Some(collector) = handler.context.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(None);
            }
            if handler.context.visitor_error.borrow().is_none() {
                *handler.context.visitor_error.borrow_mut() = Some(error);
            }
        }
        VisitResult::PreserveHtml => {
            let html = serialize_node(handler.node_handle, handler.parser);
            if let Some(collector) = handler.context.structure_collector.as_ref() {
                collector.borrow_mut().replace_current_element(Some(&html));
            }
            handler.output.push_str(&html);
            return false;
        }
    }
    true
}

#[cfg(feature = "metadata")]
fn record_link_metadata(tag: &tl::HTMLTag<'_>, data: &LinkData<'_>, label: &str, context: &Context) {
    if !context.metadata_wants_links {
        return;
    }
    let Some(collector) = context.metadata_collector.as_ref() else {
        return;
    };
    let rel = tag
        .attributes()
        .get("rel")
        .flatten()
        .map(|value| value.as_utf8_str().to_string());
    let attributes = tag
        .attributes()
        .iter()
        .filter(|(key, _)| key.as_ref() != "href")
        .map(|(key, value)| {
            (
                key.to_string(),
                value.map(|value| value.to_string()).unwrap_or_default(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    collector.borrow_mut().add_link(
        data.href.clone(),
        label.to_string(),
        data.title.as_deref().map(str::to_string),
        rel,
        attributes,
    );
}

fn walk_handles_to_output(children: impl Iterator<Item = tl::NodeHandle>, handler: &mut HandlerContext<'_>) {
    for child in children {
        walk_node(
            &child,
            handler.parser,
            handler.output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                handler.context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
}

/// Partition an anchor's DIRECT children into (inline, block) using `node_is_block_level`
/// (issue #490).
///
/// ~keep Deliberately direct children only, not `collect_link_label_text`'s topmost block
/// ~keep *descendants* (which can sit several inline wrappers deep) -- every byte of the
/// ~keep anchor's content must land in exactly one of the two halves, which is also what
/// ~keep catches a table buried under a block wrapper (`<a><div><table>...`).
fn partition_link_children(
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> (Vec<tl::NodeHandle>, Vec<tl::NodeHandle>) {
    children
        .iter()
        .copied()
        .partition(|child| !node_is_block_level(child, parser, dom_ctx))
}

/// Decide whether an anchor's deferred (direct block) children should be rendered as
/// separate blocks after the link, instead of being walked into the inline label
/// (issue #490).
///
/// ~keep A wrapped `<table>` crushed a whole GFM table into the link label: the label would
/// ~keep contain literal `|`/`\n` that either get escaped into noise or, unescaped, corrupt
/// ~keep the OUTER row on reparse. Emitting the table as a separate block after the link is
/// ~keep the only shape that round-trips. Gated on an actual `<table>` inside a deferred
/// ~keep subtree so the common case (a `<p>`/`<div>`-only anchor) pays nothing and behaves
/// ~keep exactly as before. `!ctx.convert_as_inline` and `!ctx.in_heading` are load-bearing,
/// ~keep not polish: a heading's `normalize_heading_text` folds `\n` to spaces, so a block
/// ~keep table inside a heading is no better than the crushed-label bug; an inline context (a
/// ~keep data cell's own label, or this link nested inside an outer link's label) has nowhere
/// ~keep to put a deferred block at all.
///
/// ~keep A layout cell is the exception among inline contexts (issue #503): it converts as
/// ~keep inline only because its row becomes one list item, and that item's text already
/// ~keep holds a bare nested table. Refusing there sent the table through the label, where
/// ~keep `escape_link_label` turned every inner link into `\[One\](/one)` text.
fn should_defer_table_blocks(
    ctx: &Context,
    deferred: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> bool {
    (!ctx.convert_as_inline || ctx.in_layout_cell)
        && !ctx.in_heading
        && deferred.iter().any(|handle| subtree_has_table(handle, parser, dom_ctx))
}

/// Short-circuiting DFS: does `handle`'s subtree (itself or any descendant) contain a
/// `<table>`?
///
/// ~keep Checked only against a deferred block child of an `<a>` (issue #490), so the cost
/// ~keep is paid only when the anchor actually has a block child, and the walk stops at the
/// ~keep first `<table>` found regardless of subtree size.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn subtree_has_table(handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
        return false;
    };
    if normalized_tag_name(tag.name().as_utf8_str()) == "table" {
        return true;
    }
    if let Some(children) = dom_ctx.children_of(handle.get_inner()) {
        children.iter().any(|child| subtree_has_table(child, parser, dom_ctx))
    } else {
        tag.children()
            .top()
            .iter()
            .any(|child| subtree_has_table(child, parser, dom_ctx))
    }
}
