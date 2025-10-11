//! hOCR to Markdown conversion
//!
//! Converts structured hOCR elements to Markdown while preserving document hierarchy.

use super::types::{HocrElement, HocrElementType};

/// Convert hOCR elements to Markdown with semantic formatting
///
/// Transforms hOCR document structure into clean, readable Markdown while preserving
/// document hierarchy and semantic meaning.
///
/// # Arguments
///
/// * `elements` - hOCR elements to convert (typically from `extract_hocr_document`)
/// * `preserve_structure` - If `true`, sorts elements by their `order` property to respect reading order
///
/// # Returns
///
/// A `String` containing the formatted Markdown output
///
/// # Semantic Conversion
///
/// All 40 hOCR 1.2 element types are converted with appropriate markdown formatting:
///
/// | hOCR Element | Markdown Output |
/// |--------------|-----------------|
/// | `ocr_title`, `ocr_chapter` | `# Heading` |
/// | `ocr_section` | `## Heading` |
/// | `ocr_subsection` | `### Heading` |
/// | `ocr_par` | Paragraph with blank lines |
/// | `ocr_blockquote` | `> Quote` |
/// | `ocr_abstract` | `**Abstract**` header |
/// | `ocr_author` | `*Author*` (italic) |
/// | `ocr_image`, `ocr_photo` | `![alt](path)` |
/// | `ocr_math`, `ocr_chem` | `` `formula` `` (inline code) |
/// | `ocr_display` | ` ```equation``` ` (code block) |
/// | `ocr_separator` | `---` (horizontal rule) |
/// | `ocr_dropcap` | `**Letter**` (bold) |
/// | `ocrx_word` | Word with markdown escaping |
///
/// # Example
///
/// ```rust
/// use html_to_markdown_rs::hocr::{extract_hocr_document, convert_to_markdown};
///
/// let html = r#"<div class="ocr_page">
///     <h1 class="ocr_title">Document Title</h1>
///     <p class="ocr_par" title="order 1">
///         <span class="ocrx_word" title="bbox 10 10 50 30; x_wconf 95">Hello</span>
///         <span class="ocrx_word" title="bbox 60 10 100 30; x_wconf 92">World</span>
///     </p>
/// </div>"#;
///
/// let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
/// let (elements, _) = extract_hocr_document(&dom, false);
/// let markdown = convert_to_markdown(&elements, true);
/// // Output: "# Document Title\n\nHello World"
/// ```
pub fn convert_to_markdown(elements: &[HocrElement], preserve_structure: bool) -> String {
    let mut output = String::new();

    // Sort elements by order property if preserve_structure is enabled
    let mut sorted_elements: Vec<_> = elements.iter().collect();
    if preserve_structure {
        sorted_elements.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
    }

    for element in sorted_elements {
        convert_element(element, &mut output, 0, preserve_structure);
    }

    output.trim().to_string()
}

