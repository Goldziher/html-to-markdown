//! HTML preprocessing and validation helpers.
//!
//! This module contains helper functions for preprocessing HTML before conversion,
//! including validation and normalization checks.

use crate::converter::dom_context::{DomContext, TagInfo};
use crate::converter::main_helpers::{is_ascii_whitespace_only, is_inline_element, starts_body};
use crate::converter::utility::attributes::{attribute_matches_any, element_has_navigation_hint};
use crate::options::ConversionOptions;
use crate::text::decode_html_entities_cow;

/// Check if an inline ancestor element is allowed to contain block-level elements.
pub fn inline_ancestor_allows_block(tag_name: &str) -> bool {
    matches!(tag_name, "a" | "ins" | "del")
}

/// Ancestor state inherited top-down while scanning for misnested elements.
///
/// ~keep Each field mirrors one of the three independent ancestor-chain scans the
/// ~keep original implementation ran separately per node (O(depth) each); carrying
/// ~keep the already-computed parent result down to children makes every node O(1)
/// ~keep instead, since each scan only ever needs the *nearest* qualifying ancestor.
#[derive(Clone, Copy)]
struct MisnestState {
    /// True if a strict ancestor is an anchor.
    inside_anchor: bool,
    /// True if this node or any strict ancestor is `<pre>`/`<code>`.
    inside_preformatted: bool,
    /// True if any strict ancestor is an inline element that disallows block children.
    blocked_by_inline_ancestor: bool,
    /// Result of a `has_p_ancestor` scan started at this node (used by its children).
    p_ancestor_state: bool,
    /// True if a strict ancestor is `<li>`/`<dt>`/`<dd>` with no intervening
    /// `<ul>`/`<ol>`/`<dl>` (i.e. we are still "inside" that same list item).
    list_item_ancestor_state: bool,
    /// True if a strict ancestor is `<td>`/`<th>` with no intervening
    /// `<table>`/`<thead>`/`<tbody>`/`<tfoot>`/`<tr>` (i.e. we are still "inside"
    /// that same cell's content, not a fresh row).
    cell_ancestor_state: bool,
}

impl MisnestState {
    const ROOT: Self = Self {
        inside_anchor: false,
        inside_preformatted: false,
        blocked_by_inline_ancestor: false,
        p_ancestor_state: false,
        list_item_ancestor_state: false,
        cell_ancestor_state: false,
    };
}

/// True when `info`, reached with inherited `state`, is one of the misnestings this pass repairs.
///
/// ~keep Each disjunct is an independent, side-effect-free shape check, so collecting them here
/// ~keep preserves the original sequence of early returns exactly -- same order, same
/// ~keep short-circuiting, same answer -- while keeping the walk itself under the complexity limit
/// ~keep (issue #465).
fn node_is_misnested(info: &TagInfo, state: MisnestState, self_inside_preformatted: bool) -> bool {
    // An anchor nested inside another anchor, outside preformatted content.
    (info.name == "a" && state.inside_anchor && !state.inside_preformatted)
        // ~keep Table elements under <p>: tl misparsed an unclosed <p> in <td>.
        || (matches!(info.name.as_str(), "td" | "tr" | "th") && state.p_ancestor_state)
        || (info.is_block && !self_inside_preformatted && state.blocked_by_inline_ancestor)
        // ~keep <li>/<dt>/<dd> nested under another one without an intervening list
        // ~keep container: tl absorbed the next item because a <p>/<div> inside the
        // ~keep previous one was left unclosed.
        || (matches!(info.name.as_str(), "li" | "dt" | "dd") && state.list_item_ancestor_state)
        // ~keep <tr> nested directly inside a <td>/<th>: not reachable per HTML5 tree
        // ~keep construction, and silently dropped by the table renderer (issue #486).
        || (info.name == "tr" && state.cell_ancestor_state)
}

/// Inherited state to hand to every child of `info`.
fn child_misnest_state(info: &TagInfo, state: MisnestState, self_inside_preformatted: bool) -> MisnestState {
    MisnestState {
        inside_anchor: state.inside_anchor || info.name == "a",
        inside_preformatted: self_inside_preformatted,
        blocked_by_inline_ancestor: state.blocked_by_inline_ancestor
            || (is_inline_element(&info.name) && !inline_ancestor_allows_block(&info.name)),
        p_ancestor_state: p_ancestor_state_for(&info.name, state.p_ancestor_state),
        list_item_ancestor_state: list_item_ancestor_state_for(&info.name, state.list_item_ancestor_state),
        cell_ancestor_state: cell_ancestor_state_for(&info.name, state.cell_ancestor_state),
    }
}

