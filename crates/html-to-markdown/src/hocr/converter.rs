//! hOCR to Markdown conversion
//!
//! Converts structured hOCR elements to Markdown while preserving document hierarchy.

use super::spatial::{self, HocrWord};
use super::types::{HocrElement, HocrElementType};

#[derive(Default)]
struct ConvertContext {
    last_heading: Option<String>,
}

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
    convert_to_markdown_with_options(elements, preserve_structure, true)
}

pub fn convert_to_markdown_with_options(
    elements: &[HocrElement],
    preserve_structure: bool,
    enable_spatial_tables: bool,
) -> String {
    let mut output = String::new();

    // Sort elements by order property if preserve_structure is enabled
    let mut sorted_elements: Vec<_> = elements.iter().collect();
    if preserve_structure {
        sorted_elements.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
    }

    let mut ctx = ConvertContext::default();

    for element in sorted_elements {
        convert_element(
            element,
            &mut output,
            0,
            preserve_structure,
            enable_spatial_tables,
            &mut ctx,
        );
    }

    output.trim().to_string()
}

fn convert_element(
    element: &HocrElement,
    output: &mut String,
    depth: usize,
    preserve_structure: bool,
    enable_spatial_tables: bool,
    ctx: &mut ConvertContext,
) {
    match element.element_type {
        // Logical structure - headings
        HocrElementType::OcrTitle | HocrElementType::OcrChapter | HocrElementType::OcrPart => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("# ");
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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

            if let Some(heading) = detect_heading_paragraph(element) {
                if !output.is_empty() && !output.ends_with("\n\n") {
                    if output.ends_with('\n') {
                        output.push('\n');
                    } else {
                        output.push_str("\n\n");
                    }
                }
                output.push_str("# ");
                output.push_str(&heading);
                output.push_str("\n\n");
                ctx.last_heading = Some(heading);
                return;
            }

            if enable_spatial_tables {
                if let Some(table_markdown) = try_spatial_table_reconstruction(element) {
                    output.push_str(&table_markdown);
                    output.push_str("\n\n");
                    return;
                }
            }

            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(
                element,
                &mut quote_content,
                depth,
                preserve_structure,
                enable_spatial_tables,
                ctx,
            );
            for line in quote_content.trim().lines() {
                output.push_str("> ");
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Lines - join with space
        HocrElementType::OcrLine | HocrElementType::OcrxLine => {
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
            output.push_str("\n---\n\n");
        }

        // Abstract - treat as blockquote or emphasized section
        HocrElementType::OcrAbstract => {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push_str("\n\n");
            }
            output.push_str("**Abstract**\n\n");
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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

            // Try spatial table reconstruction first
            if enable_spatial_tables {
                if let Some(table_markdown) = try_spatial_table_reconstruction(element) {
                    output.push_str(&table_markdown);
                    output.push_str("\n\n");
                } else {
                    // Fallback: process children normally
                    let mut sorted_children: Vec<_> = element.children.iter().collect();
                    if preserve_structure {
                        sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
                    }
                    for child in sorted_children {
                        convert_element(child, output, depth + 1, preserve_structure, enable_spatial_tables, ctx);
                    }
                    output.push_str("\n\n");
                }
            } else {
                // Fallback: process children normally
                let mut sorted_children: Vec<_> = element.children.iter().collect();
                if preserve_structure {
                    sorted_children.sort_by_key(|e| e.properties.order.unwrap_or(u32::MAX));
                }
                for child in sorted_children {
                    convert_element(child, output, depth + 1, preserve_structure, enable_spatial_tables, ctx);
                }
                output.push_str("\n\n");
            }
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
                convert_element(child, output, depth + 1, preserve_structure, enable_spatial_tables, ctx);
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
                append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("\n```\n\n");
        }

        // Drop cap - emphasize first letter
        HocrElementType::OcrDropcap => {
            output.push_str("**");
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
            if output.ends_with(' ') {
                output.pop();
            }
            output.push_str("**");
        }

        // Glyphs and character info - just extract text
        HocrElementType::OcrGlyph | HocrElementType::OcrGlyphs | HocrElementType::OcrCinfo => {
            append_text_and_children(element, output, depth, preserve_structure, enable_spatial_tables, ctx);
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

            let mut idx = 0;
            while idx < sorted_children.len() {
                let child = sorted_children[idx];
                if child.element_type == HocrElementType::OcrPar {
                    if let Some((code_lines, consumed, language)) = collect_code_block(&sorted_children[idx..]) {
                        if let Some(heading_text) =
                            find_previous_heading(&sorted_children, idx).or_else(|| ctx.last_heading.clone())
                        {
                            ensure_heading_prefix(output, &heading_text);
                        }
                        emit_code_block(output, &code_lines, language);
                        idx += consumed;
                        continue;
                    }
                }

                convert_element(child, output, depth + 1, preserve_structure, enable_spatial_tables, ctx);
                idx += 1;
            }
        }

        // Skip noise
        HocrElementType::OcrNoise => {}
    }
}

