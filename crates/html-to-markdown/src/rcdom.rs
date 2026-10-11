// ~keep Vendored from markup5ever_rcdom v0.36.0+unofficial
// ~keep Original source: https://github.com/servo/html5ever (rcdom/)
// ~keep Copyright (c) 2014 The html5ever Project Developers
// ~keep Licensed under MIT OR Apache-2.0 (see ATTRIBUTIONS.md)
// ~keep
// ~keep Vendored to:
// ~keep - Remove unused xml5ever transitive dependency
// ~keep - Eliminate pinned external dependency on "+unofficial" crate
// ~keep - Gain full control over this small, critical module
// ~keep
// ~keep Changes from upstream:
// ~keep - Replaced `extern crate markup5ever` / `extern crate tendril` with
// ~keep   `use` imports through `html5ever` (edition 2024 compatibility)
// ~keep - Added module-level clippy allows for vendored code style

// ~keep reason: this is vendored upstream code (markup5ever_rcdom v0.36.0) reproduced verbatim
// ~keep to eliminate the pinned "+unofficial" crate dependency. The original code uses panics
// ~keep and expect() calls as part of its internal invariant enforcement. All lints are
// ~keep suppressed on this module to keep the diff from upstream minimal and make future
// ~keep upstream syncs straightforward.
#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::module_name_repetitions,
    clippy::redundant_else,
    clippy::match_wildcard_for_single_variants,
    clippy::similar_names,
    clippy::items_after_statements,
    clippy::use_self,
    clippy::missing_fields_in_debug,
    clippy::semicolon_if_nothing_returned,
    missing_docs
)]

//! A simple reference-counted DOM.
//!
//! This is sufficient as a static parse tree, but don't build a
//! web browser using it. :)

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::default::Default;
use std::fmt;
use std::io;
use std::mem;
use std::rc::{Rc, Weak};

use html5ever::tendril::StrTendril;

use html5ever::Attribute;
use html5ever::ExpandedName;
use html5ever::QualName;
use html5ever::interface::tree_builder;
use html5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::serialize::{Serialize, Serializer};

/// The different kinds of nodes in the DOM.
#[derive(Debug)]
pub enum NodeData {
    /// The `Document` itself - the root node of a HTML document.
    Document,

    /// A `DOCTYPE` with name, public id, and system id. See
    /// [document type declaration on wikipedia][dtd wiki].
    ///
    /// [dtd wiki]: https://en.wikipedia.org/wiki/Document_type_declaration
    Doctype {
        name: StrTendril,
        // ~keep Fields required by html5ever's DOM model; not accessed during conversion.
        #[allow(dead_code)]
        public_id: StrTendril,
        #[allow(dead_code)]
        system_id: StrTendril,
    },

    /// A text node.
    Text { contents: RefCell<StrTendril> },

    /// A comment.
    Comment { contents: StrTendril },

    /// An element with attributes.
    Element {
        name: QualName,
        attrs: RefCell<Vec<Attribute>>,

        /// For HTML \<template\> elements, the [template contents].
        ///
        /// [template contents]: https://html.spec.whatwg.org/multipage/#template-contents
        template_contents: RefCell<Option<Handle>>,

        /// Whether the node is a [HTML integration point].
        ///
        /// [HTML integration point]: https://html.spec.whatwg.org/multipage/#html-integration-point
        mathml_annotation_xml_integration_point: bool,
    },

    /// A Processing instruction.
    ProcessingInstruction { target: StrTendril, contents: StrTendril },
}

/// A DOM node.
pub struct Node {
    /// Parent node.
    pub parent: Cell<Option<WeakHandle>>,
    /// The `<template>` element whose contents this node is. The contents are not a child of
    /// the template and have no parent; only the depth count of the repair reads this link.
    template_host: Cell<Option<WeakHandle>>,
    /// Child nodes of this node.
    pub children: RefCell<Vec<Handle>>,
    /// Represents this node's data.
    pub data: NodeData,
}

