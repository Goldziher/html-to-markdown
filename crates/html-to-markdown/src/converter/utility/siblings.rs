//! Sibling node navigation and handling.
//!
//! Utilities for working with sibling nodes in the DOM tree, including navigation functions
//! and inline/block element detection for whitespace handling.

use crate::converter::DomContext;

/// Get the tag name of the next sibling element.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn get_next_sibling_tag<'a>(
    node_handle: &tl::NodeHandle,
    parser: &'a tl::Parser,
    dom_ctx: &'a DomContext,
) -> Option<&'a str> {
    dom_ctx.next_tag_name(*node_handle, parser)
}

/// Get the tag name of the previous sibling element.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn get_previous_sibling_tag<'a>(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &'a DomContext,
) -> Option<&'a str> {
    let id = node_handle.get_inner();
    if let Some(previous) = dom_ctx.prev_tag_map.get(id as usize).and_then(std::cell::OnceCell::get) {
        return previous
            .and_then(|id| dom_ctx.tag_info(id, parser))
            .map(|info| info.name.as_str());
    }
    let parent = dom_ctx.parent_of(id);

    let siblings = if let Some(parent_id) = parent {
        dom_ctx.children_of(parent_id)?
    } else {
        &dom_ctx.root_children
    };

    let position = dom_ctx.sibling_index(id).or_else(|| {
        siblings
            .iter()
            .position(|handle: &tl::NodeHandle| handle.get_inner() == id)
    })?;

    for sibling in siblings.iter().take(position).rev() {
        if let Some(info) = dom_ctx.tag_info(sibling.get_inner(), parser) {
            return Some(info.name.as_str());
        }
        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
            if !raw.as_utf8_str().trim().is_empty() {
                return None;
            }
        }
    }

    None
}

