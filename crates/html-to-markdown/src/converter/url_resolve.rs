//! Resolution of relative `href`/`src` destinations against a caller-supplied
//! `base_url`, honoring a document's own `<base href>` the way a browser does.
//!
//! Computed exactly once per conversion (in `convert_api.rs`, before the
//! Tier-1/Tier-2 split) and threaded into both tiers as the same value, so
//! they cannot disagree on what a relative URL resolves to.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use html5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{BufferQueue, Tokenizer, TokenizerOpts};
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};
use html5ever::{Attribute, ExpandedName, QualName, TokenizerResult, local_name, ns};
use url::Url;

use crate::rcdom::{Handle, NodeData, RcDom};

/// Compute the effective base URL for a conversion.
///
/// ~keep Precedence mirrors a browser's document-base algorithm: the first `<base>`
/// ~keep with an `href` in tree order is itself resolved against the caller's
/// ~keep `base_url` (so a page at `https://example.com/blog/` with
/// ~keep `<base href="/assets/">` gets an effective base of
/// ~keep `https://example.com/assets/`, and `<base href="https://cdn.example/">`
/// ~keep overrides the caller's base entirely, exactly as `Url::join` resolves an
/// ~keep already-absolute reference). When the `<base href>` is present but fails
/// ~keep to parse/join (malformed markup), or resolves to a `data:` or `javascript:`
/// ~keep URL, it is ignored and the caller's `base_url` alone is used, as the HTML
/// ~keep "frozen base URL" steps fall back to the document's own URL.
///
/// Returns `None` when `caller_base_url` itself does not parse as an absolute
/// URL -- resolution is then a no-op everywhere, which keeps a malformed
/// `base_url` from ever panicking or corrupting output.
///
/// `document_base_href` is [`document_base_href`] of the conversion's input.
pub fn compute_effective_base(document_base_href: Option<&str>, caller_base_url: &str) -> Option<Url> {
    let caller_base = Url::parse(caller_base_url).ok()?;

    match document_base_href {
        Some(href) if !href.is_empty() => Some(
            caller_base
                .join(href)
                .ok()
                .filter(|base| !matches!(base.scheme(), "data" | "javascript"))
                .unwrap_or(caller_base),
        ),
        // ~keep An empty `<base href="">` (or no `<base>` at all) leaves the document's own
        // ~keep URL as the base, per the HTML "document base URL" algorithm -- i.e. the
        // ~keep caller's `base_url`, unchanged.
        _ => Some(caller_base),
    }
}

/// Resolve a single `href`/`src` attribute value against `base`.
///
/// Returns `Some(resolved)` when `value` was a resolvable relative reference;
/// returns `None` when `value` should be left exactly as written -- an empty
/// value, an already-absolute URL (including non-hierarchical schemes such as
/// `mailto:`, `tel:`, `javascript:`, `data:`), or a reference that fails to
/// join against `base` (malformed input never panics and never produces a
/// corrupted destination; the caller keeps using the original text).
///
/// A bare fragment (`"#section"`) resolves against `base` -- the same
/// "resolve against the current document" rule a browser applies -- rather
/// than staying a same-page-only fragment, because `base` carries the full
/// crawled page URL: `https://example.com/blog/post#section` is exactly the
/// followable link the `base_url` option exists to produce, whereas a bare
/// `#section` left untouched is not followable outside the original page. ~keep
pub fn resolve_attribute_url(base: &Url, value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    // ~keep A value that already parses as a standalone (base-less) URL is either already
    // ~keep absolute (`https://…`) or a non-hierarchical/"cannot-be-a-base" scheme
    // ~keep (`mailto:`, `tel:`, `javascript:`, `data:`) -- both cases must pass through
    // ~keep unchanged, and `Url::parse` succeeding without a base is exactly the test for
    // ~keep "this text is already a complete URL reference".
    if Url::parse(value).is_ok() {
        return None;
    }

    base.join(value).ok().map(|joined| joined.to_string())
}

