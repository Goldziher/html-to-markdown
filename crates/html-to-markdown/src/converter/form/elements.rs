//! Handlers for HTML form elements.
//!
//! This module provides comprehensive handling for all HTML form-related elements,
//! including form containers, input controls, and measurement elements.
//!
//! Processed elements:
//! - **Form containers**: `<form>`, `<fieldset>`, `<legend>`, `<label>`
//! - **Text inputs**: `<input>`, `<textarea>`, `<button>`
//! - **Select controls**: `<select>`, `<option>`, `<optgroup>`
//! - **Measurement**: `<progress>`, `<meter>`, `<output>`, `<datalist>`
//!
//! In Markdown, forms are typically not fully representable, so the handlers
//! extract and format the content in a readable manner.

use super::FormContext;
use super::option::{handle_option, handle_option_list};
use super::spacing::{ControlStart, line_goes_on_after_parent, separate_from_next_text};
use super::walk_node;
use crate::converter::walk::{Close, Open};
#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Run `<form>`'s visitor hook, if one is installed.
///
/// Returns `true` when the visitor already produced (or explicitly skipped) the output for
/// this node and the caller must return without falling through to its default rendering;
/// `false` when there is no visitor or it returned `VisitResult::Continue`.
///
/// `handle_form` calls it, and so does the dispatcher for a form inside a heading, which is
/// written as a `div`.
#[cfg(feature = "visitor")]
pub fn dispatch_form_visitor(
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) -> bool {
    let FormContext {
        ctx, depth, dom_ctx, ..
    } = context;
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(ref visitor_handle) = ctx.visitor else {
        return false;
    };

    let node_id = node_handle.get_inner();
    let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
    let action = tag
        .attributes()
        .get("action")
        .flatten()
        .map(|v| v.as_utf8_str().into_owned());
    let method = tag
        .attributes()
        .get("method")
        .flatten()
        .map(|v| v.as_utf8_str().into_owned());
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Form,
        Cow::Borrowed("form"),
        tag,
        depth,
        index_in_parent,
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_form(&node_ctx, action.as_deref(), method.as_deref())
    };
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str(&custom);
            output.push_str("\n\n");
            true
        }
        VisitResult::PreserveHtml => {
            use crate::converter::utility::serialization::serialize_node;
            output.push_str(&serialize_node(node_handle, parser));
            true
        }
        VisitResult::Error(err) => {
            if ctx.visitor_error.borrow().is_none() {
                *ctx.visitor_error.borrow_mut() = Some(err);
            }
            true
        }
    }
}

