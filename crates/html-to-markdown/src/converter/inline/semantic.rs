//! Handler for semantic inline elements (mark, del, s, ins, u, small, sub, sup).
//!
//! Converts HTML semantic tags to Markdown formatting with support for:
//! - Highlight/mark element with configurable styles (==, ::, ^^, <mark>)
//! - Strikethrough (del, s tags) with ~~ syntax
//! - Underline/inserted text (ins, u tags) with == syntax
//! - Small text (passes through without formatting)
//! - Subscript and superscript with configurable symbols
//! - Visitor callbacks for custom processing (feature-gated)

use crate::options::ConversionOptions;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handler for semantic inline elements: mark, del, s, ins, u, small, sub, sup.
///
/// Processes semantic content based on tag and options:
/// - Mark: configurable highlight style (==, ::, ^^, <mark>, **bold, none)
/// - Del/S: strikethrough with ~~ and visitor callback support
/// - Ins: underline with == and visitor callback support
/// - U: underline with visitor callback support
/// - Small: pass through without formatting
/// - Sub/Sup: wrap with configurable symbols
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
        "mark" => {
            handle_mark(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "del" | "s" => {
            handle_strikethrough(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "ins" => {
            handle_inserted(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "u" => {
            handle_underline(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "small" => {
            handle_small(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "sub" => {
            handle_subscript(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        "sup" => {
            handle_superscript(node_handle, parser, output, options, ctx, depth, dom_ctx);
        }
        _ => {}
    }
}

/// Handle mark (highlight) element with configurable styles.
///
/// Supports multiple highlight styles:
/// - DoubleEqual: `==highlighted==`
/// - Html: `<mark>highlighted</mark>`
/// - Bold: `**highlighted**`
/// - None: just pass through content
fn handle_mark(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.convert_as_inline {
        // In inline conversion context, just pass through children
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    } else {
        use crate::options::HighlightStyle;
        match options.highlight_style {
            HighlightStyle::DoubleEqual => {
                output.push_str("==");
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
                output.push_str("==");
            }
            HighlightStyle::Html => {
                output.push_str("<mark>");
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
                output.push_str("</mark>");
            }
            HighlightStyle::Bold => {
                let symbol = options.strong_em_symbol.to_string().repeat(2);
                output.push_str(&symbol);
                let bold_ctx = Context {
                    in_strong: true,
                    ..ctx.clone()
                };
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, &bold_ctx, depth + 1, dom_ctx);
                }
                output.push_str(&symbol);
            }
            HighlightStyle::None => {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
        }
    }
}

/// Handle strikethrough element (del, s tags).
///
/// Converts to `~~content~~` syntax. Suppresses formatting in code context.
/// Supports visitor callbacks when the visitor feature is enabled.
#[allow(unused_variables)]
fn handle_strikethrough(
    tag_name: &str,
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

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if ctx.in_code {
        // Suppress strikethrough in code context, just process children
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    } else {
        let mut content = String::with_capacity(32);
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
        }

        #[cfg(feature = "visitor")]
        let strikethrough_output = if let Some(ref visitor_handle) = ctx.visitor {
            use crate::converter::get_text_content;
            use crate::visitor::{NodeContext, NodeType, VisitResult};
            use std::collections::BTreeMap;

            let text_content = get_text_content(node_handle, parser, dom_ctx);
            let attributes: BTreeMap<String, String> = tag
                .attributes()
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
                .collect();

            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

            let node_ctx = NodeContext {
                node_type: NodeType::Strikethrough,
                tag_name: tag_name.to_string(),
                attributes,
                depth,
                index_in_parent,
                parent_tag,
                is_inline: true,
            };

            let mut visitor = visitor_handle.borrow_mut();
            match visitor.visit_strikethrough(&node_ctx, &text_content) {
                VisitResult::Continue => None,
                VisitResult::Custom(custom) => Some(custom),
                VisitResult::Skip => Some(String::new()),
                VisitResult::PreserveHtml => {
                    use crate::converter::serialize_node;
                    Some(serialize_node(node_handle, parser))
                }
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
        if let Some(custom_output) = strikethrough_output {
            output.push_str(&custom_output);
        } else {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                output.push_str("~~");
                output.push_str(trimmed);
                output.push_str("~~");
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            }
        }

        #[cfg(not(feature = "visitor"))]
        {
            let (prefix, suffix, trimmed) = chomp_inline(&content);
            if !content.trim().is_empty() {
                output.push_str(prefix);
                output.push_str("~~");
                output.push_str(trimmed);
                output.push_str("~~");
                append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
            } else if !content.is_empty() {
                output.push_str(prefix);
                append_inline_suffix(output, suffix, false, node_handle, parser, dom_ctx);
            }
        }
    }
}

/// Handle inserted/underlined text (ins tag).
///
/// Converts to `==content==` syntax. Supports visitor callbacks when enabled.
fn handle_inserted(
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

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let mut content = String::with_capacity(32);
    let children = tag.children();
    for child_handle in children.top().iter() {
        walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
    }

    #[cfg(feature = "visitor")]
    let underline_output = if let Some(ref visitor_handle) = ctx.visitor {
        use crate::converter::get_text_content;
        use crate::visitor::{NodeContext, NodeType, VisitResult};
        use std::collections::BTreeMap;

        let text_content = get_text_content(node_handle, parser, dom_ctx);
        let attributes: BTreeMap<String, String> = tag
            .attributes()
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
            .collect();

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

        let node_ctx = NodeContext {
            node_type: NodeType::Underline,
            tag_name: "ins".to_string(),
            attributes,
            depth,
            index_in_parent,
            parent_tag,
            is_inline: true,
        };

        let mut visitor = visitor_handle.borrow_mut();
        match visitor.visit_underline(&node_ctx, &text_content) {
            VisitResult::Continue => None,
            VisitResult::Custom(custom) => Some(custom),
            VisitResult::Skip => Some(String::new()),
            VisitResult::PreserveHtml => {
                use crate::converter::serialize_node;
                Some(serialize_node(node_handle, parser))
            }
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
    if let Some(custom_output) = underline_output {
        output.push_str(&custom_output);
    } else {
        let (prefix, suffix, trimmed) = chomp_inline(&content);
        if !trimmed.is_empty() {
            output.push_str(prefix);
            output.push_str("==");
            output.push_str(trimmed);
            output.push_str("==");
            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
        }
    }

    #[cfg(not(feature = "visitor"))]
    {
        let (prefix, suffix, trimmed) = chomp_inline(&content);
        if !trimmed.is_empty() {
            output.push_str(prefix);
            output.push_str("==");
            output.push_str(trimmed);
            output.push_str("==");
            append_inline_suffix(output, suffix, !trimmed.is_empty(), node_handle, parser, dom_ctx);
        }
    }
}

/// Handle underline element (u tag).
///
/// Just passes through content (HTML doesn't have native underline in Markdown).
/// Supports visitor callbacks when enabled, which can provide custom formatting.
fn handle_underline(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::converter::get_text_content;
        use crate::visitor::{NodeContext, NodeType, VisitResult};
        use std::collections::BTreeMap;

        let text_content = get_text_content(node_handle, parser, dom_ctx);
        let attributes: BTreeMap<String, String> = tag
            .attributes()
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
            .collect();

        let node_id = node_handle.get_inner();
        let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
        let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

        let node_ctx = NodeContext {
            node_type: NodeType::Underline,
            tag_name: "u".to_string(),
            attributes,
            depth,
            index_in_parent,
            parent_tag,
            is_inline: true,
        };

        let mut visitor = visitor_handle.borrow_mut();
        match visitor.visit_underline(&node_ctx, &text_content) {
            VisitResult::Continue => {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
            VisitResult::Custom(custom) => {
                output.push_str(&custom);
            }
            VisitResult::Skip => {}
            VisitResult::PreserveHtml => {
                use crate::converter::serialize_node;
                output.push_str(&serialize_node(node_handle, parser));
            }
            VisitResult::Error(err) => {
                if ctx.visitor_error.borrow().is_none() {
                    *ctx.visitor_error.borrow_mut() = Some(err);
                }
                let children = tag.children();
                for child_handle in children.top().iter() {
                    walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
                }
            }
        }
    } else {
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    }

    #[cfg(not(feature = "visitor"))]
    {
        let children = tag.children();
        for child_handle in children.top().iter() {
            walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
        }
    }
}

/// Handle small element.
///
/// Small text has no direct Markdown equivalent, so just pass through content.
fn handle_small(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    let children = tag.children();
    for child_handle in children.top().iter() {
        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
    }
}

/// Handle subscript element (sub tag).
///
/// Wraps content with configurable subscript symbol from options.
fn handle_subscript(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if !ctx.in_code && !options.sub_symbol.is_empty() {
        output.push_str(&options.sub_symbol);
    }

    let children = tag.children();
    for child_handle in children.top().iter() {
        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
    }

    if !ctx.in_code && !options.sub_symbol.is_empty() {
        if options.sub_symbol.starts_with('<') && !options.sub_symbol.starts_with("</") {
            output.push_str(&options.sub_symbol.replace('<', "</"));
        } else {
            output.push_str(&options.sub_symbol);
        }
    }
}

/// Handle superscript element (sup tag).
///
/// Wraps content with configurable superscript symbol from options.
fn handle_superscript(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let Some(node) = node_handle.get(parser) else { return };

    let tag = match node {
        tl::Node::Tag(tag) => tag,
        _ => return,
    };

    if !ctx.in_code && !options.sup_symbol.is_empty() {
        output.push_str(&options.sup_symbol);
    }

    let children = tag.children();
    for child_handle in children.top().iter() {
        walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
    }

    if !ctx.in_code && !options.sup_symbol.is_empty() {
        if options.sup_symbol.starts_with('<') && !options.sup_symbol.starts_with("</") {
            output.push_str(&options.sup_symbol.replace('<', "</"));
        } else {
            output.push_str(&options.sup_symbol);
        }
    }
}
