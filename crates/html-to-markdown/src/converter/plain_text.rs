//! Plain text extraction from parsed HTML DOM.
//!
//! Provides a fast-path text extractor that walks the DOM tree collecting only
//! visible text content with structural whitespace, bypassing the full
//! Markdown/Djot conversion pipeline.

mod output;
#[cfg(test)]
mod stack_tests;
mod walker;

use std::collections::HashSet;

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
#[derive(Clone, Copy, Debug)]
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
    // ~keep The terms and the descriptions are the blocks of a definition list; the list
    // ~keep itself writes nothing of its own, so an empty list stays out of the line.
    "dt",
    "dd",
];

/// State carried by each scheduled DOM action.
///
/// Holds the options, visitor (feature-gated), and current DOM depth.
/// Using a struct avoids feature-gated function parameters at call sites.
#[derive(Clone, Copy)]
struct WalkState<'a> {
    options: &'a ConversionOptions,
    excluded_node_ids: &'a HashSet<u32>,
    /// Inside a list item, the item's buffer length right after its marker, so a block that
    /// follows the marker directly starts on its line. A table cell starts its own buffer with `None`.
    item_marker_end: Option<usize>,
    in_pre: bool,
    /// Inside the element that writes an option list: the outermost `<select>`, `<datalist>`
    /// or `<option>`. An option there is a word of that list.
    in_option_list: bool,
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
            in_option_list: self.in_option_list,
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

    /// Whether a selector of `exclude_selectors` names the node.
    fn excludes(&self, node_handle: &tl::NodeHandle) -> bool {
        self.excluded_node_ids.contains(&node_handle.get_inner())
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
        in_option_list: false,
        in_document_body: false,
        in_content_header_scope: false,
        depth: 0,
        #[cfg(feature = "visitor")]
        visitor: options.visitor.as_ref(),
    };

    let mut text = walker::extract(dom.children(), parser, state);
    post_process(&mut text);
    text
}

/// Writes a text node. `after_passed_over` says whether the walker passed over the node right
/// before the text.
fn emit_raw_text(bytes: &tl::Bytes<'_>, buf: &mut PlainOutput, state: &WalkState<'_>, after_passed_over: bool) {
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
    // ~keep A node that the walker passed over is no part of the text, so the white space on
    // ~keep its two sides is one run: the space that opens this text is not written after a
    // ~keep space. An element that was walked and wrote nothing keeps both spaces, and so does
    // ~keep a no-break space that opens the text, which `normalized` holds as a space.
    let joins_run = after_passed_over && decoded.starts_with([' ', '\t']) && buf.ends_with_space();
    let words = normalized
        .strip_prefix(' ')
        .filter(|_| joins_run)
        .unwrap_or_else(|| normalized.as_ref());
    if !(words.is_empty() || words == " " && buf.ends_with_line_end()) {
        buf.push_text(words);
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

/// Collect all descendant `NodeHandle`s matching `target_tag` (by cloning handles). An element
/// that `exclude_selectors` names is not collected, and neither is an element in it.
fn collect_descendant_handles(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    target_tag: &str,
    state: &WalkState<'_>,
    result: &mut Vec<tl::NodeHandle>,
) {
    let children = tag.children();
    let mut stack: Vec<_> = children.top().iter().copied().collect();
    stack.reverse();

    while let Some(handle) = stack.pop() {
        if state.excludes(&handle) {
            continue;
        }
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
