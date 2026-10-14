//! Handler for structural container elements.
//!
//! This module provides handlers for structural containers that process their
//! children without special formatting or whitespace truncation:
//! - body, html: Structural document containers
//! - time, data: Inline semantic containers
//! - thead, tbody, tfoot, tr, th, td: Table structure (handled elsewhere)
//! - source: Media source element
//! - wbr: Word break opportunity (no-op)

use crate::converter::walk::Open;
use crate::options::ConversionOptions;
use tl::{NodeHandle, Parser};

type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

#[derive(Clone, Copy)]
pub struct HandlerContext<'a> {
    pub options: &'a ConversionOptions,
    pub ctx: &'a Context,
    pub depth: usize,
    pub dom_ctx: &'a DomContext,
}

impl<'a> HandlerContext<'a> {
    pub const fn new(options: &'a ConversionOptions, ctx: &'a Context, depth: usize, dom_ctx: &'a DomContext) -> Self {
        Self {
            options,
            ctx,
            depth,
            dom_ctx,
        }
    }
}

/// Open a container that writes nothing of its own: `body` and `html`, `time` and `data`, `small`,
/// a stray table part. The walk visits every child with the state of the container.
pub fn open_children(node_handle: &NodeHandle, parser: &Parser, children: &mut Vec<NodeHandle>) -> Option<Open> {
    let Some(tl::Node::Tag(tag)) = node_handle.get(parser) else {
        return None;
    };
    children.extend(tag.children().top().iter().copied());
    Some(Open::children_only())
}

/// Write the children of a container that writes nothing of its own, for a handler that the walk
/// does not drive.
pub fn handle_passthrough(node_handle: &NodeHandle, parser: &Parser, output: &mut String, handler: HandlerContext<'_>) {
    crate::converter::main::walk_opened(parser, output, handler, |_, children| {
        open_children(node_handle, parser, children)
    });
}

/// Handle no-op container elements that should be ignored.
///
/// This is used for elements like `wbr` (word break opportunity) and `source`
/// (media source specification) that should not produce any output.
///
/// # Arguments
/// * `_node_handle` - Handle to the HTML node (unused)
/// * `_parser` - The HTML parser (unused)
/// * `_output` - Accumulation buffer for Markdown output (unused)
/// * `_options` - Conversion options (unused)
/// * `_ctx` - Current conversion context (unused)
/// * `_depth` - Current recursion depth (unused)
/// * `_dom_ctx` - DOM context (unused)
#[inline]
pub const fn handle_noop() {}
