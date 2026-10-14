//! Handlers for the `<select>`, `<datalist>`, `<optgroup>` and `<option>` form elements.

use super::FormContext;
use super::spacing::separate_from_next_text;
use super::walk_node;
use crate::converter::walk::{Close, Open};

/// Handles the `<select>` and `<datalist>` elements.
///
/// The options are one control: one space separates each option from the option before it,
/// and the control from the text before and after it. Only the options of a `<select>` or a
/// `<datalist>` are inline text.
pub fn handle_option_list(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    let FormContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut rendered = String::new();
        // ~keep A select list and a data list are one control of inline text. An option that
        // ~keep no such element holds is no list control in a browser, so a block in it stays
        // ~keep a block: its children keep the context of its parent.
        let children_ctx = if matches!(tag_name, "select" | "datalist") {
            ctx.inline_buffer(output, false)
        } else {
            ctx.clone()
        };
        let list_ctx = super::Context {
            in_option_list: true,
            ..children_ctx
        };
        if tag_name == "option" {
            let list_context = FormContext {
                ctx: &list_ctx,
                ..context
            };
            handle_option(tag_name, node_handle, parser, &mut rendered, list_context);
        } else {
            for child_handle in tag.children().top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut rendered,
                    crate::converter::block::container::HandlerContext::new(options, &list_ctx, depth + 1, dom_ctx),
                );
            }
        }
        if super::spacing::write_option_list(output, &rendered, ctx.in_code) {
            separate_from_next_text(output, *node_handle, parser, dom_ctx);
        }
    }
}

/// Writes an `<option>` of an option list: a word of the list.
///
/// Selection state does not change the visible label.
pub fn handle_option(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    let FormContext {
        options,
        ctx,
        depth,
        dom_ctx,
    } = context;
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut text = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut text,
                    crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
                );
            }
        }

        super::spacing::write_option(output, &text);
    }
}

pub fn open_option(
    kind: &str,
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    handler: crate::converter::block::container::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    let list = kind != "option" || !handler.ctx.in_option_list;
    let child_ctx = if matches!(kind, "select" | "datalist") {
        handler.ctx.inline_buffer(output, false)
    } else {
        handler.ctx.clone()
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: Some(std::rc::Rc::new(super::Context {
            in_option_list: true,
            ..child_ctx
        })),
        close: Close::Capture(crate::converter::walk::CaptureEnd::Option(OptionEnd {
            list,
            word: kind == "option",
        })),
    })
}

/// What ends an `<option>` or a `<select>` that captured its children.
pub struct OptionEnd {
    pub(crate) list: bool,
    pub(crate) word: bool,
}

pub fn close_option(
    end: &OptionEnd,
    content: &str,
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
) {
    if !end.list {
        super::spacing::write_option(output, content);
        return;
    }
    let mut rendered = String::new();
    let content = if end.word {
        super::spacing::write_option(&mut rendered, content);
        &rendered
    } else {
        content
    };
    if super::spacing::write_option_list(output, content, handler.ctx.in_code) {
        separate_from_next_text(output, *node, parser, handler.dom_ctx);
    }
}
