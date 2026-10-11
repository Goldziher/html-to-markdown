//! Code and pre element handlers for HTML to Markdown conversion.
//!
//! Handles `<code>` and `<pre>` elements including:
//! - Inline code with backtick formatting
//! - Code block formatting (indented or fenced)
//! - Language detection from class attributes
//! - Whitespace normalization and dedenting
//! - Visitor callback integration

use crate::converter::Context;
use crate::converter::dom_context::DomContext;
use crate::converter::inline::HandlerContext;
use crate::converter::inline::wrapped::emit_code_span;
use crate::converter::main::walk_node;
use crate::converter::text::dedent_code_block;
use crate::options::ConversionOptions;

#[cfg(feature = "visitor")]
use crate::converter::utility::serialization::serialize_node;
#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Minimum length of a Markdown code fence (```` ``` ```` or `~~~`) per CommonMark.
const MIN_FENCE_LENGTH: usize = 3;

/// Compute the length of the longest consecutive run of `marker` in `content`.
fn longest_consecutive_run(content: &str, marker: char) -> usize {
    content
        .chars()
        .fold((0usize, 0usize), |(max, current), c| {
            if c == marker {
                let next = current + 1;
                (max.max(next), next)
            } else {
                (max, 0)
            }
        })
        .0
}

/// Smallest backtick-run length (starting at 1) that does not occur as a run inside `content`.
///
/// CommonMark closes an inline code span at the next backtick string of the *same* length as
/// the opening delimiter (6.1) — a longer or shorter run never matches. So the delimiter only
/// needs to avoid colliding with a run length that actually appears in `content`; it does not
/// need to exceed the longest run (unlike a fenced block, whose closing rule matches on *any*
/// run at least as long as the fence). Picking `longest_run + 1` unconditionally over-escapes:
/// content `` `` `` (a single length-2 run, no length-1 run) is valid with a single backtick.
fn min_safe_code_span_delimiter_length(content: &str) -> usize {
    let mut run_lengths = std::collections::HashSet::new();
    let mut current = 0usize;
    for c in content.chars() {
        if c == '`' {
            current += 1;
        } else {
            if current > 0 {
                run_lengths.insert(current);
            }
            current = 0;
        }
    }
    if current > 0 {
        run_lengths.insert(current);
    }

    let mut candidate = 1usize;
    while run_lengths.contains(&candidate) {
        candidate += 1;
    }
    candidate
}

/// Handle an inline `<code>` element and convert to Markdown.
///
/// This handler processes inline code elements including:
/// - Extracting code content and applying backtick delimiters
/// - Handling backticks in content by using multiple delimiters
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Generating appropriate markdown output with proper escaping
pub fn handle_code(tag: &tl::HTMLTag, mut handler: HandlerContext<'_>) {
    let links = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let code_ctx = Context {
        in_code: true,
        inline_code_links: if handler.context.in_code {
            handler.context.inline_code_links.clone()
        } else if contains_anchor(tag, handler.parser) {
            Some(links.clone())
        } else {
            None
        },
        ..handler.context.clone()
    };
    if handler.context.in_code {
        walk_children_to_output(tag, &code_ctx, &mut handler);
        return;
    }
    let mut content = String::with_capacity(32);
    walk_children(tag, &mut content, &code_ctx, &handler);
    // ~keep An all-whitespace body is a real code span, not an empty element (#481).
    if content.is_empty() {
        return;
    }

    #[cfg(feature = "visitor")]
    if tag.name().as_utf8_str().eq_ignore_ascii_case("code") {
        if let Some(custom_output) = visit_inline_code(tag, &content, &handler) {
            handler.output.push_str(&custom_output);
            return;
        }
    }
    emit_inline_code_with_links(&content, &links.borrow(), &mut handler);
}