#[cfg(test)]
thread_local! {
    /// The eager following-content classifications and cached lookups on this thread. ~keep
    pub(crate) static FOLLOWING_CONTENT_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// The sibling entries that `text_place` read on this thread. ~keep
    pub(crate) static TEXT_PLACE_SIBLING_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Actual sibling entries visited while looking through inline wrappers for a block. ~keep
    pub(crate) static PREVIOUS_CONTENT_BLOCK_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Number of actual preceding-block queries on this thread. ~keep
    pub(crate) static PREVIOUS_CONTENT_BLOCK_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_sibling_visits(visits: usize) {
    TEXT_PLACE_SIBLING_VISITS.with(|total| total.set(total.get() + visits));
}

#[cfg(not(test))]
const fn count_sibling_visits(_visits: usize) {}

/// Count both index construction and queries, so a zero-work lookup cannot hide setup cost. ~keep
#[cfg(test)]
pub fn count_following_content_read() {
    FOLLOWING_CONTENT_READS.with(|total| total.set(total.get() + 1));
}

/// Compile operation counting out of production builds. ~keep
#[cfg(not(test))]
pub const fn count_following_content_read() {}

/// The index of `id` in `siblings` by a scan from the first sibling, for a node that the index
/// of the tree does not hold.
fn scan_for_sibling_index(siblings: &[tl::NodeHandle], id: u32) -> Option<usize> {
    let position = siblings.iter().position(|sibling| sibling.get_inner() == id);
    count_sibling_visits(position.map_or(siblings.len(), |position| position + 1));
    position
}

/// What stands right before the text `node_handle`: a `<br>` that ends a line, the start of its
/// parent element, an element that holds no text, or text. A comment before the text does not
/// count. An element holds no text when it has only white space, comments and elements with no
/// text in it (`DomContext::holds_text`): `<span> </span>` is one, as `<span></span>` is.
///
/// ~keep `in_cell` tells that the text is in a table cell. A cell has one line: the `<br>` there
/// ~keep is a space between two words, an element with no text, also in a heading in the cell.
/// ~keep The index of the tree gives the position of the text: a scan from the first sibling
/// ~keep for each text is quadratic in the count of siblings.
pub fn text_place(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    in_cell: bool,
) -> crate::converter::utility::white_space::TextPlace {
    use crate::converter::utility::white_space::TextPlace;
    if get_previous_sibling_tag(&node_handle, parser, dom_ctx) == Some("br") {
        return if in_cell {
            TextPlace::AfterEmptyElement
        } else {
            TextPlace::AfterBreak
        };
    }
    let id = node_handle.get_inner();
    let siblings = dom_ctx
        .parent_of(id)
        .map_or(Some(&dom_ctx.root_children), |parent| dom_ctx.children_of(parent));
    let before = siblings
        .and_then(|siblings| {
            let position = dom_ctx
                .sibling_index(id)
                .or_else(|| scan_for_sibling_index(siblings, id))?;
            siblings.get(..position)
        })
        .unwrap_or_default();
    let previous = before
        .iter()
        .enumerate()
        .rev()
        .inspect(|_| count_sibling_visits(1))
        .find(|(_, sibling)| !matches!(sibling.get(parser), Some(tl::Node::Comment(_))));
    match previous {
        None => TextPlace::ElementStart,
        Some((index, previous)) if matches!(previous.get(parser), Some(tl::Node::Tag(_))) => {
            place_after_element(previous.get_inner(), &before[..index], parser, dom_ctx, in_cell)
        }
        Some(_) => TextPlace::AfterText,
    }
}

/// The place of a text that has the element `element` right before it. `before` holds the
/// siblings before that element.
///
/// ~keep An element in the text flow ([`writes_in_text_flow`]) writes no mark: the space that
/// ~keep ends its text is a written space of the text, and the Unicode space after the element
/// ~keep is a character after it (`a <span>x </span>&nbsp;b` has two spaces). When it has no
/// ~keep content (`DomContext::holds_content`: empty elements in it are none) it writes
/// ~keep nothing, and the line end before it is the white space that the text
/// ~keep after it starts with: the two are one run (`a\n<span></span>&nbsp;b` has one space). A
/// ~keep table cell has no line end: the rule of the empty element holds there.
fn place_after_element(
    element: u32,
    before: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    in_cell: bool,
) -> crate::converter::utility::white_space::TextPlace {
    use crate::converter::utility::white_space::{TextPlace, writes_in_text_flow};
    let info = dom_ctx.tag_info(element, parser);
    let in_flow = info.is_some_and(|info| writes_in_text_flow(&info.name));
    if dom_ctx.holds_text(element, parser) {
        return if in_flow && ends_with_text_space(element, parser, dom_ctx) {
            TextPlace::AfterFlowElement
        } else {
            TextPlace::AfterText
        };
    }
    if info.is_some_and(|info| info.name == "font")
        && dom_ctx.blank_font_renders.borrow().contains(&element)
        && !dom_ctx.font_boundary_has_content(element, parser)
    {
        return TextPlace::AfterText;
    }
    let empty_flow = in_flow || info.is_some_and(|info| matches!(info.name.as_str(), "option" | "optgroup"));
    if empty_flow && !in_cell && ends_with_line_end(before.last(), parser) && !dom_ctx.holds_content(element, parser) {
        TextPlace::AfterText
    } else {
        TextPlace::AfterEmptyElement
    }
}

/// Whether `node` ends with a collapsible run containing a line end.
///
/// ~keep Empty inline boundaries share the full converter's main policy even where main's
/// ~keep forced-fast route kept a second space. A preceding Unicode space remains a character (#868).
fn ends_with_line_end(node: Option<&tl::NodeHandle>, parser: &tl::Parser) -> bool {
    let Some(tl::Node::Raw(raw)) = node.and_then(|node| node.get(parser)) else {
        return false;
    };
    let raw = raw.as_utf8_str();
    let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
    let before = decoded.trim_end_matches(crate::converter::utility::white_space::is_collapsible);
    decoded[before.len()..].contains(['\n', '\r'])
        && (before.is_empty() || before.ends_with(|character: char| !character.is_whitespace()))
}

/// Whether the content of the element `element` ends with a text that ends with collapsible
/// white space: its last node, comments apart, is such a text, or an element in the text flow
/// whose content ends so.
///
/// ~keep An element that writes marks puts its end space outside them: that space is no text
/// ~keep of the `span` around it, and `a <span><a href="/u">x </a></span>&nbsp;b` has one space.
/// ~keep A loop rather than recursion: nested elements are attacker-controlled depth. Each
/// ~keep element is the last node of one parent, so no node is read for two texts.
fn ends_with_text_space(mut element: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    use crate::converter::utility::white_space::{is_collapsible, writes_in_text_flow};
    loop {
        let last = dom_ctx.children_of(element).and_then(|children| {
            children
                .iter()
                .rev()
                .find(|child| !matches!(child.get(parser), Some(tl::Node::Comment(_))))
        });
        match last.and_then(|last| last.get(parser).map(|node| (last.get_inner(), node))) {
            Some((_, tl::Node::Raw(raw))) => {
                let raw = raw.as_utf8_str();
                return crate::text::decode_html_entities_cow(raw.as_ref()).ends_with(is_collapsible);
            }
            Some((inner, tl::Node::Tag(_)))
                if dom_ctx
                    .tag_info(inner, parser)
                    .is_some_and(|info| writes_in_text_flow(&info.name)) =>
            {
                element = inner;
            }
            _ => return false,
        }
    }
}

/// The block element the content before `node_handle` in its parent ends with: the previous
/// sibling when it is a block, or the block an inline sibling's own last content ends with
/// (`<span><ul>...</ul></span>`). Whitespace text and comments are skipped (issue #585).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn previous_content_block<'a>(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &'a DomContext,
) -> Option<&'a str> {
    previous_content_block_through(
        node_handle,
        parser,
        dom_ctx,
        crate::converter::main_helpers::is_inline_element,
    )
}

