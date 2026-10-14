//! DOM context providing efficient access to parent/child relationships and text content.
//!
//! This module defines the `DomContext` structure which is built once during conversion
//! and provides O(1) access to node relationships via precomputed maps.

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;

use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::normalized_tag_name;
use crate::converter::utility::escaping::is_block_level_name;
use crate::text;

/// Cached information about an HTML tag element.
///
/// This struct stores pre-computed information about tag elements to avoid
/// repeated parsing during tree traversal.
pub struct TagInfo {
    /// The normalized (lowercase) tag name.
    pub(crate) name: String,
    /// Whether this element behaves like an inline element (including script/style).
    pub(crate) is_inline_like: bool,
    /// Whether this element is a block-level element.
    pub(crate) is_block: bool,
}

/// Memoized summary of a `<table>` element's full subtree (including nested tables):
/// whether it contains non-whitespace text, links, header cells, or a caption.
///
/// ~keep Caching this per table node (see [`DomContext::cached_table_content_summary`]) is
/// ~keep what keeps `scanner::scan_table` linear across a chain of nested tables — without it,
/// ~keep every table's own full-subtree scan re-walks every table nested below it, an O(n²)
/// ~keep cost documented at `tests/deep_nesting_overflow.rs`.
#[derive(Clone, Copy, Default)]
pub struct TableContentSummary {
    pub(crate) has_text: bool,
    pub(crate) link_count: usize,
    pub(crate) has_header: bool,
    pub(crate) has_caption: bool,
}

impl TableContentSummary {
    pub(crate) const fn merge(&mut self, other: Self) {
        self.has_text |= other.has_text;
        self.link_count += other.link_count;
        self.has_header |= other.has_header;
        self.has_caption |= other.has_caption;
    }
}

/// DOM context that provides efficient access to parent/child relationships and text content.
///
/// This context is built once during conversion and provides O(1) access to node relationships
/// via precomputed maps.
pub struct DomContext {
    pub(crate) parent_map: Vec<Option<u32>>,
    pub(crate) children_map: Vec<Option<Vec<tl::NodeHandle>>>,
    pub(crate) sibling_index_map: Vec<Option<usize>>,
    pub(crate) root_children: Vec<tl::NodeHandle>,
    pub(crate) node_map: Vec<Option<tl::NodeHandle>>,
    pub(crate) tag_info_map: Vec<OnceCell<Option<TagInfo>>>,
    pub(crate) prev_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) prev_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_whitespace_map: Vec<OnceCell<bool>>,
    pub(crate) table_content_summary_cache: RefCell<HashMap<u32, TableContentSummary>>,
}

impl DomContext {
    pub(crate) fn ensure_capacity(&mut self, id: u32) {
        let idx = id as usize;
        if self.parent_map.len() <= idx {
            let new_len = idx + 1;
            self.parent_map.resize(new_len, None);
            self.children_map.resize_with(new_len, || None);
            self.sibling_index_map.resize_with(new_len, || None);
            self.node_map.resize(new_len, None);
            self.tag_info_map.resize_with(new_len, OnceCell::new);
            self.prev_inline_like_map.resize_with(new_len, OnceCell::new);
            self.next_inline_like_map.resize_with(new_len, OnceCell::new);
            self.prev_tag_map.resize_with(new_len, OnceCell::new);
            self.next_tag_map.resize_with(new_len, OnceCell::new);
            self.next_whitespace_map.resize_with(new_len, OnceCell::new);
        }
    }

    pub(crate) fn parent_of(&self, id: u32) -> Option<u32> {
        self.parent_map.get(id as usize).copied().flatten()
    }

    pub(crate) fn node_handle(&self, id: u32) -> Option<&tl::NodeHandle> {
        self.node_map.get(id as usize).and_then(|node| node.as_ref())
    }

    pub(crate) fn children_of(&self, id: u32) -> Option<&Vec<tl::NodeHandle>> {
        self.children_map
            .get(id as usize)
            .and_then(|children| children.as_ref())
    }

    pub(crate) fn sibling_index(&self, id: u32) -> Option<usize> {
        self.sibling_index_map.get(id as usize).copied().flatten()
    }

