//! Builds a [`DocumentStructure`] from a parsed `tl::VDom`.
//!
//! Walk the DOM once, mapping each HTML element to the appropriate [`NodeContent`] variant,
//! collecting inline [`TextAnnotation`]s, tracking parent/child relationships, and generating
//! heading-based [`NodeContent::Group`] hierarchy.

use std::collections::HashMap;

use crate::options::conversion::NATIVE_STACK_SAFE_DEPTH;

use super::document::{AnnotationKind, DocumentNode, DocumentStructure, MetadataEntry, NodeContent, TextAnnotation};
use super::tables::{GridCell, TableGrid};

fn reversed_child_handles(tag: &tl::HTMLTag) -> Vec<tl::NodeHandle> {
    let children = tag.children();
    let mut handles: Vec<_> = children.top().iter().copied().collect();
    handles.reverse();
    handles
}

/// Extract plain text from a tag's descendants, decoding HTML entities.
fn extract_text(tag: &tl::HTMLTag, parser: &tl::Parser) -> String {
    let mut buf = String::new();
    let mut stack = reversed_child_handles(tag);

    while let Some(handle) = stack.pop() {
        let Some(node) = handle.get(parser) else {
            continue;
        };
        match node {
            tl::Node::Raw(bytes) => {
                let raw = bytes.as_utf8_str();
                let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
                buf.push_str(&decoded);
            }
            tl::Node::Tag(child_tag) => {
                let name = child_tag.name().as_utf8_str().to_ascii_lowercase();
                if matches!(name.as_str(), "script" | "style" | "head") {
                    continue;
                }
                for child_handle in reversed_child_handles(child_tag) {
                    stack.push(child_handle);
                }
            }
            tl::Node::Comment(_) => {}
        }
    }

    buf
}

/// Scan the children of `tag` and collect [`TextAnnotation`]s into `annotations`.
///
/// `text` is the pre-extracted full text of the enclosing block node; annotation
/// byte offsets are computed relative to that string.
fn collect_annotations(tag: &tl::HTMLTag, parser: &tl::Parser, text: &str, annotations: &mut Vec<TextAnnotation>) {
    enum Frame {
        Visit(tl::NodeHandle),
        Finish { start: usize, kind: Option<AnnotationKind> },
    }

    let mut offset = 0usize;
    let mut stack: Vec<_> = reversed_child_handles(tag).into_iter().map(Frame::Visit).collect();

    while let Some(frame) = stack.pop() {
        let handle = match frame {
            Frame::Visit(handle) => handle,
            Frame::Finish { start, kind } => {
                let end = offset;
                if let Some(kind) = kind {
                    if start < end && end <= text.len() {
                        annotations.push(TextAnnotation {
                            start: start as u32,
                            end: end as u32,
                            kind,
                        });
                    }
                }
                continue;
            }
        };

        let Some(node) = handle.get(parser) else {
            continue;
        };
        match node {
            tl::Node::Raw(bytes) => {
                let raw = bytes.as_utf8_str();
                let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
                offset += decoded.len();
            }
            tl::Node::Tag(child_tag) => {
                let name = child_tag.name().as_utf8_str().to_ascii_lowercase();
                if matches!(name.as_str(), "script" | "style" | "head") {
                    continue;
                }

                let kind = annotation_kind_for_tag(name.as_str(), child_tag);

                stack.push(Frame::Finish { start: offset, kind });
                for child_handle in reversed_child_handles(child_tag) {
                    stack.push(Frame::Visit(child_handle));
                }
            }
            tl::Node::Comment(_) => {}
        }
    }
}

pub(crate) fn annotation_kind_for_tag(name: &str, tag: &tl::HTMLTag) -> Option<AnnotationKind> {
    match name {
        "strong" | "b" => Some(AnnotationKind::Bold),
        "em" | "i" => Some(AnnotationKind::Italic),
        "u" | "ins" => Some(AnnotationKind::Underline),
        "s" | "del" | "strike" => Some(AnnotationKind::Strikethrough),
        "code" | "kbd" | "samp" => Some(AnnotationKind::Code),
        "sub" => Some(AnnotationKind::Subscript),
        "sup" => Some(AnnotationKind::Superscript),
        "mark" => Some(AnnotationKind::Highlight),
        "a" => {
            let url = tag
                .attributes()
                .get("href")
                .flatten()
                .map(|value| value.as_utf8_str().to_string())
                .unwrap_or_default();
            let title = tag
                .attributes()
                .get("title")
                .flatten()
                .map(|value| value.as_utf8_str().to_string());
            Some(AnnotationKind::Link { url, title })
        }
        _ => None,
    }
}

