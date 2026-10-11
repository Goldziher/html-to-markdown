//! Main conversion pipeline for HTML to Markdown.
//!
//! This module implements the core conversion functions and the recursive tree walker
//! that transforms HTML DOM nodes into Markdown output.

#![allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::items_after_statements
)]

use std::borrow::Cow;
use std::collections::HashSet;

use crate::converter::dom_context::DomContext;
use crate::converter::main_helpers::{
    collapse_excess_blank_lines, effective_max_depth, extract_head_metadata, format_metadata_frontmatter,
    has_custom_element_tags, is_inline_element, repair_with_html5ever, strip_trailing_backslash_breaks,
    trim_line_end_whitespace, trim_trailing_whitespace,
};
use crate::converter::plain_text::extract_plain_text;
use crate::converter::preprocessing_helpers::{is_page_header, should_drop_for_preprocessing};
use crate::converter::utility::content::{is_block_level_element, normalized_tag_name};
use crate::converter::utility::preprocessing::{
    PRESERVED_MENU_ATTRIBUTE, normalize_bogus_comment_endings, normalize_menu_elements, normalize_split_closing_tags,
    normalize_unclosed_list_items, preprocess_html, restore_preserved_menu_elements, strip_bogus_comments,
    strip_hidden_elements, strip_script_and_style_tags,
};
use crate::converter::utility::serialization::serialize_tag_to_html;
use crate::options::{NewlineStyle, OutputFormat};

use crate::converter::block::container::HandlerContext;
use crate::error::Result;
use crate::options::ConversionOptions;

use crate::converter::context::{Context, ContextParameters, InlineCollectorHandle};
use crate::types::structure_collector::StructureCollectorHandle;

mod parse;
use self::parse::{ParseOutcome, parse_for_conversion};

type ConversionOutput = (
    String,
    Option<crate::types::DocumentStructure>,
    Vec<crate::types::TableData>,
    Option<crate::types::ProcessingWarning>,
);

pub struct ConversionParameters<'a> {
    pub inline_collector: Option<InlineCollectorHandle>,
    #[cfg(feature = "metadata")]
    pub metadata_collector: Option<crate::metadata::MetadataCollectorHandle>,
    #[cfg(feature = "visitor")]
    pub visitor: Option<crate::visitor::VisitorHandle>,
    pub structure_collector: Option<StructureCollectorHandle>,
    pub base_url: Option<std::rc::Rc<url::Url>>,
    pub document_base_href: Option<&'a str>,
}
/// Internal implementation of HTML to Markdown conversion.
///
/// Returns the converted content, optional document structure, extracted tables, and an
/// optional depth-limit warning.
pub fn convert_html_impl(
    html: &str,
    options: &ConversionOptions,
    parameters: ConversionParameters<'_>,
) -> Result<ConversionOutput> {
    let ConversionParameters {
        inline_collector,
        #[cfg(feature = "metadata")]
        metadata_collector,
        #[cfg(feature = "visitor")]
        visitor,
        structure_collector,
        base_url,
        document_base_href,
    } = parameters;
    let preserve_menu = options.preserve_tags.iter().any(|tag| tag.eq_ignore_ascii_case("menu"));
    let mut preprocessed = prepare_html(html, preserve_menu);
    let mut attempted_misnest_repair = false;
    let (dom, dom_ctx) = loop {
        let repaired = match parse_for_conversion(&preprocessed, preserve_menu, &mut attempted_misnest_repair)? {
            ParseOutcome::Ready { dom, dom_ctx } => break (dom, dom_ctx),
            ParseOutcome::Retry(repaired) => repaired,
        };
        preprocessed = repaired;
    };
    let preprocessed_len = preprocessed.len();
    trace_parse_complete(&dom, preprocessed_len);
    let parser = dom.parser();
    let mut output = String::with_capacity(preprocessed_len.saturating_add(preprocessed_len / 4));
    let is_plain_text = options.output_format == OutputFormat::Plain;
    let frontmatter = prepare_frontmatter(
        &dom,
        parser,
        options,
        document_base_href,
        #[cfg(feature = "metadata")]
        metadata_collector.as_ref(),
    );
    output.push_str(&frontmatter);

    let reference_collector = create_reference_collector(options);

    let mut ctx = Context::new(
        options,
        ContextParameters {
            inline_collector,
            #[cfg(feature = "metadata")]
            metadata_collector,
            #[cfg(feature = "visitor")]
            visitor,
            structure_collector: structure_collector.as_ref().map(std::rc::Rc::clone),
            reference_collector: reference_collector.as_ref().map(std::rc::Rc::clone),
            base_url,
        },
    );

    apply_exclusions(&dom, options, &mut ctx);
    walk_document(&dom, parser, &mut output, options, &ctx, &dom_ctx);

    tracing::debug!(
        target: "html_to_markdown::convert",
        depth_limit_reached = ctx.depth_limit_reached.get(),
        "dom walk stage complete"
    );

    check_visitor_error(&ctx)?;

    let depth_warning = depth_warning(&ctx, options);

    // ~keep Drop ctx before unwrapping the structure collector Rc — ctx holds a cloned Rc
    // ~keep reference to the same collector, and Rc::try_unwrap requires exactly one reference.
    drop(ctx);

    append_references(&mut output, reference_collector);
    let output = finalize_output(&dom, parser, options, output, &frontmatter, is_plain_text);
    let (document, tables) = finish_structure_collector(structure_collector);
    trace_render_complete(&output, &tables);
    Ok((output, document, tables, depth_warning))
}

