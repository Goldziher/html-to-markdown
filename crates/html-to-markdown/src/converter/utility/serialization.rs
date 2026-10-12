//! Output serialization and formatting.
//!
//! Utilities for serializing HTML elements back to string format, used for preserving
//! original HTML for elements like SVG, math, and custom elements.

use crate::converter::utility::attributes::decoded_attribute;
use crate::converter::utility::content::normalized_tag_name;
use crate::options::conversion::NATIVE_STACK_SAFE_DEPTH;
use std::borrow::Cow;

/// Escape a reconstructed HTML attribute value for safe interpolation into a `"..."`-quoted
/// attribute.
///
/// The reconstruction always re-quotes with `"` regardless of the source HTML's original quote
/// character, so a value that legitimately contained a literal `"` (valid and common inside a
/// single-quoted HTML attribute, e.g. `title='He said "hi"'`) must have that `"` escaped —
/// otherwise it terminates the reconstructed attribute early and whatever follows in the
/// original value is reinterpreted as new, unintended attributes (audit #24).
pub fn escape_html_attribute_value(value: &str) -> Cow<'_, str> {
    if value.contains('"') {
        Cow::Owned(value.replace('"', "&quot;"))
    } else {
        Cow::Borrowed(value)
    }
}

/// Serialize an element to HTML string (for SVG and Math elements).
///
/// Depth-guarded by [`NATIVE_STACK_SAFE_DEPTH`] (see [`serialize_element_at_depth`]): none
/// of this module's callers track an ambient DOM depth to pass in, so the conservative
/// native-stack-safe ceiling is applied unconditionally rather than the caller-configurable
/// `effective_max_depth`.
#[allow(clippy::trivially_copy_pass_by_ref)]
// ~keep reason: used only when the visitor feature is active (PreserveHtml path) or
// ~keep inline-images feature is active (SVG serialization).
#[allow(dead_code)]
pub fn serialize_element(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    serialize_element_at_depth(node_handle, parser, 0)
}

/// Serialize a node to HTML string.
///
/// See [`serialize_element`] for the depth-guard contract.
#[allow(clippy::trivially_copy_pass_by_ref)]
// ~keep reason: used only when the visitor feature is active (PreserveHtml path).
#[allow(dead_code)]
pub fn serialize_node(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    serialize_node_at_depth(node_handle, parser, 0)
}

/// Serialize an element to HTML string, stopping descent once `depth` reaches
/// [`NATIVE_STACK_SAFE_DEPTH`].
///
/// Mutually recursive with [`serialize_node_at_depth`] over `tag.children()`, with no depth
/// bound previously — a pathologically nested "preserve raw HTML" subtree (reachable from
/// ~20 call sites across the crate, e.g. visitor `PreserveHtml` results) could overflow the
/// stack (audit #23). The element's own opening tag and attributes are always emitted; only
/// its descendants are dropped once the budget is exhausted.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_element_at_depth(node_handle: &tl::NodeHandle, parser: &tl::Parser, depth: usize) -> String {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let tag_name = normalized_tag_name(tag.name().as_utf8_str());
        let mut html = String::with_capacity(256);
        html.push('<');
        html.push_str(&tag_name);

        for (key, value_opt) in tag.attributes().iter() {
            html.push(' ');
            html.push_str(&key);
            if let Some(value) = value_opt {
                html.push_str("=\"");
                html.push_str(&escape_html_attribute_value(&value));
                html.push('"');
            }
        }

        let has_children = !tag.children().top().is_empty();
        if has_children {
            html.push('>');
            if depth >= NATIVE_STACK_SAFE_DEPTH {
                tracing::warn!(
                    target: "html_to_markdown::convert",
                    max_depth = NATIVE_STACK_SAFE_DEPTH,
                    tag = %tag_name,
                    "raw HTML serialization reached the effective depth limit; descendants were skipped"
                );
            } else {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    html.push_str(&serialize_node_at_depth(child_handle, parser, depth + 1));
                }
            }
            html.push_str("</");
            html.push_str(&tag_name);
            html.push('>');
        } else {
            html.push_str(" />");
        }
        return html;
    }
    String::new()
}

/// Serialize a node to HTML string, stopping descent once `depth` reaches
/// [`NATIVE_STACK_SAFE_DEPTH`]. See [`serialize_element_at_depth`] for the depth-guard
/// contract.
fn serialize_node_at_depth(node_handle: &tl::NodeHandle, parser: &tl::Parser, depth: usize) -> String {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => bytes.as_utf8_str().to_string(),
            tl::Node::Tag(_) => serialize_element_at_depth(node_handle, parser, depth),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

/// Serialize a tag to HTML, wrapping `serialize_node_to_html`.
pub fn serialize_tag_to_html(handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    let mut html = String::new();
    serialize_node_to_html(handle, parser, &mut html);
    html
}

/// Recursively serialize a node to HTML.
///
/// Depth-guarded by [`NATIVE_STACK_SAFE_DEPTH`]; see [`serialize_element_at_depth`] for the
/// rationale.
#[allow(clippy::trivially_copy_pass_by_ref)]
// ~keep reason: used only when the visitor feature is active (PreserveHtml path).
#[allow(dead_code)]
pub fn serialize_node_to_html(handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
    serialize_node_to_html_at_depth(handle, parser, output, 0);
}

fn serialize_node_to_html_at_depth(handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String, depth: usize) {
    match handle.get(parser) {
        Some(tl::Node::Tag(tag)) => {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());

            output.push('<');
            output.push_str(&tag_name);

            for (key, value) in tag.attributes().iter() {
                output.push(' ');
                output.push_str(&key);
                if let Some(val) = value {
                    output.push_str("=\"");
                    output.push_str(&escape_html_attribute_value(&val));
                    output.push('"');
                }
            }

            output.push('>');

            if depth >= NATIVE_STACK_SAFE_DEPTH {
                tracing::warn!(
                    target: "html_to_markdown::convert",
                    max_depth = NATIVE_STACK_SAFE_DEPTH,
                    tag = %tag_name,
                    "raw HTML serialization reached the effective depth limit; descendants were skipped"
                );
            } else {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    if tag_name == "script"
                        && let Some(tl::Node::Raw(text)) = child_handle.get(parser)
                    {
                        output.push_str(&script_text_as_written(tag, &text.as_utf8_str()));
                    } else {
                        serialize_node_to_html_at_depth(child_handle, parser, output, depth + 1);
                    }
                }
            }

            if !matches!(
                tag_name.as_ref(),
                "br" | "hr"
                    | "img"
                    | "input"
                    | "meta"
                    | "link"
                    | "area"
                    | "base"
                    | "col"
                    | "embed"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            ) {
                output.push_str("</");
                output.push_str(&tag_name);
                output.push('>');
            }
        }
        Some(tl::Node::Raw(bytes)) => {
            if let Ok(text) = std::str::from_utf8(bytes.as_bytes()) {
                output.push_str(text);
            }
        }
        _ => {}
    }
}

