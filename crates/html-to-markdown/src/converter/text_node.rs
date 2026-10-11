//! Text node processing for HTML to Markdown conversion.
//!
//! Handles raw text nodes with:
//! - HTML entity decoding
//! - Whitespace normalization and stripping
//! - Text escaping with configurable escape modes
//! - Visitor callbacks (when feature enabled)
//! - List item indentation

use std::borrow::Cow;

use crate::converter::block::container::HandlerContext;
use crate::converter::dom_context::DomContext;
use crate::converter::main_helpers::{hard_break_marker, is_inline_element};
use crate::converter::utility::content::{NextElement, ZERO_WIDTH_SPACE, line_end_before_element};
use crate::converter::utility::siblings::{
    FollowingContent, br_follows_enclosing_elements, content_starts_with_block, following_sibling_content,
    get_next_sibling_tag, get_previous_sibling_tag, next_sibling_is_inline_tag, zero_width_space_follows,
};
use crate::converter::utility::white_space::{
    TextClass, cell_text, classify_text, edges, is_blank, is_collapsible, space_is_owed, visible, with_line_feeds,
};
use crate::text;
#[cfg(feature = "visitor")]
use crate::visitor::EMPTY_ATTRS;

type Context = crate::converter::Context;

/// Process a raw text node during HTML to Markdown conversion.
///
/// Handles:
/// - HTML entity decoding
/// - Whitespace normalization and stripping
/// - Text escaping with configurable escape modes
/// - Visitor callbacks (when feature enabled)
/// - List item indentation
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn process_text_node(
    raw: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    TextProcessor {
        node_handle,
        parser,
        output,
        handler,
    }
    .process(raw);
}

/// Whether white space is written at the start of a fresh block, where it is no space.
/// `was_fresh` is the fresh block flag of the context before the node wrote anything.
const fn is_fresh_block_start(ctx: &Context, was_fresh: bool) -> bool {
    was_fresh && !ctx.convert_as_inline && !ctx.in_table_cell && !ctx.in_list_item
}

struct WhitespaceFacts {
    had_newlines: bool,
    has_double_newline: bool,
    was_fresh_block_start: bool,
}

struct ProcessedText {
    output: String,
    semantic: Option<String>,
}

impl ProcessedText {
    fn same(text: String, capture_semantic: bool) -> Self {
        Self {
            semantic: capture_semantic.then(|| text.clone()),
            output: text,
        }
    }
}

struct TextProcessor<'dom, 'output, 'handler> {
    node_handle: &'dom tl::NodeHandle,
    parser: &'dom tl::Parser<'dom>,
    output: &'output mut String,
    handler: HandlerContext<'handler>,
}