fn create_reference_collector(
    options: &ConversionOptions,
) -> Option<crate::converter::reference_collector::ReferenceCollectorHandle> {
    (options.link_style == crate::options::LinkStyle::Reference).then(|| {
        std::rc::Rc::new(std::cell::RefCell::new(
            crate::converter::reference_collector::ReferenceCollector::new(),
        ))
    })
}

fn trace_parse_complete(dom: &tl::VDom<'_>, input_len: usize) {
    tracing::debug!(
        target: "html_to_markdown::convert",
        node_count = dom.nodes().len(),
        input_len,
        "html parse stage complete"
    );
}

fn trace_render_complete(output: &str, tables: &[crate::types::TableData]) {
    tracing::debug!(
        target: "html_to_markdown::convert",
        output_len = output.len(),
        table_count = tables.len(),
        "render stage complete"
    );
}

fn apply_exclusions(dom: &tl::VDom<'_>, options: &ConversionOptions, ctx: &mut Context) {
    if options.exclude_selectors.is_empty() {
        return;
    }
    let mut excluded: HashSet<u32> = HashSet::new();
    for selector in &options.exclude_selectors {
        if let Some(iter) = dom.query_selector(selector) {
            excluded.extend(iter.map(|handle| handle.get_inner()));
        }
    }
    ctx.set_excluded_node_ids(excluded);
}

fn walk_document(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    dom_ctx: &DomContext,
) {
    let top_level_start = output.len();
    let handler = HandlerContext::new(options, ctx, 0, dom_ctx);
    for child_handle in dom.children() {
        walk_node(child_handle, parser, output, handler);
    }
    // ~keep The document end is the one block boundary no subsequent dispatch can observe.
    if options.newline_style == NewlineStyle::Backslash {
        strip_trailing_backslash_breaks(output, top_level_start);
    }
}

fn depth_warning(ctx: &Context, options: &ConversionOptions) -> Option<crate::types::ProcessingWarning> {
    let max_depth = effective_max_depth(options);
    ctx.depth_limit_reached.get().then(|| {
        tracing::warn!(
            target: "html_to_markdown::convert",
            max_depth,
            "DOM traversal reached the effective depth limit; deeper nodes were skipped"
        );
        crate::types::ProcessingWarning {
            kind: crate::types::WarningKind::DepthLimitExceeded,
            message: format!(
                "DOM traversal reached the effective depth limit of {max_depth}; deeper nodes were skipped."
            ),
        }
    })
}

fn append_references(
    output: &mut String,
    reference_collector: Option<
        std::rc::Rc<std::cell::RefCell<crate::converter::reference_collector::ReferenceCollector>>,
    >,
) {
    let Some(rc) = reference_collector else { return };
    let Ok(collector) = std::rc::Rc::try_unwrap(rc) else {
        return;
    };
    let ref_section = collector.into_inner().finish();
    if ref_section.is_empty() {
        return;
    }
    output.truncate(output.trim_end_matches('\n').len());
    output.push_str("\n\n");
    output.push_str(&ref_section);
}

