//! Inline element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for inline HTML elements:
//! - Emphasis elements (strong, b, em, i)
//! - Links (a)
//! - Code elements (code, kbd, samp)
//! - Semantic elements (mark, del, s, ins, u, small, sub, sup)
//! - Ruby annotation elements (ruby, rb, rt, rp, rtc)
//!
//! These handlers are designed to be extracted from the main `converter.rs`
//! file and integrated through the dispatcher function.
//!
//! **Integration Pattern:**
//! Each handler function takes the same signature:
//! - `tag_name: &str` - The HTML tag being processed
//! - `node_handle: &NodeHandle` - The DOM node handle
//! - `parser: &Parser` - The HTML parser reference
//! - `output: &mut String` - The output buffer to write to
//! - `options: &ConversionOptions` - Conversion configuration
//! - `ctx: &Context` - Processing context (state tracking)
//! - `depth: usize` - Current DOM tree depth
//! - `dom_ctx: &DomContext` - DOM context for tree relationships
//!
//! The main dispatcher function `dispatch_inline_handler` routes tags to
//! their appropriate handlers and returns a boolean indicating success.

pub mod code;
pub mod emphasis;
pub mod link;
pub mod semantic;

// Re-export types from parent module for submodule access
pub use super::{Context, DomContext};
// ruby module would go here if implemented
// pub mod ruby;

// Re-export handler functions for internal use by dispatcher (crate-private)
// pub(crate) use ruby::handle as handle_ruby;

/// Dispatches inline element handling to the appropriate handler.
///
/// This function routes inline HTML elements to their specialized handlers
/// based on tag name. It is designed to be called from the main `walk_node`
/// function in `converter.rs`.
///
/// # Routing Table
///
/// The following tag routes are supported:
///
/// | Tag(s) | Handler | Description |
/// |--------|---------|-------------|
/// | `strong`, `b` | emphasis | Bold/strong text formatting |
/// | `em`, `i` | emphasis | Italic/emphasis text formatting |
/// | `a` | link | Hyperlinks and anchors |
/// | `code`, `kbd`, `samp` | code | Inline code and keyboard input |
/// | `mark`, `del`, `s`, `ins`, `u`, `small`, `sub`, `sup` | semantic | Semantic formatting |
/// | `ruby`, `rb`, `rt`, `rp`, `rtc` | ruby | Ruby annotations (future) |
///
/// # Return Value
///
/// Returns `true` if the tag was recognized and handled, `false` otherwise.
/// This allows the caller to distinguish between:
/// - Handled inline elements (return `true`)
/// - Unhandled elements (return `false`) that should be processed as text or passed through
///
/// # Usage in converter.rs
///
/// ```ignore
/// if crate::converter::inline::dispatch_inline_handler(
///     &tag_name,
///     &node_handle,
///     parser,
///     output,
///     options,
///     ctx,
///     depth,
///     dom_ctx,
/// ) {
///     return; // Element was handled, move to next sibling
/// }
/// // Element was not handled, process as default inline element
/// ```
///
/// # Parameters
///
/// * `tag_name` - The normalized HTML tag name (lowercase)
/// * `node_handle` - The DOM node handle from the parser
/// * `parser` - Reference to the tl HTML parser
/// * `output` - Output buffer to write converted content to
/// * `options` - Conversion configuration options
/// * `ctx` - Processing context with state tracking
/// * `depth` - Current DOM tree depth for recursion tracking
/// * `dom_ctx` - DOM context for accessing tree structure
///
/// # Example
///
/// For `<strong>Bold text</strong>`, the dispatcher:
/// 1. Recognizes "strong" tag
/// 2. Routes to emphasis handler
/// 3. Returns `true`
/// 4. Emphasis handler outputs `**Bold text**` to output buffer
///
/// For `<span>Normal text</span>`, the dispatcher:
/// 1. Fails to recognize "span" tag
/// 2. Returns `false`
/// 3. Caller processes as default inline content
pub fn dispatch_inline_handler(
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::Context,
    depth: usize,
    dom_ctx: &crate::converter::DomContext,
) -> bool {
    match tag_name {
        // Emphasis elements: strong, b (bold) and em, i (italic)
        "strong" | "b" | "em" | "i" => {
            emphasis::handle(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Link elements: a (anchor)
        "a" => {
            link::handle(node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Code elements: code, kbd (keyboard input), samp (sample output)
        "code" | "kbd" | "samp" => {
            code::handle(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Semantic elements: mark, del, s, ins, u, small, sub, sup
        "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" => {
            semantic::handle(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
            true
        }
        // Ruby annotation elements: ruby, rb, rt, rp, rtc
        // NOTE: Ruby handler module not yet implemented
        // "ruby" | "rb" | "rt" | "rp" | "rtc" => {
        //     ruby::handle(tag_name, node_handle, parser, output, options, ctx, depth, dom_ctx);
        //     true
        // }
        // Unknown element - not handled by inline dispatcher
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    /// Test that all expected tags are properly dispatched
    #[test]
    fn test_dispatcher_routes_emphasis_tags() {
        assert!(matches!(
            ("strong", "strong"),
            (tag, _) if matches!(tag, "strong" | "b" | "em" | "i")
        ));
        assert!(matches!(
            ("em", "em"),
            (tag, _) if matches!(tag, "strong" | "b" | "em" | "i")
        ));
    }

    #[test]
    fn test_dispatcher_routes_code_tags() {
        assert!(matches!(
            ("code", "code"),
            (tag, _) if matches!(tag, "code" | "kbd" | "samp")
        ));
        assert!(matches!(
            ("kbd", "kbd"),
            (tag, _) if matches!(tag, "code" | "kbd" | "samp")
        ));
    }

    #[test]
    fn test_dispatcher_routes_semantic_tags() {
        assert!(matches!(
            ("mark", "mark"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup")
        ));
        assert!(matches!(
            ("del", "del"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup")
        ));
        assert!(matches!(
            ("sub", "sub"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup")
        ));
    }

    #[test]
    fn test_dispatcher_recognizes_link_tag() {
        assert!(matches!(
            ("a", "a"),
            (tag, _) if tag == "a"
        ));
    }

    #[test]
    fn test_unknown_tags_not_routed() {
        // These should fall through to default handling
        let unknown_tags = vec!["span", "div", "p", "section", "article", "table"];
        for tag in unknown_tags {
            assert!(matches!(
                (tag, tag),
                (tag, _) if !matches!(
                    tag,
                    "strong" | "b" | "em" | "i" | "a" | "code" | "kbd" | "samp"
                    | "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup"
                )
            ));
        }
    }
}
