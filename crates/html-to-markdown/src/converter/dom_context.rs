//! DOM context providing efficient access to parent/child relationships and text content.
//!
//! This module defines the `DomContext` structure which is built once during conversion
//! and provides O(1) access to node relationships via precomputed maps.

use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::normalized_tag_name;
use crate::converter::utility::escaping::is_block_level_name;
use crate::converter::utility::white_space::{TextClass, classify_text};
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

    /// Whether the scan counted nothing.
    pub(crate) const fn is_empty(&self) -> bool {
        !self.has_text && self.link_count == 0 && !self.has_header && !self.has_caption
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineEndOwner {
    Code,
    Del,
}

/// DOM context that provides efficient access to parent/child relationships and text content.
///
/// This context is built once during conversion and provides O(1) access to node relationships
/// via precomputed maps.
pub struct DomContext {
    pub(crate) parent_map: Vec<Option<u32>>,
    pub(crate) literal_pre_map: Vec<bool>,
    pub(crate) line_end_owner_map: Vec<Option<LineEndOwner>>,
    pub(crate) children_map: Vec<Option<Vec<tl::NodeHandle>>>,
    pub(crate) sibling_index_map: Vec<Option<usize>>,
    pub(crate) root_children: Vec<tl::NodeHandle>,
    pub(crate) node_map: Vec<Option<tl::NodeHandle>>,
    pub(crate) tag_info_map: Vec<OnceCell<Option<TagInfo>>>,
    pub(crate) next_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) prev_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) following_content_map: Vec<crate::converter::utility::siblings::FollowingContent>,
    pub(crate) next_whitespace_map: Vec<OnceCell<bool>>,
    pub(crate) table_content_summary_cache: RefCell<HashMap<u32, TableContentSummary>>,
    pub(crate) datalist_content_cache: RefCell<HashMap<u32, bool>>,
    /// For each element that [`DomContext::holds_text`] read: whether it holds text.
    pub(crate) holds_text_cache: RefCell<HashMap<u32, bool>>,
    pub(crate) silent_heading_images: RefCell<HashSet<u32>>,
    pub(crate) blank_passthrough_renders: RefCell<HashSet<u32>>,
    pub(crate) authored_break_cache: RefCell<HashMap<u32, bool>>,
    pub(crate) omitted_break_owners: RefCell<HashSet<u32>>,
}

