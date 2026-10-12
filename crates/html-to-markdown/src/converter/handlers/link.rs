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
use crate::converter::append_inline_suffix;
use crate::converter::block::heading::{find_single_heading_child, heading_allows_inline_images, push_heading};
use crate::converter::dom_context::DomContext;
use crate::converter::inline::HandlerContext;
use crate::converter::inline::link::{MarkdownLink, append_markdown_link_in_context, has_uri_scheme};
use crate::converter::main::walk_node;
use crate::converter::media::inline_data_treatment;
use crate::converter::text_node::is_fresh_block_start;
use crate::converter::utility::content::{
    collect_link_label_text, indent_continuation_lines, keeps_source_white_space, link_accessible_name,
    link_label_edges, link_text_content, link_text_edges, node_is_block_level, normalize_link_label,
    normalized_tag_name, push_inline_prefix,
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
    indent_continuation_lines(label, &indent);
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
    // ~keep A link in a code span stays a link: the code marks go inside its label (#813).
    let span_links = handler.context.inline_code_links.clone();
    if let Some(links) = span_links.filter(|_| !handler.context.in_code_block) {
        emit_code_link(tag, &data, &links, &mut handler);
        return;
    }
    // ~keep A code block shows every character as text, so a link there writes its text and no
    // ~keep marks (#781). A visitor sees the links that it sees outside code, and decides what
    // ~keep they write.
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
            emit_link(tag, &data, &text, &LabelEdges::NONE, false, &mut handler);
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
    let content = build_label_content(&data, &handler);
    let (prefix, suffix, trimmed) = if keeps_source_white_space(handler.options, handler.context) {
        ("", "", content.as_str())
    } else {
        link_label_edges(&content)
    };
    let edges = LabelEdges {
        prefix,
        suffix,
        has_text: !trimmed.is_empty(),
    };
    let mut label = normalize_link_label(trimmed);
    apply_label_fallbacks(&data, &mut label, &handler);
    indent_hard_break_continuations(&mut label, handler.context, handler.options);
    let drop_link = data.is_dropped(&label, &handler);
    let emit_deferred = emit_link(tag, &data, &label, &edges, drop_link, &mut handler);
    #[cfg(feature = "metadata")]
    record_link_metadata(tag, &data, &label, handler.context);
    if data.emit_blocks_separately && emit_deferred {
        walk_handles_to_output(data.deferred.iter().copied(), &mut handler);
    }
}

/// Write a link in a code span: its text goes to the span, and `links` records the range of
/// that text with the link that replaces it when the span is written.
fn emit_code_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData<'_>,
    links: &std::cell::RefCell<Vec<crate::converter::context::InlineCodeLink>>,
    handler: &mut HandlerContext<'_>,
) {
    let context = Context {
        inline_code_links: None,
        ..handler.context.clone()
    };
    let mut raw = collect_code_link_text(data, &context, handler);
    if (handler.context.in_table_cell || handler.context.in_heading) && followed_by_break(handler) {
        raw.truncate(raw.trim_end().len());
    }
    // ~keep A link to a place in its own page whose content gives no text is left out in a code
    // ~keep span as it is outside code: its white space stays text of the span.
    let left_out = data.leads_nowhere_else() && normalize_link_label(&raw).is_empty();
    if !data.href.is_empty() && !data.href_addr_dropped && !raw.is_empty() && !left_out {
        // ~keep The label is a finished code span, and the link marks are outside it.
        let link_context = Context {
            in_code: false,
            ..context
        };
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
            &link_context,
            handler.depth,
            handler.dom_context,
        ));
        emit_link(tag, data, &label, &LabelEdges::NONE, false, &mut link_handler);
        if !markdown.is_empty() {
            let start = handler.output.len();
            handler.output.push_str(&raw);
            links.borrow_mut().push(crate::converter::context::InlineCodeLink {
                range: start..handler.output.len(),
                markdown,
            });
        }
    } else {
        handler.output.push_str(&raw);
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

/// The white space that the label of a link had at its two ends (see [`link_label_edges`]).
///
/// ~keep It belongs to the link that the converter writes itself, so only [`write_with_edges`]
/// ~keep writes it. A link that is left out, and a link that a visitor skips, replaces or keeps
/// ~keep as HTML, has no label of the converter: it writes what it wrote before the label gave
/// ~keep up its white space.
struct LabelEdges<'a> {
    prefix: &'a str,
    suffix: &'a str,
    has_text: bool,
}

