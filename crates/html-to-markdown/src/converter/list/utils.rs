//! Utility functions for list processing.
//!
//! Contains helper functions for loose list detection, indentation calculation,
//! list spacing, and list child processing.

use crate::options::{ConversionOptions, ListIndentType};
use tl;

// Type aliases for Context and DomContext to avoid circular imports
// These are imported from converter.rs and should be made accessible
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Remove trailing spaces and tabs from output string.
fn trim_trailing_whitespace(output: &mut String) {
    while output.ends_with(' ') || output.ends_with('\t') {
        output.pop();
    }
}

/// Check if tag names are equal (case-insensitive).
fn tag_name_eq<'a>(a: impl AsRef<str>, b: &str) -> bool {
    a.as_ref().eq_ignore_ascii_case(b)
}

/// Calculate indentation level for list item continuations.
///
/// Returns the number of 4-space indent groups needed for list continuations.
///
/// List continuations (block elements inside list items) need special indentation:
/// - Base indentation: (depth - 1) groups (for the nesting level)
/// - Content indentation: depth groups (for the list item content)
/// - Combined formula: (2 * depth - 1) groups of 4 spaces each
///
/// # Examples
///
/// ```text
/// * Item 1           (depth=0, no continuation)
/// * Item 2           (depth=0)
///     Continuation   (depth=0: 0 groups = 0 spaces)
///
/// * Level 1          (depth=0)
///     + Level 2      (depth=1)
///             Cont   (depth=1: (2*1-1) = 1 group = 4 spaces, total 12 with bullet indent)
/// ```
pub const fn calculate_list_continuation_indent(depth: usize) -> usize {
    if depth > 0 { 2 * depth - 1 } else { 0 }
}

