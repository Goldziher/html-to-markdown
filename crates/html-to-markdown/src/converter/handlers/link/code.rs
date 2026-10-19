//! Links in code: the text of a link in a code span or a code block.

use std::rc::Rc;

#[cfg(feature = "metadata")]
use super::record_link_metadata;
use super::{LabelEdges, LabelWalk, LinkData, apply_label_fallbacks, emit_link, heading_context};
use crate::converter::Context;
use crate::converter::block::heading::find_single_heading_child;
use crate::converter::inline::HandlerContext;
use crate::converter::utility::content::normalize_link_label;
use crate::converter::walk::{Child, HookSite};

/// The work that is next for a link in code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CodeStage {
    /// The children write the text of the link.
    Text,
    /// The one heading of the link writes what it writes outside code.
    ProbeHeading,
    /// The children write the label that the link has outside code.
    ProbeLabel,
    Done,
}

/// A link in code, which writes its text. When a visitor observes the walk, the link asks what
/// its content writes outside code, to know whether the visitor gets a call (see
/// [`CodeText::is_shown`]). The walk visits the children for the text, then the one heading
/// and the children of the label for that answer, each when the answer is still open.
///
/// ~keep The answer comes from the walk that writes a link outside code, run once more on
/// ~keep the content of this link. No second set of rules says what a label is, so a rule of the
/// ~keep traversal (a stripped tag, an excluded node, a preset) cannot be missing from the answer.
/// ~keep No state is shared between a link and a link inside it.
pub(super) struct CodeText {
    stage: CodeStage,
    text_start: usize,
    text: String,
    /// The count of the listed children that write the text.
    texts: usize,
    /// The one heading of the link, when the walk asks what it writes outside code.
    heading: Option<tl::NodeHandle>,
    heading_text: String,
    label: Option<LabelWalk>,
    /// Whether a visitor gets a `visit_link` call. `None` while the answer is open.
    shown: Option<bool>,
}

impl CodeText {
    pub(super) fn open(data: &LinkData, handler: &HandlerContext<'_>, children: &mut Vec<tl::NodeHandle>) -> Self {
        children.extend(data.children.iter().copied());
        let asks = is_observed(handler.context) && !data.is_autolink(handler.options);
        let heading = find_single_heading_child(*handler.node_handle, handler.parser)
            .filter(|_| asks && !data.href_addr_dropped)
            .map(|(_, heading_handle)| heading_handle);
        if asks {
            children.extend(heading);
            children.extend(data.label_children().iter().copied());
        }
        Self {
            stage: CodeStage::Text,
            text_start: handler.output.len(),
            text: String::new(),
            texts: data.children.len(),
            heading,
            heading_text: String::new(),
            label: None,
            shown: (!asks).then_some(false),
        }
    }

    /// The part that the listed child `index` belongs to, with its index in that part.
    fn part(&self, index: usize) -> (CodeStage, usize) {
        let headings = usize::from(self.heading.is_some());
        if index < self.texts {
            (CodeStage::Text, index)
        } else if index < self.texts + headings {
            (CodeStage::ProbeHeading, index - self.texts)
        } else {
            (CodeStage::ProbeLabel, index - self.texts - headings)
        }
    }

    pub(super) fn before_child(&mut self, index: usize, data: &LinkData, element: &mut HookSite<'_>) -> Child {
        let (part, _) = self.part(index);
        if let Some(tl::Node::Tag(tag)) = element.node_handle.get(element.parser) {
            self.advance(part, tag, data, element);
        }
        match (part, self.heading, self.label.as_mut()) {
            (CodeStage::Text, _, _) => Child::IN_PLACE,
            _ if self.shown.is_some() => Child::Skip,
            (CodeStage::ProbeHeading, Some(heading_handle), _) => unobserved_outside_code(element.context)
                .and_then(|outside| heading_context(heading_handle, element.parser, &outside))
                .map_or(Child::Skip, |context| Child::Walk {
                    depth: None,
                    buffer: Some(String::new()),
                    scope: Some(Rc::new(context)),
                }),
            (CodeStage::ProbeLabel, _, Some(label)) => label.before_child(),
            _ => Child::Skip,
        }
    }