fn finalize_output(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
    options: &ConversionOptions,
    mut output: String,
    frontmatter: &str,
    is_plain_text: bool,
) -> String {
    if is_plain_text {
        output = extract_plain_text(dom, parser, options);
    } else {
        trim_line_end_whitespace(&mut output);
        collapse_excess_blank_lines(&mut output);
    }
    if options.wrap {
        wrap_after_frontmatter(&output, frontmatter, options)
    } else {
        output
    }
}

fn prepare_frontmatter(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
    options: &ConversionOptions,
    document_base_href: Option<&str>,
    #[cfg(feature = "metadata")] metadata_collector: Option<&crate::metadata::MetadataCollectorHandle>,
) -> String {
    let wants_frontmatter = options.extract_metadata && !options.convert_as_inline;
    #[cfg(feature = "metadata")]
    let wants_document = metadata_collector.is_some_and(|collector| collector.borrow().wants_document());
    #[cfg(not(feature = "metadata"))]
    let wants_document = false;
    if !wants_frontmatter && !wants_document {
        return String::new();
    }
    let head_metadata = extract_head_metadata(dom.children(), parser, options, document_base_href);
    let frontmatter = if wants_frontmatter && !head_metadata.is_empty() {
        format_metadata_frontmatter(&head_metadata)
    } else {
        String::new()
    };
    #[cfg(feature = "metadata")]
    populate_metadata_collector(dom, parser, metadata_collector, head_metadata, wants_document);
    frontmatter
}

#[cfg(feature = "visitor")]
fn check_visitor_error(ctx: &Context) -> Result<()> {
    let Some(err) = ctx.visitor_error.borrow().as_ref().cloned() else {
        return Ok(());
    };
    tracing::error!(
        target: "html_to_markdown::convert",
        error = %err,
        "visitor callback returned an error; aborting conversion"
    );
    Err(crate::error::ConversionError::Visitor(err))
}

#[cfg(not(feature = "visitor"))]
const fn check_visitor_error(_ctx: &Context) -> Result<()> {
    Ok(())
}

#[cfg(feature = "metadata")]
fn populate_metadata_collector(
    dom: &tl::VDom<'_>,
    parser: &tl::Parser<'_>,
    metadata_collector: Option<&crate::metadata::MetadataCollectorHandle>,
    head_metadata: std::collections::BTreeMap<String, String>,
    wants_document: bool,
) {
    if !wants_document {
        return;
    }
    let Some(collector) = metadata_collector else {
        return;
    };
    let (language, direction) = document_language_and_direction(dom.children(), parser);
    let mut collector = collector.borrow_mut();
    if !head_metadata.is_empty() {
        collector.set_head_metadata(head_metadata);
    }
    if let Some(language) = language {
        collector.set_language(language);
    }
    if let Some(direction) = direction {
        collector.set_text_direction(direction);
    }
}

#[cfg(feature = "metadata")]
fn document_language_and_direction(
    children: &[tl::NodeHandle],
    parser: &tl::Parser<'_>,
) -> (Option<String>, Option<String>) {
    let mut language = None;
    let mut direction = None;
    for child_handle in children {
        let Some(tl::Node::Tag(tag)) = child_handle.get(parser) else {
            continue;
        };
        if !matches!(tag.name().as_utf8_str().as_ref(), "html" | "body") {
            continue;
        }
        language = language.or_else(|| {
            tag.attributes()
                .get("lang")
                .flatten()
                .map(|value| value.as_utf8_str().to_string())
        });
        direction = direction.or_else(|| {
            tag.attributes()
                .get("dir")
                .flatten()
                .map(|value| value.as_utf8_str().to_string())
        });
    }
    (language, direction)
}

fn prepare_html(html: &str, preserve_menu: bool) -> String {
    let mut preprocessed = preprocess_initial_html(html, preserve_menu);
    if has_custom_element_tags(&preprocessed) {
        preprocessed = repair_custom_elements(preprocessed, preserve_menu);
    }
    preprocessed
}

fn preprocess_initial_html(html: &str, preserve_menu: bool) -> String {
    let stripped = strip_script_and_style_tags(html);
    // ~keep Bogus comments must be removed before tag-shaped preprocessing examines them.
    let stripped = strip_bogus_comments(&stripped);
    let stripped = strip_hidden_elements(&stripped);
    let stripped = normalize_bogus_comment_endings(&stripped);
    let stripped = normalize_split_closing_tags(&stripped);
    // ~keep Implicit list-item closes prevent deeply nested parser output on large changelogs.
    let stripped = normalize_unclosed_list_items(&stripped);
    let stripped = normalize_menu_elements(&stripped, preserve_menu);
    preprocess_html(&stripped).into_owned()
}

