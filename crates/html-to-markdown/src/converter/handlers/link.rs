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

mod code;

use self::code::{CodeStage, CodeText, emit_code_link};
use crate::converter::Context;
use crate::converter::append_inline_suffix;
use crate::converter::block::heading::{find_single_heading_child, heading_allows_inline_images, push_heading};
use crate::converter::dom_context::DomContext;
use crate::converter::inline::HandlerContext;
use crate::converter::inline::link::{MarkdownLink, append_markdown_link_in_context, has_uri_scheme};
use crate::converter::media::inline_data_treatment;
use crate::converter::text_node::is_fresh_block_start;
use crate::converter::utility::content::{
    collect_link_label_text, indent_continuation_lines, keeps_source_white_space, link_accessible_name,
    link_label_edges, link_text_content, link_text_edges, node_is_block_level, normalize_link_label,
    normalized_tag_name, push_inline_prefix,
};
use crate::converter::utility::escaping::escape_link_label;
use crate::converter::walk::{Child, ChildHook, Close, HookSite, Open};
use crate::options::{ConversionOptions, InlineDataMedia};
use crate::text;
use std::borrow::Cow;
use std::rc::Rc;

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

/// Open an `<a>` (link) element and list in `children` the nodes that the walk visits for it.
///
/// The link is written by the hook that this function gives:
/// - Extracting href and title attributes
/// - Detecting autolinks (where text equals href)
/// - Handling links that contain heading elements
/// - Processing complex link content (mixed block/inline)
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Collecting link metadata when the metadata feature is enabled
/// - Generating appropriate markdown link output
///
/// Returns `None` for a link that is complete.
pub fn open_link(
    tag: &tl::HTMLTag,
    handler: &mut HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(data) = LinkData::new(tag, handler) else {
        children.extend(tag.children().top().iter().copied());
        return Some(Open::children_only());
    };
    let flow = if handler.context.inline_code_links.is_some() && !handler.context.in_code_block {
        // ~keep A link in a code span stays a link: the code marks go inside its label (#813).
        children.extend(data.children.iter().copied());
        LinkFlow::CodeSpan {
            raw: String::new(),
            scope: Rc::new(Context {
                inline_code_links: None,
                ..handler.context.clone()
            }),
        }
    } else if handler.context.in_code {
        // ~keep A preformatted link preserves its destination (#928); the code walk keeps its label's whitespace.
        handler.context.inline_data_replaced.set(false);
        LinkFlow::Code(CodeText::open(&data, handler, children))
    } else {
        if emit_autolink(&data, handler) {
            return None;
        }
        LinkFlow::Label(LabelFlow::open(&data, handler, children))
    };
    Some(Open {
        scope: None,
        close: Close::Hooked(Box::new(LinkHook { data, flow })),
    })
}

/// The work between the children of a link.
struct LinkHook {
    data: LinkData,
    flow: LinkFlow,
}

/// Where a link is, which decides what the walk visits for it.
enum LinkFlow {
    /// In a code span: the children write the text of the link, with `scope` as their state.
    CodeSpan { raw: String, scope: Rc<Context> },
    /// In other code.
    Code(CodeText),
    /// Outside code.
    Label(LabelFlow),
}

impl ChildHook for LinkHook {
    fn before_child(&mut self, index: usize, element: &mut HookSite<'_>) -> Child {
        match &mut self.flow {
            LinkFlow::CodeSpan { raw, scope } => Child::Walk {
                depth: None,
                buffer: Some(std::mem::take(raw)),
                scope: Some(Rc::clone(scope)),
            },
            LinkFlow::Code(code) => code.before_child(index, &self.data, element),
            LinkFlow::Label(label) => label.before_child(index, &self.data, element),
        }
    }

    fn after_child(&mut self, index: usize, own: Option<String>, element: &mut HookSite<'_>) {
        match &mut self.flow {
            LinkFlow::CodeSpan { raw, .. } => *raw = own.unwrap_or_default(),
            LinkFlow::Code(code) => code.after_child(index, own, &self.data, element),
            LinkFlow::Label(label) => label.after_child(index, own, &self.data, element),
        }
    }

    fn close(self: Box<Self>, element: &mut HookSite<'_>) {
        let Some(tl::Node::Tag(tag)) = element.node_handle.get(element.parser) else {
            return;
        };
        let Self { data, flow } = *self;
        match flow {
            LinkFlow::CodeSpan { raw, scope } => emit_code_link(tag, &data, raw, &scope, element),
            LinkFlow::Code(mut code) => code.advance(CodeStage::Done, tag, &data, element),
            LinkFlow::Label(mut label) => label.advance(LabelStage::Done, tag, &data, element),
        }
    }
}

