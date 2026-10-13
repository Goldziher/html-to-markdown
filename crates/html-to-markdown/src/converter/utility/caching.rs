//! Performance caching utilities.
//!
//! Caching mechanisms for expensive operations during conversion, including
//! DOM context building.

use crate::converter::DomContext;
use crate::converter::dom_context::LineEndOwner;
use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::normalized_tag_name;

#[cfg(test)]
thread_local! {
    pub(crate) static DOM_CONTEXT_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn inline_like_value(node_handle: tl::NodeHandle, parser: &tl::Parser) -> Option<bool> {
    match node_handle.get(parser) {
        Some(tl::Node::Tag(tag)) => {
            let name = normalized_tag_name(tag.name().as_utf8_str());
            Some(is_inline_element(&name) || matches!(name.as_ref(), "script" | "style"))
        }
        Some(tl::Node::Raw(raw)) if raw.as_utf8_str().trim().is_empty() => None,
        Some(tl::Node::Raw(_)) => Some(false),
        Some(tl::Node::Comment(_)) | None => None,
    }
}

fn cache_sibling_context(siblings: &[tl::NodeHandle], parser: &tl::Parser, ctx: &mut DomContext) {
    let mut previous_tag = None;
    for sibling in siblings {
        let id = sibling.get_inner();
        ctx.ensure_capacity(id);
        let _ = ctx.prev_tag_map[id as usize].set(previous_tag);
        match sibling.get(parser) {
            Some(tl::Node::Tag(_)) => previous_tag = Some(id),
            Some(tl::Node::Raw(raw)) if !raw.as_utf8_str().trim().is_empty() => previous_tag = None,
            _ => {}
        }
    }

    let mut next_inline_like = false;
    let mut next_tag = None;
    let mut next_whitespace = false;
    let mut following_content = crate::converter::utility::siblings::FollowingContent::Absent;
    for sibling in siblings.iter().rev() {
        let id = sibling.get_inner();
        ctx.ensure_capacity(id);
        let _ = ctx.next_inline_like_map[id as usize].set(next_inline_like);
        let _ = ctx.next_tag_map[id as usize].set(next_tag);
        let _ = ctx.next_whitespace_map[id as usize].set(next_whitespace);
        // ~keep Blank text and comments inherit the same following-content answer. Recording
        // ~keep it during this reverse walk avoids a suffix scan for every form-feed text (#791).
        crate::converter::utility::siblings::count_following_content_read();
        ctx.following_content_map[id as usize] = following_content;

        match sibling.get(parser) {
            Some(tl::Node::Tag(_)) => {
                next_inline_like = inline_like_value(*sibling, parser).unwrap_or(false);
                next_tag = Some(id);
                next_whitespace = false;
                following_content = if next_inline_like {
                    crate::converter::utility::siblings::FollowingContent::Inline
                } else {
                    crate::converter::utility::siblings::FollowingContent::NotInline
                };
            }
            Some(tl::Node::Raw(raw)) => {
                let is_whitespace = raw.as_utf8_str().trim().is_empty();
                if !is_whitespace {
                    next_inline_like = false;
                    next_tag = None;
                    following_content = if raw.as_utf8_str().starts_with(char::is_whitespace) {
                        crate::converter::utility::siblings::FollowingContent::NotInline
                    } else {
                        crate::converter::utility::siblings::FollowingContent::Inline
                    };
                }
                next_whitespace = is_whitespace;
            }
            Some(tl::Node::Comment(_)) | None => {}
        }
    }
}

/// Build a DOM context with hierarchical node information.
///
/// Pre-computes parent-child relationships, sibling indices, and caches
/// tag information for efficient DOM navigation during conversion.
#[must_use]
pub fn build_dom_context(dom: &tl::VDom, parser: &tl::Parser, _input_len: usize) -> DomContext {
    #[cfg(test)]
    DOM_CONTEXT_BUILDS.with(|builds| builds.set(builds.get() + 1));
    let mut ctx = DomContext {
        parent_map: Vec::new(),
        literal_pre_map: Vec::new(),
        line_end_owner_map: Vec::new(),
        children_map: Vec::new(),
        sibling_index_map: Vec::new(),
        root_children: dom.children().to_vec(),
        node_map: Vec::new(),
        tag_info_map: Vec::new(),
        next_inline_like_map: Vec::new(),
        prev_tag_map: Vec::new(),
        next_tag_map: Vec::new(),
        next_whitespace_map: Vec::new(),
        following_content_map: Vec::new(),
        table_content_summary_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        datalist_content_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        holds_text_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        silent_heading_images: std::cell::RefCell::new(std::collections::HashSet::new()),
        blank_font_renders: std::cell::RefCell::new(std::collections::HashSet::new()),
    };

    // ~keep Node ids index `dom.nodes()`, so the arena length is an exact upper bound on them.
    // ~keep Sizing every map once here replaces incremental grows with one allocation per map.
    if let Some(highest_node_id) = dom.nodes().len().checked_sub(1).and_then(|id| u32::try_from(id).ok()) {
        ctx.ensure_capacity(highest_node_id);
    }

    for (index, child_handle) in dom.children().iter().enumerate() {
        let id = child_handle.get_inner();
        ctx.ensure_capacity(id);
        ctx.sibling_index_map[id as usize] = Some(index);
        record_node_hierarchy(*child_handle, None, parser, &mut ctx);
    }
    cache_sibling_context(dom.children(), parser, &mut ctx);

    ctx
}

/// Record node hierarchy into DOM context.
///
/// Builds the complete parent-child relationship map for efficient tree traversal.
pub fn record_node_hierarchy(
    node_handle: tl::NodeHandle,
    parent: Option<u32>,
    parser: &tl::Parser,
    ctx: &mut DomContext,
) {
    let mut work = vec![(node_handle, parent)];
    while let Some((node_handle, parent)) = work.pop() {
        let id = node_handle.get_inner();
        ctx.ensure_capacity(id);
        ctx.parent_map[id as usize] = parent;
        // ~keep Preserve literal pre ownership at every depth without rescanning ancestors (#868).
        ctx.literal_pre_map[id as usize] = parent
            .and_then(|parent| ctx.literal_pre_map.get(parent as usize))
            .copied()
            .unwrap_or(false);
        ctx.line_end_owner_map[id as usize] = parent
            .and_then(|parent| ctx.line_end_owner_map.get(parent as usize))
            .copied()
            .flatten();
        ctx.node_map[id as usize] = Some(node_handle);

        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            ctx.literal_pre_map[id as usize] |= tag.name().as_utf8_str().eq_ignore_ascii_case("pre");
            let name = tag.name().as_utf8_str();
            // ~keep Code renders its entire subtree literally, including a nested del (#868).
            if name.eq_ignore_ascii_case("code") {
                ctx.line_end_owner_map[id as usize] = Some(LineEndOwner::Code);
            } else if name.eq_ignore_ascii_case("del")
                && ctx.line_end_owner_map[id as usize] != Some(LineEndOwner::Code)
            {
                ctx.line_end_owner_map[id as usize] = Some(LineEndOwner::Del);
            }

            let tag_children = tag.children();
            // ~keep `RawChildren::iter` reports no size hint, so collecting through it regrows the
            // ~keep Vec from empty (and over-allocates to the growth step). Copying the slice sizes
            // ~keep the allocation exactly once; the Vec itself must be owned because `children_map`
            // ~keep outlives this borrow of the parser.
            let children = tag_children.top().as_slice();
            for (index, child) in children.iter().enumerate() {
                let child_id = child.get_inner();
                ctx.ensure_capacity(child_id);
                ctx.sibling_index_map[child_id as usize] = Some(index);
                work.push((*child, Some(id)));
            }
            cache_sibling_context(children, parser, ctx);
            ctx.children_map[id as usize] = Some(children.to_vec());
        }
    }
}