impl TextProcessor<'_, '_, '_> {
    fn process(&mut self, raw: &str) {
        let mut decoded = text::decode_html_entities_cow(raw);
        if !self.handler.ctx.in_code {
            decoded = with_line_feeds(decoded);
        }
        if decoded.is_empty() {
            return;
        }
        let facts = WhitespaceFacts {
            had_newlines: decoded.contains('\n'),
            has_double_newline: decoded.contains("\n\n") || decoded.contains("\r\n\r\n"),
            was_fresh_block_start: self.handler.ctx.at_fresh_block_start.get(),
        };
        if self.handler.options.strip_newlines && (decoded.contains('\r') || decoded.contains('\n')) {
            decoded = Cow::Owned(decoded.replace(['\r', '\n'], " "));
        }
        if is_blank(&decoded) {
            let output_start = self.output.len();
            self.emit_whitespace(decoded.as_ref(), &facts);
            if crate::converter::structure_capture::is_text_capture_active(self.handler.ctx) {
                self.append_semantic_output(output_start);
            }
            return;
        }
        let decoded = self.without_line_end_before_zero_width_space(decoded);
        let escape_asterisks = self.escape_asterisks();
        let capture_semantic = crate::converter::structure_capture::is_text_capture_active(self.handler.ctx);
        let processed = self.process_content(decoded, escape_asterisks, facts.was_fresh_block_start, capture_semantic);
        #[cfg(feature = "visitor")]
        let Some(final_text) = self.apply_visitor(processed) else {
            return;
        };
        #[cfg(not(feature = "visitor"))]
        let final_text = processed;
        self.emit_processed(&final_text.output);
        if let Some(semantic) = final_text.semantic.as_deref() {
            crate::converter::structure_capture::append_text(self.handler.ctx, semantic);
        }
    }

    /// Removes the white space with a line end that `text` ends with when a zero-width space
    /// follows it.
    ///
    /// ~keep A zero-width space is a place where a line can break, not a space: a browser
    /// ~keep drops the line end beside it, so `long\n<span></span>&#8203;word` is one word.
    fn without_line_end_before_zero_width_space<'text>(&self, text: Cow<'text, str>) -> Cow<'text, str> {
        if self.handler.ctx.in_code || self.handler.options.whitespace_mode == crate::options::WhitespaceMode::Strict {
            return text;
        }
        let TextClass::Text {
            core,
            suffix: Some(suffix),
            ..
        } = classify_text(text.as_ref())
        else {
            return text;
        };
        // ~keep The segment break goes when a zero-width space is on either side of it: the
        // ~keep last character of this text, or the first character after it.
        let after =
            zero_width_space_follows(self.node_handle, self.parser, self.handler.dom_ctx).then_some(ZERO_WIDTH_SPACE);
        if suffix.writes_space(core.chars().last(), after) {
            return text;
        }
        Cow::Owned(text.trim_end_matches(is_collapsible).to_string())
    }

    fn escape_asterisks(&self) -> bool {
        self.handler.options.escape_asterisks
            || self
                .handler
                .ctx
                .djot_rule_like_text
                .as_ref()
                .is_some_and(crate::converter::context::DjotRuleLikeText::current)
    }

    fn emit_whitespace(&mut self, value: &str, facts: &WhitespaceFacts) {
        let ctx = self.handler.ctx;
        if ctx.in_code {
            self.output.push_str(value);
            return;
        }
        if self.handler.options.whitespace_mode == crate::options::WhitespaceMode::Strict {
            self.emit_strict_whitespace(value, facts.has_double_newline);
            return;
        }
        if self.between_adjacent_images() {
            self.output.push(if facts.had_newlines { '\n' } else { ' ' });
            return;
        }
        if self.at_paragraph_buffer_start() || self.at_fresh_block_start(facts.was_fresh_block_start) {
            return;
        }
        if facts.had_newlines {
            self.emit_newline_whitespace(value, facts.was_fresh_block_start);
        } else if !matches!(classify_text(value), TextClass::WhiteSpaceOnly(_)) {
            // ~keep A blank text that holds a no-break space is written, whatever white space
            // ~keep of the source is before it: a browser shows `one <span>&nbsp;</span>two`
            // ~keep with both spaces. Only its own collapsible white space joins a space
            // ~keep already written (rule 4). `<li>&nbsp;</li>` is a block start and drops it.
            let kept = if space_is_owed(self.output) {
                value
            } else {
                value.trim_start_matches(is_collapsible)
            };
            self.output.push_str(kept);
        } else if !self.at_block_line_start() && !self.output.ends_with(' ') && !self.after_hard_break() {
            // ~keep A space-only text between two blocks of a quote (`<p>8</p> <p>0</p>`) is a
            // ~keep line of one space, as before this change: only the block's own buffer
            // ~keep refuses a space at a line start. A line that a `<br>` ended starts the next
            // ~keep one in a list item's own buffer too: the text after it writes the indent.
            self.output.push(' ');
        }
    }

    fn after_hard_break(&self) -> bool {
        self.output.ends_with(hard_break_marker(self.handler.options))
    }

    fn at_block_line_start(&self) -> bool {
        std::ptr::from_ref::<String>(self.output) as usize == self.handler.ctx.block_output_ptr
            && (self.output.is_empty() || self.output.ends_with('\n'))
    }

    fn between_adjacent_images(&self) -> bool {
        get_previous_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx) == Some("img")
            && get_next_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx) == Some("img")
    }

    fn emit_strict_whitespace(&mut self, value: &str, has_double_newline: bool) {
        let ctx = self.handler.ctx;
        if ctx.convert_as_inline || ctx.in_table_cell || ctx.in_list_item {
            self.output.push_str(value);
        } else if has_double_newline {
            if !self.output.ends_with("\n\n") {
                self.output.push('\n');
            }
        } else {
            self.output.push_str(value);
        }
    }

    fn at_paragraph_buffer_start(&self) -> bool {
        self.handler.ctx.in_paragraph
            && std::ptr::from_ref::<String>(self.output) as usize == self.handler.ctx.block_output_ptr
            && self.output.len() == self.handler.ctx.block_content_start
    }

    const fn at_fresh_block_start(&self, was_fresh: bool) -> bool {
        is_fresh_block_start(self.handler.ctx, was_fresh)
    }

    /// Writes the one space of a blank text that holds a line end, where the rule owes it:
    /// before an inline sibling, or at the end of an inline element that a word follows. A
    /// no-break space in such a text is written as it is when it is the only one.
    fn emit_newline_whitespace(&mut self, value: &str, was_fresh: bool) {
        if self.output.is_empty() {
            if !was_fresh && self.handler.ctx.inline_depth > 0 {
                self.output.push(' ');
            }
            return;
        }
        if self.output.ends_with("\n\n") {
            return;
        }
        let next_tag = get_next_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx);
        if next_tag.is_some_and(|next_tag| !is_inline_element(next_tag)) {
            return;
        }
        let mut kept = value.chars().filter(|character| !is_collapsible(*character));
        let first_kept = kept.next();
        let holds_more = kept.next().is_some();
        if let (Some(character), false) = (first_kept, holds_more) {
            self.output.push(character);
            return;
        }
        let needs_space = next_tag.is_some()
            || holds_more
            || newline_span_needs_separating_space(self.node_handle, self.parser, self.handler.dom_ctx);
        if needs_space && space_is_owed(self.output) {
            self.output.push(' ');
        }
    }

    fn process_content(
        &self,
        value: Cow<'_, str>,
        escape_asterisks: bool,
        was_fresh: bool,
        capture_semantic: bool,
    ) -> ProcessedText {
        let ctx = self.handler.ctx;
        if ((ctx.in_code && !ctx.in_code_block) || ctx.in_ruby) && ctx.in_table_cell {
            return ProcessedText::same(
                text::fold_cell_line_breaks_verbatim_cow(value.as_ref()).into_owned(),
                capture_semantic,
            );
        }
        if ctx.in_code && !ctx.in_code_block {
            return ProcessedText::same(
                text::fold_cell_line_breaks_verbatim_cow(value.as_ref()).into_owned(),
                capture_semantic,
            );
        }
        if ctx.in_code || ctx.in_ruby {
            return ProcessedText::same(value.into_owned(), capture_semantic);
        }
        if ctx.in_table_cell {
            return self.process_table_cell(value.as_ref(), escape_asterisks, capture_semantic);
        }
        if self.handler.options.whitespace_mode == crate::options::WhitespaceMode::Strict {
            return self.process_strict(value.as_ref(), escape_asterisks, capture_semantic);
        }
        self.process_normalized(value.as_ref(), escape_asterisks, was_fresh, capture_semantic)
    }

    fn process_table_cell(&self, value: &str, escape_asterisks: bool, capture_semantic: bool) -> ProcessedText {
        let options = self.handler.options;
        let normalized = if options.whitespace_mode == crate::options::WhitespaceMode::Normalized {
            let collapsed = cell_text(value);
            // ~keep White space after a space is the same run of white space, as in `skip_prefix`.
            match collapsed.strip_prefix(' ') {
                Some(rest) if !space_is_owed(self.output) => Cow::Owned(rest.to_string()),
                _ => collapsed,
            }
        } else {
            text::fold_cell_line_breaks_verbatim_cow(value)
        };
        let mut output = String::with_capacity(normalized.len());
        text::escape_into(
            &mut output,
            normalized.as_ref(),
            options.escape_misc,
            escape_asterisks,
            options.escape_underscores,
            options.escape_ascii,
        );
        if !options.escape_misc && !options.escape_ascii && output.contains('|') {
            output = output.replace('|', r"\|");
        }
        let output = crate::converter::utility::escaping::escape_djot_table_cell_literal(
            &output,
            options.output_format,
            self.handler.ctx.in_table_cell,
        )
        .into_owned();
        ProcessedText {
            output,
            semantic: capture_semantic.then(|| normalized.into_owned()),
        }
    }

    fn process_strict(&self, value: &str, escape_asterisks: bool, capture_semantic: bool) -> ProcessedText {
        let follows_break = get_next_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx) == Some("br")
            || br_follows_enclosing_elements(self.node_handle.get_inner(), self.parser, self.handler.dom_ctx);
        let trimmed_end = follows_break
            .then(|| strip_single_trailing_line_ending(value))
            .flatten()
            .unwrap_or(value);
        let preceded_by_break = get_previous_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx)
            == Some("br")
            || (self.handler.ctx.inline_buffer_after_hard_break && self.output.trim_matches([' ', '\t']).is_empty());
        let strict = preceded_by_break
            .then(|| strip_single_leading_line_ending(trimmed_end))
            .flatten()
            .unwrap_or(trimmed_end);
        let options = self.handler.options;
        let output = text::escape(
            strict,
            options.escape_misc,
            escape_asterisks,
            options.escape_underscores,
            options.escape_ascii,
        )
        .into_owned();
        ProcessedText {
            output,
            semantic: capture_semantic.then(|| strict.to_string()),
        }
    }

    fn process_normalized(
        &self,
        value: &str,
        escape_asterisks: bool,
        was_fresh: bool,
        capture_semantic: bool,
    ) -> ProcessedText {
        let has_double_newline = value.contains("\n\n") || value.contains("\r\n\r\n");
        let trailing_single_newline = value.ends_with('\n') && !value.ends_with("\n\n") && !value.ends_with("\r\n\r\n");
        let (prefix, suffix) = edges(value);
        let core = visible(value);
        let mut output = String::with_capacity(prefix.len() + core.len() + suffix.len() + 2);
        let mut semantic = capture_semantic.then(|| String::with_capacity(output.capacity()));
        if !self.skip_prefix(prefix, was_fresh) && !prefix.is_empty() {
            output.push_str(prefix);
            if let Some(semantic) = semantic.as_mut() {
                semantic.push_str(prefix);
            }
        }
        let options = self.handler.options;
        output.push_str(&text::escape(
            core.as_ref(),
            options.escape_misc,
            escape_asterisks,
            options.escape_underscores,
            options.escape_ascii,
        ));
        if let Some(semantic) = semantic.as_mut() {
            semantic.push_str(core.as_ref());
        }
        if !suffix.is_empty() {
            output.push_str(suffix);
            if let Some(semantic) = semantic.as_mut() {
                semantic.push_str(suffix);
            }
        } else if trailing_single_newline {
            let output_end = output.len();
            self.append_trailing_line_ending(&mut output, has_double_newline);
            if let Some(semantic) = semantic.as_mut() {
                semantic.push_str(&output[output_end..]);
            }
        }
        ProcessedText { output, semantic }
    }

    fn skip_prefix(&self, prefix: &str, was_fresh: bool) -> bool {
        is_fresh_block_start(self.handler.ctx, was_fresh)
            || self.output.ends_with("\n\n")
            || ["* ", "- ", ". ", "] "]
                .iter()
                .any(|ending| self.output.ends_with(ending))
            // ~keep White space after a space is the same run of white space, whatever lies
            // ~keep between the two: an element that wrote nothing is no word.
            || (!space_is_owed(self.output) && prefix == " ")
    }

    fn append_trailing_line_ending(&self, output: &mut String, has_double_newline: bool) {
        let safe_start = crate::converter::utility::content::floor_char_boundary(
            self.output,
            self.handler.ctx.block_content_start.min(self.output.len()),
        );
        if self.output[safe_start..].ends_with("\n\n") {
            return;
        }
        if has_double_newline {
            output.push('\n');
            return;
        }
        if let Some(next_tag) = get_next_sibling_tag(self.node_handle, self.parser, self.handler.dom_ctx) {
            self.append_before_next_tag(output, next_tag);
        } else if self.handler.ctx.inline_depth > 0
            || self.handler.ctx.convert_as_inline
            || self.handler.ctx.in_paragraph
        {
            output.push(' ');
        } else if !br_follows_enclosing_elements(self.node_handle.get_inner(), self.parser, self.handler.dom_ctx) {
            output.push('\n');
        }
    }

    fn append_before_next_tag(&self, output: &mut String, next_tag: &str) {
        // ~keep Only a line break ends the line: before any other element the line end of the
        // ~keep source is white space between two words, also when the element is empty (#778).
        if next_tag == "br" {
            return;
        }
        let ctx = self.handler.ctx;
        let in_running_text = ctx.inline_depth > 0 || ctx.convert_as_inline || ctx.in_paragraph;
        let dom_ctx = self.handler.dom_ctx;
        let starts_with_block = dom_ctx
            .next_tag_id(self.node_handle.get_inner(), self.parser)
            .is_some_and(|next| content_starts_with_block(next, self.parser, dom_ctx));
        let next = NextElement::new(is_inline_element(next_tag), starts_with_block);
        output.push(line_end_before_element(in_running_text, next));
    }

    #[cfg(feature = "visitor")]
    fn apply_visitor(&self, processed: ProcessedText) -> Option<ProcessedText> {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        let Some(visitor_handle) = self.handler.ctx.visitor.as_ref() else {
            return Some(processed);
        };
        let node_id = self.node_handle.get_inner();
        let parent_tag = self.handler.dom_ctx.parent_tag_name(node_id, self.parser);
        let node_ctx = NodeContext::with_borrowed_attributes(
            NodeType::Text,
            Cow::Borrowed(""),
            &EMPTY_ATTRS,
            self.handler.depth,
            self.handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0),
            parent_tag.map(Cow::Borrowed),
            true,
        );
        let result = visitor_handle
            .lock()
            .expect("visitor mutex poisoned")
            .visit_text(&node_ctx, &processed.output);
        match result {
            VisitResult::Continue | VisitResult::PreserveHtml => Some(processed),
            VisitResult::Custom(custom) if self.handler.ctx.inline_depth == 0 && !self.handler.ctx.in_heading => {
                Some(ProcessedText {
                    semantic: processed.semantic.as_ref().map(|_| custom.clone()),
                    output: custom,
                })
            }
            VisitResult::Custom(_) => Some(processed),
            VisitResult::Skip => None,
            VisitResult::Error(error) => {
                if self.handler.ctx.visitor_error.borrow().is_none() {
                    *self.handler.ctx.visitor_error.borrow_mut() = Some(error);
                }
                None
            }
        }
    }

    fn append_semantic_output(&self, output_start: usize) {
        if let Some(text) = self.output.get(output_start..) {
            crate::converter::structure_capture::append_text(self.handler.ctx, text);
        }
    }

    fn emit_processed(&mut self, final_text: &str) {
        push_running_text(
            self.output,
            final_text,
            TextSite {
                node_handle: self.node_handle,
                parser: self.parser,
                options: self.handler.options,
                ctx: self.handler.ctx,
                dom_ctx: self.handler.dom_ctx,
            },
        );
    }
}

