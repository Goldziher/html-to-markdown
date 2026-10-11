//! Plain text extraction from parsed HTML DOM.
//!
//! Provides a fast-path text extractor that walks the DOM tree collecting only
//! visible text content with structural whitespace, bypassing the full
//! Markdown/Djot conversion pipeline.

mod output;

use std::collections::HashSet;

use crate::converter::form::spacing;
use crate::converter::main_helpers::effective_max_depth;
use crate::converter::preprocessing_helpers::should_drop_for_preprocessing;
use crate::options::ConversionOptions;
use crate::text;
use output::PlainOutput;

#[cfg(feature = "visitor")]
use crate::converter::utility::content::is_block_level_element;
#[cfg(feature = "visitor")]
use crate::visitor::EMPTY_ATTRS;
#[cfg(feature = "visitor")]
use crate::visitor::{NodeContext, NodeType, VisitResult, VisitorHandle};
#[cfg(feature = "visitor")]
use output::Checkpoint;
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
const SKIP_TAGS: &[&str] = &["script", "style", "head", "template", "noscript", "math"];

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

    /// The state for text that a control writes to a buffer of its own. The marker end of a
    /// list item is a length of the item's buffer, so this state holds none.
    const fn for_own_buffer(&self) -> Self {
        WalkState {
            item_marker_end: None,
            ..*self
        }
    }
}

/// Extract plain text from a parsed DOM tree.
///
/// Walks the tree collecting visible text with structural whitespace:
/// - Block elements get blank-line separation
/// - `<br>` becomes a newline, `<hr>` a blank line
/// - `<pre>` preserves internal whitespace
/// - `<img>` outputs alt text, and an inline `<svg>` its text (unless `skip_images` is set)
/// - `<script>`, `<style>`, `<head>`, `<template>`, `<noscript>` are skipped
/// - Tables: cells separated by tab, rows by newline
/// - Inline elements are recursed without markers
/// - A form control that writes text is a word of its own, and so is each option of a list
/// - Nodes matching `excluded_node_ids` (from `exclude_selectors`) are dropped entirely
/// - When a visitor is configured, `visit_element_start`, `visit_element_end`, and
///   `visit_text` callbacks are fired and their results are honoured.
pub fn extract_plain_text(dom: &tl::VDom, parser: &tl::Parser, options: &ConversionOptions) -> String {
    let mut buf = PlainOutput::with_capacity(1024);
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

    let mut text = buf.into_text();
    post_process(&mut text);
    text
}

/// Recursive plain-text walker.
fn walk_plain(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
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

fn emit_raw_text(bytes: &tl::Bytes<'_>, buf: &mut PlainOutput, state: &WalkState<'_>) {
    let raw = bytes.as_utf8_str();
    let decoded = text::decode_html_entities_cow(raw.as_ref());
    #[cfg(feature = "visitor")]
    if visit_plain_text(&decoded, buf, state) {
        return;
    }
    if state.in_pre {
        buf.push_text(&decoded);
        return;
    }
    let normalized = text::normalize_whitespace_cow(&decoded);
    if !(normalized.is_empty() || normalized.as_ref() == " " && buf.ends_with_line_end()) {
        buf.push_text(&normalized);
    }
}

#[cfg(feature = "visitor")]
fn visit_plain_text(decoded: &str, buf: &mut PlainOutput, state: &WalkState<'_>) -> bool {
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
            buf.push_text(&custom);
            true
        }
        _ => false,
    }
}

fn walk_tag(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
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
    // ~keep The checkpoint is open while the element writes. `visit_plain_element_end` closes
    // ~keep it on each of its paths, and `dispatch_plain_tag` has no other way out.
    #[cfg(feature = "visitor")]
    let element_start = state.visitor.map(|visitor_handle| (visitor_handle, buf.open()));
    dispatch_plain_tag(tag_str, tag, parser, buf, list_ctx, state);
    #[cfg(feature = "visitor")]
    if let Some((visitor_handle, element_start)) = element_start {
        visit_plain_element_end(visitor_handle, tag_str, tag, buf, state.depth, element_start);
    }
}

#[cfg(feature = "visitor")]
fn visit_plain_element_start(
    tag_name: &str,
    tag: &tl::HTMLTag<'_>,
    buf: &mut PlainOutput,
    state: &WalkState<'_>,
) -> bool {
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
    let start = buf.open();
    buf.push_text(&custom);
    let end_result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_element_end(&node_ctx, &custom);
    close_plain_checkpoint(buf, start, end_result);
    true
}

/// Calls the visitor at the end of an element and closes `element_start`, the checkpoint
/// opened before the element wrote.
#[cfg(feature = "visitor")]
fn visit_plain_element_end(
    visitor_handle: &VisitorHandle,
    tag_name: &str,
    tag: &tl::HTMLTag<'_>,
    buf: &mut PlainOutput,
    depth: usize,
    element_start: Checkpoint,
) {
    let node_ctx = plain_element_context(tag_name, tag, depth);
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_element_end(&node_ctx, buf.since(&element_start));
    close_plain_checkpoint(buf, element_start, result);
}