// ~keep An empty href refers to the document; an empty image/media src remains absent (#756).
pub fn resolve_link_url(base: &Url, value: &str) -> Option<String> {
    if value.is_empty() {
        base.join(value).ok().map(|url| url.to_string())
    } else {
        resolve_attribute_url(base, value)
    }
}

/// The `href` of the first HTML `<base>` with one, in tree order, as a browser picks it:
/// html5ever builds the document, so a `<base>` in a comment, in raw text, in `<template>`
/// contents or in SVG does not count, and foster parenting and `<frameset>` apply. ~keep
///
/// `html` is the normalized input both tiers convert. Both the effective base and the `base`
/// metadata come from this one reading.
pub fn document_base_href(html: &str) -> Option<String> {
    let parse = parse_base_href(html);
    tracing::debug!(
        target: "html_to_markdown::convert",
        read = parse.read,
        visited = parse.visited,
        "document base href parsed"
    );
    parse.href
}

/// [`document_base_href`], with what finding it cost.
struct BaseParse {
    href: Option<String>,
    /// Bytes of `html` fed to the parser.
    read: usize,
    /// Tree nodes visited while looking for the base.
    visited: usize,
}

fn parse_base_href(html: &str) -> BaseParse {
    // ~keep A `<base>` element only comes from a start tag named `base`, so a document
    // ~keep without `<base` (in any case) is not parsed.
    let bytes = html.as_bytes();
    if !has_start_tag(bytes, b"base") {
        return BaseParse {
            href: None,
            read: 0,
            visited: 0,
        };
    }

    let tokenizer = Tokenizer::new(
        TreeBuilder::new(BaseRecordingDom::default(), TreeBuilderOpts::default()),
        TokenizerOpts::default(),
    );
    let dom = &tokenizer.sink.sink;
    let input = BufferQueue::default();
    let mut fed = 0;
    let mut visited = 0;
    let mut looked_at = 0;
    let mut may_stop = true;
    while fed < html.len() {
        // ~keep Fed in pieces cut after a `>`, an ASCII byte and so a char boundary.
        let until = memchr::memchr(b'>', &bytes[(fed + PARSE_PIECE).min(html.len())..])
            .map_or(html.len(), |at| fed + PARSE_PIECE + at + 1);
        input.push_back(StrTendril::from(&html[fed..until]));
        while !matches!(tokenizer.feed(&input), TokenizerResult::Done) {}
        fed = until;
        // ~keep Later nodes go in after every node already in the tree, except a node foster
        // ~keep parented in front of an open table and a `<frameset>` replacing the body. So the
        // ~keep first `<base href>` created that is in the document decides: outside those two
        // ~keep reaches it stays first and the rest is not parsed; inside one, the parse runs to
        // ~keep the end. Each `<base>` is looked at once, through its ancestors only, and the
        // ~keep looking stops once it has visited more nodes than bytes read (many `<base>` tags
        // ~keep deep in `<template>` contents, never in the document), so it stays linear.
        let bases = dom.bases.borrow();
        while may_stop && looked_at < bases.len() {
            if visited > fed {
                may_stop = false;
                break;
            }
            let base = &bases[looked_at];
            looked_at += 1;
            let Some(place) = place_in_document(base, &dom.dom.document, &mut visited) else {
                continue;
            };
            if place.in_table || (place.in_body && has_start_tag(bytes, b"frameset")) {
                may_stop = false;
            } else {
                return BaseParse {
                    href: base_href(base),
                    read: fed,
                    visited,
                };
            }
        }
    }
    tokenizer.end();
    BaseParse {
        href: first_base_href(&dom.dom.document, &mut visited),
        read: fed,
        visited,
    }
}

/// An [`RcDom`] that keeps each HTML `<base>` element with an `href` it creates, in order.
#[derive(Default)]
struct BaseRecordingDom {
    dom: RcDom,
    bases: RefCell<Vec<Handle>>,
}

