//! Anchor provenance across html5ever's tree repair.
//!
//! The adoption agency algorithm closes an `<a>` at a block boundary and *reconstructs* it
//! inside the block, so `<a href="/o"><div><a href="/i">Inner</a></div></a>` repairs to
//! `<a href="/o"></a><div><a href="/o"></a><a href="/i">Inner</a></div>` -- the same tree a
//! browser builds. Rendered faithfully that is two links to `/o`, one of them empty, which
//! is the duplication issue #493 reports. html5ever creates the clone through the same
//! `TreeSink::create_element` call as an authored element, with no flag, so the renderer
//! cannot tell them apart -- and a renderer-side rule keyed on shape either drops a genuine
//! empty anchor (`CommonMark` example 484) or a deliberately authored duplicate.
//!
//! The clone does carry one exact signal: it is created from the *original start-tag token's*
//! attributes. Stamping every `<a>` start tag with a private origin id before it reaches the
//! tree builder therefore marks the authored element and each of its clones with the same id,
//! and the split can be undone on the repaired tree before it is serialized. ~keep

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts, TreeSink};
use html5ever::{Attribute, LocalName, QualName, TokenizerResult, ns};

use crate::rcdom::{Handle, NodeData, RcDom};

/// The private attribute that carries an anchor's origin id through the tree builder.
const ORIGIN_ATTR: &str = "data-h2m-anchor-origin";

/// The deepest tree the HTML tree builder makes for a repair.
pub(super) const MAX_REPAIR_DEPTH: usize = 512;

/// Elements that make an anchor non-empty even without text.
const CONTENT_BEARING: [&str; 14] = [
    "img", "picture", "video", "audio", "svg", "canvas", "iframe", "object", "embed", "input", "math", "button",
    "textarea", "select",
];

/// Token sink that stamps each `<a>` start tag with a fresh origin id before forwarding it.
struct AnchorOriginStamper<S> {
    inner: S,
    next_origin: Cell<u32>,
}

impl<S> AnchorOriginStamper<S> {
    fn stamp(&self, tag: &mut Tag) {
        if tag.kind != TagKind::StartTag || &*tag.name != "a" {
            return;
        }
        // ~keep Input may already carry the private attribute; it must never survive as an
        // ~keep origin claim, so it is dropped before the genuine stamp is added.
        tag.attrs.retain(|attribute| &*attribute.name.local != ORIGIN_ATTR);
        let origin = self.next_origin.get();
        self.next_origin.set(origin.wrapping_add(1));
        tag.attrs.push(Attribute {
            name: QualName::new(None, ns!(), LocalName::from(ORIGIN_ATTR)),
            value: StrTendril::from(origin.to_string()),
        });
    }
}

impl TokenSink for AnchorOriginStamper<TreeBuilder<Handle, RcDom>> {
    type Handle = Handle;

    fn process_token(&self, mut token: Token, line_number: u64) -> TokenSinkResult<Self::Handle> {
        let is_tag = matches!(token, Token::TagToken(_));
        if let Token::TagToken(ref mut tag) = token {
            self.stamp(tag);
        }
        let result = self.inner.process_token(token, line_number);
        // ~keep The parse stops only after a tag the tree builder took, so the rest of the
        // ~keep input starts at a tag boundary and no tag is dropped. It does not stop after a
        // ~keep raw text start tag: the script or style text would become visible text.
        if is_tag
            && self.inner.sink.repair_limit.get().is_some()
            && !matches!(result, TokenSinkResult::RawData(_) | TokenSinkResult::Plaintext)
        {
            return TokenSinkResult::Script(Rc::clone(&self.inner.sink.document));
        }
        result
    }

    fn end(&self) {
        self.inner.end();
    }

    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        self.inner.adjusted_current_node_present_but_not_in_html_namespace()
    }
}