impl Node {
    /// Create a new node from its contents
    pub fn new(data: NodeData) -> Rc<Self> {
        Rc::new(Node {
            data,
            parent: Cell::new(None),
            template_host: Cell::new(None),
            children: RefCell::new(Vec::new()),
        })
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let mut nodes = mem::take(&mut *self.children.borrow_mut());
        while let Some(node) = nodes.pop() {
            let children = mem::take(&mut *node.children.borrow_mut());
            nodes.extend(children);
            if let NodeData::Element {
                ref template_contents, ..
            } = node.data
            {
                if let Some(template_contents) = template_contents.borrow_mut().take() {
                    nodes.push(template_contents);
                }
            }
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Node")
            .field("data", &self.data)
            .field("children", &self.children)
            .finish()
    }
}

/// Reference to a DOM node.
pub type Handle = Rc<Node>;

/// Weak reference to a DOM node, used for parent pointers.
pub type WeakHandle = Weak<Node>;

/// Append a parentless node to another nodes' children
fn append(new_parent: &Handle, child: Handle) {
    let previous_parent = child.parent.replace(Some(Rc::downgrade(new_parent)));
    assert!(previous_parent.is_none());
    new_parent.children.borrow_mut().push(child);
}

/// The node one level above `node` for the depth count of the repair: its parent, or the
/// template when `node` is the contents of one.
fn depth_ancestor(node: &Handle) -> Option<Handle> {
    let read = |link: &Cell<Option<WeakHandle>>| {
        let weak = link.take();
        let handle = weak.as_ref().and_then(Weak::upgrade);
        link.set(weak);
        handle
    };
    read(&node.parent).or_else(|| read(&node.template_host))
}

/// If the node has a parent, get it and this node's position in its children
fn get_parent_and_index(target: &Handle) -> Option<(Handle, usize)> {
    if let Some(weak) = target.parent.take() {
        let parent = weak.upgrade().expect("dangling weak pointer");
        target.parent.set(Some(weak));
        let i = match parent
            .children
            .borrow()
            .iter()
            .enumerate()
            .find(|&(_, child)| Rc::ptr_eq(child, target))
        {
            Some((i, _)) => i,
            None => panic!("have parent but couldn't find in parent's children!"),
        };
        Some((parent, i))
    } else {
        None
    }
}

fn append_to_existing_text(prev: &Handle, text: &str) -> bool {
    match prev.data {
        NodeData::Text { ref contents } => {
            contents.borrow_mut().push_slice(text);
            true
        }
        _ => false,
    }
}

fn remove_from_parent(target: &Handle) {
    if let Some((parent, i)) = get_parent_and_index(target) {
        parent.children.borrow_mut().remove(i);
        target.parent.set(None);
    }
}

/// The DOM itself; the result of parsing.
pub struct RcDom {
    /// The `Document` itself.
    pub document: Handle,

    /// Errors that occurred during parsing.
    pub errors: RefCell<Vec<Cow<'static, str>>>,

    /// The document's quirks mode.
    pub quirks_mode: Cell<QuirksMode>,
    node_count: Cell<usize>,
    max_nodes: usize,
    max_depth: usize,
    pub(crate) repair_limit: Cell<Option<crate::types::WarningKind>>,
}

impl RcDom {
    pub(crate) fn bounded(max_nodes: usize, max_depth: usize) -> Self {
        Self {
            max_nodes,
            max_depth,
            ..Self::default()
        }
    }

    fn new_node(&self, data: NodeData) -> Handle {
        self.node_count.set(self.node_count.get().saturating_add(1));
        if self.node_count.get() > self.max_nodes {
            self.repair_limit.set(Some(crate::types::WarningKind::TruncatedInput));
        }
        Node::new(data)
    }

    fn subtree_fits_depth(&self, parent: &Handle, children: &[Handle]) -> bool {
        if self.max_depth == usize::MAX || children.is_empty() {
            return true;
        }
        let mut depth = 0;
        let mut current = Rc::clone(parent);
        while let Some(ancestor) = depth_ancestor(&current) {
            depth += 1;
            if depth > self.max_depth {
                self.repair_limit
                    .set(Some(crate::types::WarningKind::DepthLimitExceeded));
                return false;
            }
            current = ancestor;
        }
        let mut work: Vec<_> = children.iter().map(|child| (Rc::clone(child), depth + 1)).collect();
        while let Some((node, depth)) = work.pop() {
            if depth > self.max_depth {
                self.repair_limit
                    .set(Some(crate::types::WarningKind::DepthLimitExceeded));
                return false;
            }
            work.extend(node.children.borrow().iter().map(|child| (Rc::clone(child), depth + 1)));
        }
        true
    }

    fn observe_depth(&self, parent: &Handle) {
        // ~keep Adoption-agency reparenting changes descendant depths; reading actual parents
        // ~keep avoids stale cached depths. The walk stops at the fixed repair ceiling (#808).
        if self.max_depth == usize::MAX {
            return;
        }
        let mut current = Rc::clone(parent);
        for _ in 0..self.max_depth {
            let Some(ancestor) = depth_ancestor(&current) else {
                return;
            };
            current = ancestor;
        }
        self.repair_limit
            .set(Some(crate::types::WarningKind::DepthLimitExceeded));
    }
}

impl TreeSink for RcDom {
    type Output = Self;
    fn finish(self) -> Self {
        self
    }

    type Handle = Handle;

    type ElemName<'a>
        = ExpandedName<'a>
    where
        Self: 'a;

    fn parse_error(&self, msg: Cow<'static, str>) {
        self.errors.borrow_mut().push(msg);
    }