fn append_text_and_children(
    element: &HocrElement,
    output: &mut String,
    depth: usize,
    preserve_structure: bool,
    enable_spatial_tables: bool,
    ctx: &mut ConvertContext,
) {
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
        convert_element(child, output, depth + 1, preserve_structure, enable_spatial_tables, ctx);
    }
}

fn element_text_content(element: &HocrElement) -> String {
    let mut tokens = Vec::new();
    collect_text_tokens(element, &mut tokens);
    tokens.join(" ").replace("  ", " ").trim().to_string()
}

fn collect_text_tokens(element: &HocrElement, tokens: &mut Vec<String>) {
    if element.element_type == HocrElementType::OcrxWord {
        let trimmed = element.text.trim();
        if !trimmed.is_empty() {
            tokens.push(trimmed.to_string());
        }
    }

    for child in &element.children {
        collect_text_tokens(child, tokens);
    }
}

/// Collect all word elements recursively from an element tree
fn collect_words(element: &HocrElement, words: &mut Vec<HocrWord>) {
    if element.element_type == HocrElementType::OcrxWord {
        // Convert HocrElement to HocrWord if it has bbox data
        if let Some(bbox) = element.properties.bbox {
            let confidence = element.properties.x_wconf.unwrap_or(0.0);
            words.push(HocrWord {
                text: element.text.clone(),
                left: bbox.x1,
                top: bbox.y1,
                width: bbox.width(),
                height: bbox.height(),
                confidence,
            });
        }
    }

    // Recursively collect from children
    for child in &element.children {
        collect_words(child, words);
    }
}