/// Where a piece of running text is written: the node it comes from and the state of the walk.
#[derive(Clone, Copy)]
pub struct TextSite<'a> {
    pub node_handle: &'a tl::NodeHandle,
    pub parser: &'a tl::Parser<'a>,
    pub options: &'a crate::options::ConversionOptions,
    pub ctx: &'a crate::converter::Context,
    pub dom_ctx: &'a DomContext,
}

/// Writes escaped running text to `output` as a text node does: the list item indent before it,
/// and the escape of a block marker where the text starts a line.
pub fn push_running_text(output: &mut String, final_text: &str, site: TextSite<'_>) {
    let ctx = site.ctx;
    let options = site.options;
    crate::converter::list::utils::indent_list_item_line_start(output, ctx, options);
    let text_start = output.len();
    push_processed_text(output, final_text, site);
    let writes_to_block = writes_to_block(std::ptr::from_ref::<String>(output) as usize, ctx);
    if !ctx.in_code && options.output_format == crate::options::OutputFormat::Markdown {
        if writes_to_block {
            crate::converter::utility::escaping::escape_block_start(
                output,
                text_start,
                ctx.in_list_item,
                next_sibling_is_inline_tag(site.node_handle, site.parser, site.dom_ctx),
            );
        }
        crate::converter::utility::escaping::escape_continuation_line_start(
            output,
            text_start,
            ctx.inline_buffer_after_hard_break,
        );
    } else if !ctx.in_code && options.output_format == crate::options::OutputFormat::Djot {
        if writes_to_block {
            crate::converter::utility::escaping::escape_djot_list_item_start(output, text_start, ctx.in_list_item);
        }
        crate::converter::utility::escaping::escape_djot_continuation_line_start(
            output,
            text_start,
            ctx.inline_buffer_after_hard_break,
        );
    }
}

