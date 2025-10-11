//! Spatial table reconstruction from hOCR bounding box coordinates
//!
//! This module provides functions to detect and reconstruct tabular data from OCR'd text
//! by analyzing the spatial positions of words using their bounding box (bbox) coordinates.

/// Represents a word extracted from hOCR with position and confidence information
#[derive(Debug, Clone)]
pub struct HocrWord {
    pub text: String,
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f64,
}

impl HocrWord {
    /// Get the right edge position
    pub fn right(&self) -> u32 {
        self.left + self.width
    }

    /// Get the bottom edge position
    pub fn bottom(&self) -> u32 {
        self.top + self.height
    }

    /// Get the vertical center position
    pub fn y_center(&self) -> f64 {
        self.top as f64 + (self.height as f64 / 2.0)
    }

    /// Get the horizontal center position
    pub fn x_center(&self) -> f64 {
        self.left as f64 + (self.width as f64 / 2.0)
    }
}

/// Parse bbox attribute from hOCR title attribute
///
/// Example: "bbox 100 50 180 80; x_wconf 95" -> (100, 50, 80, 30)
fn parse_bbox(title: &str, debug: bool) -> Option<(u32, u32, u32, u32)> {
    let known_attributes = [
        "bbox",
        "x_wconf",
        "baseline",
        "x_size",
        "x_descenders",
        "x_ascenders",
        "textangle",
        "poly",
        "order",
        "x_font",
        "x_fsize",
        "x_confs",
    ];

    for part in title.split(';') {
        let part = part.trim();

        if debug && !part.is_empty() {
            let attr_name = part.split_whitespace().next().unwrap_or("");
            if !known_attributes.iter().any(|&k| part.starts_with(k)) {
                eprintln!("[hOCR] Info: Found unknown title attribute: '{}'", attr_name);
            }
        }

        if let Some(bbox_str) = part.strip_prefix("bbox ") {
            let coords: Vec<&str> = bbox_str.split_whitespace().collect();
            if coords.len() == 4 {
                if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                    coords[0].parse::<u32>(),
                    coords[1].parse::<u32>(),
                    coords[2].parse::<u32>(),
                    coords[3].parse::<u32>(),
                ) {
                    let width = x2.saturating_sub(x1);
                    let height = y2.saturating_sub(y1);
                    return Some((x1, y1, width, height));
                }
            }
        }
    }
    None
}

/// Parse confidence from hOCR title attribute
///
/// Example: "bbox 100 50 180 80; x_wconf 95" -> 95.0
fn parse_confidence(title: &str) -> f64 {
    for part in title.split(';') {
        let part = part.trim();
        if let Some(conf_str) = part.strip_prefix("x_wconf ") {
            if let Ok(conf) = conf_str.trim().parse::<f64>() {
                return conf;
            }
        }
    }
    0.0
}

/// Extract text content from a node
fn get_text_content(node_handle: &tl::NodeHandle, parser: &tl::Parser) -> String {
    let mut text = String::new();

    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Raw(bytes) => {
                text.push_str(&bytes.as_utf8_str());
            }
            tl::Node::Tag(tag) => {
                let children = tag.children();
                for child_handle in children.top().iter() {
                    text.push_str(&get_text_content(child_handle, parser));
                }
            }
            tl::Node::Comment(_) => {}
        }
    }

    text
}