/// Closes `checkpoint` as the visitor answered at the end of an element: its text takes the
/// place of the text written since, a skip removes that text, and every other answer keeps it.
#[cfg(feature = "visitor")]
fn close_plain_checkpoint(buf: &mut PlainOutput, checkpoint: Checkpoint, end_result: VisitResult) {
    match end_result {
        VisitResult::Custom(custom) => {
            buf.rollback(checkpoint);
            buf.push_text(&custom);
        }
        VisitResult::Skip => buf.rollback(checkpoint),
        _ => buf.commit(checkpoint),
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
    buf: &mut PlainOutput,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    let child_state = WalkState {
        in_document_body: state.in_document_body || tag_name == "body",
        in_content_header_scope: state.in_content_header_scope || matches!(tag_name, "article" | "section" | "main"),
        ..state.descend()
    };
    match tag_name {
        "br" => buf.push_mark("\n"),
        "hr" => buf.end_block(),
        "pre" => walk_plain_pre(tag, parser, buf, list_ctx, state),
        "img" => emit_plain_image(tag, buf, state.options),
        // ~keep An inline graphic is an image: its text is written like an alt text, and its
        // ~keep style sheets and scripts are not text of the page.
        "svg" => {
            if !state.options.skip_images {
                buf.push_text(&crate::converter::media::svg::graphic_text(tag, parser));
            }
        }
        "table" => {
            buf.end_block();
            walk_table(tag, parser, buf, &child_state);
            buf.end_block();
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
                buf.end_block();
            }
            walk_children(tag, parser, buf, list_ctx, &child_state);
            buf.end_block();
        }
        // ~keep The Markdown writer writes an option list and an option with the same two functions.
        "select" | "datalist" => walk_plain_option_list(tag, parser, buf, list_ctx, &child_state.for_own_buffer()),
        "option" => {
            let mut option = PlainOutput::default();
            walk_children(tag, parser, &mut option, list_ctx, &child_state.for_own_buffer());
            buf.write_option(&option.into_text());
        }
        _ if spacing::writes_own_text(tag_name) => {
            walk_plain_control(tag, parser, buf, list_ctx, &child_state);
        }
        _ => walk_children(tag, parser, buf, list_ctx, &child_state),
    }
}

/// Writes the options of a select list or a data list. The list is one control in its line:
/// one space separates it from the text before it and from a word after it.
fn walk_plain_option_list(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    let mut options = PlainOutput::default();
    walk_children(tag, parser, &mut options, list_ctx, state);
    // ~keep This writer keeps the line ends of the source in running text: outside `pre`, a
    // ~keep line end in a list is one space.
    let mut options = options.into_text();
    if !state.in_pre {
        options = options.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    buf.write_option_list(&options, state.in_pre);
}

/// Writes a control that holds text of its own: a button, an output, a meter, a progress bar
/// or a text area. The text is a word of its own: one space separates it from the text before
/// it and from a word after it in the same line. In a `pre` element the text is written as
/// the source has it.
fn walk_plain_control(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    if state.in_pre {
        walk_children(tag, parser, buf, list_ctx, state);
        return;
    }
    let control = buf.start_control();
    walk_children(tag, parser, buf, list_ctx, state);
    buf.finish_control(control);
}

fn walk_plain_pre(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    buf.end_block();
    let pre_state = WalkState {
        in_pre: true,
        ..state.descend()
    };
    walk_children(tag, parser, buf, list_ctx, &pre_state);
    buf.end_block();
}

fn emit_plain_image(tag: &tl::HTMLTag<'_>, buf: &mut PlainOutput, options: &ConversionOptions) {
    if options.skip_images {
        return;
    }
    if let Some(alt) = crate::converter::utility::attributes::decoded_attribute(tag, "alt")
        && !alt.is_empty()
    {
        buf.push_text(alt.as_ref());
    }
}

fn walk_plain_list(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
    mut list_context: ListContext,
    state: &WalkState<'_>,
) {
    buf.end_line();
    walk_children(tag, parser, buf, &mut list_context, state);
    buf.end_line();
}

fn walk_plain_list_item(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
    list_ctx: &mut ListContext,
    state: &WalkState<'_>,
) {
    buf.end_line();
    match list_ctx {
        ListContext::Unordered | ListContext::None => buf.push_mark("- "),
        ListContext::Ordered { next_index } => {
            buf.push_mark(&format!("{next_index}. "));
            *next_index += 1;
        }
    }
    let item_state = WalkState {
        item_marker_end: Some(buf.len()),
        in_pre: false,
        ..state.descend()
    };
    walk_children(tag, parser, buf, list_ctx, &item_state);
    buf.end_line();
}

/// Walk all children of a tag.
fn walk_children(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    buf: &mut PlainOutput,
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
fn walk_table(table_tag: &tl::HTMLTag, parser: &tl::Parser, buf: &mut PlainOutput, state: &WalkState<'_>) {
    let mut row_handles = Vec::new();
    collect_descendant_handles(table_tag, parser, "tr", &mut row_handles);

    for (row_idx, row_handle) in row_handles.iter().enumerate() {
        if row_idx > 0 {
            buf.push_mark("\n");
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
                buf.push_mark("\t");
            }
            let mut cell_buf = PlainOutput::default();
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let mut cell_list_ctx = ListContext::None;
                walk_children(cell_tag, parser, &mut cell_buf, &mut cell_list_ctx, &cell_state);
            }
            buf.push_text(cell_buf.into_text().trim());
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
            if child_tag.name().as_utf8_str().eq_ignore_ascii_case(target_tag) {
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