#[cfg(test)]
thread_local! {
    /// The nodes that `DomContext::holds_text` and `DomContext::holds_content` read on this thread.
    pub(crate) static HOLDS_TEXT_NODE_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static AUTHORED_BREAK_CACHE_STATS: std::cell::Cell<(usize,usize)> = const { std::cell::Cell::new((0,0)) };
    /// Actual emitted bytes examined by observed passthrough registration. ~keep
    pub(crate) static FONT_RENDER_BYTE_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Number of actual observed font registration guards, including length rejections. ~keep
    pub(crate) static FONT_RENDER_GUARD_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Maximum actual sparse observed-font entries and reserved capacity during this test. ~keep
    pub(crate) static FONT_RENDER_SET_STATS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

#[cfg(test)]
fn count_node_read() {
    HOLDS_TEXT_NODE_READS.with(|total| total.set(total.get() + 1));
}

#[cfg(not(test))]
const fn count_node_read() {}

/// What one node is to a walk of [`DomContext::holds`].
enum Held {
    /// The node is what the walk looks for.
    Yes,
    /// The node is not.
    No,
    /// The node is an element: the walk reads its nodes.
    Inside,
}

/// The answer that a walk recorded for the element `id`.
fn recorded(record: &RefCell<HashMap<u32, bool>>, id: u32) -> Option<bool> {
    record.borrow().get(&id).copied()
}

impl DomContext {
    pub(crate) fn ensure_capacity(&mut self, id: u32) {
        let idx = id as usize;
        if self.parent_map.len() <= idx {
            let new_len = idx + 1;
            self.parent_map.resize(new_len, None);
            self.literal_pre_map.resize(new_len, false);
            self.line_end_owner_map.resize(new_len, None);
            self.children_map.resize_with(new_len, || None);
            self.sibling_index_map.resize_with(new_len, || None);
            self.node_map.resize(new_len, None);
            self.tag_info_map.resize_with(new_len, OnceCell::new);
            self.next_inline_like_map.resize_with(new_len, OnceCell::new);
            self.prev_tag_map.resize_with(new_len, OnceCell::new);
            self.next_tag_map.resize_with(new_len, OnceCell::new);
            self.next_whitespace_map.resize_with(new_len, OnceCell::new);
            self.following_content_map
                .resize(new_len, crate::converter::utility::siblings::FollowingContent::Absent);
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

    /// Whether the element `id` holds text: a text node in it, at any depth, has a character that
    /// is not collapsible white space ([`classify_text`]). A comment is no text, and an element
    /// with no text node (an image, a `<br>`) holds none.
    ///
    /// ~keep The walk reads the nodes in the order of the document and stops at the first text.
    /// ~keep It records the answer of each element that it leaves or stops in, so no node is read
    /// ~keep twice in one conversion: the texts after `n` nested elements ask `n` times, and
    /// ~keep without the record each of them reads the elements below it again.
    /// ~keep A loop rather than recursion: nested elements are attacker-controlled depth.
    pub(crate) fn holds_text(&self, id: u32, parser: &tl::Parser) -> bool {
        self.holds(id, parser, &self.holds_text_cache, |node, _| match node {
            tl::Node::Raw(raw) => {
                let raw = raw.as_utf8_str();
                let decoded = text::decode_html_entities_cow(raw.as_ref());
                if matches!(classify_text(decoded.as_ref()), TextClass::WhiteSpaceOnly(_)) {
                    Held::No
                } else {
                    Held::Yes
                }
            }
            tl::Node::Tag(_) => Held::Inside,
            tl::Node::Comment(_) => Held::No,
        })
    }

    /// Whether the element `id` holds content: text other than code/del-owned line ends, literal pre content,
    /// or a void element
    /// that can write Markdown outside a table cell (a break, a rule or an image), at any depth.
    /// A comment is no content, and an element with
    /// neither in it is none: `<span><b></b></span>` and `<span><span><!-- c --></span></span>`
    /// hold none, and `<span><br></span>` holds content.
    ///
    /// ~keep A void element writes output with no content of its own (a `<br>`, an image): the
    /// ~keep element around it is not empty.
    /// ~keep The walk is that of [`DomContext::holds_text`], with a record that lives for this
    /// ~keep one question. The caller asks only for an element that holds no text, right before a
    /// ~keep text that starts with a Unicode space. Such a text is text, so no element that is
    /// ~keep asked about holds another one that is asked about: no node is read for two questions.
    pub(crate) fn holds_content(&self, id: u32, parser: &tl::Parser) -> bool {
        let record = RefCell::new(HashMap::new());
        self.holds(id, parser, &record, |node, child| match node {
            tl::Node::Raw(raw) => {
                let raw = raw.as_utf8_str();
                let decoded = text::decode_html_entities_cow(raw.as_ref());
                let parent = self.parent_tag_name(child, parser);
                let owner = self.line_end_owner_map.get(child as usize).copied().flatten();
                let line_ends_only = match owner {
                    Some(LineEndOwner::Code) => decoded.trim_matches('\n').is_empty(),
                    // ~keep Nested CR-only spans retain their existing boundary; only authored line ends collapse (#868).
                    Some(LineEndOwner::Del) => {
                        decoded.trim_matches(['\n', '\r']).is_empty()
                            && (parent == Some("del") || decoded.contains('\n'))
                    }
                    None => false,
                };
                if line_ends_only && !self.literal_pre_map.get(child as usize).copied().unwrap_or(false) {
                    Held::No
                } else {
                    Held::Yes
                }
            }
            tl::Node::Tag(_) if self.writes_void_content(child, parser) => Held::Yes,
            tl::Node::Tag(_) => Held::Inside,
            tl::Node::Comment(_) => Held::No,
        })
    }

    pub(crate) fn begin_passthrough_render(
        &self,
        id: u32,
        output_len: usize,
        handler: crate::converter::block::container::HandlerContext<'_>,
    ) -> Option<usize> {
        let ctx = handler.ctx;
        #[cfg(feature = "visitor")]
        let observer_suppressed = ctx.skip_visitor_hooks && ctx.has_observer();
        #[cfg(not(feature = "visitor"))]
        let observer_suppressed = false;
        (!observer_suppressed
            && !ctx.code_element_is_transparent()
            && !ctx.in_code_block
            && handler.options.whitespace_mode != crate::WhitespaceMode::Strict)
            .then(|| {
                self.blank_passthrough_renders.borrow_mut().remove(&id);
                output_len
            })
    }

    pub(crate) fn record_blank_passthrough_render(&self, id: u32, written: &str) {
        #[cfg(test)]
        FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(calls.get() + 1));
        let blank = written.is_empty()
            || (written.len() == 1 && {
                #[cfg(test)]
                FONT_RENDER_BYTE_READS.with(|reads| reads.set(reads.get() + 1));
                matches!(written.as_bytes()[0], b' ' | b'\t')
            });
        if blank {
            let mut fonts = self.blank_passthrough_renders.borrow_mut();
            fonts.insert(id);
            #[cfg(test)]
            FONT_RENDER_SET_STATS.with(|stats| {
                let (entries, capacity) = stats.get();
                stats.set((entries.max(fonts.len()), capacity.max(fonts.capacity())));
            });
        }
    }

    pub(crate) fn holds_authored_break(&self, id: u32, parser: &tl::Parser) -> bool {
        let found = self.holds(id, parser, &self.authored_break_cache, |node, child| match node {
            tl::Node::Tag(_) if self.tag_info(child, parser).is_some_and(|info| info.name == "br") => {
                if recorded(&self.authored_break_cache, child) == Some(false) {
                    Held::No
                } else {
                    Held::Yes
                }
            }
            tl::Node::Tag(_) => Held::Inside,
            tl::Node::Raw(_) | tl::Node::Comment(_) => Held::No,
        });
        #[cfg(test)]
        AUTHORED_BREAK_CACHE_STATS.with(|stats| {
            let cache = self.authored_break_cache.borrow();
            let (entries, capacity) = stats.get();
            stats.set((entries.max(cache.len()), capacity.max(cache.capacity())));
        });
        found
    }

    // ~keep Strict wrappers must not transport BRs in omitted subtrees or space-only callbacks.
    // ~keep Later renders can retain them; status changes invalidate ancestors, preserving siblings.
    // ~keep Ordinary negative query memo entries are not omissions and survive retaining renders.
    pub(crate) fn record_absent_breaks(&self, id: u32, skipped: bool) {
        let changed = if skipped {
            self.omitted_break_owners.borrow_mut().insert(id)
        } else {
            self.omitted_break_owners.borrow_mut().remove(&id)
        };
        if !changed {
            return;
        }
        let mut cache = self.authored_break_cache.borrow_mut();
        if skipped {
            cache.insert(id, false);
        } else {
            cache.remove(&id);
        }
        let mut parent = self.parent_of(id);
        while let Some(id) = parent {
            cache.remove(&id);
            parent = self.parent_of(id);
        }
    }

    /// ~keep Code/del can render an empty line end, and a break can render whitespace; neither is a blank passthrough element.
    pub(crate) fn passthrough_boundary_has_content(&self, id: u32, parser: &tl::Parser) -> bool {
        let record = RefCell::new(HashMap::new());
        self.holds(id, parser, &record, |node, child| match node {
            tl::Node::Raw(raw) => {
                let raw = raw.as_utf8_str();
                let decoded = text::decode_html_entities_cow(raw.as_ref());
                if matches!(classify_text(decoded.as_ref()), TextClass::WhiteSpaceOnly(_)) {
                    Held::No
                } else {
                    Held::Yes
                }
            }
            tl::Node::Tag(_)
                if self.writes_void_content(child, parser)
                    || self
                        .tag_info(child, parser)
                        .is_some_and(|info| matches!(info.name.as_str(), "code" | "del" | "pre")) =>
            {
                Held::Yes
            }
            tl::Node::Tag(_) => Held::Inside,
            tl::Node::Comment(_) => Held::No,
        })
    }

    /// ~keep Silent void descendants do not separate the line end from a following NBSP (#868).
    /// ~keep That boundary rule skips table cells, whose checkboxes can write their state.
    fn writes_void_content(&self, id: u32, parser: &tl::Parser) -> bool {
        self.tag_info(id, parser).is_some_and(|info| match info.name.as_str() {
            "br" | "hr" => true,
            "img" => !self.silent_heading_images.borrow().contains(&id),
            _ => false,
        })
    }

    /// Whether a node in the element `id` is held, as `held` answers for each node, with the
    /// answers of `record`.
    fn holds(
        &self,
        id: u32,
        parser: &tl::Parser,
        record: &RefCell<HashMap<u32, bool>>,
        held: impl Fn(&tl::Node, u32) -> Held,
    ) -> bool {
        if let Some(known) = recorded(record, id) {
            return known;
        }
        let mut open = vec![(id, 0_usize)];
        while let Some((element, position)) = open.last_mut() {
            let child = self
                .children_of(*element)
                .and_then(|children| children.get(*position))
                .copied();
            *position += 1;
            let Some(child) = child else {
                record.borrow_mut().insert(*element, false);
                open.pop();
                continue;
            };
            count_node_read();
            let found = match child.get(parser).map(|node| held(node, child.get_inner())) {
                Some(Held::Yes) => true,
                Some(Held::Inside) => {
                    let known = recorded(record, child.get_inner());
                    if known.is_none() {
                        open.push((child.get_inner(), 0));
                    }
                    known.unwrap_or(false)
                }
                Some(Held::No) | None => false,
            };
            if found {
                for (element, _) in &open {
                    record.borrow_mut().insert(*element, true);
                }
                return true;
            }
        }
        false
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

#[cfg(test)]
mod tests {
    #[test]
    fn should_keep_ordinary_negative_break_memo_when_no_owner_was_omitted() {
        let source = "<b><span> </span></b>";
        let dom = tl::parse(source, tl::ParserOptions::default()).expect("blank nodes");
        let context = crate::converter::utility::caching::build_dom_context(&dom, dom.parser(), source.len());
        let root = dom.children()[0].get_inner();
        let child = context.children_of(root).expect("one span")[0].get_inner();
        super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        assert!(!context.holds_authored_break(root, dom.parser()));
        super::HOLDS_TEXT_NODE_READS.with(|reads| assert_eq!(reads.get(), 2));
        context.record_absent_breaks(child, false);
        super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        assert!(!context.holds_authored_break(root, dom.parser()));
        super::HOLDS_TEXT_NODE_READS.with(|reads| assert_eq!(reads.get(), 0));
    }

    #[cfg(feature = "visitor")]
    #[test]
    fn should_refresh_cached_break_ancestors_without_forgetting_siblings() {
        let source = "<b><span><br></span><i><br></i></b>";
        for order in [[0, 1], [1, 0]] {
            let dom = tl::parse(source, tl::ParserOptions::default()).expect("break siblings");
            let parser = dom.parser();
            let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
            let root = dom.children()[0].get_inner();
            let breaks = dom
                .query_selector("br")
                .expect("break selector")
                .map(|handle| handle.get_inner())
                .collect::<Vec<_>>();
            assert_eq!(breaks.len(), 2);
            assert!(context.holds_authored_break(root, parser));
            context.record_absent_breaks(breaks[order[0]], true);
            assert!(context.holds_authored_break(root, parser));
            context.record_absent_breaks(breaks[order[1]], true);
            assert!(!context.holds_authored_break(root, parser));
            context.record_absent_breaks(breaks[order[0]], false);
            assert!(context.holds_authored_break(root, parser));
            context.record_absent_breaks(breaks[order[0]], true);
            assert!(!context.holds_authored_break(root, parser));
        }
    }

    #[test]
    fn should_invalidate_omitted_subtree_ancestors_and_keep_retained_siblings() {
        let source = "<b><span><br></span><i><br></i></b>";
        for order in [[0, 1], [1, 0]] {
            let dom = tl::parse(source, tl::ParserOptions::default()).expect("break subtrees");
            let parser = dom.parser();
            let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
            let root = dom.children()[0].get_inner();
            let children = context.children_of(root).expect("two subtrees");
            assert_eq!(children.len(), 2);
            assert!(context.holds_authored_break(root, parser));
            context.record_absent_breaks(children[order[0]].get_inner(), true);
            assert!(context.holds_authored_break(root, parser));
            context.record_absent_breaks(children[order[1]].get_inner(), true);
            assert!(!context.holds_authored_break(root, parser));
            context.record_absent_breaks(children[order[0]].get_inner(), false);
            assert!(context.holds_authored_break(root, parser));
        }
    }

    #[test]
    fn should_bound_observed_font_registration_and_boundary_reads() {
        let mut jobs = 0;
        for overlapping in [false, true] {
            for depth in [4, 16, 64, 256] {
                let closing = if overlapping {
                    "</font>&nbsp;".repeat(depth)
                } else {
                    "</font>".repeat(depth) + "&nbsp;"
                };
                let source = format!("<p>a{} {closing}b</p>", "<font>".repeat(depth));
                let dom = tl::parse(&source, tl::ParserOptions::default()).expect("font nodes");
                let nodes = dom.nodes().len();
                super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
                super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
                super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
                super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
                let result = crate::convert(
                    &source,
                    crate::ConversionOptions {
                        tier_strategy: crate::TierStrategy::Tier2,
                        extract_metadata: false,
                        max_depth: Some(1024),
                        ..Default::default()
                    },
                )
                .expect("font nesting");
                let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
                let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
                assert_eq!(result.content.as_deref(), Some("a b\n"));
                assert_eq!(byte_reads, depth);
                assert_eq!(node_reads, if overlapping { depth + 1 } else { 2 * depth });
                assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
                assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0, depth);
                assert!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1 <= 2 * depth);
                assert!(
                    byte_reads > 0 && byte_reads <= 4 * nodes,
                    "font registration {byte_reads}/{nodes}"
                );
                assert!(
                    node_reads > 0 && node_reads <= 4 * nodes,
                    "font boundary {node_reads}/{nodes}"
                );
                jobs += 1;
            }
        }
        assert_eq!(jobs, 8);
    }
    #[cfg(feature = "visitor")]
    #[test]
    fn should_bound_font_registration_with_growing_custom_child_whitespace() {
        use std::sync::{Arc, Mutex};
        #[derive(Debug)]
        struct BlankChild {
            width: usize,
            calls: usize,
        }
        impl crate::visitor::HtmlVisitor for BlankChild {
            fn visit_element_start(&mut self, ctx: &crate::visitor::NodeContext) -> crate::visitor::VisitResult {
                if ctx.tag_name == "span" {
                    self.calls += 1;
                    crate::visitor::VisitResult::Custom(" ".repeat(self.width))
                } else {
                    crate::visitor::VisitResult::Continue
                }
            }
        }
        let mut jobs = 0;
        for depth in [4, 16, 64, 256] {
            let source = format!(
                "<p>a{}<span></span>{}&nbsp;b</p>",
                "<font>".repeat(depth),
                "</font>".repeat(depth)
            );
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("custom font nodes");
            let nodes = dom.nodes().len();
            let visitor = Arc::new(Mutex::new(BlankChild { width: depth, calls: 0 }));
            super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
            super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
            let result = crate::convert(
                &source,
                crate::ConversionOptions {
                    visitor: Some(visitor.clone()),
                    tier_strategy: crate::TierStrategy::Tier2,
                    extract_metadata: false,
                    max_depth: Some(1024),
                    ..Default::default()
                },
            )
            .expect("custom child whitespace");
            let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
            let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
            assert_eq!(
                result.content.as_deref(),
                Some(format!("a{}b\n", " ".repeat(depth + 1)).as_str())
            );
            assert_eq!(visitor.lock().expect("visitor calls").calls, 1);
            assert_eq!(byte_reads, 0);
            assert_eq!(node_reads, depth);
            assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get), (0, 0));
            assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
            assert!(node_reads > 0 && node_reads <= 4 * nodes);
            jobs += 1;
        }
        assert_eq!(jobs, 4);
    }
    #[test]
    #[allow(clippy::print_stdout)]
    fn should_bound_observed_unknown_registration_and_boundary_reads() {
        let mut jobs = 0;
        for overlapping in [false, true] {
            for depth in [4, 16, 64, 256] {
                let closing = if overlapping {
                    "</x-y>&nbsp;".repeat(depth)
                } else {
                    "</x-y>".repeat(depth) + "&nbsp;"
                };
                let source = format!("<p>a{} {closing}b</p>", "<x-y>".repeat(depth));
                let dom = tl::parse(&source, tl::ParserOptions::default()).expect("unknown nodes");
                let nodes = dom.nodes().len();
                assert_eq!(nodes, if overlapping { 2 * depth + 3 } else { depth + 4 });
                super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
                super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
                super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
                super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
                let result = crate::convert(
                    &source,
                    crate::ConversionOptions {
                        tier_strategy: crate::TierStrategy::Tier2,
                        extract_metadata: false,
                        max_depth: Some(1024),
                        ..Default::default()
                    },
                )
                .expect("unknown nesting");
                let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
                let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
                assert_eq!(result.content.as_deref(), Some("a b\n"));
                assert_eq!(byte_reads, depth);
                assert_eq!(node_reads, if overlapping { depth + 1 } else { 2 * depth });
                assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
                assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0, depth);
                assert!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1 <= 2 * depth);
                assert!(
                    byte_reads > 0 && byte_reads <= 4 * nodes,
                    "unknown registration {byte_reads}/{nodes}"
                );
                assert!(
                    node_reads > 0 && node_reads <= 4 * nodes,
                    "unknown boundary {node_reads}/{nodes}"
                );
                std::println!(
                    "unknown-cost overlap={overlapping} depth={depth} nodes={nodes} bytes={byte_reads} reads={node_reads} calls={} entries={} capacity={}",
                    super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get),
                    super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0,
                    super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1
                );
                jobs += 1;
            }
        }
        assert_eq!(jobs, 8);
    }
    #[cfg(feature = "visitor")]
    #[test]
    #[allow(clippy::print_stdout)]
    fn should_bound_unknown_registration_with_growing_custom_child_whitespace() {
        use std::sync::{Arc, Mutex};
        #[derive(Debug)]
        struct BlankChild {
            width: usize,
            calls: usize,
        }
        impl crate::visitor::HtmlVisitor for BlankChild {
            fn visit_element_start(&mut self, ctx: &crate::visitor::NodeContext) -> crate::visitor::VisitResult {
                if ctx.tag_name == "span" {
                    self.calls += 1;
                    crate::visitor::VisitResult::Custom(" ".repeat(self.width))
                } else {
                    crate::visitor::VisitResult::Continue
                }
            }
        }
        let mut jobs = 0;
        for depth in [4, 16, 64, 256] {
            let source = format!(
                "<p>a{}<span></span>{}&nbsp;b</p>",
                "<x-y>".repeat(depth),
                "</x-y>".repeat(depth)
            );
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("custom unknown nodes");
            let nodes = dom.nodes().len();
            assert_eq!(nodes, depth + 4);
            let visitor = Arc::new(Mutex::new(BlankChild { width: depth, calls: 0 }));
            super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
            super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
            let result = crate::convert(
                &source,
                crate::ConversionOptions {
                    visitor: Some(visitor.clone()),
                    tier_strategy: crate::TierStrategy::Tier2,
                    extract_metadata: false,
                    max_depth: Some(1024),
                    ..Default::default()
                },
            )
            .expect("custom child whitespace");
            let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
            let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
            assert_eq!(
                result.content.as_deref(),
                Some(format!("a{}b\n", " ".repeat(depth + 1)).as_str())
            );
            assert_eq!(visitor.lock().expect("visitor calls").calls, 1);
            assert_eq!(byte_reads, 0);
            assert_eq!(node_reads, depth);
            assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get), (0, 0));
            assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
            assert!(node_reads > 0 && node_reads <= 4 * nodes);
            std::println!(
                "unknown-cost custom depth={depth} nodes={nodes} bytes={byte_reads} reads={node_reads} calls={} entries={} capacity={}",
                super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get),
                super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0,
                super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1
            );
            jobs += 1;
        }
        assert_eq!(jobs, 4);
    }
    #[test]
    fn should_classify_visible_void_descendants_below_inline_wrappers() {
        for (body, expected) in [
            ("<em><wbr></em>", false),
            ("<em><input></em>", false),
            ("<b><br></b>", true),
            ("<b><img alt='X' src='x'></b>", true),
            ("<b><hr></b>", true),
        ] {
            let source = format!("<span>{body}</span>");
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse descendants");
            let parser = dom.parser();
            let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
            let root = dom.children()[0].get_inner();
            assert_eq!(context.holds_content(root, parser), expected, "{source}");
        }
    }
    #[test]
    fn should_keep_literal_preformatted_content_when_inline_line_ends_are_empty() {
        for (body, expected) in [
            ("<code>\n</code>", false),
            ("<del>\n</del>", false),
            ("<code> </code>", true),
            ("<pre>\n</pre>", true),
            ("<pre><code>\n\n</code></pre>", true),
            ("<pre><!-- x --></pre>", false),
            ("<pre><i></i></pre>", false),
            ("<pre><span><code>\n</code></span></pre>", true),
            ("<code>&#13;&#10;</code>", true),
            ("<code><span>\n</span></code>", false),
            ("<del><span>\n</span></del>", false),
            ("<code><del><span>&#13;</span></del></code>", true),
            ("<code><del><span>&#13;&#10;</span></del></code>", true),
            ("<del><code><span>&#13;</span></code></del>", true),
            ("<del><span>&#13;</span></del>", true),
            ("<pre><code><span>\n</span></code></pre>", true),
        ] {
            let source = format!("<span>{body}</span>");
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse literal ownership");
            let parser = dom.parser();
            let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
            assert_eq!(
                context.holds_content(dom.children()[0].get_inner(), parser),
                expected,
                "{source}"
            );
            for (index, node) in dom.nodes().iter().enumerate() {
                if matches!(node, tl::Node::Tag(_)) {
                    let id = u32::try_from(index).expect("small node arena");
                    assert_eq!(context.holds_content(id, parser), expected, "rooted {source}");
                }
            }
        }
    }
    #[test]
    fn should_preserve_deep_pre_ownership_without_changing_text_cache() {
        let depth = 2048;
        let source = format!(
            "<span><pre>{}<code>\n</code>{}</pre></span>",
            "<span>".repeat(depth),
            "</span>".repeat(depth)
        );
        let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse deep literal ownership");
        let parser = dom.parser();
        let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
        super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        let root = dom.children()[0].get_inner();
        let pre = dom
            .query_selector("pre")
            .expect("pre selector")
            .next()
            .expect("pre exists")
            .get_inner();
        let code = dom
            .query_selector("code")
            .expect("code selector")
            .next()
            .expect("code exists")
            .get_inner();
        for id in [root, pre, code] {
            assert!(context.holds_content(id, parser), "literal query at {id}");
            assert!(!context.holds_text(id, parser), "text query at {id}");
        }
        let reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
        assert_eq!(reads, 3 * depth + 9);
    }
}
