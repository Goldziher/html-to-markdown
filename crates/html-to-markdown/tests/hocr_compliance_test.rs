//! hOCR 1.2 compliance integration tests
//!
//! Tests full hOCR specification support across all element types and properties.

use html_to_markdown_rs::hocr::{HocrElement, HocrElementType, convert_to_markdown, extract_hocr_document};

#[test]
fn test_full_hocr_document_structure() {
    let hocr = r#"<!DOCTYPE html>
<html>
<head>
<meta name="ocr-system" content="tesseract 5.0" />
<meta name="ocr-capabilities" content="ocr_page ocr_carea ocr_par ocr_line ocrx_word" />
<meta name="ocr-number-of-pages" content="1" />
</head>
<body>
<div class="ocr_page" title="bbox 0 0 1000 1000">
    <div class="ocr_carea" title="bbox 0 0 1000 100">
        <h1 class="ocr_title" title="bbox 0 0 500 50">Document Title</h1>
    </div>
    <div class="ocr_carea" title="bbox 0 100 1000 500">
        <h2 class="ocr_chapter" title="bbox 0 100 300 130">Chapter 1</h2>
        <p class="ocr_par" title="bbox 0 150 900 250">
            <span class="ocr_line" title="bbox 0 150 800 180">
                <span class="ocrx_word" title="bbox 0 150 50 180; x_wconf 95">This</span>
                <span class="ocrx_word" title="bbox 60 150 90 180; x_wconf 92">is</span>
                <span class="ocrx_word" title="bbox 100 150 150 180; x_wconf 98">text</span>
            </span>
        </p>
    </div>
</div>
</body>
</html>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, metadata) = extract_hocr_document(&dom, false);

    // Verify metadata
    assert_eq!(metadata.ocr_system, Some("tesseract 5.0".to_string()));
    assert_eq!(metadata.ocr_number_of_pages, Some(1));
    assert!(metadata.ocr_capabilities.contains(&"ocr_page".to_string()));

    // Verify structure
    assert!(!elements.is_empty());

    // Convert to markdown
    let markdown = convert_to_markdown(&elements, true);
    assert!(markdown.contains("Document Title"));
    assert!(markdown.contains("Chapter 1"));
    assert!(markdown.contains("This is text"));
}

#[test]
fn test_advanced_properties() {
    let hocr = r#"<div class="ocr_page" title="bbox 0 0 1000 1000">
    <span class="ocr_line" title="bbox 100 50 500 80; baseline 0.015 -18; x_font &quot;Arial&quot;; x_fsize 12">
        <span class="ocrx_word" title="bbox 100 50 150 80; x_wconf 95; textangle 2.5">Word</span>
    </span>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    // Find the line element
    fn find_line(elements: &[HocrElement]) -> Option<&HocrElement> {
        for elem in elements {
            if matches!(elem.element_type, HocrElementType::OcrLine) {
                return Some(elem);
            }
            if let Some(found) = find_line(&elem.children) {
                return Some(found);
            }
        }
        None
    }

    let line = find_line(&elements).expect("Should find ocr_line");

    // Verify advanced properties
    assert!(line.properties.baseline.is_some());
    assert_eq!(line.properties.baseline.unwrap().slope, 0.015);
    assert_eq!(line.properties.baseline.unwrap().constant, -18);
    assert_eq!(line.properties.x_font, Some("Arial".to_string()));
    assert_eq!(line.properties.x_fsize, Some(12));

    // Verify word properties
    assert_eq!(line.children.len(), 1);
    assert_eq!(line.children[0].properties.textangle, Some(2.5));
}

#[test]
fn test_all_logical_elements() {
    let hocr = r#"<div class="ocr_document">
    <div class="ocr_part"><span class="ocrx_word" title="bbox 0 0 50 20">Part</span></div>
    <div class="ocr_chapter"><span class="ocrx_word" title="bbox 0 0 50 20">Chapter</span></div>
    <div class="ocr_section"><span class="ocrx_word" title="bbox 0 0 50 20">Section</span></div>
    <div class="ocr_subsection"><span class="ocrx_word" title="bbox 0 0 50 20">Subsection</span></div>
    <p class="ocr_par"><span class="ocrx_word" title="bbox 0 0 50 20">Paragraph</span></p>
    <blockquote class="ocr_blockquote"><span class="ocrx_word" title="bbox 0 0 50 20">Quote</span></blockquote>
    <div class="ocr_caption"><span class="ocrx_word" title="bbox 0 0 50 20">Caption</span></div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    let markdown = convert_to_markdown(&elements, true);

    // Verify conversion
    assert!(markdown.contains("# Part") || markdown.contains("# Chapter"));
    assert!(markdown.contains("## Section"));
    assert!(markdown.contains("### Subsection"));
    assert!(markdown.contains("Paragraph"));
    assert!(markdown.contains("> Quote"));
    assert!(markdown.contains("*Caption*"));
}