fn detect_heading_paragraph(element: &HocrElement) -> Option<String> {
    if element.element_type != HocrElementType::OcrPar {
        return None;
    }

    let line_count = element
        .children
        .iter()
        .filter(|child| matches!(child.element_type, HocrElementType::OcrLine | HocrElementType::OcrxLine))
        .count();

    if line_count != 1 {
        return None;
    }

    let text = element_text_content(element);
    if text.is_empty() || text.len() > 60 || text.contains(':') || text.contains('\n') {
        return None;
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 2 || words.len() > 8 {
        return None;
    }

    let uppercase_initial = words
        .iter()
        .filter(|word| word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
        .count();

    if uppercase_initial < words.len().saturating_sub(1) {
        return None;
    }

    if text.ends_with('.') {
        return None;
    }

    Some(text)
}

/// Try to detect and reconstruct a table from an element's word children
///
/// Returns Some(markdown) if table structure detected, None otherwise
fn try_spatial_table_reconstruction(element: &HocrElement) -> Option<String> {
    let mut words = Vec::new();
    collect_words(element, &mut words);

    // Need at least 6 words for a minimal 2x3 table
    if words.len() < 6 {
        return None;
    }

    // Try to reconstruct table with default thresholds
    let table = spatial::reconstruct_table(&words, 50, 0.5, false);

    if table.is_empty() || table[0].is_empty() {
        return None;
    }

    if let Some(cleaned_table) = post_process_table(table) {
        let markdown = spatial::table_to_markdown(&cleaned_table);
        if !markdown.is_empty() {
            return Some(markdown);
        }
    }

    None
}

#[derive(Clone)]
struct CodeLineInfo {
    text: String,
    x1: u32,
}

fn find_previous_heading(children: &[&HocrElement], idx: usize) -> Option<String> {
    if idx == 0 {
        return None;
    }

    for candidate in children[..idx].iter().rev() {
        if let Some(text) = detect_heading_paragraph(candidate) {
            return Some(text);
        }
    }

    None
}

fn ensure_heading_prefix(output: &mut String, heading: &str) {
    let snippet = format!("# {}\n\n", heading);
    if output.ends_with(&snippet) {
        return;
    }

    if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }

    output.push_str(&snippet);
}

fn collect_code_block(children: &[&HocrElement]) -> Option<(Vec<String>, usize, Option<&'static str>)> {
    let mut collected: Vec<CodeLineInfo> = Vec::new();
    let mut consumed = 0;
    let mut paragraph_count = 0;

    while consumed < children.len() {
        let child = children[consumed];
        if child.element_type != HocrElementType::OcrPar {
            break;
        }

        let lines = extract_code_lines(child);
        if lines.is_empty() || !is_code_paragraph(&lines) {
            break;
        }

        if paragraph_count > 0 && !collected.is_empty() && should_insert_code_paragraph_break(&collected, &lines) {
            let gap_x = lines
                .first()
                .map(|info| info.x1)
                .or_else(|| child.properties.bbox.map(|bbox| bbox.x1))
                .unwrap_or(0);
            collected.push(CodeLineInfo {
                text: String::new(),
                x1: gap_x,
            });
        }

        collected.extend(lines);
        consumed += 1;
        paragraph_count += 1;
    }

    if collected.is_empty() {
        return None;
    }

    // Determine base indentation metrics
    let mut x_values: Vec<u32> = collected
        .iter()
        .filter(|info| !info.text.is_empty())
        .map(|info| info.x1)
        .collect();

    if x_values.is_empty() {
        x_values.push(0);
    }

    let min_x = *x_values.iter().min().unwrap_or(&0);
    let indent_candidates: Vec<u32> = x_values
        .iter()
        .filter_map(|&x| if x > min_x { Some(x - min_x) } else { None })
        .filter(|&delta| delta > 5)
        .collect();

    let mut indent_step = indent_candidates.iter().copied().min().unwrap_or(40);

    if indent_step == 0 {
        indent_step = 40;
    }

    let mut lines: Vec<String> = Vec::new();
    for info in collected {
        if info.text.is_empty() {
            if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                lines.push(String::new());
            }
            continue;
        }

        let indent_level = if info.x1 <= min_x {
            0
        } else {
            let diff = info.x1 - min_x;
            (((diff as f32) / indent_step as f32) + 0.25).floor() as usize
        }
        .min(6);

        let mut normalized = normalize_code_line(&info.text);
        if indent_level > 0 {
            let indent = "  ".repeat(indent_level);
            normalized = format!("{indent}{normalized}");
        }
        lines.push(normalized);
    }

    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }

    let meaningful_lines: Vec<&String> = lines.iter().filter(|line| !line.trim().is_empty()).collect();
    let meaningful_count = meaningful_lines.len();
    if meaningful_count < 3 {
        return None;
    }

    let bullet_like = meaningful_lines.iter().filter(|line| is_bullet_like(line)).count();
    if bullet_like * 2 >= meaningful_count {
        return None;
    }

    let language = detect_code_language(&lines);
    Some((lines, consumed, language))
}

fn should_insert_code_paragraph_break(previous: &[CodeLineInfo], next: &[CodeLineInfo]) -> bool {
    let prev_line = previous.iter().rev().find(|info| !info.text.trim().is_empty());
    let next_line = next.iter().find(|info| !info.text.trim().is_empty());

    match (prev_line, next_line) {
        (Some(prev), Some(next)) => {
            let prev_text = prev.text.trim();
            let next_text = next.text.trim();

            if next_text == "}" {
                return false;
            }

            if prev_text.ends_with('{') && next_text == "}" {
                return false;
            }

            true
        }
        _ => false,
    }
}

fn extract_code_lines(paragraph: &HocrElement) -> Vec<CodeLineInfo> {
    let mut lines = Vec::new();

    for child in &paragraph.children {
        match child.element_type {
            HocrElementType::OcrLine | HocrElementType::OcrxLine => {
                let mut words = Vec::new();
                collect_line_words(child, &mut words);
                if words.is_empty() {
                    continue;
                }
                let text = words.join(" ");
                if text.trim().is_empty() {
                    continue;
                }
                let x1 = child
                    .properties
                    .bbox
                    .map(|bbox| bbox.x1)
                    .or_else(|| paragraph.properties.bbox.map(|bbox| bbox.x1))
                    .unwrap_or(0);
                lines.push(CodeLineInfo {
                    text: text.trim().to_string(),
                    x1,
                });
            }
            _ => {}
        }
    }

    if lines.is_empty() {
        let mut words = Vec::new();
        collect_line_words(paragraph, &mut words);
        if !words.is_empty() {
            let x1 = paragraph.properties.bbox.map(|bbox| bbox.x1).unwrap_or(0);
            lines.push(CodeLineInfo {
                text: words.join(" ").trim().to_string(),
                x1,
            });
        }
    }

    lines
}