fn convert_element(element: &HocrElement, output: &mut String, depth: usize, preserve_structure: bool) {
    match element.element_type {
        // Logical structure - headings
        HocrElementType::OcrTitle | HocrElementType::OcrChapter | HocrElementType::OcrPart => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("# ");
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }
        HocrElementType::OcrSection => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("## ");
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }
        HocrElementType::OcrSubsection => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("### ");
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }
        HocrElementType::OcrSubsubsection => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("#### ");
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }

        // Paragraphs
        HocrElementType::OcrPar => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }

        // Blockquotes
        HocrElementType::OcrBlockquote => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            let mut quote_content = String::new();
            append_text_and_children(element, &mut quote_content, depth, preserve_structure);
            for line in quote_content.trim().lines() {
                output.push_str("> ");
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Lines - join with space
        HocrElementType::OcrLine | HocrElementType::OcrxLine => {
            append_text_and_children(element, output, depth, preserve_structure);
            if !output.ends_with(' ') && !output.ends_with('\n') {
                output.push(' ');
            }
        }

        // Words - join with space
        HocrElementType::OcrxWord => {
            if !element.text.is_empty() {
                output.push_str(&element.text);
                output.push(' ');
            }
        }

        // Headers and footers as italic
        HocrElementType::OcrHeader | HocrElementType::OcrFooter => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push('*');
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space before closing italic
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("*\n\n");
        }

        // Captions
        HocrElementType::OcrCaption => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push('*');
            append_text_and_children(element, output, depth, preserve_structure);
            // Trim trailing space before closing italic
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("*\n\n");
        }

        // Page numbers
        HocrElementType::OcrPageno => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("---\n");
            append_text_and_children(element, output, depth, preserve_structure);
            output.push_str("\n---\n\n");
        }

        // Abstract - treat as blockquote or emphasized section
        HocrElementType::OcrAbstract => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("**Abstract**\n\n");
            append_text_and_children(element, output, depth, preserve_structure);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n\n");
        }

        // Author - emphasize
        HocrElementType::OcrAuthor => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push('*');
            append_text_and_children(element, output, depth, preserve_structure);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("*\n\n");
        }

        // Separator - horizontal rule
        HocrElementType::OcrSeparator => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("---\n\n");
        }

        // Tables and float elements - containers with context
        HocrElementType::OcrTable => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            // Sort children by order property if preserve_structure is enabled
            let mut sorted_children: Vec<_> = element.children.iter().collect();
            if preserve_structure {
                sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
            }
            for child in sorted_children {
                convert_element(child, output, depth + 1, preserve_structure);
            }
            output.push_str("\n\n");
        }

        HocrElementType::OcrFloat | HocrElementType::OcrTextfloat | HocrElementType::OcrTextimage => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            // Sort children by order property if preserve_structure is enabled
            let mut sorted_children: Vec<_> = element.children.iter().collect();
            if preserve_structure {
                sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
            }
            for child in sorted_children {
                convert_element(child, output, depth + 1, preserve_structure);
            }
            output.push_str("\n\n");
        }

        // Images - markdown image placeholder or alt text
        HocrElementType::OcrImage | HocrElementType::OcrPhoto | HocrElementType::OcrLinedrawing => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            // Extract image path from properties if available
            if let Some(ref image_path) = element.properties.image {
                output.push_str("![");
                append_text_and_children(element, output, depth, preserve_structure);
                if output.ends_with(' ') {
                    output.pop();
                }
                output.push_str("](");
                output.push_str(image_path);
                output.push_str(")\n\n");
            } else {
                // No image path, just extract any text content
                output.push_str("![Image]\n\n");
            }
        }

        // Math and chemistry - wrap in code or special markers
        HocrElementType::OcrMath | HocrElementType::OcrChem => {
            output.push('`');
            append_text_and_children(element, output, depth, preserve_structure);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push('`');
        }

        // Display equations - block-level math
        HocrElementType::OcrDisplay => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("```\n");
            append_text_and_children(element, output, depth, preserve_structure);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n```\n\n");
        }

        // Drop cap - emphasize first letter
        HocrElementType::OcrDropcap => {
            output.push_str("**");
            append_text_and_children(element, output, depth, preserve_structure);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("**");
        }

        // Glyphs and character info - just extract text
        HocrElementType::OcrGlyph | HocrElementType::OcrGlyphs | HocrElementType::OcrCinfo => {
            append_text_and_children(element, output, depth, preserve_structure);
        }

        // Container elements - just process children
        HocrElementType::OcrPage
        | HocrElementType::OcrCarea
        | HocrElementType::OcrDocument
        | HocrElementType::OcrLinear
        | HocrElementType::OcrxBlock
        | HocrElementType::OcrColumn
        | HocrElementType::OcrXycut => {
            // Sort children by order property if preserve_structure is enabled
            let mut sorted_children: Vec<_> = element.children.iter().collect();
            if preserve_structure {
                sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
            }

            for child in sorted_children {
                convert_element(child, output, depth + 1, preserve_structure);
            }
        }

        // Skip noise
        HocrElementType::OcrNoise => {}
    }
}

fn append_text_and_children(element: &HocrElement, output: &mut String, depth: usize, preserve_structure: bool) {
    if !element.text.is_empty() {
        output.push_str(&element.text);
        if !element.children.is_empty() {
            output.push(' ');
        }
    }

    // Sort children by order property if preserve_structure is enabled
    let mut sorted_children: Vec<_> = element.children.iter().collect();
    if preserve_structure {
        sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
    }

    for child in sorted_children {
        convert_element(child, output, depth + 1, preserve_structure);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hocr::types::{HocrElement, HocrElementType, HocrProperties};

    #[test]
    fn test_convert_title() {
        let element = HocrElement {
            element_type: HocrElementType::OcrTitle,
            properties: HocrProperties::default(),
            text: "Document Title".to_string(),
            children: vec![],
        };

        let markdown = convert_to_markdown(&[element], true);
        assert_eq!(markdown, "# Document Title");
    }

    #[test]
    fn test_convert_paragraph_with_words() {
        let par = HocrElement {
            element_type: HocrElementType::OcrPar,
            properties: HocrProperties::default(),
            text: String::new(),
            children: vec![
                HocrElement {
                    element_type: HocrElementType::OcrxWord,
                    properties: HocrProperties::default(),
                    text: "Hello".to_string(),
                    children: vec![],
                },
                HocrElement {
                    element_type: HocrElementType::OcrxWord,
                    properties: HocrProperties::default(),
                    text: "World".to_string(),
                    children: vec![],
                },
            ],
        };

        let markdown = convert_to_markdown(&[par], true);
        assert!(markdown.contains("Hello"));
        assert!(markdown.contains("World"));
    }

    #[test]
    fn test_convert_blockquote() {
        let quote = HocrElement {
            element_type: HocrElementType::OcrBlockquote,
            properties: HocrProperties::default(),
            text: "This is a quote".to_string(),
            children: vec![],
        };

        let markdown = convert_to_markdown(&[quote], true);
        assert!(markdown.starts_with("> "));
    }
}