/// [`previous_content_block`], where the walk looks only into the elements that `looks_into`
/// accepts. Any other element that is no block ends the walk: no block is found.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn previous_content_block_through<'a>(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &'a DomContext,
    looks_into: fn(&str) -> bool,
) -> Option<&'a str> {
    #[cfg(test)]
    PREVIOUS_CONTENT_BLOCK_CALLS.with(|calls| calls.set(calls.get() + 1));
    let id = node_handle.get_inner();
    let siblings = match dom_ctx.parent_of(id) {
        Some(parent_id) => match dom_ctx.children_of(parent_id) {
            Some(children) => children.as_slice(),
            None => return None,
        },
        None => dom_ctx.root_children.as_slice(),
    };
    let position = dom_ctx
        .sibling_index(id)
        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))?;
    let mut nodes = &siblings[..position.min(siblings.len())];
    // ~keep A loop rather than recursion: nested inline wrappers are attacker-controlled depth.
    'outer: loop {
        for sibling in nodes.iter().rev() {
            #[cfg(test)]
            PREVIOUS_CONTENT_BLOCK_READS.with(|reads| reads.set(reads.get() + 1));
            if let Some(info) = dom_ctx.tag_info(sibling.get_inner(), parser) {
                if crate::converter::utility::content::is_block_level_element(&info.name) {
                    return Some(info.name.as_str());
                }
                if !looks_into(&info.name) {
                    return None;
                }
                match dom_ctx.children_of(sibling.get_inner()) {
                    Some(children) => {
                        nodes = children.as_slice();
                        continue 'outer;
                    }
                    None => return None,
                }
            }
            if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                if !raw.as_utf8_str().trim().is_empty() {
                    return None;
                }
            }
        }
        return None;
    }
}

/// Check if the next sibling is whitespace-only text.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn next_sibling_is_whitespace_text(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> bool {
    dom_ctx.next_whitespace_text(*node_handle, parser)
}

/// Check if the next sibling is an inline tag.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn next_sibling_is_inline_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx.next_inline_like(*node_handle, parser)
}

/// What follows a node among its siblings, as far as a word separator is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FollowingContent {
    /// The next meaningful sibling is inline content that butts against this node.
    Inline,
    /// The next meaningful sibling is a block, or text that opens with its own whitespace.
    NotInline,
    /// Nothing but whitespace-only text follows this node in its parent.
    Absent,
}

