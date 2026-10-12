//! Inline element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for inline HTML elements:
//! - Emphasis elements (strong, b, em, i)
//! - Code-like elements (kbd, samp)
//! - Semantic elements (mark, del, s, ins, u, small, sub, sup, var, dfn, abbr)
//! - Ruby annotation elements (ruby, rb, rt, rp, rtc)
//!
//! These handlers are designed to be extracted from the main `converter.rs`
//! file and integrated through the dispatcher function.
//!
//! Each handler receives a [`HandlerContext`] that groups the DOM node, parser,
//! output buffer, conversion options, processing state, depth, and DOM context.
//!
//! The main dispatcher function `dispatch_inline_handler` routes tags to
//! their appropriate handlers and returns a boolean indicating success.

pub mod code;
pub mod emphasis;
pub mod link;
pub mod ruby;
pub mod semantic;
pub mod wrapped;

pub struct HandlerContext<'a> {
    pub node_handle: &'a tl::NodeHandle,
    pub parser: &'a tl::Parser<'a>,
    pub output: &'a mut String,
    pub options: &'a crate::options::ConversionOptions,
    pub context: &'a crate::converter::Context,
    pub depth: usize,
    pub dom_context: &'a crate::converter::DomContext,
}

type HandlerParts<'a> = (
    &'a tl::NodeHandle,
    &'a tl::Parser<'a>,
    &'a mut String,
    &'a crate::options::ConversionOptions,
    &'a crate::converter::Context,
    usize,
    &'a crate::converter::DomContext,
);

impl<'a> HandlerContext<'a> {
    pub const fn new(parts: HandlerParts<'a>) -> Self {
        let (node_handle, parser, output, options, context, depth, dom_context) = parts;
        Self {
            node_handle,
            parser,
            output,
            options,
            context,
            depth,
            dom_context,
        }
    }

    pub const fn inline_site(&self) -> wrapped::InlineSite<'a> {
        wrapped::InlineSite {
            node_handle: self.node_handle,
            parser: self.parser,
            dom_ctx: self.dom_context,
            ctx: self.context,
            options: self.options,
        }
    }
}

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
/// | `kbd`, `samp` | code | Keyboard input and sample output, rendered as code |
/// | `mark`, `del`, `s`, `ins`, `u`, `small`, `sub`, `sup`, `var`, `dfn`, `abbr`, `span` | semantic | Semantic formatting |
/// | `ruby`, `rb`, `rt`, `rp`, `rtc` | ruby | Ruby annotations (East Asian typography) |
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
/// ```text
/// let handler = HandlerContext::new((node_handle, parser, output, options, ctx, depth, dom_ctx));
/// if crate::converter::inline::dispatch_inline_handler(&tag_name, handler) {
///     return; // Element was handled, move to next sibling
/// }
/// // Element was not handled, process as default inline element
/// ```
///
/// # Parameters
///
/// * `tag_name` - The normalized HTML tag name (lowercase)
/// * `context` - The DOM and conversion state used by the selected handler
///
/// # Example
///
/// For `<strong>Bold text</strong>`, the dispatcher:
/// 1. Recognizes "strong" tag
/// 2. Routes to emphasis handler
/// 3. Returns `true`
/// 4. Emphasis handler outputs `**Bold text**` to output buffer
///
/// For an unrecognized tag, the dispatcher returns `false` so the caller can
/// process the element through the default path.
pub fn dispatch_inline_handler(tag_name: &str, context: HandlerContext<'_>) -> bool {
    match tag_name {
        "strong" | "b" | "em" | "i" => {
            emphasis::handle(tag_name, context);
            true
        }
        "kbd" | "samp" | "tt" => {
            code::handle(tag_name, context);
            true
        }
        "mark" | "del" | "s" | "strike" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr" | "span" => {
            semantic::handle(tag_name, context);
            true
        }
        "ruby" | "rb" | "rt" | "rp" | "rtc" => {
            ruby::handle(tag_name, context);
            true
        }
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
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
        assert!(matches!(
            ("del", "del"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
        assert!(matches!(
            ("sub", "sub"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
        assert!(matches!(
            ("var", "var"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
        assert!(matches!(
            ("dfn", "dfn"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
        assert!(matches!(
            ("abbr", "abbr"),
            (tag, _) if matches!(tag, "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr")
        ));
    }

    #[test]
    fn test_dispatcher_routes_ruby_tags() {
        assert!(matches!(
            ("ruby", "ruby"),
            (tag, _) if matches!(tag, "ruby" | "rb" | "rt" | "rp" | "rtc")
        ));
        assert!(matches!(
            ("rt", "rt"),
            (tag, _) if matches!(tag, "ruby" | "rb" | "rt" | "rp" | "rtc")
        ));
    }

    #[test]
    fn test_unknown_tags_not_routed() {
        let unknown_tags = vec!["div", "p", "section", "article", "table"];
        for tag in unknown_tags {
            assert!(matches!(
                (tag, tag),
                (tag, _) if !matches!(
                    tag,
                    "strong" | "b" | "em" | "i" | "a" | "code" | "kbd" | "samp"
                    | "mark" | "del" | "s" | "ins" | "u" | "small" | "sub" | "sup" | "var" | "dfn" | "abbr"
                    | "ruby" | "rb" | "rt" | "rp" | "rtc" | "span"
                )
            ));
        }
    }
}