/// Detect malformed nesting that requires HTML5 tree repair.
///
/// Nested anchors must be closed by the HTML5 adoption agency algorithm so their
/// destinations remain separate Markdown links (issue #479).
///
/// Excludes elements inside `<pre>` or `<code>` blocks, as they have special
/// whitespace preservation rules and should not be repaired.
///
/// Also detects table structural elements (`td`, `tr`, `th`) nested under `<p>` —
/// a structural impossibility in valid HTML that signals the `tl` parser absorbed
/// a table into a paragraph because of an unclosed `<p>` (common in Word/Outlook
/// HTML such as `<p class='MsoNormal'>` cells). Issue #336.
///
/// Also detects a `<li>`/`<dt>`/`<dd>` nested under another `<li>`/`<dt>`/`<dd>`
/// with no intervening `<ul>`/`<ol>`/`<dl>`. `<li>content<div><li>more` is a
/// structural impossibility in valid HTML: the HTML5 spec implies a closing
/// `</li>` when a new `<li>` starts, regardless of what block elements (`<p>`,
/// `<div>`, …) are still open inside it. `tl` does not apply that rule, so an
/// unclosed `<p>`/`<div>` inside a list item causes it to nest the next `<li>`
/// as a child instead of treating it as a sibling.
///
/// Also detects a `<tr>` nested directly inside a `<td>`/`<th>` — a structural
/// impossibility in valid HTML (a `<tr>` is only ever a child of `<table>`,
/// `<thead>`, `<tbody>`, or `<tfoot>`). HTML5's "in cell" insertion mode closes the
/// open cell (and its row) before starting the new row as a sibling of the one that
/// held the cell; `tl` instead nests it literally, and the table renderer only walks
/// a cell's own row/table ancestry, so the nested row's content is never reached and
/// is silently dropped. Issue #486.
///
/// Also detects a `<table>` with a direct child that HTML5's "in table" insertion
/// mode would foster-parent (non-whitespace text) or restructure (any element
/// outside the small set valid directly under `<table>`, e.g. a stray `<td>` or
/// `<a>` with no `<tr>`). `tl` has no foster-parenting or table auto-fixup: it
/// leaves such content exactly where it was written, as a direct child of the
/// `<table>` node, and the table handler (`converter::block::table::builder`)
/// only recognises `caption`/`thead`/`tbody`/`tfoot`/`tr`/`colgroup`/`col` there —
/// silently dropping raw text and routing anything else through a no-op handler.
///
/// Also detects a `<tr>`/`<row>` or table section (`<thead>`/`<tbody>`/`<tfoot>`) with a
/// misplaced element child (e.g. a `<p>` stranded inside a `<tr>` by a dropped `</table>`
/// close tag). See [`is_row_context_with_misplaced_element`]. Issue #489.
///
/// ~keep Walks the tree top-down exactly once, carrying inherited ancestor state
/// ~keep (see [`MisnestState`]) instead of re-walking every node's ancestor chain.
/// ~keep The original per-node ancestor walk was O(depth) per node — O(n²) total on
/// ~keep a deeply nested chain (e.g. 20k nested `<div>`s took ~30s; this pass alone
/// ~keep accounted for essentially all of it, confirmed via phase timing in
/// ~keep `tools/benchmark-harness/examples/profile_deep_nesting_phases.rs`).
pub fn has_inline_block_misnest(dom_ctx: &DomContext, parser: &tl::Parser) -> bool {
    let mut stack: Vec<(tl::NodeHandle, MisnestState)> = dom_ctx
        .root_children
        .iter()
        .map(|handle| (*handle, MisnestState::ROOT))
        .collect();

    while let Some((handle, state)) = stack.pop() {
        let node_id = handle.get_inner();
        if !matches!(handle.get(parser), Some(tl::Node::Tag(_))) {
            continue;
        }
        let Some(info) = dom_ctx.tag_info(node_id, parser) else {
            continue;
        };

        let self_inside_preformatted = state.inside_preformatted || matches!(info.name.as_str(), "pre" | "code");
        if node_is_misnested(info, state, self_inside_preformatted) {
            return true;
        }

        if let Some(children) = dom_ctx.children_of(node_id) {
            // ~keep <table> with a direct child that a spec-compliant parser would
            // ~keep foster-parent or restructure (see the doc comment above).
            if info.name == "table"
                && children
                    .iter()
                    .any(|child| is_foster_parenting_candidate(*child, parser, dom_ctx))
            {
                return true;
            }

            // ~keep A <tr>/<row> or table section (<thead>/<tbody>/<tfoot>) holding a
            // ~keep misplaced element child: `tl`'s `read_end` (astral-tl
            // ~keep parser/base.rs:240-256) silently discards an unmatched close tag
            // ~keep instead of erroring, so `</table>` inside a still-open `<tr>` is
            // ~keep dropped and a following sibling element (e.g. a footer `<p>`)
            // ~keep attaches as a child of that `<tr>` instead of becoming `<table>`'s
            // ~keep sibling. `collect_table_cells` only walks `td`/`th`/`cell`
            // ~keep children, so the misattached element -- and everything after it --
            // ~keep is silently dropped (issue #489). Deliberately element-only:
            // ~keep `is_foster_parenting_candidate`'s text arm decodes entities and
            // ~keep treats `&nbsp;` as non-whitespace, so widening this to text would
            // ~keep route every Outlook/newsletter `<tr>&nbsp;</tr>` through a full
            // ~keep html5ever re-parse for no benefit. `<tr>stray text</tr>` losing its
            // ~keep text stays a separate, documented defect (see the CONTROL test in
            // ~keep `issue_489_regressions.rs`).
            if is_row_context_with_misplaced_element(info, children, parser, dom_ctx) {
                return true;
            }

            let child_state = child_misnest_state(info, state, self_inside_preformatted);
            stack.extend(children.iter().map(|child| (*child, child_state)));
        }
    }

    false
}

