//! Main conversion pipeline for HTML to Markdown.
//!
//! This module implements the core conversion functions and the recursive tree walker
//! that transforms HTML DOM nodes into Markdown output.

#![allow(
    clippy::assigning_clones,
    clippy::useless_let_if_seq,
    clippy::ref_option,
    clippy::format_push_string,
    clippy::struct_excessive_bools,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::branches_sharing_code,
    clippy::trivially_copy_pass_by_ref,
    clippy::manual_let_else,
    clippy::collapsible_match,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    clippy::option_if_let_else,
    clippy::needless_pass_by_value,
    clippy::match_same_arms,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::items_after_statements,
    clippy::cast_sign_loss,
    clippy::default_trait_access,
    clippy::unused_self,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::extra_unused_lifetimes,
    clippy::unnecessary_lazy_evaluations
)]

use lru::LruCache;
use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

use std::borrow::Cow;
use std::str;

use crate::converter::block::heading::{find_single_heading_child, heading_allows_inline_images, push_heading};
use crate::converter::block::table::builder::indent_table_for_list;
use crate::converter::context::TagInfo;
use crate::converter::inline::link::append_markdown_link;
use crate::converter::list::utils::{
    add_list_continuation_indent, add_list_leading_separator, add_nested_list_trailing_separator,
    calculate_list_nesting_depth, is_loose_list, process_list_children,
};
use crate::converter::text::{dedent_code_block, normalize_heading_text};
use crate::converter::utility::attributes::{is_hocr_document, may_be_hocr};
use crate::converter::utility::caching::build_dom_context;
#[allow(unused_imports)]
use crate::converter::utility::content::{
    chomp_inline, collect_link_label_text, escape_link_label, get_text_content, is_block_level_element,
    is_block_level_name, is_empty_inline_element, normalize_link_label, normalized_tag_name, truncate_at_char_boundary,
};
use crate::converter::utility::preprocessing::{preprocess_html, strip_script_and_style_tags};
#[allow(unused_imports)]
use crate::converter::utility::serialization::{
    serialize_element, serialize_node, serialize_node_to_html, serialize_tag_to_html,
};

#[cfg(feature = "inline-images")]
use crate::converter::media::{handle_inline_data_image, handle_inline_svg};
use crate::converter::utility::siblings::{
    append_inline_suffix, get_next_sibling_tag, get_previous_sibling_tag, next_sibling_is_inline_tag,
    previous_sibling_is_inline_tag,
};
use crate::error::Result;
#[cfg(feature = "inline-images")]
use crate::inline_images::InlineImageCollector;
use crate::options::{ConversionOptions, ListIndentType};
use crate::text;

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = Rc<RefCell<InlineImageCollector>>;
#[cfg(not(feature = "inline-images"))]
type InlineCollectorHandle = ();

#[cfg(feature = "metadata")]
type ImageMetadataPayload = (BTreeMap<String, String>, Option<u32>, Option<u32>);

/// Conversion context that tracks state during HTML to Markdown conversion.
///
/// This context is passed through the recursive tree walker and maintains information
/// about the current position in the document tree, nesting levels, and enabled features.
#[derive(Clone)]
pub struct Context {
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
    /// Are we inside a link element (collecting link label text)?
    pub(crate) in_link: bool,
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

/// DOM context that provides efficient access to parent/child relationships and text content.
///
/// This context is built once during conversion and provides O(1) access to node relationships
/// via precomputed maps. It also includes an LRU cache for text content extraction.
pub struct DomContext {
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

/// Check if tag names are equal (case-insensitive).
fn tag_name_eq<'a>(a: impl AsRef<str>, b: &str) -> bool {
    a.as_ref().eq_ignore_ascii_case(b)
}

pub(crate) fn trim_trailing_whitespace(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

/// Remove trailing spaces/tabs from every line while preserving newlines.
pub(crate) fn trim_line_end_whitespace(output: &mut String) {
    if output.is_empty() {
        return;
    }

    let mut cleaned = String::with_capacity(output.len());
    for (idx, line) in output.split('\n').enumerate() {
        if idx > 0 {
            cleaned.push('\n');
        }

        let has_soft_break = line.ends_with("  ");
        let trimmed = line.trim_end_matches([' ', '\t']);

        cleaned.push_str(trimmed);
        if has_soft_break {
            cleaned.push_str("  ");
        }
    }

    cleaned.push('\n');
    *output = cleaned;
}

/// Check if an inline ancestor element is allowed to contain block-level elements.
fn inline_ancestor_allows_block(tag_name: &str) -> bool {
    matches!(tag_name, "a" | "ins" | "del")
}

/// Detect block elements that were incorrectly nested under inline ancestors.
///
/// Excludes elements inside `<pre>` or `<code>` blocks, as they have special
/// whitespace preservation rules and should not be repaired.
fn has_inline_block_misnest(dom_ctx: &DomContext, parser: &tl::Parser) -> bool {
    for handle in dom_ctx.node_map.iter().flatten() {
        if let Some(tl::Node::Tag(_tag)) = handle.get(parser) {
            let is_block = dom_ctx
                .tag_info(handle.get_inner(), parser)
                .map(|info| info.is_block)
                .unwrap_or(false);
            if is_block {
                // Check if this block element or any ancestor is pre/code
                let mut check_parent = Some(handle.get_inner());
                let mut inside_preformatted = false;
                while let Some(node_id) = check_parent {
                    if let Some(info) = dom_ctx.tag_info(node_id, parser) {
                        if matches!(info.name.as_str(), "pre" | "code") {
                            inside_preformatted = true;
                            break;
                        }
                    }
                    check_parent = dom_ctx.parent_of(node_id);
                }

                // Skip misnesting check for elements inside pre/code blocks
                if inside_preformatted {
                    continue;
                }

                let mut current = dom_ctx.parent_of(handle.get_inner());
                while let Some(parent_id) = current {
                    if let Some(parent_info) = dom_ctx.tag_info(parent_id, parser) {
                        if is_inline_element(&parent_info.name) && !inline_ancestor_allows_block(&parent_info.name) {
                            return true;
                        }
                    } else if let Some(parent_handle) = dom_ctx.node_handle(parent_id) {
                        if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
                            let parent_name = normalized_tag_name(parent_tag.name().as_utf8_str());
                            if is_inline_element(&parent_name) && !inline_ancestor_allows_block(&parent_name) {
                                return true;
                            }
                        }
                    }
                    current = dom_ctx.parent_of(parent_id);
                }
            }
        }
    }

    false
}

/// Check if HTML contains custom element tags.
fn has_custom_element_tags(html: &str) -> bool {
    // Custom elements must have a hyphen in their name
    html.contains('<') && html.contains('-')
}

/// Try to repair HTML using html5ever parser.
///
/// Returns Some(repaired_html) if repair was successful, None otherwise.
fn repair_with_html5ever(input: &str) -> Option<String> {
    use html5ever::serialize::{SerializeOpts, serialize};
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::{RcDom, SerializableHandle};

    let dom = html5ever::parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut input.as_bytes())
        .ok()?;

    let mut buf = Vec::with_capacity(input.len());
    let handle = SerializableHandle::from(dom.document);
    serialize(&mut buf, &handle, SerializeOpts::default()).ok()?;
    String::from_utf8(buf).ok()
}

/// Format metadata as YAML frontmatter.
fn format_metadata_frontmatter(metadata: &BTreeMap<String, String>) -> String {
    let mut result = String::from("---\n");
    for (key, value) in metadata {
        result.push_str(&format!("{}: {}\n", key, value));
    }
    result.push_str("---\n");
    result
}

/// Determine if a node should be dropped during preprocessing.
fn should_drop_for_preprocessing(
    _node_handle: &tl::NodeHandle,
    _tag_name: &str,
    _tag: &tl::HTMLTag,
    _parser: &tl::Parser,
    _dom_ctx: &DomContext,
    _options: &ConversionOptions,
) -> bool {
    // For now, don't drop any nodes
    false
}