impl TreeSink for BaseRecordingDom {
    type Handle = Handle;
    type Output = Self;
    type ElemName<'a>
        = ExpandedName<'a>
    where
        Self: 'a;

    fn finish(self) -> Self {
        self
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, flags: ElementFlags) -> Handle {
        let element = self.dom.create_element(name, attrs, flags);
        if base_href(&element).is_some() {
            self.bases.borrow_mut().push(Rc::clone(&element));
        }
        element
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        self.dom.parse_error(msg);
    }

    fn get_document(&self) -> Handle {
        self.dom.get_document()
    }

    fn elem_name<'a>(&'a self, target: &'a Handle) -> ExpandedName<'a> {
        self.dom.elem_name(target)
    }

    fn create_comment(&self, text: StrTendril) -> Handle {
        self.dom.create_comment(text)
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Handle {
        self.dom.create_pi(target, data)
    }

    fn append(&self, parent: &Handle, child: NodeOrText<Handle>) {
        self.dom.append(parent, child);
    }

    fn append_based_on_parent_node(&self, element: &Handle, prev_element: &Handle, child: NodeOrText<Handle>) {
        self.dom.append_based_on_parent_node(element, prev_element, child);
    }

    fn append_doctype_to_document(&self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        self.dom.append_doctype_to_document(name, public_id, system_id);
    }

    fn get_template_contents(&self, target: &Handle) -> Handle {
        self.dom.get_template_contents(target)
    }

    fn same_node(&self, x: &Handle, y: &Handle) -> bool {
        self.dom.same_node(x, y)
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.dom.set_quirks_mode(mode);
    }

    fn append_before_sibling(&self, sibling: &Handle, new_node: NodeOrText<Handle>) {
        self.dom.append_before_sibling(sibling, new_node);
    }

    fn add_attrs_if_missing(&self, target: &Handle, attrs: Vec<Attribute>) {
        self.dom.add_attrs_if_missing(target, attrs);
    }

    fn remove_from_parent(&self, target: &Handle) {
        self.dom.remove_from_parent(target);
    }

    fn reparent_children(&self, node: &Handle, new_parent: &Handle) {
        self.dom.reparent_children(node, new_parent);
    }

    fn is_mathml_annotation_xml_integration_point(&self, handle: &Handle) -> bool {
        self.dom.is_mathml_annotation_xml_integration_point(handle)
    }
}

/// Bytes of source fed to html5ever between looks for a final `<base href>`.
const PARSE_PIECE: usize = 4096;

/// Whether `bytes` holds `<` followed by `name` in any case.
fn has_start_tag(bytes: &[u8], name: &[u8]) -> bool {
    memchr::memchr_iter(b'<', bytes).any(|at| {
        bytes
            .get(at + 1..at + 1 + name.len())
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
    })
}

/// The `href` of an HTML `<base>` element, or `None` for any other node or a `<base>` without one.
fn base_href(node: &Handle) -> Option<String> {
    let NodeData::Element { name, attrs, .. } = &node.data else {
        return None;
    };
    if name.ns != ns!(html) || name.local != local_name!("base") {
        return None;
    }
    attrs
        .borrow()
        .iter()
        .find(|attr| attr.name.local == local_name!("href"))
        .map(|attr| attr.value.to_string())
}

/// Where a node sits in a document.
struct Place {
    /// Below a `<table>`, where foster parenting can later insert a node in front of it.
    in_table: bool,
    /// Below `<body>`, which a later `<frameset>` can replace.
    in_body: bool,
}

/// Where `node` sits below `document`, from its ancestors, or `None` when it is not in
/// `document` (it is in `<template>` contents, or was removed).
fn place_in_document(node: &Handle, document: &Handle, visited: &mut usize) -> Option<Place> {
    let mut place = Place {
        in_table: false,
        in_body: false,
    };
    let mut current = Rc::clone(node);
    loop {
        *visited += 1;
        let parent = current.parent.take();
        current.parent.set(parent.clone());
        let Some(parent) = parent.and_then(|weak| weak.upgrade()) else {
            return Rc::ptr_eq(&current, document).then_some(place);
        };
        if let NodeData::Element { name, .. } = &parent.data {
            if name.ns == ns!(html) {
                place.in_table |= name.local == local_name!("table");
                place.in_body |= name.local == local_name!("body");
            }
        }
        current = parent;
    }
}

/// The `href` of the first `<base>` with one below `root`, in tree order. `<template>` contents
/// are not children, so they are not searched.
fn first_base_href(root: &Handle, visited: &mut usize) -> Option<String> {
    // ~keep The walk holds clones while the tree is alive: dropping the last handle to a
    // ~keep node empties its whole subtree (`rcdom::Node`'s drop).
    let mut pending = vec![Rc::clone(root)];
    while let Some(node) = pending.pop() {
        *visited += 1;
        if let Some(href) = base_href(&node) {
            return Some(href);
        }
        pending.extend(node.children.borrow().iter().rev().map(Rc::clone));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[test]
    fn test_resolve_leaves_absolute_url_unchanged() {
        let result = resolve_attribute_url(&base("https://example.com/page"), "https://other.example/x");
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_protocol_relative_uses_base_scheme() {
        let result = resolve_attribute_url(&base("https://example.com/page"), "//cdn.example.com/x.png");
        assert_eq!(result, Some("https://cdn.example.com/x.png".to_string()));
    }

    #[test]
    fn test_resolve_fragment_resolves_against_full_page_url() {
        let result = resolve_attribute_url(&base("https://example.com/blog/post.html"), "#section");
        assert_eq!(result, Some("https://example.com/blog/post.html#section".to_string()));
    }

    #[test]
    fn test_resolve_leaves_mailto_unchanged() {
        assert_eq!(
            resolve_attribute_url(&base("https://example.com/"), "mailto:foo@bar.com"),
            None
        );
    }

    #[test]
    fn test_resolve_leaves_tel_unchanged() {
        assert_eq!(resolve_attribute_url(&base("https://example.com/"), "tel:+12345"), None);
    }

    #[test]
    fn test_resolve_leaves_javascript_unchanged() {
        assert_eq!(
            resolve_attribute_url(&base("https://example.com/"), "javascript:alert(1)"),
            None
        );
    }

    #[test]
    fn test_resolve_leaves_data_uri_unchanged() {
        assert_eq!(
            resolve_attribute_url(&base("https://example.com/"), "data:image/png;base64,AAA"),
            None
        );
    }

    #[test]
    fn test_resolve_leaves_empty_href_unchanged() {
        assert_eq!(resolve_attribute_url(&base("https://example.com/"), ""), None);
    }

    #[test]
    fn test_resolve_leaves_malformed_url_unchanged() {
        assert_eq!(
            resolve_attribute_url(&base("https://example.com/"), "http://[not-a-valid-host"),
            None
        );
    }

    #[test]
    fn test_resolve_relative_path_joins_against_base_directory() {
        let result = resolve_attribute_url(&base("https://example.com/blog/index.html"), "post/child.html");
        assert_eq!(result, Some("https://example.com/blog/post/child.html".to_string()));
    }

    #[test]
    fn test_resolve_absolute_path_replaces_base_path() {
        let result = resolve_attribute_url(&base("https://example.com/blog/index.html"), "/about");
        assert_eq!(result, Some("https://example.com/about".to_string()));
    }

    #[test]
    fn test_compute_effective_base_no_document_base_uses_caller_base() {
        let effective = compute_effective_base(
            document_base_href("<html><body></body></html>").as_deref(),
            "https://example.com/blog/",
        );
        assert_eq!(effective.unwrap().as_str(), "https://example.com/blog/");
    }

    #[test]
    fn test_compute_effective_base_relative_document_base_resolves_against_caller_base() {
        let html = r#"<html><head><base href="/assets/"></head><body></body></html>"#;
        let effective = compute_effective_base(
            document_base_href(html).as_deref(),
            "https://example.com/blog/post.html",
        );
        assert_eq!(effective.unwrap().as_str(), "https://example.com/assets/");
    }

    #[test]
    fn test_compute_effective_base_absolute_document_base_overrides_caller_base() {
        let html = r#"<html><head><base href="https://cdn.example.com/"></head><body></body></html>"#;
        let effective = compute_effective_base(
            document_base_href(html).as_deref(),
            "https://example.com/blog/post.html",
        );
        assert_eq!(effective.unwrap().as_str(), "https://cdn.example.com/");
    }

    #[test]
    fn test_compute_effective_base_empty_document_base_href_uses_caller_base() {
        let html = r#"<html><head><base href=""></head><body></body></html>"#;
        let effective = compute_effective_base(
            document_base_href(html).as_deref(),
            "https://example.com/blog/post.html",
        );
        assert_eq!(effective.unwrap().as_str(), "https://example.com/blog/post.html");
    }

    #[test]
    fn test_compute_effective_base_malformed_caller_base_disables_resolution() {
        let effective = compute_effective_base(document_base_href("<html></html>").as_deref(), "not a url");
        assert_eq!(effective, None);
    }

    #[test]
    fn test_compute_effective_base_finds_base_href_outside_head_range() {
        // ~keep A `<base>` before any `<head>` still counts: the parser places it in the
        // ~keep head it opens.
        let html = r#"<base href="/x/"><body></body>"#;
        let effective = compute_effective_base(document_base_href(html).as_deref(), "https://example.com/");
        assert_eq!(effective.unwrap().as_str(), "https://example.com/x/");
    }

    #[test]
    fn test_document_base_href_decodes_entities() {
        let html = r#"<head><base href="/a&amp;b"></head>"#;
        assert_eq!(document_base_href(html), Some("/a&b".to_string()));
    }

    #[test]
    fn test_document_base_href_ignores_tag_named_basefoo() {
        let html = r#"<basefoo href="/wrong"></basefoo><base href="/right">"#;
        assert_eq!(document_base_href(html), Some("/right".to_string()));
    }

    const PAGE: &str = "https://example.com/blog/post.html";

    fn effective(html: &str) -> String {
        compute_effective_base(document_base_href(html).as_deref(), PAGE)
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_inside_a_comment() {
        let html = r#"<head><!-- <base href="https://evil.example/"> --></head><body></body>"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_inside_raw_text() {
        for element in [
            "title", "textarea", "script", "style", "xmp", "iframe", "noembed", "noframes", "noscript",
        ] {
            let html = format!(
                r#"<head><{element}><base href="https://evil.example/"></{element}><base href="/real/"></head>"#
            );
            assert_eq!(effective(&html), "https://example.com/real/", "inside <{element}>");
        }
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_after_plaintext() {
        let html = r#"<body><plaintext><base href="https://evil.example/">"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_takes_a_base_in_the_body_when_it_is_the_first() {
        let html = r#"<head></head><body><p>text</p><base href="/in-body/"></body>"#;
        assert_eq!(effective(html), "https://example.com/in-body/");
    }

    #[test]
    fn test_compute_effective_base_takes_the_first_base_in_tree_order_not_source_order() {
        // ~keep The loose `<base>` is foster-parented in front of the table, so it precedes
        // ~keep the one inside the cell in tree order although it comes later in the source.
        let html = r#"<body><table><tr><td><base href="/in-cell/"></td></tr><base href="/loose/"></table>"#;
        assert_eq!(effective(html), "https://example.com/loose/");
    }

    #[test]
    fn test_compute_effective_base_keeps_tree_order_across_parse_pieces() {
        // ~keep The cell's `<base>` is parsed pieces before the loose one that precedes it.
        let padding = "x".repeat(3 * PARSE_PIECE);
        let html =
            format!(r#"<body><table><tr><td><base href="/in-cell/">{padding}</td></tr><base href="/loose/"></table>"#);
        assert_eq!(effective(&html), "https://example.com/loose/");
    }

    #[test]
    fn test_compute_effective_base_cuts_parse_pieces_on_char_boundaries() {
        let padding = "\u{e9}".repeat(3 * PARSE_PIECE);
        // ~keep The odd-length prefix puts a byte offset of `PARSE_PIECE` inside a two-byte `é`.
        let html = format!(r#"<body><p title="x{padding}">{padding}</p><base href="/late/"></body>"#);
        assert_eq!(effective(&html), "https://example.com/late/");
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_in_a_body_a_frameset_replaces() {
        // ~keep The body must be implied: an explicit `<body>` tag stops a frameset replacing it.
        let html = r#"<head></head><div><base href="https://evil.example/"></div><frameset></frameset>"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_inside_a_template() {
        let html = r#"<head><template><base href="https://evil.example/"></template></head>"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_inside_svg() {
        let html = r#"<body><svg><base href="https://evil.example/"></base></svg></body>"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_ignores_a_base_inside_svg_cdata() {
        // ~keep In SVG, `<![CDATA[` runs to `]]>`; read as a bogus comment it would end at the
        // ~keep first `>` and let `<p>` break out of SVG ahead of the `<base>`.
        assert_eq!(
            effective(r#"<body><svg><![CDATA[x><p><base href="https://evil.example/">]]></svg>"#),
            PAGE
        );
    }

    #[test]
    fn test_compute_effective_base_skips_a_first_base_without_href() {
        let html = r#"<head><base target="_blank"><base href="/second/"></head>"#;
        assert_eq!(effective(html), "https://example.com/second/");
    }

    #[test]
    fn test_compute_effective_base_first_base_with_empty_href_wins_over_a_later_one() {
        let html = r#"<head><base href=""><base href="https://evil.example/"></head>"#;
        assert_eq!(effective(html), PAGE);
    }

    #[test]
    fn test_compute_effective_base_reads_an_upper_case_base_tag() {
        let html = r#"<HEAD><BASE HREF="/upper/"></HEAD>"#;
        assert_eq!(effective(html), "https://example.com/upper/");
    }

    #[test]
    fn test_compute_effective_base_reads_a_tag_name_ended_by_a_form_feed() {
        let html = "<head><base\x0chref=\"/ff/\"></head>";
        assert_eq!(effective(html), "https://example.com/ff/");
    }

    #[test]
    fn test_compute_effective_base_falls_back_to_the_page_for_a_data_or_javascript_base() {
        for href in ["data:text/html,x", "javascript:void(0)"] {
            let html = format!(r#"<head><base href="{href}"><base href="/second/"></head>"#);
            assert_eq!(effective(&html), PAGE, "{href}");
        }
        let html = r#"<head><base href="https://cdn.example/"></head>"#;
        assert_eq!(effective(html), "https://cdn.example/");
    }

    #[test]
    fn test_parse_base_href_stops_at_a_first_base_in_the_body() {
        let html = format!(
            r#"<head></head><body><p>text</p><base href="/in-body/">{}"#,
            "<p>more</p>".repeat(PARSE_PIECE)
        );
        let parse = parse_base_href(&html);
        assert_eq!(parse.href.as_deref(), Some("/in-body/"));
        assert!(
            parse.read <= 2 * PARSE_PIECE,
            "read {} of {} bytes",
            parse.read,
            html.len()
        );
    }

    #[test]
    fn test_parse_base_href_stops_at_a_first_base_in_the_head() {
        let html = format!(
            r#"<head><base href="/in-head/"></head><body>{}"#,
            "<p>more</p>".repeat(PARSE_PIECE)
        );
        let parse = parse_base_href(&html);
        assert_eq!(parse.href.as_deref(), Some("/in-head/"));
        assert!(
            parse.read <= 2 * PARSE_PIECE,
            "read {} of {} bytes",
            parse.read,
            html.len()
        );
    }

    #[test]
    fn test_parse_base_href_stops_at_a_first_base_with_a_gt_in_a_quoted_value() {
        // ~keep The first piece ends at the `>` inside the quoted value, mid tag.
        let href = format!("https://a.example/?q={}>", "x".repeat(PARSE_PIECE));
        let html = format!(
            r#"<head><base href="{href}"></head><body>{}"#,
            "<p>more</p>".repeat(PARSE_PIECE)
        );
        let parse = parse_base_href(&html);
        assert_eq!(parse.href.as_deref(), Some(href.as_str()));
        assert!(
            parse.read <= 3 * PARSE_PIECE,
            "read {} of {} bytes",
            parse.read,
            html.len()
        );
    }

    #[test]
    fn test_parse_base_href_does_not_parse_a_page_without_base_text() {
        assert_eq!(parse_base_href("<html><body><p>x</p></body></html>").read, 0);
    }

    #[test]
    fn test_parse_base_href_visits_each_node_a_bounded_number_of_times() {
        // ~keep Every piece holds a `<base>` without an `href` and `<base href>` tags in templates,
        // ~keep and none of them lets the parse stop early.
        let piece = format!(
            "<base target=_blank>{}",
            "<template><base href=/t/></template>".repeat(PARSE_PIECE / 38)
        );
        let html = format!("<body>{}<table><td><base href=/cell/></td></table>", piece.repeat(64));
        let parse = parse_base_href(&html);
        assert_eq!((parse.href.as_deref(), parse.read), (Some("/cell/"), html.len()));
        let tags = memchr::memchr_iter(b'<', html.as_bytes()).count();
        assert!(
            parse.visited <= 2 * tags,
            "visited {} nodes for {tags} tags",
            parse.visited
        );
    }

    #[test]
    fn test_parse_base_href_stops_looking_at_bases_deep_in_a_template() {
        let html = format!(
            "<template>{}{}</template><body>{}",
            "<div>".repeat(2000),
            "<base href=/t/>".repeat(2000),
            "<p>more</p>".repeat(PARSE_PIECE)
        );
        let parse = parse_base_href(&html);
        assert_eq!((parse.href, parse.read), (None, html.len()));
        assert!(
            parse.visited <= 2 * html.len(),
            "visited {} nodes for {} bytes",
            parse.visited,
            html.len()
        );
    }

    #[test]
    fn test_parse_base_href_looks_at_a_base_through_its_ancestors_only() {
        // ~keep The comment and `</base>` add no `<base>`; the template's one is looked at once.
        let html = r#"<!-- <base href="/x/"> --><template><base href="/t/"></template><body><p>x</p></base></body>"#;
        let parse = parse_base_href(html);
        assert_eq!((parse.href, parse.read), (None, html.len()));
        // ~keep The template's `<base>` and its contents fragment, then the walk after the parse:
        // ~keep document, comment, html, head, template, body, p, text.
        assert_eq!(parse.visited, 10);
    }

    #[test]
    fn test_compute_effective_base_takes_a_base_in_an_html_integration_point_of_mathml() {
        let html =
            r#"<body><math><annotation-xml encoding="text/html"><base href="/in-math/"></annotation-xml></math>"#;
        assert_eq!(effective(html), "https://example.com/in-math/");
    }

    #[test]
    fn test_compute_effective_base_keeps_a_first_base_in_a_closed_table() {
        let html = r#"<body><table><tr><td><base href="/in-cell/"></td></tr></table><p><base href="/after/"></p>"#;
        assert_eq!(effective(html), "https://example.com/in-cell/");
    }

    #[test]
    fn test_compute_effective_base_ignores_a_body_base_a_frameset_pieces_later_replaces() {
        // ~keep `<div>` keeps the frameset allowed, where text would not.
        let padding = "<div></div>".repeat(PARSE_PIECE);
        let html =
            format!(r#"<head></head><div><base href="https://evil.example/"></div>{padding}<frameset></frameset>"#);
        assert_eq!(effective(&html), PAGE);
    }
}
