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
    collect_link_label_text, link_accessible_name, link_text_content, node_is_block_level, normalize_link_label,
    normalized_tag_name,
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
    // ~keep Code shows every character as text, so a link in code writes its text and no marks.
    // ~keep A visitor sees the links that it sees outside code, and decides what they write.
    if handler.context.in_code {
        handler.context.inline_data_replaced.set(false);
        let text_start = handler.output.len();
        walk_handles_to_output(data.children.iter().copied(), &mut handler);
        let text = handler.output.split_off(text_start);
        #[cfg(feature = "visitor")]
        let is_shown = handler.context.has_observer() && data.is_shown_to_visitor_in_code(&text, &handler);
        #[cfg(not(feature = "visitor"))]
        let is_shown = false;
        if is_shown {
            emit_link(tag, &data, &text, false, &mut handler);
        } else {
            handler.output.push_str(&text);
        }
        #[cfg(feature = "metadata")]
        record_link_metadata(tag, &data, text.trim(), handler.context);
        return;
    }
    if emit_autolink(&data, &mut handler) || emit_heading_link(&data, &mut handler) {
        return;
    }

    handler.context.inline_data_replaced.set(false);
    let mut label = build_label(&data, &handler);
    apply_label_fallbacks(&data, &mut label, &handler);
    indent_hard_break_continuations(&mut label, handler.context, handler.options);
    let drop_link = data.is_dropped(&label, &handler);
    let emit_deferred = emit_link(tag, &data, &label, drop_link, &mut handler);
    #[cfg(feature = "metadata")]
    record_link_metadata(tag, &data, &label, handler.context);
    if data.emit_blocks_separately && emit_deferred {
        walk_handles_to_output(data.deferred.iter().copied(), &mut handler);
    }
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
    aria_label: Option<Cow<'a, str>>,
    same_page_fragment: bool,
}

impl<'a> LinkData<'a> {
    /// The name of the link apart from its content: its `aria-label`, else its `title`.
    fn accessible_name(&self) -> Option<&str> {
        link_accessible_name(self.aria_label.as_deref(), self.title.as_deref())
    }

    /// Whether the link is left out of the output: its content gives no text, and either the
    /// link points into its own page or the `inline_data_media` choice removed its content.
    ///
    /// ~keep A link into its own page with nothing to read is the icon of a heading permalink
    /// ~keep or of a "back to top" link. It leads nowhere else, so it is left out whatever its
    /// ~keep content is (a graphic, an empty element, nothing) and whether or not it has a name.
    fn is_dropped(&self, label: &str, handler: &HandlerContext<'_>) -> bool {
        label.is_empty() && (handler.context.inline_data_replaced.get() || self.same_page_fragment)
    }

    /// Whether the link has the visible `<address>` form outside code.
    fn is_autolink(&self, options: &ConversionOptions) -> bool {
        // ~keep Deferred tables and dropped data addresses can never use the visible `<href>` form (#120, #490).
        options.autolinks
            && !options.default_title
            && !self.emit_blocks_separately
            && !self.href.is_empty()
            && !self.href_addr_dropped
            && has_uri_scheme(&self.href)
            && (self.raw_text == self.href || (self.href.starts_with("mailto:") && self.raw_text == self.href[7..]))
    }