/// Extract metadata from the head element.
fn extract_head_metadata(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    _options: &ConversionOptions,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        // Check if this is a head tag
        if tag.name().as_utf8_str().eq_ignore_ascii_case("head") {
            let children = tag.children();
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    // Look for meta tags
                    if child_tag.name().as_utf8_str().eq_ignore_ascii_case("meta") {
                        if let (Some(name), Some(content)) = (
                            child_tag.attributes().get("name").flatten(),
                            child_tag.attributes().get("content").flatten(),
                        ) {
                            let name_str = name.as_utf8_str();
                            let content_str = content.as_utf8_str();
                            metadata.insert(name_str.to_string(), content_str.to_string());
                        }
                        // Also check for property attribute (Open Graph, etc.)
                        if let (Some(property), Some(content)) = (
                            child_tag.attributes().get("property").flatten(),
                            child_tag.attributes().get("content").flatten(),
                        ) {
                            let property_str = property.as_utf8_str();
                            let content_str = content.as_utf8_str();
                            metadata.insert(property_str.to_string(), content_str.to_string());
                        }
                    }
                    // Look for title tag
                    if child_tag.name().as_utf8_str().eq_ignore_ascii_case("title") {
                        // Extract text content from title tag
                        let mut title_content = String::new();
                        let title_children = child_tag.children();
                        for title_child in title_children.top().iter() {
                            if let Some(tl::Node::Raw(raw)) = title_child.get(parser) {
                                title_content.push_str(raw.as_utf8_str().as_ref());
                            }
                        }
                        title_content = title_content.trim().to_string();
                        if !title_content.is_empty() {
                            metadata.insert("title".to_string(), title_content);
                        }
                    }
                    // Look for link tags with rel attribute (e.g., canonical)
                    if child_tag.name().as_utf8_str().eq_ignore_ascii_case("link") {
                        if let Some(rel_attr) = child_tag.attributes().get("rel").flatten() {
                            let rel_str = rel_attr.as_utf8_str();
                            // Check for canonical link
                            if rel_str.contains("canonical") {
                                if let Some(href_attr) = child_tag.attributes().get("href").flatten() {
                                    let href_str = href_attr.as_utf8_str();
                                    metadata.insert("canonical".to_string(), href_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // If this is not a head tag, recursively search children for head tag
            let children = tag.children();
            for child_handle in children.top().iter() {
                let child_metadata = extract_head_metadata(child_handle, parser, _options);
                if !child_metadata.is_empty() {
                    metadata.extend(child_metadata);
                    break; // Only process first head tag found
                }
            }
        }
    }

    metadata
}

/// Converts HTML to Markdown using the provided conversion options.
///
/// This is the main entry point for HTML to Markdown conversion.
pub fn convert_html(html: &str, options: &ConversionOptions) -> Result<String> {
    convert_html_impl(html, options, None, None, None)
}

/// Converts HTML to Markdown with a custom visitor for callbacks during traversal.
///
/// This variant allows passing a visitor that will receive callbacks for each node
/// during the tree walk, enabling custom processing or analysis.
#[cfg(feature = "visitor")]
pub fn convert_html_with_visitor(
    html: &str,
    options: &ConversionOptions,
    visitor: Option<crate::visitor::VisitorHandle>,
) -> Result<String> {
    convert_html_impl(html, options, None, None, visitor)
}

/// Internal implementation of HTML to Markdown conversion.
///
/// This function handles the actual conversion logic with optional inline image collection,
/// metadata extraction, and visitor callbacks depending on enabled features.
#[cfg_attr(
    any(not(feature = "inline-images"), not(feature = "metadata"), not(feature = "visitor")),
    allow(unused_variables)
)]
#[allow(clippy::too_many_lines)]
pub(crate) fn convert_html_impl(
    html: &str,
    options: &ConversionOptions,
    inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")] metadata_collector: Option<crate::metadata::MetadataCollectorHandle>,
    #[cfg(not(feature = "metadata"))] _metadata_collector: Option<()>,
    #[cfg(feature = "visitor")] visitor: Option<crate::visitor::VisitorHandle>,
    #[cfg(not(feature = "visitor"))] _visitor: Option<()>,
) -> Result<String> {
    // Strip script and style tags completely to prevent parser confusion from HTML-like content
    // inside script/style elements. This preserves JSON-LD for metadata extraction.
    let stripped = strip_script_and_style_tags(html);
    let mut preprocessed = preprocess_html(&stripped).into_owned();
    let mut preprocessed_len = preprocessed.len();

    if has_custom_element_tags(&preprocessed) {
        if let Some(repaired_html) = repair_with_html5ever(&preprocessed) {
            let repaired = preprocess_html(&repaired_html).into_owned();
            preprocessed = repaired;
            preprocessed_len = preprocessed.len();
        }
    }
    let parser_options = tl::ParserOptions::default();
    let mut dom = loop {
        if let Ok(dom) = tl::parse(&preprocessed, parser_options) {
            break dom;
        }
        if let Some(repaired_html) = repair_with_html5ever(&preprocessed) {
            preprocessed = preprocess_html(&repaired_html).into_owned();
            preprocessed_len = preprocessed.len();
            continue;
        }
        return Err(crate::error::ConversionError::ParseError(
            "Failed to parse HTML".to_string(),
        ));
    };
    let mut parser = dom.parser();
    let mut output = String::with_capacity(preprocessed_len.saturating_add(preprocessed_len / 4));

    let mut is_hocr = false;
    if may_be_hocr(preprocessed.as_ref()) {
        for child_handle in dom.children() {
            if is_hocr_document(*child_handle, parser) {
                is_hocr = true;
                break;
            }
        }
    }

    if is_hocr {
        use crate::hocr::{convert_to_markdown_with_options as convert_hocr_to_markdown, extract_hocr_document};

        let (elements, metadata) = extract_hocr_document(&dom);

        if options.extract_metadata && !options.convert_as_inline {
            let mut metadata_map = BTreeMap::new();
            if let Some(system) = metadata.ocr_system {
                metadata_map.insert("ocr-system".to_string(), system);
            }
            if !metadata.ocr_capabilities.is_empty() {
                metadata_map.insert("ocr-capabilities".to_string(), metadata.ocr_capabilities.join(", "));
            }
            if let Some(pages) = metadata.ocr_number_of_pages {
                metadata_map.insert("ocr-number-of-pages".to_string(), pages.to_string());
            }
            if !metadata.ocr_langs.is_empty() {
                metadata_map.insert("ocr-langs".to_string(), metadata.ocr_langs.join(", "));
            }
            if !metadata.ocr_scripts.is_empty() {
                metadata_map.insert("ocr-scripts".to_string(), metadata.ocr_scripts.join(", "));
            }

            if !metadata_map.is_empty() {
                output.push_str(&format_metadata_frontmatter(&metadata_map));
            }
        }

        let mut markdown = convert_hocr_to_markdown(&elements, true, options.hocr_spatial_tables);

        if markdown.trim().is_empty() {
            return Ok(output);
        }

        markdown.truncate(markdown.trim_end().len());
        output.push_str(&markdown);
        output.push('\n');

        return Ok(output);
    }

    let mut dom_ctx = build_dom_context(&dom, parser, preprocessed_len);

    // Check for inline-block misnesting and repair if needed
    if has_inline_block_misnest(&dom_ctx, parser) {
        if let Some(repaired_html) = repair_with_html5ever(&preprocessed) {
            // Drop dom to release borrow on preprocessed
            drop(dom);
            preprocessed = preprocess_html(&repaired_html).into_owned();
            preprocessed_len = preprocessed.len();
            // Re-parse with repaired HTML
            dom = tl::parse(&preprocessed, parser_options)
                .map_err(|_| crate::error::ConversionError::ParseError("Failed to parse repaired HTML".to_string()))?;
            parser = dom.parser();
            dom_ctx = build_dom_context(&dom, parser, preprocessed_len);
            output = String::with_capacity(preprocessed_len.saturating_add(preprocessed_len / 4));
        }
    }

    let wants_frontmatter = options.extract_metadata && !options.convert_as_inline;
    #[cfg(feature = "metadata")]
    let wants_document = metadata_collector
        .as_ref()
        .is_some_and(|collector| collector.borrow().wants_document());
    #[cfg(not(feature = "metadata"))]
    let wants_document = false;

    if wants_frontmatter || wants_document {
        let mut head_metadata: Option<BTreeMap<String, String>> = None;
        #[cfg(feature = "metadata")]
        let mut document_lang: Option<String> = None;
        #[cfg(feature = "metadata")]
        let mut document_dir: Option<String> = None;

        for child_handle in dom.children() {
            if head_metadata.is_none() {
                let metadata = extract_head_metadata(child_handle, parser, options);
                if !metadata.is_empty() {
                    head_metadata = Some(metadata);
                }
            }

            #[cfg(feature = "metadata")]
            if wants_document {
                if let Some(tl::Node::Tag(tag)) = child_handle.get(parser) {
                    let tag_name = tag.name().as_utf8_str();
                    if tag_name == "html" || tag_name == "body" {
                        if document_lang.is_none() {
                            if let Some(Some(lang_bytes)) = tag.attributes().get("lang") {
                                document_lang = Some(lang_bytes.as_utf8_str().to_string());
                            }
                        }
                        if document_dir.is_none() {
                            if let Some(Some(dir_bytes)) = tag.attributes().get("dir") {
                                document_dir = Some(dir_bytes.as_utf8_str().to_string());
                            }
                        }
                    }
                }
            }
        }

        if wants_frontmatter {
            if let Some(metadata) = head_metadata.as_ref() {
                if !metadata.is_empty() {
                    let metadata_frontmatter = format_metadata_frontmatter(metadata);
                    output.push_str(&metadata_frontmatter);
                }
            }
        }

        #[cfg(feature = "metadata")]
        if wants_document {
            if let Some(ref collector) = metadata_collector {
                if let Some(metadata) = head_metadata {
                    if !metadata.is_empty() {
                        collector.borrow_mut().set_head_metadata(metadata);
                    }
                }
                if let Some(lang) = document_lang {
                    collector.borrow_mut().set_language(lang);
                }
                if let Some(dir) = document_dir {
                    collector.borrow_mut().set_text_direction(dir);
                }
            }
        }
    }

    #[cfg(feature = "metadata")]
    let (
        metadata_wants_document,
        metadata_wants_headers,
        metadata_wants_links,
        metadata_wants_images,
        metadata_wants_structured_data,
    ) = if let Some(ref collector) = metadata_collector {
        let guard = collector.borrow();
        (
            guard.wants_document(),
            guard.wants_headers(),
            guard.wants_links(),
            guard.wants_images(),
            guard.wants_structured_data(),
        )
    } else {
        (false, false, false, false, false)
    };

    let ctx = Context {
        in_code: false,
        list_counter: 0,
        in_ordered_list: false,
        last_was_dt: false,
        blockquote_depth: 0,
        in_table_cell: false,
        convert_as_inline: options.convert_as_inline,
        inline_depth: 0,
        in_list_item: false,
        list_depth: 0,
        ul_depth: 0,
        in_list: false,
        loose_list: false,
        prev_item_had_blocks: false,
        in_heading: false,
        heading_allow_inline_images: false,
        in_paragraph: false,
        in_ruby: false,
        in_strong: false,
        in_link: false,
        strip_tags: Rc::new(options.strip_tags.iter().cloned().collect()),
        preserve_tags: Rc::new(options.preserve_tags.iter().cloned().collect()),
        keep_inline_images_in: Rc::new(options.keep_inline_images_in.iter().cloned().collect()),
        #[cfg(feature = "inline-images")]
        inline_collector,
        #[cfg(feature = "metadata")]
        metadata_collector,
        #[cfg(feature = "metadata")]
        metadata_wants_document,
        #[cfg(feature = "metadata")]
        metadata_wants_headers,
        #[cfg(feature = "metadata")]
        metadata_wants_links,
        #[cfg(feature = "metadata")]
        metadata_wants_images,
        #[cfg(feature = "metadata")]
        metadata_wants_structured_data,
        #[cfg(feature = "visitor")]
        visitor: visitor.clone(),
        #[cfg(feature = "visitor")]
        visitor_error: Rc::new(RefCell::new(None)),
    };

    for child_handle in dom.children() {
        walk_node(child_handle, parser, &mut output, options, &ctx, 0, &dom_ctx);
    }

    #[cfg(feature = "visitor")]
    if let Some(err) = ctx.visitor_error.borrow().as_ref() {
        return Err(crate::error::ConversionError::Visitor(err.clone()));
    }

    trim_line_end_whitespace(&mut output);
    let trimmed = output.trim_end_matches('\n');
    if trimmed.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{trimmed}\n"))
    }
}
fn has_more_than_one_char(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some() && chars.next().is_some()
}
/// Check if an element is inline (not block-level).
fn is_inline_element(tag_name: &str) -> bool {
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

/// Recursively walk DOM nodes and convert to Markdown.
#[allow(clippy::only_used_in_recursion)]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn walk_node(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    let Some(node) = node_handle.get(parser) else { return };

    // Log entry to walk_node for body and immediate children
    if options.debug {
        match node {
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                if tag_name == "body" || tag_name == "html" || depth <= 2 {}
            }
            tl::Node::Raw(_) => if depth <= 2 {},
            _ => {}
        }
    }

    match node {
        tl::Node::Raw(bytes) => {
            let raw = bytes.as_utf8_str();
            let mut text = text::decode_html_entities_cow(raw.as_ref());

            if text.is_empty() {
                return;
            }

            let text_ref = text.as_ref();
            let had_newlines = text_ref.contains('\n');
            let has_double_newline = text_ref.contains("\n\n") || text_ref.contains("\r\n\r\n");

            if options.strip_newlines {
                text = Cow::Owned(text.replace(['\r', '\n'], " "));
            }

            if text.trim().is_empty() {
                if ctx.in_code {
                    output.push_str(text.as_ref());
                    return;
                }

                if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                    if ctx.convert_as_inline || ctx.in_table_cell || ctx.in_list_item {
                        output.push_str(text.as_ref());
                        return;
                    }
                    if has_double_newline {
                        if !output.ends_with("\n\n") {
                            output.push('\n');
                        }
                        return;
                    }
                    output.push_str(text.as_ref());
                    return;
                }

                if had_newlines {
                    if output.is_empty() {
                        return;
                    }
                    if !output.ends_with("\n\n") {
                        if let Some(next_tag) = get_next_sibling_tag(node_handle, parser, dom_ctx) {
                            if is_inline_element(next_tag) {
                                return;
                            }
                        }
                    }
                    return;
                }

                if previous_sibling_is_inline_tag(node_handle, parser, dom_ctx)
                    && next_sibling_is_inline_tag(node_handle, parser, dom_ctx)
                {
                    if has_more_than_one_char(text.as_ref()) {
                        if !output.ends_with(' ') {
                            output.push(' ');
                        }
                    } else {
                        output.push_str(text.as_ref());
                    }
                } else {
                    output.push_str(text.as_ref());
                }
                return;
            }

            let processed_text = if ctx.in_code || ctx.in_ruby {
                text.into_owned()
            } else if ctx.in_table_cell {
                let escaped = if options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
                    let normalized_text = text::normalize_whitespace_cow(text.as_ref());
                    text::escape(
                        normalized_text.as_ref(),
                        options.escape_misc,
                        options.escape_asterisks,
                        options.escape_underscores,
                        options.escape_ascii,
                    )
                } else {
                    text::escape(
                        text.as_ref(),
                        options.escape_misc,
                        options.escape_asterisks,
                        options.escape_underscores,
                        options.escape_ascii,
                    )
                };
                if options.escape_misc {
                    escaped
                } else {
                    escaped.replace('|', r"\|")
                }
            } else if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                text::escape(
                    text.as_ref(),
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                    options.escape_ascii,
                )
            } else {
                let has_double_newline = text.contains("\n\n") || text.contains("\r\n\r\n");
                let has_trailing_single_newline =
                    text.ends_with('\n') && !text.ends_with("\n\n") && !text.ends_with("\r\n\r\n");

                let normalized_text = text::normalize_whitespace_cow(text.as_ref());

                let (prefix, suffix, core) = text::chomp(normalized_text.as_ref());

                let skip_prefix = output.ends_with("\n\n")
                    || output.ends_with("* ")
                    || output.ends_with("- ")
                    || output.ends_with(". ")
                    || output.ends_with("] ")
                    || (output.ends_with('\n') && prefix == " ")
                    || (output.ends_with(' ')
                        && prefix == " "
                        && !previous_sibling_is_inline_tag(node_handle, parser, dom_ctx));

                let mut final_text = String::with_capacity(prefix.len() + core.len() + suffix.len() + 2);
                if !skip_prefix && !prefix.is_empty() {
                    final_text.push_str(prefix);
                }

                let escaped_core = text::escape(
                    core,
                    options.escape_misc,
                    options.escape_asterisks,
                    options.escape_underscores,
                    options.escape_ascii,
                );
                final_text.push_str(&escaped_core);

                if !suffix.is_empty() {
                    final_text.push_str(suffix);
                } else if has_trailing_single_newline {
                    let at_paragraph_break = output.ends_with("\n\n");
                    if !at_paragraph_break {
                        if has_double_newline {
                            final_text.push('\n');
                        } else if let Some(next_tag) = get_next_sibling_tag(node_handle, parser, dom_ctx) {
                            if matches!(next_tag, "span") {
                            } else if ctx.inline_depth > 0 || ctx.convert_as_inline || ctx.in_paragraph {
                                final_text.push(' ');
                            } else {
                                final_text.push('\n');
                            }
                        } else if ctx.inline_depth > 0 || ctx.convert_as_inline || ctx.in_paragraph {
                            final_text.push(' ');
                        } else {
                            final_text.push('\n');
                        }
                    }
                }

                final_text
            };

            #[cfg(feature = "visitor")]
            let final_text = if let Some(ref visitor_handle) = ctx.visitor {
                use crate::visitor::{NodeContext, NodeType, VisitResult};
                use std::collections::BTreeMap;

                let node_id = node_handle.get_inner();
                let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                let node_ctx = NodeContext {
                    node_type: NodeType::Text,
                    tag_name: String::new(),
                    attributes: BTreeMap::new(),
                    depth,
                    index_in_parent,
                    parent_tag,
                    is_inline: true,
                };

                let mut visitor = visitor_handle.borrow_mut();
                match visitor.visit_text(&node_ctx, &processed_text) {
                    VisitResult::Continue => processed_text,
                    VisitResult::Custom(custom) => {
                        if ctx.inline_depth > 0 || ctx.in_heading {
                            processed_text
                        } else {
                            custom
                        }
                    }
                    VisitResult::Skip => return,
                    VisitResult::Error(err) => {
                        if ctx.visitor_error.borrow().is_none() {
                            *ctx.visitor_error.borrow_mut() = Some(err);
                        }
                        return;
                    }
                    VisitResult::PreserveHtml => processed_text,
                }
            } else {
                processed_text
            };

            #[cfg(not(feature = "visitor"))]
            let final_text = processed_text;

            if ctx.in_list_item && final_text.contains("\n\n") {
                let indent = " ".repeat(4 * ctx.list_depth);
                let mut first = true;
                for part in final_text.split("\n\n") {
                    if !first {
                        output.push_str("\n\n");
                        output.push_str(&indent);
                    }
                    first = false;
                    output.push_str(part.trim());
                }
            } else {
                output.push_str(&final_text);
            }
        }

        tl::Node::Tag(tag) => {
            let tag_name = match dom_ctx.tag_info(node_handle.get_inner(), parser) {
                Some(info) => Cow::Borrowed(info.name.as_str()),
                None => normalized_tag_name(tag.name().as_utf8_str()),
            };

            #[cfg(feature = "visitor")]
            if let Some(ref visitor_handle) = ctx.visitor {
                use crate::visitor::{NodeContext, NodeType};
                use std::collections::BTreeMap;

                let attributes: BTreeMap<String, String> = tag
                    .attributes()
                    .iter()
                    .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                    .collect();

                let node_id = node_handle.get_inner();
                let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                let node_ctx = NodeContext {
                    node_type: NodeType::Element,
                    tag_name: tag_name.to_string(),
                    attributes,
                    depth,
                    index_in_parent,
                    parent_tag,
                    is_inline: !is_block_level_element(&tag_name),
                };

                let visitor_start_result = {
                    let mut visitor = visitor_handle.borrow_mut();
                    visitor.visit_element_start(&node_ctx)
                };

                match visitor_start_result {
                    crate::visitor::VisitResult::Continue => {}
                    crate::visitor::VisitResult::Skip => return,
                    crate::visitor::VisitResult::Custom(custom_output) => {
                        output.push_str(&custom_output);

                        #[cfg(feature = "visitor")]
                        if let Some(ref visitor_handle) = ctx.visitor {
                            if !matches!(tag_name.as_ref(), "table") {
                                let element_content = &custom_output;
                                let mut visitor = visitor_handle.borrow_mut();
                                let _ = visitor.visit_element_end(&node_ctx, element_content);
                            }
                        }

                        return;
                    }
                    crate::visitor::VisitResult::Error(_msg) => {
                        return;
                    }
                    _ => {}
                }
            }

            if should_drop_for_preprocessing(node_handle, tag_name.as_ref(), tag, parser, dom_ctx, options) {
                trim_trailing_whitespace(output);
                return;
            }

            if ctx.strip_tags.contains(tag_name.as_ref()) {
                let children = tag.children();
                {
                    for child_handle in children.top().iter() {
                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                    }
                }
                return;
            }

            if ctx.preserve_tags.contains(tag_name.as_ref()) {
                let html = serialize_tag_to_html(node_handle, parser);
                output.push_str(&html);
                return;
            }

            #[cfg(feature = "metadata")]
            if matches!(tag_name.as_ref(), "html" | "head" | "body") && ctx.metadata_wants_document {
                if let Some(ref collector) = ctx.metadata_collector {
                    let mut c = collector.borrow_mut();

                    if let Some(lang) = tag.attributes().get("lang").flatten() {
                        c.set_language(lang.as_utf8_str().to_string());
                    }

                    if let Some(dir) = tag.attributes().get("dir").flatten() {
                        c.set_text_direction(dir.as_utf8_str().to_string());
                    }
                }
            }

            #[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
            let element_output_start = output.len();

            match tag_name.as_ref() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = tag_name.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;

                    // Add spacing before heading if needed (similar to paragraph handling)
                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && ctx.blockquote_depth == 0
                        && !output.is_empty()
                        && !output.ends_with("\n\n");

                    if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let mut text = String::new();
                    let heading_ctx = Context {
                        in_heading: true,
                        convert_as_inline: true,
                        heading_allow_inline_images: heading_allows_inline_images(
                            tag_name.as_ref(),
                            &ctx.keep_inline_images_in,
                        ),
                        ..ctx.clone()
                    };
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut text,
                                options,
                                &heading_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let normalized = normalize_heading_text(trimmed);

                        #[cfg(feature = "visitor")]
                        let heading_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let id_attr = tag
                                .attributes()
                                .get("id")
                                .flatten()
                                .map(|v| v.as_utf8_str().to_string());

                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Heading,
                                tag_name: tag_name.to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: false,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_heading(&node_ctx, level as u32, &normalized, id_attr.as_deref()) {
                                VisitResult::Continue => {
                                    let mut buf = String::new();
                                    push_heading(&mut buf, ctx, options, level, normalized.as_ref());
                                    Some(buf)
                                }
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => None,
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                                VisitResult::PreserveHtml => {
                                    let mut buf = String::new();
                                    push_heading(&mut buf, ctx, options, level, normalized.as_ref());
                                    Some(buf)
                                }
                            }
                        } else {
                            let mut buf = String::new();
                            push_heading(&mut buf, ctx, options, level, normalized.as_ref());
                            Some(buf)
                        };

                        #[cfg(not(feature = "visitor"))]
                        let heading_output = {
                            let mut buf = String::new();
                            push_heading(&mut buf, ctx, options, level, normalized.as_ref());
                            Some(buf)
                        };

                        if let Some(heading_text) = heading_output {
                            output.push_str(&heading_text);
                        }

                        #[cfg(feature = "metadata")]
                        if ctx.metadata_wants_headers {
                            if let Some(ref collector) = ctx.metadata_collector {
                                let id = tag
                                    .attributes()
                                    .get("id")
                                    .flatten()
                                    .map(|v| v.as_utf8_str().to_string());
                                collector
                                    .borrow_mut()
                                    .add_header(level as u8, normalized.to_string(), id, depth, 0);
                            }
                        }
                    }
                }

                "p" => {
                    let content_start_pos = output.len();

                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    let after_code_block = output.ends_with("```\n");
                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && ctx.blockquote_depth == 0
                        && !output.is_empty()
                        && !output.ends_with("\n\n")
                        && !after_code_block;

                    if is_table_continuation {
                        trim_trailing_whitespace(output);
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        add_list_continuation_indent(output, ctx.list_depth, true, options);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let p_ctx = Context {
                        in_paragraph: true,
                        ..ctx.clone()
                    };

                    let children = tag.children();
                    {
                        let child_handles: Vec<_> = children.top().iter().collect();

                        for (i, child_handle) in child_handles.iter().enumerate() {
                            if let Some(node) = child_handle.get(parser) {
                                if let tl::Node::Raw(bytes) = node {
                                    let text = bytes.as_utf8_str();
                                    if text.trim().is_empty() && i > 0 && i < child_handles.len() - 1 {
                                        let prev = &child_handles[i - 1];
                                        let next = &child_handles[i + 1];
                                        if is_empty_inline_element(prev, parser, dom_ctx)
                                            && is_empty_inline_element(next, parser, dom_ctx)
                                        {
                                            continue;
                                        }
                                    }
                                }
                            }

                            walk_node(child_handle, parser, output, options, &p_ctx, depth + 1, dom_ctx);
                        }
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
                        output.push_str("\n\n");
                    }
                }

                "strong" | "b" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            let strong_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                in_strong: true,
                                ..ctx.clone()
                            };
                            for child_handle in children.top().iter() {
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut content,
                                    options,
                                    &strong_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
                            }
                        }

                        #[cfg(feature = "visitor")]
                        let strong_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let text_content = get_text_content(node_handle, parser, dom_ctx);
                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Strong,
                                tag_name: tag_name.to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: true,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_strong(&node_ctx, &text_content) {
                                VisitResult::Continue => None,
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => Some(String::new()),
                                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        #[cfg(feature = "visitor")]
                        if let Some(custom_output) = strong_output {
                            output.push_str(&custom_output);
                        } else {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                if ctx.in_strong {
                                    output.push_str(trimmed);
                                } else {
                                    output.push(options.strong_em_symbol);
                                    output.push(options.strong_em_symbol);
                                    output.push_str(trimmed);
                                    output.push(options.strong_em_symbol);
                                    output.push(options.strong_em_symbol);
                                }
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            }
                        }

                        #[cfg(not(feature = "visitor"))]
                        {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                if ctx.in_strong {
                                    output.push_str(trimmed);
                                } else {
                                    output.push(options.strong_em_symbol);
                                    output.push(options.strong_em_symbol);
                                    output.push_str(trimmed);
                                    output.push(options.strong_em_symbol);
                                    output.push(options.strong_em_symbol);
                                }
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            }
                        }
                    }
                }

                "em" | "i" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(64);
                        let children = tag.children();
                        {
                            let em_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                ..ctx.clone()
                            };
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, &em_ctx, depth + 1, dom_ctx);
                            }
                        }

                        #[cfg(feature = "visitor")]
                        let em_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let text_content = get_text_content(node_handle, parser, dom_ctx);
                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Em,
                                tag_name: tag_name.to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: true,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_emphasis(&node_ctx, &text_content) {
                                VisitResult::Continue => None,
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => Some(String::new()),
                                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        #[cfg(feature = "visitor")]
                        if let Some(custom_output) = em_output {
                            output.push_str(&custom_output);
                        } else {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                output.push(options.strong_em_symbol);
                                output.push_str(trimmed);
                                output.push(options.strong_em_symbol);
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            } else if let Some(class_value) = tag
                                .attributes()
                                .get("class")
                                .and_then(|v| v.as_ref().map(|val| val.as_utf8_str().to_string()))
                            {
                                if class_value.contains("caret") && !output.ends_with(' ') {
                                    output.push_str(" > ");
                                }
                            }
                        }

                        #[cfg(not(feature = "visitor"))]
                        {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                output.push(options.strong_em_symbol);
                                output.push_str(trimmed);
                                output.push(options.strong_em_symbol);
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            } else if let Some(class_value) = tag
                                .attributes()
                                .get("class")
                                .and_then(|v| v.as_ref().map(|val| val.as_utf8_str().to_string()))
                            {
                                if class_value.contains("caret") && !output.ends_with(' ') {
                                    output.push_str(" > ");
                                }
                            }
                        }
                    }
                }

                "a" => {
                    const MAX_LINK_LABEL_LEN: usize = 512;

                    let href_attr = tag
                        .attributes()
                        .get("href")
                        .flatten()
                        .map(|v| text::decode_html_entities(&v.as_utf8_str()));
                    let title = tag
                        .attributes()
                        .get("title")
                        .flatten()
                        .map(|v| v.as_utf8_str().to_string());

                    if let Some(href) = href_attr {
                        let raw_text = text::normalize_whitespace(&get_text_content(node_handle, parser, dom_ctx))
                            .trim()
                            .to_string();

                        let is_autolink = options.autolinks
                            && !options.default_title
                            && !href.is_empty()
                            && (raw_text == href || (href.starts_with("mailto:") && raw_text == href[7..]));

                        if is_autolink {
                            output.push('<');
                            if href.starts_with("mailto:") && raw_text == href[7..] {
                                output.push_str(&raw_text);
                            } else {
                                output.push_str(&href);
                            }
                            output.push('>');
                            return;
                        }

                        if let Some((heading_level, heading_handle)) = find_single_heading_child(*node_handle, parser) {
                            if let Some(heading_node) = heading_handle.get(parser) {
                                if let tl::Node::Tag(heading_tag) = heading_node {
                                    let heading_name =
                                        normalized_tag_name(heading_tag.name().as_utf8_str()).into_owned();
                                    let mut heading_text = String::new();
                                    let heading_ctx = Context {
                                        in_heading: true,
                                        convert_as_inline: true,
                                        heading_allow_inline_images: heading_allows_inline_images(
                                            &heading_name,
                                            &ctx.keep_inline_images_in,
                                        ),
                                        ..ctx.clone()
                                    };
                                    walk_node(
                                        &heading_handle,
                                        parser,
                                        &mut heading_text,
                                        options,
                                        &heading_ctx,
                                        depth + 1,
                                        dom_ctx,
                                    );
                                    let trimmed_heading = heading_text.trim();
                                    if !trimmed_heading.is_empty() {
                                        let escaped_label = escape_link_label(trimmed_heading);
                                        let mut link_buffer = String::new();
                                        append_markdown_link(
                                            &mut link_buffer,
                                            &escaped_label,
                                            href.as_str(),
                                            title.as_deref(),
                                            raw_text.as_str(),
                                            options,
                                        );
                                        push_heading(output, ctx, options, heading_level, link_buffer.as_str());
                                        return;
                                    }
                                }
                            }
                        }

                        let children: Vec<_> = tag.children().top().iter().copied().collect();
                        let (inline_label, _block_nodes, saw_block) =
                            collect_link_label_text(&children, parser, dom_ctx);
                        let mut label = if saw_block {
                            let mut content = String::new();
                            let link_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                convert_as_inline: true,
                                ..ctx.clone()
                            };
                            for child_handle in &children {
                                let mut child_buf = String::new();
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut child_buf,
                                    options,
                                    &link_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
                                if !child_buf.trim().is_empty()
                                    && !content.is_empty()
                                    && !content.chars().last().is_none_or(char::is_whitespace)
                                    && !child_buf.chars().next().is_none_or(char::is_whitespace)
                                {
                                    content.push(' ');
                                }
                                content.push_str(&child_buf);
                            }
                            if content.trim().is_empty() {
                                normalize_link_label(&inline_label)
                            } else {
                                normalize_link_label(&content)
                            }
                        } else {
                            let mut content = String::new();
                            let link_ctx = Context {
                                inline_depth: ctx.inline_depth + 1,
                                ..ctx.clone()
                            };
                            for child_handle in &children {
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut content,
                                    options,
                                    &link_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
                            }
                            normalize_link_label(&content)
                        };

                        if label.is_empty() && saw_block {
                            let fallback = text::normalize_whitespace(&get_text_content(node_handle, parser, dom_ctx));
                            label = normalize_link_label(&fallback);
                        }

                        if label.is_empty() && !raw_text.is_empty() {
                            label = normalize_link_label(&raw_text);
                        }

                        if label.is_empty() && !href.is_empty() && !children.is_empty() {
                            label = href.clone();
                        }

                        if label.len() > MAX_LINK_LABEL_LEN {
                            truncate_at_char_boundary(&mut label, MAX_LINK_LABEL_LEN);
                            label.push('…');
                        }

                        let escaped_label = escape_link_label(&label);

                        #[cfg(feature = "visitor")]
                        let link_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Link,
                                tag_name: "a".to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: true,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_link(&node_ctx, &href, &label, title.as_deref()) {
                                VisitResult::Continue => {
                                    let mut buf = String::new();
                                    append_markdown_link(
                                        &mut buf,
                                        &escaped_label,
                                        href.as_str(),
                                        title.as_deref(),
                                        label.as_str(),
                                        options,
                                    );
                                    Some(buf)
                                }
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => None,
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                            }
                        } else {
                            let mut buf = String::new();
                            append_markdown_link(
                                &mut buf,
                                &escaped_label,
                                href.as_str(),
                                title.as_deref(),
                                label.as_str(),
                                options,
                            );
                            Some(buf)
                        };

                        #[cfg(not(feature = "visitor"))]
                        let link_output = {
                            let mut buf = String::new();
                            append_markdown_link(
                                &mut buf,
                                &escaped_label,
                                href.as_str(),
                                title.as_deref(),
                                label.as_str(),
                                options,
                            );
                            Some(buf)
                        };

                        if let Some(link_text) = link_output {
                            output.push_str(&link_text);
                        }

                        #[cfg(feature = "metadata")]
                        if ctx.metadata_wants_links {
                            if let Some(ref collector) = ctx.metadata_collector {
                                let rel_attr = tag
                                    .attributes()
                                    .get("rel")
                                    .flatten()
                                    .map(|v| v.as_utf8_str().to_string());
                                let mut attributes_map = BTreeMap::new();
                                for (key, value_opt) in tag.attributes().iter() {
                                    let key_str = key.to_string();
                                    if key_str == "href" {
                                        continue;
                                    }

                                    let value = value_opt.map(|v| v.to_string()).unwrap_or_default();
                                    attributes_map.insert(key_str, value);
                                }
                                collector.borrow_mut().add_link(
                                    href.clone(),
                                    label.clone(),
                                    title.clone(),
                                    rel_attr,
                                    attributes_map,
                                );
                            }
                        }
                    } else {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    }
                }

                "img" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

                    let alt = tag
                        .attributes()
                        .get("alt")
                        .flatten()
                        .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

                    let title = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str());
                    #[cfg(feature = "metadata")]
                    let mut metadata_payload: Option<ImageMetadataPayload> = None;
                    #[cfg(feature = "metadata")]
                    if ctx.metadata_wants_images {
                        let mut attributes_map = BTreeMap::new();
                        let mut width: Option<u32> = None;
                        let mut height: Option<u32> = None;
                        for (key, value_opt) in tag.attributes().iter() {
                            let key_str = key.to_string();
                            if key_str == "src" {
                                continue;
                            }
                            let value = value_opt.map(|v| v.to_string()).unwrap_or_default();
                            if key_str == "width" {
                                if let Ok(parsed) = value.parse::<u32>() {
                                    width = Some(parsed);
                                }
                            } else if key_str == "height" {
                                if let Ok(parsed) = value.parse::<u32>() {
                                    height = Some(parsed);
                                }
                            }
                            attributes_map.insert(key_str, value);
                        }
                        metadata_payload = Some((attributes_map, width, height));
                    }

                    #[cfg(feature = "inline-images")]
                    if let Some(ref collector_ref) = ctx.inline_collector {
                        if src.trim_start().starts_with("data:") {
                            let mut attributes_map = BTreeMap::new();
                            for (key, value_opt) in tag.attributes().iter() {
                                let key_str = key.to_string();
                                let keep = key_str == "width"
                                    || key_str == "height"
                                    || key_str == "filename"
                                    || key_str == "aria-label"
                                    || key_str.starts_with("data-");
                                if keep {
                                    let value = value_opt.map(|value| value.to_string()).unwrap_or_default();
                                    attributes_map.insert(key_str, value);
                                }
                            }
                            handle_inline_data_image(
                                collector_ref,
                                src.as_ref(),
                                alt.as_ref(),
                                title.as_deref(),
                                attributes_map,
                            );
                        }
                    }

                    let keep_as_markdown = ctx.in_heading && ctx.heading_allow_inline_images;

                    let should_use_alt_text = !keep_as_markdown
                        && (ctx.convert_as_inline || (ctx.in_heading && !ctx.heading_allow_inline_images));

                    #[cfg(feature = "visitor")]
                    let image_output = if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Image,
                            tag_name: "img".to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: true,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_image(&node_ctx, &src, &alt, title.as_deref()) {
                            VisitResult::Continue => {
                                let mut buf = String::new();
                                if should_use_alt_text {
                                    buf.push_str(&alt);
                                } else {
                                    buf.push_str("![");
                                    buf.push_str(&alt);
                                    buf.push_str("](");
                                    buf.push_str(&src);
                                    if let Some(ref title_text) = title {
                                        buf.push_str(" \"");
                                        buf.push_str(title_text);
                                        buf.push('"');
                                    }
                                    buf.push(')');
                                }
                                Some(buf)
                            }
                            VisitResult::Custom(custom) => Some(custom),
                            VisitResult::Skip => None,
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                None
                            }
                            VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                        }
                    } else {
                        let mut buf = String::new();
                        if should_use_alt_text {
                            buf.push_str(&alt);
                        } else {
                            buf.push_str("![");
                            buf.push_str(&alt);
                            buf.push_str("](");
                            buf.push_str(&src);
                            if let Some(ref title_text) = title {
                                buf.push_str(" \"");
                                buf.push_str(title_text);
                                buf.push('"');
                            }
                            buf.push(')');
                        }
                        Some(buf)
                    };

                    #[cfg(not(feature = "visitor"))]
                    let image_output = {
                        let mut buf = String::new();
                        if should_use_alt_text {
                            buf.push_str(&alt);
                        } else {
                            buf.push_str("![");
                            buf.push_str(&alt);
                            buf.push_str("](");
                            buf.push_str(&src);
                            if let Some(ref title_text) = title {
                                buf.push_str(" \"");
                                buf.push_str(title_text);
                                buf.push('"');
                            }
                            buf.push(')');
                        }
                        Some(buf)
                    };

                    if let Some(img_text) = image_output {
                        output.push_str(&img_text);
                    }

                    #[cfg(feature = "metadata")]
                    if ctx.metadata_wants_images {
                        if let Some(ref collector) = ctx.metadata_collector {
                            if let Some((attributes_map, width, height)) = metadata_payload {
                                if !src.is_empty() {
                                    let dimensions = match (width, height) {
                                        (Some(w), Some(h)) => Some((w, h)),
                                        _ => None,
                                    };
                                    collector.borrow_mut().add_image(
                                        src.to_string(),
                                        if alt.is_empty() { None } else { Some(alt.to_string()) },
                                        title.as_deref().map(std::string::ToString::to_string),
                                        dimensions,
                                        attributes_map,
                                    );
                                }
                            }
                        }
                    }
                }

                "graphic" => {
                    use std::borrow::Cow;

                    // Check source attributes in order: url, href, xlink:href, src
                    let src = tag
                        .attributes()
                        .get("url")
                        .flatten()
                        .or_else(|| tag.attributes().get("href").flatten())
                        .or_else(|| tag.attributes().get("xlink:href").flatten())
                        .or_else(|| tag.attributes().get("src").flatten())
                        .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

                    // Use "alt" attribute, fallback to "filename"
                    let alt = tag
                        .attributes()
                        .get("alt")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .or_else(|| tag.attributes().get("filename").flatten().map(|v| v.as_utf8_str()))
                        .unwrap_or(Cow::Borrowed(""));

                    let title = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str());
                    #[cfg(feature = "metadata")]
                    let mut metadata_payload: Option<ImageMetadataPayload> = None;
                    #[cfg(feature = "metadata")]
                    if ctx.metadata_wants_images {
                        let mut attributes_map = BTreeMap::new();
                        let mut width: Option<u32> = None;
                        let mut height: Option<u32> = None;
                        for (key, value_opt) in tag.attributes().iter() {
                            let key_str = key.to_string();
                            if key_str == "url" || key_str == "href" || key_str == "xlink:href" || key_str == "src" {
                                continue;
                            }
                            let value = value_opt.map(|v| v.to_string()).unwrap_or_default();
                            if key_str == "width" {
                                if let Ok(parsed) = value.parse::<u32>() {
                                    width = Some(parsed);
                                }
                            } else if key_str == "height" {
                                if let Ok(parsed) = value.parse::<u32>() {
                                    height = Some(parsed);
                                }
                            }
                            attributes_map.insert(key_str, value);
                        }
                        metadata_payload = Some((attributes_map, width, height));
                    }

                    let keep_as_markdown = ctx.in_heading && ctx.heading_allow_inline_images;

                    let should_use_alt_text = !keep_as_markdown
                        && (ctx.convert_as_inline || (ctx.in_heading && !ctx.heading_allow_inline_images));

                    #[cfg(feature = "visitor")]
                    let graphic_output = if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Image,
                            tag_name: "graphic".to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: true,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_image(&node_ctx, &src, &alt, title.as_deref()) {
                            VisitResult::Continue => {
                                let mut buf = String::new();
                                if should_use_alt_text {
                                    buf.push_str(&alt);
                                } else {
                                    buf.push_str("![");
                                    buf.push_str(&alt);
                                    buf.push_str("](");
                                    buf.push_str(&src);
                                    if let Some(ref title_text) = title {
                                        buf.push_str(" \"");
                                        buf.push_str(title_text);
                                        buf.push('"');
                                    }
                                    buf.push(')');
                                }
                                Some(buf)
                            }
                            VisitResult::Custom(custom) => Some(custom),
                            VisitResult::Skip => None,
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                None
                            }
                            VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                        }
                    } else {
                        let mut buf = String::new();
                        if should_use_alt_text {
                            buf.push_str(&alt);
                        } else {
                            buf.push_str("![");
                            buf.push_str(&alt);
                            buf.push_str("](");
                            buf.push_str(&src);
                            if let Some(ref title_text) = title {
                                buf.push_str(" \"");
                                buf.push_str(title_text);
                                buf.push('"');
                            }
                            buf.push(')');
                        }
                        Some(buf)
                    };

                    #[cfg(not(feature = "visitor"))]
                    let graphic_output = {
                        let mut buf = String::new();
                        if should_use_alt_text {
                            buf.push_str(&alt);
                        } else {
                            buf.push_str("![");
                            buf.push_str(&alt);
                            buf.push_str("](");
                            buf.push_str(&src);
                            if let Some(ref title_text) = title {
                                buf.push_str(" \"");
                                buf.push_str(title_text);
                                buf.push('"');
                            }
                            buf.push(')');
                        }
                        Some(buf)
                    };

                    if let Some(graphic_text) = graphic_output {
                        output.push_str(&graphic_text);
                    }

                    #[cfg(feature = "metadata")]
                    if ctx.metadata_wants_images {
                        if let Some(ref collector) = ctx.metadata_collector {
                            if let Some((attributes_map, width, height)) = metadata_payload {
                                if !src.is_empty() {
                                    let dimensions = match (width, height) {
                                        (Some(w), Some(h)) => Some((w, h)),
                                        _ => None,
                                    };
                                    collector.borrow_mut().add_image(
                                        src.to_string(),
                                        if alt.is_empty() { None } else { Some(alt.to_string()) },
                                        title.as_deref().map(std::string::ToString::to_string),
                                        dimensions,
                                        attributes_map,
                                    );
                                }
                            }
                        }
                    }
                }

                "mark" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        use crate::options::HighlightStyle;
                        match options.highlight_style {
                            HighlightStyle::DoubleEqual => {
                                output.push_str("==");
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str("==");
                            }
                            HighlightStyle::Html => {
                                output.push_str("<mark>");
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str("</mark>");
                            }
                            HighlightStyle::Bold => {
                                let symbol = options.strong_em_symbol.to_string().repeat(2);
                                output.push_str(&symbol);
                                let bold_ctx = Context {
                                    in_strong: true,
                                    ..ctx.clone()
                                };
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, &bold_ctx, depth + 1, dom_ctx);
                                    }
                                }
                                output.push_str(&symbol);
                            }
                            HighlightStyle::None => {
                                let children = tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                    }
                                }
                            }
                        }
                    }
                }

                "del" | "s" => {
                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(32);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                            }
                        }

                        #[cfg(feature = "visitor")]
                        let strikethrough_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let text_content = get_text_content(node_handle, parser, dom_ctx);
                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Strikethrough,
                                tag_name: tag_name.to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: true,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_strikethrough(&node_ctx, &text_content) {
                                VisitResult::Continue => None,
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => Some(String::new()),
                                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        #[cfg(feature = "visitor")]
                        if let Some(custom_output) = strikethrough_output {
                            output.push_str(&custom_output);
                        } else {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                output.push_str("~~");
                                output.push_str(trimmed);
                                output.push_str("~~");
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            }
                        }

                        #[cfg(not(feature = "visitor"))]
                        {
                            let (prefix, suffix, trimmed) = chomp_inline(&content);
                            if !content.trim().is_empty() {
                                output.push_str(prefix);
                                output.push_str("~~");
                                output.push_str(trimmed);
                                output.push_str("~~");
                                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                            } else if !content.is_empty() {
                                output.push_str(prefix);
                                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                            }
                        }
                    }
                }

                "ins" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    #[cfg(feature = "visitor")]
                    let underline_output = if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let text_content = get_text_content(node_handle, parser, dom_ctx);
                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Underline,
                            tag_name: tag_name.to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: true,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_underline(&node_ctx, &text_content) {
                            VisitResult::Continue => None,
                            VisitResult::Custom(custom) => Some(custom),
                            VisitResult::Skip => Some(String::new()),
                            VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                None
                            }
                        }
                    } else {
                        None
                    };

                    #[cfg(feature = "visitor")]
                    if let Some(custom_output) = underline_output {
                        output.push_str(&custom_output);
                    } else {
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !trimmed.is_empty() {
                            output.push_str(prefix);
                            output.push_str("==");
                            output.push_str(trimmed);
                            output.push_str("==");
                            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                        }
                    }

                    #[cfg(not(feature = "visitor"))]
                    {
                        let (prefix, suffix, trimmed) = chomp_inline(&content);
                        if !trimmed.is_empty() {
                            output.push_str(prefix);
                            output.push_str("==");
                            output.push_str(trimmed);
                            output.push_str("==");
                            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                        }
                    }
                }

                "u" => {
                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let text_content = get_text_content(node_handle, parser, dom_ctx);
                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Underline,
                            tag_name: tag_name.to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: true,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_underline(&node_ctx, &text_content) {
                            VisitResult::Continue => {
                                let children = tag.children();
                                for child_handle in children.top().iter() {
                                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                }
                            }
                            VisitResult::Custom(custom) => {
                                output.push_str(&custom);
                            }
                            VisitResult::Skip => {}
                            VisitResult::PreserveHtml => {
                                output.push_str(&serialize_node(node_handle, parser));
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                let children = tag.children();
                                for child_handle in children.top().iter() {
                                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                                }
                            }
                        }
                    } else {
                        let children = tag.children();
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    #[cfg(not(feature = "visitor"))]
                    {
                        let children = tag.children();
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "small" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "sub" => {
                    if !ctx.in_code && !options.sub_symbol.is_empty() {
                        output.push_str(&options.sub_symbol);
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    if !ctx.in_code && !options.sub_symbol.is_empty() {
                        if options.sub_symbol.starts_with('<') && !options.sub_symbol.starts_with("</") {
                            output.push_str(&options.sub_symbol.replace('<', "</"));
                        } else {
                            output.push_str(&options.sub_symbol);
                        }
                    }
                }

                "sup" => {
                    if !ctx.in_code && !options.sup_symbol.is_empty() {
                        output.push_str(&options.sup_symbol);
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    if !ctx.in_code && !options.sup_symbol.is_empty() {
                        if options.sup_symbol.starts_with('<') && !options.sup_symbol.starts_with("</") {
                            output.push_str(&options.sup_symbol.replace('<', "</"));
                        } else {
                            output.push_str(&options.sup_symbol);
                        }
                    }
                }

                "kbd" | "samp" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &code_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                    let normalized = text::normalize_whitespace(&content);
                    let (prefix, suffix, trimmed) = chomp_inline(&normalized);
                    if !content.trim().is_empty() {
                        output.push_str(prefix);
                        output.push('`');
                        output.push_str(trimmed);
                        output.push('`');
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    } else if !content.is_empty() {
                        output.push_str(prefix);
                        append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
                    }
                }

                "var" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    }
                }

                "dfn" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let (prefix, suffix, trimmed) = chomp_inline(&content);
                    if !trimmed.is_empty() {
                        output.push_str(prefix);
                        output.push(options.strong_em_symbol);
                        output.push_str(trimmed);
                        output.push(options.strong_em_symbol);
                        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
                    }
                }

                "abbr" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();

                    if !trimmed.is_empty() {
                        output.push_str(trimmed);

                        if let Some(title) = tag.attributes().get("title").flatten().map(|v| v.as_utf8_str()) {
                            let trimmed_title = title.trim();
                            if !trimmed_title.is_empty() {
                                output.push_str(" (");
                                output.push_str(trimmed_title);
                                output.push(')');
                            }
                        }
                    }
                }

                "time" | "data" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "wbr" => {}

                "code" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    if ctx.in_code {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, &code_ctx, depth + 1, dom_ctx);
                            }
                        }
                    } else {
                        let mut content = String::with_capacity(32);
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(
                                    child_handle,
                                    parser,
                                    &mut content,
                                    options,
                                    &code_ctx,
                                    depth + 1,
                                    dom_ctx,
                                );
                            }
                        }

                        let trimmed = &content;

                        if !content.trim().is_empty() {
                            #[cfg(feature = "visitor")]
                            let code_output = if let Some(ref visitor_handle) = ctx.visitor {
                                use crate::visitor::{NodeContext, NodeType, VisitResult};
                                use std::collections::BTreeMap;

                                let attributes: BTreeMap<String, String> = tag
                                    .attributes()
                                    .iter()
                                    .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                    .collect();

                                let node_id = node_handle.get_inner();
                                let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                                let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                                let node_ctx = NodeContext {
                                    node_type: NodeType::Code,
                                    tag_name: tag_name.to_string(),
                                    attributes,
                                    depth,
                                    index_in_parent,
                                    parent_tag,
                                    is_inline: true,
                                };

                                let mut visitor = visitor_handle.borrow_mut();
                                match visitor.visit_code_inline(&node_ctx, trimmed) {
                                    VisitResult::Continue => None,
                                    VisitResult::Custom(custom) => Some(custom),
                                    VisitResult::Skip => Some(String::new()),
                                    VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                                    VisitResult::Error(err) => {
                                        if ctx.visitor_error.borrow().is_none() {
                                            *ctx.visitor_error.borrow_mut() = Some(err);
                                        }
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            #[cfg(feature = "visitor")]
                            if let Some(custom_output) = code_output {
                                output.push_str(&custom_output);
                            } else {
                                let contains_backtick = trimmed.contains('`');

                                let needs_delimiter_spaces = {
                                    let first_char = trimmed.chars().next();
                                    let last_char = trimmed.chars().last();
                                    let starts_with_space = first_char == Some(' ');
                                    let ends_with_space = last_char == Some(' ');
                                    let starts_with_backtick = first_char == Some('`');
                                    let ends_with_backtick = last_char == Some('`');
                                    let all_spaces = trimmed.chars().all(|c| c == ' ');

                                    all_spaces
                                        || starts_with_backtick
                                        || ends_with_backtick
                                        || (starts_with_space && ends_with_space && contains_backtick)
                                };

                                let (num_backticks, needs_spaces) = if contains_backtick {
                                    let max_consecutive = trimmed
                                        .chars()
                                        .fold((0, 0), |(max, current), c| {
                                            if c == '`' {
                                                let new_current = current + 1;
                                                (max.max(new_current), new_current)
                                            } else {
                                                (max, 0)
                                            }
                                        })
                                        .0;
                                    let num = if max_consecutive == 1 { 2 } else { 1 };
                                    (num, needs_delimiter_spaces)
                                } else {
                                    (1, needs_delimiter_spaces)
                                };

                                for _ in 0..num_backticks {
                                    output.push('`');
                                }
                                if needs_spaces {
                                    output.push(' ');
                                }
                                output.push_str(trimmed);
                                if needs_spaces {
                                    output.push(' ');
                                }
                                for _ in 0..num_backticks {
                                    output.push('`');
                                }
                            }

                            #[cfg(not(feature = "visitor"))]
                            {
                                let contains_backtick = trimmed.contains('`');

                                let needs_delimiter_spaces = {
                                    let first_char = trimmed.chars().next();
                                    let last_char = trimmed.chars().last();
                                    let starts_with_space = first_char == Some(' ');
                                    let ends_with_space = last_char == Some(' ');
                                    let starts_with_backtick = first_char == Some('`');
                                    let ends_with_backtick = last_char == Some('`');
                                    let all_spaces = trimmed.chars().all(|c| c == ' ');

                                    all_spaces
                                        || starts_with_backtick
                                        || ends_with_backtick
                                        || (starts_with_space && ends_with_space && contains_backtick)
                                };

                                let (num_backticks, needs_spaces) = if contains_backtick {
                                    let max_consecutive = trimmed
                                        .chars()
                                        .fold((0, 0), |(max, current), c| {
                                            if c == '`' {
                                                let new_current = current + 1;
                                                (max.max(new_current), new_current)
                                            } else {
                                                (max, 0)
                                            }
                                        })
                                        .0;
                                    let num = if max_consecutive == 1 { 2 } else { 1 };
                                    (num, needs_delimiter_spaces)
                                } else {
                                    (1, needs_delimiter_spaces)
                                };

                                for _ in 0..num_backticks {
                                    output.push('`');
                                }
                                if needs_spaces {
                                    output.push(' ');
                                }
                                output.push_str(trimmed);
                                if needs_spaces {
                                    output.push(' ');
                                }
                                for _ in 0..num_backticks {
                                    output.push('`');
                                }
                            }
                        }
                    }
                }

                "pre" => {
                    let code_ctx = Context {
                        in_code: true,
                        ..ctx.clone()
                    };

                    #[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
                    let language: Option<String> = {
                        let mut lang: Option<String> = None;

                        // First, try to extract language from <pre> tag's class attribute
                        if let Some(class_attr) = tag.attributes().get("class") {
                            if let Some(class_bytes) = class_attr {
                                let class_str = class_bytes.as_utf8_str();
                                for cls in class_str.split_whitespace() {
                                    if let Some(stripped) = cls.strip_prefix("language-") {
                                        lang = Some(String::from(stripped));
                                        break;
                                    } else if let Some(stripped) = cls.strip_prefix("lang-") {
                                        lang = Some(String::from(stripped));
                                        break;
                                    }
                                }
                            }
                        }

                        // If not found on <pre>, try to extract from nested <code> tag's class attribute
                        if lang.is_none() {
                            let children = tag.children();
                            for child_handle in children.top().iter() {
                                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                    if child_tag.name() == "code" {
                                        if let Some(class_attr) = child_tag.attributes().get("class") {
                                            if let Some(class_bytes) = class_attr {
                                                let class_str = class_bytes.as_utf8_str();
                                                for cls in class_str.split_whitespace() {
                                                    if let Some(stripped) = cls.strip_prefix("language-") {
                                                        lang = Some(String::from(stripped));
                                                        break;
                                                    } else if let Some(stripped) = cls.strip_prefix("lang-") {
                                                        lang = Some(String::from(stripped));
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        lang
                    };

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &code_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }

                    if !content.is_empty() {
                        let leading_newlines = content.chars().take_while(|&c| c == '\n').count();
                        let trailing_newlines = content.chars().rev().take_while(|&c| c == '\n').count();
                        let core = content.trim_matches('\n');
                        let is_whitespace_only = core.trim().is_empty();

                        let processed_content = if options.whitespace_mode == crate::options::WhitespaceMode::Strict {
                            content
                        } else {
                            // Always dedent code blocks to remove common leading whitespace
                            let mut core_text = dedent_code_block(core);

                            if is_whitespace_only {
                                let mut rebuilt = String::new();
                                for _ in 0..leading_newlines {
                                    rebuilt.push('\n');
                                }
                                rebuilt.push_str(&core_text);
                                for _ in 0..trailing_newlines {
                                    rebuilt.push('\n');
                                }
                                rebuilt
                            } else {
                                for _ in 0..trailing_newlines {
                                    core_text.push('\n');
                                }
                                core_text
                            }
                        };

                        #[cfg(feature = "visitor")]
                        let code_block_output = if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let node_id = node_handle.get_inner();
                            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::Pre,
                                tag_name: tag_name.to_string(),
                                attributes,
                                depth,
                                index_in_parent,
                                parent_tag,
                                is_inline: false,
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_code_block(&node_ctx, language.as_deref(), &processed_content) {
                                VisitResult::Continue => None,
                                VisitResult::Custom(custom) => Some(custom),
                                VisitResult::Skip => Some(String::new()),
                                VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        #[cfg(feature = "visitor")]
                        if let Some(custom_output) = code_block_output {
                            output.push_str(&custom_output);
                        } else {
                            match options.code_block_style {
                                crate::options::CodeBlockStyle::Indented => {
                                    if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                        if output.ends_with('\n') {
                                            output.push('\n');
                                        } else {
                                            output.push_str("\n\n");
                                        }
                                    }

                                    let indented = processed_content
                                        .lines()
                                        .map(|line| {
                                            if line.is_empty() {
                                                String::new()
                                            } else {
                                                format!("    {line}")
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    output.push_str(&indented);

                                    output.push_str("\n\n");
                                }
                                crate::options::CodeBlockStyle::Backticks | crate::options::CodeBlockStyle::Tildes => {
                                    if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                        if output.ends_with('\n') {
                                            output.push('\n');
                                        } else {
                                            output.push_str("\n\n");
                                        }
                                    }

                                    let fence = if options.code_block_style == crate::options::CodeBlockStyle::Backticks
                                    {
                                        "```"
                                    } else {
                                        "~~~"
                                    };

                                    output.push_str(fence);
                                    if !options.code_language.is_empty() {
                                        output.push_str(&options.code_language);
                                    }
                                    output.push('\n');
                                    output.push_str(&processed_content);
                                    output.push('\n');
                                    output.push_str(fence);
                                    output.push('\n');
                                }
                            }
                        }

                        #[cfg(not(feature = "visitor"))]
                        {
                            match options.code_block_style {
                                crate::options::CodeBlockStyle::Indented => {
                                    if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                        if output.ends_with('\n') {
                                            output.push('\n');
                                        } else {
                                            output.push_str("\n\n");
                                        }
                                    }

                                    let indented = processed_content
                                        .lines()
                                        .map(|line| {
                                            if line.is_empty() {
                                                String::new()
                                            } else {
                                                format!("    {line}")
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    output.push_str(&indented);

                                    output.push_str("\n\n");
                                }
                                crate::options::CodeBlockStyle::Backticks | crate::options::CodeBlockStyle::Tildes => {
                                    if !ctx.convert_as_inline && !output.is_empty() && !output.ends_with("\n\n") {
                                        if output.ends_with('\n') {
                                            output.push('\n');
                                        } else {
                                            output.push_str("\n\n");
                                        }
                                    }

                                    let fence = if options.code_block_style == crate::options::CodeBlockStyle::Backticks
                                    {
                                        "```"
                                    } else {
                                        "~~~"
                                    };

                                    output.push_str(fence);
                                    if !options.code_language.is_empty() {
                                        output.push_str(&options.code_language);
                                    }
                                    output.push('\n');
                                    output.push_str(&processed_content);
                                    output.push('\n');
                                    output.push_str(fence);
                                    output.push('\n');
                                }
                            }
                        }
                    }
                }

                "blockquote" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                        return;
                    }

                    let cite = tag
                        .attributes()
                        .get("cite")
                        .flatten()
                        .map(|v| v.as_utf8_str().to_string());

                    let blockquote_ctx = Context {
                        blockquote_depth: ctx.blockquote_depth + 1,
                        ..ctx.clone()
                    };
                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &blockquote_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }

                    let trimmed_content = content.trim();

                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let node_id = node_handle.get_inner();
                        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::Blockquote,
                            tag_name: "blockquote".to_string(),
                            attributes,
                            depth,
                            index_in_parent,
                            parent_tag,
                            is_inline: false,
                        };

                        let mut visitor_ref = visitor.borrow_mut();
                        match visitor_ref.visit_blockquote(&node_ctx, trimmed_content, ctx.blockquote_depth) {
                            VisitResult::Continue => {}
                            VisitResult::Custom(custom) => {
                                output.push_str(&custom);
                                return;
                            }
                            VisitResult::Skip => return,
                            VisitResult::PreserveHtml => {
                                let mut html_output = String::new();
                                serialize_node_to_html(node_handle, parser, &mut html_output);
                                output.push_str(&html_output);
                                return;
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                return;
                            }
                        }
                    }
                    if !trimmed_content.is_empty() {
                        if ctx.blockquote_depth > 0 {
                            output.push_str("\n\n\n");
                        } else if !output.is_empty() {
                            if output.ends_with("\n\n") {
                                // Paragraph already added \n\n; blockquote needs just \n
                                output.truncate(output.len() - 1);
                            } else if !output.ends_with('\n') {
                                output.push_str("\n\n");
                            } else if !output.ends_with("\n\n") {
                                output.push('\n');
                            }
                        }

                        let prefix = "> ";

                        for line in trimmed_content.lines() {
                            output.push_str(prefix);
                            output.push_str(line.trim());
                            output.push('\n');
                        }

                        if let Some(url) = cite {
                            output.push('\n');
                            output.push_str("— <");
                            output.push_str(&url);
                            output.push_str(">\n\n");
                        }

                        // Add trailing newlines only when appropriate for proper spacing
                        // (matching paragraph conditional logic for CommonMark compliance)
                        if !ctx.convert_as_inline && !ctx.in_table_cell && !ctx.in_list_item {
                            while output.ends_with('\n') {
                                output.truncate(output.len() - 1);
                            }
                            output.push_str("\n\n");
                        }
                    }
                }

                "br" => {
                    if ctx.in_heading {
                        trim_trailing_whitespace(output);
                        output.push_str("  ");
                    } else {
                        use crate::options::NewlineStyle;
                        if output.is_empty() || output.ends_with('\n') {
                            output.push('\n');
                        } else {
                            match options.newline_style {
                                NewlineStyle::Spaces => output.push_str("  \n"),
                                NewlineStyle::Backslash => output.push_str("\\\n"),
                            }
                        }
                    }
                }

                "hr" => {
                    if !output.is_empty() {
                        let prev_tag = get_previous_sibling_tag(node_handle, parser, dom_ctx);
                        let last_line_is_blockquote = output
                            .rsplit('\n')
                            .find(|line| !line.trim().is_empty())
                            .is_some_and(|line| line.trim_start().starts_with('>'));
                        let needs_blank_line =
                            !ctx.in_paragraph && !matches!(prev_tag, Some("blockquote")) && !last_line_is_blockquote;

                        // If previous element was a blockquote, it added \n\n; reduce to \n
                        if matches!(prev_tag, Some("blockquote")) && output.ends_with("\n\n") {
                            output.truncate(output.len() - 1);
                        } else if ctx.in_paragraph || !needs_blank_line {
                            if !output.ends_with('\n') {
                                output.push('\n');
                            }
                        } else {
                            trim_trailing_whitespace(output);
                            if output.ends_with('\n') {
                                if !output.ends_with("\n\n") {
                                    output.push('\n');
                                }
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    }
                    output.push_str("---\n");
                }

                "ul" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

                    #[cfg(feature = "visitor")]
                    let list_output_start = output.len();

                    #[cfg(feature = "visitor")]
                    let mut list_start_custom: Option<String> = None;

                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let parent_tag = dom_ctx
                            .parent_of(node_handle.get_inner())
                            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
                            .map(|s| s.to_string());

                        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::List,
                            tag_name: "ul".to_string(),
                            attributes,
                            depth,
                            index_in_parent: index,
                            parent_tag,
                            is_inline: false,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_list_start(&node_ctx, false) {
                            VisitResult::Continue => {}
                            VisitResult::Custom(custom) => {
                                list_start_custom = Some(custom);
                            }
                            VisitResult::Skip => {
                                return;
                            }
                            VisitResult::PreserveHtml => {
                                serialize_node_to_html(node_handle, parser, output);
                                return;
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                return;
                            }
                        }
                    }

                    process_list_children(
                        *node_handle,
                        parser,
                        output,
                        options,
                        ctx,
                        depth,
                        false,
                        is_loose,
                        nested_depth,
                        1,
                        dom_ctx,
                    );

                    add_nested_list_trailing_separator(output, ctx);

                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let parent_tag = dom_ctx
                            .parent_of(node_handle.get_inner())
                            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
                            .map(|s| s.to_string());

                        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::List,
                            tag_name: "ul".to_string(),
                            attributes,
                            depth,
                            index_in_parent: index,
                            parent_tag,
                            is_inline: false,
                        };

                        let list_content = &output[list_output_start..];

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_list_end(&node_ctx, false, list_content) {
                            VisitResult::Continue => {
                                if let Some(custom_start) = list_start_custom {
                                    output.insert_str(list_output_start, &custom_start);
                                }
                            }
                            VisitResult::Custom(custom) => {
                                let children_output = output[list_output_start..].to_string();
                                output.truncate(list_output_start);
                                if let Some(custom_start) = list_start_custom {
                                    output.push_str(&custom_start);
                                }
                                output.push_str(&children_output);
                                output.push_str(&custom);
                            }
                            VisitResult::Skip => {
                                output.truncate(list_output_start);
                            }
                            VisitResult::PreserveHtml => {
                                output.truncate(list_output_start);
                                serialize_node_to_html(node_handle, parser, output);
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                output.truncate(list_output_start);
                            }
                        }
                    }
                }

                "ol" => {
                    add_list_leading_separator(output, ctx);

                    let nested_depth = calculate_list_nesting_depth(ctx);
                    let is_loose = is_loose_list(*node_handle, parser, dom_ctx);

                    let start = tag
                        .attributes()
                        .get("start")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse::<usize>().ok())
                        .unwrap_or(1);

                    #[cfg(feature = "visitor")]
                    let list_output_start = output.len();

                    #[cfg(feature = "visitor")]
                    let mut list_start_custom: Option<String> = None;

                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let parent_tag = dom_ctx
                            .parent_of(node_handle.get_inner())
                            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
                            .map(|s| s.to_string());

                        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::List,
                            tag_name: "ol".to_string(),
                            attributes,
                            depth,
                            index_in_parent: index,
                            parent_tag,
                            is_inline: false,
                        };

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_list_start(&node_ctx, true) {
                            VisitResult::Continue => {}
                            VisitResult::Custom(custom) => {
                                list_start_custom = Some(custom);
                            }
                            VisitResult::Skip => {
                                return;
                            }
                            VisitResult::PreserveHtml => {
                                serialize_node_to_html(node_handle, parser, output);
                                return;
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                return;
                            }
                        }
                    }

                    process_list_children(
                        *node_handle,
                        parser,
                        output,
                        options,
                        ctx,
                        depth,
                        true,
                        is_loose,
                        nested_depth,
                        start,
                        dom_ctx,
                    );

                    add_nested_list_trailing_separator(output, ctx);

                    #[cfg(feature = "visitor")]
                    if let Some(ref visitor_handle) = ctx.visitor {
                        use crate::visitor::{NodeContext, NodeType, VisitResult};
                        use std::collections::BTreeMap;

                        let attributes: BTreeMap<String, String> = tag
                            .attributes()
                            .iter()
                            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                            .collect();

                        let parent_tag = dom_ctx
                            .parent_of(node_handle.get_inner())
                            .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
                            .map(|s| s.to_string());

                        let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

                        let node_ctx = NodeContext {
                            node_type: NodeType::List,
                            tag_name: "ol".to_string(),
                            attributes,
                            depth,
                            index_in_parent: index,
                            parent_tag,
                            is_inline: false,
                        };

                        let list_content = &output[list_output_start..];

                        let mut visitor = visitor_handle.borrow_mut();
                        match visitor.visit_list_end(&node_ctx, true, list_content) {
                            VisitResult::Continue => {
                                if let Some(custom_start) = list_start_custom {
                                    output.insert_str(list_output_start, &custom_start);
                                }
                            }
                            VisitResult::Custom(custom) => {
                                let children_output = output[list_output_start..].to_string();
                                output.truncate(list_output_start);
                                if let Some(custom_start) = list_start_custom {
                                    output.push_str(&custom_start);
                                }
                                output.push_str(&children_output);
                                output.push_str(&custom);
                            }
                            VisitResult::Skip => {
                                output.truncate(list_output_start);
                            }
                            VisitResult::PreserveHtml => {
                                output.truncate(list_output_start);
                                serialize_node_to_html(node_handle, parser, output);
                            }
                            VisitResult::Error(err) => {
                                if ctx.visitor_error.borrow().is_none() {
                                    *ctx.visitor_error.borrow_mut() = Some(err);
                                }
                                output.truncate(list_output_start);
                            }
                        }
                    }
                }

                "li" => {
                    if ctx.list_depth > 0 {
                        let indent = match options.list_indent_type {
                            ListIndentType::Tabs => "\t".repeat(ctx.list_depth),
                            ListIndentType::Spaces => " ".repeat(ctx.list_depth * options.list_indent_width),
                        };
                        output.push_str(&indent);
                    }

                    let mut has_block_children = false;
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(info) = dom_ctx.tag_info(child_handle.get_inner(), parser) {
                                if matches!(
                                    info.name.as_str(),
                                    "p" | "div" | "blockquote" | "pre" | "table" | "hr" | "dl"
                                ) {
                                    has_block_children = true;
                                    break;
                                }
                            } else if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                if matches!(
                                    tag_name.as_ref(),
                                    "p" | "div" | "blockquote" | "pre" | "table" | "hr" | "dl"
                                ) {
                                    has_block_children = true;
                                    break;
                                }
                            }
                        }
                    }

                    #[allow(clippy::trivially_copy_pass_by_ref)]
                    fn find_checkbox<'a>(
                        node_handle: &tl::NodeHandle,
                        parser: &'a tl::Parser<'a>,
                    ) -> Option<(bool, tl::NodeHandle)> {
                        if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                            if tag_name_eq(node_tag.name().as_utf8_str(), "input") {
                                let input_type = node_tag.attributes().get("type").flatten().map(|v| v.as_utf8_str());

                                if input_type.as_deref() == Some("checkbox") {
                                    let checked = node_tag.attributes().get("checked").is_some();
                                    return Some((checked, *node_handle));
                                }
                            }

                            let children = node_tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(result) = find_checkbox(child_handle, parser) {
                                        return Some(result);
                                    }
                                }
                            }
                        }
                        None
                    }

                    let (is_task_list, task_checked, checkbox_node) =
                        if let Some((checked, node)) = find_checkbox(node_handle, parser) {
                            (true, checked, Some(node))
                        } else {
                            (false, false, None)
                        };

                    let li_ctx = Context {
                        in_list_item: true,
                        list_depth: ctx.list_depth + 1,
                        ..ctx.clone()
                    };

                    if is_task_list {
                        output.push('-');
                        output.push(' ');
                        output.push_str(if task_checked { "[x]" } else { "[ ]" });

                        fn is_checkbox_node(node_handle: &tl::NodeHandle, checkbox: &Option<tl::NodeHandle>) -> bool {
                            if let Some(cb) = checkbox {
                                node_handle == cb
                            } else {
                                false
                            }
                        }

                        fn contains_checkbox<'a>(
                            node_handle: &tl::NodeHandle,
                            parser: &'a tl::Parser<'a>,
                            checkbox: &Option<tl::NodeHandle>,
                        ) -> bool {
                            if is_checkbox_node(node_handle, checkbox) {
                                return true;
                            }
                            if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                                let children = node_tag.children();
                                {
                                    for child_handle in children.top().iter() {
                                        if contains_checkbox(child_handle, parser, checkbox) {
                                            return true;
                                        }
                                    }
                                }
                            }
                            false
                        }

                        #[allow(clippy::too_many_arguments)]
                        fn render_li_content<'a>(
                            node_handle: &tl::NodeHandle,
                            parser: &'a tl::Parser<'a>,
                            output: &mut String,
                            options: &ConversionOptions,
                            ctx: &Context,
                            depth: usize,
                            checkbox: &Option<tl::NodeHandle>,
                            dom_ctx: &DomContext,
                        ) {
                            if is_checkbox_node(node_handle, checkbox) {
                                return;
                            }

                            if contains_checkbox(node_handle, parser, checkbox) {
                                if let Some(tl::Node::Tag(node_tag)) = node_handle.get(parser) {
                                    let children = node_tag.children();
                                    {
                                        for child_handle in children.top().iter() {
                                            render_li_content(
                                                child_handle,
                                                parser,
                                                output,
                                                options,
                                                ctx,
                                                depth,
                                                checkbox,
                                                dom_ctx,
                                            );
                                        }
                                    }
                                }
                            } else {
                                walk_node(node_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }

                        let mut task_text = String::new();
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                render_li_content(
                                    child_handle,
                                    parser,
                                    &mut task_text,
                                    options,
                                    &li_ctx,
                                    depth + 1,
                                    &checkbox_node,
                                    dom_ctx,
                                );
                            }
                        }
                        output.push(' ');
                        let trimmed_task = task_text.trim();
                        if !trimmed_task.is_empty() {
                            output.push_str(trimmed_task);
                        }
                    } else {
                        if !ctx.in_table_cell {
                            if ctx.in_ordered_list {
                                output.push_str(&format!("{}. ", ctx.list_counter));
                            } else {
                                let bullets: Vec<char> = options.bullets.chars().collect();
                                let bullet_index = if ctx.ul_depth > 0 { ctx.ul_depth - 1 } else { 0 };
                                let bullet = bullets.get(bullet_index % bullets.len()).copied().unwrap_or('*');
                                output.push(bullet);
                                output.push(' ');
                            }
                        }

                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, &li_ctx, depth + 1, dom_ctx);
                            }
                        }

                        trim_trailing_whitespace(output);

                        #[cfg(feature = "visitor")]
                        if let Some(ref visitor_handle) = ctx.visitor {
                            use crate::visitor::{NodeContext, NodeType, VisitResult};
                            use std::collections::BTreeMap;

                            let attributes: BTreeMap<String, String> = tag
                                .attributes()
                                .iter()
                                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                                .collect();

                            let parent_tag = dom_ctx
                                .parent_of(node_handle.get_inner())
                                .and_then(|pid| dom_ctx.tag_name_for(dom_ctx.node_handle(pid).copied()?, parser))
                                .map(|s| s.to_string());

                            let index = dom_ctx.sibling_index(node_handle.get_inner()).unwrap_or(0);

                            let node_ctx = NodeContext {
                                node_type: NodeType::ListItem,
                                tag_name: "li".to_string(),
                                attributes,
                                depth,
                                index_in_parent: index,
                                parent_tag,
                                is_inline: false,
                            };

                            let last_line_start = output.rfind('\n').map_or(0, |pos| pos + 1);
                            let last_line = &output[last_line_start..];

                            let (marker, text_content) = if is_task_list {
                                let task_marker = if task_checked { "- [x]" } else { "- [ ]" };
                                let text_start = last_line.find(task_marker).map_or(0, |pos| pos + task_marker.len());
                                (task_marker.to_string(), last_line[text_start..].trim().to_string())
                            } else if ctx.in_ordered_list {
                                let marker_text = format!("{}.", ctx.list_counter);
                                let text_start = last_line.find(&marker_text).map_or(0, |pos| pos + marker_text.len());
                                (marker_text, last_line[text_start..].trim().to_string())
                            } else {
                                let bullets: Vec<char> = options.bullets.chars().collect();
                                let bullet_index = if ctx.ul_depth > 0 { ctx.ul_depth - 1 } else { 0 };
                                let bullet = bullets.get(bullet_index % bullets.len()).copied().unwrap_or('*');
                                let bullet_str = bullet.to_string();
                                let text_start = last_line.find(bullet).map_or(0, |pos| pos + 1);
                                (bullet_str, last_line[text_start..].trim().to_string())
                            };

                            let mut visitor = visitor_handle.borrow_mut();
                            match visitor.visit_list_item(&node_ctx, ctx.in_ordered_list, &marker, &text_content) {
                                VisitResult::Continue => {}
                                VisitResult::Custom(custom) => {
                                    output.truncate(last_line_start);
                                    output.push_str(&custom);
                                    if !ctx.in_table_cell && !output.ends_with('\n') {
                                        output.push('\n');
                                    }
                                    return;
                                }
                                VisitResult::Skip => {
                                    output.truncate(last_line_start);
                                    return;
                                }
                                VisitResult::PreserveHtml => {
                                    output.truncate(last_line_start);
                                    serialize_node_to_html(node_handle, parser, output);
                                    if !ctx.in_table_cell && !output.ends_with('\n') {
                                        output.push('\n');
                                    }
                                    return;
                                }
                                VisitResult::Error(err) => {
                                    if ctx.visitor_error.borrow().is_none() {
                                        *ctx.visitor_error.borrow_mut() = Some(err);
                                    }
                                    return;
                                }
                            }
                        }
                    }

                    if !ctx.in_table_cell {
                        if has_block_children || ctx.loose_list || ctx.prev_item_had_blocks {
                            if !output.ends_with("\n\n") {
                                if output.ends_with('\n') {
                                    output.push('\n');
                                } else {
                                    output.push_str("\n\n");
                                }
                            }
                        } else if !output.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                }

                "table" => {
                    let mut table_output = String::new();
                    crate::converter::block::table::handle_table(
                        node_handle,
                        parser,
                        &mut table_output,
                        options,
                        ctx,
                        dom_ctx,
                        depth,
                    );

                    if ctx.in_list_item {
                        let has_caption = table_output.starts_with('*');

                        if !has_caption {
                            trim_trailing_whitespace(output);
                            if !output.is_empty() && !output.ends_with('\n') {
                                output.push('\n');
                            }
                        }

                        let indented = indent_table_for_list(&table_output, ctx.list_depth, options);
                        output.push_str(&indented);
                    } else {
                        if !output.ends_with("\n\n") {
                            if output.is_empty() || !output.ends_with('\n') {
                                output.push_str("\n\n");
                            } else {
                                output.push('\n');
                            }
                        }
                        output.push_str(&table_output);
                    }

                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }

                "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" => {}

                "caption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        let escaped_text = text.replace('-', r"\-");
                        output.push('*');
                        output.push_str(&escaped_text);
                        output.push_str("*\n\n");
                    }
                }

                "colgroup" | "col" => {}

                "article" | "section" | "nav" | "aside" | "header" | "footer" | "main" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    if content.trim().is_empty() {
                        return;
                    }

                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                    output.push_str(&content);
                    if content.ends_with('\n') && !content.ends_with("\n\n") {
                        output.push('\n');
                    } else if !content.ends_with('\n') {
                        output.push_str("\n\n");
                    }
                }

                "figure" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    if !output.is_empty() && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }

                    let mut figure_content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut figure_content, options, ctx, depth, dom_ctx);
                        }
                    }

                    figure_content = figure_content.replace("\n![", "![");
                    figure_content = figure_content.replace(" ![", "![");

                    let trimmed = figure_content.trim_matches(|c| c == '\n' || c == ' ' || c == '\t');
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !output.ends_with('\n') {
                            output.push('\n');
                        }
                        if !output.ends_with("\n\n") {
                            output.push('\n');
                        }
                    }
                }

                "figcaption" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let text = text.trim();
                    if !text.is_empty() {
                        if !output.is_empty() {
                            if output.ends_with("```\n") {
                                output.push('\n');
                            } else {
                                trim_trailing_whitespace(output);
                                if output.ends_with('\n') && !output.ends_with("\n\n") {
                                    output.push('\n');
                                } else if !output.ends_with('\n') {
                                    output.push_str("\n\n");
                                }
                            }
                        }
                        output.push('*');
                        output.push_str(text);
                        output.push_str("*\n\n");
                    }
                }

                "hgroup" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                "cite" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push('*');
                            output.push_str(trimmed);
                            output.push('*');
                        }
                    }
                }

                "q" => {
                    let mut content = String::with_capacity(32);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push('"');
                            let escaped = trimmed.replace('\\', r"\\").replace('"', r#"\""#);
                            output.push_str(&escaped);
                            output.push('"');
                        }
                    }
                }

                "dl" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::new();
                    let mut in_dt_group = false;
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let (is_definition_term, is_definition_description) =
                                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                    let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                    (tag_name == "dt", tag_name == "dd")
                                } else {
                                    (false, false)
                                };

                            let child_ctx = Context {
                                last_was_dt: in_dt_group && is_definition_description,
                                ..ctx.clone()
                            };
                            walk_node(child_handle, parser, &mut content, options, &child_ctx, depth, dom_ctx);

                            if is_definition_term {
                                in_dt_group = true;
                            } else if !is_definition_description {
                                in_dt_group = false;
                            }
                        }
                    }

                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "dt" => {
                    let mut content = String::with_capacity(64);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            output.push_str(trimmed);
                            output.push('\n');
                        }
                    }
                }

                "dd" => {
                    let mut content = String::with_capacity(128);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    let trimmed = content.trim();

                    if ctx.convert_as_inline {
                        if !trimmed.is_empty() {
                            output.push_str(trimmed);
                        }
                    } else if ctx.last_was_dt {
                        if trimmed.is_empty() {
                            output.push_str(":   \n\n");
                        } else {
                            output.push_str(":   ");
                            output.push_str(trimmed);
                            output.push_str("\n\n");
                        }
                    } else if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "details" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::with_capacity(256);
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "summary" => {
                    let mut content = String::with_capacity(64);
                    let mut summary_ctx = ctx.clone();
                    if !ctx.convert_as_inline {
                        summary_ctx.in_strong = true;
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &summary_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            output.push_str(trimmed);
                            output.push_str(&symbol);
                            output.push_str("\n\n");
                        }
                    }
                }

                "dialog" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let content_start = output.len();

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }

                    while output.len() > content_start && (output.ends_with(' ') || output.ends_with('\t')) {
                        output.pop();
                    }

                    if output.len() > content_start && !output.ends_with("\n\n") {
                        output.push_str("\n\n");
                    }
                }

                "menu" => {
                    let content_start = output.len();

                    let menu_options = ConversionOptions {
                        bullets: "-".to_string(),
                        ..options.clone()
                    };

                    let list_ctx = Context {
                        in_ordered_list: false,
                        list_counter: 0,
                        in_list: true,
                        list_depth: ctx.list_depth,
                        ..ctx.clone()
                    };

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, &menu_options, &list_ctx, depth, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > content_start {
                        if !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    } else if ctx.convert_as_inline {
                        while output.ends_with('\n') {
                            output.pop();
                        }
                    }
                }

                "audio" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .or_else(|| {
                            let children = tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                        if tag_name_eq(child_tag.name().as_utf8_str(), "source") {
                                            return child_tag
                                                .attributes()
                                                .get("src")
                                                .flatten()
                                                .map(|v| v.as_utf8_str());
                                        }
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or(Cow::Borrowed(""));

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    let mut fallback = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let is_source = if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                tag_name_eq(child_tag.name().as_utf8_str(), "source")
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    }
                    if !fallback.is_empty() {
                        output.push_str(fallback.trim());
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "video" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map(|v| v.as_utf8_str())
                        .or_else(|| {
                            let children = tag.children();
                            {
                                for child_handle in children.top().iter() {
                                    if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                        if tag_name_eq(child_tag.name().as_utf8_str(), "source") {
                                            return child_tag
                                                .attributes()
                                                .get("src")
                                                .flatten()
                                                .map(|v| v.as_utf8_str());
                                        }
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or(Cow::Borrowed(""));

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }

                    let mut fallback = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            let is_source = if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                tag_name_eq(child_tag.name().as_utf8_str(), "source")
                            } else {
                                false
                            };

                            if !is_source {
                                walk_node(child_handle, parser, &mut fallback, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                    }
                    if !fallback.is_empty() {
                        output.push_str(fallback.trim());
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "source" => {}

                "picture" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                if tag_name_eq(child_tag.name().as_utf8_str(), "img") {
                                    walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                                    break;
                                }
                            }
                        }
                    }
                }

                "iframe" => {
                    use std::borrow::Cow;

                    let src = tag
                        .attributes()
                        .get("src")
                        .flatten()
                        .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

                    if !src.is_empty() {
                        output.push('[');
                        output.push_str(&src);
                        output.push_str("](");
                        output.push_str(&src);
                        output.push(')');
                        if !ctx.in_paragraph && !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "svg" => {
                    let mut title = String::from("SVG Image");
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                if tag_name_eq(child_tag.name().as_utf8_str(), "title") {
                                    title = get_text_content(child_handle, parser, dom_ctx).trim().to_string();
                                    break;
                                }
                            }
                        }
                    }

                    #[cfg(feature = "inline-images")]
                    if let Some(ref collector_ref) = ctx.inline_collector {
                        let title_opt = if title == "SVG Image" {
                            None
                        } else {
                            Some(title.clone())
                        };
                        let mut attributes_map = BTreeMap::new();
                        for (key, value_opt) in tag.attributes().iter() {
                            let key_str = key.to_string();
                            let keep = key_str == "width"
                                || key_str == "height"
                                || key_str == "filename"
                                || key_str == "aria-label"
                                || key_str.starts_with("data-");
                            if keep {
                                let value = value_opt.map(|value| value.to_string()).unwrap_or_default();
                                attributes_map.insert(key_str, value);
                            }
                        }
                        handle_inline_svg(collector_ref, node_handle, parser, title_opt, attributes_map);
                    }

                    if ctx.convert_as_inline {
                        output.push_str(&title);
                    } else {
                        use base64::{Engine as _, engine::general_purpose::STANDARD};

                        let svg_html = serialize_element(node_handle, parser);

                        let base64_svg = STANDARD.encode(svg_html.as_bytes());

                        output.push_str("![");
                        output.push_str(&title);
                        output.push_str("](data:image/svg+xml;base64,");
                        output.push_str(&base64_svg);
                        output.push(')');
                    }
                }

                "math" => {
                    let text_content = get_text_content(node_handle, parser, dom_ctx).trim().to_string();

                    if text_content.is_empty() {
                        return;
                    }

                    let math_html = serialize_element(node_handle, parser);

                    let escaped_text = text::escape(
                        &text_content,
                        options.escape_misc,
                        options.escape_asterisks,
                        options.escape_underscores,
                        options.escape_ascii,
                    );

                    let is_display_block = tag
                        .attributes()
                        .get("display")
                        .flatten()
                        .is_some_and(|v| v.as_utf8_str() == "block");

                    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
                        output.push_str("\n\n");
                    }

                    output.push_str("<!-- MathML: ");
                    output.push_str(&math_html);
                    output.push_str(" --> ");
                    output.push_str(&escaped_text);

                    if is_display_block && !ctx.in_paragraph && !ctx.convert_as_inline {
                        output.push_str("\n\n");
                    }
                }

                "form" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }

                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "fieldset" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                            }
                        }
                        return;
                    }
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            output.push_str("\n\n");
                        }
                        output.push_str(trimmed);
                        output.push_str("\n\n");
                    }
                }

                "legend" => {
                    let mut content = String::new();
                    let mut legend_ctx = ctx.clone();
                    if !ctx.convert_as_inline {
                        legend_ctx.in_strong = true;
                    }
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(
                                child_handle,
                                parser,
                                &mut content,
                                options,
                                &legend_ctx,
                                depth + 1,
                                dom_ctx,
                            );
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        if ctx.convert_as_inline {
                            output.push_str(trimmed);
                        } else {
                            let symbol = options.strong_em_symbol.to_string().repeat(2);
                            output.push_str(&symbol);
                            output.push_str(trimmed);
                            output.push_str(&symbol);
                            output.push_str("\n\n");
                        }
                    }
                }

                "label" => {
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                        if !ctx.convert_as_inline {
                            output.push_str("\n\n");
                        }
                    }
                }

                "input" => {}

                "textarea" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "select" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "option" => {
                    let selected = tag.attributes().iter().any(|(name, _)| name.as_ref() == "selected");

                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if selected && !ctx.convert_as_inline {
                            output.push_str("* ");
                        }
                        output.push_str(trimmed);
                        if !ctx.convert_as_inline {
                            output.push('\n');
                        }
                    }
                }

                "optgroup" => {
                    use std::borrow::Cow;

                    let label = tag
                        .attributes()
                        .get("label")
                        .flatten()
                        .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

                    if !label.is_empty() {
                        let symbol = options.strong_em_symbol.to_string().repeat(2);
                        output.push_str(&symbol);
                        output.push_str(&label);
                        output.push_str(&symbol);
                        output.push('\n');
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "button" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "progress" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "meter" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "output" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push_str("\n\n");
                    }
                }

                "datalist" => {
                    let start_len = output.len();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    if !ctx.convert_as_inline && output.len() > start_len {
                        output.push('\n');
                    }
                }

                "ruby" => {
                    let ruby_ctx = ctx.clone();

                    let tag_sequence: Vec<String> = tag
                        .children()
                        .top()
                        .iter()
                        .filter_map(|child_handle| {
                            if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                if matches!(tag_name.as_ref(), "rb" | "rt" | "rtc") {
                                    Some(tag_name.into_owned())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    let has_rtc = tag_sequence.iter().any(|tag| tag == "rtc");

                    let is_interleaved = tag_sequence.windows(2).any(|w| w[0] == "rb" && w[1] == "rt");

                    if is_interleaved && !has_rtc {
                        let mut current_base = String::new();
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                if let Some(node) = child_handle.get(parser) {
                                    match node {
                                        tl::Node::Tag(child_tag) => {
                                            let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                            if tag_name == "rt" {
                                                let mut annotation = String::new();
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut annotation,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                                if !current_base.is_empty() {
                                                    output.push_str(current_base.trim());
                                                    current_base.clear();
                                                }
                                                output.push_str(annotation.trim());
                                            } else if tag_name == "rb" {
                                                if !current_base.is_empty() {
                                                    output.push_str(current_base.trim());
                                                    current_base.clear();
                                                }
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut current_base,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                            } else if tag_name != "rp" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut current_base,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                            }
                                        }
                                        tl::Node::Raw(_) => {
                                            walk_node(
                                                child_handle,
                                                parser,
                                                &mut current_base,
                                                options,
                                                &ruby_ctx,
                                                depth,
                                                dom_ctx,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        if !current_base.is_empty() {
                            output.push_str(current_base.trim());
                        }
                    } else {
                        let mut base_text = String::new();
                        let mut rt_annotations = Vec::new();
                        let mut rtc_content = String::new();

                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                if let Some(node) = child_handle.get(parser) {
                                    match node {
                                        tl::Node::Tag(child_tag) => {
                                            let tag_name = normalized_tag_name(child_tag.name().as_utf8_str());
                                            if tag_name == "rt" {
                                                let mut annotation = String::new();
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut annotation,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                                rt_annotations.push(annotation);
                                            } else if tag_name == "rtc" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut rtc_content,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                            } else if tag_name != "rp" {
                                                walk_node(
                                                    child_handle,
                                                    parser,
                                                    &mut base_text,
                                                    options,
                                                    &ruby_ctx,
                                                    depth,
                                                    dom_ctx,
                                                );
                                            }
                                        }
                                        tl::Node::Raw(_) => {
                                            walk_node(
                                                child_handle,
                                                parser,
                                                &mut base_text,
                                                options,
                                                &ruby_ctx,
                                                depth,
                                                dom_ctx,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        let trimmed_base = base_text.trim();

                        output.push_str(trimmed_base);

                        if !rt_annotations.is_empty() {
                            let rt_text = rt_annotations.iter().map(|s| s.trim()).collect::<Vec<_>>().join("");
                            if !rt_text.is_empty() {
                                if has_rtc && !rtc_content.trim().is_empty() && rt_annotations.len() > 1 {
                                    output.push('(');
                                    output.push_str(&rt_text);
                                    output.push(')');
                                } else {
                                    output.push_str(&rt_text);
                                }
                            }
                        }

                        if !rtc_content.trim().is_empty() {
                            output.push_str(rtc_content.trim());
                        }
                    }
                }

                "rb" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    output.push_str(text.trim());
                }

                "rt" => {
                    let mut text = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = text.trim();

                    if output.ends_with('(') {
                        output.push_str(trimmed);
                    } else {
                        output.push('(');
                        output.push_str(trimmed);
                        output.push(')');
                    }
                }

                "rp" => {
                    let mut content = String::new();
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                    }
                }

                "rtc" => {
                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                "div" => {
                    if ctx.convert_as_inline {
                        let children = tag.children();
                        {
                            for child_handle in children.top().iter() {
                                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                            }
                        }
                        return;
                    }

                    let content_start_pos = output.len();

                    let is_table_continuation =
                        ctx.in_table_cell && !output.is_empty() && !output.ends_with('|') && !output.ends_with("<br>");

                    let is_list_continuation = ctx.in_list_item
                        && !output.is_empty()
                        && !output.ends_with("* ")
                        && !output.ends_with("- ")
                        && !output.ends_with(". ");

                    let needs_leading_sep = !ctx.in_table_cell
                        && !ctx.in_list_item
                        && !ctx.convert_as_inline
                        && !output.is_empty()
                        && !output.ends_with("\n\n");

                    if is_table_continuation {
                        trim_trailing_whitespace(output);
                        output.push_str("<br>");
                    } else if is_list_continuation {
                        add_list_continuation_indent(output, ctx.list_depth, false, options);
                    } else if needs_leading_sep {
                        trim_trailing_whitespace(output);
                        output.push_str("\n\n");
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    let has_content = output.len() > content_start_pos;

                    if has_content {
                        if content_start_pos == 0 && output.starts_with('\n') && !output.starts_with("\n\n") {
                            output.remove(0);
                        }
                        trim_trailing_whitespace(output);

                        if ctx.in_table_cell {
                        } else if ctx.in_list_item {
                            if is_list_continuation {
                                if !output.ends_with('\n') {
                                    output.push('\n');
                                }
                            } else if !output.ends_with("\n\n") {
                                if output.ends_with('\n') {
                                    output.push('\n');
                                } else {
                                    output.push_str("\n\n");
                                }
                            }
                        } else if !ctx.in_list_item && !ctx.convert_as_inline {
                            if output.ends_with("\n\n") {
                            } else if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                    }
                }

                "head" => {
                    let children = tag.children();
                    let has_body_like = children.top().iter().any(|child_handle| {
                        if let Some(child_name) = dom_ctx.tag_name_for(*child_handle, parser) {
                            matches!(
                                child_name.as_ref(),
                                "body" | "main" | "article" | "section" | "div" | "p"
                            )
                        } else {
                            false
                        }
                    });

                    #[cfg(feature = "metadata")]
                    if ctx.metadata_wants_structured_data {
                        if let Some(ref collector) = ctx.metadata_collector {
                            for child_handle in children.top().iter() {
                                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                                    let child_name = dom_ctx
                                        .tag_name_for(*child_handle, parser)
                                        .unwrap_or_else(|| normalized_tag_name(child_tag.name().as_utf8_str()));
                                    if child_name.as_ref() == "script" {
                                        if let Some(type_attr) = child_tag.attributes().get("type").flatten() {
                                            let type_value = type_attr.as_utf8_str();
                                            let type_value = type_value.as_ref();
                                            let type_value = type_value.split(';').next().unwrap_or(type_value);
                                            if type_value.trim().eq_ignore_ascii_case("application/ld+json") {
                                                let json = child_tag.inner_text(parser);
                                                let json = json.trim();
                                                if !json.is_empty() {
                                                    let json = text::decode_html_entities(json).clone();
                                                    if !json.is_empty() {
                                                        collector.borrow_mut().add_json_ld(json);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if has_body_like {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                "script" =>
                {
                    #[cfg(feature = "metadata")]
                    if let Some(type_attr) = tag.attributes().get("type").flatten() {
                        let type_value = type_attr.as_utf8_str();
                        let type_value = type_value.as_ref();
                        let type_value = type_value.split(';').next().unwrap_or(type_value);
                        if type_value.trim().eq_ignore_ascii_case("application/ld+json")
                            && ctx.metadata_wants_structured_data
                        {
                            if let Some(ref collector) = ctx.metadata_collector {
                                let json = tag.inner_text(parser);
                                let json = json.trim();
                                if !json.is_empty() {
                                    let json = text::decode_html_entities(json);
                                    if !json.is_empty() {
                                        collector.borrow_mut().add_json_ld(json);
                                    }
                                }
                            }
                        }
                    }
                }
                "style" => {}

                "span" => {
                    let is_hocr_word = tag.attributes().iter().any(|(name, value)| {
                        name.as_ref() == "class" && value.as_ref().is_some_and(|v| v.as_ref().contains("ocrx_word"))
                    });

                    if is_hocr_word
                        && !output.is_empty()
                        && !output.ends_with(' ')
                        && !output.ends_with('\t')
                        && !output.ends_with('\n')
                    {
                        output.push(' ');
                    }

                    if !ctx.in_code
                        && options.whitespace_mode == crate::options::WhitespaceMode::Normalized
                        && output.ends_with('\n')
                        && !output.ends_with("\n\n")
                    {
                        output.pop();
                    }

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                        }
                    }
                }

                "body" | "html" => {
                    // Process children of body/html tags directly without whitespace truncation
                    // These are structural container tags that should always preserve their content

                    let children = tag.children();

                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }
                }

                _ => {
                    let len_before = output.len();
                    let had_trailing_space = output.ends_with(' ');

                    let children = tag.children();
                    {
                        for child_handle in children.top().iter() {
                            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                        }
                    }

                    let len_after = output.len();
                    if len_after > len_before {
                        let start_idx = if output.is_char_boundary(len_before) {
                            len_before
                        } else {
                            let capped = len_before.min(output.len());
                            output
                                .char_indices()
                                .map(|(idx, _)| idx)
                                .take_while(|idx| *idx <= capped)
                                .last()
                                .unwrap_or(capped)
                        };

                        let added_content = output[start_idx..].to_string();

                        let is_code_block = added_content.starts_with("    ")
                            || added_content.starts_with("```")
                            || added_content.starts_with("~~~");

                        if added_content.trim().is_empty() && !is_code_block {
                            output.truncate(start_idx);
                            if !had_trailing_space && added_content.contains(' ') {
                                output.push(' ');
                            }
                        }
                    }
                }
            }

            #[cfg(feature = "visitor")]
            if !matches!(tag_name.as_ref(), "table") {
                if let Some(ref visitor_handle) = ctx.visitor {
                    use crate::visitor::{NodeContext, NodeType, VisitResult};
                    use std::collections::BTreeMap;

                    let attributes: BTreeMap<String, String> = tag
                        .attributes()
                        .iter()
                        .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                        .collect();

                    let node_id = node_handle.get_inner();
                    let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                    let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                    let node_ctx = NodeContext {
                        node_type: NodeType::Element,
                        tag_name: tag_name.to_string(),
                        attributes,
                        depth,
                        index_in_parent,
                        parent_tag,
                        is_inline: !is_block_level_element(&tag_name),
                    };

                    let element_content = &output[element_output_start..];

                    let mut visitor = visitor_handle.borrow_mut();
                    match visitor.visit_element_end(&node_ctx, element_content) {
                        VisitResult::Continue => {}
                        VisitResult::Custom(custom) => {
                            output.truncate(element_output_start);
                            output.push_str(&custom);
                        }
                        VisitResult::Skip => {
                            output.truncate(element_output_start);
                        }
                        VisitResult::Error(err) => {
                            if ctx.visitor_error.borrow().is_none() {
                                *ctx.visitor_error.borrow_mut() = Some(err);
                            }
                        }
                        VisitResult::PreserveHtml => {}
                    }
                }
            }
        }

        tl::Node::Comment(_) => {}
    }
}