/// Handles the `<form>` element.
///
/// A form element is a container for form controls. In Markdown, it's rendered
/// as a block container with its content visible.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is collected, trimmed, and wrapped with blank lines
/// - **Empty content**: Skipped entirely
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_form(
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
        #[cfg(feature = "visitor")]
        if dispatch_form_visitor(node_handle, tag, parser, output, context) {
            return;
        }

        if ctx.convert_as_inline {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    super::walk_node(
                        child_handle,
                        parser,
                        output,
                        crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
                    );
                }
            }
            return;
        }

        let mut rendered = String::new();
        let children = tag.children();
        {
            // ~keep The element is written at the start of the line, so inside it a list item
            // ~keep has ended (issue #583).
            let block_ctx = super::Context {
                list_item_open: false,
                ..ctx.clone()
            };
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    &mut rendered,
                    crate::converter::block::container::HandlerContext::new(options, &block_ctx, depth + 1, dom_ctx),
                );
            }
        }

        let trimmed = rendered.trim();
        if !trimmed.is_empty() {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }

            output.push_str(trimmed);
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<fieldset>` element.
///
/// A fieldset element groups form controls with an optional legend.
/// In Markdown, it's rendered as a block container.
///
/// # Behavior
///
/// - **Inline mode**: Children are processed inline without block spacing
/// - **Block mode**: Content is collected, trimmed, and wrapped with blank lines
/// - **Empty content**: Skipped entirely
pub fn handle_fieldset(
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
    if ctx.in_code && !ctx.in_code_block {
        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            write_code_form_group(tag, parser, output, context);
        }
        return;
    }
    crate::converter::block::div::handle(
        node_handle,
        parser,
        output,
        crate::converter::block::container::HandlerContext::new(options, ctx, depth, dom_ctx),
    );
}

/// Handles the `<legend>` element.
///
/// A legend element provides a caption for a fieldset. It's rendered as
/// strong (bold) text to distinguish it from regular content.
///
/// # Behavior
///
/// - **Block mode**: Content is wrapped in strong markers (e.g., `**text**`)
/// - **Inline mode**: Content is rendered without emphasis
/// - Uses the configured strong/emphasis symbol from `ConversionOptions`
pub fn handle_legend(
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
        if ctx.in_code && !ctx.in_code_block {
            write_code_form_group(tag, parser, output, context);
            return;
        }
        let mut rendered = String::new();

        let mut legend_ctx = ctx.inline_buffer(output, !ctx.convert_as_inline);
        if !ctx.convert_as_inline {
            legend_ctx.in_strong = true;
        }

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut rendered,
                    crate::converter::block::container::HandlerContext::new(options, &legend_ctx, depth + 1, dom_ctx),
                );
            }
        }

        let trimmed = rendered.trim();
        if !trimmed.is_empty() {
            if ctx.convert_as_inline {
                output.push_str(trimmed);
            } else {
                let mut bold = String::with_capacity(trimmed.len() + 4);
                crate::converter::inline::emphasis::emit_strong_wrapped_blocks(
                    &mut bold,
                    trimmed,
                    crate::converter::inline::wrapped::InlineSite {
                        node_handle,
                        parser,
                        dom_ctx,
                        ctx,
                        options,
                    },
                );
                crate::converter::block::div::push_block(output, options, ctx, &bold);
            }
        }
    }
}

// ~keep Form-group block boundaries collapse to spaces inside one code span (#815),
// ~keep and children stay in its buffer so collected link ranges remain valid.
fn write_code_form_group(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    output: &mut String,
    context: FormContext<'_>,
) {
    let original_len = output.len();
    if !output.is_empty() && !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
    let content_start = output.len();
    let code_ctx = super::Context {
        convert_as_inline: true,
        ..context.ctx.clone()
    };
    for child in tag.children().top().iter() {
        walk_node(
            child,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(
                context.options,
                &code_ctx,
                context.depth + 1,
                context.dom_ctx,
            ),
        );
    }
    if output.len() == content_start {
        output.truncate(original_len);
    } else if !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
}

/// Handles the `<label>` element.
///
/// A label element associates text with a form control. It is inline content. White space at
/// the end of its content stays in the line, so `<label>Name <input></label>` and the text after
/// it do not join.
pub fn handle_label(
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
        let children = tag.children();
        // ~keep A code block shows the text of a label as written, so nothing is trimmed (issue #822).
        if ctx.in_code_block {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    output,
                    crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
                );
            }
            return;
        }
        let mut rendered = String::new();
        let label_ctx = ctx.inline_buffer(output, false);
        let first = first_content(tag, parser);
        let mut first_button_wrote = false;
        {
            for child_handle in children.top().iter() {
                ctx.button_wrote.set(false);
                super::walk_node(
                    child_handle,
                    parser,
                    &mut rendered,
                    crate::converter::block::container::HandlerContext::new(options, &label_ctx, depth + 1, dom_ctx),
                );
                if first.is_some_and(|handle| handle.get_inner() == child_handle.get_inner()) {
                    first_button_wrote = ctx.button_wrote.get();
                }
            }
        }

        let trimmed = rendered.trim();
        if !trimmed.is_empty() {
            // ~keep In the label's own buffer a control at its start has no text before it.
            let start = ControlStart::new(output);
            output.push_str(trimmed);
            // ~keep In code the text of a control is written as the source has it.
            if !ctx.in_code && starts_with_control(first, parser, dom_ctx, first_button_wrote) {
                start.finish(output);
            }
            if rendered.ends_with([' ', '\t']) {
                output.push(' ');
            }
        }
    }
}

/// The first child with content after transparent spans, white space and comments are ignored.
fn first_content(tag: &tl::HTMLTag, parser: &tl::Parser) -> Option<tl::NodeHandle> {
    tag.children()
        .top()
        .iter()
        .copied()
        .find(|child| first_visible_content(*child, parser).is_some())
}

// ~keep Spans are transparent to control boundaries; an iterative search avoids source-controlled recursion.
fn first_visible_content(handle: tl::NodeHandle, parser: &tl::Parser) -> Option<tl::NodeHandle> {
    let mut pending = vec![handle];
    while let Some(handle) = pending.pop() {
        match handle.get(parser) {
            Some(tl::Node::Raw(raw)) if !raw.as_utf8_str().trim().is_empty() => return Some(handle),
            Some(tl::Node::Tag(tag)) if tag.name().as_utf8_str().eq_ignore_ascii_case("span") => {
                let start = pending.len();
                pending.extend(tag.children().top().iter().copied());
                pending[start..].reverse();
            }
            Some(tl::Node::Tag(_)) => return Some(handle),
            _ => {}
        }
    }
    None
}

/// Whether `first`, the first content of a label, separates the label from the text before it:
/// a control that writes text of its own, or an input with white space after it. A button is one
/// only when it wrote text of its own (`button_wrote`): an empty button, or one whose visitor
/// skipped it or wrote text for it, does not separate.
fn starts_with_control(
    first: Option<tl::NodeHandle>,
    parser: &tl::Parser,
    dom_ctx: &crate::converter::DomContext,
    button_wrote: bool,
) -> bool {
    let Some(handle) = first.and_then(|handle| first_visible_content(handle, parser)) else {
        return false;
    };
    let Some(tl::Node::Tag(control)) = handle.get(parser) else {
        return false;
    };
    match crate::converter::utility::content::normalized_tag_name(control.name().as_utf8_str()).as_ref() {
        "input" => super::spacing::white_space_follows(handle.get_inner(), parser, dom_ctx),
        "button" => button_wrote,
        name => super::spacing::writes_own_text(name),
    }
}

/// Whether the `type` of the input `tag` is `checkbox`.
fn has_checkbox_type(tag: &tl::HTMLTag) -> bool {
    tag.attributes()
        .get("type")
        .flatten()
        .is_some_and(|value| value.as_utf8_str().eq_ignore_ascii_case("checkbox"))
}

/// The checked state of `tag` when it is a checkbox a reader sees as one. An input whose `role`
/// names another role is not one: `role="button"` marks the switch of a menu.
pub fn checkbox_state(tag: &tl::HTMLTag) -> Option<bool> {
    let attributes = tag.attributes();
    let is_checkbox = has_checkbox_type(tag);
    let has_other_role = attributes.get("role").flatten().is_some_and(|role| {
        role.as_utf8_str().split_ascii_whitespace().next().is_some_and(|first| {
            !["checkbox", "switch", "menuitemcheckbox"]
                .iter()
                .any(|checkbox_role| first.eq_ignore_ascii_case(checkbox_role))
        })
    });
    (is_checkbox && !has_other_role).then(|| attributes.get("checked").is_some())
}

/// Writes what a reader sees of an input. An input is a control, not text: it writes nothing and
/// adds no space. A checkbox is the content of a table cell that holds nothing else, so it writes
/// its state there.
///
/// ~keep The task marker `[ ]` of a list item belongs to `list/item.rs`.
fn emit_input(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, context: FormContext<'_>) {
    let FormContext {
        options, ctx, dom_ctx, ..
    } = context;
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    if !has_checkbox_type(tag) {
        return;
    }
    let id = node_handle.get_inner();
    // ~keep Strict mode writes what it wrote before the rules of running text: any space
    // ~keep before the checkbox goes, the indent of a line too.
    let space_goes = if crate::converter::utility::content::keeps_source_white_space(options, ctx) {
        output.ends_with(' ')
    } else {
        ends_with_a_space_between_words(output)
    };
    match checkbox_state(tag) {
        Some(checked) if ctx.in_cell_of_inputs => output.push_str(if checked { "[x]" } else { "[ ]" }),
        // ~keep The white space on both sides of a checkbox that writes nothing is one space
        // ~keep between two words. A checkbox with the role of a button writes nothing too.
        // ~keep The space after a line end is the indent of the line, not white space between
        // ~keep words: `<li><p>x</p><input type="checkbox"> a</li>` keeps the two spaces of `a`.
        _ if !ctx.in_code && space_goes && super::spacing::white_space_follows(id, parser, dom_ctx) => {
            output.pop();
        }
        _ => {}
    }
}

/// Whether `output` ends with a space that follows a word on its line.
fn ends_with_a_space_between_words(output: &str) -> bool {
    let before_spaces = output.trim_end_matches(' ');
    before_spaces.len() < output.len() && !before_spaces.is_empty() && !before_spaces.ends_with('\n')
}

/// Handles the `<input>` element.
///
/// A checkbox writes its checked state; other inputs have no text output.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_input(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    let FormContext {
        ctx, depth, dom_ctx, ..
    } = context;
    #[cfg(feature = "visitor")]
    if let Some(ref visitor_handle) = ctx.visitor {
        use crate::visitor::{NodeContext, NodeType, VisitResult};

        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
            let input_type = tag
                .attributes()
                .get("type")
                .flatten()
                .map_or_else(|| std::borrow::Cow::Borrowed("text"), |v| v.as_utf8_str())
                .into_owned();
            let name = tag
                .attributes()
                .get("name")
                .flatten()
                .map(|v| v.as_utf8_str().into_owned());
            let value = tag
                .attributes()
                .get("value")
                .flatten()
                .map(|v| v.as_utf8_str().into_owned());
            let node_ctx = NodeContext::with_lazy_attributes(
                NodeType::Input,
                Cow::Borrowed("input"),
                tag,
                depth,
                index_in_parent,
                parent_tag.map(Cow::Borrowed),
                true,
            );
            let visit_result = {
                let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
                visitor.visit_input(&node_ctx, &input_type, name.as_deref(), value.as_deref())
            };
            match visit_result {
                VisitResult::Continue => {}
                VisitResult::Skip => return,
                VisitResult::Custom(custom) => {
                    output.push_str(&custom);
                    return;
                }
                VisitResult::PreserveHtml => {
                    use crate::converter::utility::serialization::serialize_node;
                    output.push_str(&serialize_node(node_handle, parser));
                    return;
                }
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return;
                }
            }
        }
    }

    emit_input(node_handle, parser, output, context);
}

/// Writes the children of a control that ends its line: `<textarea>`, `<output>`, `<meter>` and
/// `<progress>`.
///
/// The text is a word of its own after the text before it. A blank line follows it where a
/// block holds the control. In inline mode and in an inline element the line goes on, and the
/// text after the control is a word of its own too.
///
/// This is the first half: it records where the control starts and lists its children.
pub fn open_line_end_control(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: None,
        close: Close::LineEndControl(LineEndControlEnd {
            start: ControlStart::new(output),
            node: *node_handle,
        }),
    })
}

/// What `close_line_end_control` needs from `open_line_end_control`.
pub struct LineEndControlEnd {
    start: ControlStart,
    node: tl::NodeHandle,
}

/// Ends the line of a control after its text.
pub fn close_line_end_control(
    end: &LineEndControlEnd,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
) {
    let LineEndControlEnd {
        start,
        node: node_handle,
    } = end;
    let node_handle = *node_handle;
    let ctx = handler.ctx;
    let dom_ctx = handler.dom_ctx;
    // ~keep In code the text of a control is written as the source has it: no space is added
    // ~keep and the line does not end.
    if ctx.in_code || !start.finish(output) {
        return;
    }
    if ctx.convert_as_inline || line_goes_on_after_parent(node_handle.get_inner(), parser, dom_ctx) {
        separate_from_next_text(output, node_handle, parser, dom_ctx);
    } else {
        output.push_str("\n\n");
    }
}

/// Gives a `<button>` element to the visitor. Returns whether the visitor wrote or skipped the
/// button, so that the walk does not open it.
///
/// A button element represents a clickable button. Its text content is plain text that ends
/// its line: without a visitor result, `open_line_end_control` writes it.
#[cfg(feature = "visitor")]
pub fn visit_button(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) -> bool {
    let FormContext {
        ctx, depth, dom_ctx, ..
    } = context;
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        if let Some(ref visitor_handle) = ctx.visitor {
            use crate::converter::get_text_content;
            use crate::visitor::{NodeContext, NodeType, VisitResult};

            let text = get_text_content(node_handle, parser, dom_ctx);
            let node_id = node_handle.get_inner();
            let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
            let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);
            let node_ctx = NodeContext::with_lazy_attributes(
                NodeType::Button,
                Cow::Borrowed("button"),
                tag,
                depth,
                index_in_parent,
                parent_tag.map(Cow::Borrowed),
                true,
            );
            let visit_result = {
                let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
                visitor.visit_button(&node_ctx, &text)
            };
            match visit_result {
                VisitResult::Continue => {}
                VisitResult::Skip => return true,
                VisitResult::Custom(custom) => {
                    output.push_str(&custom);
                    return true;
                }
                VisitResult::PreserveHtml => {
                    use crate::converter::utility::serialization::serialize_node;
                    output.push_str(&serialize_node(node_handle, parser));
                    return true;
                }
                VisitResult::Error(err) => {
                    if ctx.visitor_error.borrow().is_none() {
                        *ctx.visitor_error.borrow_mut() = Some(err);
                    }
                    return true;
                }
            }
        }
    }
    false
}

/// Dispatcher for form elements.
///
/// Routes all form-related elements to their respective handlers.
pub fn handle(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    match tag_name {
        "form" => handle_form(tag_name, node_handle, parser, output, context),
        "fieldset" => handle_fieldset(tag_name, node_handle, parser, output, context),
        "legend" => handle_legend(tag_name, node_handle, parser, output, context),
        "label" => handle_label(tag_name, node_handle, parser, output, context),
        "input" => handle_input(tag_name, node_handle, parser, output, context),
        "select" | "datalist" => handle_option_list(tag_name, node_handle, parser, output, context),
        "option" if !context.ctx.in_option_list => {
            handle_option_list(tag_name, node_handle, parser, output, context);
        }
        "option" => handle_option(tag_name, node_handle, parser, output, context),
        _ => {}
    }
}

pub struct ButtonEnd {
    start: ControlStart,
    content_start: usize,
    node: tl::NodeHandle,
}

pub fn open_button(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: None,
        close: Close::Button(ButtonEnd {
            start: ControlStart::new(output),
            content_start: output.len(),
            node: *node,
        }),
    })
}

pub fn close_button(
    end: &ButtonEnd,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
) {
    if output
        .get(..end.content_start)
        .is_some_and(|before| before.trim().is_empty())
    {
        handler.ctx.button_wrote.set(
            output
                .get(end.content_start..)
                .is_some_and(|content| !content.trim().is_empty()),
        );
    }
    if !handler.ctx.in_code && end.start.finish(output) {
        separate_from_next_text(output, end.node, parser, handler.dom_ctx);
    }
}

pub struct CodeGroupEnd {
    original_len: usize,
    content_start: usize,
}

pub fn open_fieldset(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    if handler.ctx.in_code && !handler.ctx.in_code_block {
        return open_code_group(node, parser, output, handler, children);
    }
    crate::converter::block::div::open(node, parser, output, handler, children)
}

fn open_code_group(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    let original_len = output.len();
    if !output.is_empty() && !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
    let end = CodeGroupEnd {
        original_len,
        content_start: output.len(),
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: Some(std::rc::Rc::new(super::Context {
            convert_as_inline: true,
            ..handler.ctx.clone()
        })),
        close: Close::CodeGroup(end),
    })
}

pub fn close_code_group(end: &CodeGroupEnd, output: &mut String) {
    if output.len() == end.content_start {
        output.truncate(end.original_len);
    } else if !output.ends_with(char::is_whitespace) {
        output.push(' ');
    }
}

pub fn open_label(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &str,
    context: FormContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    if context.ctx.in_code_block {
        return Some(Open::children_only());
    }
    let first = first_content(tag, parser);
    Some(Open {
        scope: None,
        close: Close::Hooked(Box::new(LabelHook {
            content: String::new(),
            ctx: std::rc::Rc::new(context.ctx.inline_buffer(output, false)),
            handles: children.clone(),
            first,
            first_button_wrote: false,
        })),
    })
}

struct LabelHook {
    content: String,
    ctx: std::rc::Rc<super::Context>,
    handles: Vec<tl::NodeHandle>,
    first: Option<tl::NodeHandle>,
    first_button_wrote: bool,
}

impl crate::converter::walk::ChildHook for LabelHook {
    fn before_child(
        &mut self,
        _index: usize,
        site: &mut crate::converter::walk::HookSite<'_>,
    ) -> crate::converter::walk::Child {
        site.context.button_wrote.set(false);
        crate::converter::walk::Child::Walk {
            depth: None,
            buffer: Some(std::mem::take(&mut self.content)),
            scope: Some(std::rc::Rc::clone(&self.ctx)),
        }
    }
    fn after_child(&mut self, index: usize, own: Option<String>, site: &mut crate::converter::walk::HookSite<'_>) {
        self.content = own.unwrap_or_default();
        if self.first == Some(self.handles[index]) {
            self.first_button_wrote = site.context.button_wrote.get();
        }
    }
    fn close(self: Box<Self>, site: &mut crate::converter::walk::HookSite<'_>) {
        let trimmed = self.content.trim();
        if trimmed.is_empty() {
            return;
        }
        let start = ControlStart::new(site.output);
        site.output.push_str(trimmed);
        if !site.context.in_code
            && starts_with_control(self.first, site.parser, site.dom_context, self.first_button_wrote)
        {
            start.finish(site.output);
        }
        if self.content.ends_with([' ', '\t']) {
            site.output.push(' ');
        }
    }
}

pub fn open_legend(
    node: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
    children: &mut Vec<tl::NodeHandle>,
) -> Option<Open> {
    if handler.ctx.in_code && !handler.ctx.in_code_block {
        return open_code_group(node, parser, output, handler, children);
    }
    let Some(tl::Node::Tag(tag)) = node.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    let mut child_ctx = handler.ctx.inline_buffer(output, !handler.ctx.convert_as_inline);
    if !handler.ctx.convert_as_inline {
        child_ctx.in_strong = true;
    }
    Some(Open {
        scope: Some(std::rc::Rc::new(child_ctx)),
        close: Close::Capture(crate::converter::walk::CaptureEnd::Legend),
    })
}

pub fn close_legend(
    rendered: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    handler: crate::converter::block::container::HandlerContext<'_>,
) {
    let options = handler.options;
    let ctx = handler.ctx;
    let dom_ctx = handler.dom_ctx;
    let trimmed = rendered.trim();
    if !trimmed.is_empty() {
        if ctx.convert_as_inline {
            output.push_str(trimmed);
        } else {
            let mut bold = String::with_capacity(trimmed.len() + 4);
            crate::converter::inline::emphasis::emit_strong_wrapped_blocks(
                &mut bold,
                trimmed,
                crate::converter::inline::wrapped::InlineSite {
                    node_handle,
                    parser,
                    dom_ctx,
                    ctx,
                    options,
                },
            );
            crate::converter::block::div::push_block(output, options, ctx, &bold);
        }
    }
}