    fn get_document(&self) -> Handle {
        self.document.clone()
    }

    fn get_template_contents(&self, target: &Handle) -> Handle {
        if let NodeData::Element {
            ref template_contents, ..
        } = target.data
        {
            template_contents
                .borrow()
                .as_ref()
                .expect("not a template element!")
                .clone()
        } else {
            panic!("not a template element!")
        }
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn same_node(&self, x: &Handle, y: &Handle) -> bool {
        Rc::ptr_eq(x, y)
    }

    fn elem_name<'a>(&self, target: &'a Handle) -> ExpandedName<'a> {
        match target.data {
            NodeData::Element { ref name, .. } => name.expanded(),
            _ => panic!("not an element!"),
        }
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, flags: ElementFlags) -> Handle {
        let contents = flags.template.then(|| self.new_node(NodeData::Document));
        let element = self.new_node(NodeData::Element {
            name,
            attrs: RefCell::new(attrs),
            template_contents: RefCell::new(contents.clone()),
            mathml_annotation_xml_integration_point: flags.mathml_annotation_xml_integration_point,
        });
        // ~keep The contents of a template are not its children, so the depth limit would
        // ~keep count each template from zero. The contents name the template as their host,
        // ~keep and the count goes on through nested templates (issue #808). The contents keep
        // ~keep no parent: a node in them is not in the document (the `<base>` lookup reads that).
        if let Some(contents) = contents {
            contents.template_host.set(Some(Rc::downgrade(&element)));
        }
        element
    }