fn collect_annotated_text(tag: &tl::HTMLTag, parser: &tl::Parser) -> (String, Vec<TextAnnotation>) {
    let raw_text = extract_text(tag, parser);
    let mut annotations = Vec::new();
    collect_annotations(tag, parser, &raw_text, &mut annotations);
    trim_annotated_text(&raw_text, annotations)
}

/// Trim the white space from the ends of `text` and move `annotations` to the trimmed text.
///
/// ~keep The end is the start plus the length of the trimmed slice, so it is never before the
/// ~keep start. Two indexes measured from the two ends cross when the text is only white space,
/// ~keep and a slice between them panics (issue #749). Such text gives the empty text here, and
/// ~keep neither structure builder records a heading, paragraph or list item for empty text.
pub(super) fn trim_annotated_text(text: &str, mut annotations: Vec<TextAnnotation>) -> (String, Vec<TextAnnotation>) {
    let without_leading = text.trim_start();
    let trimmed = without_leading.trim_end();
    let text_start = text.len() - without_leading.len();
    let text_end = text_start + trimmed.len();
    annotations.retain_mut(|annotation| {
        let start = (annotation.start as usize).max(text_start);
        let end = (annotation.end as usize).min(text_end);
        if start >= end {
            return false;
        }
        annotation.start = (start - text_start) as u32;
        annotation.end = (end - text_start) as u32;
        true
    });
    (trimmed.to_owned(), annotations)
}

/// Build a [`TableGrid`] from a `<table>` element.
fn extract_table_grid(table_tag: &tl::HTMLTag, parser: &tl::Parser) -> TableGrid {
    let mut row_handles: Vec<tl::NodeHandle> = Vec::new();
    collect_tr_handles(table_tag, parser, &mut row_handles);

    let mut cells: Vec<GridCell> = Vec::new();
    let mut max_col: u32 = 0;

    for (row_idx, row_handle) in row_handles.iter().enumerate() {
        let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) else {
            continue;
        };

        let mut col_idx: u32 = 0;
        let row_children = row_tag.children();

        for child_handle in row_children.top().iter() {
            let Some(tl::Node::Tag(cell_tag)) = child_handle.get(parser) else {
                continue;
            };
            let cell_name = cell_tag.name().as_utf8_str().to_ascii_lowercase();
            let is_cell = cell_name == "td" || cell_name == "th";
            if !is_cell {
                continue;
            }

            let is_header = cell_name == "th";

            let row_span = cell_tag
                .attributes()
                .get("rowspan")
                .flatten()
                .and_then(|v| v.as_utf8_str().parse::<u32>().ok())
                .unwrap_or(1)
                .max(1);

            let col_span = cell_tag
                .attributes()
                .get("colspan")
                .flatten()
                .and_then(|v| v.as_utf8_str().parse::<u32>().ok())
                .unwrap_or(1)
                .max(1);

            let content = extract_text(cell_tag, parser).trim().to_string();

            cells.push(GridCell {
                content,
                row: row_idx as u32,
                col: col_idx,
                row_span,
                col_span,
                is_header,
            });

            col_idx += col_span;
            if col_idx > max_col {
                max_col = col_idx;
            }
        }
    }

    let rows = row_handles.len() as u32;
    TableGrid {
        rows,
        cols: max_col,
        cells,
    }
}