fn push_processed_text(output: &mut String, final_text: &str, site: TextSite<'_>) {
    let ctx = site.ctx;
    // ~keep Code is verbatim. The handler of the code block adds the indent of the list item to
    // ~keep every line of the finished block, so a blank line in the code starts no paragraph here.
    if !ctx.in_list_item || ctx.in_code {
        output.push_str(final_text);
        return;
    }
    if final_text.contains('\n') && !final_text.contains("\n\n") {
        let mut lines = final_text.split_inclusive('\n').peekable();
        while let Some(line) = lines.next() {
            output.push_str(line);
            if lines.peek().is_some() {
                crate::converter::list::utils::indent_list_item_line_start(output, ctx, site.options);
            }
        }
        return;
    }
    if !final_text.contains("\n\n") {
        output.push_str(final_text);
        return;
    }
    let indent = " ".repeat(4 * ctx.list_depth);
    for (index, part) in final_text.split("\n\n").enumerate() {
        if index > 0 {
            output.push_str("\n\n");
            output.push_str(&indent);
        }
        output.push_str(part.trim());
    }
}

fn writes_to_block(output_ptr: usize, ctx: &crate::converter::Context) -> bool {
    let writes_to_task_marker = ctx.task_item_scope == Some((ctx.list_depth, ctx.blockquote_depth))
        && ctx
            .first_writer
            .as_ref()
            .is_some_and(crate::converter::list::item::FirstWriter::is_open);
    !ctx.convert_as_inline
        && !ctx.in_heading
        && !ctx.in_table_cell
        && !ctx.in_marker_text()
        && !writes_to_task_marker
        && (ctx.block_output_ptr == 0 || output_ptr == ctx.block_output_ptr)
}