    /// Whether a visitor gets a `visit_link` call for this link in code, where `text` is what the
    /// content of the link writes there.
    ///
    /// ~keep The calls are those of a link outside code, which makes none for a link in the
    /// ~keep `<address>` form, for a link around one heading, and for a link that is left out (see
    /// ~keep `is_dropped`). A highlighter writes one empty anchor into its own page for each line
    /// ~keep of code. An image writes nothing in code, so a link whose text is empty there asks
    /// ~keep what its content writes outside code (`outside_code`): a link with a label there is
    /// ~keep not left out, and a heading with a text there makes a heading link.
    #[cfg(feature = "visitor")]
    fn is_shown_to_visitor_in_code(&self, text: &str, handler: &HandlerContext<'_>) -> bool {
        if self.is_autolink(handler.options) {
            return false;
        }
        let heading =
            find_single_heading_child(*handler.node_handle, handler.parser).filter(|_| !self.href_addr_dropped);
        if let Some((_, heading_handle)) = heading {
            let writes_text = |outside: &HandlerContext<'_>| {
                heading_link_text(heading_handle, outside).is_some_and(|written| !written.trim().is_empty())
            };
            if !text.trim().is_empty() || outside_code(handler, writes_text) {
                return false;
            }
        }
        let mut label = normalize_link_label(text);
        apply_label_fallbacks(self, &mut label, handler);
        !self.is_dropped(&label, handler) || outside_code(handler, |outside| !build_label(self, outside).is_empty())
    }

    fn new(tag: &'a tl::HTMLTag<'a>, handler: &HandlerContext<'_>) -> Option<Self> {
        let raw_href = tag
            .attributes()
            .get("href")
            .flatten()
            .map(|value| text::decode_attribute_value_cow(&value.as_utf8_str()).into_owned())?;
        let href = handler
            .context
            .resolve_url(&raw_href, handler.node_handle, handler.parser, handler.dom_context)
            .unwrap_or_else(|| raw_href.clone());
        let same_page_fragment = handler.context.own_page.holds(&raw_href, &href);
        // ~keep Empty titles are absent because Markdown serializers drop `""` on reparse.
        let title =
            crate::converter::utility::attributes::decoded_attribute(tag, "title").filter(|value| !value.is_empty());
        let children = handler
            .dom_context
            .children_of(handler.node_handle.get_inner())
            .map_or_else(|| tag.children().top().iter().copied().collect(), ToOwned::to_owned);
        let (inline_label, _, saw_block) = collect_link_label_text(&children, handler.parser, handler.dom_context);
        let text_source = if saw_block {
            link_text_content(&children, handler.parser, handler.dom_context)
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
            aria_label: crate::converter::utility::attributes::decoded_attribute(tag, "aria-label"),
            same_page_fragment,
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

/// What `ask` answers for the link of `handler` when that link is outside code: `ask` gets a
/// handler that writes as outside code and that nothing observes (see
/// `Context::unobserved_outside_code`).
///
/// ~keep The answer comes from the functions that write a link outside code, run once more on
/// ~keep the content of this link. No second set of rules says what a label is, so a rule of the
/// ~keep traversal (a stripped tag, an excluded node, a preset) cannot be missing from the answer.
/// ~keep No state is shared between a link and a link inside it.
#[cfg(feature = "visitor")]
fn outside_code<T>(handler: &HandlerContext<'_>, ask: impl FnOnce(&HandlerContext<'_>) -> T) -> T {
    let context = handler.context.unobserved_outside_code();
    let mut unused = String::new();
    ask(&HandlerContext {
        node_handle: handler.node_handle,
        parser: handler.parser,
        output: &mut unused,
        options: handler.options,
        context: &context,
        depth: handler.depth,
        dom_context: handler.dom_context,
    })
}

/// The text that the one heading of a link writes with the context of `handler`.
fn heading_link_text(heading_handle: tl::NodeHandle, handler: &HandlerContext<'_>) -> Option<String> {
    let heading_context = heading_context(heading_handle, handler)?;
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
    Some(heading_text)
}

/// The context that the one heading of a link is written with.
fn heading_context(heading_handle: tl::NodeHandle, handler: &HandlerContext<'_>) -> Option<Context> {
    let tl::Node::Tag(heading_tag) = heading_handle.get(handler.parser)? else {
        return None;
    };
    let heading_name = normalized_tag_name(heading_tag.name().as_utf8_str());
    Some(Context {
        in_heading: true,
        convert_as_inline: true,
        heading_allow_inline_images: heading_allows_inline_images(
            &heading_name,
            &handler.context.keep_inline_images_in,
        ),
        ..handler.context.clone()
    })
}

fn emit_autolink(data: &LinkData<'_>, handler: &mut HandlerContext<'_>) -> bool {
    if !data.is_autolink(handler.options) {
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
    let Some(heading_text) = heading_link_text(heading_handle, handler) else {
        return false;
    };
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

/// The context that the label of a link is built with outside code.
fn label_context(data: &LinkData<'_>, handler: &HandlerContext<'_>, merge_child_spacing: bool) -> Context {
    Context {
        inline_depth: handler.context.inline_depth + 1,
        in_link: true,
        convert_as_inline: handler.context.convert_as_inline || merge_child_spacing,
        link_allow_inline_images: data.link_allow_inline_images,
        ..handler.context.clone()
    }
}

fn walk_label_content(
    children: &[tl::NodeHandle],
    merge_child_spacing: bool,
    data: &LinkData<'_>,
    handler: &HandlerContext<'_>,
) -> String {
    let link_context = label_context(data, handler, merge_child_spacing);
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
    // ~keep A link whose content gives no text, whatever that content is, is named as a browser
    // ~keep names it: by its `aria-label`, then by its `title`. A named link is also kept when
    // ~keep `inline_data_media` removed its content. A link with no child node at all is named
    // ~keep too: its icon comes from a style sheet. Its own address is the last label, for a link
    // ~keep that has content and no name. A link into its own page is not named: it is left out.
    if label.is_empty() && !data.same_page_fragment {
        if let Some(name) = data.accessible_name() {
            *label = normalize_link_label(name);
        }
    }
    let drop_link = data.is_dropped(label, handler);
    if label.is_empty() && !data.href.is_empty() && !data.children.is_empty() && !drop_link && !data.href_addr_dropped {
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
    if data.href_addr_dropped || context.in_code {
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
    if !context.metadata_wants.links {
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
