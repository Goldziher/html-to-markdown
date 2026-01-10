//! Context management for HTML to Markdown conversion.
//!
//! This module contains the core context types and helper functions used during conversion:
//! - `Context`: Tracks conversion state (code mode, list nesting, etc.)
//! - `DomContext`: Caches DOM structure information for efficient tree traversal
//! - `TableScan`: Analyzes table structure before rendering

use lru::LruCache;
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

#[cfg(feature = "inline-images")]
use crate::inline_images::InlineImageCollector;
use crate::text;

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = Rc<RefCell<InlineImageCollector>>;
#[cfg(not(feature = "inline-images"))]
type InlineCollectorHandle = ();

#[cfg(feature = "metadata")]
type ImageMetadataPayload = (std::collections::BTreeMap<String, String>, Option<u32>, Option<u32>);

/// Main context structure tracking conversion state.
///
/// Maintains flags and counters needed throughout the conversion process.
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Context {
    /// Are we inside a code-like element (pre, code, kbd, samp)?
    pub(crate) in_code: bool,
    /// Current list item counter for ordered lists
    pub(crate) list_counter: usize,
    /// Are we in an ordered list (vs unordered)?
    pub(crate) in_ordered_list: bool,
    /// Track if previous sibling in dl was a dt
    pub(crate) last_was_dt: bool,
    /// Blockquote nesting depth
    pub(crate) blockquote_depth: usize,
    /// Are we inside a table cell (td/th)?
    pub(crate) in_table_cell: bool,
    /// Should we convert block elements as inline?
    pub(crate) convert_as_inline: bool,
    /// Depth of inline formatting elements (strong/emphasis/span/etc).
    pub(crate) inline_depth: usize,
    /// Are we inside a list item?
    pub(crate) in_list_item: bool,
    /// List nesting depth (for indentation)
    pub(crate) list_depth: usize,
    /// Unordered list nesting depth (for bullet cycling)
    pub(crate) ul_depth: usize,
    /// Are we inside any list (ul or ol)?
    pub(crate) in_list: bool,
    /// Is this a "loose" list where all items should have blank lines?
    pub(crate) loose_list: bool,
    /// Did a previous list item have block children?
    pub(crate) prev_item_had_blocks: bool,
    /// Are we inside a heading element (h1-h6)?
    pub(crate) in_heading: bool,
    /// Whether inline images should remain markdown inside the current heading.
    pub(crate) heading_allow_inline_images: bool,
    /// Are we inside a paragraph element?
    pub(crate) in_paragraph: bool,
    /// Are we inside a ruby element?
    pub(crate) in_ruby: bool,
    /// Are we inside a `<strong>` / `<b>` element?
    pub(crate) in_strong: bool,
    /// Tag names that should be stripped during conversion.
    pub(crate) strip_tags: Rc<HashSet<String>>,
    /// Tag names that should be preserved as raw HTML.
    pub(crate) preserve_tags: Rc<HashSet<String>>,
    /// Tag names that allow inline images inside headings.
    pub(crate) keep_inline_images_in: Rc<HashSet<String>>,
    #[cfg(feature = "inline-images")]
    /// Shared collector for inline images when enabled.
    pub(crate) inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")]
    /// Shared collector for metadata when enabled.
    pub(crate) metadata_collector: Option<crate::metadata::MetadataCollectorHandle>,
    #[cfg(feature = "metadata")]
    pub(crate) metadata_wants_document: bool,
    #[cfg(feature = "metadata")]
    pub(crate) metadata_wants_headers: bool,
    #[cfg(feature = "metadata")]
    pub(crate) metadata_wants_links: bool,
    #[cfg(feature = "metadata")]
    pub(crate) metadata_wants_images: bool,
    #[cfg(feature = "metadata")]
    pub(crate) metadata_wants_structured_data: bool,
    #[cfg(feature = "visitor")]
    /// Optional visitor for custom HTML traversal callbacks.
    pub(crate) visitor: Option<crate::visitor::VisitorHandle>,
    #[cfg(feature = "visitor")]
    /// Stores the first visitor error encountered during traversal.
    pub(crate) visitor_error: Rc<RefCell<Option<String>>>,
}