impl LabelEdges<'_> {
    /// A label that gives up no white space: the text of a link in code.
    const NONE: Self = Self {
        prefix: "",
        suffix: "",
        has_text: false,
    };
}

struct LinkData<'a> {
    href: String,
    title: Option<Cow<'a, str>>,
    children: Vec<tl::NodeHandle>,
    inline_label: String,
    raw_text: String,
    /// The white space at the two ends of the text that `raw_text` is trimmed from.
    text_edges: (&'static str, &'static str),
    inline_children: Vec<tl::NodeHandle>,
    deferred: Vec<tl::NodeHandle>,
    emit_blocks_separately: bool,
    href_addr_dropped: bool,
    link_allow_inline_images: bool,
    saw_block: bool,
    aria_label: Option<Cow<'a, str>>,
    same_page_fragment: bool,
    /// The address has a fragment that is not empty: it names a place.
    names_place: bool,
    empty_span_content: bool,
}

impl<'a> LinkData<'a> {
    /// The name of the link apart from its content: its `aria-label`, else its `title`.
    fn accessible_name(&self) -> Option<&str> {
        link_accessible_name(self.aria_label.as_deref(), self.title.as_deref())
    }

    /// Whether the link is left out of the output: its content gives no text, and either the
    /// link points into its own page or the `inline_data_media` choice removed its content.
    ///
    /// ~keep See `leads_nowhere_else` for the link into its own page.
    fn is_dropped(&self, label: &str, handler: &HandlerContext<'_>) -> bool {
        let content_removed = handler.context.inline_data_replaced.get()
            && handler.options.inline_data_media == InlineDataMedia::DropElement;
        label.is_empty() && (content_removed || self.leads_nowhere_else())
    }

    /// Whether the link points into its own page and is left out when its content gives no text.
    ///
    /// ~keep A link to a place in the page (`#id`) with nothing to read is a heading permalink,
    /// ~keep a "back to top" link or the line anchor of a highlighter. It is left out whatever its
    /// ~keep content is and whether or not it has a name (pull request #764). A link that names no place
    /// ~keep (`#` alone) is a control that a script runs: it is kept, with its name or an empty
    /// ~keep label, when it has no child or only empty text wrappers (#771), and left out when
    /// ~keep it holds a graphic or an icon (#750).
    const fn leads_nowhere_else(&self) -> bool {
        self.same_page_fragment && (self.names_place || !(self.empty_span_content || self.children.is_empty()))
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
    /// ~keep of code. An image with no alt text writes nothing in code, so a link whose text is empty asks
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
        !self.is_dropped(&label, handler)
            || outside_code(handler, |outside| {
                !normalize_link_label(&build_label_content(self, outside)).is_empty()
            })
    }

