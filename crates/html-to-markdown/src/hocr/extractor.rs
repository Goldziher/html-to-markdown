//! hOCR element extraction
//!
//! Extracts structured hOCR elements from HTML DOM.

use super::parser::parse_properties;
use super::types::{HocrElement, HocrElementType, HocrMetadata};

/// Extract complete hOCR document structure from HTML DOM
///
/// Parses an HTML document containing hOCR annotations and extracts all hOCR elements
/// along with document metadata.
///
/// # Arguments
///
/// * `dom` - The parsed HTML DOM (from the astral-tl parser)
/// * `debug` - Enable debug logging for property parsing
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<HocrElement>` - All top-level hOCR elements with their full hierarchies
/// * `HocrMetadata` - Document metadata from `<head>` meta tags
///
/// # hOCR 1.2 Compliance
///
/// Supports all 40 element types:
/// - Logical structure (12): ocr_title, ocr_chapter, ocr_section, ocr_par, etc.
/// - Typesetting (6): ocr_page, ocr_carea, ocr_line, etc.
/// - Float elements (13): ocr_image, ocr_table, ocr_math, etc.
/// - Inline elements (6): ocr_dropcap, ocr_glyph, etc.
/// - Engine-specific (3): ocrx_block, ocrx_line, ocrx_word
///
/// Extracts all 20+ properties from title attributes (bbox, x_wconf, baseline, order, etc.)
/// and all 5 metadata fields (ocr-system, ocr-capabilities, ocr-langs, etc.)
///
/// # Example
///
/// ```rust
/// use html_to_markdown_rs::hocr::extract_hocr_document;
///
/// let html = r#"<div class="ocr_page" title="bbox 0 0 1000 1500">
///     <p class="ocr_par" title="bbox 100 100 900 200">
///         <span class="ocrx_word" title="bbox 100 100 150 130; x_wconf 95">Hello</span>
///     </p>
/// </div>"#;
/// let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
/// let (elements, metadata) = extract_hocr_document(&dom, false);
/// ```
pub fn extract_hocr_document(dom: &tl::VDom, debug: bool) -> (Vec<HocrElement>, HocrMetadata) {
    let parser = dom.parser();
    let mut elements = Vec::new();
    let metadata = extract_metadata(dom);

    for child_handle in dom.children().iter() {
        collect_hocr_elements(child_handle, parser, &mut elements, debug);
    }

    (elements, metadata)
}

/// Recursively collect hOCR elements from DOM tree
fn collect_hocr_elements(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    elements: &mut Vec<HocrElement>,
    debug: bool,
) {
    if let Some(element) = extract_element(node_handle, parser, debug) {
        elements.push(element);
    } else if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let children = tag.children();
        for child_handle in children.top().iter() {
            collect_hocr_elements(child_handle, parser, elements, debug);
        }
    }
}

/// Extract hOCR metadata from HTML head (or from orphaned meta tags after sanitization)
fn extract_metadata(dom: &tl::VDom) -> HocrMetadata {
    let mut metadata = HocrMetadata::default();
    let parser = dom.parser();

    fn extract_from_meta_tag(meta_tag: &tl::HTMLTag, metadata: &mut HocrMetadata) {
        let attrs = meta_tag.attributes();
        if let (Some(name), Some(content)) = (attrs.get("name").flatten(), attrs.get("content").flatten()) {
            let name_str = name.as_utf8_str();
            let content_str = content.as_utf8_str().to_string();

            match name_str.as_ref() {
                "ocr-system" => metadata.ocr_system = Some(content_str),
                "ocr-capabilities" => {
                    metadata.ocr_capabilities = content_str.split_whitespace().map(|s| s.to_string()).collect();
                }
                "ocr-number-of-pages" => {
                    metadata.ocr_number_of_pages = content_str.parse().ok();
                }
                "ocr-langs" => {
                    metadata.ocr_langs = content_str.split_whitespace().map(|s| s.to_string()).collect();
                }
                "ocr-scripts" => {
                    metadata.ocr_scripts = content_str.split_whitespace().map(|s| s.to_string()).collect();
                }
                _ => {}
            }
        }
    }

    fn find_meta_tags<'a>(node_handle: &tl::NodeHandle, parser: &'a tl::Parser<'a>, metadata: &mut HocrMetadata) {
        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            let tag_name = tag.name().as_utf8_str();

            if tag_name == "meta" {
                extract_from_meta_tag(tag, metadata);
            }

            let children = tag.children();
            for child_handle in children.top().iter() {
                find_meta_tags(child_handle, parser, metadata);
            }
        }
    }

    for child_handle in dom.children().iter() {
        find_meta_tags(child_handle, parser, &mut metadata);
    }

    metadata
}

/// Extract a single hOCR element and its children
fn extract_element(node_handle: &tl::NodeHandle, parser: &tl::Parser, debug: bool) -> Option<HocrElement> {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let attrs = tag.attributes();
        let class_attr = attrs.get("class").flatten()?;
        let classes = class_attr.as_utf8_str();
        if !classes.as_ref().contains("ocr") {
            return None;
        }

        let element_type = classes.split_whitespace().find_map(HocrElementType::from_class)?;

        let properties = if let Some(title) = attrs.get("title").flatten() {
            parse_properties(&title.as_utf8_str(), debug)
        } else {
            Default::default()
        };

        let mut text = String::new();
        let mut children = Vec::new();

        let tag_children = tag.children();
        for child_handle in tag_children.top().iter() {
            if let Some(tl::Node::Raw(bytes)) = child_handle.get(parser) {
                text.push_str(&bytes.as_utf8_str());
            } else if let Some(child_element) = extract_element(child_handle, parser, debug) {
                children.push(child_element);
            }
        }

        Some(HocrElement {
            element_type,
            properties,
            text: text.trim().to_string(),
            children,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_word() {
        let html = r#"<span class="ocrx_word" title="bbox 100 50 150 80; x_wconf 95">Hello</span>"#;
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let element = extract_element(&dom.children()[0], parser, false).unwrap();
        assert!(matches!(element.element_type, HocrElementType::OcrxWord));
        assert_eq!(element.text, "Hello");
        assert!(element.properties.bbox.is_some());
        assert_eq!(element.properties.x_wconf, Some(95.0));
    }

    #[test]
    fn test_extract_paragraph() {
        let html = r#"<p class="ocr_par" title="bbox 0 0 200 100">
            <span class="ocrx_word" title="bbox 10 10 50 30; x_wconf 90">First</span>
            <span class="ocrx_word" title="bbox 60 10 100 30; x_wconf 92">Word</span>
        </p>"#;
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let element = extract_element(&dom.children()[0], parser, false).unwrap();
        assert!(matches!(element.element_type, HocrElementType::OcrPar));
        assert_eq!(element.children.len(), 2);
        assert!(matches!(element.children[0].element_type, HocrElementType::OcrxWord));
    }

    #[test]
    fn test_extract_metadata() {
        let html = r#"<!DOCTYPE html>
<html>
<head>
<meta name="ocr-system" content="tesseract 4.1.1" />
<meta name="ocr-capabilities" content="ocr_page ocr_carea ocr_par ocr_line ocrx_word" />
<meta name="ocr-number-of-pages" content="5" />
</head>
<body>
<div class="ocr_page"></div>
</body>
</html>"#;
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        let (_, metadata) = extract_hocr_document(&dom, false);

        assert_eq!(metadata.ocr_system, Some("tesseract 4.1.1".to_string()));
        assert!(metadata.ocr_capabilities.contains(&"ocr_page".to_string()));
        assert_eq!(metadata.ocr_number_of_pages, Some(5));
    }
}
