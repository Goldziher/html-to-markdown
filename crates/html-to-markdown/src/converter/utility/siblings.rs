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

/// The block element the content before `node_handle` in its parent ends with: the previous
/// sibling when it is a block, or the block an inline sibling's own last content ends with
/// (`<span><ul>...</ul></span>`). Whitespace text and comments are skipped (issue #585).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn previous_content_block<'a>(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &'a DomContext,
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
                if !crate::converter::main_helpers::is_inline_element(&info.name) {
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

/// Check if the previous sibling is an inline tag.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn previous_sibling_is_inline_tag(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx.previous_inline_like(*node_handle, parser)
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
