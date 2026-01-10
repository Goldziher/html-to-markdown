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

// Note: Context and DomContext are defined in converter.rs
// walk_node is also defined there and must be called via the parent module
use super::walk_node;
use std::borrow::Cow;

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
pub fn handle_form(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        // In inline context, just process children inline
        if ctx.convert_as_inline {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    super::walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                }
            }
            return;
        }

        // Collect content
        let mut content = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            // Add spacing before if needed
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }

            // Output content
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        // In inline context, just process children inline
        if ctx.convert_as_inline {
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    super::walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
                }
            }
            return;
        }

        // Collect content
        let mut content = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, &mut content, options, ctx, depth, dom_ctx);
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            // Add spacing before if needed
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }

            // Output content
            output.push_str(trimmed);
            output.push_str("\n\n");
        }
    }
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
/// - Uses the configured strong/emphasis symbol from ConversionOptions
pub fn handle_legend(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::new();

        // Set strong context for nested content
        let mut legend_ctx = ctx.clone();
        if !ctx.convert_as_inline {
            legend_ctx.in_strong = true;
        }

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(
                    child_handle,
                    parser,
                    &mut content,
                    options,
                    &legend_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            if ctx.convert_as_inline {
                output.push_str(trimmed);
            } else {
                let symbol = options.strong_em_symbol.to_string().repeat(2);
                output.push_str(&symbol);
                output.push_str(trimmed);
                output.push_str(&symbol);
                output.push_str("\n\n");
            }
        }
    }
}

/// Handles the `<label>` element.
///
/// A label element associates text with a form control. It's rendered as
/// a block or inline element depending on context.
///
/// # Behavior
///
/// - Content is collected from children
/// - Non-empty content is output followed by blank lines (in block mode)
/// - Blank lines are suppressed in inline mode
pub fn handle_label(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut content = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
            }
        }

        let trimmed = content.trim();
        if !trimmed.is_empty() {
            output.push_str(trimmed);
            if !ctx.convert_as_inline {
                output.push_str("\n\n");
            }
        }
    }
}

/// Handles the `<input>` element.
///
/// An input element represents a form control for user input. Since input
/// elements typically have no text content, this handler produces no output.
pub fn handle_input(
    _tag_name: &str,
    _node_handle: &tl::NodeHandle,
    _parser: &tl::Parser,
    _output: &mut String,
    _options: &crate::options::ConversionOptions,
    _ctx: &super::Context,
    _depth: usize,
    _dom_ctx: &super::DomContext,
) {
    // Input elements have no text content; render nothing
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
/// rendered as a list, with newlines between options.
///
/// # Behavior
///
/// - Content (options) is collected from children
/// - A single newline is added after the select in block mode
pub fn handle_select(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
            }
        }

        if !ctx.convert_as_inline && output.len() > start_len {
            output.push('\n');
        }
    }
}

/// Handles the `<option>` element.
///
/// An option element represents a choice within a select element.
/// Selected options are marked with a bullet point (`*`) in block mode.
///
/// # Behavior
///
/// - Content is collected from children
/// - If the option has the `selected` attribute, it's prefixed with `* ` in block mode
/// - A newline is added after each option in block mode
pub fn handle_option(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let selected = tag.attributes().iter().any(|(name, _)| name.as_ref() == "selected");

        let mut text = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
            }
        }

        let trimmed = text.trim();
        if !trimmed.is_empty() {
            if selected && !ctx.convert_as_inline {
                output.push_str("* ");
            }
            output.push_str(trimmed);
            if !ctx.convert_as_inline {
                output.push('\n');
            }
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let label = tag
            .attributes()
            .get("label")
            .flatten()
            .map_or(Cow::Borrowed(""), |v| v.as_utf8_str());

        if !label.is_empty() {
            let symbol = options.strong_em_symbol.to_string().repeat(2);
            output.push_str(&symbol);
            output.push_str(&label);
            output.push_str(&symbol);
            output.push('\n');
        }

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
pub fn handle_button(
    _tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let start_len = output.len();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
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
    options: &crate::options::ConversionOptions,
    ctx: &super::Context,
    depth: usize,
    dom_ctx: &super::DomContext,
) {
    match tag_name {
        "form" => handle_form(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "fieldset" => handle_fieldset(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "legend" => handle_legend(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "label" => handle_label(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "input" => handle_input(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "textarea" => handle_textarea(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "select" => handle_select(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "option" => handle_option(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "optgroup" => handle_optgroup(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "button" => handle_button(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "progress" => handle_progress(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "meter" => handle_meter(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "output" => handle_output(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        "datalist" => handle_datalist(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx),
        _ => {}
    }
}