/// Classify what follows `id`: inline *content* — an inline-like tag, or a bare text node — a
/// block or self-separating text, or nothing at all.
///
/// `next_sibling_is_inline_tag` answers a narrower question: its scan stops and returns `false` at
/// the first non-whitespace text sibling, because its callers care specifically about an adjacent
/// *element*. A newline-only inline wrapper needs the broader question, since `a<span>\n</span>b`
/// separates two words exactly as `a<span>\n</span><span>b</span>` does (issue #491); asking the
/// narrower one welded them together. Kept separate rather than widening
/// `DomContext::next_inline_like`, which is memoised and shared with an unrelated caller.
/// `Absent` is distinct from `NotInline` so a caller can climb to the parent and ask again:
/// the newline at the end of `<span><span>\n</span></span>Beta` still separates words, but only
/// the outer wrapper can see `Beta` (issue #505). ~keep
pub fn following_sibling_content(id: u32, _parser: &tl::Parser, dom_ctx: &DomContext) -> FollowingContent {
    count_following_content_read();
    dom_ctx
        .following_content_map
        .get(id as usize)
        .copied()
        .unwrap_or(FollowingContent::Absent)
}

/// Whether a `<br>` follows the elements that `id` is the last content of:
/// `<span>First\n</span><br>` ends the line of `First` at that `<br>` (issue #683), just as
/// `First\n<br>` does. Whitespace text and comments between them are skipped. A block
/// ancestor ends its own line before the `<br>` is written, so the answer changes the output
/// only for elements that write no line end: `<span>`, `<font>`, a custom element.
pub fn br_follows_enclosing_elements(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut id = id;
    // ~keep A loop rather than recursion: nested wrappers are attacker-controlled depth.
    while following_sibling_content(id, parser, dom_ctx) == FollowingContent::Absent {
        let Some(parent) = dom_ctx.parent_of(id) else {
            return false;
        };
        if let Some(next) = dom_ctx.next_tag_id(parent, parser) {
            return dom_ctx.tag_info(next, parser).is_some_and(|info| info.name == "br");
        }
        id = parent;
    }
    false
}

/// Whether the text that follows `node_handle` starts with a zero-width space. The walk looks
/// into the elements that only wrap text, leaves such an element at its end (the text node can
/// be the last content of one) and passes the empty ones, comments and `<wbr>`. Any other
/// element is content of its own and ends the walk, and so does an element whose `style`
/// attribute sets `display` or `white-space`: it can be a box of its own
/// (`display: inline-block`), and a browser keeps the line end before such a box. Inside an
/// element with such a `style` attribute the answer is no.
///
/// ~keep Tier-1 answers the same question on the bytes (`zero_width_space_is_upcoming`).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn zero_width_space_follows(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    use crate::converter::utility::content::ZERO_WIDTH_SPACE;

    let mut current = node_handle.get_inner();
    // ~keep A loop rather than recursion: nested wrappers are attacker-controlled depth. Each
    // ~keep turn moves forward in the document, so the walk ends.
    loop {
        let mut node = loop {
            if let Some(next) = next_sibling(current, dom_ctx) {
                break next;
            }
            match dom_ctx.parent_of(current) {
                Some(parent) if is_plain_text_wrapper(parent, parser, dom_ctx) => current = parent,
                _ => return false,
            }
        };
        loop {
            match node.get(parser) {
                Some(tl::Node::Raw(raw)) => {
                    let raw = raw.as_utf8_str();
                    let text = crate::text::decode_html_entities_cow(raw.as_ref());
                    if text.is_empty() {
                        break;
                    }
                    return text.starts_with(ZERO_WIDTH_SPACE)
                        && !is_inside_display_or_white_space_style(node_handle.get_inner(), parser, dom_ctx);
                }
                Some(tl::Node::Tag(_)) => {
                    if !is_plain_text_wrapper(node.get_inner(), parser, dom_ctx) {
                        return false;
                    }
                    match dom_ctx
                        .children_of(node.get_inner())
                        .and_then(|children| children.first())
                    {
                        Some(first) => node = *first,
                        None => break,
                    }
                }
                Some(tl::Node::Comment(_)) => break,
                None => return false,
            }
        }
        current = node.get_inner();
    }
}