    fn new(tag: &'a tl::HTMLTag<'a>, handler: &HandlerContext<'_>) -> Option<Self> {
        let raw_href = tag
            .attributes()
            .get("href")
            .flatten()
            .map(|value| text::decode_attribute_value_cow(&value.as_utf8_str()).into_owned())?;
        let href = handler
            .context
            .resolve_link_url(&raw_href, handler.node_handle, handler.parser, handler.dom_context)
            .unwrap_or_else(|| raw_href.clone());
        let same_page_fragment = handler.context.own_page.holds(&raw_href, &href);
        // ~keep Empty titles are absent because Markdown serializers drop `""` on reparse.
        let title =
            crate::converter::utility::attributes::decoded_attribute(tag, "title").filter(|value| !value.is_empty());
        let children = handler
            .dom_context
            .children_of(handler.node_handle.get_inner())
            .map_or_else(|| tag.children().top().iter().copied().collect(), ToOwned::to_owned);
        let hidden_content = handler.options.hidden_content;
        let (inline_text, _, saw_block) =
            collect_link_label_text(&children, handler.parser, handler.dom_context, hidden_content);
        let text_source = if saw_block {
            link_text_content(&children, handler.parser, handler.dom_context, hidden_content)
        } else {
            inline_text.clone()
        };
        let inline_label = inline_text.text;
        let raw_text = text::normalize_whitespace_cow(&text_source.text).trim().to_string();
        let (inline_children, deferred) = partition_link_children(&children, handler.parser, handler.dom_context);
        let holds_element = children
            .iter()
            .any(|child| matches!(child.get(handler.parser), Some(tl::Node::Tag(_))));
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
            names_place: href.split_once('#').is_some_and(|(_, fragment)| !fragment.is_empty()),
            // ~keep White space alone is content of a link to another page, not an empty wrapper:
            // ~keep the address names that link. A link into its own page gets no such label, so
            // ~keep white space and comments are no content there, as for a link with no child
            // ~keep (#851).
            empty_span_content: !children.is_empty()
                && (same_page_fragment || holds_element)
                && children
                    .iter()
                    .all(|child| empty_text_wrapper_child(child, handler.parser)),
            href,
            title,
            children,
            inline_label,
            text_edges: link_text_edges(&text_source),
            raw_text,
            inline_children,
            deferred,
            saw_block,
        })
    }
}

// ~keep Empty presentational wrappers carry no textual or media fallback; preserve their destination
// ~keep without inserting URL words into document content (#771). An icon font or an image is
// ~keep no empty wrapper: such a link gets its name or its address as the label (#775), except
// ~keep a link into its own page, which `leads_nowhere_else` leaves out.
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
    // ~keep The label of this form is the address. The white space at its ends comes from the
    // ~keep text that made the link this form (`LinkData::text_edges`): the content is not
    // ~keep walked, so an element in it writes nothing, as before.
    let (prefix, suffix) = if keeps_source_white_space(handler.options, handler.context) {
        ("", "")
    } else {
        data.text_edges
    };
    // ~keep The first text of the document has no white space at its start: the text of a label
    // ~keep loses it in the walk, by the same rule.
    let context = handler.context;
    let prefix = if is_fresh_block_start(context, context.at_fresh_block_start.get()) {
        ""
    } else {
        prefix
    };
    let edges = LabelEdges {
        prefix,
        suffix,
        has_text: true,
    };
    write_with_edges(&edges, handler, |handler| {
        handler.output.push('<');
        if data.href.starts_with("mailto:") && data.raw_text == data.href[7..] {
            handler.output.push_str(&data.raw_text);
        } else {
            handler.output.push_str(&data.href);
        }
        handler.output.push('>');
    });
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

/// The content of the label as the walk writes it. The caller removes the white space at its
/// two ends.
fn build_label_content(data: &LinkData<'_>, handler: &HandlerContext<'_>) -> String {
    if data.emit_blocks_separately {
        return walk_label_content(&data.inline_children, false, data, handler);
    }
    if data.saw_block {
        let content = walk_label_content(&data.children, true, data, handler);
        return if content.trim().is_empty() {
            data.inline_label.clone()
        } else {
            content
        };
    }
    walk_label_content(&data.children, false, data, handler)
}

/// The context that the label of a link is built with outside code.
fn label_context(data: &LinkData<'_>, handler: &HandlerContext<'_>, convert_as_inline: bool) -> Context {
    Context {
        inline_depth: handler.context.inline_depth + 1,
        in_link: handler.context.in_link || !data.href.is_empty(),
        media_boundary: handler
            .context
            .media_boundary
            .clone()
            .or_else(|| Some(std::rc::Rc::new(std::cell::Cell::new(Default::default())))),
        convert_as_inline: handler.context.convert_as_inline || convert_as_inline,
        link_allow_inline_images: data.link_allow_inline_images,
        ..handler.context.clone()
    }
}