#[test]
fn test_float_elements() {
    let hocr = r#"<div class="ocr_page" title="bbox 0 0 1000 1000">
    <div class="ocr_header" title="bbox 0 0 1000 50"><span class="ocrx_word" title="bbox 0 0 50 20">Header</span></div>
    <div class="ocr_footer" title="bbox 0 950 1000 1000"><span class="ocrx_word" title="bbox 0 0 50 20">Footer</span></div>
    <div class="ocr_table" title="bbox 100 100 500 300">
        <span class="ocrx_word" title="bbox 100 100 150 120">Table</span>
    </div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    let markdown = convert_to_markdown(&elements, true);

    // Headers and footers should be in italic
    assert!(markdown.contains("*Header*"));
    assert!(markdown.contains("*Footer*"));
}

#[test]
fn test_character_level_properties() {
    let hocr = r#"<span class="ocr_cinfo" title="x_confs 95.3 87.2 92.1; x_bboxes 0 0 10 20 10 0 20 20 20 0 30 20">
    <span class="ocrx_word" title="bbox 0 0 30 20">ABC</span>
</span>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    // Find ocr_cinfo
    fn find_cinfo(elements: &[HocrElement]) -> Option<&HocrElement> {
        for elem in elements {
            if matches!(elem.element_type, HocrElementType::OcrCinfo) {
                return Some(elem);
            }
            if let Some(found) = find_cinfo(&elem.children) {
                return Some(found);
            }
        }
        None
    }

    if let Some(cinfo) = find_cinfo(&elements) {
        // Verify character-level confidences
        assert_eq!(cinfo.properties.x_confs, vec![95.3, 87.2, 92.1]);

        // Verify character bounding boxes
        assert_eq!(cinfo.properties.x_bboxes.len(), 3);
        assert_eq!(cinfo.properties.x_bboxes[0].x1, 0);
        assert_eq!(cinfo.properties.x_bboxes[1].x1, 10);
        assert_eq!(cinfo.properties.x_bboxes[2].x1, 20);
    }
}

#[test]
fn test_page_properties() {
    let hocr = r#"<div class="ocr_page" title="bbox 0 0 2480 3508; image &quot;/path/to/image.png&quot;; ppageno 5; lpageno &quot;V&quot;; scan_res 300 300">
    <p class="ocr_par">Content</p>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    // Find page element
    fn find_page(elements: &[HocrElement]) -> Option<&HocrElement> {
        for elem in elements {
            if matches!(elem.element_type, HocrElementType::OcrPage) {
                return Some(elem);
            }
            if let Some(found) = find_page(&elem.children) {
                return Some(found);
            }
        }
        None
    }

    let page = find_page(&elements).expect("Should find ocr_page");

    // Verify page properties
    assert_eq!(page.properties.image, Some("/path/to/image.png".to_string()));
    assert_eq!(page.properties.ppageno, Some(5));
    assert_eq!(page.properties.lpageno, Some("V".to_string()));
    assert_eq!(page.properties.scan_res, Some((300, 300)));
}

#[test]
fn test_content_flow_and_order() {
    let hocr = r#"<div class="ocr_linear" title="order 1; cflow &quot;main-flow&quot;">
    <p class="ocr_par" title="order 2"><span class="ocrx_word" title="bbox 0 0 50 20">Second</span></p>
    <p class="ocr_par" title="order 1"><span class="ocrx_word" title="bbox 0 0 50 20">First</span></p>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);

    // Find linear element
    fn find_linear(elements: &[HocrElement]) -> Option<&HocrElement> {
        for elem in elements {
            if matches!(elem.element_type, HocrElementType::OcrLinear) {
                return Some(elem);
            }
            if let Some(found) = find_linear(&elem.children) {
                return Some(found);
            }
        }
        None
    }

    let linear = find_linear(&elements).expect("Should find ocr_linear");

    // Verify order properties
    assert_eq!(linear.properties.order, Some(1));
    assert_eq!(linear.properties.cflow, Some("main-flow".to_string()));

    // Verify child order properties
    assert_eq!(linear.children[0].properties.order, Some(2));
    assert_eq!(linear.children[1].properties.order, Some(1));
}

