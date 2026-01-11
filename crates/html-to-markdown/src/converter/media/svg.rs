//! SVG and MathML element handling with serialization and base64 encoding.

#[allow(unused_imports)]
use std::collections::BTreeMap;
use tl::{NodeHandle, Parser};

#[cfg(feature = "inline-images")]
use crate::inline_images::{InlineImageCollector, InlineImageFormat, InlineImageSource};

#[cfg(feature = "inline-images")]
type InlineCollectorHandle = std::rc::Rc<std::cell::RefCell<InlineImageCollector>>;

/// Handle inline SVG elements with size limits and base64 encoding.
///
/// # Features
/// - SVG serialization to HTML string
/// - Size validation with configurable limits
/// - Base64 encoding for data URI
/// - Metadata extraction (aria-label, title, dimensions)
#[cfg(feature = "inline-images")]
#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::option_if_let_else)]
pub(crate) fn handle_inline_svg(
    collector_ref: &InlineCollectorHandle,
    node_handle: &NodeHandle,
    parser: &Parser,
    title_opt: Option<String>,
    attributes: BTreeMap<String, String>,
) {
    let max_size = {
        let borrow = collector_ref.borrow();
        if !borrow.capture_svg() {
            return;
        }
        borrow.max_decoded_size()
    };

    if max_size == 0 {
        let mut collector = collector_ref.borrow_mut();
        let index = collector.next_index();
        collector.warn_skip(index, "max SVG payload size is zero");
        return;
    }

    let mut collector = collector_ref.borrow_mut();
    let index = collector.next_index();

    let serialized = serialize_element(node_handle, parser);
    if serialized.is_empty() {
        collector.warn_skip(index, "unable to serialize SVG element");
        return;
    }

    let data = serialized.into_bytes();
    if data.len() as u64 > max_size {
        collector.warn_skip(
            index,
            format!(
                "serialized SVG payload ({} bytes) exceeds configured max ({})",
                data.len(),
                max_size
            ),
        );
        return;
    }

    let description = attributes
        .get("aria-label")
        .and_then(|value| non_empty_trimmed(value))
        .or_else(|| title_opt.as_deref().and_then(non_empty_trimmed));

    let filename_candidate = attributes
        .get("data-filename")
        .cloned()
        .or_else(|| attributes.get("filename").cloned())
        .or_else(|| attributes.get("data-name").cloned());

    let image = collector.build_image(
        data,
        InlineImageFormat::Svg,
        filename_candidate,
        description,
        None,
        InlineImageSource::SvgElement,
        attributes,
    );

    collector.push_image(index, image);
}

/// Serialize an element to HTML string (for SVG and Math elements).
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_element(node_handle: &NodeHandle, parser: &Parser) -> String {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let tag_name = normalized_tag_name(tag.name().as_utf8_str().as_ref());
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
pub(crate) fn serialize_node(node_handle: &NodeHandle, parser: &Parser) -> String {
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

/// Normalize tag name to lowercase.
fn normalized_tag_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Extract non-empty trimmed string or return None.
#[cfg(feature = "inline-images")]
fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Encode SVG to base64 data URI.
pub(crate) fn encode_svg_to_data_uri(svg_html: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(svg_html.as_bytes())
}