/// True when an element with content is still open at the end of the input, so the HTML tree
/// builder must decide where it ends.
///
/// ~keep `tl` closes an element only for an end tag that names the element on top of its stack
/// ~keep and discards every other end tag (astral-tl `parser/base.rs`, `read_end`). In
/// ~keep `<div><p>text</div>` the `div` and the `p` stay open and take the rest of the page as
/// ~keep descendants (issue #772). The open elements are the stack of `tl` at the end of the
/// ~keep input: the last root node, its last child, and so on, so the walk is O(depth).
/// ~keep `html` and `body` are exempt: their end tags close no element in the standard.
pub fn has_omitted_end_tag(dom_ctx: &DomContext, parser: &tl::Parser) -> bool {
    let mut last = dom_ctx.root_children.last().copied();
    while let Some(handle) = last {
        let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
            return false;
        };
        if has_end_tag(tag) {
            return false;
        }
        let children = dom_ctx.children_of(handle.get_inner());
        let has_content = children.is_some_and(|children| !children.is_empty());
        let name = tag.name().as_bytes();
        if has_content && !name.eq_ignore_ascii_case(b"html") && !name.eq_ignore_ascii_case(b"body") {
            return true;
        }
        last = children.and_then(|children| children.last().copied());
    }
    false
}

/// True when the source range of `tag` ends with its end tag. For an element that `tl` never
/// closed, the range is the start tag alone.
pub fn has_end_tag(tag: &tl::HTMLTag<'_>) -> bool {
    let name = tag.name().as_bytes();
    tag.raw()
        .as_bytes()
        .strip_suffix(b">")
        .and_then(|raw| raw.strip_suffix(name))
        .is_some_and(|raw| raw.ends_with(b"</"))
}

/// The children of one element (or of the page) in the walk of [`has_discarded_end_tag`].
struct SiblingList<'a> {
    children: &'a [tl::NodeHandle],
    next: usize,
    /// The source offset where the last node ended, when the offset is known.
    previous_end: Option<usize>,
    /// The source offset of the end tag of the element, for an element that has one.
    end_tag: Option<usize>,
    /// The name of the element in lower case. The page has none.
    name: Option<std::borrow::Cow<'a, [u8]>>,
}

