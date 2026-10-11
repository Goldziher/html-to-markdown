//! Attribute handling and extraction utilities.
//!
//! Functions for working with element attributes, semantic detection, and hOCR document detection.

use std::borrow::Cow;

use crate::converter::DomContext;
use crate::converter::utility::content::normalized_tag_name;

/// Read an attribute whose value reaches the Markdown output, decoding character references.
///
/// HTML attribute values carry character references exactly as text nodes do, so
/// `title="A&amp;B"` means the three characters `A&B`. Every user-visible attribute must be
/// decoded before it is escaped for its Markdown context.
///
/// ~keep Issue #494: `<a href>` was the ONLY attribute site that decoded, so eleven others
/// ~keep emitted the raw entity as literal text. Route every such read through here rather
/// ~keep than calling `as_utf8_str()` directly, so a new attribute cannot reintroduce the gap.
/// ~keep The Markdown escaping downstream was always correct -- a literal `"` in a title was
/// ~keep already escaped -- it simply never received the decoded character.
///
/// ~keep Borrow-preserving: `decode_html_entities_cow` returns early on a value with no `&`,
/// ~keep so the common case still allocates nothing. That was the stated reason the original
/// ~keep `<a title>` read skipped decoding; the reason does not hold.
// ~keep `name` carries the tag's lifetime because `tl`'s `Attributes::get` is generic over
// ~keep `Into<Bytes<'a>>`; every call site passes a literal, so this costs nothing.
pub fn decoded_attribute<'a>(tag: &'a tl::HTMLTag<'a>, name: &'a str) -> Option<Cow<'a, str>> {
    let raw = tag.attributes().get(name).flatten()?.as_utf8_str();
    Some(match raw {
        Cow::Borrowed(borrowed) => crate::text::decode_attribute_value_cow(borrowed),
        Cow::Owned(owned) => Cow::Owned(crate::text::decode_attribute_value_cow(&owned).into_owned()),
    })
}

/// Whether the value of a `style` attribute sets `display` or `white-space`.
///
/// Such an element can be a box of its own (`display: inline-block`) or keep its line ends
/// (`white-space: pre`). Any other `style` value, the empty one too, changes neither.
///
/// ~keep This is no CSS parser: a declaration ends at each `;`, and its property name is the
/// ~keep text before the first `:`. A `;` in a quoted value starts a declaration here.
/// ~keep It reads the value as text: a character reference in `style` is no `:` and no letter
/// ~keep here. Ask [`style_attribute_sets_display_or_white_space`] about source bytes.
fn style_sets_display_or_white_space(style: &[u8]) -> bool {
    style.split(|&byte| byte == b';').any(|declaration| {
        declaration.iter().position(|&byte| byte == b':').is_some_and(|colon| {
            let name = declaration[..colon].trim_ascii();
            name.eq_ignore_ascii_case(b"display") || name.eq_ignore_ascii_case(b"white-space")
        })
    })
}

/// Whether a `style` attribute, given as the bytes of its value in the source, sets `display`
/// or `white-space`.
///
/// ~keep The source value carries character references (`white-space&#58;pre` is
/// ~keep `white-space:pre`), so the value is decoded first, as a browser does.
/// ~keep Both converters ask this one function: Tier-2 with the parser's attribute, Tier-1 with
/// ~keep the attribute its scanner collects. Neither of the two decodes a value.
/// ~keep A value without `&` has no reference and is read as it is, with no allocation.
pub fn style_attribute_sets_display_or_white_space(source_value: &[u8]) -> bool {
    if !source_value.contains(&b'&') {
        return style_sets_display_or_white_space(source_value);
    }
    let text = String::from_utf8_lossy(source_value);
    style_sets_display_or_white_space(crate::text::decode_attribute_value_cow(&text).as_bytes())
}

/// Check if a tag has main content semantics based on role or class.
pub fn tag_has_main_semantics(tag: &tl::HTMLTag) -> bool {
    if let Some(Some(role)) = tag.attributes().get("role") {
        let lowered = role.as_utf8_str().to_ascii_lowercase();
        if matches!(lowered.as_str(), "main" | "article" | "document" | "region") {
            return true;
        }
    }

    if let Some(Some(class_bytes)) = tag.attributes().get("class") {
        let class_value = class_bytes.as_utf8_str().to_ascii_lowercase();
        const MAIN_CLASS_HINTS: &[&str] = &[
            "mw-body",
            "mw-parser-output",
            "content-body",
            "content-container",
            "article-body",
            "article-content",
            "main-content",
            "page-content",
            "entry-content",
            "post-content",
            "document-body",
        ];
        if MAIN_CLASS_HINTS.iter().any(|hint| class_value.contains(hint)) {
            return true;
        }
    }

    false
}

/// Navigation-hint keywords matched against `class` and `id` token values.
///
/// Shared between [`element_has_navigation_hint`] (Tier-2 DOM path) and the
/// Tier-1 byte-scanner's preprocessing skip logic so both paths use the same
/// canonical keyword list.
pub const NAV_KEYWORDS: &[&str] = &[
    "nav",
    "navigation",
    "navbar",
    "breadcrumbs",
    "breadcrumb",
    "toc",
    "sidebar",
    "sidenav",
    "menu",
    "menubar",
    "mainmenu",
    "subnav",
    "tabs",
    "tablist",
    "toolbar",
    "pager",
    "pagination",
    "skipnav",
    "skip-link",
    "skiplinks",
    "site-nav",
    "site-menu",
    "site-header",
    "site-footer",
    "topbar",
    "bottombar",
    "masthead",
    "vector-nav",
    "vector-header",
    "vector-footer",
];