/// Whether the element `id` only wraps text (or is a `<wbr>`) and its `style` attribute, if
/// it has one, sets neither `display` nor `white-space`.
fn is_plain_text_wrapper(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    use crate::converter::utility::content::is_text_wrapper;

    let Some(tl::Node::Tag(tag)) = tl::NodeHandle::new(id).get(parser) else {
        return false;
    };
    dom_ctx
        .tag_info(id, parser)
        .is_some_and(|info| info.name == "wbr" || is_text_wrapper(&info.name))
        && !tag_sets_display_or_white_space(tag)
}

/// Whether the `style` attribute of `tag` sets `display` or `white-space`.
fn tag_sets_display_or_white_space(tag: &tl::HTMLTag) -> bool {
    use crate::converter::utility::attributes::style_attribute_sets_display_or_white_space;

    tag.attributes()
        .get("style")
        .flatten()
        .is_some_and(|style| style_attribute_sets_display_or_white_space(style.as_bytes()))
}

/// Whether the node `id` is inside an element whose `style` attribute sets `display` or
/// `white-space`.
///
/// ~keep Such an element can keep its line ends (`white-space: pre`), and its content inherits
/// ~keep that: a browser then shows the line end before a zero-width space.
/// ~keep Tier-1 answers the same question on its open elements
/// ~keep (`open_element_sets_display_or_white_space`).
fn is_inside_display_or_white_space_style(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut current = id;
    while let Some(parent) = dom_ctx.parent_of(current) {
        if let Some(tl::Node::Tag(tag)) = tl::NodeHandle::new(parent).get(parser)
            && tag_sets_display_or_white_space(tag)
        {
            return true;
        }
        current = parent;
    }
    false
}

/// Whether the content of the element `id` starts with a block. White space and comments are
/// no content, and the walk looks into the elements that only wrap text (`<a><b><div>`).
///
/// ~keep A `<span>` writes no marks, so a block that only `<span>` elements wrap breaks the
/// ~keep line itself and the answer is no.
/// ~keep Tier-1 answers the same question on the bytes (`upcoming_inline_starts_with_block`).
pub fn content_starts_with_block(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    use crate::converter::utility::content::is_text_wrapper;

    let mut current = id;
    let mut inside_marks = false;
    // ~keep A loop rather than recursion: nested wrappers are attacker-controlled depth.
    loop {
        let Some(wrapper) = dom_ctx.tag_info(current, parser) else {
            return false;
        };
        if !is_text_wrapper(&wrapper.name) {
            return false;
        }
        inside_marks |= wrapper.name != "span";
        let first_content = dom_ctx.children_of(current).and_then(|children| {
            children.iter().find(|child| match child.get(parser) {
                Some(tl::Node::Raw(raw)) => !raw.as_bytes().iter().all(u8::is_ascii_whitespace),
                Some(tl::Node::Comment(_)) => false,
                _ => true,
            })
        });
        let Some(first_content) = first_content else {
            return false;
        };
        match dom_ctx.tag_info(first_content.get_inner(), parser) {
            Some(info) if info.is_block => return inside_marks,
            Some(_) => current = first_content.get_inner(),
            None => return false,
        }
    }
}

/// The node after `id` among the children of its parent.
fn next_sibling(id: u32, dom_ctx: &DomContext) -> Option<tl::NodeHandle> {
    let siblings = match dom_ctx.parent_of(id) {
        Some(parent_id) => dom_ctx.children_of(parent_id)?,
        None => &dom_ctx.root_children,
    };
    siblings.get(dom_ctx.sibling_index(id)? + 1).copied()
}

/// Append an inline suffix to output, with smart whitespace handling.
///
/// Avoids adding spaces before siblings that are already whitespace.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn append_inline_suffix(
    output: &mut String,
    suffix: &str,
    has_core_content: bool,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) {
    if suffix.is_empty() {
        return;
    }

    if suffix == " " && has_core_content && next_sibling_is_whitespace_text(node_handle, parser, dom_ctx) {
        return;
    }

    output.push_str(suffix);
}

#[cfg(test)]
mod tests {
    use super::TEXT_PLACE_SIBLING_VISITS;
    use crate::converter::dom_context::HOLDS_TEXT_NODE_READS;
    use crate::{ConversionOptions, TierStrategy, convert};