fn preprocess_repaired_html(html: &str, preserve_menu: bool) -> String {
    let stripped = strip_script_and_style_tags(html);
    let stripped = strip_hidden_elements(&stripped);
    let stripped = normalize_bogus_comment_endings(&stripped);
    let stripped = normalize_split_closing_tags(&stripped);
    let stripped = normalize_unclosed_list_items(&stripped);
    let stripped = normalize_menu_elements(&stripped, preserve_menu);
    preprocess_html(&stripped).into_owned()
}

fn repair_custom_elements(preprocessed: String, preserve_menu: bool) -> String {
    let Some(repaired) = repair_with_html5ever(&preprocessed) else {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "custom element tags detected; html5ever repair failed, proceeding with unrepaired markup"
        );
        return preprocessed;
    };
    tracing::warn!(
        target: "html_to_markdown::convert",
        "custom element tags detected; re-parsed input with html5ever repair fallback"
    );
    preprocess_repaired_html(&repaired, preserve_menu)
}

/// Wrap `output` at the wrap width, leaving the `frontmatter` it starts with as it is.
///
/// ~keep The frontmatter is YAML, not Markdown: wrapped, its closing `---` reads as a heading
/// ~keep underline and its keys join into one line that YAML cannot parse. The plain-text output
/// ~keep carries no frontmatter, so all of it is wrapped.
fn wrap_after_frontmatter(output: &str, frontmatter: &str, options: &ConversionOptions) -> String {
    let body_start = if output.starts_with(frontmatter) {
        frontmatter.len()
    } else {
        0
    };
    let mut wrapped = output[..body_start].to_owned();
    wrapped.push_str(&crate::wrapper::wrap_markdown(&output[body_start..], options));
    wrapped
}

/// Consume the structure collector and return the [`DocumentStructure`] and extracted
/// [`TableData`] entries.  Returns `(None, vec![])` when no collector was provided.
fn finish_structure_collector(
    sc: Option<StructureCollectorHandle>,
) -> (Option<crate::types::DocumentStructure>, Vec<crate::types::TableData>) {
    match sc.and_then(|rc| std::rc::Rc::try_unwrap(rc).ok()) {
        Some(cell) => {
            let (doc, tables) = cell.into_inner().finish();
            (Some(doc), tables)
        }
        None => (None, Vec::new()),
    }
}

/// Separate `node` from a block that ends right before it, so it does not continue that block's
/// last line (issues #570, #571, #583, #585).
///
/// ~keep Every block writes its own leading blank line, so a block after a list, a table or a
/// ~keep rule is separated whatever that block ended with. Inline content writes none, so it
/// ~keep continued the block's last line: a lazy continuation of the last list item, or one more
/// ~keep table row. In HTML, inline content after a block starts a block of its own, also when
/// ~keep the block sits at the end of an inline wrapper (`<span><ul>...</ul></span>text`).
/// ~keep Inside a list item the same holds, and `CommonMark` keeps a block in the item only when
/// ~keep its lines start at the item's content column: see `separate_in_list_item`.
fn separate_from_block(
    node: &tl::Node,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let HandlerContext {
        options, ctx, dom_ctx, ..
    } = handler;
    // ~keep A heading or a link label converts inline, but in a cell the text after a block needs the break.
    if output.is_empty() || (ctx.convert_as_inline && !ctx.in_table_cell) || ctx.in_code {
        return;
    }
    if ctx.in_table_cell {
        // ~keep A block in a cell ends with no line end, so the cell break separates the inline
        // ~keep content after it (issue #645). A line break is a break of its own, and without
        // ~keep `br_in_tables` a text's leading space is the break. A kept HTML block is text in a cell.
        // ~keep A nested table adds no break: a table moved out of the cell ends its own line, and the
        // ~keep text after a table folded into the cell joins the table's last row.
        if is_inline_content(node, node_handle, parser, dom_ctx)
            && !is_line_break(node_handle, parser, dom_ctx)
            && (options.br_in_tables || !starts_with_space(node))
            && crate::converter::utility::siblings::previous_content_block(node_handle, parser, dom_ctx)
                .is_some_and(|block| block != "table" && !ctx.preserve_tags.contains(block))
        {
            crate::converter::main_helpers::separate_block_in_cell(output, options.br_in_tables);
        }
        return;
    }
    if ctx.in_list_item {
        if !parent_is_list(node_handle, parser, dom_ctx) {
            separate_in_list_item(node, node_handle, parser, output, handler);
        }
    } else if !ctx.in_list
        && ends_with_block_line_end(output)
        && !output.ends_with("\n\n")
        && is_inline_content(node, node_handle, parser, dom_ctx)
        && crate::converter::utility::siblings::previous_content_block(node_handle, parser, dom_ctx).is_some()
    {
        output.push('\n');
    }
}