/// DOM context caching tree relationships and metadata.
///
/// This structure caches parent/child relationships and tag information to avoid
/// repeated traversals of the DOM tree during conversion.
pub(crate) struct DomContext {
    pub(crate) parent_map: Vec<Option<u32>>,
    pub(crate) children_map: Vec<Option<Vec<tl::NodeHandle>>>,
    pub(crate) sibling_index_map: Vec<Option<usize>>,
    pub(crate) root_children: Vec<tl::NodeHandle>,
    pub(crate) node_map: Vec<Option<tl::NodeHandle>>,
    pub(crate) tag_info_map: Vec<OnceCell<Option<TagInfo>>>,
    pub(crate) prev_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_whitespace_map: Vec<OnceCell<bool>>,
    pub(crate) text_cache: RefCell<LruCache<u32, String>>,
}

const TEXT_CACHE_CAPACITY: usize = 4096;

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
    ) -> Option<Cow<'a, str>> {
        if let Some(info) = self.tag_info(node_handle.get_inner(), parser) {
            return Some(Cow::Borrowed(info.name.as_str()));
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
                        if let Some(info) = self.tag_info(sibling.get_inner(), parser) {
                            let sibling_id = sibling.get_inner();
                            if info.name == "script" || info.name == "style" {
                                return Some(sibling_id);
                            }
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

    fn build_tag_info(&self, id: u32, parser: &tl::Parser) -> Option<TagInfo> {
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

    pub(crate) fn text_content(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> String {
        let id = node_handle.get_inner();
        let cached = {
            let mut cache = self.text_cache.borrow_mut();
            cache.get(&id).cloned()
        };
        if let Some(value) = cached {
            return value;
        }

        let value = self.text_content_uncached(node_handle, parser);
        self.text_cache.borrow_mut().put(id, value.clone());
        value
    }

    pub(crate) fn text_content_uncached(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> String {
        let mut text = String::with_capacity(64);
        if let Some(node) = node_handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    let raw = bytes.as_utf8_str();
                    let decoded = text::decode_html_entities_cow(raw.as_ref());
                    text.push_str(decoded.as_ref());
                }
                tl::Node::Tag(tag) => {
                    let children = tag.children();
                    for child_handle in children.top().iter() {
                        text.push_str(&self.text_content(*child_handle, parser));
                    }
                }
                tl::Node::Comment(_) => {}
            }
        }
        text
    }

    /// Get the parent tag name for a given node ID.
    ///
    /// Returns the tag name of the parent element if it exists and is a tag,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn parent_tag_name(&self, node_id: u32, parser: &tl::Parser) -> Option<String> {
        let parent_id = self.parent_of(node_id)?;
        let parent_handle = self.node_handle(parent_id)?;

        if let Some(info) = self.tag_info(parent_id, parser) {
            return Some(info.name.clone());
        }

        if let Some(tl::Node::Tag(tag)) = parent_handle.get(parser) {
            let name = normalized_tag_name(tag.name().as_utf8_str());
            return Some(name.into_owned());
        }

        None
    }

    /// Get the index of a node among its siblings.
    ///
    /// Returns the 0-based index if the node has siblings,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn get_sibling_index(&self, node_id: u32) -> Option<usize> {
        self.sibling_index(node_id)
    }
}

pub(crate) struct TagInfo {
    pub(crate) name: String,
    pub(crate) is_inline_like: bool,
    pub(crate) is_block: bool,
}

/// Normalizes HTML tag names to lowercase.
///
/// HTML tag names are case-insensitive, so we normalize them to lowercase
/// for consistent comparison and processing.
pub(crate) fn normalized_tag_name(raw: Cow<'_, str>) -> Cow<'_, str> {
    if raw.as_bytes().iter().any(u8::is_ascii_uppercase) {
        let mut owned = raw.into_owned();
        owned.make_ascii_lowercase();
        Cow::Owned(owned)
    } else {
        raw
    }
}

/// Checks if an element is an inline element.
///
/// Inline elements do not start new lines and only take up as much width as necessary.
/// This includes elements like `<a>`, `<span>`, `<strong>`, `<em>`, etc.
pub(crate) fn is_inline_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "cite"
            | "code"
            | "data"
            | "dfn"
            | "em"
            | "i"
            | "kbd"
            | "mark"
            | "q"
            | "rp"
            | "rt"
            | "ruby"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "u"
            | "var"
            | "wbr"
            | "del"
            | "ins"
            | "img"
            | "map"
            | "area"
            | "audio"
            | "video"
            | "picture"
            | "source"
            | "track"
            | "embed"
            | "object"
            | "param"
            | "input"
            | "label"
            | "button"
            | "select"
            | "textarea"
            | "output"
            | "progress"
            | "meter"
    )
}