/// Extract hOCR words from a DOM tree
///
/// Walks the DOM and extracts all elements with `ocrx_word` class,
/// parsing their bbox and confidence information.
pub fn extract_hocr_words(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    min_confidence: f64,
    debug: bool,
) -> Vec<HocrWord> {
    let mut words = Vec::new();

    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let tag_name = tag.name().as_utf8_str();
        let attrs = tag.attributes();

        let class_attr = attrs.get("class").flatten().map(|v| v.as_utf8_str().to_string());

        if let Some(ref classes) = class_attr {
            let known_classes = [
                "ocr_page",
                "ocr_carea",
                "ocr_par",
                "ocr_line",
                "ocrx_word",
                "ocr_header",
                "ocr_footer",
                "ocr_table",
                "ocr_caption",
                "ocr_textfloat",
                "ocr_separator",
                "ocr_noise",
            ];

            let class_list: Vec<&str> = classes.split_whitespace().collect();
            let has_ocr_class = class_list.iter().any(|c| c.starts_with("ocr"));

            if has_ocr_class && debug {
                for class in &class_list {
                    if class.starts_with("ocr") && !known_classes.contains(class) {
                        eprintln!("[hOCR] Info: Found unhandled hOCR class '{}' on <{}>", class, tag_name);
                    }
                }
            }
        }

        if tag_name == "span" {
            let is_word = class_attr.as_ref().is_some_and(|c| c.contains("ocrx_word"));
            let title = attrs.get("title").flatten().map(|v| v.as_utf8_str());

            if is_word {
                let title_str = title.as_deref().unwrap_or("");
                if let Some((left, top, width, height)) = parse_bbox(title_str, debug) {
                    let confidence = parse_confidence(title_str);

                    if confidence >= min_confidence {
                        let text = get_text_content(node_handle, parser).trim().to_string();

                        if !text.is_empty() {
                            words.push(HocrWord {
                                text,
                                left,
                                top,
                                width,
                                height,
                                confidence,
                            });
                        } else if debug {
                            eprintln!(
                                "[hOCR] Warning: ocrx_word element has no text content (bbox: {})",
                                title_str
                            );
                        }
                    } else if debug {
                        eprintln!(
                            "[hOCR] Warning: Word confidence ({:.1}) below threshold ({:.1}): {}",
                            confidence,
                            min_confidence,
                            get_text_content(node_handle, parser).trim()
                        );
                    }
                } else if debug {
                    let text = get_text_content(node_handle, parser);
                    let trimmed = text.trim();
                    eprintln!(
                        "[hOCR] Warning: Failed to parse bbox for ocrx_word element: {} (title: {})",
                        if trimmed.is_empty() { "<empty>" } else { trimmed },
                        title_str
                    );
                }
            }
        }

        let children = tag.children();
        for child_handle in children.top().iter() {
            words.extend(extract_hocr_words(child_handle, parser, min_confidence, debug));
        }
    }

    words
}

/// Detect column positions from word positions
///
/// Groups words by their x-position and returns the median x-position
/// for each detected column.
///
/// Optimized with O(n log n) complexity using sorted insertion.
pub fn detect_columns(words: &[HocrWord], column_threshold: u32) -> Vec<u32> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut x_positions: Vec<u32> = words.iter().map(|w| w.left).collect();
    x_positions.sort_unstable();

    let mut position_groups: Vec<Vec<u32>> = Vec::new();
    let mut current_group = vec![x_positions[0]];

    for &x_pos in &x_positions[1..] {
        let matches_group = current_group.iter().any(|&pos| x_pos.abs_diff(pos) <= column_threshold);

        if matches_group {
            current_group.push(x_pos);
        } else {
            position_groups.push(std::mem::replace(&mut current_group, vec![x_pos]));
        }
    }

    if !current_group.is_empty() {
        position_groups.push(current_group);
    }

    let mut columns: Vec<u32> = position_groups
        .iter()
        .map(|group| {
            let mid = group.len() / 2;
            group[mid]
        })
        .collect();

    columns.sort_unstable();
    columns
}

/// Detect row positions from word positions
///
/// Groups words by their vertical center position and returns the median
/// y-position for each detected row.
///
/// Optimized with O(n log n) complexity using sorted insertion.
pub fn detect_rows(words: &[HocrWord], row_threshold_ratio: f64) -> Vec<u32> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut heights: Vec<u32> = words.iter().map(|w| w.height).collect();
    heights.sort_unstable();
    let median_height = heights[heights.len() / 2];
    let row_threshold = median_height as f64 * row_threshold_ratio;

    let mut y_centers: Vec<f64> = words.iter().map(|w| w.y_center()).collect();
    y_centers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut position_groups: Vec<Vec<f64>> = Vec::new();
    let mut current_group = vec![y_centers[0]];

    for &y_center in &y_centers[1..] {
        let matches_group = current_group.iter().any(|&pos| (y_center - pos).abs() <= row_threshold);

        if matches_group {
            current_group.push(y_center);
        } else {
            position_groups.push(std::mem::replace(&mut current_group, vec![y_center]));
        }
    }

    if !current_group.is_empty() {
        position_groups.push(current_group);
    }

    let mut rows: Vec<u32> = position_groups
        .iter()
        .map(|group| {
            let mid = group.len() / 2;
            group[mid] as u32
        })
        .collect();

    rows.sort_unstable();
    rows
}

