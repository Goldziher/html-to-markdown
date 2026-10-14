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
use super::walk_node;
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
/// A label element associates text with a form control. It's rendered as
/// inline content.
///
/// # Behavior
///
/// - Content is collected from children
/// - Non-empty content is output without adding block separators
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
            output.push_str(trimmed);
        }
    }
}

fn emit_checkbox_input(node_handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return;
    };
    let is_checkbox = tag
        .attributes()
        .get("type")
        .flatten()
        .is_some_and(|value| value.as_utf8_str().eq_ignore_ascii_case("checkbox"));
    if is_checkbox {
        output.push_str(if tag.attributes().get("checked").is_some() {
            "[x]"
        } else {
            "[ ]"
        });
    }
}

/// Handles the `<input>` element.
///
/// Checkbox inputs render their visible checked state; other inputs have no text output. ~keep
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
#[allow(clippy::ptr_arg, clippy::needless_pass_by_ref_mut, clippy::missing_const_for_fn)]
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

    emit_checkbox_input(node_handle, parser, output);
}

/// Handles the `<textarea>` element.
///
/// A textarea element represents a multi-line text input. Its content is
/// rendered as plain text, with blank lines added after in block mode.
///
/// # Behavior
///
/// - Content is collected from children
/// - Blank lines are added after content in block mode only
pub fn handle_textarea(
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
        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<select>` element.
///
/// A select element represents a dropdown list of options. Its options are
/// rendered as inline text.
///
/// # Behavior
///
/// - Content (options) is collected from children
/// - No block separator is added after the select
pub fn handle_select(
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

/// Handles the `<option>` element.
///
/// An option element represents a choice within a select element.
/// Options are rendered as inline text.
///
/// # Behavior
///
/// - Content is collected from children
/// - Selection state does not change the visible label
/// - No block separator is added after an option
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
            output.push_str(trimmed);
        }
    }
}

/// Handles the `<optgroup>` element.
///
/// An optgroup element groups options within a select element with an optional label.
/// The label is rendered as strong (bold) text, followed by the grouped options.
///
/// # Behavior
///
/// - The `label` attribute is output as strong text (if present)
/// - Options within the group are rendered normally
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
        let label = tag
            .attributes()
            .get("label")
            .flatten()
            .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

        if !label.is_empty() {
            let mut symbol = String::with_capacity(2);
            symbol.push(options.strong_em_symbol);
            symbol.push(options.strong_em_symbol);
            output.push_str(&symbol);
            output.push_str(&label);
            output.push_str(&symbol);
            output.push('\n');
        }

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
/// A button element represents a clickable button. Its text content is rendered
/// as plain text, with blank lines added in block mode.
///
/// # Behavior
///
/// - Content is collected from children
/// - Blank lines are added after content in block mode only
#[cfg_attr(not(feature = "visitor"), allow(unused_variables))]
pub fn handle_button(
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

        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<progress>` element.
///
/// A progress element represents a progress bar. It typically has no visible
/// text content, but we render any child content present.
///
/// # Behavior
///
/// - Content is collected from children (usually empty)
/// - Blank lines are added after content in block mode only
pub fn handle_progress(
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
        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<meter>` element.
///
/// A meter element represents a scalar measurement (e.g., disk usage, temperature).
/// It typically has no visible text content, but we render any child content.
///
/// # Behavior
///
/// - Content is collected from children (usually empty)
/// - Blank lines are added after content in block mode only
pub fn handle_meter(
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
        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<output>` element.
///
/// An output element represents the result of a calculation. It renders its
/// text content as plain output, with blank lines in block mode.
///
/// # Behavior
///
/// - Content is collected from children
/// - Blank lines are added after content in block mode only
pub fn handle_output(
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
        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push_str("\n\n");
        }
    }
}

/// Handles the `<datalist>` element.
///
/// A datalist element provides a list of predefined options for an input element.
/// Options are rendered as a list with newlines between them.
///
/// # Behavior
///
/// - Content (options) is collected from children
/// - A single newline is added after the datalist in block mode
pub fn handle_datalist(
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
        let start_len = output.len();
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

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push('\n');
        }
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
        "textarea" => handle_textarea(tag_name, node_handle, parser, output, context),
        "select" => handle_select(tag_name, node_handle, parser, output, context),
        "option" => handle_option(tag_name, node_handle, parser, output, context),
        "optgroup" => handle_optgroup(tag_name, node_handle, parser, output, context),
        "button" => handle_button(tag_name, node_handle, parser, output, context),
        "progress" => handle_progress(tag_name, node_handle, parser, output, context),
        "meter" => handle_meter(tag_name, node_handle, parser, output, context),
        "output" => handle_output(tag_name, node_handle, parser, output, context),
        "datalist" => handle_datalist(tag_name, node_handle, parser, output, context),
        _ => {}
    }
}