pub(in crate::converter) fn contains_anchor(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> bool {
    let mut pending = tag.children().top().to_vec();
    while let Some(handle) = pending.pop() {
        if let Some(tl::Node::Tag(child)) = handle.get(parser) {
            if child.name().as_utf8_str().eq_ignore_ascii_case("a") {
                return true;
            }
            pending.extend(child.children().top().iter().copied());
        }
    }
    false
}

fn emit_inline_code_with_links(
    content: &str,
    links: &[crate::converter::context::InlineCodeLink],
    handler: &mut HandlerContext<'_>,
) {
    if links.is_empty() {
        emit_code_content(content, handler);
        return;
    }
    let mut start = 0;
    let mut pending_break = false;
    let mut emitted = false;
    for link in links {
        emit_linked_code_text(
            &content[start..link.range.start],
            &mut pending_break,
            &mut emitted,
            handler,
        );
        if pending_break && emitted {
            emit_collected_code_break(handler);
        }
        pending_break = false;
        handler.output.push_str(&link.markdown);
        emitted = true;
        start = link.range.end;
    }
    emit_linked_code_text(&content[start..], &mut pending_break, &mut emitted, handler);
}

fn emit_linked_code_text(
    content: &str,
    pending_break: &mut bool,
    emitted: &mut bool,
    handler: &mut HandlerContext<'_>,
) {
    let break_count = content.matches('\n').count();
    for (index, segment) in content.split('\n').enumerate() {
        *pending_break |= index > 0;
        if segment.is_empty() {
            continue;
        }
        if *pending_break && *emitted {
            emit_collected_code_break(handler);
        }
        let segment = if (handler.context.in_table_cell || handler.context.in_heading) && index < break_count {
            segment.trim_end()
        } else {
            segment
        };
        if segment.is_empty() {
            continue;
        }
        emit_code_content(segment, handler);
        *emitted = true;
        *pending_break = false;
    }
}

fn emit_collected_code_break(handler: &mut HandlerContext<'_>) {
    if handler.context.in_table_cell || handler.context.in_heading {
        handler.output.push(' ');
    } else {
        handler
            .output
            .push_str(crate::converter::main_helpers::hard_break_marker(handler.options));
    }
}

fn emit_code_content(content: &str, handler: &mut HandlerContext<'_>) {
    emit_inline_code(
        content,
        handler.output,
        handler.options,
        handler.node_handle,
        handler.parser,
        handler.dom_context,
    );
}

fn walk_children_to_output(tag: &tl::HTMLTag<'_>, context: &Context, handler: &mut HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            handler.parser,
            handler.output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
}

fn walk_children(tag: &tl::HTMLTag<'_>, output: &mut String, context: &Context, handler: &HandlerContext<'_>) {
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            handler.parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                handler.options,
                context,
                handler.depth + 1,
                handler.dom_context,
            ),
        );
    }
}

#[cfg(feature = "visitor")]
fn visit_inline_code(tag: &tl::HTMLTag<'_>, content: &str, handler: &HandlerContext<'_>) -> Option<String> {
    use crate::visitor::{NodeContext, NodeType};

    let visitor_handle = handler.context.visitor.as_ref()?;
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        NodeType::Code,
        Cow::Borrowed("code"),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        true,
    );
    let result = visitor_handle
        .lock()
        .expect("visitor mutex poisoned")
        .visit_code_inline(&node_context, content);
    visitor_output(result, handler)
}

/// Render a `<code>` element's content as one or more backtick spans, then emit through the
/// adjacent-span merge.
///
/// `content` may contain `'\n'` bytes: `line_break.rs`'s code-SPAN branch pushes one as an
/// internal-only split marker for each `<br>` the element contained (a source text node's own
/// literal line ending is already folded to a space by the time it reaches this buffer, so
/// every `'\n'` here unambiguously came from a `<br>` — see `text_node.rs`'s
/// `in_code && !in_code_block` branch). Split on it and render each segment as its own
/// backtick span, joined by the configured `newline_style` hard-break marker OUTSIDE the
/// backticks, where it is syntax rather than span content (issue #487) — matching how
/// `<b>`/`<i>` already turn an internal `<br>` into a hard break between two delimiter pairs.
/// An empty segment (an adjacent, leading, or trailing `<br>`) is dropped rather than emitted
/// as a dangling empty `` `` `` pair with nothing before or after it. ~keep
pub(in crate::converter) fn emit_inline_code(
    content: &str,
    output: &mut String,
    options: &ConversionOptions,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) {
    let separator = crate::converter::main_helpers::hard_break_marker(options);
    let mut first = true;
    for segment in content.split('\n').filter(|segment| !segment.is_empty()) {
        if first {
            let mut span = String::with_capacity(segment.len() + 2);
            format_inline_code(segment, &mut span);
            emit_code_span(&span, segment, output, node_handle, parser, dom_ctx);
        } else {
            // ~keep Only the FIRST segment may merge into an immediately preceding
            // ~keep sibling code span (issue #483): every later segment is preceded by
            // ~keep our own separator, not a bare closing backtick, so `emit_code_span`'s
            // ~keep merge check would never fire for it anyway -- rendered directly
            // ~keep without going through that check at all.
            output.push_str(separator);
            format_inline_code(segment, output);
        }
        first = false;
    }
}