/// Start `node` at the list item's content column when it is a block after other content of the
/// item, or inline content after a block of the item (issue #583).
///
/// ~keep `CommonMark` keeps a block inside a list item only when every line of it starts at the
/// ~keep item's content column. A block after the item's text starts on a new line at that
/// ~keep column; each block handler then writes the blank line it needs before itself. Inline
/// ~keep content after a block starts a paragraph of its own: a blank line, then the column. A
/// ~keep heading is one line that nothing continues, so after it the column alone does, and the
/// ~keep list stays tight (spec example 300: `- ## Bar\n  baz`).
/// ~keep Where the item is not open (a list between markers, issue #615), a quote or a
/// ~keep list written within 3 columns of the innermost real item's content column (the start
/// ~keep of the line when no item is real) is still a real block, and text after it on the next
/// ~keep line continues its last paragraph lazily. That text gets the blank line and the real
/// ~keep item's column. Further in, the block's lines are the paragraph's own text, and a blank
/// ~keep line would split the paragraph between the markers.
/// ~keep A list is left out as the block: it already starts its own line at its own column.
/// ~keep A lone line break is not a block's last line: the block before it wrote nothing.
fn separate_in_list_item(
    node: &tl::Node,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let HandlerContext {
        options, ctx, dom_ctx, ..
    } = handler;
    // ~keep A block that preprocessing drops (a `<nav>`) writes nothing, so it gets no line.
    let starts_block = match node {
        tl::Node::Tag(tag) => dom_ctx.tag_info(node_handle.get_inner(), parser).is_some_and(|info| {
            is_block_level_element(&info.name)
                && !matches!(info.name.as_str(), "ul" | "ol" | "li")
                && !should_drop_for_preprocessing(
                    &info.name,
                    tag,
                    options,
                    info.name == "header" && is_page_header(node_handle, parser, dom_ctx),
                )
        }),
        _ => false,
    };
    let indent =
        crate::converter::list::utils::continuation_indent_string(ctx.list_indent_columns, options).unwrap_or_default();
    let (blank_line, block_continues_lazily) = if starts_block {
        // ~keep A hard break right before a block is dropped here too, since the line end
        // ~keep written below would hide it from the dispatch strip in `walk_node`.
        if options.newline_style == NewlineStyle::Backslash {
            strip_trailing_backslash_breaks(output, ctx.block_content_start);
        }
        let line_start = output.rfind('\n').map_or(0, |pos| pos + 1);
        let line = &output[line_start..];
        let after_content = line.is_empty()
            || (!line.trim().is_empty() && !crate::converter::list::utils::line_is_bare_list_marker(output));
        if !after_content {
            return;
        }
        (false, false)
    } else if ends_with_block_line_end(output) && is_inline_content(node, node_handle, parser, dom_ctx) {
        match crate::converter::utility::siblings::previous_content_block(node_handle, parser, dom_ctx) {
            Some(block) => (
                !matches!(block, "h1" | "h2" | "h3" | "h4" | "h5" | "h6"),
                matches!(block, "blockquote" | "ul" | "ol"),
            ),
            None => return,
        }
    } else {
        return;
    };
    let item_is_open = crate::converter::list::utils::item_is_open(output, &indent, ctx);
    let separates =
        item_is_open || (block_continues_lazily && crate::converter::list::utils::block_is_real(ctx, options));
    if !separates {
        return;
    }
    trim_trailing_whitespace(output);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    if blank_line && !output.ends_with("\n\n") {
        output.push('\n');
    }
    if item_is_open {
        output.push_str(&indent);
    } else if let Some(real_item_indent) =
        crate::converter::list::utils::continuation_indent_string(ctx.real_item_columns, options)
    {
        output.push_str(&real_item_indent);
    }
}