/// The work that is next for a link outside code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LabelStage {
    /// The one heading of the link writes its text.
    Heading,
    /// The children write the label.
    Label,
    /// The blocks that the link does not hold in its label are written after it.
    Deferred,
    Done,
}

/// A link outside code. The walk visits the one heading of the link when it has one, then the
/// children of the label, then the blocks that are written after the link. Each part decides
/// whether the walk visits the next one.
struct LabelFlow {
    stage: LabelStage,
    /// The level and the node of the one heading of the link.
    heading: Option<(usize, tl::NodeHandle)>,
    heading_text: String,
    label: Option<LabelWalk>,
    /// Whether the blocks after the link are written.
    emits_deferred: bool,
}

impl LabelFlow {
    fn open(data: &LinkData, handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Self {
        let heading =
            find_single_heading_child(*handler.node_handle, handler.parser).filter(|_| !data.href_addr_dropped);
        children.extend(heading.map(|(_, heading_handle)| heading_handle));
        children.extend(data.label_children().iter().copied());
        if data.emit_blocks_separately {
            children.extend(data.deferred.iter().copied());
        }
        Self {
            stage: LabelStage::Heading,
            heading,
            heading_text: String::new(),
            label: None,
            emits_deferred: false,
        }
    }

    /// The part that the listed child `index` belongs to, with its index in that part.
    fn part(&self, index: usize, data: &LinkData) -> (LabelStage, usize) {
        let headings = usize::from(self.heading.is_some());
        let labels = data.label_children().len();
        if index < headings {
            (LabelStage::Heading, index)
        } else if index < headings + labels {
            (LabelStage::Label, index - headings)
        } else {
            (LabelStage::Deferred, index - headings - labels)
        }
    }

    fn before_child(&mut self, index: usize, data: &LinkData, element: &mut HookSite<'_>) -> Child {
        let (part, _) = self.part(index, data);
        if let Some(tl::Node::Tag(tag)) = element.node_handle.get(element.parser) {
            self.advance(part, tag, data, element);
        }
        if self.stage != part {
            return Child::Skip;
        }
        match (part, self.heading, self.label.as_mut()) {
            (LabelStage::Heading, Some((_, heading_handle)), _) => {
                heading_context(heading_handle, element.parser, element.context).map_or(Child::Skip, |context| {
                    Child::Walk {
                        depth: None,
                        buffer: Some(String::new()),
                        scope: Some(Rc::new(context)),
                    }
                })
            }
            (LabelStage::Label, _, Some(label)) => label.before_child(),
            (LabelStage::Deferred, _, _) if self.emits_deferred => Child::IN_PLACE,
            _ => Child::Skip,
        }
    }

    fn after_child(&mut self, index: usize, own: Option<String>, data: &LinkData, element: &HookSite<'_>) {
        match (self.part(index, data), self.label.as_mut()) {
            ((LabelStage::Heading, _), _) => self.heading_text = own.unwrap_or_default(),
            ((LabelStage::Label, at), Some(label)) => {
                label.after_child(data.label_children()[at], own, element);
            }
            _ => {}
        }
    }

    /// Do the work of each stage before `to`.
    fn advance(&mut self, to: LabelStage, tag: &tl::HTMLTag<'_>, data: &LinkData, element: &mut HookSite<'_>) {
        while self.stage < to {
            self.stage = match self.stage {
                LabelStage::Heading => {
                    let heading_text = self.heading_text.trim();
                    match self.heading {
                        Some((level, _)) if !heading_text.is_empty() => {
                            emit_heading_link(tag, data, level, heading_text, element);
                            LabelStage::Done
                        }
                        _ => {
                            element.context.inline_data_replaced.set(false);
                            self.label = Some(LabelWalk::new(data, element.options, element.context));
                            LabelStage::Label
                        }
                    }
                }
                LabelStage::Label => {
                    let content = self
                        .label
                        .take()
                        .map(|label| label.into_content(data))
                        .unwrap_or_default();
                    self.emits_deferred = emit_label_link(tag, data, &content, element) && data.emit_blocks_separately;
                    LabelStage::Deferred
                }
                LabelStage::Deferred | LabelStage::Done => LabelStage::Done,
            };
        }
    }
}

/// Write the link of the converter around `content`, the content of its label as the walk wrote
/// it. Returns whether the blocks after the link are written.
fn emit_label_link(tag: &tl::HTMLTag<'_>, data: &LinkData, content: &str, handler: &mut HandlerContext<'_>) -> bool {
    let (prefix, suffix, trimmed) = if keeps_source_white_space(handler.options, handler.context) {
        ("", "", content)
    } else {
        link_label_edges(content)
    };
    let edges = LabelEdges {
        prefix,
        suffix,
        has_text: !trimmed.is_empty(),
    };
    let mut label = normalize_link_label(trimmed);
    apply_label_fallbacks(tag, data, &mut label, handler);
    indent_hard_break_continuations(&mut label, handler.context, handler.options);
    let drop_link = data.is_dropped(&label, handler);
    let emit_deferred = emit_link(tag, data, &label, &edges, drop_link, handler);
    #[cfg(feature = "metadata")]
    record_link_metadata(tag, data, &label, handler.context);
    emit_deferred
}

/// The white space that the label of a link had at its two ends (see [`chomp_inline`]).
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

/// What a link is, apart from its `title` and its `aria-label`, which [`link_title`] and
/// [`apply_label_fallbacks`] read from the tag.
struct LinkData {
    href: String,
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
    same_page_fragment: bool,
    /// The address has a fragment that is not empty: it names a place.
    names_place: bool,
    empty_span_content: bool,
}

impl LinkData {
    /// The children that write the label, in a walk that writes the blocks among them inline
    /// when the link holds a block and writes no block after itself.
    fn label_children(&self) -> &[tl::NodeHandle] {
        if self.emit_blocks_separately {
            &self.inline_children
        } else {
            &self.children
        }
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

    fn new(tag: &tl::HTMLTag<'_>, handler: &HandlerContext<'_>) -> Option<Self> {
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

/// The context that the one heading of a link is written with, where `context` is the state of
/// the link.
fn heading_context(heading_handle: tl::NodeHandle, parser: &tl::Parser<'_>, context: &Context) -> Option<Context> {
    let tl::Node::Tag(heading_tag) = heading_handle.get(parser)? else {
        return None;
    };
    let heading_name = normalized_tag_name(heading_tag.name().as_utf8_str());
    Some(Context {
        in_heading: true,
        convert_as_inline: true,
        heading_allow_inline_images: heading_allows_inline_images(&heading_name, &context.keep_inline_images_in),
        ..context.clone()
    })
}

/// The `title` of a link.
///
/// ~keep Empty titles are absent because Markdown serializers drop `""` on reparse.
fn link_title<'a>(tag: &'a tl::HTMLTag<'a>) -> Option<Cow<'a, str>> {
    crate::converter::utility::attributes::decoded_attribute(tag, "title").filter(|value| !value.is_empty())
}

fn emit_autolink(data: &LinkData, handler: &mut HandlerContext<'_>) -> bool {
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

/// Write a link around its one heading as a heading of that `level`, where `heading_text` is
/// the text that the heading wrote.
fn emit_heading_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
    level: usize,
    heading_text: &str,
    handler: &mut HandlerContext<'_>,
) {
    let mut link = String::new();
    append_link(
        &mut link,
        &MarkdownLink {
            label: &escape_link_label(heading_text),
            href: &data.href,
            title: link_title(tag).as_deref(),
            raw_text: &data.raw_text,
        },
        handler.options,
        handler.context,
    );
    push_heading(handler.output, handler.context, handler.options, level, &link);
}

/// The context that the label of a link is built with outside code, where `context` is the
/// state of the link.
fn label_context(data: &LinkData, context: &Context, convert_as_inline: bool) -> Context {
    Context {
        inline_depth: context.inline_depth + 1,
        in_link: context.in_link || !data.href.is_empty(),
        media_boundary: context
            .media_boundary
            .clone()
            .or_else(|| Some(std::rc::Rc::new(std::cell::Cell::new(Default::default())))),
        convert_as_inline: context.convert_as_inline || convert_as_inline,
        link_allow_inline_images: data.link_allow_inline_images,
        ..context.clone()
    }
}

/// The walk of the children of a link into its label. With `convert_as_inline` the blocks among
/// them are written inline, and the walk writes one space at each block boundary.
///
/// ~keep In such a label an image is written as its alternative text. A reader sees the image
/// ~keep and the content beside it as two things, so the text of an image child is a word of
/// ~keep its own: one space separates it from the child before it and the child after it.
struct LabelWalk {
    /// The state of the children.
    scope: Rc<Context>,
    convert_as_inline: bool,
    strict: bool,
    content: String,
    /// The length of `content` before the child that the walk visits now.
    child_start: usize,
    after_image: bool,
}

impl LabelWalk {
    /// The walk of the label of the link `data`, where `context` is the state of the link.
    fn new(data: &LinkData, options: &ConversionOptions, context: &Context) -> Self {
        let convert_as_inline = data.saw_block && !data.emit_blocks_separately;
        Self {
            scope: Rc::new(label_context(data, context, convert_as_inline)),
            convert_as_inline,
            strict: keeps_source_white_space(options, context),
            content: String::new(),
            child_start: 0,
            after_image: false,
        }
    }

    fn before_child(&mut self) -> Child {
        self.child_start = self.content.len();
        Child::Walk {
            depth: None,
            buffer: Some(std::mem::take(&mut self.content)),
            scope: Some(Rc::clone(&self.scope)),
        }
    }

    /// `child` wrote to `own`, the content of the label.
    fn after_child(&mut self, child: tl::NodeHandle, own: Option<String>, element: &HookSite<'_>) {
        self.content = own.unwrap_or_default();
        let child_start = self.child_start;
        if !self.convert_as_inline || self.content.len() == child_start {
            return;
        }
        let is_image = element
            .dom_context
            .tag_info(child.get_inner(), element.parser)
            .is_some_and(|info| matches!(info.name.as_str(), "img" | "picture" | "svg"));
        // ~keep The strict mode writes no space at a block boundary (`separate_from_block`). It
        // ~keep keeps the rule that a label had before: one space between two children that touch.
        let separate = if self.strict {
            texts_touch_at(&self.content, child_start)
        } else {
            (is_image || self.after_image) && words_touch_at(&self.content, child_start)
        };
        if separate {
            self.content.insert(child_start, ' ');
        }
        self.after_image = is_image;
    }

    /// The content of the label as the walk wrote it. The caller removes the white space at its
    /// two ends.
    fn into_content(self, data: &LinkData) -> String {
        if self.convert_as_inline && self.content.trim().is_empty() {
            data.inline_label.clone()
        } else {
            self.content
        }
    }
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

fn apply_label_fallbacks(tag: &tl::HTMLTag<'_>, data: &LinkData, label: &mut String, handler: &HandlerContext<'_>) {
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
        let aria_label = crate::converter::utility::attributes::decoded_attribute(tag, "aria-label");
        let title = link_title(tag);
        if let Some(name) = link_accessible_name(aria_label.as_deref(), title.as_deref()) {
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

fn emit_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
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
    write_link_with_edges(tag, data, label, edges, handler);
    true
}

/// Writes the link of the converter: the white space of the label start, the link, the white
/// space of the label end.
fn write_link_with_edges(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
    label: &str,
    edges: &LabelEdges<'_>,
    handler: &mut HandlerContext<'_>,
) {
    write_with_edges(edges, handler, |handler| {
        write_link(handler.output, tag, data, label, handler.options, handler.context);
    });
}

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

fn write_link(
    output: &mut String,
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
    label: &str,
    options: &ConversionOptions,
    context: &Context,
) {
    if data.href_addr_dropped
        || data.href.is_empty()
        || (context.in_code && !context.in_code_block)
        || (context.in_code_block && normalize_link_label(label).is_empty())
    {
        output.push_str(label);
        return;
    }
    append_link(
        output,
        &MarkdownLink {
            label: &escape_link_label(label),
            href: &data.href,
            title: link_title(tag).as_deref(),
            raw_text: label,
        },
        options,
        context,
    );
}

fn append_link(output: &mut String, link: &MarkdownLink<'_>, options: &ConversionOptions, context: &Context) {
    append_markdown_link_in_context(
        output,
        link,
        options,
        context.reference_collector.as_ref(),
        context.in_table_cell,
    );
}

#[cfg(feature = "visitor")]
fn visit_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
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
        link_title(tag).as_deref(),
    );
    if handler.options.whitespace_mode == crate::options::WhitespaceMode::Strict
        && (matches!(&result, VisitResult::Skip)
            || matches!(&result, VisitResult::Custom(custom)
                if !data.emit_blocks_separately && custom.trim_matches([' ', '\t']).is_empty()))
    {
        handler.dom_context.record_absent_breaks(node_id, true);
    }
    match result {
        VisitResult::Continue => write_link_with_edges(tag, data, label, edges, handler),
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
fn record_link_metadata(tag: &tl::HTMLTag<'_>, data: &LinkData, label: &str, context: &Context) {
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
        link_title(tag).map(Cow::into_owned),
        rel,
        attributes,
    );
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
    // ~keep Rendering's max_depth does not bound the raw DOM scanned here (issue #923).
    let mut pending = vec![*handle];
    while let Some(handle) = pending.pop() {
        let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
            continue;
        };
        if normalized_tag_name(tag.name().as_utf8_str()) == "table" {
            return true;
        }
        let descendants_start = pending.len();
        if let Some(children) = dom_ctx.children_of(handle.get_inner()) {
            pending.extend(children.iter().copied());
        } else {
            pending.extend(tag.children().top().iter().copied());
        }
        pending[descendants_start..].reverse();
    }
    false
}
