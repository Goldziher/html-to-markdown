//! Plain text extraction from parsed HTML DOM.
//!
//! Provides a fast-path text extractor that walks the DOM tree collecting only
//! visible text content with structural whitespace, bypassing the full
//! Markdown/Djot conversion pipeline.

use std::collections::HashSet;
use std::fmt::Write;

use crate::converter::main_helpers::effective_max_depth;
use crate::converter::preprocessing_helpers::should_drop_for_preprocessing;
use crate::options::ConversionOptions;
use crate::text;

#[cfg(feature = "visitor")]
use crate::converter::utility::content::is_block_level_element;
#[cfg(feature = "visitor")]
use crate::visitor::EMPTY_ATTRS;
#[cfg(feature = "visitor")]
use crate::visitor::{NodeContext, NodeType, VisitResult, VisitorHandle};
#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Tracks list context for proper marker emission on `<li>` elements.
#[derive(Clone, Debug)]
enum ListContext {
    /// Not inside any list.
    None,
    /// Inside `<ul>` — each `<li>` gets a `- ` prefix.
    Unordered,
    /// Inside `<ol>` — each `<li>` gets a sequential `N. ` prefix.
    /// The `next_index` is incremented after each `<li>`.
    Ordered { next_index: u32 },
}

/// Tags whose content should be skipped entirely.
const SKIP_TAGS: &[&str] = &["script", "style", "head", "template", "noscript", "svg", "math"];

/// Block-level tags that should be separated by blank lines.
const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "blockquote",
    "section",
    "article",
    "aside",
    "main",
    "nav",
    "header",
    "footer",
    "figure",
    "figcaption",
    "details",
    "summary",
    "address",
    "hgroup",
    "search",
    "center",
    "dialog",
    "menu",
    "legend",
];

/// Shared walker state threaded through all recursive calls.
///
/// Holds the options, visitor (feature-gated), and current DOM depth.
/// Using a struct avoids feature-gated function parameters at call sites.
struct WalkState<'a> {
    options: &'a ConversionOptions,
    excluded_node_ids: &'a HashSet<u32>,
    /// Inside a list item, the item's buffer length right after its marker, so a block that
    /// follows the marker directly starts on its line. A table cell starts its own buffer with `None`.
    item_marker_end: Option<usize>,
    in_pre: bool,
    in_document_body: bool,
    in_content_header_scope: bool,
    depth: usize,
    #[cfg(feature = "visitor")]
    visitor: Option<&'a VisitorHandle>,
}

impl WalkState<'_> {
    const fn descend(&self) -> Self {
        WalkState {
            options: self.options,
            excluded_node_ids: self.excluded_node_ids,
            item_marker_end: self.item_marker_end,
            in_pre: self.in_pre,
            in_document_body: self.in_document_body,
            in_content_header_scope: self.in_content_header_scope,
            depth: self.depth + 1,
            #[cfg(feature = "visitor")]
            visitor: self.visitor,
        }
    }
}

/// Extract plain text from a parsed DOM tree.
///
/// Walks the tree collecting visible text with structural whitespace:
/// - Block elements get blank-line separation
/// - `<br>` becomes a newline, `<hr>` a blank line
/// - `<pre>` preserves internal whitespace
/// - `<img>` outputs alt text (unless `skip_images` is set)
/// - `<script>`, `<style>`, `<head>`, `<template>`, `<noscript>` are skipped
/// - Tables: cells separated by tab, rows by newline
/// - Inline elements are recursed without markers
/// - Nodes matching `excluded_node_ids` (from `exclude_selectors`) are dropped entirely
/// - When a visitor is configured, `visit_element_start`, `visit_element_end`, and
///   `visit_text` callbacks are fired and their results are honoured.
pub fn extract_plain_text(dom: &tl::VDom, parser: &tl::Parser, options: &ConversionOptions) -> String {
    let mut buf = String::with_capacity(1024);
    let mut list_ctx = ListContext::None;

    let excluded_node_ids: HashSet<u32> = if options.exclude_selectors.is_empty() {
        HashSet::new()
    } else {
        let mut ids = HashSet::new();
        for selector in &options.exclude_selectors {
            if let Some(iter) = dom.query_selector(selector) {
                for handle in iter {
                    ids.insert(handle.get_inner());
                }
            }
        }
        ids
    };

    let state = WalkState {
        options,
        excluded_node_ids: &excluded_node_ids,
        item_marker_end: None,
        in_pre: false,
        in_document_body: false,
        in_content_header_scope: false,
        depth: 0,
        #[cfg(feature = "visitor")]
        visitor: options.visitor.as_ref(),
    };

    for child_handle in dom.children() {
        walk_plain(child_handle, parser, &mut buf, &mut list_ctx, &state);
    }

    post_process(&mut buf);
    buf
}