/// Handle a `<pre>` element and convert to Markdown.
///
/// This handler processes code block elements including:
/// - Extracting language information from class attributes
/// - Processing whitespace and dedenting code content
/// - Supporting multiple code block styles (indented, backticks, tildes)
/// - Invoking visitor callbacks when the visitor feature is enabled
/// - Generating appropriate markdown output
pub fn handle_pre(tag: &tl::HTMLTag, handler: HandlerContext<'_>) {
    let cell_break_offsets = (handler.context.in_table_cell && handler.options.br_in_tables)
        .then(|| std::rc::Rc::new(std::cell::RefCell::new(Vec::new())));
    let code_ctx = Context {
        in_code: true,
        in_code_block: true,
        pre_cell_break_offsets: cell_break_offsets.clone(),
        ..handler.context.clone()
    };
    let language = detect_language(tag, handler.parser);
    let mut content = String::with_capacity(256);
    walk_children(tag, &mut content, &code_ctx, &handler);
    if content.is_empty() {
        return;
    }

    let offsets = cell_break_offsets
        .as_ref()
        .map(|offsets| offsets.borrow().clone())
        .unwrap_or_default();
    let segmented = (!offsets.is_empty()).then(|| content.clone());
    let processed = process_pre_content(content, handler.options.whitespace_mode);
    #[cfg(feature = "visitor")]
    if let Some(custom_output) = visit_code_block(tag, language.as_deref(), &processed, &handler) {
        handler.output.push_str(&custom_output);
    } else {
        format_code_block(
            segmented.as_deref().unwrap_or(&processed),
            &offsets,
            language.as_deref(),
            handler.output,
            handler.options,
            handler.context,
        );
    }
    #[cfg(not(feature = "visitor"))]
    format_code_block(
        segmented.as_deref().unwrap_or(&processed),
        &offsets,
        language.as_deref(),
        handler.output,
        handler.options,
        handler.context,
    );
    if let Some(ref collector) = handler.context.structure_collector {
        collector.borrow_mut().push_code(&processed, language.as_deref());
    }
}