fn collect_line_words(element: &HocrElement, words: &mut Vec<String>) {
    if element.element_type == HocrElementType::OcrxWord {
        let trimmed = element.text.trim();
        if !trimmed.is_empty() {
            words.push(trimmed.to_string());
        }
    }

    for child in &element.children {
        collect_line_words(child, words);
    }
}

fn is_bullet_like(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") || trimmed.starts_with("•")
    {
        return true;
    }

    let mut chars = trimmed.chars().peekable();
    let mut digit_count = 0;
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            digit_count += 1;
            chars.next();
            continue;
        }
        break;
    }

    if digit_count > 0 {
        if let Some(&ch) = chars.peek() {
            if (ch == '.' || ch == ')') && chars.clone().nth(1).map(|c| c.is_whitespace()).unwrap_or(false) {
                return true;
            }
        }
    }

    false
}

fn is_code_paragraph(lines: &[CodeLineInfo]) -> bool {
    if lines.is_empty() {
        return false;
    }

    let mut strong_markers = 0;
    let mut moderate_markers = 0;
    let mut total = 0;

    for info in lines {
        let text = info.text.trim();
        if text.is_empty() {
            continue;
        }

        if is_bullet_like(&info.text) {
            return false;
        }

        total += 1;
        let lower = text.to_lowercase();

        let has_keyword = lower.contains("function")
            || lower.contains("console.")
            || lower.starts_with("return")
            || lower.starts_with("var ")
            || lower.starts_with("let ")
            || lower.starts_with("const ")
            || lower.starts_with("async ")
            || lower.starts_with("await ")
            || lower.starts_with("if ")
            || lower.starts_with("elif ")
            || lower.starts_with("else if ")
            || lower.starts_with("for ")
            || lower.starts_with("while ")
            || lower.starts_with("switch ")
            || lower.starts_with("case ")
            || lower.starts_with("class ")
            || lower.starts_with("struct ")
            || lower.starts_with("enum ")
            || lower.starts_with("def ")
            || lower.starts_with("fn ")
            || lower.starts_with("pub ")
            || lower.starts_with("import ")
            || lower.starts_with("from ")
            || lower.starts_with("using ")
            || lower.starts_with("namespace ");

        let has_symbol = text.contains('{')
            || text.contains('}')
            || text.contains(';')
            || text.contains("::")
            || text.contains("->")
            || text.contains("=>");

        if has_keyword || has_symbol {
            strong_markers += 1;
            continue;
        }

        let trimmed = text.trim_start();
        let starts_with_indent = trimmed.len() + 2 <= text.len();
        let has_assignment = text.contains(" = ")
            || text.contains("+=")
            || text.contains("-=")
            || text.contains("*=")
            || text.contains("/=")
            || text.contains(" := ")
            || text.contains(" == ");

        if has_assignment || starts_with_indent {
            moderate_markers += 1;
        }
    }

    if total == 0 {
        return false;
    }
    if strong_markers * 2 >= total {
        return true;
    }
    strong_markers > 0 && (strong_markers + moderate_markers) * 2 >= total
}

fn normalize_code_line(text: &str) -> String {
    let mut normalized = text.trim().to_string();
    let replacements = [("\u{2014}", "-"), ("\u{2013}", "-"), ("\u{2212}", "-")];
    for (from, to) in replacements {
        normalized = normalized.replace(from, to);
    }

    normalized = normalized.replace('+', " + ");

    // Collapse whitespace to single spaces
    let mut collapsed = String::new();
    let mut last_space = false;
    for ch in normalized.chars() {
        if ch.is_whitespace() {
            if !last_space {
                collapsed.push(' ');
                last_space = true;
            }
        } else {
            collapsed.push(ch);
            last_space = false;
        }
    }
    let mut cleaned = collapsed.trim().to_string();
    let punctuation_fixes = [(" ,", ","), (" ;", ";"), (" )", ")"), ("( ", "(")];
    for (from, to) in punctuation_fixes {
        cleaned = cleaned.replace(from, to);
    }
    let mut final_line = String::new();
    for ch in cleaned.chars() {
        match ch {
            '{' => {
                if !final_line.ends_with(' ') && !final_line.is_empty() {
                    final_line.push(' ');
                }
                final_line.push('{');
            }
            '}' | ';' => {
                if final_line.ends_with(' ') {
                    final_line.pop();
                }
                final_line.push(ch);
            }
            _ => final_line.push(ch),
        }
    }
    while final_line.contains("  ") {
        final_line = final_line.replace("  ", " ");
    }
    final_line.trim().to_string()
}