/// True when the first parse discarded an end tag that names an element that was open, so the
/// HTML tree builder must decide where that element ends (issue #810).
///
/// ~keep `tl` discards an end tag that does not name the element on top of its stack (astral-tl
/// ~keep `parser/base.rs`, `read_end`) and makes no node for it. Each node is a slice of the
/// ~keep source, so a discarded end tag is the source between two sibling nodes, or between
/// ~keep the last child and the end tag of the parent, that is in no node. In
/// ~keep `<div><p>one</div>two</p>` the `p` holds the text nodes `one` and `two`, and the
/// ~keep source between them is `</div>`. The walk reads each node once and no token.
/// ~keep A comment and an attribute value are inside a node, so an end tag in one is not seen.
/// ~keep Not read: the source between a start tag and the first child of an element that has
/// ~keep its end tag, because `tl` does not keep the length of a start tag.
pub fn has_discarded_end_tag(dom_ctx: &DomContext, parser: &tl::Parser, input: &str) -> bool {
    let source = input.as_bytes();
    let mut open: std::collections::HashMap<std::borrow::Cow<'_, [u8]>, usize> = std::collections::HashMap::new();
    let mut lists = vec![SiblingList {
        children: &dom_ctx.root_children,
        next: 0,
        previous_end: None,
        end_tag: None,
        name: None,
    }];
    while let Some(list) = lists.last_mut() {
        let Some(handle) = list.children.get(list.next) else {
            if names_open_element(source, list.previous_end, list.end_tag, &open) {
                return true;
            }
            if let Some(count) = list.name.as_ref().and_then(|name| open.get_mut(name)) {
                *count -= 1;
            }
            lists.pop();
            continue;
        };
        list.next += 1;
        let Some(node) = handle.get(parser) else {
            list.previous_end = None;
            continue;
        };
        let span = node_span(node, source);
        if names_open_element(source, list.previous_end, span.map(|(start, _)| start), &open) {
            return true;
        }
        list.previous_end = span.map(|(_, end)| end);
        let tl::Node::Tag(tag) = node else { continue };
        let Some(children) = dom_ctx
            .children_of(handle.get_inner())
            .filter(|children| !children.is_empty())
        else {
            continue;
        };
        let name = tag.name().as_bytes();
        let closed = has_end_tag(tag);
        let name_in_lower_case = lower_case(name);
        *open.entry(name_in_lower_case.clone()).or_insert(0) += 1;
        lists.push(SiblingList {
            children,
            next: 0,
            // ~keep The source range of an element with no end tag is its start tag.
            previous_end: span.filter(|_| !closed).map(|(_, end)| end),
            end_tag: span.filter(|_| closed).map(|(_, end)| end - name.len() - 3),
            name: Some(name_in_lower_case),
        });
    }
    false
}