fn detect_language(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Option<String> {
    language_from_class(tag).or_else(|| {
        tag.children().top().iter().find_map(|child_handle| {
            let tl::Node::Tag(child_tag) = child_handle.get(parser)? else {
                return None;
            };
            (child_tag.name() == "code")
                .then(|| language_from_class(child_tag))
                .flatten()
        })
    })
}

fn language_from_class(tag: &tl::HTMLTag<'_>) -> Option<String> {
    let classes = tag.attributes().get("class")??;
    let class_text = classes.as_utf8_str();
    let classes = crate::text::decode_attribute_value_cow(&class_text);
    classes.split_whitespace().find_map(|class| {
        class
            .strip_prefix("language-")
            .or_else(|| class.strip_prefix("lang-"))
            .map(str::to_string)
    })
}

fn process_pre_content(content: String, whitespace_mode: crate::options::WhitespaceMode) -> String {
    if whitespace_mode == crate::options::WhitespaceMode::Strict {
        return content;
    }
    let leading_newlines = content.chars().take_while(|&character| character == '\n').count();
    let trailing_newlines = content.chars().rev().take_while(|&character| character == '\n').count();
    let core = content.trim_matches('\n');
    let mut processed = dedent_code_block(core);
    if core.trim().is_empty() {
        processed.insert_str(0, &"\n".repeat(leading_newlines));
    }
    processed.push_str(&"\n".repeat(trailing_newlines));
    processed
}

#[cfg(feature = "visitor")]
fn visit_code_block(
    tag: &tl::HTMLTag<'_>,
    language: Option<&str>,
    content: &str,
    handler: &HandlerContext<'_>,
) -> Option<String> {
    use crate::visitor::{NodeContext, NodeType};

    let visitor_handle = handler.context.visitor.as_ref()?;
    let node_id = handler.node_handle.get_inner();
    let node_context = NodeContext::with_lazy_attributes(
        NodeType::Pre,
        Cow::Borrowed("pre"),
        tag,
        handler.depth,
        handler.dom_context.get_sibling_index(node_id).unwrap_or(0),
        handler
            .dom_context
            .parent_tag_name(node_id, handler.parser)
            .map(Cow::Borrowed),
        false,
    );
    let result =
        visitor_handle
            .lock()
            .expect("visitor mutex poisoned")
            .visit_code_block(&node_context, language, content);
    visitor_output(result, handler)
}

#[cfg(feature = "visitor")]
fn visitor_output(result: crate::visitor::VisitResult, handler: &HandlerContext<'_>) -> Option<String> {
    use crate::visitor::VisitResult;

    match result {
        VisitResult::Continue => None,
        VisitResult::Custom(custom) => {
            crate::converter::structure_capture::replace_element(handler.context, Some(&custom));
            Some(custom)
        }
        VisitResult::Skip => {
            crate::converter::structure_capture::replace_element(handler.context, None);
            Some(String::new())
        }
        VisitResult::PreserveHtml => {
            let html = serialize_node(handler.node_handle, handler.parser);
            crate::converter::structure_capture::replace_element(handler.context, Some(&html));
            Some(html)
        }
        VisitResult::Error(error) => {
            crate::converter::structure_capture::replace_element(handler.context, None);
            if handler.context.visitor_error.borrow().is_none() {
                *handler.context.visitor_error.borrow_mut() = Some(error);
            }
            None
        }
    }
}

/// Format inline code with appropriate backtick delimiters.
///
/// Handles:
/// - Single backticks for normal content
/// - Double backticks when content contains backticks
/// - Space padding when needed to avoid backtick adjacency
pub(in crate::converter) fn format_inline_code(content: &str, output: &mut String) {
    let contains_backtick = content.contains('`');

    let needs_delimiter_spaces = {
        let first_char = content.chars().next();
        let last_char = content.chars().last();
        let starts_with_space = first_char == Some(' ');
        let ends_with_space = last_char == Some(' ');
        let starts_with_backtick = first_char == Some('`');
        let ends_with_backtick = last_char == Some('`');
        // ~keep CommonMark strips one space from each end of a code span ONLY when the
        // ~keep content is not entirely spaces, so an all-spaces body needs no padding --
        // ~keep and padding it changes what the span contains (`` ` ` `` is one space,
        // ~keep `` `   ` `` is three). Spec example 138 is exactly this case.
        starts_with_backtick || ends_with_backtick || (starts_with_space && ends_with_space && contains_backtick)
    };

    let (num_backticks, needs_spaces) = if contains_backtick {
        (min_safe_code_span_delimiter_length(content), needs_delimiter_spaces)
    } else {
        (1, needs_delimiter_spaces)
    };

    for _ in 0..num_backticks {
        output.push('`');
    }
    if needs_spaces {
        output.push(' ');
    }
    output.push_str(content);
    if needs_spaces {
        output.push(' ');
    }
    for _ in 0..num_backticks {
        output.push('`');
    }
}

/// Format a code block with the specified style and language.
///
/// Supports:
/// - Indented style (4-space indentation)
/// - Fenced style with backticks (```language)
/// - Fenced style with tildes (~~~language)
fn format_code_block(
    content: &str,
    cell_break_offsets: &[usize],
    language: Option<&str>,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
) {
    if ctx.in_table_cell {
        // ~keep Neither code-block style can exist inside a GFM pipe cell: the fence (or the
        // ~keep 4-space indent) is line-structured and both styles bracket the block with blank
        // ~keep lines, so the row would be split across physical lines (issue #456). Emit the
        // ~keep content inline with the block syntax dropped — the same degradation Tier-1's
        // ~keep `close_pre` already performs, and consistent with headings and list items
        // ~keep shedding their markers in a cell. Line breaks fold to a space rather than to
        // ~keep `<br>` so that the two tiers stay byte-equal. The cell break separates the block
        // ~keep from the cell content before it (issue #645).
        let trimmed_content = content.trim_matches('\n');
        if !trimmed_content.is_empty() && !ctx.convert_as_inline && !ctx.in_code {
            crate::converter::main_helpers::separate_block_in_cell(output, options.br_in_tables);
        }
        format_preformatted_cell_content(
            content,
            cell_break_offsets,
            output,
            options.br_in_tables,
            options.whitespace_mode == crate::options::WhitespaceMode::Strict,
        );
        return;
    }

    if ctx.in_list_item {
        format_code_block_in_list_item(content, language, output, options, ctx);
        return;
    }

    separate_code_block(output, ctx);
    match options.code_block_style {
        crate::options::CodeBlockStyle::Indented => format_indented_code_block(content, output),
        crate::options::CodeBlockStyle::Backticks | crate::options::CodeBlockStyle::Tildes => {
            format_fenced_code_block(content, language, output, options);
        }
    }
}

fn separate_code_block(output: &mut String, context: &Context) {
    if context.convert_as_inline || output.is_empty() || output.ends_with("\n\n") {
        return;
    }
    if output.ends_with('\n') {
        output.push('\n');
    } else {
        output.push_str("\n\n");
    }
}

fn format_indented_code_block(content: &str, output: &mut String) {
    let indented = content
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

fn format_fenced_code_block(content: &str, language: Option<&str>, output: &mut String, options: &ConversionOptions) {
    let fence_char = if options.code_block_style == crate::options::CodeBlockStyle::Backticks {
        '`'
    } else {
        '~'
    };
    // ~keep A fence must exceed the longest matching run in the content (CommonMark 4.5).
    let fence_length = (longest_consecutive_run(content, fence_char) + 1).max(MIN_FENCE_LENGTH);
    let fence: String = std::iter::repeat_n(fence_char, fence_length).collect();
    output.push_str(&fence);
    if let Some(language) = language {
        output.push_str(language);
    } else if !options.code_language.is_empty() {
        output.push_str(&options.code_language);
    }
    output.push('\n');
    output.push_str(content.trim_end_matches('\n'));
    output.push('\n');
    output.push_str(&fence);
    output.push_str("\n\n");
}

/// Render preformatted table-cell content as code spans, keeping real `<br>` nodes outside. ~keep
pub(in crate::converter) fn format_preformatted_cell_content(
    content: &str,
    break_offsets: &[usize],
    output: &mut String,
    br_in_tables: bool,
    preserve_whitespace: bool,
) {
    if !br_in_tables || break_offsets.is_empty() {
        let normalized = normalize_preformatted_cell_segment(content, preserve_whitespace);
        let folded = crate::text::fold_cell_line_breaks_verbatim_cow(&normalized);
        if !folded.is_empty() {
            format_inline_code(folded.as_ref(), output);
        }
        return;
    }

    let (normalized, break_offsets) = normalize_preformatted_cell_content(content, break_offsets, preserve_whitespace);
    let mut start = 0;
    let mut wrote_segment = false;
    let mut pending_breaks = 0;
    for offset in break_offsets {
        if offset < start || !normalized.is_char_boundary(offset) {
            continue;
        }
        let segment = crate::text::fold_cell_line_breaks_verbatim_cow(&normalized[start..offset]);
        if !segment.is_empty() {
            if wrote_segment {
                for _ in 0..pending_breaks {
                    output.push_str("<br>");
                }
            }
            format_inline_code(segment.as_ref(), output);
            wrote_segment = true;
            pending_breaks = 0;
        }
        pending_breaks += 1;
        start = offset + 1;
    }
    let segment = crate::text::fold_cell_line_breaks_verbatim_cow(&normalized[start..]);
    if !segment.is_empty() {
        if wrote_segment {
            for _ in 0..pending_breaks {
                output.push_str("<br>");
            }
        }
        format_inline_code(segment.as_ref(), output);
    }
}

fn normalize_preformatted_cell_content(
    content: &str,
    break_offsets: &[usize],
    preserve_whitespace: bool,
) -> (String, Vec<usize>) {
    let core = content.trim_matches('\n');
    let core_start = content.len() - content.trim_start_matches('\n').len();
    let normalized = if preserve_whitespace {
        core.to_string()
    } else {
        dedent_code_block(core)
    };
    let newline_offsets: Vec<usize> = normalized.match_indices('\n').map(|(offset, _)| offset).collect();
    let normalized_break_offsets = break_offsets
        .iter()
        .filter_map(|&offset| offset.checked_sub(core_start))
        .filter(|&offset| offset < core.len() && core.as_bytes().get(offset) == Some(&b'\n'))
        .map(|offset| core[..offset].bytes().filter(|&byte| byte == b'\n').count())
        .filter_map(|ordinal| newline_offsets.get(ordinal).copied())
        .collect();
    (normalized, normalized_break_offsets)
}

fn normalize_preformatted_cell_segment(content: &str, preserve_whitespace: bool) -> String {
    let content = content.trim_matches('\n');
    if preserve_whitespace {
        content.to_string()
    } else {
        dedent_code_block(content)
    }
}

/// Format a code block that is a child of a list item.
///
/// ~keep A fenced (or indented) code block spans several physical lines, but the
/// ~keep only call site that indented list continuation content
/// ~keep (`block/paragraph.rs`'s list continuation) indented a single
/// ~keep leading position, not every line a block emits. CommonMark's list
/// ~keep container match is per physical line: a non-blank line that is not
/// ~keep indented to `list_indent_columns` is not part of the item, so an
/// ~keep unindented closing fence (or any interior content line) drops the rest
/// ~keep of the block, and the item itself, out of the list on re-parse
/// ~keep (CommonMark spec examples 263, 273, 274, 318, 324). Render into a
/// ~keep scratch buffer first so every line can be indented uniformly, then only
/// ~keep skip the indent on the very first line when this block sits directly
/// ~keep after the marker text (i.e. it is the item's first content, not a
/// ~keep continuation) — that line already starts at the right column.
fn format_code_block_in_list_item(
    content: &str,
    language: Option<&str>,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
) {
    let mut rendered = String::new();
    let plain_ctx = Context {
        in_list_item: false,
        ..ctx.clone()
    };
    format_code_block(content, &[], language, &mut rendered, options, &plain_ctx);

    // ~keep A plain suffix check like `output.ends_with("* ")` also matches the closing
    // ~keep "**"/"*" of `<strong>`/`<em>` immediately followed by a migrated trailing
    // ~keep space, indistinguishable from a real bare bullet by suffix alone -- and, being
    // ~keep hardcoded to `-`/`*`, never matched the third bullet `+` at all. Both false
    // ~keep positive (fake marker) and false negative (real `+ ` marker) misclassified
    // ~keep this fenced block relative to the marker, either gluing the opening fence onto
    // ~keep the previous inline line (breaking the fence syntax) or doubly indenting the
    // ~keep first line. See `list::utils::line_is_bare_list_marker`'s doc comment.
    let is_continuation = !ctx.convert_as_inline
        && !output.is_empty()
        && !crate::converter::list::utils::line_is_bare_list_marker(output);

    if is_continuation {
        crate::converter::trim_trailing_whitespace(output);
        if !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
    }

    let indent =
        crate::converter::list::utils::continuation_indent_string(ctx.list_indent_columns, options).unwrap_or_default();

    for (index, segment) in rendered.split_inclusive('\n').enumerate() {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        if line.is_empty() || (index == 0 && !is_continuation) {
            output.push_str(segment);
        } else {
            output.push_str(&indent);
            output.push_str(segment);
        }
    }
}