    pub(crate) fn tag_info(&self, id: u32, parser: &tl::Parser) -> Option<&TagInfo> {
        self.tag_info_map
            .get(id as usize)
            .and_then(|cell| cell.get_or_init(|| self.build_tag_info(id, parser)).as_ref())
    }

    pub(crate) fn tag_name_for<'a>(
        &'a self,
        node_handle: tl::NodeHandle,
        parser: &'a tl::Parser,
    ) -> Option<std::borrow::Cow<'a, str>> {
        if let Some(info) = self.tag_info(node_handle.get_inner(), parser) {
            return Some(std::borrow::Cow::Borrowed(info.name.as_str()));
        }
        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            return Some(normalized_tag_name(tag.name().as_utf8_str()));
        }
        None
    }

    pub(crate) fn next_tag_name<'a>(&'a self, node_handle: tl::NodeHandle, parser: &'a tl::Parser) -> Option<&'a str> {
        let next_id = self.next_tag_id(node_handle.get_inner(), parser)?;
        self.tag_info(next_id, parser).map(|info| info.name.as_str())
    }

    pub(crate) fn previous_inline_like(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> bool {
        let id = node_handle.get_inner();
        self.prev_inline_like_map.get(id as usize).is_some_and(|cell| {
            *cell.get_or_init(|| {
                let parent = self.parent_of(id);
                let siblings = if let Some(parent_id) = parent {
                    if let Some(children) = self.children_of(parent_id) {
                        children
                    } else {
                        return false;
                    }
                } else {
                    &self.root_children
                };

                let Some(position) = self
                    .sibling_index(id)
                    .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                else {
                    return false;
                };

                for sibling in siblings.iter().take(position).rev() {
                    if let Some(info) = self.tag_info(sibling.get_inner(), parser) {
                        return info.is_inline_like;
                    }
                    if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                        if raw.as_utf8_str().trim().is_empty() {
                            continue;
                        }
                        return false;
                    }
                }

                false
            })
        })
    }

    pub(crate) fn next_inline_like(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> bool {
        let id = node_handle.get_inner();
        self.next_inline_like_map.get(id as usize).is_some_and(|cell| {
            *cell.get_or_init(|| {
                let parent = self.parent_of(id);
                let siblings = if let Some(parent_id) = parent {
                    if let Some(children) = self.children_of(parent_id) {
                        children
                    } else {
                        return false;
                    }
                } else {
                    &self.root_children
                };

                let Some(position) = self
                    .sibling_index(id)
                    .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                else {
                    return false;
                };

                for sibling in siblings.iter().skip(position + 1) {
                    if let Some(info) = self.tag_info(sibling.get_inner(), parser) {
                        return info.is_inline_like;
                    }
                    if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                        if raw.as_utf8_str().trim().is_empty() {
                            continue;
                        }
                        return false;
                    }
                }

                false
            })
        })
    }

    pub(crate) fn next_whitespace_text(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> bool {
        let id = node_handle.get_inner();
        self.next_whitespace_map.get(id as usize).is_some_and(|cell| {
            *cell.get_or_init(|| {
                let parent = self.parent_of(id);
                let siblings = if let Some(parent_id) = parent {
                    if let Some(children) = self.children_of(parent_id) {
                        children
                    } else {
                        return false;
                    }
                } else {
                    &self.root_children
                };

                let Some(position) = self
                    .sibling_index(id)
                    .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                else {
                    return false;
                };

                for sibling in siblings.iter().skip(position + 1) {
                    if let Some(node) = sibling.get(parser) {
                        match node {
                            tl::Node::Raw(raw) => return raw.as_utf8_str().trim().is_empty(),
                            tl::Node::Tag(_) => return false,
                            tl::Node::Comment(_) => {}
                        }
                    }
                }

                false
            })
        })
    }

    pub(crate) fn next_tag_id(&self, id: u32, parser: &tl::Parser) -> Option<u32> {
        self.next_tag_map
            .get(id as usize)
            .and_then(|cell| {
                cell.get_or_init(|| {
                    let parent = self.parent_of(id);
                    let siblings = if let Some(parent_id) = parent {
                        self.children_of(parent_id)?
                    } else {
                        &self.root_children
                    };

                    let position = self
                        .sibling_index(id)
                        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))?;

                    for sibling in siblings.iter().skip(position + 1) {
                        if self.tag_info(sibling.get_inner(), parser).is_some() {
                            let sibling_id = sibling.get_inner();
                            return Some(sibling_id);
                        }
                        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                            if !raw.as_utf8_str().trim().is_empty() {
                                return None;
                            }
                        }
                    }
                    None
                })
                .as_ref()
            })
            .copied()
    }

    pub(crate) fn build_tag_info(&self, id: u32, parser: &tl::Parser) -> Option<TagInfo> {
        let node_handle = self.node_handle(id)?;
        match node_handle.get(parser) {
            Some(tl::Node::Tag(tag)) => {
                let name = normalized_tag_name(tag.name().as_utf8_str()).into_owned();
                let is_inline = is_inline_element(&name);
                let is_inline_like = is_inline || matches!(name.as_str(), "script" | "style");
                let is_block = is_block_level_name(&name, is_inline);
                Some(TagInfo {
                    name,
                    is_inline_like,
                    is_block,
                })
            }
            _ => None,
        }
    }

    /// ~keep Deliberately uncached. An LRU keyed by node id used to live here; it was removed
    /// ~keep because a cache *hit* still cloned - every caller needs an owned `String` - so the
    /// ~keep cache only ever saved the subtree walk while adding one allocation on every miss.
    /// ~keep Since 3a7c115a4 the table width pre-pass reuses its rendered cell text instead of
    /// ~keep re-reading it, so the dominant repeat-read is gone and the hit rate can only fall.
    pub(crate) fn text_content(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> String {
        self.text_content_uncached(node_handle, parser)
    }

    pub(crate) fn text_content_uncached(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> String {
        let mut text = String::with_capacity(64);

        let mut stack = vec![node_handle];
        while let Some(handle) = stack.pop() {
            let Some(node) = handle.get(parser) else {
                continue;
            };

            match node {
                tl::Node::Raw(bytes) => {
                    let raw = bytes.as_utf8_str();
                    let decoded = text::decode_html_entities_cow(raw.as_ref());
                    text.push_str(decoded.as_ref());
                }
                tl::Node::Tag(tag) => {
                    if let Some(children) = self.children_of(handle.get_inner()) {
                        for child_handle in children.iter().rev() {
                            stack.push(*child_handle);
                        }
                    } else {
                        let children = tag.children();
                        let mut child_handles: Vec<_> = children.top().iter().copied().collect();
                        child_handles.reverse();
                        for child_handle in child_handles {
                            stack.push(child_handle);
                        }
                    }
                }
                tl::Node::Comment(_) => {}
            }
        }
        text
    }

    /// Get the parent tag name for a given node ID.
    ///
    /// Returns a borrow into the cached [`TagInfo`] for the parent element,
    /// or `None` if there is no tag parent. The result has the same lifetime
    /// as `&self` because `TagInfo` is stored in a `OnceCell` owned by this
    /// context.
    pub(crate) fn parent_tag_name<'s>(&'s self, node_id: u32, parser: &tl::Parser) -> Option<&'s str> {
        let parent_id = self.parent_of(node_id)?;
        self.tag_info(parent_id, parser).map(|info| info.name.as_str())
    }

    pub(crate) fn has_raw_text_ancestor(&self, node_id: u32, parser: &tl::Parser) -> bool {
        let mut current_id = node_id;
        while let Some(parent_id) = self.parent_of(current_id) {
            if self.tag_info(parent_id, parser).is_some_and(|info| {
                crate::converter::tier1::tags::lookup(info.name.as_bytes()).is_some_and(|spec| spec.is_rawtext)
            }) {
                return true;
            }
            current_id = parent_id;
        }
        false
    }

    /// Get the index of a node among its siblings.
    ///
    /// Returns the 0-based index if the node has siblings,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn get_sibling_index(&self, node_id: u32) -> Option<usize> {
        self.sibling_index(node_id)
    }

    /// Look up a memoized table content summary, if already computed.
    pub(crate) fn cached_table_content_summary(&self, table_id: u32) -> Option<TableContentSummary> {
        self.table_content_summary_cache.borrow().get(&table_id).copied()
    }

    /// Store a computed table content summary for reuse by ancestor/descendant table scans.
    pub(crate) fn cache_table_content_summary(&self, table_id: u32, summary: TableContentSummary) {
        self.table_content_summary_cache.borrow_mut().insert(table_id, summary);
    }
}