/// Recursive plain-text walker.
fn walk_plain(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    if state.depth >= effective_max_depth(state.options) {
        return;
    }

    let Some(node) = node_handle.get(parser) else {
        return;
    };

    match node {
        tl::Node::Raw(bytes) => emit_raw_text(bytes, buf, state),
        tl::Node::Tag(tag) => {
            if state.excluded_node_ids.contains(&node_handle.get_inner()) {
                return;
            }
            walk_tag(tag, parser, buf, list_ctx, state);
        }
        tl::Node::Comment(_) => {}
    }
}

fn emit_raw_text(bytes: &tl::Bytes<'_>, buf: &mut String, state: &WalkState<'_>) {
    let raw = bytes.as_utf8_str();
    let decoded = text::decode_html_entities_cow(raw.as_ref());
    #[cfg(feature = "visitor")]
    if visit_plain_text(&decoded, buf, state) {
        return;
    }
    if state.in_pre {
        buf.push_str(&decoded);
        return;
    }
    let normalized = text::normalize_whitespace_cow(&decoded);
    if !(normalized.is_empty() || normalized.as_ref() == " " && buf.ends_with('\n')) {
        buf.push_str(&normalized);
    }
}

#[cfg(feature = "visitor")]
fn visit_plain_text(decoded: &str, buf: &mut String, state: &WalkState<'_>) -> bool {
    let Some(visitor_handle) = state.visitor else {
        return false;
    };
    let node_ctx = NodeContext::with_borrowed_attributes(
        NodeType::Text,
        Cow::Borrowed(""),
        &EMPTY_ATTRS,
        state.depth,
        0,
        None,
        true,
    );
    let visit_result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_text(&node_ctx, decoded);
    match visit_result {
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            buf.push_str(&custom);
            true
        }
        _ => false,
    }
}

fn walk_tag(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    let tag_name = tag.name().as_utf8_str().to_ascii_lowercase();
    let tag_str = tag_name.as_str();
    if SKIP_TAGS.contains(&tag_str)
        || should_drop_for_preprocessing(
            tag_str,
            tag,
            state.options,
            tag_str == "header" && state.in_document_body && !state.in_content_header_scope,
        )
    {
        return;
    }
    #[cfg(feature = "visitor")]
    if visit_plain_element_start(tag_str, tag, buf, state) {
        return;
    }
    #[cfg(feature = "visitor")]
    let element_output_start = buf.len();
    dispatch_plain_tag(tag_str, tag, parser, buf, list_ctx, state);
    #[cfg(feature = "visitor")]
    visit_plain_element_end(tag_str, tag, buf, state, element_output_start);
}

#[cfg(feature = "visitor")]
fn visit_plain_element_start(tag_name: &str, tag: &tl::HTMLTag<'_>, buf: &mut String, state: &WalkState<'_>) -> bool {
    let Some(visitor_handle) = state.visitor else {
        return false;
    };
    let node_ctx = plain_element_context(tag_name, tag, state.depth);
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_element_start(&node_ctx);
    let VisitResult::Custom(custom) = result else {
        return matches!(result, VisitResult::Skip);
    };
    buf.push_str(&custom);
    let end_result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_element_end(&node_ctx, &custom);
    if let VisitResult::Custom(replacement) = end_result {
        buf.truncate(buf.len() - custom.len());
        buf.push_str(&replacement);
    } else if matches!(end_result, VisitResult::Skip) {
        buf.truncate(buf.len() - custom.len());
    }
    true
}

#[cfg(feature = "visitor")]
fn visit_plain_element_end(
    tag_name: &str,
    tag: &tl::HTMLTag<'_>,
    buf: &mut String,
    state: &WalkState<'_>,
    output_start: usize,
) {
    let Some(visitor_handle) = state.visitor else {
        return;
    };
    let node_ctx = plain_element_context(tag_name, tag, state.depth);
    let safe_start = output_start.min(buf.len());
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_element_end(&node_ctx, &buf[safe_start..]);
    match result {
        VisitResult::Custom(custom) => {
            buf.truncate(safe_start);
            buf.push_str(&custom);
        }
        VisitResult::Skip => buf.truncate(safe_start),
        _ => {}
    }
}