/// Walks the children of a link into its label. With `convert_as_inline` the blocks among them
/// are written inline, and the walk writes one space at each block boundary.
///
/// ~keep In such a label an image is written as its alternative text. A reader sees the image
/// ~keep and the content beside it as two things, so the text of an image child is a word of
/// ~keep its own: one space separates it from the child before it and the child after it.
fn walk_label_content(
    children: &[tl::NodeHandle],
    convert_as_inline: bool,
    data: &LinkData<'_>,
    handler: &HandlerContext<'_>,
) -> String {
    let link_context = label_context(data, handler, convert_as_inline);
    let strict = keeps_source_white_space(handler.options, handler.context);
    let mut content = String::new();
    let mut after_image = false;
    for child in children {
        let child_start = content.len();
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
        if !convert_as_inline || content.len() == child_start {
            continue;
        }
        let is_image = handler
            .dom_context
            .tag_info(child.get_inner(), handler.parser)
            .is_some_and(|info| matches!(info.name.as_str(), "img" | "picture" | "svg"));
        // ~keep The strict mode writes no space at a block boundary (`separate_from_block`). It
        // ~keep keeps the rule that a label had before: one space between two children that touch.
        let separate = if strict {
            texts_touch_at(&content, child_start)
        } else {
            (is_image || after_image) && words_touch_at(&content, child_start)
        };
        if separate {
            content.insert(child_start, ' ');
        }
        after_image = is_image;
    }
    content
}

/// Whether two texts touch at the byte offset `at` of `content`: no white space on either side.
///
/// ~keep A child can shorten the label (a block trims the white space before it), so `at` can
/// ~keep lie past the end or inside a character: then nothing touches there.
fn texts_touch_at(content: &str, at: usize) -> bool {
    let (Some(before), Some(after)) = (content.get(..at), content.get(at..)) else {
        return false;
    };
    before.ends_with(|character: char| !character.is_whitespace())
        && after.starts_with(|character: char| !character.is_whitespace())
}

/// Whether two words touch at the byte offset `at` of `content`: two texts touch there, with no
/// opening punctuation before it and no closing punctuation after it.
fn words_touch_at(content: &str, at: usize) -> bool {
    texts_touch_at(content, at)
        && !content[..at].ends_with(['(', '[', '{'])
        && !content[at..].starts_with(['.', ',', ';', ':', '!', '?', ')', ']', '}'])
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
    // ~keep that has content and no name; a link that holds only empty text wrappers keeps an
    // ~keep empty label (#771). A link that `leads_nowhere_else` is not named: it is left out.
    if label.is_empty() && !data.leads_nowhere_else() {
        if let Some(name) = data.accessible_name() {
            *label = normalize_link_label(name);
        }
    }
    let drop_link = data.is_dropped(label, handler);
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
    edges: &LabelEdges<'_>,
    drop_link: bool,
    handler: &mut HandlerContext<'_>,
) -> bool {
    if drop_link {
        return true;
    }
    #[cfg(feature = "visitor")]
    if let Some(visitor) = handler.context.visitor.clone() {
        return visit_link(tag, data, label, edges, &visitor, handler);
    }
    write_link_with_edges(data, label, edges, handler);
    true
}

/// Writes the link of the converter: the white space of the label start, the link, the white
/// space of the label end.
fn write_link_with_edges(data: &LinkData<'_>, label: &str, edges: &LabelEdges<'_>, handler: &mut HandlerContext<'_>) {
    write_with_edges(edges, handler, |handler| {
        write_link(handler.output, data, label, handler.options, handler.context);
    });
}

/// Writes the white space of the label start, what `write_marks` writes for the link, and the
/// white space of the label end. Every form of a link is written through here.
fn write_with_edges(
    edges: &LabelEdges<'_>,
    handler: &mut HandlerContext<'_>,
    write_marks: impl FnOnce(&mut HandlerContext<'_>),
) {
    if edges.has_text {
        push_inline_prefix(handler.output, edges.prefix, handler.options, handler.context);
    }
    write_marks(handler);
    append_inline_suffix(
        handler.output,
        edges.suffix,
        edges.has_text,
        handler.node_handle,
        handler.parser,
        handler.dom_context,
    );
}

fn write_link(output: &mut String, data: &LinkData<'_>, label: &str, options: &ConversionOptions, context: &Context) {
    if data.href_addr_dropped || data.href.is_empty() || context.in_code || context.in_code_block {
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
    edges: &LabelEdges<'_>,
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
        VisitResult::Continue => write_link_with_edges(data, label, edges, handler),
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
