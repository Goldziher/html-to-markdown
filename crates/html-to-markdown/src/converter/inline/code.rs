//! Handler for code-related inline elements (code, kbd, samp).
//!
//! Converts HTML code elements to Markdown inline code formatting with support for:
//! - Inline code blocks with backtick delimiters
//! - Keyboard input (<kbd>) rendered as code
//! - Sample output (<samp>) rendered as code
//! - Smart backtick escaping for nested backticks
//! - Delimiter spacing to prevent ambiguous parsing
//! - Nested code context tracking (suppress formatting in <code> within <code>)
//! - Visitor callbacks for custom code processing
//! - Whitespace normalization for kbd/samp elements

use crate::options::ConversionOptions;
use crate::text;
#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handler for code-related inline elements: code, kbd (keyboard), and samp (sample output).
///
/// Processes code content based on context:
/// - For <code> within <code>: passes content through without wrapping backticks (nested code detection)
/// - For <kbd> and <samp>: normalizes whitespace and wraps with backticks
/// - For standalone <code>: applies smart backtick escaping and delimiter spacing
/// - Handles visitor callbacks for custom behavior when feature is enabled
/// - Properly escapes backticks in content that contains them
///
/// # Note
/// This function references helper functions and `walk_node` from converter.rs
/// which must be accessible (pub(crate)) for this module to work correctly.
pub(crate) fn handle(
    tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    // Import helper functions from parent converter module

    match tag_name {
        "code" => {
            handle_code(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "kbd" | "samp" => {
            handle_kbd_samp(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        _ => {}
    }
}

/// Handle inline code element (<code> tag).
///
/// Smart handling of nested code:
/// - If already in code context (ctx.in_code), just process children without wrapping
/// - Otherwise, wraps content with backticks with smart escaping logic
///
/// Smart backtick escaping:
/// - Detects consecutive backticks in content
/// - Adds extra backticks if needed to avoid ambiguous parsing
/// - Adds space delimiters if content starts/ends with backticks or spaces
fn handle_code(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    #[allow(unused_imports)]
    use crate::converter::{serialize_node, walk_node};

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let code_ctx = Context {
        in_code: true,
        ..ctx.clone()
    };

    // Nested code detection: if already in code, just process children
    if ctx.in_code {
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, &code_ctx, depth + 1, dom_ctx);
        }
    } else {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &code_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }

        let trimmed = &content;

        if !content.trim().is_empty() {
            #[cfg(feature = "visitor")]
            let code_output = if let Some(ref visitor_handle) = ctx.visitor {
                use crate::visitor::{NodeContext, NodeType, VisitResult};

                let attributes: BTreeMap<String, String> = tag
                    .attributes()
                    .iter()
                    .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                    .collect();

                let node_id = node_handle.get_inner();
                let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
                let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

                let node_ctx = NodeContext {
                    node_type: NodeType::Code,
                    tag_name: tag.name().as_utf8_str().to_string(),
                    attributes,
                    depth,
                    index_in_parent,
                    parent_tag,
                    is_inline: true,
                };

                let mut visitor = visitor_handle.borrow_mut();
                match visitor.visit_code_inline(&node_ctx, trimmed) {
                    VisitResult::Continue => None,
                    VisitResult::Custom(custom) => Some(custom),
                    VisitResult::Skip => Some(String::new()),
                    VisitResult::PreserveHtml => Some(serialize_node(node_handle, parser)),
                    VisitResult::Error(err) => {
                        if ctx.visitor_error.borrow().is_none() {
                            *ctx.visitor_error.borrow_mut() = Some(err);
                        }
                        None
                    }
                }
            } else {
                None
            };

            #[cfg(feature = "visitor")]
            if let Some(custom_output) = code_output {
                output.push_str(&custom_output);
            } else {
                render_code_with_escaping(trimmed, output);
            }

            #[cfg(not(feature = "visitor"))]
            {
                render_code_with_escaping(trimmed, output);
            }
        }
    }
}

/// Handle keyboard and sample output elements (<kbd> and <samp> tags).
///
/// These elements are rendered as inline code with:
/// - Whitespace normalization (via text::normalize_whitespace)
/// - Chomp inline handling for prefix/suffix spacing
/// - Simple single backtick wrapping (no smart escaping for keyboard/sample)
fn handle_kbd_samp(
    _tag_name: &str,
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::{append_inline_suffix, chomp_inline, walk_node};

    let Some(node) = node_handle.get(parser) else { return };

    let _tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let code_ctx = Context {
        in_code: true,
        ..ctx.clone()
    };

    let mut content = String::with_capacity(32);
    let children = _tag.children();
    {
        for child_handle in children.top().iter() {
            walk_node(
                child_handle,
                parser,
                &mut content,
                options,
                &code_ctx,
                depth + 1,
                dom_ctx,
            );
        }
    }

    // Normalize whitespace for kbd/samp (unlike code, which preserves it)
    let normalized = text::normalize_whitespace(&content);
    let (prefix, suffix, trimmed) = chomp_inline(&normalized);

    if !content.trim().is_empty() {
        output.push_str(prefix);
        output.push('`');
        output.push_str(trimmed);
        output.push('`');
        append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
    } else if !content.is_empty() {
        output.push_str(prefix);
        append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
    }
}

/// Render inline code with smart backtick escaping.
///
/// Handles the logic for:
/// 1. Detecting backticks in content
/// 2. Calculating the required number of backticks
/// 3. Adding delimiter spaces when needed
///
/// # Backtick Escaping Logic
///
/// - If content contains no backticks: use single backtick delimiters
/// - If content has single backticks: use double backtick delimiters
/// - If content has consecutive backticks: use single backtick but rely on spacing
///
/// # Delimiter Space Rules
///
/// Add space delimiters if:
/// - Content is all spaces
/// - Content starts/ends with backtick
/// - Content starts and ends with spaces AND contains backticks
fn render_code_with_escaping(trimmed: &str, output: &mut String) {
    let contains_backtick = trimmed.contains('`');

    let needs_delimiter_spaces = {
        let first_char = trimmed.chars().next();
        let last_char = trimmed.chars().last();
        let starts_with_space = first_char == Some(' ');
        let ends_with_space = last_char == Some(' ');
        let starts_with_backtick = first_char == Some('`');
        let ends_with_backtick = last_char == Some('`');
        let all_spaces = trimmed.chars().all(|c| c == ' ');

        all_spaces
            || starts_with_backtick
            || ends_with_backtick
            || (starts_with_space && ends_with_space && contains_backtick)
    };

    let (num_backticks, needs_spaces) = if contains_backtick {
        let max_consecutive = trimmed
            .chars()
            .fold((0, 0), |(max, current), c| {
                if c == '`' {
                    let new_current = current + 1;
                    (max.max(new_current), new_current)
                } else {
                    (max, 0)
                }
            })
            .0;
        let num = if max_consecutive == 1 { 2 } else { 1 };
        (num, needs_delimiter_spaces)
    } else {
        (1, needs_delimiter_spaces)
    };

    for _ in 0..num_backticks {
        output.push('`');
    }
    if needs_spaces {
        output.push(' ');
    }
    output.push_str(trimmed);
    if needs_spaces {
        output.push(' ');
    }
    for _ in 0..num_backticks {
        output.push('`');
    }
}