#[cfg(feature = "visitor")]
fn plain_element_context<'a>(tag_name: &'a str, tag: &'a tl::HTMLTag<'a>, depth: usize) -> NodeContext<'a> {
    NodeContext::with_lazy_attributes(
        NodeType::Element,
        Cow::Borrowed(tag_name),
        tag,
        depth,
        0,
        None,
        !is_block_level_element(tag_name),
    )
}

fn dispatch_plain_tag(
    tag_name: &str,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    let child_state = WalkState {
        in_document_body: state.in_document_body || tag_name == "body",
        in_content_header_scope: state.in_content_header_scope || matches!(tag_name, "article" | "section" | "main"),
        ..state.descend()
    };
    match tag_name {
        "br" => buf.push('\n'),
        "hr" => ensure_blank_line(buf),
        "pre" => walk_plain_pre(tag, parser, buf, list_ctx, state),
        "img" => emit_plain_image(tag, buf, state.options),
        "table" => {
            ensure_blank_line(buf);
            walk_table(tag, parser, buf, &child_state);
            ensure_blank_line(buf);
        }
        "ul" => {
            let list_state = WalkState {
                in_pre: false,
                ..child_state
            };
            walk_plain_list(tag, parser, buf, ListContext::Unordered, &list_state);
        }
        "ol" => {
            let start = tag
                .attributes()
                .get("start")
                .flatten()
                .and_then(|value| value.as_utf8_str().parse::<u32>().ok())
                .unwrap_or(1);
            let list_state = WalkState {
                in_pre: false,
                ..child_state
            };
            walk_plain_list(
                tag,
                parser,
                buf,
                ListContext::Ordered { next_index: start },
                &list_state,
            );
        }
        "li" => walk_plain_list_item(tag, parser, buf, list_ctx, state),
        _ if BLOCK_TAGS.contains(&tag_name) => {
            if state.item_marker_end != Some(buf.len()) {
                ensure_blank_line(buf);
            }
            walk_children(tag, parser, buf, list_ctx, &child_state);
            ensure_blank_line(buf);
        }
        _ => walk_children(tag, parser, buf, list_ctx, &child_state),
    }
}

fn walk_plain_pre(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    ensure_blank_line(buf);
    let pre_state = WalkState {
        in_pre: true,
        ..state.descend()
    };
    walk_children(tag, parser, buf, list_ctx, &pre_state);
    ensure_blank_line(buf);
}

fn emit_plain_image(tag: &tl::HTMLTag<'_>, buf: &mut String, options: &ConversionOptions) {
    if options.skip_images {
        return;
    }
    if let Some(alt) = crate::converter::utility::attributes::decoded_attribute(tag, "alt")
        && !alt.is_empty()
    {
        buf.push_str(alt.as_ref());
    }
}

fn walk_plain_list(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut String,
    mut list_context: ListContext,
    state: &WalkState<'_>,
) {
    ensure_newline(buf);
    walk_children(tag, parser, buf, &mut list_context, state);
    ensure_newline(buf);
}

fn walk_plain_list_item(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    ensure_newline(buf);
    match list_ctx {
        ListContext::Unordered | ListContext::None => buf.push_str("- "),
        ListContext::Ordered { next_index } => {
            let _ = write!(buf, "{next_index}. ");
            *next_index += 1;
        }
    }
    let item_state = WalkState {
        item_marker_end: Some(buf.len()),
        in_pre: false,
        ..state.descend()
    };
    walk_children(tag, parser, buf, list_ctx, &item_state);
    ensure_newline(buf);
}

/// Walk all children of a tag.
fn walk_children(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    buf: &mut String,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    let children = tag.children();
    let top = children.top();
    for child in top.iter() {
        walk_plain(child, parser, buf, list_ctx, state);
    }
}