/// Collect all `<tr>` `NodeHandle`s from within a table element.
fn collect_tr_handles(tag: &tl::HTMLTag, parser: &tl::Parser, result: &mut Vec<tl::NodeHandle>) {
    let mut stack = reversed_child_handles(tag);

    while let Some(handle) = stack.pop() {
        if let Some(tl::Node::Tag(child_tag)) = handle.get(parser) {
            let name = child_tag.name().as_utf8_str().to_ascii_lowercase();
            if name == "tr" {
                result.push(handle);
            } else {
                for child_handle in reversed_child_handles(child_tag) {
                    stack.push(child_handle);
                }
            }
        }
    }
}

/// Generate a deterministic node ID from the node type, an excerpt of its text content,
/// and its position (index) in the flat node list.
fn make_node_id(node_type: &str, text: &str, index: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    node_type.hash(&mut hasher);
    let end = crate::converter::utility::content::floor_char_boundary(text, text.len().min(64));
    text[..end].hash(&mut hasher);
    index.hash(&mut hasher);
    let digest = hasher.finish();
    format!("{node_type}-{digest:016x}")
}

/// Collect `<dt>`/`<dd>` pairs from a `<dl>` element.
///
/// Returns `(term_text, definition_text)` tuples.  Consecutive `<dt>` elements share
/// the next `<dd>`; orphan `<dd>`s use an empty term.
fn collect_definition_items(dl_tag: &tl::HTMLTag, parser: &tl::Parser) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = Vec::new();
    let mut pending_terms: Vec<String> = Vec::new();

    let children = dl_tag.children();
    for handle in children.top().iter() {
        let Some(tl::Node::Tag(child_tag)) = handle.get(parser) else {
            continue;
        };
        let name = child_tag.name().as_utf8_str().to_ascii_lowercase();
        match name.as_str() {
            "dt" => {
                pending_terms.push(extract_text(child_tag, parser).trim().to_string());
            }
            "dd" => {
                let definition = extract_text(child_tag, parser).trim().to_string();
                if pending_terms.is_empty() {
                    items.push((String::new(), definition));
                } else {
                    let mut drained: Vec<String> = std::mem::take(&mut pending_terms);
                    let last_term = drained.pop();
                    for term in drained {
                        items.push((term, String::new()));
                    }
                    if let Some(term) = last_term {
                        items.push((term, definition));
                    }
                }
            }
            _ => {}
        }
    }

    for term in pending_terms {
        items.push((term, String::new()));
    }

    items
}