/// Whether the last line of `output` holds nothing but indentation and list markers.
///
/// ~keep The backward scan stops at the first other character, so a long line costs nothing.
fn at_line_start(output: &str) -> bool {
    let before =
        output.trim_end_matches(|c: char| c.is_ascii_digit() || matches!(c, ' ' | '\t' | '-' | '*' | '+' | '.' | ')'));
    (before.is_empty() || before.ends_with('\n'))
        && (output[before.len()..].trim().is_empty() || crate::converter::list::utils::line_is_bare_list_marker(output))
}

/// Whether `output` ends with a line end that a block wrote: a lone line break is not one, since
/// the block before it wrote nothing.
fn ends_with_block_line_end(output: &str) -> bool {
    output.len() >= 2 && output.ends_with('\n')
}

/// Whether `node` is inline content: non-blank text or an inline element.
fn is_inline_content(node: &tl::Node, node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    match node {
        tl::Node::Raw(bytes) => !bytes.as_utf8_str().trim().is_empty(),
        tl::Node::Tag(_) => dom_ctx
            .tag_info(node_handle.get_inner(), parser)
            .is_some_and(|info| is_inline_element(&info.name)),
        tl::Node::Comment(_) => false,
    }
}

/// Whether `node` is text that starts with whitespace.
fn starts_with_space(node: &tl::Node) -> bool {
    matches!(node, tl::Node::Raw(bytes) if bytes.as_bytes().first().is_some_and(u8::is_ascii_whitespace))
}

/// Whether `node_handle` is a `<br>` element.
fn is_line_break(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx
        .tag_info(node_handle.get_inner(), parser)
        .is_some_and(|info| info.name == "br")
}

/// Whether the parent of `node_handle` is a `<ul>` or `<ol>` (text or items between list items).
fn parent_is_list(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    dom_ctx
        .parent_of(node_handle.get_inner())
        .and_then(|parent_id| dom_ctx.tag_info(parent_id, parser))
        .is_some_and(|info| matches!(info.name.as_str(), "ul" | "ol"))
}

/// Recursively walk DOM nodes and convert to Markdown.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn walk_node(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    let ctx = handler.ctx;
    ctx.last_list.enter(output);
    // ~keep In a task item, the render of each node before the first content reports whether
    // ~keep it wrote, so the item knows which element wrote first (issue #650).
    match ctx.first_writer.as_ref().filter(|first_writer| first_writer.is_open()) {
        Some(first_writer) => {
            let start = output.len();
            convert_node(node_handle, parser, output, handler);
            first_writer.record(*node_handle, parser, output.get(start..));
        }
        None => convert_node(node_handle, parser, output, handler),
    }
    ctx.last_list.leave(output);
}

/// Convert one DOM node and its children to Markdown.
#[allow(clippy::only_used_in_recursion)]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::cast_possible_truncation)]
fn convert_node(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, handler: HandlerContext<'_>) {
    let options = handler.options;
    let ctx = handler.ctx;
    let depth = handler.depth;
    let dom_ctx = handler.dom_ctx;
    let Some(node) = node_handle.get(parser) else { return };

    if depth >= effective_max_depth(options) {
        ctx.depth_limit_reached.set(true);
        return;
    }

    separate_from_block(node, node_handle, parser, output, handler);

    match node {
        tl::Node::Raw(bytes) => {
            let raw = bytes.as_utf8_str();
            crate::converter::text_node::process_text_node(
                raw.as_ref(),
                node_handle,
                parser,
                output,
                crate::converter::block::container::HandlerContext::new(options, ctx, depth, dom_ctx),
            );
        }

        tl::Node::Tag(tag) => convert_tag(node_handle, tag, parser, output, handler),

        tl::Node::Comment(_) => {}
    }
}

