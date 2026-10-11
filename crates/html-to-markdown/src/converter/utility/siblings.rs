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
pub fn following_sibling_content(id: u32, parser: &tl::Parser, dom_ctx: &DomContext) -> FollowingContent {
    let siblings = match dom_ctx.parent_of(id) {
        Some(parent_id) => match dom_ctx.children_of(parent_id) {
            Some(children) => children,
            None => return FollowingContent::Absent,
        },
        None => &dom_ctx.root_children,
    };

    let Some(position) = dom_ctx
        .sibling_index(id)
        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
    else {
        return FollowingContent::Absent;
    };

    for sibling in siblings.iter().skip(position + 1) {
        if let Some(info) = dom_ctx.tag_info(sibling.get_inner(), parser) {
            return if info.is_inline_like {
                FollowingContent::Inline
            } else {
                FollowingContent::NotInline
            };
        }
        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
            let decoded = raw.as_utf8_str();
            if decoded.trim().is_empty() {
                continue;
            }
            // ~keep Text that already opens with whitespace supplies the separator itself, so
            // ~keep reporting it as inline content here stacks a second space on top of it.
            // ~keep `<span>\n</span>\n   Tip` (a Docusaurus admonition icon) regressed from
            // ~keep ") Tip" to ")  Tip" exactly this way -- the wrapper is only load-bearing
            // ~keep when the next word butts straight up against it, as in issue #491's
            // ~keep `Alpha<span>\n</span>13`.
            return if decoded.starts_with(char::is_whitespace) {
                FollowingContent::NotInline
            } else {
                FollowingContent::Inline
            };
        }
    }

    FollowingContent::Absent
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