/// Check if an element has navigation-related hints in its attributes.
pub fn element_has_navigation_hint(tag: &tl::HTMLTag) -> bool {
    if attribute_matches_any(tag, "role", &["navigation", "menubar", "tablist", "toolbar"]) {
        return true;
    }

    if attribute_contains_any(
        tag,
        "aria-label",
        &["navigation", "menu", "contents", "table of contents", "toc"],
    ) {
        return true;
    }

    attribute_matches_any(tag, "class", NAV_KEYWORDS) || attribute_matches_any(tag, "id", NAV_KEYWORDS)
}

/// Check if an attribute value matches any of the given keywords (space or custom-separator aware).
pub fn attribute_matches_any(tag: &tl::HTMLTag, attr: &str, keywords: &[&str]) -> bool {
    let Some(attr_value) = tag.attributes().get(attr) else {
        return false;
    };
    let Some(value) = attr_value else {
        return false;
    };
    let raw = value.as_utf8_str();
    raw.split_whitespace()
        .map(|token| {
            token
                .chars()
                .map(|c| match c {
                    '_' | ':' | '.' | '/' => '-',
                    _ => c,
                })
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .any(|token| keywords.iter().any(|kw| token == *kw))
}

/// Check if an attribute contains any of the given keywords (substring match).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn attribute_contains_any(tag: &tl::HTMLTag, attr: &str, keywords: &[&str]) -> bool {
    let Some(attr_value) = tag.attributes().get(attr) else {
        return false;
    };
    let Some(value) = attr_value else {
        return false;
    };
    let lower = value.as_utf8_str().to_ascii_lowercase();
    keywords.iter().any(|kw| lower.contains(*kw))
}

/// Check if a node has a semantic content ancestor (main, article, section).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn has_semantic_content_ancestor(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let mut current_id = node_handle.get_inner();
    while let Some(parent_id) = dom_ctx.parent_of(current_id) {
        if let Some(parent_info) = dom_ctx.tag_info(parent_id, parser) {
            if matches!(parent_info.name.as_str(), "main" | "article" | "section") {
                return true;
            }
        }
        if let Some(parent_handle) = dom_ctx.node_handle(parent_id) {
            if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
                let parent_name = normalized_tag_name(parent_tag.name().as_utf8_str());
                if matches!(parent_name.as_ref(), "main" | "article" | "section") {
                    return true;
                }
                if tag_has_main_semantics(parent_tag) {
                    return true;
                }
            }
        }
        current_id = parent_id;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::style_sets_display_or_white_space;

    #[test]
    fn a_style_with_a_display_or_a_white_space_declaration_sets_one() {
        for style in [
            "display:inline-block",
            "display:block",
            "white-space:pre",
            "white-space: pre-wrap",
            "DISPLAY:none",
            "White-Space:PRE",
            "display : inline",
            " display\t:\tinline",
            "\n white-space \n : pre",
            "color:red;display:block",
            "color:red; white-space:pre;",
            ";;display:block",
            "display:",
        ] {
            assert!(style_sets_display_or_white_space(style.as_bytes()), "{style:?}");
        }
    }

    #[test]
    fn a_style_with_no_display_and_no_white_space_declaration_sets_none() {
        for style in [
            "",
            " ",
            ";",
            "color:red",
            "color:red;",
            "display",
            "display;color:red",
            "--display:block",
            "text-white-space-x:pre",
            "white-space-collapse:preserve",
            "displays:block",
            "white space:pre",
            "color:display",
            "content:\"display:block\"",
            ":display",
        ] {
            assert!(!style_sets_display_or_white_space(style.as_bytes()), "{style:?}");
        }
    }

    #[test]
    fn a_semicolon_in_a_quoted_value_starts_a_declaration() {
        assert!(style_sets_display_or_white_space(b"content:\"a;display:x\""));
    }

    #[test]
    fn a_source_value_is_decoded_before_it_is_read() {
        use super::style_attribute_sets_display_or_white_space;

        for value in [
            "white-space:pre",
            "white-space&#58;pre",
            "display&#x3a;inline-block",
            "display&#X3A;inline-block",
            "display&colon;inline-block",
            "dis&#112;lay:inline-block",
            "DIS&#80;LAY:BLOCK",
            "color:red&#59;display:block",
            "&#32;display&#9;:block",
            "a&amp;b:c;display:block",
        ] {
            assert!(
                style_attribute_sets_display_or_white_space(value.as_bytes()),
                "{value:?}"
            );
        }
        for value in [
            "color:red",
            "color&#58;red",
            "display-x&#58;1",
            "white-space&amp;#58;pre",
            "display&amp;colon;block",
            "&display:block",
            "<!-- display:block -->&#58;",
        ] {
            assert!(
                !style_attribute_sets_display_or_white_space(value.as_bytes()),
                "{value:?}"
            );
        }
    }
}