/// Remove one source line ending and its following indentation only when it is not a blank line. ~keep
fn strip_single_trailing_line_ending(text: &str) -> Option<&str> {
    let content_end = text.trim_end_matches([' ', '\t']).len();
    let without_lf = text[..content_end].strip_suffix('\n')?;
    let without_line_ending = without_lf.strip_suffix('\r').unwrap_or(without_lf);
    (!without_line_ending.ends_with('\n') && !without_line_ending.ends_with('\r')).then_some(without_line_ending)
}

/// Remove one leading source line ending only when it does not begin a blank line. ~keep
fn strip_single_leading_line_ending(text: &str) -> Option<&str> {
    let without_line_ending = text.strip_prefix("\r\n").or_else(|| text.strip_prefix('\n'))?;
    (!without_line_ending.starts_with('\n') && !without_line_ending.starts_with('\r')).then_some(without_line_ending)
}

/// Whether a whitespace-only newline text node with no in-parent next sibling
/// still separates inline content (issue #430).
///
/// Returns true when the node's parent is an inline-like element that is itself
/// followed by inline content — e.g. the lone `"\n"` inside the middle `<span>`
/// of `<span>a</span><span>\n</span><span>b</span>` — or when the nearest such
/// ancestor with a following sibling is (issue #505).
///
/// "Inline content" deliberately includes a bare text sibling, not just an element: a browser
/// renders `a<span>\n</span>b` and `a<span>\n</span><span>b</span>` identically, so asking only
/// about a following *tag* dropped the separator and welded the words together (issue #491). ~keep
fn newline_span_needs_separating_space(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    dom_ctx: &DomContext,
) -> bool {
    // ~keep Climb through every transparent inline ancestor that has nothing after it: in
    // ~keep `<span><span>\n</span></span>Beta` only the outer wrapper is followed by `Beta`,
    // ~keep and asking the inner one alone welded the words together (issue #505). The first
    // ~keep block ancestor ends the climb -- a newline at the end of a block separates nothing.
    let mut node_id = node_handle.get_inner();
    loop {
        let Some(parent_id) = dom_ctx.parent_of(node_id) else {
            return false;
        };
        let parent_is_inline = dom_ctx
            .tag_info(parent_id, parser)
            .is_some_and(|info| info.is_inline_like);
        if !parent_is_inline {
            return false;
        }
        match following_sibling_content(parent_id, parser, dom_ctx) {
            FollowingContent::Inline => return true,
            FollowingContent::NotInline => return false,
            FollowingContent::Absent => node_id = parent_id,
        }
    }
}