    pub(super) fn after_child(&mut self, index: usize, own: Option<String>, data: &LinkData, element: &HookSite<'_>) {
        match (self.part(index), self.label.as_mut()) {
            ((CodeStage::ProbeHeading, _), _) => self.heading_text = own.unwrap_or_default(),
            ((CodeStage::ProbeLabel, at), Some(label)) => {
                label.after_child(data.label_children()[at], own, element);
            }
            _ => {}
        }
    }

    /// Do the work of each stage before `to`.
    ///
    /// ~keep A visitor gets the calls of a link outside code, which makes none for a link in the
    /// ~keep `<address>` form, for a link around one heading, and for a link that is left out (see
    /// ~keep `is_dropped`). A highlighter writes one empty anchor into its own page for each line
    /// ~keep of code. An image with no alt text writes nothing in code, so a link whose text is
    /// ~keep empty asks what its content writes outside code: a link with a label there is not
    /// ~keep left out, and a heading with a text there makes a heading link.
    pub(super) fn advance(
        &mut self,
        to: CodeStage,
        tag: &tl::HTMLTag<'_>,
        data: &LinkData,
        element: &mut HookSite<'_>,
    ) {
        while self.stage < to {
            self.stage = match self.stage {
                CodeStage::Text => {
                    self.text = element.output.split_off(self.text_start);
                    if self.heading.is_some() && !self.text.trim().is_empty() {
                        self.shown = Some(false);
                    }
                    CodeStage::ProbeHeading
                }
                CodeStage::ProbeHeading => {
                    if self.shown.is_none() {
                        self.ask_label(tag, data, element);
                    }
                    CodeStage::ProbeLabel
                }
                CodeStage::ProbeLabel => {
                    let label = self.label.take().map(|label| label.into_content(data));
                    let shown = self
                        .shown
                        .unwrap_or_else(|| label.is_some_and(|content| !normalize_link_label(&content).is_empty()));
                    if shown {
                        emit_link(tag, data, &self.text, &LabelEdges::NONE, false, element);
                    } else {
                        element.output.push_str(&self.text);
                    }
                    #[cfg(feature = "metadata")]
                    record_link_metadata(tag, data, self.text.trim(), element.context);
                    CodeStage::Done
                }
                CodeStage::Done => CodeStage::Done,
            };
        }
    }

    /// Answer from the text of the link and the text of its heading outside code, or start the
    /// walk of the label that the link has outside code.
    fn ask_label(&mut self, tag: &tl::HTMLTag<'_>, data: &LinkData, element: &HookSite<'_>) {
        if !self.heading_text.trim().is_empty() {
            self.shown = Some(false);
            return;
        }
        let mut label = normalize_link_label(&self.text);
        apply_label_fallbacks(tag, data, &mut label, element);
        if !data.is_dropped(&label, element) {
            self.shown = Some(true);
        } else if let Some(outside) = unobserved_outside_code(element.context) {
            self.label = Some(LabelWalk::new(data, element.options, &outside));
        }
    }
}

/// Whether a visitor observes the walk with the state `context`.
#[cfg(feature = "visitor")]
fn is_observed(context: &Context) -> bool {
    context.has_observer()
}

/// Whether a visitor observes the walk with the state `context`.
#[cfg(not(feature = "visitor"))]
const fn is_observed(_context: &Context) -> bool {
    false
}

/// The state of a walk that writes as outside code and that nothing observes (see
/// `Context::unobserved_outside_code`). `None` in a build with no visitor.
#[cfg(feature = "visitor")]
fn unobserved_outside_code(context: &Context) -> Option<Context> {
    Some(context.unobserved_outside_code())
}

/// The state of a walk that writes as outside code and that nothing observes. `None` in a build
/// with no visitor.
#[cfg(not(feature = "visitor"))]
const fn unobserved_outside_code(_context: &Context) -> Option<Context> {
    None
}

/// Write a link in a code span: `raw`, the text that its children wrote with the state
/// `context`, goes to the span, and the links of the span record the range of that text with
/// the link that replaces it when the span is written.
pub(super) fn emit_code_link(
    tag: &tl::HTMLTag<'_>,
    data: &LinkData,
    mut raw: String,
    context: &Context,
    handler: &mut HandlerContext<'_>,
) {
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
            ..context.clone()
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
            if let Some(links) = handler.context.inline_code_links.as_ref() {
                links.borrow_mut().push(crate::converter::context::InlineCodeLink {
                    buffer: std::ptr::from_ref::<String>(handler.output) as usize,
                    range: start..handler.output.len(),
                    markdown,
                });
            }
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
