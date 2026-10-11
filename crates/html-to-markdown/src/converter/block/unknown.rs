//! Handler for unknown/unspecified HTML elements.
//!
//! Processes HTML elements that don't have specific handlers by passing through
//! their children while managing whitespace and content preservation:
//! - Gives all child nodes to the walk
//! - Validates UTF-8 character boundaries when checking content
//! - Preserves code blocks (indented or fenced) while removing empty content
//! - Manages trailing whitespace intelligently

#[cfg(feature = "visitor")]
use crate::converter::block::container::HandlerContext;
use crate::converter::walk::{Close, Open};
#[cfg(feature = "visitor")]
use std::borrow::Cow;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handles unknown/unspecified HTML elements.
///
/// For elements without specific handlers, this function:
/// 1. Records the output position before processing children
/// 2. Gives all children to the walk
/// 3. Checks if any content was added
/// 4. Validates UTF-8 character boundaries for safe string slicing
/// 5. Detects if added content is a code block (indented or fenced)
/// 6. Removes empty content while preserving code blocks and spacing
///
/// # UTF-8 Safety
/// The function carefully handles UTF-8 boundaries to prevent panic when
/// slicing the output string. It finds the nearest valid boundary if the
/// position falls in the middle of a multi-byte character.
///
/// # Code Block Detection
/// Code blocks (identified by markdown formatting) are always preserved,
/// even if they appear "empty" according to `trim()`.
///
/// This is the first half: it measures the output before the children and lists them in
/// `children`. With a visitor, `visit` runs before it and decides whether it runs at all.
pub fn open(node_handle: &NodeHandle, parser: &Parser, output: &str, children: &mut Vec<NodeHandle>) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };

    let len_before = output.len();
    let had_trailing_space = output.ends_with(' ');

    children.extend(tag.children().top().iter().copied());
    Some(Open {
        scope: None,
        close: Close::Unknown(UnknownEnd {
            len_before,
            had_trailing_space,
        }),
    })
}

/// What `close` needs from `open`.
pub struct UnknownEnd {
    len_before: usize,
    had_trailing_space: bool,
}

/// Remove what the children of an unknown element wrote when it is white space only.
pub fn close(end: &UnknownEnd, output: &mut String) {
    let UnknownEnd {
        len_before,
        had_trailing_space,
    } = *end;
    let len_after = output.len();
    if len_after > len_before {
        let start_idx = if output.is_char_boundary(len_before) {
            len_before
        } else {
            let capped = len_before.min(output.len());
            output
                .char_indices()
                .map(|(idx, _)| idx)
                .take_while(|idx| *idx <= capped)
                .last()
                .unwrap_or(capped)
        };

        let added_content = output[start_idx..].to_string();

        let is_code_block =
            added_content.starts_with("    ") || added_content.starts_with("```") || added_content.starts_with("~~~");

        if added_content.trim().is_empty() && !is_code_block {
            output.truncate(start_idx);
            if !had_trailing_space && added_content.contains(' ') {
                output.push(' ');
            }
        }
    }
}

/// Give an unknown element to the visitor. Returns whether the visitor wrote or skipped the
/// element, so that the walk does not open it.
#[cfg(feature = "visitor")]
pub fn visit(node_handle: &NodeHandle, parser: &Parser, output: &mut String, handler: HandlerContext<'_>) -> bool {
    use crate::converter::utility::serialization::serialize_tag_to_html;
    use crate::visitor::{NodeContext, NodeType, VisitResult};

    let Some(ref visitor_handle) = handler.ctx.visitor else {
        return false;
    };
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return false;
    };
    let tag_name = tag.name().as_utf8_str();
    let raw_html = serialize_tag_to_html(node_handle, parser);
    let node_id = node_handle.get_inner();
    let parent_tag = handler.dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = handler.dom_ctx.get_sibling_index(node_id).unwrap_or(0);
    let node_ctx = NodeContext::with_lazy_attributes(
        NodeType::Custom,
        Cow::Borrowed(tag_name.as_ref()),
        tag,
        handler.depth,
        index_in_parent,
        parent_tag.map(Cow::Borrowed),
        false,
    );
    let visit_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_custom_element(&node_ctx, &tag_name, &raw_html)
    };
    match visit_result {
        VisitResult::Continue => false,
        VisitResult::Skip => true,
        VisitResult::Custom(custom) => {
            output.push_str(&custom);
            true
        }
        VisitResult::PreserveHtml => {
            output.push_str(&raw_html);
            true
        }
        VisitResult::Error(err) => {
            if handler.ctx.visitor_error.borrow().is_none() {
                *handler.ctx.visitor_error.borrow_mut() = Some(err);
            }
            true
        }
    }
}
