//! Output serialization and formatting.
//!
//! Utilities for serializing HTML elements back to string format, used for preserving
//! original HTML for elements like SVG, math, and custom elements.

use crate::converter::utility::content::normalized_tag_name;

/// Serialize an element to HTML string (for SVG and Math elements).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_element(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
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
                html.push_str(&value);
                html.push('"');
            }
        }

        let has_children = !tag.children().top().is_empty();
        if has_children {
            html.push('>');
            let children = tag.children();
            {
                for child_handle in children.top().iter() {
                    html.push_str(&serialize_node(child_handle, parser));
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

/// Serialize a node to HTML string.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_node(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => bytes.as_utf8_str().to_string(),
            tl::Node::Tag(_) => serialize_element(node_handle, parser),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

/// Serialize a tag to HTML, wrapping serialize_node_to_html.
pub(crate) fn serialize_tag_to_html(handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    let mut html = String::new();
    serialize_node_to_html(handle, parser, &mut html);
    html
}

/// Recursively serialize a node to HTML.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_node_to_html(handle: &tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
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
                    output.push_str(&val);
                    output.push('"');
                }
            }

            output.push('>');

            let children = tag.children();
            for child_handle in children.top().iter() {
                serialize_node_to_html(child_handle, parser, output);
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
