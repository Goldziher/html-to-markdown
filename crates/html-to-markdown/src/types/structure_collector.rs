//! Collector that builds a [`DocumentStructure`] during the converter's HTML DOM walk.
//!
//! Follows the same single-pass collector pattern used by [`crate::metadata::MetadataCollector`]:
//! an `Rc<RefCell<StructureCollector>>` handle is threaded through the [`crate::converter::Context`]
//! and individual element handlers call `push_*` methods as they encounter content.
//!
//! # Design
//!
//! - **Flat node array** with index-based parent/child links (matches [`DocumentStructure`]).
//! - **`section_stack`** tracks the currently-open heading groups (`(level, group_node_index)`).
//! - **`container_stack`** tracks open blockquote containers.
//! - **`list_stack`** tracks open list containers so `push_list_item` attaches items to the right list.
//! - IDs are deterministic hashes of `(node_type, text_prefix, index)`.

use std::cell::RefCell;
use std::rc::Rc;

use super::document::{DocumentNode, DocumentStructure, NodeContent, TextAnnotation};
use super::tables::{TableData, TableGrid};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextCaptureKind {
    Heading,
    Paragraph,
    ListItem,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextCaptureId(u64);

struct OpenAnnotation {
    token: u64,
    start: usize,
    kind: super::document::AnnotationKind,
}

struct TextCapture {
    id: TextCaptureId,
    kind: TextCaptureKind,
    text: String,
    annotations: Vec<TextAnnotation>,
    open_annotations: Vec<OpenAnnotation>,
    suspended: bool,
}

struct CaptureSnapshot {
    id: TextCaptureId,
    text_len: usize,
    annotations_len: usize,
    open_annotations_len: usize,
    suspended: bool,
}

enum ElementCaptureResult {
    Keep,
    Replace(Option<String>),
}

struct ElementCapture {
    snapshots: Vec<CaptureSnapshot>,
    annotation_token: Option<u64>,
    result: ElementCaptureResult,
    nodes_len: usize,
    section_stack: Vec<(u8, u32)>,
    container_stack: Vec<u32>,
    list_stack: Vec<u32>,
    tables_len: usize,
}

/// Shared mutable handle used in [`crate::converter::Context`].
pub type StructureCollectorHandle = Rc<RefCell<StructureCollector>>;

/// Incremental builder for [`DocumentStructure`] during a single DOM walk.
pub struct StructureCollector {
    /// Accumulated nodes in document order.
    nodes: Vec<DocumentNode>,
    /// Open heading-group stack: `(heading_level, node_index)`.
    /// Mirrors the `group_stack` in `structure_builder`.
    section_stack: Vec<(u8, u32)>,
    /// Open blockquote container indices (innermost last).
    container_stack: Vec<u32>,
    /// Open list container indices (innermost last).
    list_stack: Vec<u32>,
    /// Extracted tables with both structured grid data and markdown rendering.
    ///
    /// Populated by [`push_table_data`] when document structure extraction is enabled.
    tables: Vec<TableData>,
    text_captures: Vec<TextCapture>,
    element_captures: Vec<ElementCapture>,
    next_capture_id: u64,
    next_annotation_token: u64,
}

impl StructureCollector {
    /// Create a new empty collector ready to accumulate nodes for a single conversion pass.
    ///
    /// The intended lifecycle is:
    ///
    /// 1. Create one `StructureCollector` (or wrap it in a [`StructureCollectorHandle`]) at the
    ///    start of a conversion.
    /// 2. Thread the handle through the converter context; individual element handlers call the
    ///    `push_*` methods as they encounter headings, paragraphs, lists, tables, and so on.
    /// 3. Call [`Self::finish`] exactly once to consume the collector and obtain the completed
    ///    [`super::document::DocumentStructure`] together with the flat list of extracted
    ///    [`super::tables::TableData`] entries.
    ///
    /// `StructureCollector` is not `Send` (it uses `Rc<RefCell<…>>` handles internally) and must
    /// not be shared across threads. Create a fresh collector per conversion.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            section_stack: Vec::new(),
            container_stack: Vec::new(),
            list_stack: Vec::new(),
            tables: Vec::new(),
            text_captures: Vec::new(),
            element_captures: Vec::new(),
            next_capture_id: 0,
            next_annotation_token: 0,
        }
    }

    pub(crate) fn begin_text_capture(&mut self, kind: TextCaptureKind) -> TextCaptureId {
        let id = TextCaptureId(self.next_capture_id);
        self.next_capture_id = self.next_capture_id.wrapping_add(1);
        self.text_captures.push(TextCapture {
            id,
            kind,
            text: String::new(),
            annotations: Vec::new(),
            open_annotations: Vec::new(),
            suspended: false,
        });
        id
    }

    pub(crate) fn finish_text_capture(&mut self, id: TextCaptureId) -> (String, Vec<TextAnnotation>) {
        let Some(position) = self.text_captures.iter().position(|capture| capture.id == id) else {
            return (String::new(), Vec::new());
        };
        let capture = self.text_captures.remove(position);
        super::structure_builder::trim_annotated_text(&capture.text, capture.annotations)
    }

    pub(crate) fn append_text(&mut self, text: &str) {
        for capture in self.text_captures.iter_mut().filter(|capture| !capture.suspended) {
            capture.text.push_str(text);
        }
    }

    pub(crate) fn has_active_text_capture(&self) -> bool {
        self.text_captures.iter().any(|capture| !capture.suspended)
    }

    pub(crate) fn begin_element(&mut self, kind: Option<super::document::AnnotationKind>) {
        let snapshots = self
            .text_captures
            .iter()
            .map(|capture| CaptureSnapshot {
                id: capture.id,
                text_len: capture.text.len(),
                annotations_len: capture.annotations.len(),
                open_annotations_len: capture.open_annotations.len(),
                suspended: capture.suspended,
            })
            .collect();
        let annotation_token = kind.map(|kind| self.begin_annotation(kind));
        self.element_captures.push(ElementCapture {
            snapshots,
            annotation_token,
            result: ElementCaptureResult::Keep,
            nodes_len: self.nodes.len(),
            section_stack: self.section_stack.clone(),
            container_stack: self.container_stack.clone(),
            list_stack: self.list_stack.clone(),
            tables_len: self.tables.len(),
        });
    }

    pub(crate) fn replace_current_element(&mut self, replacement: Option<&str>) {
        if let Some(element) = self.element_captures.last_mut() {
            element.result = ElementCaptureResult::Replace(replacement.map(str::to_string));
        }
    }

    pub(crate) fn finish_element(&mut self) {
        let Some(element) = self.element_captures.pop() else {
            return;
        };
        match &element.result {
            ElementCaptureResult::Keep => {
                if let Some(token) = element.annotation_token {
                    self.finish_annotation(token);
                }
            }
            ElementCaptureResult::Replace(replacement) => {
                self.restore_captures(&element.snapshots);
                self.restore_structure(&element);
                if let Some(replacement) = replacement.as_deref() {
                    self.append_text(replacement);
                }
            }
        }
    }

    pub(crate) fn suspend_list_item_captures(&mut self) -> Vec<TextCaptureId> {
        let mut suspended = Vec::new();
        for capture in self
            .text_captures
            .iter_mut()
            .filter(|capture| capture.kind == TextCaptureKind::ListItem && !capture.suspended)
        {
            capture.suspended = true;
            suspended.push(capture.id);
        }
        suspended
    }

    pub(crate) fn resume_text_captures(&mut self, ids: &[TextCaptureId]) {
        for capture in &mut self.text_captures {
            if ids.contains(&capture.id) {
                capture.suspended = false;
            }
        }
    }

    /// Record a heading element.
    ///
    /// Creates a [`NodeContent::Group`] (which owns all subsequent sibling content until a
    /// heading of equal or higher rank closes it) followed by a [`NodeContent::Heading`] child.
    ///
    /// Returns the index of the **heading** node (the group node is one before it).
    pub fn push_heading(&mut self, level: u8, text: &str, id: Option<&str>) -> u32 {
        self.push_heading_with_annotations(level, text, id, Vec::new())
    }

    pub(crate) fn push_heading_with_annotations(
        &mut self,
        level: u8,
        text: &str,
        id: Option<&str>,
        annotations: Vec<TextAnnotation>,
    ) -> u32 {
        while let Some(&(open_level, _)) = self.section_stack.last() {
            if open_level >= level {
                self.section_stack.pop();
            } else {
                break;
            }
        }

        let group_parent = self.current_structural_parent();

        let group_id = Self::generate_id("group", text, self.nodes.len() as u32);
        let group_idx = self.raw_push(DocumentNode {
            id: group_id,
            content: NodeContent::Group {
                label: Some(text.to_string()),
                heading_level: Some(level),
                heading_text: Some(text.to_string()),
            },
            parent: group_parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(gp) = group_parent {
            self.add_child(gp, group_idx);
        }
        self.section_stack.push((level, group_idx));

        let heading_id = Self::generate_id("heading", text, self.nodes.len() as u32);
        let heading_idx = self.raw_push(DocumentNode {
            id: heading_id,
            content: NodeContent::Heading {
                level,
                text: text.to_string(),
            },
            parent: Some(group_idx),
            children: Vec::new(),
            annotations,
            attributes: id.map(|v| {
                let mut m = std::collections::HashMap::new();
                m.insert("id".to_string(), v.to_string());
                m
            }),
        });
        self.add_child(group_idx, heading_idx);
        heading_idx
    }

    /// Record a paragraph element.
    ///
    /// Returns the node index.
    pub fn push_paragraph(&mut self, text: &str) -> u32 {
        self.push_paragraph_with_annotations(text, Vec::new())
    }

    pub(crate) fn push_paragraph_with_annotations(&mut self, text: &str, annotations: Vec<TextAnnotation>) -> u32 {
        if text.is_empty() {
            return u32::MAX;
        }
        let parent = self.current_structural_parent();
        let id = Self::generate_id("paragraph", text, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Paragraph { text: text.to_string() },
            parent,
            children: Vec::new(),
            annotations,
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Open a list container.
    ///
    /// Returns the node index; call [`Self::push_list_end`] to close it.
    pub fn push_list_start(&mut self, ordered: bool) -> u32 {
        let parent = self.current_structural_parent();
        let label = if ordered { "ordered" } else { "unordered" };
        let id = Self::generate_id("list", label, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::List { ordered },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        self.list_stack.push(idx);
        idx
    }

    /// Close the innermost open list container.
    pub fn push_list_end(&mut self) {
        self.list_stack.pop();
    }

    /// Record a list item under the current open list.
    ///
    /// If there is no open list, the item is parented under the current section/container.
    /// Returns the node index.
    pub fn push_list_item(&mut self, text: &str) -> u32 {
        self.push_list_item_with_annotations(text, Vec::new())
    }

    pub(crate) fn push_list_item_with_annotations(&mut self, text: &str, annotations: Vec<TextAnnotation>) -> u32 {
        let parent = self
            .list_stack
            .last()
            .copied()
            .or_else(|| self.current_structural_parent());
        let id = Self::generate_id("list_item", text, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::ListItem { text: text.to_string() },
            parent,
            children: Vec::new(),
            annotations,
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Record a table with both structured grid data and its markdown rendering.
    ///
    /// Adds the table to the document tree as a [`NodeContent::Table`] node and also
    /// appends a [`TableData`] entry (grid + markdown) to the flat tables list that is
    /// exposed via [`crate::ConversionResult::tables`].
    ///
    /// Returns the node index.
    pub fn push_table_data(&mut self, grid: TableGrid, markdown: String) -> u32 {
        let parent = self.current_structural_parent();
        let label = grid.rows.to_string();
        let id = Self::generate_id("table", &label, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Table { grid: grid.clone() },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        self.tables.push(TableData { grid, markdown });
        idx
    }

    /// Record a table (grid only, no markdown rendering).
    ///
    /// Prefer [`Self::push_table_data`] when the markdown rendering is available; use this
    /// method only when the markdown is not yet computed.
    ///
    /// Returns the node index.
    pub fn push_table(&mut self, grid: TableGrid) -> u32 {
        let parent = self.current_structural_parent();
        let label = grid.rows.to_string();
        let id = Self::generate_id("table", &label, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Table { grid },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Record an image element.
    ///
    /// Returns the node index.
    pub fn push_image(&mut self, src: Option<&str>, alt: Option<&str>) -> u32 {
        let parent = self.current_structural_parent();
        let label = src.unwrap_or("img");
        let id = Self::generate_id("image", label, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Image {
                description: alt.filter(|s| !s.is_empty()).map(str::to_string),
                src: src.map(str::to_string),
                image_index: None,
            },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Record a code block.
    ///
    /// Returns the node index.
    pub fn push_code(&mut self, text: &str, language: Option<&str>) -> u32 {
        let parent = self.current_structural_parent();
        let id = Self::generate_id("code", text, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Code {
                text: text.to_string(),
                language: language.map(str::to_string),
            },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Open a blockquote container.
    ///
    /// Returns the node index; call [`Self::push_quote_end`] to close it.
    pub fn push_quote_start(&mut self) -> u32 {
        let parent = self.current_structural_parent();
        let id = Self::generate_id("quote", "blockquote", self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::Quote,
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        self.container_stack.push(idx);
        idx
    }

    /// Close the innermost open blockquote container.
    pub fn push_quote_end(&mut self) {
        self.container_stack.pop();
    }

    /// Record a raw block (e.g. preserved `<script>` or `<style>` content).
    ///
    /// Returns the node index.
    pub fn push_raw_block(&mut self, format: &str, content: &str) -> u32 {
        let parent = self.current_structural_parent();
        let id = Self::generate_id("raw_block", format, self.nodes.len() as u32);
        let idx = self.raw_push(DocumentNode {
            id,
            content: NodeContent::RawBlock {
                format: format.to_string(),
                content: content.to_string(),
            },
            parent,
            children: Vec::new(),
            annotations: Vec::new(),
            attributes: None,
        });
        if let Some(p) = parent {
            self.add_child(p, idx);
        }
        idx
    }

    /// Consume the collector and return the completed [`DocumentStructure`] and extracted
    /// [`TableData`] entries.
    ///
    /// Returns `(DocumentStructure, Vec<TableData>)`.  The tables vec contains one entry per
    /// `<table>` element that was processed via [`Self::push_table_data`].
    #[must_use]
    pub fn finish(self) -> (DocumentStructure, Vec<TableData>) {
        let doc = DocumentStructure {
            nodes: self.nodes,
            source_format: Some("html".to_string()),
        };
        (doc, self.tables)
    }

    fn begin_annotation(&mut self, kind: super::document::AnnotationKind) -> u64 {
        let token = self.next_annotation_token;
        self.next_annotation_token = self.next_annotation_token.wrapping_add(1);
        for capture in self.text_captures.iter_mut().filter(|capture| !capture.suspended) {
            capture.open_annotations.push(OpenAnnotation {
                token,
                start: capture.text.len(),
                kind: kind.clone(),
            });
        }
        token
    }

    fn finish_annotation(&mut self, token: u64) {
        for capture in self.text_captures.iter_mut().filter(|capture| !capture.suspended) {
            let Some(position) = capture
                .open_annotations
                .iter()
                .rposition(|annotation| annotation.token == token)
            else {
                continue;
            };
            let annotation = capture.open_annotations.remove(position);
            if annotation.start < capture.text.len() {
                capture.annotations.push(TextAnnotation {
                    start: annotation.start as u32,
                    end: capture.text.len() as u32,
                    kind: annotation.kind,
                });
            }
        }
    }

    fn restore_captures(&mut self, snapshots: &[CaptureSnapshot]) {
        let mut restored = Vec::with_capacity(snapshots.len());
        for snapshot in snapshots {
            let Some(position) = self.text_captures.iter().position(|capture| capture.id == snapshot.id) else {
                continue;
            };
            let mut capture = self.text_captures.remove(position);
            capture.text.truncate(snapshot.text_len);
            capture.annotations.truncate(snapshot.annotations_len);
            capture.open_annotations.truncate(snapshot.open_annotations_len);
            capture.suspended = snapshot.suspended;
            restored.push(capture);
        }
        self.text_captures = restored;
    }

    fn restore_structure(&mut self, element: &ElementCapture) {
        self.nodes.truncate(element.nodes_len);
        for node in &mut self.nodes {
            node.children.retain(|child| (*child as usize) < element.nodes_len);
        }
        self.section_stack.clone_from(&element.section_stack);
        self.container_stack.clone_from(&element.container_stack);
        self.list_stack.clone_from(&element.list_stack);
        self.tables.truncate(element.tables_len);
    }

    /// The effective structural parent for a new node:
    /// list stack > container stack > section stack > None.
    fn current_structural_parent(&self) -> Option<u32> {
        if let Some(&q) = self.container_stack.last() {
            return Some(q);
        }
        if let Some(&(_, g)) = self.section_stack.last() {
            return Some(g);
        }
        None
    }

    /// Append a node to the flat list and return its index.
    fn raw_push(&mut self, node: DocumentNode) -> u32 {
        let idx = self.nodes.len() as u32;
        self.nodes.push(node);
        idx
    }

    /// Record `child_idx` as a child of `parent_idx`.
    fn add_child(&mut self, parent_idx: u32, child_idx: u32) {
        if let Some(parent) = self.nodes.get_mut(parent_idx as usize) {
            parent.children.push(child_idx);
        }
    }

    /// Generate a deterministic node ID: `"{node_type}-{hash:016x}"`.
    ///
    /// Hashes `(node_type, text[..64], index)` with `DefaultHasher`.
    fn generate_id(node_type: &str, text: &str, index: u32) -> String {
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
}

impl Default for StructureCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_creates_group_and_heading() {
        let mut c = StructureCollector::new();
        let heading_idx = c.push_heading(1, "Title", None);
        assert_eq!(heading_idx, 1);
        assert_eq!(c.nodes.len(), 2);

        let group = &c.nodes[0];
        matches!(
            &group.content,
            NodeContent::Group {
                heading_level: Some(1),
                ..
            }
        );
        assert!(group.children.contains(&1));

        let heading = &c.nodes[1];
        assert!(matches!(&heading.content, NodeContent::Heading { level: 1, .. }));
        assert_eq!(heading.parent, Some(0));
    }

    #[test]
    fn test_heading_closes_deeper_groups() {
        let mut c = StructureCollector::new();
        c.push_heading(1, "H1", None);
        c.push_heading(2, "H2", None);
        c.push_heading(1, "H1b", None);
        assert_eq!(c.section_stack.len(), 1);
        let (level, _) = c.section_stack[0];
        assert_eq!(level, 1);
    }

    #[test]
    fn test_paragraph_parents_under_section() {
        let mut c = StructureCollector::new();
        c.push_heading(1, "Title", None);
        let p_idx = c.push_paragraph("Some text");
        let para = &c.nodes[p_idx as usize];
        assert_eq!(para.parent, Some(0));
    }

    #[test]
    fn test_list_items_attach_to_list() {
        let mut c = StructureCollector::new();
        let list_idx = c.push_list_start(false);
        let item_idx = c.push_list_item("Item 1");
        c.push_list_end();
        assert_eq!(c.nodes[item_idx as usize].parent, Some(list_idx));
        let list = &c.nodes[list_idx as usize];
        assert!(list.children.contains(&item_idx));
    }

    #[test]
    fn test_quote_container() {
        let mut c = StructureCollector::new();
        let q_idx = c.push_quote_start();
        let p_idx = c.push_paragraph("Quoted text");
        c.push_quote_end();
        assert_eq!(c.nodes[p_idx as usize].parent, Some(q_idx));
    }

    #[test]
    fn test_finish_returns_document_structure() {
        let mut c = StructureCollector::new();
        c.push_heading(1, "Title", None);
        c.push_paragraph("Text");
        let (doc, tables) = c.finish();
        assert_eq!(doc.source_format, Some("html".to_string()));
        assert_eq!(doc.nodes.len(), 3);
        assert!(tables.is_empty());
    }
}