#[test]
fn test_abstract_and_author() {
    let hocr = r#"<div class="ocr_document">
    <div class="ocr_abstract"><span class="ocrx_word" title="bbox 0 0 50 20">This is an abstract</span></div>
    <div class="ocr_author"><span class="ocrx_word" title="bbox 0 0 50 20">John Doe</span></div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("**Abstract**"));
    assert!(markdown.contains("This is an abstract"));
    assert!(markdown.contains("*John Doe*"));
}

#[test]
fn test_separator() {
    let hocr = r#"<div class="ocr_page">
    <p class="ocr_par"><span class="ocrx_word" title="bbox 0 0 50 20">Text before</span></p>
    <div class="ocr_separator"></div>
    <p class="ocr_par"><span class="ocrx_word" title="bbox 0 0 50 20">Text after</span></p>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("Text before"));
    assert!(markdown.contains("---"));
    assert!(markdown.contains("Text after"));
}

#[test]
fn test_image_elements() {
    let hocr = r#"<div class="ocr_page">
    <div class="ocr_image" title="image &quot;/path/to/image.png&quot;"><span class="ocrx_word" title="bbox 0 0 50 20">Alt text</span></div>
    <div class="ocr_photo"><span class="ocrx_word" title="bbox 0 0 50 20">Photo caption</span></div>
    <div class="ocr_linedrawing"><span class="ocrx_word" title="bbox 0 0 50 20">Drawing caption</span></div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("![Alt text](/path/to/image.png)"));
    assert!(markdown.contains("![Image]"));
}

#[test]
fn test_math_and_chem() {
    let hocr = r#"<div class="ocr_page">
    <span class="ocr_math"><span class="ocrx_word" title="bbox 0 0 50 20">E=mc^2</span></span>
    <span class="ocr_chem"><span class="ocrx_word" title="bbox 0 0 50 20">H2O</span></span>
    <div class="ocr_display"><span class="ocrx_word" title="bbox 0 0 50 20">x^2 + y^2 = z^2</span></div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("`E=mc^2`"));
    assert!(markdown.contains("`H2O`"));
    assert!(markdown.contains("```"));
    assert!(markdown.contains("x^2 + y^2 = z^2"));
}

#[test]
fn test_dropcap_and_glyphs() {
    let hocr = r#"<div class="ocr_page">
    <p class="ocr_par">
        <span class="ocr_dropcap"><span class="ocrx_word" title="bbox 0 0 50 20">T</span></span>
        <span class="ocrx_word" title="bbox 0 0 50 20">his is text</span>
    </p>
    <span class="ocr_glyph" title="bbox 0 0 10 20">A</span>
    <span class="ocr_glyphs">
        <span class="ocr_glyph" title="bbox 0 0 10 20">B</span>
        <span class="ocr_glyph" title="bbox 0 0 10 20">C</span>
    </span>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("**T**"));
    assert!(markdown.contains("his is text"));
    assert!(markdown.contains("A"));
    assert!(markdown.contains("B"));
    assert!(markdown.contains("C"));
}

#[test]
fn test_float_elements_comprehensive() {
    let hocr = r#"<div class="ocr_page">
    <div class="ocr_float"><span class="ocrx_word" title="bbox 0 0 50 20">Float content</span></div>
    <div class="ocr_textfloat"><span class="ocrx_word" title="bbox 0 0 50 20">Text float</span></div>
    <div class="ocr_textimage"><span class="ocrx_word" title="bbox 0 0 50 20">Text with image</span></div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("Float content"));
    assert!(markdown.contains("Text float"));
    assert!(markdown.contains("Text with image"));
}

#[test]
fn test_container_elements() {
    let hocr = r#"<div class="ocr_document">
    <div class="ocr_column">
        <p class="ocr_par"><span class="ocrx_word" title="bbox 0 0 50 20">Column text</span></p>
    </div>
    <div class="ocr_xycut">
        <p class="ocr_par"><span class="ocrx_word" title="bbox 0 0 50 20">Layout analysis</span></p>
    </div>
    <div class="ocrx_block">
        <span class="ocrx_word" title="bbox 0 0 50 20">Block content</span>
    </div>
</div>"#;

    let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
    let (elements, _) = extract_hocr_document(&dom, false);
    let markdown = convert_to_markdown(&elements, true);

    assert!(markdown.contains("Column text"));
    assert!(markdown.contains("Layout analysis"));
    assert!(markdown.contains("Block content"));
}