/// Parse `html` as a document with every `<a>` stamped with its origin id.
///
/// Drives the tokenizer by hand: `html5ever::parse_document` hard-codes
/// `Tokenizer<TreeBuilder<..>>` and leaves no room for a sink between the two. The input is
/// already a `str`, so nothing the `TendrilSink` driver adds (UTF-8 decoding) is lost.
///
/// Returns the tree and the input the parse did not read. That input is empty unless the
/// parse stopped at a limit (`RcDom::repair_limit`).
pub fn parse_with_anchor_origins(html: &str) -> (RcDom, String) {
    // ~keep Formatting reconstruction can create hundreds of clones per source tag (#809).
    // ~keep Both allocation and depth budgets apply to every HTML5 repair route (#808).
    const MAX_NODE_MULTIPLIER: usize = 8;
    const IMPLICIT_NODE_ALLOWANCE: usize = 16;
    let markup_count = html
        .as_bytes()
        .windows(2)
        .filter(|pair| pair[0] == b'<' && pair[1] != b'/')
        .count();
    let max_nodes = markup_count
        .saturating_mul(MAX_NODE_MULTIPLIER)
        .saturating_add(IMPLICIT_NODE_ALLOWANCE);
    let tree_builder = TreeBuilder::new(RcDom::bounded(max_nodes, MAX_REPAIR_DEPTH), TreeBuilderOpts::default());
    let tokenizer = Tokenizer::new(
        AnchorOriginStamper {
            inner: tree_builder,
            next_origin: Cell::new(0),
        },
        TokenizerOpts::default(),
    );
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(html));
    while tokenizer.sink.inner.sink.repair_limit.get().is_none() {
        if matches!(tokenizer.feed(&input), TokenizerResult::Done) {
            break;
        }
    }
    if tokenizer.sink.inner.sink.repair_limit.get().is_none() {
        tokenizer.end();
    } else {
        tokenizer.sink.inner.end();
    }
    let mut unparsed = String::new();
    while let Some(chunk) = input.pop_front() {
        unparsed.push_str(&chunk);
    }
    (TreeSink::finish(tokenizer.sink.inner.sink), unparsed)
}

/// Undo the adoption agency's split of an anchor around a block, then strip every origin stamp.
///
/// An origin that the repair left as more than one `<a>` keeps only the members with content
/// of their own; the rest are unwrapped in place so their children (whitespace, an empty
/// `<span>`) stay where they were. When no member has content the first -- the authored one --
/// is kept, so `<a href="/o"><div>…</div></a>` still records its destination once as `[](/o)`,
/// exactly as an authored `<a href></a>` renders. An origin with a single member is untouched.
pub fn collapse_split_anchors(document: &Handle) {
    let anchors = collect_stamped_anchors(document);
    let mut groups: BTreeMap<u32, Vec<Handle>> = BTreeMap::new();
    for (origin, handle) in &anchors {
        groups.entry(*origin).or_default().push(Rc::clone(handle));
    }
    for members in groups.values().filter(|members| members.len() > 1) {
        collapse_group(members);
    }
    for (_, handle) in &anchors {
        strip_origin_attr(handle);
    }
}

/// Every stamped `<a>` under `document`, in document order.
fn collect_stamped_anchors(document: &Handle) -> Vec<(u32, Handle)> {
    let mut anchors = Vec::new();
    let mut stack = vec![Rc::clone(document)];
    while let Some(node) = stack.pop() {
        if let Some(origin) = origin_of(&node) {
            anchors.push((origin, Rc::clone(&node)));
        }
        let children = node.children.borrow();
        stack.extend(children.iter().rev().cloned());
    }
    anchors
}

/// The origin id stamped on `node`, if it is a stamped `<a>`.
fn origin_of(node: &Handle) -> Option<u32> {
    let NodeData::Element { name, attrs, .. } = &node.data else {
        return None;
    };
    if &*name.local != "a" {
        return None;
    }
    attrs
        .borrow()
        .iter()
        .find(|attribute| &*attribute.name.local == ORIGIN_ATTR)
        .and_then(|attribute| attribute.value.parse().ok())
}

/// Unwrap every member of a split origin that has no content of its own, keeping the first
/// member when none has.
fn collapse_group(members: &[Handle]) {
    let empty: Vec<bool> = members.iter().map(|member| !has_own_content(member)).collect();
    let keep_first = empty.iter().all(|is_empty| *is_empty);
    for (index, member) in members.iter().enumerate() {
        if empty[index] && !(keep_first && index == 0) {
            unwrap_element(member);
        }
    }
}

/// Whether `anchor` holds non-whitespace text or a content-bearing element of its own.
///
/// Descendant `<a>` subtrees are not searched: whatever a nested anchor holds is its own
/// content, and must not keep an otherwise empty outer half alive. ~keep
fn has_own_content(anchor: &Handle) -> bool {
    let mut stack: Vec<Handle> = anchor.children.borrow().iter().cloned().collect();
    while let Some(node) = stack.pop() {
        match &node.data {
            NodeData::Text { contents } => {
                if !contents.borrow().trim().is_empty() {
                    return true;
                }
            }
            NodeData::Element { name, .. } => {
                let local: &str = &name.local;
                if local == "a" {
                    continue;
                }
                if CONTENT_BEARING.contains(&local) {
                    return true;
                }
                stack.extend(node.children.borrow().iter().cloned());
            }
            _ => {}
        }
    }
    false
}

/// Replace `node` in its parent with its own children, in place.
fn unwrap_element(node: &Handle) {
    let Some(parent) = node.parent.take().and_then(|weak| weak.upgrade()) else {
        return;
    };
    let children = std::mem::take(&mut *node.children.borrow_mut());
    for child in &children {
        child.parent.set(Some(Rc::downgrade(&parent)));
    }
    let mut siblings = parent.children.borrow_mut();
    let Some(index) = siblings.iter().position(|sibling| Rc::ptr_eq(sibling, node)) else {
        return;
    };
    siblings.remove(index);
    for (offset, child) in children.into_iter().enumerate() {
        siblings.insert(index + offset, child);
    }
}