#[cfg_attr(not(feature = "visitor"), allow(clippy::needless_return))]
fn convert_tag(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) {
    let options = handler.options;
    let depth = handler.depth;
    let dom_ctx = handler.dom_ctx;
    let tag_name = normalized_node_tag_name(node_handle, tag, parser, dom_ctx);
    let djot_scope = djot_context(node_handle, parser, tag_name.as_ref(), handler);
    let ctx = djot_scope.as_ref().unwrap_or(handler.ctx);
    #[cfg(feature = "visitor")]
    let visitor_output_start = output.len();
    #[cfg(feature = "visitor")]
    let visitor_element_state = if ctx.skip_visitor_hooks {
        None
    } else if let Some(ref visitor_handle) = ctx.visitor {
        use crate::converter::visitor_hooks::{VisitAction, build_visitor_element_state, handle_visitor_element_start};

        let state = build_visitor_element_state(tag_name.as_ref(), node_handle, parser, dom_ctx);
        let action = handle_visitor_element_start(visitor_handle, tag_name.as_ref(), tag, &state, output, depth);

        match action {
            VisitAction::Continue => Some(state),
            VisitAction::Skip => return,
            VisitAction::Custom => {
                if let Some(replacement) = output.get(visitor_output_start..) {
                    collect_document_attributes(tag_name.as_ref(), tag, ctx);
                    super::structure_capture::replace_element_at_start(tag_name.as_ref(), tag, ctx, replacement);
                }
                return;
            }
            VisitAction::Error => return,
        }
    } else {
        None
    };

    if skip_or_render_tag(
        tag_name.as_ref(),
        node_handle,
        tag,
        parser,
        output,
        HandlerContext::new(options, ctx, depth, dom_ctx),
    ) {
        return;
    }
    collect_document_attributes(tag_name.as_ref(), tag, ctx);
    let structure_capture = super::structure_capture::begin_element(tag_name.as_ref(), tag, ctx);
    #[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
    let element_output_start = output.len();
    strip_breaks_before_block(tag_name.as_ref(), output, options, ctx);

    #[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
    let stop = super::main_dispatch::dispatch_tag(
        tag_name.as_ref(),
        node_handle,
        tag,
        parser,
        output,
        HandlerContext::new(options, ctx, depth, dom_ctx),
    );

    #[cfg(feature = "visitor")]
    if !stop {
        super::structure_capture::handle_visitor_end(super::structure_capture::VisitorEndCapture {
            state: visitor_element_state.as_ref(),
            tag_name: tag_name.as_ref(),
            tag,
            output,
            element_output_start,
            ctx,
            depth,
        });
    }
    super::structure_capture::finish_element(structure_capture, tag_name.as_ref(), tag, ctx);
}

fn normalized_node_tag_name<'a>(
    node_handle: &tl::NodeHandle,
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    dom_ctx: &'a DomContext,
) -> Cow<'a, str> {
    match dom_ctx.tag_info(node_handle.get_inner(), parser) {
        Some(info) => Cow::Borrowed(info.name.as_str()),
        None => normalized_tag_name(tag.name().as_utf8_str()),
    }
}

fn djot_context(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    tag_name: &str,
    handler: HandlerContext<'_>,
) -> Option<Context> {
    let ctx = handler.ctx;
    (handler.options.output_format == OutputFormat::Djot
        && (ctx.djot_rule_like_text.is_none() || is_block_level_element(tag_name)))
    .then(|| Context {
        djot_rule_like_text: Some(crate::converter::context::DjotRuleLikeText::new(djot_rule_like_lines(
            *node_handle,
            parser,
            handler.dom_ctx,
        ))),
        ..ctx.clone()
    })
}

fn skip_or_render_tag(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
) -> bool {
    if should_skip_tag(tag_name, node_handle, tag, parser, handler) {
        if handler.ctx.inline_code_links.is_none() {
            trim_trailing_whitespace(output);
        }
        return true;
    }
    if handler.ctx.strip_tags.contains(tag_name) {
        for child_handle in tag.children().top().iter() {
            walk_node(
                child_handle,
                parser,
                output,
                HandlerContext::new(handler.options, handler.ctx, handler.depth + 1, handler.dom_ctx),
            );
        }
        return true;
    }
    let preserved_menu_placeholder = tag_name == "ul"
        && handler.ctx.preserve_tags.contains("menu")
        && tag.attributes().get(PRESERVED_MENU_ATTRIBUTE).is_some();
    (handler.ctx.preserve_tags.contains(tag_name) || preserved_menu_placeholder)
        && render_preserved_tag(
            tag_name,
            node_handle,
            parser,
            output,
            handler,
            preserved_menu_placeholder,
        )
}