/// Checks if an element is block-level (not inline).
pub(crate) fn is_block_level_element(tag_name: &str) -> bool {
    is_block_level_name(tag_name, is_inline_element(tag_name))
}

/// Checks if an element is block-level given its inline status.
pub(crate) fn is_block_level_name(tag_name: &str, is_inline: bool) -> bool {
    !is_inline
        && matches!(
            tag_name,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "canvas"
                | "dd"
                | "div"
                | "dl"
                | "dt"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hr"
                | "li"
                | "main"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "tfoot"
                | "ul"
        )
}

/// Table structure analysis for rendering decisions.
///
/// This structure analyzes a table before rendering to determine optimal formatting.
#[derive(Default)]
pub(crate) struct TableScan {
    pub(crate) row_counts: Vec<usize>,
    pub(crate) has_span: bool,
    pub(crate) has_header: bool,
    pub(crate) has_caption: bool,
    pub(crate) has_nested_table: bool,
    pub(crate) link_count: usize,
    pub(crate) has_text: bool,
}

/// Scan a table element to determine its structure.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn scan_table(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> TableScan {
    let mut scan = TableScan::default();
    scan_table_node(node_handle, parser, dom_ctx, true, &mut scan);
    scan
}

/// Recursively scan table nodes to gather structure information.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn scan_table_node(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    is_root: bool,
    scan: &mut TableScan,
) {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => {
                if !scan.has_text {
                    let raw = bytes.as_utf8_str();
                    let decoded = text::decode_html_entities_cow(raw.as_ref());
                    if !decoded.trim().is_empty() {
                        scan.has_text = true;
                    }
                }
            }
            tl::Node::Tag(tag) => {
                let tag_name: Cow<'_, str> = dom_ctx.tag_info(node_handle.get_inner(), parser).map_or_else(
                    || normalized_tag_name(tag.name().as_utf8_str()).into_owned().into(),
                    |info| Cow::Borrowed(info.name.as_str()),
                );

                match tag_name.as_ref() {
                    "a" => scan.link_count += 1,
                    "caption" => scan.has_caption = true,
                    "th" => scan.has_header = true,
                    "img" | "graphic" => {
                        // Images with src or alt attributes count as content
                        if tag.attributes().get("src").is_some() || tag.attributes().get("alt").is_some() {
                            scan.has_text = true;
                        }
                    }
                    "cell" => {
                        // Check if cell has role="head" attribute
                        if let Some(role) = tag.attributes().get("role") {
                            if let Some(role_val) = role {
                                let role_str = role_val.as_utf8_str();
                                if role_str == "head" {
                                    scan.has_header = true;
                                }
                            }
                        }
                    }
                    "table" if !is_root => scan.has_nested_table = true,
                    "tr" | "row" => {
                        let mut cell_count = 0;
                        for child in tag.children().top().iter() {
                            if let Some(tl::Node::Tag(cell_tag)) = child.get(parser) {
                                let cell_name: Cow<'_, str> = dom_ctx
                                    .tag_info(child.get_inner(), parser)
                                    .map(|info| Cow::Borrowed(info.name.as_str()))
                                    .unwrap_or_else(|| {
                                        normalized_tag_name(cell_tag.name().as_utf8_str()).into_owned().into()
                                    });
                                if matches!(cell_name.as_ref(), "td" | "th" | "cell") {
                                    cell_count += 1;
                                    let attrs = cell_tag.attributes();
                                    if attrs.get("colspan").is_some() || attrs.get("rowspan").is_some() {
                                        scan.has_span = true;
                                    }
                                }
                            }
                            scan_table_node(child, parser, dom_ctx, false, scan);
                        }
                        scan.row_counts.push(cell_count);
                        return;
                    }
                    _ => {}
                }

                for child in tag.children().top().iter() {
                    scan_table_node(child, parser, dom_ctx, false, scan);
                }
            }
            _ => {}
        }
    }
}