/// Check if a list (ul or ol) is "loose".
///
/// A loose list is one where any list item contains block-level elements
/// like paragraphs (<p>). In loose lists, all items should have blank line
/// separation (ending with \n\n) regardless of their own content.
///
/// # Examples
///
/// ```html
/// <!-- Loose list (has <p> in an item) -->
/// <ul>
///   <li><p>Item 1</p></li>
///   <li>Item 2</li>  <!-- Also gets \n\n ending -->
/// </ul>
///
/// <!-- Tight list (no block elements) -->
/// <ul>
///   <li>Item 1</li>
///   <li>Item 2</li>
/// </ul>
/// ```
pub fn is_loose_list(node_handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                let is_li = dom_ctx.tag_info(child_handle.get_inner(), parser).map_or_else(
                    || {
                        matches!(
                            child_handle.get(parser),
                            Some(tl::Node::Tag(child_tag))
                                if tag_name_eq(child_tag.name().as_utf8_str(), "li")
                        )
                    },
                    |info| info.name == "li",
                );
                if !is_li {
                    continue;
                }

                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let li_children = child_tag.children();
                    for li_child_handle in li_children.top().iter() {
                        let is_p = dom_ctx.tag_info(li_child_handle.get_inner(), parser).map_or_else(
                            || {
                                matches!(
                                    li_child_handle.get(parser),
                                    Some(tl::Node::Tag(li_child_tag))
                                        if tag_name_eq(li_child_tag.name().as_utf8_str(), "p")
                                )
                            },
                            |info| info.name == "p",
                        );
                        if is_p {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Add list continuation indentation to output.
///
/// Used when block elements (like <p> or <div>) appear inside list items.
/// Adds appropriate line separation and indentation to continue the list item.
///
/// # Arguments
///
/// * `output` - The output string to append to
/// * `list_depth` - Current list nesting depth
/// * `blank_line` - If true, adds blank line separation (\n\n); if false, single newline (\n)
///
/// # Examples
///
/// ```text
/// Paragraph continuation (blank_line = true):
///   * First para
///
///       Second para  (blank line + indentation)
///
/// Div continuation (blank_line = false):
///   * First div
///       Second div   (single newline + indentation)
/// ```
pub fn add_list_continuation_indent(
    output: &mut String,
    list_depth: usize,
    blank_line: bool,
    options: &ConversionOptions,
) {
    trim_trailing_whitespace(output);

    if blank_line {
        if !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
    } else if !output.ends_with('\n') {
        output.push('\n');
    }

    let indent_level = calculate_list_continuation_indent(list_depth);
    let indent_char = match options.list_indent_type {
        ListIndentType::Tabs => "\t",
        ListIndentType::Spaces => &" ".repeat(options.list_indent_width),
    };
    output.push_str(&indent_char.repeat(indent_level));
}

/// Calculate the indentation string for list continuations based on depth and options.
pub fn continuation_indent_string(list_depth: usize, options: &ConversionOptions) -> Option<String> {
    let indent_level = calculate_list_continuation_indent(list_depth);
    if indent_level == 0 {
        return None;
    }

    let indent = match options.list_indent_type {
        ListIndentType::Tabs => "\t".repeat(indent_level),
        ListIndentType::Spaces => " ".repeat(options.list_indent_width * indent_level),
    };
    Some(indent)
}

/// Add appropriate leading separator before a list.
///
/// Lists need different separators depending on context:
/// - In table cells: <br> tag if there's already content
/// - Outside lists: blank line (\n\n) if needed
/// - Inside list items: blank line before nested list
pub fn add_list_leading_separator(output: &mut String, ctx: &Context) {
    if ctx.in_table_cell {
        let is_table_continuation =
            !output.is_empty() && !output.ends_with('|') && !output.ends_with(' ') && !output.ends_with("<br>");
        if is_table_continuation {
            output.push_str("<br>");
        }
        return;
    }

    if !output.is_empty() && !ctx.in_list {
        let needs_newline =
            !output.ends_with("\n\n") && !output.ends_with("* ") && !output.ends_with("- ") && !output.ends_with(". ");
        if needs_newline {
            output.push_str("\n\n");
        }
        return;
    }

    if ctx.in_list_item && !output.is_empty() {
        let needs_newline =
            !output.ends_with('\n') && !output.ends_with("* ") && !output.ends_with("- ") && !output.ends_with(". ");
        if needs_newline {
            trim_trailing_whitespace(output);
            output.push('\n');
        }
    }
}

/// Add appropriate trailing separator after a nested list.
///
/// Nested lists inside list items need trailing newlines to separate
/// from following content. In loose lists, use blank line (\n\n). In tight lists, single newline (\n).
pub fn add_nested_list_trailing_separator(output: &mut String, ctx: &Context) {
    if !ctx.in_list_item {
        return;
    }

    if ctx.loose_list {
        if !output.ends_with("\n\n") {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push('\n');
        }
    } else if !output.ends_with('\n') {
        output.push('\n');
    }
}

/// Calculate the nesting depth for a list.
///
/// If we're in a list but NOT in a list item, this is incorrectly nested HTML
/// and we need to increment the depth. If in a list item, the depth was already
/// incremented by the <li> element.
pub const fn calculate_list_nesting_depth(ctx: &Context) -> usize {
    if ctx.in_list && !ctx.in_list_item {
        ctx.list_depth + 1
    } else {
        ctx.list_depth
    }
}

/// Check if a node is a list item element.
pub fn is_list_item(node_handle: tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    if let Some(info) = dom_ctx.tag_info(node_handle.get_inner(), parser) {
        return info.name == "li";
    }
    matches!(
        node_handle.get(parser),
        Some(tl::Node::Tag(tag)) if tag_name_eq(tag.name().as_utf8_str(), "li")
    )
}

/// Process a list's children, tracking which items had block elements.
///
/// This is used to determine proper spacing between list items.
/// Returns true if the last processed item had block children.
#[allow(clippy::too_many_arguments)]
pub fn process_list_children(
    node_handle: tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    is_ordered: bool,
    is_loose: bool,
    nested_depth: usize,
    start_counter: usize,
    dom_ctx: &DomContext,
) {
    let mut counter = start_counter;

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                if let Some(tl::Node::Raw(bytes)) = child_handle.get(parser) {
                    if bytes.as_utf8_str().trim().is_empty() {
                        continue;
                    }
                }

                let list_ctx = Context {
                    in_ordered_list: is_ordered,
                    list_counter: if is_ordered { counter } else { 0 },
                    in_list: true,
                    list_depth: nested_depth,
                    ul_depth: if is_ordered { ctx.ul_depth } else { ctx.ul_depth + 1 },
                    loose_list: is_loose,
                    prev_item_had_blocks: false,
                    ..ctx.clone()
                };

                use crate::converter::walk_node;
                walk_node(child_handle, parser, output, options, &list_ctx, depth, dom_ctx);

                if is_ordered && is_list_item(*child_handle, parser, dom_ctx) {
                    counter += 1;
                }
            }
        }
    }
}