/// The text of the element `script` as the page has it. `text` is a text node of the element.
///
/// ~keep The first pass writes each `&` of a structured data script as `&amp;`, so that the
/// ~keep reader of the metadata decodes it once and gives the JSON as written
/// ~keep (`push_kept_element`). HTML output has no such decode: it takes that one escape back,
/// ~keep or `?a=1&b=2` is written as `?a=1&amp;b=2`. The `&lt;` of a `<` stays, as before that
/// ~keep escape. A script of another type is not escaped by the first pass and is not changed.
fn script_text_as_written<'a>(script: &tl::HTMLTag<'_>, text: &'a str) -> Cow<'a, str> {
    if !text.contains("&amp;") {
        return Cow::Borrowed(text);
    }
    let is_structured_data = decoded_attribute(script, "type").is_some_and(|type_value| {
        let media_type = type_value.split(';').next().unwrap_or_else(|| type_value.as_ref());
        media_type.trim().eq_ignore_ascii_case("application/ld+json")
    });
    if is_structured_data {
        Cow::Owned(text.replace("&amp;", "&"))
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{serialize_element, serialize_tag_to_html};

    fn written_again(html: &str) -> String {
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_handle = dom
            .children()
            .iter()
            .find(|handle| matches!(handle.get(parser), Some(tl::Node::Tag(_))))
            .expect("tag node");
        serialize_tag_to_html(node_handle, parser)
    }

    /// The first pass wrote `a & b &amp; c < d` as `a &amp; b &amp;amp; c &lt; d`.
    #[test]
    fn a_structured_data_script_is_written_with_the_ampersands_of_the_page() {
        for open_tag in [
            r#"<script type="application/ld+json">"#,
            r#"<script type="Application/LD+JSON; charset=utf-8">"#,
        ] {
            let html = format!("{open_tag}a &amp; b &amp;amp; c &lt; d</script>");
            assert_eq!(
                written_again(&html),
                format!("{open_tag}a & b &amp; c &lt; d</script>"),
                "{html:?}"
            );
        }
    }

    /// The first pass escapes a structured data script only: a script of another type (it is in
    /// the tree when it is in an `svg` element) and another element are written as the tree has
    /// them.
    #[test]
    fn a_script_of_another_type_is_written_as_the_tree_has_it() {
        for html in [
            "<script>if (a &amp;&amp; b) {}</script>",
            r#"<script type="text/javascript">if (a &amp;&amp; b) {}</script>"#,
            r#"<div type="application/ld+json">a &amp; b</div>"#,
        ] {
            assert_eq!(written_again(html), html);
        }
    }

    /// ~keep audit #24 finding 3: the source HTML's `"` is valid, inert content inside a
    /// single-quoted attribute (e.g. `title='x" onclick="alert(1)" y='`). Reconstructing it
    /// into a double-quoted attribute without escaping manufactures a real `onclick` that
    /// never existed as an attribute in the original document.
    const QUOTE_BREAKING_HTML: &str = r#"<foo title='x" onclick="alert(1)" y=' data-safe="1">"#;

    #[test]
    fn should_escape_a_quote_inside_serialize_element_attribute_reconstruction() {
        let dom = tl::parse(QUOTE_BREAKING_HTML, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_handle = dom
            .children()
            .iter()
            .find(|handle| matches!(handle.get(parser), Some(tl::Node::Tag(_))))
            .expect("tag node");
        let result = serialize_element(node_handle, parser);
        assert_eq!(
            result,
            "<foo title=\"x&quot; onclick=&quot;alert(1)&quot; y=\" data-safe=\"1\" />"
        );
    }

    #[test]
    fn should_escape_a_quote_inside_serialize_tag_to_html_attribute_reconstruction() {
        let dom = tl::parse(QUOTE_BREAKING_HTML, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_handle = dom
            .children()
            .iter()
            .find(|handle| matches!(handle.get(parser), Some(tl::Node::Tag(_))))
            .expect("tag node");
        let result = serialize_tag_to_html(node_handle, parser);
        assert_eq!(
            result,
            "<foo title=\"x&quot; onclick=&quot;alert(1)&quot; y=\" data-safe=\"1\"></foo>"
        );
    }
}