/// Extract `<meta name=… content=…>` and `<title>` entries from a `<head>` element.
fn extract_head_metadata_entries(head_tag: &tl::HTMLTag, parser: &tl::Parser) -> Vec<MetadataEntry> {
    let mut entries: Vec<MetadataEntry> = Vec::new();

    let children = head_tag.children();
    for handle in children.top().iter() {
        let Some(tl::Node::Tag(child_tag)) = handle.get(parser) else {
            continue;
        };
        let name = child_tag.name().as_utf8_str().to_ascii_lowercase();
        match name.as_str() {
            "title" => {
                let title = extract_text(child_tag, parser).trim().to_string();
                if !title.is_empty() {
                    entries.push(MetadataEntry {
                        key: "title".to_string(),
                        value: title,
                    });
                }
            }
            "meta" => {
                let content = crate::converter::utility::attributes::decoded_attribute(child_tag, "content");
                for key_attr in ["name", "property"] {
                    if let (Some(Some(key)), Some(content)) = (child_tag.attributes().get(key_attr), content.as_ref()) {
                        entries.push(MetadataEntry {
                            key: key.as_utf8_str().to_string(),
                            value: content.to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    entries
}

/// State threaded through the recursive walk.
struct BuilderState {
    /// Accumulated nodes (flat list in document order).
    nodes: Vec<DocumentNode>,
    /// Stack of open heading-group indices: `(heading_level, node_index)`.
    group_stack: Vec<(u8, u32)>,
    /// The document's head, the only `<head>` that gives a metadata block.
    head: Option<tl::NodeHandle>,
}

impl BuilderState {
    const fn new(head: Option<tl::NodeHandle>) -> Self {
        Self {
            nodes: Vec::new(),
            group_stack: Vec::new(),
            head,
        }
    }

    /// Append a node and return its index.
    fn push(&mut self, node: DocumentNode) -> u32 {
        let idx = self.nodes.len() as u32;
        self.nodes.push(node);
        idx
    }

    fn push_attached(&mut self, node: DocumentNode) -> u32 {
        let parent = node.parent;
        let idx = self.push(node);
        if let Some(parent) = parent {
            self.add_child(parent, idx);
        }
        idx
    }

    /// Index of the innermost open group, if any.
    fn current_group(&self) -> Option<u32> {
        self.group_stack.last().map(|(_, idx)| *idx)
    }

    /// Record `child_idx` as a child of `parent_idx`.
    fn add_child(&mut self, parent_idx: u32, child_idx: u32) {
        if let Some(parent) = self.nodes.get_mut(parent_idx as usize) {
            parent.children.push(child_idx);
        }
    }
}

/// Build a [`DocumentStructure`] from an already-parsed `tl::VDom`.
///
/// Walks the DOM once, mapping HTML elements to semantic [`NodeContent`] variants,
/// tracking parent/child relationships, extracting inline [`TextAnnotation`]s, and
/// constructing heading-based [`NodeContent::Group`] nodes.
///
/// This function is infallible: malformed or unexpected HTML is silently skipped rather than
/// returned as an error. Callers that need warnings about skipped content should use the
/// incremental `StructureCollector` directly and inspect the conversion pipeline's warning
/// channel.
///
/// The returned [`DocumentStructure`] has `source_format` set to `"html"`. The node array is
/// in document reading order; each node carries index-based `parent` and `children` references
/// into the same array.
///
/// # Examples
///
/// ```rust,no_run
/// use html_to_markdown_rs::types::build_document_structure;
///
/// let dom = tl::parse("<h1>Title</h1><p>Body</p>", tl::ParserOptions::default())
///     .expect("valid HTML");
/// let doc = build_document_structure(&dom);
/// assert_eq!(doc.source_format.as_deref(), Some("html"));
/// assert!(!doc.nodes.is_empty());
/// ```
#[must_use]
pub fn build_document_structure(dom: &tl::VDom<'_>) -> DocumentStructure {
    let parser = dom.parser();
    let mut state = BuilderState::new(crate::converter::document_head(dom.children(), parser));

    for handle in dom.children() {
        walk(&mut state, handle, parser, None, 0);
    }

    DocumentStructure {
        nodes: state.nodes,
        source_format: Some("html".to_string()),
    }
}

/// Recursive DOM walker.
///
/// `parent_idx` is the flat-list index of the nearest structural parent, if any.
fn walk(state: &mut BuilderState, handle: &tl::NodeHandle, parser: &tl::Parser, parent_idx: Option<u32>, depth: usize) {
    if depth >= NATIVE_STACK_SAFE_DEPTH {
        return;
    }

    let Some(node) = handle.get(parser) else {
        return;
    };

    match node {
        tl::Node::Raw(_) | tl::Node::Comment(_) => {}
        tl::Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str().to_ascii_lowercase();
            if tag_name == "head" && state.head != Some(*handle) {
                return;
            }
            process_tag(state, tag_name.as_str(), tag, parser, parent_idx, depth);
        }
    }
}

/// Decide how to handle a given tag, creating nodes and visiting children as needed.
fn process_tag(
    state: &mut BuilderState,
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
    depth: usize,
) {
    match tag_name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            process_heading(state, tag_name, tag, parser, parent_idx);
        }

        "p" => {
            process_paragraph(state, tag, parser, parent_idx);
        }

        "ul" | "ol" => {
            process_list(state, tag_name == "ol", tag, parser, parent_idx, depth);
        }

        "li" => {
            process_list_item(state, tag, parser, parent_idx);
        }

        "table" => {
            process_table(state, tag, parser, parent_idx);
        }

        "img" => {
            process_image(state, tag, parent_idx);
        }

        "pre" => {
            process_code_block(state, tag, parser, parent_idx);
        }

        "blockquote" => {
            process_quote(state, tag, parser, parent_idx, depth);
        }

        "dl" => {
            process_definition_list(state, tag, parser, parent_idx);
        }

        "script" | "style" => {
            process_raw_block(state, tag_name, tag, parser, parent_idx);
        }

        "head" => {
            process_head(state, tag, parser);
        }

        "main" | "article" | "section" | "header" | "footer" | "nav" | "aside" => {
            process_section_group(state, tag_name, tag, parser, parent_idx, depth);
        }

        "html" | "body" | "div" | "figure" | "figcaption" | "details" | "summary" | "address" | "hgroup" | "search"
        | "form" | "fieldset" => {
            walk_children(state, tag, parser, parent_idx, depth);
        }

        _ => {
            walk_children(state, tag, parser, parent_idx, depth);
        }
    }
}

fn process_raw_block(
    state: &mut BuilderState,
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
) {
    let format = if tag_name == "script" {
        tag.attributes()
            .get("type")
            .flatten()
            .map_or_else(|| "javascript".to_string(), |value| value.as_utf8_str().to_string())
    } else {
        "css".to_string()
    };
    let content = extract_text(tag, parser);
    if content.trim().is_empty() {
        return;
    }
    state.push_attached(DocumentNode {
        id: make_node_id("raw_block", &format, state.nodes.len()),
        content: NodeContent::RawBlock { format, content },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
}

fn process_head(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser) {
    let entries = extract_head_metadata_entries(tag, parser);
    if entries.is_empty() {
        return;
    }
    state.push(DocumentNode {
        id: make_node_id("metadata_block", "head", state.nodes.len()),
        content: NodeContent::MetadataBlock { entries },
        parent: None,
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
}

fn process_section_group(
    state: &mut BuilderState,
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
    depth: usize,
) {
    let label = tag
        .attributes()
        .get("aria-label")
        .flatten()
        .map(|value| value.as_utf8_str().to_string());
    let group_idx = state.push_attached(DocumentNode {
        id: make_node_id("group", tag_name, state.nodes.len()),
        content: NodeContent::Group {
            label,
            heading_level: None,
            heading_text: None,
        },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: collect_attributes(tag),
    });
    walk_children(state, tag, parser, Some(group_idx), depth);
}

fn walk_children(
    state: &mut BuilderState,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
    depth: usize,
) {
    for child_handle in tag.children().top().iter() {
        walk(state, child_handle, parser, parent_idx, depth + 1);
    }
}

fn process_code_block(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser, parent_idx: Option<u32>) {
    let (text, language) = find_code_child(tag, parser).unwrap_or_else(|| (extract_text(tag, parser), None));
    state.push_attached(DocumentNode {
        id: make_node_id("code", &text, state.nodes.len()),
        content: NodeContent::Code { text, language },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
}

fn find_code_child(tag: &tl::HTMLTag, parser: &tl::Parser) -> Option<(String, Option<String>)> {
    for child_handle in tag.children().top().iter() {
        let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) else {
            continue;
        };
        if !child_tag.name().as_utf8_str().eq_ignore_ascii_case("code") {
            continue;
        }
        let language = child_tag.attributes().get("class").flatten().and_then(|value| {
            value
                .as_utf8_str()
                .split_whitespace()
                .find_map(|token| token.strip_prefix("language-").map(str::to_string))
        });
        return Some((extract_text(child_tag, parser), language));
    }
    None
}

fn process_quote(
    state: &mut BuilderState,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
    depth: usize,
) {
    let quote_idx = state.push_attached(DocumentNode {
        id: make_node_id("quote", "blockquote", state.nodes.len()),
        content: NodeContent::Quote,
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
    walk_children(state, tag, parser, Some(quote_idx), depth);
}

fn process_definition_list(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser, parent_idx: Option<u32>) {
    let list_idx = state.push_attached(DocumentNode {
        id: make_node_id("definition_list", "dl", state.nodes.len()),
        content: NodeContent::DefinitionList,
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
    for (term, definition) in collect_definition_items(tag, parser) {
        state.push_attached(DocumentNode {
            id: make_node_id("definition_item", &term, state.nodes.len()),
            content: NodeContent::DefinitionItem { term, definition },
            parent: Some(list_idx),
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
    }
}

fn process_paragraph(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser, parent_idx: Option<u32>) {
    let (text, annotations) = collect_annotated_text(tag, parser);
    if text.is_empty() {
        return;
    }
    state.push_attached(DocumentNode {
        id: make_node_id("paragraph", &text, state.nodes.len()),
        content: NodeContent::Paragraph { text },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations,
        attributes: None,
    });
}

fn process_list(
    state: &mut BuilderState,
    ordered: bool,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
    depth: usize,
) {
    let label = if ordered { "ordered" } else { "unordered" };
    let list_idx = state.push_attached(DocumentNode {
        id: make_node_id("list", label, state.nodes.len()),
        content: NodeContent::List { ordered },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
    walk_children(state, tag, parser, Some(list_idx), depth);
}

fn process_list_item(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser, parent_idx: Option<u32>) {
    let (text, annotations) = collect_annotated_text(tag, parser);
    if text.is_empty() {
        return;
    }
    state.push_attached(DocumentNode {
        id: make_node_id("list_item", &text, state.nodes.len()),
        content: NodeContent::ListItem { text },
        parent: parent_idx.or_else(|| state.current_group()),
        children: Vec::new(),
        annotations,
        attributes: None,
    });
}

fn process_table(state: &mut BuilderState, tag: &tl::HTMLTag, parser: &tl::Parser, parent_idx: Option<u32>) {
    let grid = extract_table_grid(tag, parser);
    state.push_attached(DocumentNode {
        id: make_node_id("table", &grid.rows.to_string(), state.nodes.len()),
        content: NodeContent::Table { grid },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
}

fn process_image(state: &mut BuilderState, tag: &tl::HTMLTag, parent_idx: Option<u32>) {
    let src = tag
        .attributes()
        .get("src")
        .flatten()
        .map(|value| value.as_utf8_str().to_string());
    let description = tag
        .attributes()
        .get("alt")
        .flatten()
        .map(|value| value.as_utf8_str().to_string())
        .filter(|value| !value.is_empty());
    state.push_attached(DocumentNode {
        id: make_node_id("image", src.as_deref().unwrap_or("img"), state.nodes.len()),
        content: NodeContent::Image {
            description,
            src,
            image_index: None,
        },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
}

fn process_heading(
    state: &mut BuilderState,
    tag_name: &str,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    parent_idx: Option<u32>,
) {
    let level = tag_name[1..].parse::<u8>().unwrap_or(1);
    let (text, annotations) = collect_annotated_text(tag, parser);
    if text.is_empty() {
        return;
    }
    while state
        .group_stack
        .last()
        .is_some_and(|(open_level, _)| *open_level >= level)
    {
        state.group_stack.pop();
    }

    let group_idx = state.push_attached(DocumentNode {
        id: make_node_id("group", &text, state.nodes.len()),
        content: NodeContent::Group {
            label: Some(text.clone()),
            heading_level: Some(level),
            heading_text: Some(text.clone()),
        },
        parent: state.current_group().or(parent_idx),
        children: Vec::new(),
        annotations: Vec::new(),
        attributes: None,
    });
    state.group_stack.push((level, group_idx));

    state.push_attached(DocumentNode {
        id: make_node_id("heading", &text, state.nodes.len()),
        content: NodeContent::Heading { level, text },
        parent: Some(group_idx),
        children: Vec::new(),
        annotations,
        attributes: None,
    });
}

/// Collect a safe subset of attributes into a `HashMap`.
///
/// Only `id`, `class`, `lang`, `dir`, and `data-*` attributes are kept.
/// Event handlers (`on*`) and other potentially unsafe attributes are dropped.
fn collect_attributes(tag: &tl::HTMLTag) -> Option<HashMap<String, String>> {
    let raw = tag.attributes().clone();
    let mut map: HashMap<String, String> = HashMap::new();

    for (key_cow, val_opt) in raw.iter() {
        let key = key_cow.to_ascii_lowercase();
        if key.starts_with("on") {
            continue;
        }
        if matches!(key.as_str(), "id" | "class" | "lang" | "dir") || key.starts_with("data-") {
            if let Some(val) = val_opt {
                map.insert(key, val.to_string());
            }
        }
    }

    if map.is_empty() { None } else { Some(map) }
}