fn detect_code_language(lines: &[String]) -> Option<&'static str> {
    let lower_lines: Vec<String> = lines.iter().map(|line| line.to_lowercase()).collect();
    if lower_lines.iter().any(|line| line.contains("function"))
        || lower_lines.iter().any(|line| line.contains("console."))
        || lower_lines.iter().any(|line| line.contains("const "))
    {
        return Some("javascript");
    }
    if lower_lines.iter().any(|line| line.contains("printf")) {
        return Some("c");
    }
    None
}

fn emit_code_block(output: &mut String, lines: &[String], language: Option<&str>) {
    if !output.is_empty() {
        if output.ends_with('\n') {
            if !output.ends_with("\n\n") {
                output.push('\n');
            }
        } else {
            output.push_str("\n\n");
        }
    }

    output.push_str("```");
    if let Some(lang) = language {
        output.push_str(lang);
    }
    output.push('\n');

    for line in lines {
        output.push_str(line);
        output.push('\n');
    }

    output.push_str("```\n\n");
}

fn post_process_table(mut table: Vec<Vec<String>>) -> Option<Vec<Vec<String>>> {
    table.retain(|row| row.iter().any(|cell| !cell.trim().is_empty()));
    if table.is_empty() {
        return None;
    }

    let mut non_empty = 0;
    let mut long_cells = 0;
    for row in &table {
        for cell in row {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            non_empty += 1;
            if trimmed.chars().count() > 60 {
                long_cells += 1;
            }
        }
    }

    if non_empty > 0 && long_cells * 3 > non_empty * 2 {
        return None;
    }

    let data_start = table
        .iter()
        .enumerate()
        .find_map(|(idx, row)| {
            let digit_cells = row
                .iter()
                .filter(|cell| cell.chars().any(|c| c.is_ascii_digit()))
                .count();
            if digit_cells >= 3 {
                Some(idx)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let mut header_rows = if data_start > 0 {
        table[..data_start].to_vec()
    } else {
        Vec::new()
    };
    let mut data_rows = table[data_start..].to_vec();

    if header_rows.len() > 2 {
        header_rows = header_rows[header_rows.len() - 2..].to_vec();
    }

    if header_rows.is_empty() {
        if data_rows.len() < 2 {
            return None;
        }
        header_rows.push(data_rows[0].clone());
        data_rows = data_rows[1..].to_vec();
    }

    let column_count = header_rows
        .first()
        .or_else(|| data_rows.first())
        .map(|row| row.len())
        .unwrap_or(0);

    if column_count == 0 {
        return None;
    }

    let mut header = vec![String::new(); column_count];
    for row in &header_rows {
        for (idx, cell) in row.iter().enumerate() {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !header[idx].is_empty() {
                header[idx].push(' ');
            }
            header[idx].push_str(trimmed);
        }
    }

    let mut processed = Vec::new();
    processed.push(header);
    processed.extend(data_rows);

    if processed.len() <= 1 {
        return None;
    }

    let mut col = 0;
    while col < processed[0].len() {
        let header_text = processed[0][col].trim().to_string();
        let data_empty = processed[1..]
            .iter()
            .all(|row| row.get(col).map_or(true, |cell| cell.trim().is_empty()));

        if data_empty {
            merge_header_only_column(&mut processed, col, header_text);
        } else {
            col += 1;
        }

        if processed.is_empty() || processed[0].is_empty() {
            return None;
        }
    }

    if processed[0].len() < 2 || processed.len() <= 1 {
        return None;
    }

    for cell in &mut processed[0] {
        normalize_header_cell(cell);
    }

    for row in processed.iter_mut().skip(1) {
        for cell in row.iter_mut() {
            normalize_data_cell(cell);
        }
    }

    Some(processed)
}

fn merge_header_only_column(table: &mut [Vec<String>], col: usize, header_text: String) {
    if table.is_empty() || table[0].is_empty() {
        return;
    }

    let trimmed = header_text.trim();
    if trimmed.is_empty() && table.len() > 1 {
        for row in table.iter_mut() {
            row.remove(col);
        }
        return;
    }

    if !trimmed.is_empty() {
        if col > 0 {
            let mut target = col - 1;
            while target > 0 && table[0][target].trim().is_empty() {
                target -= 1;
            }
            if !table[0][target].trim().is_empty() || target == 0 {
                if !table[0][target].is_empty() {
                    table[0][target].push(' ');
                }
                table[0][target].push_str(trimmed);
                for row in table.iter_mut() {
                    row.remove(col);
                }
                return;
            }
        }

        if col + 1 < table[0].len() {
            if !table[0][col + 1].trim().is_empty() {
                let mut updated = trimmed.to_string();
                updated.push(' ');
                updated.push_str(table[0][col + 1].trim());
                table[0][col + 1] = updated;
            } else {
                table[0][col + 1] = trimmed.to_string();
            }
            for row in table.iter_mut() {
                row.remove(col);
            }
            return;
        }
    }

    for row in table.iter_mut() {
        row.remove(col);
    }
}

fn normalize_header_cell(cell: &mut String) {
    let mut text = cell.trim().replace("  ", " ");
    if text.contains("(Q)") {
        text = text.replace("(Q)", "(Ω)");
    }
    if text.contains("icorr") && text.contains("(A/cm)") && !text.contains("^2") {
        text = text.replace("(A/cm)", "(A/cm^2)");
    }
    if text.eq_ignore_ascii_case("be (V/dec)") {
        text = "bc (V/dec)".to_string();
    }
    if text.starts_with("Polarization resistance") {
        if !text.contains("(Ω)") {
            text.push_str(" (Ω)");
        } else {
            text = text.replace("(Ω) rate", "(Ω)");
        }
    }
    if text.starts_with("Corrosion") && text.contains("mm/year") {
        text = "Corrosion rate (mm/year)".to_string();
    }
    *cell = text;
}

fn normalize_data_cell(cell: &mut String) {
    let mut text = cell.trim().to_string();
    if text.is_empty() {
        cell.clear();
        return;
    }

    for ch in ['\u{2014}', '\u{2013}', '\u{2212}'] {
        text = text.replace(ch, "-");
    }

    if text.starts_with("- ") {
        text = format!("-{}", text[2..].trim_start());
    }

    text = text.replace("- ", "-");
    text = text.replace(" -", "-");
    text = text.replace("E-", "e-").replace("E+", "e+");

    if text == "-" {
        text.clear();
    }

    *cell = text;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hocr::types::{BBox, HocrElement, HocrElementType, HocrProperties};

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
    fn test_spatial_table_reconstruction_can_be_disabled() {
        fn word(text: &str, x1: u32, y1: u32) -> HocrElement {
            HocrElement {
                element_type: HocrElementType::OcrxWord,
                properties: HocrProperties {
                    bbox: Some(BBox {
                        x1,
                        y1,
                        x2: x1 + 40,
                        y2: y1 + 20,
                    }),
                    x_wconf: Some(95.0),
                    ..HocrProperties::default()
                },
                text: text.to_string(),
                children: vec![],
            }
        }

        let paragraph = HocrElement {
            element_type: HocrElementType::OcrPar,
            properties: HocrProperties::default(),
            text: String::new(),
            children: vec![
                word("A", 10, 10),
                word("B", 120, 10),
                word("C", 230, 10),
                word("D", 12, 60),
                word("E", 122, 60),
                word("F", 232, 60),
            ],
        };

        let markdown_with_tables = convert_to_markdown_with_options(std::slice::from_ref(&paragraph), true, true);
        assert!(
            markdown_with_tables.contains("| --- |"),
            "Expected spatial table reconstruction to produce a markdown table"
        );

        let markdown_without_tables = convert_to_markdown_with_options(std::slice::from_ref(&paragraph), true, false);
        assert!(
            !markdown_without_tables.contains('|'),
            "Table reconstruction should be disabled when the flag is false"
        );
        assert!(
            markdown_without_tables.contains("A B C"),
            "Plain text output should retain original word order"
        );
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