    fn create_comment(&self, text: StrTendril) -> Handle {
        self.new_node(NodeData::Comment { contents: text })
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Handle {
        self.new_node(NodeData::ProcessingInstruction { target, contents: data })
    }

    fn append(&self, parent: &Handle, child: NodeOrText<Handle>) {
        self.observe_depth(parent);
        if let NodeOrText::AppendNode(node) = &child {
            if !node.children.borrow().is_empty() && !self.subtree_fits_depth(parent, std::slice::from_ref(node)) {
                return;
            }
        }
        if let NodeOrText::AppendText(text) = &child {
            if let Some(h) = parent.children.borrow().last() {
                if append_to_existing_text(h, text) {
                    return;
                }
            }
        }

        append(
            parent,
            match child {
                NodeOrText::AppendText(text) => self.new_node(NodeData::Text {
                    contents: RefCell::new(text),
                }),
                NodeOrText::AppendNode(node) => node,
            },
        );
    }

    fn append_before_sibling(&self, sibling: &Handle, child: NodeOrText<Handle>) {
        let (parent, i) = get_parent_and_index(sibling).expect("append_before_sibling called on node without parent");

        self.observe_depth(&parent);
        if let NodeOrText::AppendNode(node) = &child {
            if !node.children.borrow().is_empty() && !self.subtree_fits_depth(&parent, std::slice::from_ref(node)) {
                return;
            }
        }
        let child = match (child, i) {
            (NodeOrText::AppendText(text), 0) => self.new_node(NodeData::Text {
                contents: RefCell::new(text),
            }),

            (NodeOrText::AppendText(text), i) => {
                let children = parent.children.borrow();
                let prev = &children[i - 1];
                if append_to_existing_text(prev, &text) {
                    return;
                }
                self.new_node(NodeData::Text {
                    contents: RefCell::new(text),
                })
            }

            (NodeOrText::AppendNode(node), _) => node,
        };

        remove_from_parent(&child);

        child.parent.set(Some(Rc::downgrade(&parent)));
        parent.children.borrow_mut().insert(i, child);
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let parent = element.parent.take();
        let has_parent = parent.is_some();
        element.parent.set(parent);

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(&self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        append(
            &self.document,
            self.new_node(NodeData::Doctype {
                name,
                public_id,
                system_id,
            }),
        );
    }

    fn add_attrs_if_missing(&self, target: &Handle, attrs: Vec<Attribute>) {
        let mut existing = if let NodeData::Element { ref attrs, .. } = target.data {
            attrs.borrow_mut()
        } else {
            panic!("not an element")
        };

        let existing_names = existing.iter().map(|e| e.name.clone()).collect::<HashSet<_>>();
        existing.extend(attrs.into_iter().filter(|attr| !existing_names.contains(&attr.name)));
    }

    fn remove_from_parent(&self, target: &Handle) {
        remove_from_parent(target);
    }

    fn reparent_children(&self, node: &Handle, new_parent: &Handle) {
        self.observe_depth(new_parent);
        // ~keep Moving a subtree can exceed the limit even when its new parent's own depth is safe.
        if !self.subtree_fits_depth(new_parent, &node.children.borrow()) {
            return;
        }
        let mut children = node.children.borrow_mut();
        let mut new_children = new_parent.children.borrow_mut();
        for child in children.iter() {
            let previous_parent = child.parent.replace(Some(Rc::downgrade(new_parent)));
            assert!(Rc::ptr_eq(
                node,
                &previous_parent
                    .expect("invariant: child must have a parent during reparenting")
                    .upgrade()
                    .expect("dangling weak")
            ))
        }
        new_children.extend(mem::take(&mut *children));
    }

    fn is_mathml_annotation_xml_integration_point(&self, target: &Handle) -> bool {
        if let NodeData::Element {
            mathml_annotation_xml_integration_point,
            ..
        } = target.data
        {
            mathml_annotation_xml_integration_point
        } else {
            panic!("not an element!")
        }
    }
}

impl Default for RcDom {
    fn default() -> RcDom {
        RcDom {
            document: Node::new(NodeData::Document),
            errors: Default::default(),
            quirks_mode: Cell::new(tree_builder::NoQuirks),
            node_count: Cell::new(1),
            max_nodes: usize::MAX,
            max_depth: usize::MAX,
            repair_limit: Cell::new(None),
        }
    }
}

/// True for a `pre`, a `textarea` or a `listing` whose text starts with a line feed.
///
/// ~keep An HTML parser drops a line feed that directly follows the start tag of these three
/// ~keep elements, so in this tree the line feed is the second one of the source. The page that
/// ~keep is written back gets the first one again. Without it the next parse drops a line of
/// ~keep the text (issue #850).
fn starts_with_line_feed_of_text(name: &QualName, element: &Handle) -> bool {
    &*name.ns == "http://www.w3.org/1999/xhtml"
        && matches!(&*name.local, "pre" | "textarea" | "listing")
        && element.children.borrow().first().is_some_and(
            |child| matches!(&child.data, NodeData::Text { contents } if contents.borrow().starts_with('\n')),
        )
}

enum SerializeOp {
    Open(Handle),
    Close(QualName),
}

pub struct SerializableHandle(Handle);

impl From<Handle> for SerializableHandle {
    fn from(h: Handle) -> SerializableHandle {
        SerializableHandle(h)
    }
}

impl Serialize for SerializableHandle {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        let mut ops = VecDeque::new();
        match traversal_scope {
            IncludeNode => ops.push_back(SerializeOp::Open(self.0.clone())),
            ChildrenOnly(_) => ops.extend(self.0.children.borrow().iter().map(|h| SerializeOp::Open(h.clone()))),
        }

        while let Some(op) = ops.pop_front() {
            match op {
                SerializeOp::Open(handle) => match handle.data {
                    NodeData::Element {
                        ref name, ref attrs, ..
                    } => {
                        serializer
                            .start_elem(name.clone(), attrs.borrow().iter().map(|at| (&at.name, &at.value[..])))?;
                        if starts_with_line_feed_of_text(name, &handle) {
                            serializer.write_text("\n")?;
                        }

                        ops.reserve(1 + handle.children.borrow().len());
                        ops.push_front(SerializeOp::Close(name.clone()));

                        for child in handle.children.borrow().iter().rev() {
                            ops.push_front(SerializeOp::Open(child.clone()));
                        }
                    }

                    NodeData::Doctype { ref name, .. } => serializer.write_doctype(name)?,

                    NodeData::Text { ref contents } => serializer.write_text(&contents.borrow())?,

                    NodeData::Comment { ref contents } => serializer.write_comment(contents)?,

                    NodeData::ProcessingInstruction {
                        ref target,
                        ref contents,
                    } => serializer.write_processing_instruction(target, contents)?,

                    NodeData::Document => panic!("Can't serialize Document node itself"),
                },

                SerializeOp::Close(name) => {
                    serializer.end_elem(name)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod bounded_reparent_tests {
    use super::*;
    use html5ever::{local_name, ns};

    #[test]
    fn bounded_repair_should_detect_descendant_depth_when_reparenting() {
        let dom = RcDom::bounded(100, 4);
        let element = || {
            dom.create_element(
                QualName::new(None, ns!(html), local_name!(div)),
                Vec::new(),
                ElementFlags::default(),
            )
        };
        let mut branch = dom.document.clone();
        let mut first = None;
        for _ in 0..4 {
            let child = element();
            dom.append(&branch, NodeOrText::AppendNode(child.clone()));
            first.get_or_insert_with(|| child.clone());
            branch = child;
        }
        let destination = element();
        dom.append(&dom.document, NodeOrText::AppendNode(destination.clone()));
        let nested_destination = element();
        dom.append(&destination, NodeOrText::AppendNode(nested_destination.clone()));
        assert_eq!(dom.repair_limit.get(), None);
        dom.reparent_children(&first.expect("source branch"), &nested_destination);
        assert_eq!(
            dom.repair_limit.get(),
            Some(crate::types::WarningKind::DepthLimitExceeded)
        );
    }
}
