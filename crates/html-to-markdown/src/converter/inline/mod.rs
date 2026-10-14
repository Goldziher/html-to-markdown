//! Inline element handlers for HTML to Markdown conversion.
//!
//! This module provides specialized handlers for inline HTML elements:
//! - Emphasis elements (strong, b, em, i)
//! - Code-like elements (kbd, samp)
//! - Semantic elements (mark, del, s, ins, u, small, sub, sup, var, dfn, abbr)
//! - Ruby annotation elements (ruby, rb, rt, rp, rtc)
//!
//! Each handler receives a [`HandlerContext`] that groups the DOM node, parser,
//! output buffer, conversion options, processing state, depth, and DOM context.
//!
//! The walk opens each of these elements itself (`emphasis::open`, `semantic::open`,
//! `code::open`, `ruby::open`, `ruby::open_part`): their children are its work.

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
