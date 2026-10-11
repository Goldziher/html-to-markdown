//! Spacing of form controls. A control is a box in the line, so its text is a word of its own:
//! one space separates it from text before it and from text after it in the same block.

use crate::converter::DomContext;
use crate::converter::tier1::tags::{TagSpec, lookup};

/// The nodes a search for the text after a control reads before it gives up.
const MAX_LOOKAHEAD_NODES: usize = 64;

/// Whether `tag_name` is a control that writes text of its own and separates it from the text
/// before it.
pub fn writes_own_text(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "select" | "datalist" | "textarea" | "button" | "output" | "meter" | "progress"
    )
}

/// Whether text written at the end of `output` needs a space before it to stay a word of its own.
pub fn needs_space_before(output: &str) -> bool {
    output
        .chars()
        .next_back()
        .is_some_and(|last| !last.is_whitespace() && !matches!(last, '(' | '[' | '{'))
}

/// Whether the line of a control goes on after the element that holds it. `parent` is the entry
/// of that element in the tag table: the line goes on after an inline element, and after an
/// element that is not in the table and is written as its children alone (a custom element).
///
/// ~keep Both converters ask this function. The fast converter has not read the text after the
/// ~keep control, so it hands the control over where the answer is yes.
pub fn line_goes_on_in(parent: Option<&TagSpec>) -> bool {
    parent.is_none_or(|spec| !spec.is_block)
}

/// Whether the line of node `id` goes on after its parent element.
pub fn line_goes_on_after_parent(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx
        .parent_tag_name(id, parser)
        .is_some_and(|name| line_goes_on_in(lookup(name.as_bytes())))
}

/// The place in the output where a control starts to write.
pub struct ControlStart {
    at: usize,
    separate: bool,
}

impl ControlStart {
    /// Records the end of `output` before the control writes.
    pub fn new(output: &str) -> Self {
        Self::at(output, output.len())
    }

    /// The start of a control that began to write at `at`, a byte offset on a character
    /// boundary of `output`.
    pub fn at(output: &str, at: usize) -> Self {
        Self {
            at,
            separate: needs_space_before(&output[..at]),
        }
    }

    /// Separates what the control wrote from the text before it. Returns whether it wrote.
    pub fn finish(&self, output: &mut String) -> bool {
        let Some(written) = output.get(self.at..).filter(|written| !written.is_empty()) else {
            return false;
        };
        if self.separate && !written.starts_with(char::is_whitespace) {
            output.insert(self.at, ' ');
        }
        true
    }
}

/// Writes the space between a control at the end of `output` and the text that follows
/// `node_handle` in the same block, when that text does not start with white space or closing
/// punctuation.
pub fn separate_from_next_text(
    output: &mut String,
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) {
    if needs_space_before(output) && word_follows(node_handle.get_inner(), parser, dom_ctx) {
        output.push(' ');
    }
}

/// The siblings of node `id` that follow it in the source.
fn siblings_after(id: u32, dom_ctx: &DomContext) -> &[tl::NodeHandle] {
    let siblings = match dom_ctx.parent_of(id) {
        Some(parent_id) => dom_ctx.children_of(parent_id),
        None => Some(&dom_ctx.root_children),
    };
    siblings
        .zip(dom_ctx.sibling_index(id))
        .and_then(|(siblings, position)| siblings.get(position + 1..))
        .unwrap_or_default()
}

/// Whether the source has white space right after node `id`. A comment is not in the way.
pub fn white_space_follows(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    siblings_after(id, dom_ctx)
        .iter()
        .filter_map(|next| next.get(parser))
        .find(|next| !matches!(next, tl::Node::Comment(_)))
        .is_some_and(|next| matches!(next, tl::Node::Raw(raw) if raw.as_utf8_str().starts_with(char::is_whitespace)))
}

/// Whether the content after node `id` in its block starts with a word.
fn word_follows(mut id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut budget = MAX_LOOKAHEAD_NODES;
    loop {
        for sibling in siblings_after(id, dom_ctx) {
            if let Some(starts_with_word) = leading_word(*sibling, parser, dom_ctx, &mut budget) {
                return starts_with_word;
            }
        }
        // ~keep The line ends with a block.
        let Some(parent_id) = dom_ctx.parent_of(id) else {
            return false;
        };
        if !line_goes_on_after_parent(id, parser, dom_ctx) {
            return false;
        }
        id = parent_id;
    }
}

/// Whether the first content `handle` writes starts with a word, or `None` when it writes nothing.
fn leading_word(handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext, budget: &mut usize) -> Option<bool> {
    if *budget == 0 {
        return Some(false);
    }
    *budget -= 1;
    match handle.get(parser)? {
        tl::Node::Comment(_) => None,
        tl::Node::Raw(raw) => raw.as_utf8_str().chars().next().map(|first| {
            !first.is_whitespace() && !matches!(first, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}')
        }),
        tl::Node::Tag(tag) => {
            let name = dom_ctx.tag_name_for(handle, parser)?;
            match name.as_ref() {
                // ~keep An input writes no text, so it is not in the way.
                "input" | "script" | "style" | "template" | "noscript" => None,
                "br" => Some(false),
                "img" => Some(true),
                // ~keep An element that writes nothing (an empty button, an empty span) is not in
                // ~keep the way either. A space before a block is at a line end, where it is removed.
                _ => tag
                    .children()
                    .top()
                    .iter()
                    .find_map(|child| leading_word(*child, parser, dom_ctx, budget)),
            }
        }
    }
}
