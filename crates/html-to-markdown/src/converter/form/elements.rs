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
use super::spacing::{ControlStart, line_goes_on_after_parent, separate_from_next_text};
use super::walk_node;
#[cfg(feature = "visitor")]
use std::borrow::Cow;

/// Run `<form>`'s visitor hook, if one is installed.
///
/// Returns `true` when the visitor already produced (or explicitly skipped) the output for
/// this node and `handle_form` must return without falling through to its default rendering;
/// `false` when there is no visitor or it returned `VisitResult::Continue`.
///
/// Extracted from `handle_form` — identical visitor dispatch and `VisitResult` handling,
/// unchanged.
#[cfg(feature = "visitor")]
fn dispatch_form_visitor(
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
        let mut rendered = String::new();
        let children = tag.children();
        let label_ctx = ctx.inline_buffer(output, false);
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut rendered,
                    crate::converter::block::container::HandlerContext::new(options, &label_ctx, depth + 1, dom_ctx),
                );
            }
        }

        let trimmed = rendered.trim();
        if !trimmed.is_empty() {
            // ~keep In the label's own buffer a control at its start has no text before it.
            let start = ControlStart::new(output);
            output.push_str(trimmed);
            // ~keep In code the text of a control is written as the source has it.
            if !ctx.in_code && starts_with_control(tag, parser, dom_ctx) {
                start.finish(output);
            }
            if rendered.ends_with([' ', '\t']) {
                output.push(' ');
            }
        }
    }
}

/// Whether the first content of `tag` separates the label from the text before it: a control
/// that writes text of its own, or an input with white space after it.
fn starts_with_control(tag: &tl::HTMLTag, parser: &tl::Parser, dom_ctx: &crate::converter::DomContext) -> bool {
    let first = tag.children().top().iter().find_map(|child| match child.get(parser)? {
        tl::Node::Raw(raw) if raw.as_utf8_str().trim().is_empty() => None,
        tl::Node::Comment(_) => None,
        node => Some((*child, node)),
    });
    let Some((handle, tl::Node::Tag(control))) = first else {
        return false;
    };
    match crate::converter::utility::content::normalized_tag_name(control.name().as_utf8_str()).as_ref() {
        "input" => super::spacing::white_space_follows(handle.get_inner(), parser, dom_ctx),
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

/// Writes the children of a control that ends its line: `<textarea>`, `<button>`, `<output>`,
/// `<meter>` and `<progress>`.
///
/// The text is a word of its own after the text before it. A blank line follows it where a
/// block holds the control. In inline mode and in an inline element the line goes on, and the
/// text after the control is a word of its own too.
fn write_line_end_control(
    node_handle: tl::NodeHandle,
    tag: &tl::HTMLTag,
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
    let start = ControlStart::new(output);
    for child_handle in tag.children().top().iter() {
        walk_node(
            child_handle,
            parser,
            output,
            crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
        );
    }
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

/// Handles the `<textarea>`, `<output>`, `<meter>` and `<progress>` elements: their text content
/// is plain text that ends its line.
pub fn handle_line_end_control(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        write_line_end_control(*node_handle, tag, parser, output, context);
    }
}

/// Handles the `<select>` and `<datalist>` elements.
///
/// The options are one control in the line: one space separates each option from the option
/// before it, and the control from the text before and after it.
pub fn handle_option_list(
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
        let mut rendered = String::new();
        let list_ctx = ctx.inline_buffer(output, false);
        for child_handle in tag.children().top().iter() {
            walk_node(
                child_handle,
                parser,
                &mut rendered,
                crate::converter::block::container::HandlerContext::new(options, &list_ctx, depth + 1, dom_ctx),
            );
        }
        let trimmed = rendered.trim();
        if trimmed.is_empty() {
            return;
        }
        // ~keep In code the white space around the options is text of the source.
        let trimmed = if ctx.in_code { rendered.as_str() } else { trimmed };
        let start = ControlStart::new(output);
        output.push_str(trimmed);
        start.finish(output);
        separate_from_next_text(output, *node_handle, parser, dom_ctx);
    }
}

/// Handles the `<option>` element.
///
/// An option is inline text. Selection state does not change the visible label.
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

        let trimmed = text.trim();
        if !trimmed.is_empty() {
            // ~keep One space separates an option from what its list already holds.
            if !output.is_empty() && !output.ends_with(char::is_whitespace) {
                output.push(' ');
            }
            output.push_str(trimmed);
        }
    }
}

/// Handles the `<optgroup>` element.
///
/// Its options are written as the options of the list are. The `label` attribute is not text of
/// the page: a browser shows it only inside the open list.
pub fn handle_optgroup(
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
        {
            for child_handle in children.top().iter() {
                walk_node(
                    child_handle,
                    parser,
                    output,
                    crate::converter::block::container::HandlerContext::new(options, ctx, depth + 1, dom_ctx),
                );
            }
        }
    }
}

/// Handles the `<button>` element.
///
/// A button element represents a clickable button. Its text content is plain text that ends
/// its line.
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_button(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    context: FormContext<'_>,
) {
    let FormContext {
        ctx, depth, dom_ctx, ..
    } = context;
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        #[cfg(feature = "visitor")]
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
                VisitResult::Skip => return,
                VisitResult::Custom(custom) => {
                    output.push_str(&custom);
                    if !ctx.convert_as_inline && !custom.ends_with('\n') {
                        output.push_str("\n\n");
                    }
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

        write_line_end_control(*node_handle, tag, parser, output, context);
    }
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
        "textarea" | "output" | "meter" | "progress" => {
            handle_line_end_control(tag_name, node_handle, parser, output, context);
        }
        "select" | "datalist" => handle_option_list(tag_name, node_handle, parser, output, context),
        "option" => handle_option(tag_name, node_handle, parser, output, context),
        "optgroup" => handle_optgroup(tag_name, node_handle, parser, output, context),
        "button" => handle_button(tag_name, node_handle, parser, output, context),
        _ => {}
    }
}