/// Remove the origin stamp from a stamped `<a>`.
fn strip_origin_attr(node: &Handle) {
    if let NodeData::Element { attrs, .. } = &node.data {
        attrs
            .borrow_mut()
            .retain(|attribute| &*attribute.name.local != ORIGIN_ATTR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rcdom::SerializableHandle;
    use html5ever::serialize::{SerializeOpts, serialize};

    fn tree_size_and_depth(document: &Handle) -> (usize, usize) {
        let mut work = vec![(Rc::clone(document), 0)];
        let mut count = 0;
        let mut max_depth = 0;
        while let Some((node, depth)) = work.pop() {
            count += 1;
            max_depth = max_depth.max(depth);
            work.extend(node.children.borrow().iter().map(|child| (Rc::clone(child), depth + 1)));
        }
        (count, max_depth)
    }

    #[test]
    fn bounded_repair_should_cap_tree_builder_depth_on_every_shape() {
        for prefix in ["<table><td>visible", "<b><p>one</p></b>", "<x-custom>"] {
            let html = format!("{prefix}{}tail", "<div>".repeat(2_000));
            let (dom, _) = parse_with_anchor_origins(&html);
            let (nodes, depth) = tree_size_and_depth(&dom.document);
            assert!(depth <= 514, "depth={depth}, nodes={nodes}");
        }
    }

    #[test]
    fn bounded_repair_should_cap_formatting_reconstruction_amplification() {
        use std::fmt::Write;
        let bold = (0..250).fold(String::new(), |mut bold, id| {
            write!(bold, "<b id='b{id}'>").expect("writing to String succeeds");
            bold
        });
        let html = format!("<p>{bold}x</p>{}", "<p>y</p>".repeat(1_000));
        let (dom, _) = parse_with_anchor_origins(&html);
        let (nodes, _) = tree_size_and_depth(&dom.document);
        assert!(nodes <= (1_251 * 8) + 512, "amplified to {nodes} nodes");
    }

    fn repaired(html: &str) -> String {
        let (dom, _) = parse_with_anchor_origins(html);
        collapse_split_anchors(&dom.document);
        let mut buf = Vec::new();
        serialize(
            &mut buf,
            &SerializableHandle::from(dom.document),
            SerializeOpts::default(),
        )
        .expect("serialize");
        String::from_utf8(buf).expect("utf8")
    }

    fn body(html: &str) -> String {
        let out = repaired(html);
        out.trim_start_matches("<html><head></head><body>")
            .trim_end_matches("</body></html>")
            .to_string()
    }

    #[test]
    fn should_unwrap_the_reconstructed_empty_clone_and_keep_the_authored_anchor() {
        assert_eq!(
            body(r#"<a href="/o"><div><a href="/i">Inner</a></div></a>"#),
            r#"<a href="/o"></a><div><a href="/i">Inner</a></div>"#
        );
    }

    #[test]
    fn should_keep_the_clone_that_carries_text_and_drop_the_empty_authored_half() {
        assert_eq!(
            body(r#"<a href="/o"><div>Text<a href="/i">Inner</a></div></a>"#),
            r#"<div><a href="/o">Text</a><a href="/i">Inner</a></div>"#
        );
    }

    #[test]
    fn should_unwrap_a_clone_holding_only_an_empty_span() {
        // ~keep The adoption agency's second pass pops the `<span>` too, so the clone keeps an
        // ~keep empty `<span>` and the inner anchor lands after it; the clone has no content
        // ~keep of its own and is unwrapped, the `<span>` staying where it was.
        assert_eq!(
            body(r#"<a href="/o"><div><span><a href="/i">Inner</a></span></div></a>"#),
            r#"<a href="/o"></a><div><span></span><a href="/i">Inner</a></div>"#
        );
    }

    #[test]
    fn should_leave_authored_duplicate_anchors_alone() {
        let html = r#"<a href="/o"></a><div><a href="/o"></a>real</div>"#;
        assert_eq!(body(html), html);
    }

    #[test]
    fn should_never_leak_the_origin_attribute_even_when_the_input_carries_it() {
        let out = repaired(r#"<a href="/o" data-h2m-anchor-origin="7"><div><a href="/i">Inner</a></div></a>"#);
        assert!(!out.contains(ORIGIN_ATTR), "leaked: {out}");
    }

    #[test]
    fn should_count_an_image_as_content() {
        assert_eq!(
            body(r#"<a href="/o"><div><img src="s"><a href="/i">Inner</a></div></a>"#),
            r#"<div><a href="/o"><img src="s"></a><a href="/i">Inner</a></div>"#
        );
    }
}
