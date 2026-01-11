//! Definition list handling (dl, dt, dd elements).
//!
//! Processes definition lists with:
//! - Definition terms (dt)
//! - Definition descriptions (dd)
//! - Proper Markdown formatting with `:   ` separator

use crate::options::ConversionOptions;
use tl;

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle definition list element (<dl>).
///
/// Groups dt/dd pairs and formats them with proper Markdown separation.
pub(crate) fn handle_dl(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    if ctx.convert_as_inline {
        let tag = match node_handle.get(parser) {
            Some(tl::Node::Tag(t)) => t,
            _ => return,
        };

        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                use crate::converter::walk_node;
                walk_node(child_handle, parser, output, options, ctx, depth, dom_ctx);
            }
        }
        return;
    }

    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    let mut content = String::new();
    let mut in_dt_group = false;
    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            let (is_definition_term, is_definition_description) =
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let tag_name = {
                        use crate::converter::normalized_tag_name;
                        normalized_tag_name(child_tag.name().as_utf8_str())
                    };
                    (tag_name == "dt", tag_name == "dd")
                } else {
                    (false, false)
                };

            let child_ctx = Context {
                last_was_dt: in_dt_group && is_definition_description,
                ..ctx.clone()
            };
            crate::converter::walk_node(child_handle, parser, &mut content, options, &child_ctx, depth, dom_ctx);

            if is_definition_term {
                in_dt_group = true;
            } else if !is_definition_description {
                in_dt_group = false;
            }
        }
    }

    let trimmed = content.trim();
    if !trimmed.is_empty() {
        if !output.is_empty() && !output.ends_with("\n\n") {
            output.push_str("\n\n");
        }
        output.push_str(trimmed);
        output.push_str("\n\n");
    }
}

/// Handle definition term element (<dt>).
///
/// Outputs the term text followed by a newline.
pub(crate) fn handle_dt(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    let mut content = String::with_capacity(64);
    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            crate::converter::walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
        }
    }
    let trimmed = content.trim();
    if !trimmed.is_empty() {
        if ctx.convert_as_inline {
            output.push_str(trimmed);
        } else {
            output.push_str(trimmed);
            output.push('\n');
        }
    }
}

/// Handle definition description element (<dd>).
///
/// Outputs the description with `:   ` prefix if it follows a dt,
/// or on its own with proper spacing.
pub(crate) fn handle_dd(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    let tag = match node_handle.get(parser) {
        Some(tl::Node::Tag(t)) => t,
        _ => return,
    };

    let mut content = String::with_capacity(128);
    let children = tag.children();
    {
        for child_handle in children.top().iter() {
            crate::converter::walk_node(child_handle, parser, &mut content, options, ctx, depth + 1, dom_ctx);
        }
    }

    let trimmed = content.trim();

    if ctx.convert_as_inline {
        if !trimmed.is_empty() {
            output.push_str(trimmed);
        }
    } else if ctx.last_was_dt {
        if trimmed.is_empty() {
            output.push_str(":   \n\n");
        } else {
            output.push_str(":   ");
            output.push_str(trimmed);
            output.push_str("\n\n");
        }
    } else if !trimmed.is_empty() {
        output.push_str(trimmed);
        output.push_str("\n\n");
    }
}
