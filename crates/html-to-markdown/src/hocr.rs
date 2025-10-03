//! hOCR table extraction and processing
//!
//! Extracts table structures from hOCR documents by analyzing word positions
//! from bbox (bounding box) attributes and reconstructing tabular layouts.

use markup5ever_rcdom::{Handle, NodeData};

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
fn parse_bbox(title: &str) -> Option<(u32, u32, u32, u32)> {
    for part in title.split(';') {
        let part = part.trim();
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
fn get_text_content(handle: &Handle) -> String {
    let mut text = String::new();

    match &handle.data {
        NodeData::Text { contents } => {
            text.push_str(&contents.borrow());
        }
        NodeData::Element { .. } => {
            for child in handle.children.borrow().iter() {
                text.push_str(&get_text_content(child));
            }
        }
        _ => {}
    }

    text
}

/// Extract hOCR words from a DOM tree
///
/// Walks the DOM and extracts all elements with `ocrx_word` class,
/// parsing their bbox and confidence information.
pub fn extract_hocr_words(handle: &Handle, min_confidence: f64) -> Vec<HocrWord> {
    let mut words = Vec::new();

    if let NodeData::Element { name, attrs, .. } = &handle.data {
        if name.local.as_ref() == "span" {
            let attrs = attrs.borrow();

            let mut is_word = false;
            let mut title = String::new();

            for attr in attrs.iter() {
                if attr.name.local.as_ref() == "class" {
                    let classes = attr.value.as_ref();
                    if classes.contains("ocrx_word") {
                        is_word = true;
                    }
                }
                if attr.name.local.as_ref() == "title" {
                    title = attr.value.to_string();
                }
            }

            if is_word {
                if let Some((left, top, width, height)) = parse_bbox(&title) {
                    let confidence = parse_confidence(&title);

                    if confidence >= min_confidence {
                        let text = get_text_content(handle).trim().to_string();

                        if !text.is_empty() {
                            words.push(HocrWord {
                                text,
                                left,
                                top,
                                width,
                                height,
                                confidence,
                            });
                        }
                    }
                }
            }
        }
    }

    for child in handle.children.borrow().iter() {
        words.extend(extract_hocr_words(child, min_confidence));
    }

    words
}

/// Detect column positions from word positions
///
/// Groups words by their x-position and returns the median x-position
/// for each detected column.
pub fn detect_columns(words: &[HocrWord], column_threshold: u32) -> Vec<u32> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut position_groups: Vec<Vec<u32>> = Vec::new();

    for word in words {
        let x_pos = word.left;

        let mut found_group = false;
        for group in &mut position_groups {
            if let Some(&first_pos) = group.first() {
                if x_pos.abs_diff(first_pos) <= column_threshold {
                    group.push(x_pos);
                    found_group = true;
                    break;
                }
            }
        }

        if !found_group {
            position_groups.push(vec![x_pos]);
        }
    }

    let mut columns: Vec<u32> = position_groups
        .iter()
        .filter(|group| !group.is_empty())
        .map(|group| {
            let mut sorted = group.clone();
            sorted.sort_unstable();
            let mid = sorted.len() / 2;
            sorted[mid]
        })
        .collect();

    columns.sort_unstable();
    columns
}

/// Detect row positions from word positions
///
/// Groups words by their vertical center position and returns the median
/// y-position for each detected row.
pub fn detect_rows(words: &[HocrWord], row_threshold_ratio: f64) -> Vec<u32> {
    if words.is_empty() {
        return Vec::new();
    }

    let mut heights: Vec<u32> = words.iter().map(|w| w.height).collect();
    heights.sort_unstable();
    let median_height = heights[heights.len() / 2];
    let row_threshold = (median_height as f64 * row_threshold_ratio) as u32;

    let mut position_groups: Vec<Vec<f64>> = Vec::new();

    for word in words {
        let y_center = word.y_center();

        let mut found_group = false;
        for group in &mut position_groups {
            if let Some(&first_pos) = group.first() {
                if (y_center - first_pos).abs() <= row_threshold as f64 {
                    group.push(y_center);
                    found_group = true;
                    break;
                }
            }
        }

        if !found_group {
            position_groups.push(vec![y_center]);
        }
    }

    let mut rows: Vec<u32> = position_groups
        .iter()
        .filter(|group| !group.is_empty())
        .map(|group| {
            let mut sorted = group.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = sorted.len() / 2;
            sorted[mid] as u32
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
pub fn reconstruct_table(words: &[HocrWord], column_threshold: u32, row_threshold_ratio: f64) -> Vec<Vec<String>> {
    if words.is_empty() {
        return Vec::new();
    }

    let col_positions = detect_columns(words, column_threshold);
    let row_positions = detect_rows(words, row_threshold_ratio);

    if col_positions.is_empty() || row_positions.is_empty() {
        return Vec::new();
    }

    let num_rows = row_positions.len();
    let num_cols = col_positions.len();
    let mut table: Vec<Vec<Vec<String>>> = vec![vec![vec![]; num_cols]; num_rows];

    for word in words {
        if let (Some(r), Some(c)) = (
            find_row_index(&row_positions, word),
            find_column_index(&col_positions, word),
        ) {
            if r < num_rows && c < num_cols {
                table[r][c].push(word.text.clone());
            }
        }
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
    use html5ever::parse_document;
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::RcDom;
    use std::io::Cursor;

    #[test]
    fn test_parse_bbox() {
        assert_eq!(parse_bbox("bbox 100 50 180 80"), Some((100, 50, 80, 30)));
        assert_eq!(parse_bbox("bbox 0 0 100 200"), Some((0, 0, 100, 200)));
        assert_eq!(parse_bbox("bbox 100 50 180 80; x_wconf 95"), Some((100, 50, 80, 30)));
        assert_eq!(parse_bbox("invalid"), None);
        assert_eq!(parse_bbox("bbox 100 50"), None);
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

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut Cursor::new(hocr.as_bytes()))
            .unwrap();

        let words = extract_hocr_words(&dom.document, 0.0);

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

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut Cursor::new(hocr.as_bytes()))
            .unwrap();

        let words = extract_hocr_words(&dom.document, 90.0);

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

        let table = reconstruct_table(&words, 50, 0.5);

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

        let table = reconstruct_table(&words, 50, 0.5);

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

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut Cursor::new(hocr.as_bytes()))
            .unwrap();

        let words = extract_hocr_words(&dom.document, 0.0);
        let table = reconstruct_table(&words, 50, 0.5);
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