fn should_skip_tag(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser,
    handler: HandlerContext<'_>,
) -> bool {
    #[cfg(feature = "visitor")]
    let visitor_is_active = handler.ctx.visitor.is_some() && !handler.ctx.skip_visitor_hooks;
    #[cfg(not(feature = "visitor"))]
    let visitor_is_active = false;
    (!visitor_is_active
        && should_drop_for_preprocessing(
            tag_name,
            tag,
            handler.options,
            tag_name == "header" && is_page_header(node_handle, parser, handler.dom_ctx),
        ))
        || (!handler.ctx.excluded_node_ids.is_empty()
            && handler.ctx.excluded_node_ids.contains(&node_handle.get_inner()))
}

fn render_preserved_tag(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: HandlerContext<'_>,
    preserved_menu_placeholder: bool,
) -> bool {
    let ctx = handler.ctx;
    let starts_line = at_line_start(output);
    let mut html = serialize_tag_to_html(node_handle, parser);
    if preserved_menu_placeholder {
        html = restore_preserved_menu_elements(&html).into_owned();
    }
    let custom_element_starts_block =
        ctx.in_list_item && tag_name.contains('-') && !crate::converter::list::utils::line_is_bare_list_marker(output);
    let opens_html_block = (starts_line || (preserved_menu_placeholder && ctx.in_list_item))
        && !ctx.in_marker_text()
        && !ctx.in_table_cell
        && !ctx.convert_as_inline
        && !ctx.in_code
        && (custom_element_starts_block || crate::converter::utility::escaping::opens_block(html.trim_start()));
    // ~keep Custom elements are not in the converter's block-tag allowlist, so
    // ~keep `separate_from_block` cannot put a preserved custom block at the item's
    // ~keep content column. A hyphenated HTML name identifies that custom-element
    // ~keep case at the preservation boundary (issue #658).
    if opens_html_block && ctx.in_list_item {
        crate::converter::list::utils::start_block_in_list_item(output, ctx, handler.options);
    }
    output.push_str(&html);
    // ~keep An HTML block ends only at a blank line, so one follows it (issue #655).
    if opens_html_block {
        output.push_str("\n\n");
    }
    true
}

#[cfg(feature = "metadata")]
fn collect_document_attributes(tag_name: &str, tag: &tl::HTMLTag<'_>, ctx: &Context) {
    if !matches!(tag_name, "html" | "head" | "body") || !ctx.metadata_wants_document {
        return;
    }
    let Some(collector) = ctx.metadata_collector.as_ref() else {
        return;
    };
    let mut collector = collector.borrow_mut();
    if let Some(lang) = tag.attributes().get("lang").flatten() {
        collector.set_language(lang.as_utf8_str().to_string());
    }
    if let Some(dir) = tag.attributes().get("dir").flatten() {
        collector.set_text_direction(dir.as_utf8_str().to_string());
    }
}

#[cfg(not(feature = "metadata"))]
const fn collect_document_attributes(_tag_name: &str, _tag: &tl::HTMLTag<'_>, _ctx: &Context) {}

/// ~keep Block dispatch is where a trailing hard-break run becomes knowably
/// ineffective; container endings handle the no-following-sibling case themselves.
fn strip_breaks_before_block(tag_name: &str, output: &mut String, options: &ConversionOptions, ctx: &Context) {
    if !ctx.in_code && options.newline_style == NewlineStyle::Backslash && is_block_level_element(tag_name) {
        strip_trailing_backslash_breaks(output, ctx.block_content_start);
    }
}

fn djot_rule_like_lines(node_handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> Vec<bool> {
    let mut text = String::with_capacity(64);
    let mut stack = vec![node_handle];
    while let Some(handle) = stack.pop() {
        match handle.get(parser) {
            Some(tl::Node::Raw(bytes)) => {
                let raw = bytes.as_utf8_str();
                text.push_str(crate::text::decode_html_entities_cow(raw.as_ref()).as_ref());
            }
            Some(tl::Node::Tag(tag)) => {
                if dom_ctx.tag_name_for(handle, parser).as_deref() == Some("br") {
                    text.push('\n');
                } else if let Some(children) = dom_ctx.children_of(handle.get_inner()) {
                    stack.extend(children.iter().rev().copied());
                } else {
                    let mut children: Vec<_> = tag.children().top().iter().copied().collect();
                    children.reverse();
                    stack.extend(children);
                }
            }
            Some(tl::Node::Comment(_)) | None => {}
        }
    }
    text.split('\n').map(crate::text::is_djot_rule_like).collect()
}