fn lower_case(name: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if name.iter().any(u8::is_ascii_uppercase) {
        std::borrow::Cow::Owned(name.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(name)
    }
}

/// The source range of `node`, or `None` when its bytes are no part of `source`: `tl` gives a
/// comment with no end an empty slice of its own.
fn node_span(node: &tl::Node<'_>, source: &[u8]) -> Option<(usize, usize)> {
    let raw = match node {
        tl::Node::Tag(tag) => tag.raw().as_bytes(),
        tl::Node::Raw(bytes) | tl::Node::Comment(bytes) => bytes.as_bytes(),
    };
    let start = (raw.as_ptr() as usize).checked_sub(source.as_ptr() as usize)?;
    (!raw.is_empty() && start + raw.len() <= source.len()).then_some((start, start + raw.len()))
}

/// True when `source[from..to]`, which is in no node, holds an end tag that names an open element.
fn names_open_element(
    source: &[u8],
    from: Option<usize>,
    to: Option<usize>,
    open: &std::collections::HashMap<std::borrow::Cow<'_, [u8]>, usize>,
) -> bool {
    let Some(gap) = from.zip(to).and_then(|(from, to)| source.get(from..to)) else {
        return false;
    };
    memchr::memmem::find_iter(gap, b"</").any(|at| {
        let name = &gap[at + 2..];
        let end = name
            .iter()
            .position(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'))
            .unwrap_or(name.len());
        open.get(&*lower_case(&name[..end])).is_some_and(|count| *count > 0)
    })
}

/// The count of characters that are not ASCII white space in the text nodes of `dom` that the
/// converter can write, with every character reference decoded.
///
/// ~keep The measure that decides whether a second parse lost text. It is taken the same way
/// ~keep on the page as written and on the page as the tree builder wrote it back, so only a
/// ~keep difference between the two pages changes it. A parse can drop text, keep it or move
/// ~keep it, and cannot put other text in its place, so a page that lost text has a smaller
/// ~keep count. A no-break space counts on both pages, written as a character or as `&nbsp;`.
pub fn readable_text_len(dom: &tl::VDom<'_>) -> usize {
    let parser = dom.parser();
    let mut total = 0;
    let mut stack = dom.children().to_vec();
    while let Some(handle) = stack.pop() {
        match handle.get(parser) {
            Some(tl::Node::Raw(text)) => total += decoded_len(text.as_bytes()),
            Some(tl::Node::Tag(tag)) if holds_readable_text(tag, parser) => {
                stack.extend(tag.children().top().iter().copied());
            }
            _ => {}
        }
    }
    total
}

/// The count of characters of `text` that are not ASCII white space, after its character
/// references are decoded. Text with no `&` is counted as it is.
fn decoded_len(text: &[u8]) -> usize {
    // ~keep A byte of the form 10xxxxxx continues a character, so it starts none.
    let characters = |bytes: &[u8]| {
        bytes
            .iter()
            .filter(|byte| !byte.is_ascii_whitespace() && (**byte & 0xC0) != 0x80)
            .count()
    };
    if memchr::memchr(b'&', text).is_none() {
        return characters(text);
    }
    characters(decode_html_entities_cow(&String::from_utf8_lossy(text)).as_bytes())
}

/// Elements whose text the count leaves out. The tree builder moves a `title` that starts a
/// page into the head; only the forced second tier wrote a `title` with no `head` as body text.
const NO_READABLE_TEXT: [&[u8]; 5] = [b"script", b"style", b"template", b"noscript", b"title"];

/// False for an element whose text the count leaves out, so text that the tree builder
/// moves into one counts as lost.
///
/// ~keep The tree builder moves a `<noframes>` that starts a page into the head, and `tl`
/// ~keep reads the markup in it as elements: `<noframes><div></noframes><p>one</p>` comes back
/// ~keep with all its text in the head. A `head` is read by the rule of the converter
/// ~keep ([`starts_body`]).
fn holds_readable_text(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> bool {
    let name = tag.name().as_bytes();
    if name.eq_ignore_ascii_case(b"head") {
        return tag.children().top().iter().any(|child| {
            matches!(
                child.get(parser),
                Some(tl::Node::Tag(child)) if starts_body(&child.name().as_bytes().to_ascii_lowercase())
            )
        });
    }
    !NO_READABLE_TEXT
        .iter()
        .any(|skipped| name.eq_ignore_ascii_case(skipped))
}

/// True if `child`, as a direct child of a `<table>` element, is content that a
/// spec-compliant HTML5 parser would foster-parent (non-whitespace text) or
/// restructure (an element outside the small set valid directly under `<table>`)
/// rather than leave in place.
///
/// A comment is neither: HTML5's "in table" insertion mode inserts a comment token
/// as a child of the current node (the table itself), so it is not lost and needs
/// no repair.
fn is_foster_parenting_candidate(child: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    match child.get(parser) {
        Some(tl::Node::Raw(bytes)) => {
            let raw = bytes.as_utf8_str();
            let decoded = crate::text::decode_html_entities_cow(raw.as_ref());
            !is_ascii_whitespace_only(&decoded)
        }
        Some(tl::Node::Tag(_)) => dom_ctx
            .tag_name_for(child, parser)
            .is_some_and(|name| !is_table_structural_child(name.as_ref())),
        _ => false,
    }
}

/// Tag names valid as a direct child of `<table>` per the HTML5 "in table" insertion
/// mode: `caption`, `colgroup`, `col`, `thead`/`tbody`/`tfoot`, `tr`, plus
/// `style`/`script`/`template`, which are left wherever they are written. `row` is
/// this codebase's normalized alias for `tr` from non-HTML input sources (see
/// `converter::block::table::scanner`).
fn is_table_structural_child(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "caption" | "colgroup" | "col" | "thead" | "tbody" | "tfoot" | "tr" | "row" | "style" | "script" | "template"
    )
}

/// True when `info` is a `<tr>`/`<row>` or table section (`<thead>`/`<tbody>`/`<tfoot>`) with an
/// element child outside the set valid in that context (see [`is_row_structural_child`] and
/// [`is_section_structural_child`]). Element children only -- see the call site's doc comment for
/// why text is deliberately excluded.
fn is_row_context_with_misplaced_element(
    info: &TagInfo,
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> bool {
    let valid: fn(&str) -> bool = match info.name.as_str() {
        "tr" | "row" => is_row_structural_child,
        "thead" | "tbody" | "tfoot" => is_section_structural_child,
        _ => return false,
    };
    children.iter().any(|child| {
        matches!(child.get(parser), Some(tl::Node::Tag(_)))
            && dom_ctx
                .tag_name_for(*child, parser)
                .is_some_and(|name| !valid(name.as_ref()))
    })
}

/// Tag names valid as a direct child of `<tr>`/`<row>`: `td`/`th`/`cell` (this codebase's
/// normalized cell aliases, see `converter::block::table::scanner`), plus `script`/`style`/
/// `template`, which are left wherever they are written.
fn is_row_structural_child(tag_name: &str) -> bool {
    matches!(tag_name, "td" | "th" | "cell" | "script" | "style" | "template")
}

/// Tag names valid as a direct child of `<thead>`/`<tbody>`/`<tfoot>`: `tr`/`row`, plus
/// `script`/`style`/`template`, which are left wherever they are written.
fn is_section_structural_child(tag_name: &str) -> bool {
    matches!(tag_name, "tr" | "row" | "script" | "style" | "template")
}

/// Compute the `has_p_ancestor` scan result to hand down to a node's children,
/// given the node's own tag name and the state its own parent handed down.
///
/// Mirrors the original ancestor walk's stopping rule: a `<p>` ancestor found
/// before crossing a `table`/`body`/`html` boundary counts; hitting the boundary
/// first resets the search to "no `<p>` ancestor".
fn p_ancestor_state_for(tag_name: &str, inherited: bool) -> bool {
    if tag_name == "p" {
        true
    } else if matches!(tag_name, "table" | "body" | "html") {
        false
    } else {
        inherited
    }
}

/// Compute the `list_item_ancestor_state` scan result to hand down to a node's
/// children, given the node's own tag name and the state its own parent handed down.
///
/// A `<li>`/`<dt>`/`<dd>` ancestor found before crossing a `<ul>`/`<ol>`/`<dl>`
/// (a new list context) counts; crossing into a new list container, or the
/// `table`/`body`/`html` boundary, resets the search — a `<li>` nested inside a
/// genuinely new list (or a new formatting context such as a table cell) is not
/// misnesting.
fn list_item_ancestor_state_for(tag_name: &str, inherited: bool) -> bool {
    if matches!(tag_name, "li" | "dt" | "dd") {
        true
    } else if matches!(tag_name, "ul" | "ol" | "dl" | "table" | "body" | "html") {
        false
    } else {
        inherited
    }
}

/// Compute the `cell_ancestor_state` scan result to hand down to a node's children,
/// given the node's own tag name and the state its own parent handed down.
///
/// A `<td>`/`<th>` ancestor found before crossing a `<table>`/`<thead>`/`<tbody>`/
/// `<tfoot>`/`<tr>` boundary counts; crossing any of those boundaries resets the
/// search, since a `<tr>` there is legitimately positioned relative to a new row or
/// table context.
fn cell_ancestor_state_for(tag_name: &str, inherited: bool) -> bool {
    if matches!(tag_name, "td" | "th") {
        true
    } else if matches!(tag_name, "table" | "thead" | "tbody" | "tfoot" | "tr") {
        false
    } else {
        inherited
    }
}

/// Determine if a node should be dropped during preprocessing.
///
/// Behavior depends on the [`PreprocessingPreset`]:
///
/// - **Minimal**: Only scripts/styles are stripped (handled elsewhere). This function
///   drops nothing — all structural elements are preserved.
/// - **Standard** (default): Drops `<nav>` unconditionally. Drops `<header>`, `<footer>`,
///   and `<aside>` only when they have navigation hints (class/role/aria attributes
///   indicating site chrome). Drops `<form>` when `remove_forms` is enabled.
/// - **Aggressive**: All of Standard, plus: drops `<footer>`, `<aside>`, `<noscript>`
///   unconditionally. Drops ANY element with navigation hints in class/id/role
///   (e.g. `<div class="sidebar">`). Drops elements with noise-related classes/roles.
pub fn should_drop_for_preprocessing(
    tag_name: &str,
    tag: &tl::HTMLTag,
    options: &ConversionOptions,
    is_page_header: bool,
) -> bool {
    use crate::options::PreprocessingPreset;

    if !options.preprocessing.enabled {
        return false;
    }

    let preset = options.preprocessing.preset;

    if preset == PreprocessingPreset::Minimal {
        return false;
    }

    if options.preprocessing.remove_forms && tag_name == "form" {
        return true;
    }

    let is_aggressive = preset == PreprocessingPreset::Aggressive;

    // ~keep Aggressive: drop <noscript> — its content is fallback for no-JS browsers.
    if is_aggressive && tag_name == "noscript" {
        return true;
    }

    if !options.preprocessing.remove_navigation {
        return false;
    }

    let has_nav_hint = element_has_navigation_hint(tag);

    if tag_name == "nav" {
        return true;
    }

    if tag_name == "header" {
        return is_page_header || has_nav_hint;
    }

    if tag_name == "footer" || tag_name == "aside" {
        return is_aggressive || has_nav_hint;
    }

    if is_aggressive && has_nav_hint {
        return true;
    }

    if is_aggressive {
        if element_has_noise_hint(tag) {
            return true;
        }
    }

    false
}

pub fn is_page_header(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut ancestor = dom_ctx.parent_of(node_handle.get_inner());
    let mut in_document_body = false;
    while let Some(node_id) = ancestor {
        if let Some(info) = dom_ctx.tag_info(node_id, parser) {
            if matches!(info.name.as_str(), "article" | "section" | "main") {
                return false;
            }
            in_document_body |= info.name == "body";
        }
        ancestor = dom_ctx.parent_of(node_id);
    }
    in_document_body
}

/// Check if an element has noise-related hints (ads, cookie banners, social sharing).
fn element_has_noise_hint(tag: &tl::HTMLTag) -> bool {
    const NOISE_KEYWORDS: &[&str] = &[
        "cookie",
        "consent",
        "gdpr",
        "banner",
        "advertisement",
        "ad-container",
        "advert",
        "social-share",
        "share-buttons",
        "popup",
        "modal-overlay",
        "newsletter-signup",
    ];

    attribute_matches_any(tag, "class", NOISE_KEYWORDS) || attribute_matches_any(tag, "id", NOISE_KEYWORDS)
}

#[cfg(test)]
mod tests {
    use super::{has_discarded_end_tag, readable_text_len};
    use crate::converter::utility::caching::build_dom_context;

    fn discards_an_end_tag_of_an_open_element(html: &str) -> bool {
        let dom = tl::parse(html, tl::ParserOptions::default()).expect("parse");
        let dom_ctx = build_dom_context(&dom, dom.parser(), html.len());
        has_discarded_end_tag(&dom_ctx, dom.parser(), html)
    }

    #[test]
    fn should_leave_a_discarded_end_tag_as_source_between_two_sibling_nodes() {
        let html = "<div><p>one</div>two</p>";
        let dom = tl::parse(html, tl::ParserOptions::default()).expect("parse");
        let parser = dom.parser();
        let paragraph = dom
            .nodes()
            .iter()
            .filter_map(tl::Node::as_tag)
            .find(|tag| tag.name() == "p")
            .expect("paragraph");
        let texts: Vec<(usize, &[u8])> = paragraph
            .children()
            .top()
            .iter()
            .filter_map(|child| match child.get(parser) {
                Some(tl::Node::Raw(text)) => Some(text.as_bytes()),
                _ => None,
            })
            .map(|text| (text.as_ptr() as usize - html.as_ptr() as usize, text))
            .collect();

        assert_eq!(texts, [(8, b"one".as_slice()), (17, b"two".as_slice())]);
        assert_eq!(&html[11..17], "</div>");
    }

    #[test]
    fn should_find_an_end_tag_that_the_first_parse_discarded_while_its_element_was_open() {
        for html in [
            "<div><p>one</div>two</p>",
            "<div><p>one</div>two</p>rest</div>tail",
            "<div><p>one<b>x</b></div>two</p>rest</div>tail",
            "<div><p>one</DIV >two</p>rest</div>tail",
            "<DIV><p>one</div>two</p>rest</DIV>tail",
            "<ul><li>one</ul>two</li></ul>tail",
            "<dl><dd>one</dl>two</dd></dl>tail",
            "<table><tr><td>one</tr>two</td></tr></table>tail",
            "<select><option>one</select>two</option></select>tail",
            "<html><body><div><p>one</div>two</p>rest</div>tail",
        ] {
            assert!(discards_an_end_tag_of_an_open_element(html), "{html:?}");
        }
    }

    #[test]
    fn should_find_no_discarded_end_tag_when_none_names_an_open_element() {
        let deep = format!("{}x{}", "<div>".repeat(600), "</div>".repeat(600));
        for html in [
            "",
            "<p>a</b>b",
            "<p>a</p></div>tail",
            "<div><p>one</p></div>tail",
            "<div><p>one</p><p>two</p></span></div>tail",
            "<div title='</div>'><p>x</p></div>",
            "<div><p>one<!-- </div> -->two</p></div>",
            "<div><p>one<!-- </div> two</p></div>",
            "<!doctype html><div><p>one</p>two</div>",
            "<div><pre>one\n</pre>two</div>",
            deep.as_str(),
        ] {
            assert!(!discards_an_end_tag_of_an_open_element(html), "{html:?}");
        }
    }

    fn text_len(html: &str) -> usize {
        readable_text_len(&tl::parse(html, tl::ParserOptions::default()).expect("parse"))
    }

    #[test]
    fn should_count_the_characters_of_text_that_are_not_white_space() {
        for (html, expected) in [
            ("", 0),
            ("<p> \n\t</p>", 0),
            ("<div><p>one</div>tail", 7),
            ("lead<DIV>one <b>two</b></DIV>", 10),
            ("<p>é ü</p>", 2),
            ("<!-- comment --><p>one<!-- two --></p>", 3),
            ("<p title=\"text\">one</p>", 3),
        ] {
            assert_eq!(text_len(html), expected, "{html:?}");
        }
    }

    #[test]
    fn should_count_a_character_reference_as_the_character_it_names() {
        for (html, expected) in [
            ("<p>&#65;</p>", 1),
            ("<p>&#x41;</p>", 1),
            ("<p>&amp;</p>", 1),
            ("<p>R&amp;D</p>", 3),
            ("<p>a&nbsp;b&#160;c&#xA0;d&#32;e</p>", 8),
            ("<p>a\u{a0}b</p>", 3),
            ("<p>&copy 2020</p>", 5),
        ] {
            assert_eq!(text_len(html), expected, "{html:?}");
        }
    }

    #[test]
    fn should_leave_out_the_text_that_the_converter_never_writes() {
        for name in ["script", "style", "template", "noscript", "title", "TITLE", "NoScript"] {
            let html = format!("<{name}>out <b>out</b></{name}><p>one</p>");
            assert_eq!(text_len(&html), 3, "{html:?}");
        }
        for html in [
            "<titles>four</titles><p>one</p>",
            "<header>four</header><p>one</p>",
            "<noframes>four</noframes><p>one</p>",
            "<iframe>four</iframe><p>one</p>",
        ] {
            assert_eq!(text_len(html), 7, "{html:?}");
        }
    }

    #[test]
    fn should_read_a_head_only_when_it_holds_body_content() {
        for (html, expected) in [
            ("<head>out<noframes><b>out</b></noframes></head><p>one</p>", 3),
            ("<HEAD><NOFRAMES><DIV>out</NOFRAMES></HEAD>", 0),
            ("<head><title>t</title><div>one</div><p>two</p>", 6),
            ("<HEAD><TITLE>t</TITLE><BODY><p>one</p>", 3),
            ("<header>one</header>", 3),
        ] {
            assert_eq!(text_len(html), expected, "{html:?}");
        }
    }
}