/// Walk a `<table>` element, extracting cells as tab-separated, rows as newline-separated.
fn walk_table(table_tag: &tl::HTMLTag, parser: &tl::Parser, buf: &mut String, state: &WalkState<'_>) {
    let mut row_handles = Vec::new();
    collect_descendant_handles(table_tag, parser, "tr", &mut row_handles);

    for (row_idx, row_handle) in row_handles.iter().enumerate() {
        if row_idx > 0 {
            buf.push('\n');
        }
        let Some(tl::Node::Tag(row_tag)) = row_handle.get(parser) else {
            continue;
        };

        let mut cell_handles = Vec::new();
        let row_children = row_tag.children();
        let row_top = row_children.top();
        for child in row_top.iter() {
            if let Some(tl::Node::Tag(child_tag)) = child.get(parser) {
                let name = child_tag.name().as_utf8_str();
                if name.eq_ignore_ascii_case("th") || name.eq_ignore_ascii_case("td") {
                    cell_handles.push(*child);
                }
            }
        }

        let cell_state = WalkState {
            item_marker_end: None,
            in_pre: false,
            ..state.descend()
        };
        for (cell_idx, cell_handle) in cell_handles.iter().enumerate() {
            if cell_idx > 0 {
                buf.push('\t');
            }
            let mut cell_buf = String::new();
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let mut cell_list_ctx = ListContext::None;
                walk_children(cell_tag, parser, &mut cell_buf, &mut cell_list_ctx, &cell_state);
            }
            buf.push_str(cell_buf.trim());
        }
    }
}

/// Collect all descendant `NodeHandle`s matching `target_tag` (by cloning handles).
fn collect_descendant_handles(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    target_tag: &str,
    result: &mut Vec<tl::NodeHandle>,
) {
    let children = tag.children();
    let mut stack: Vec<_> = children.top().iter().copied().collect();
    stack.reverse();

    while let Some(handle) = stack.pop() {
        if let Some(tl::Node::Tag(child_tag)) = handle.get(parser) {
            let name = child_tag.name().as_utf8_str();
            if SKIP_TAGS.iter().any(|skip| name.eq_ignore_ascii_case(skip)) {
                continue;
            }
            if name.eq_ignore_ascii_case(target_tag) {
                result.push(handle);
            } else {
                let child_children = child_tag.children();
                let mut child_handles: Vec<_> = child_children.top().iter().copied().collect();
                child_handles.reverse();
                for child in child_handles {
                    stack.push(child);
                }
            }
        }
    }
}

/// Ensure the buffer ends with a blank line (two newlines).
fn ensure_blank_line(buf: &mut String) {
    if buf.is_empty() {
        return;
    }
    while buf.ends_with(' ') || buf.ends_with('\t') {
        buf.pop();
    }
    let current_newlines = buf.chars().rev().take_while(|&c| c == '\n').count();
    for _ in current_newlines..2 {
        buf.push('\n');
    }
}

/// Ensure the buffer ends with at least one newline.
fn ensure_newline(buf: &mut String) {
    if buf.is_empty() {
        return;
    }
    if !buf.ends_with('\n') {
        buf.push('\n');
    }
}

/// Single-pass post-processor: trims trailing whitespace per line, collapses runs of 3+
/// newlines to exactly 2 (one blank line between paragraphs), and normalizes the trailing
/// newline. Uses `str::lines()` so multibyte UTF-8 codepoints are never split mid-character
/// — the original byte-oriented implementation cast `u8 as char`, mangling anything outside
/// ASCII (see issue #362).
///
/// Empty / whitespace-only input lines are folded into the surrounding newline run, so any
/// number of space-only lines between paragraphs collapses to a single blank line.
///
/// Algorithm contributed by @xitep in
/// <https://github.com/xberg-io/html-to-markdown/issues/362>: 2–7× faster than the
/// previous char-by-char rewrite while preserving identical semantics.
fn normalize_plain_output(buf: &mut String) {
    let input = std::mem::take(buf);
    let mut out = String::with_capacity(input.len());
    let mut last_was_blank = false;
    for line in input.lines().map(str::trim_end) {
        if line.is_empty() {
            if !last_was_blank {
                out.push('\n');
                last_was_blank = true;
            }
        } else {
            out.push_str(line);
            out.push('\n');
            last_was_blank = false;
        }
    }
    let keep = out.trim_end_matches('\n').len();
    out.truncate(keep);
    if !out.is_empty() {
        out.push('\n');
    }
    *buf = out;
}

/// Post-process the accumulated plain-text buffer.
fn post_process(buf: &mut String) {
    normalize_plain_output(buf);
}