/// Reconstruct table structure from words
///
/// Takes detected words and reconstructs a 2D table by:
/// 1. Detecting column and row positions
/// 2. Assigning words to cells based on position
/// 3. Combining words within the same cell
pub fn reconstruct_table(
    words: &[HocrWord],
    column_threshold: u32,
    row_threshold_ratio: f64,
    debug: bool,
) -> Vec<Vec<String>> {
    if words.is_empty() {
        if debug {
            eprintln!("[hOCR] Warning: No words to reconstruct table from");
        }
        return Vec::new();
    }

    let col_positions = detect_columns(words, column_threshold);
    let row_positions = detect_rows(words, row_threshold_ratio);

    if col_positions.is_empty() || row_positions.is_empty() {
        if debug {
            eprintln!(
                "[hOCR] Warning: Could not detect table structure (columns: {}, rows: {})",
                col_positions.len(),
                row_positions.len()
            );
        }
        return Vec::new();
    }

    if debug {
        eprintln!(
            "[hOCR] Detected table structure: {} rows × {} columns",
            row_positions.len(),
            col_positions.len()
        );
    }

    let num_rows = row_positions.len();
    let num_cols = col_positions.len();
    let mut table: Vec<Vec<Vec<String>>> = vec![vec![vec![]; num_cols]; num_rows];
    let mut unassigned_words = 0;

    for word in words {
        if let (Some(r), Some(c)) = (
            find_row_index(&row_positions, word),
            find_column_index(&col_positions, word),
        ) {
            if r < num_rows && c < num_cols {
                table[r][c].push(word.text.clone());
            } else {
                unassigned_words += 1;
                if debug {
                    eprintln!(
                        "[hOCR] Warning: Word '{}' assigned to out-of-bounds cell ({}, {})",
                        word.text, r, c
                    );
                }
            }
        } else {
            unassigned_words += 1;
            if debug {
                eprintln!(
                    "[hOCR] Warning: Could not assign word '{}' to any cell (position: {}, {})",
                    word.text, word.left, word.top
                );
            }
        }
    }

    if debug && unassigned_words > 0 {
        eprintln!(
            "[hOCR] Warning: {} out of {} words could not be assigned to table cells",
            unassigned_words,
            words.len()
        );
    }

    let result: Vec<Vec<String>> = table
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell_words| {
                    if cell_words.is_empty() {
                        String::new()
                    } else {
                        cell_words.join(" ")
                    }
                })
                .collect()
        })
        .collect();

    remove_empty_rows_and_columns(result)
}

/// Find which row a word belongs to based on its y-center
fn find_row_index(row_positions: &[u32], word: &HocrWord) -> Option<usize> {
    let y_center = word.y_center() as u32;

    row_positions
        .iter()
        .enumerate()
        .min_by_key(|&(_, row_y)| row_y.abs_diff(y_center))
        .map(|(idx, _)| idx)
}

/// Find which column a word belongs to based on its x-position
fn find_column_index(col_positions: &[u32], word: &HocrWord) -> Option<usize> {
    let x_pos = word.left;

    col_positions
        .iter()
        .enumerate()
        .min_by_key(|&(_, col_x)| col_x.abs_diff(x_pos))
        .map(|(idx, _)| idx)
}

/// Remove empty rows and columns from table
fn remove_empty_rows_and_columns(table: Vec<Vec<String>>) -> Vec<Vec<String>> {
    if table.is_empty() {
        return table;
    }

    let num_cols = table[0].len();
    let mut non_empty_cols: Vec<bool> = vec![false; num_cols];

    for row in &table {
        for (col_idx, cell) in row.iter().enumerate() {
            if !cell.trim().is_empty() {
                non_empty_cols[col_idx] = true;
            }
        }
    }

    table
        .into_iter()
        .filter(|row| row.iter().any(|cell| !cell.trim().is_empty()))
        .map(|row| {
            row.into_iter()
                .enumerate()
                .filter(|(idx, _)| non_empty_cols[*idx])
                .map(|(_, cell)| cell)
                .collect()
        })
        .collect()
}