    /// The sibling entries that `text_place` reads for `count` texts that each start with a
    /// no-break space after a space and an empty element, all in one paragraph.
    fn sibling_visits_for(count: usize) -> usize {
        let html = format!("<p>{}</p>", "a <span></span>&nbsp;b ".repeat(count));
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        TEXT_PLACE_SIBLING_VISITS.with(|visits| visits.set(0));
        let result = convert(&html, options).expect("convert HTML");
        let text = result.content.unwrap_or_default();
        assert_eq!(text.matches("a  b").count(), count, "every no-break space is kept");
        TEXT_PLACE_SIBLING_VISITS.with(std::cell::Cell::get)
    }

    /// The nodes that the test for an element with no text reads while the full converter writes
    /// `html`, and the text it writes.
    fn node_reads_for(html: &str) -> (usize, String) {
        let options = ConversionOptions {
            tier_strategy: TierStrategy::Tier2,
            ..ConversionOptions::default()
        };
        HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        let result = convert(html, options).expect("convert HTML");
        (
            HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get),
            result.content.unwrap_or_default(),
        )
    }

    #[test]
    fn should_read_a_bounded_count_of_nodes_for_each_text_after_nested_elements() {
        // ~keep `depth` elements, one in the other, each with a text that starts with a no-break
        // ~keep space after it: each of these texts asks whether the element before it holds
        // ~keep text. An element that starts with text answers at its first node. An element that
        // ~keep starts with white space answers from the record of the element in it. Without
        // ~keep the stop at the first text, or without the record, the count grows with the
        // ~keep square of the depth. The count of nodes is the measure, not the time.
        for depth in [10, 20, 40] {
            let ends = "</span>&nbsp;b ".repeat(depth);
            let with_text = format!("<p>{}x {ends}</p>", "<span>a ".repeat(depth));
            let (reads, text) = node_reads_for(&with_text);
            assert_eq!(text.matches('b').count(), depth, "with text: {text:?}");
            assert!(text.contains("x  b  b"), "with text: {text:?}");
            assert!((depth..=3 * depth).contains(&reads), "with text: {reads} nodes");

            let white_space = format!("<p>a {}{ends}</p>", "<span> ".repeat(depth));
            let (reads, text) = node_reads_for(&white_space);
            assert!(text.starts_with("a  b  b"), "white space: {text:?}");
            assert_eq!(text.matches('b').count(), depth, "white space: {text:?}");
            assert!((depth..=3 * depth).contains(&reads), "white space: {reads} nodes");

            // ~keep An empty `span` in an empty `span`, `depth` deep, after a line end: one walk
            // ~keep for the text and one for the content, each of them reads each element once.
            let empty = format!("<p>a\n{}{}&nbsp;b</p>", "<span>".repeat(depth), "</span>".repeat(depth));
            let (reads, text) = node_reads_for(&empty);
            assert_eq!(text, "a b\n", "empty: {text:?}");
            assert!((depth..=3 * depth).contains(&reads), "empty: {reads} nodes");
        }
    }

    #[test]
    fn should_preserve_following_content_classification_across_comments_and_blank_text() {
        use super::{FollowingContent, following_sibling_content};
        for (suffix, expected) in [
            ("<!--c--> \x0C\n", FollowingContent::Absent),
            ("<!--c--> \x0C\nword", FollowingContent::NotInline),
            ("<!--c-->word", FollowingContent::Inline),
            ("<!--c--><em>x</em>", FollowingContent::Inline),
            ("<!--c--><script>x</script>", FollowingContent::Inline),
            ("<!--c--><br>", FollowingContent::Inline),
            ("<!--c--><div>x</div>", FollowingContent::NotInline),
            ("<!--c-->&nbsp;word", FollowingContent::Inline),
        ] {
            let html = format!("<span>x</span>{suffix}");
            let dom = tl::parse(&html, tl::ParserOptions::default()).expect("parse HTML");
            let parser = dom.parser();
            let ctx = crate::converter::utility::caching::build_dom_context(&dom, parser, html.len());
            let first = dom.children()[0].get_inner();
            assert_eq!(following_sibling_content(first, parser, &ctx), expected, "{suffix:?}");
        }
    }

    #[test]
    fn should_preserve_comment_form_feed_content_on_all_conversion_routes() {
        for count in [100, 200, 400] {
            let run = "<!--c-->\x0C\n".repeat(count);
            for html in [format!("a\n{run}<br>b"), format!("<span>a\n{run}</span><br>b")] {
                let full = format!("a{}  \nb\n", "\x0C".repeat(count));
                let fast = format!("a\n{}  \nb\n", "\x0C\n".repeat(count).trim_end_matches('\n'));
                let text = full.as_str();
                assert_eq!(text.matches('\x0C').count(), count);
                assert!(!text.contains("<!--"));
                assert!(text.starts_with('a') && text.ends_with("b\n"));
                for (tier_strategy, extract_metadata) in [
                    (TierStrategy::Auto, false),
                    (TierStrategy::Auto, true),
                    (TierStrategy::Tier1, false),
                    (TierStrategy::Tier1, true),
                    (TierStrategy::Tier2, false),
                    (TierStrategy::Tier2, true),
                ] {
                    let options = ConversionOptions {
                        extract_metadata,
                        tier_strategy,
                        ..Default::default()
                    };
                    let expected = if tier_strategy == TierStrategy::Tier1 {
                        &fast
                    } else {
                        &full
                    };
                    assert_eq!(
                        convert(&html, options).expect("convert HTML").content.as_deref(),
                        Some(expected.as_str())
                    );
                }
                assert_eq!(
                    convert(&html, None).expect("default converter").content.as_deref(),
                    Some(full.as_str())
                );
                let options = ConversionOptions {
                    extract_metadata: false,
                    ..Default::default()
                };
                let (page, report) = crate::prescan::run(&html);
                assert_eq!(
                    crate::tier1::run(&page, &report, &options).expect("raw fast converter"),
                    fast
                );
            }
        }
    }

    #[cfg(feature = "visitor")]
    #[test]
    fn should_visit_every_text_in_comment_form_feed_runs() {
        use std::sync::{Arc, Mutex};
        #[derive(Debug)]
        struct TextVisits(usize);
        impl crate::visitor::HtmlVisitor for TextVisits {
            fn visit_text(&mut self, _ctx: &crate::visitor::NodeContext, _text: &str) -> crate::visitor::VisitResult {
                self.0 += 1;
                crate::visitor::VisitResult::Continue
            }
        }
        for count in [100, 200, 400] {
            let html = format!("<span>a\n{}</span><br>b", "<!--c-->\x0C\n".repeat(count));
            let visitor = Arc::new(Mutex::new(TextVisits(0)));
            let options = ConversionOptions {
                visitor: Some(visitor.clone()),
                extract_metadata: false,
                ..Default::default()
            };
            let text = convert(&html, options)
                .expect("visitor conversion")
                .content
                .expect("content");
            assert_eq!(text.matches('\x0C').count(), count);
            assert_eq!(visitor.lock().expect("visitor state").0, count + 2);
        }
    }

    #[test]
    fn should_bound_comment_form_feed_sibling_lookahead() {
        for count in [100, 200, 400] {
            let html = format!("<span>a\n{}</span><br>b", "<!--c-->\x0C\n".repeat(count));
            let options = ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: false,
                ..ConversionOptions::default()
            };
            super::FOLLOWING_CONTENT_READS.with(|visits| visits.set(0));
            let result = convert(&html, options).expect("convert HTML");
            let text = result.content.unwrap_or_default();
            assert_eq!(text.matches('\x0C').count(), count);
            let visits = super::FOLLOWING_CONTENT_READS.with(std::cell::Cell::get);
            assert_eq!(visits, 3 * count + 5, "{count} lines read {visits} siblings");
        }
    }

    #[test]
    fn should_read_one_sibling_for_each_text_after_an_empty_element() {
        // ~keep A scan from the first sibling for each text reads `count * (count + 2)` entries
        // ~keep more: 1,002,000 at 1,000 texts. The count of entries is the measure, not the time.
        for count in [1_000, 2_000] {
            assert_eq!(sibling_visits_for(count), count, "{count} texts");
        }
    }
}