/// Convert table to markdown format
pub fn table_to_markdown(table: &[Vec<String>]) -> String {
    if table.is_empty() {
        return String::new();
    }

    let num_cols = table[0].len();
    if num_cols == 0 {
        return String::new();
    }

    let mut markdown = String::new();

    for (row_idx, row) in table.iter().enumerate() {
        markdown.push('|');
        for cell in row {
            markdown.push(' ');
            markdown.push_str(&cell.replace('|', "\\|"));
            markdown.push_str(" |");
        }
        markdown.push('\n');

        if row_idx == 0 {
            markdown.push('|');
            for _ in 0..num_cols {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');
        }
    }

    markdown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bbox() {
        assert_eq!(parse_bbox("bbox 100 50 180 80", false), Some((100, 50, 80, 30)));
        assert_eq!(parse_bbox("bbox 0 0 100 200", false), Some((0, 0, 100, 200)));
        assert_eq!(
            parse_bbox("bbox 100 50 180 80; x_wconf 95", false),
            Some((100, 50, 80, 30))
        );
        assert_eq!(parse_bbox("invalid", false), None);
        assert_eq!(parse_bbox("bbox 100 50", false), None);
    }

    #[test]
    fn test_parse_confidence() {
        assert_eq!(parse_confidence("x_wconf 95.5"), 95.5);
        assert_eq!(parse_confidence("bbox 100 50 180 80; x_wconf 92"), 92.0);
        assert_eq!(parse_confidence("invalid"), 0.0);
    }

    #[test]
    fn test_hocr_word_methods() {
        let word = HocrWord {
            text: "Hello".to_string(),
            left: 100,
            top: 50,
            width: 80,
            height: 30,
            confidence: 95.5,
        };

        assert_eq!(word.right(), 180);
        assert_eq!(word.bottom(), 80);
        assert_eq!(word.y_center(), 65.0);
        assert_eq!(word.x_center(), 140.0);
    }

    #[test]
    fn test_detect_columns() {
        let words = vec![
            HocrWord {
                text: "A".to_string(),
                left: 100,
                top: 50,
                width: 20,
                height: 30,
                confidence: 95.0,
            },
            HocrWord {
                text: "B".to_string(),
                left: 200,
                top: 50,
                width: 20,
                height: 30,
                confidence: 95.0,
            },
            HocrWord {
                text: "C".to_string(),
                left: 105,
                top: 100,
                width: 20,
                height: 30,
                confidence: 95.0,
            },
        ];

        let columns = detect_columns(&words, 50);
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&100) || columns.contains(&105));
        assert!(columns.contains(&200));
    }

    #[test]
    fn test_table_to_markdown() {
        let table = vec![
            vec!["Header1".to_string(), "Header2".to_string()],
            vec!["Cell1".to_string(), "Cell2".to_string()],
        ];

        let markdown = table_to_markdown(&table);
        assert!(markdown.contains("| Header1 | Header2 |"));
        assert!(markdown.contains("| --- | --- |"));
        assert!(markdown.contains("| Cell1 | Cell2 |"));
    }

    #[test]
    fn test_table_to_markdown_escape_pipes() {
        let table = vec![vec!["A|B".to_string(), "C".to_string()]];

        let markdown = table_to_markdown(&table);
        assert!(markdown.contains("A\\|B"));
    }

    #[test]
    fn test_extract_hocr_words() {
        let hocr = r#"
            <div class="ocr_page">
                <span class="ocrx_word" title="bbox 100 50 150 80; x_wconf 95">Hello</span>
                <span class="ocrx_word" title="bbox 160 50 210 80; x_wconf 92">World</span>
            </div>
        "#;

        let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut words = Vec::new();
        for child_handle in dom.children().iter() {
            words.extend(extract_hocr_words(child_handle, parser, 0.0, false));
        }

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "Hello");
        assert_eq!(words[0].left, 100);
        assert_eq!(words[0].confidence, 95.0);

        assert_eq!(words[1].text, "World");
        assert_eq!(words[1].left, 160);
        assert_eq!(words[1].confidence, 92.0);
    }

    #[test]
    fn test_extract_hocr_words_confidence_filter() {
        let hocr = r#"
            <div class="ocr_page">
                <span class="ocrx_word" title="bbox 100 50 150 80; x_wconf 95">HighConf</span>
                <span class="ocrx_word" title="bbox 160 50 210 80; x_wconf 50">LowConf</span>
                <span class="ocrx_word" title="bbox 220 50 270 80; x_wconf 98">VeryHigh</span>
            </div>
        "#;

        let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut words = Vec::new();
        for child_handle in dom.children().iter() {
            words.extend(extract_hocr_words(child_handle, parser, 90.0, false));
        }

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "HighConf");
        assert_eq!(words[1].text, "VeryHigh");
    }

    #[test]
    fn test_reconstruct_simple_table() {
        let words = vec![
            HocrWord {
                text: "Name".to_string(),
                left: 100,
                top: 50,
                width: 50,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "Age".to_string(),
                left: 200,
                top: 50,
                width: 50,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "Alice".to_string(),
                left: 100,
                top: 100,
                width: 50,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "30".to_string(),
                left: 200,
                top: 100,
                width: 50,
                height: 20,
                confidence: 95.0,
            },
        ];

        let table = reconstruct_table(&words, 50, 0.5, false);

        assert_eq!(table.len(), 2);
        assert_eq!(table[0].len(), 2);
        assert_eq!(table[0][0], "Name");
        assert_eq!(table[0][1], "Age");
        assert_eq!(table[1][0], "Alice");
        assert_eq!(table[1][1], "30");
    }

    #[test]
    fn test_reconstruct_table_with_multi_word_cells() {
        let words = vec![
            HocrWord {
                text: "First".to_string(),
                left: 100,
                top: 50,
                width: 30,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "Name".to_string(),
                left: 135,
                top: 50,
                width: 30,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "Last".to_string(),
                left: 200,
                top: 50,
                width: 30,
                height: 20,
                confidence: 95.0,
            },
            HocrWord {
                text: "Name".to_string(),
                left: 235,
                top: 50,
                width: 30,
                height: 20,
                confidence: 95.0,
            },
        ];

        let table = reconstruct_table(&words, 50, 0.5, false);

        assert_eq!(table.len(), 1);
        assert_eq!(table[0].len(), 2);
        assert_eq!(table[0][0], "First Name");
        assert_eq!(table[0][1], "Last Name");
    }

    #[test]
    fn test_end_to_end_hocr_table_extraction() {
        let hocr = r#"
            <div class="ocr_page">
                <span class="ocrx_word" title="bbox 100 50 140 70; x_wconf 95">Product</span>
                <span class="ocrx_word" title="bbox 200 50 240 70; x_wconf 95">Price</span>
                <span class="ocrx_word" title="bbox 100 100 140 120; x_wconf 95">Apple</span>
                <span class="ocrx_word" title="bbox 200 100 240 120; x_wconf 95">$1.50</span>
                <span class="ocrx_word" title="bbox 100 150 140 170; x_wconf 95">Orange</span>
                <span class="ocrx_word" title="bbox 200 150 240 170; x_wconf 95">$2.00</span>
            </div>
        "#;

        let dom = tl::parse(hocr, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut words = Vec::new();
        for child_handle in dom.children().iter() {
            words.extend(extract_hocr_words(child_handle, parser, 0.0, false));
        }

        let table = reconstruct_table(&words, 50, 0.5, false);
        let markdown = table_to_markdown(&table);

        assert_eq!(table.len(), 3);
        assert_eq!(table[0][0], "Product");
        assert_eq!(table[0][1], "Price");
        assert_eq!(table[1][0], "Apple");
        assert_eq!(table[1][1], "$1.50");
        assert_eq!(table[2][0], "Orange");
        assert_eq!(table[2][1], "$2.00");

        assert!(markdown.contains("| Product | Price |"));
        assert!(markdown.contains("| Apple | $1.50 |"));
        assert!(markdown.contains("| Orange | $2.00 |"));
    }
}
