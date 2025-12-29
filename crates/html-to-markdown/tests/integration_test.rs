//! Integration tests for HTML to Markdown conversion.
//!
//! These tests verify end-to-end conversion of various HTML elements
//! to ensure correct Markdown output.

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_basic_paragraph() {
    let html = "<p>Hello world</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Hello world\n");
}

#[test]
fn test_multiple_paragraphs() {
    let html = "<p>First paragraph</p><p>Second paragraph</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "First paragraph\n\nSecond paragraph\n");
}

#[test]
fn test_atx_headings() {
    let html = "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6\n");
}

#[test]
fn test_bold() {
    let html = "<p>Text with <strong>bold</strong> word</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Text with **bold** word\n");
}

#[test]
fn test_italic() {
    let html = "<p>Text with <em>italic</em> word</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Text with *italic* word\n");
}

#[test]
fn test_bold_and_italic() {
    let html = "<p><strong><em>Bold and italic</em></strong></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "***Bold and italic***\n");
}

#[test]
fn test_inline_code() {
    let html = "<p>Use <code>code</code> here</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Use `code` here\n");
}

#[test]
fn test_code_block() {
    let html = "<pre><code>fn main() {\n    println!(\"Hello\");\n}</code></pre>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "    fn main() {\n        println!(\"Hello\");\n    }\n");
}

#[test]
fn test_simple_link() {
    let html = "<p><a href=\"https://example.com\">Link text</a></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "[Link text](https://example.com)\n");
}

#[test]
fn test_link_with_title() {
    let html = "<p><a href=\"/url\" title=\"title\">Link</a></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "[Link](/url \"title\")\n");
}

#[test]
fn test_simple_image() {
    let html = "<p><img src=\"image.jpg\" alt=\"Alt text\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Alt text](image.jpg)\n");
}

#[test]
fn test_graphic_with_url() {
    let html = "<p><graphic url=\"diagram.svg\" alt=\"Diagram\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Diagram](diagram.svg)\n");
}

#[test]
fn test_graphic_with_href() {
    let html = "<p><graphic href=\"image.png\" alt=\"Image\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Image](image.png)\n");
}

#[test]
fn test_graphic_with_xlink_href() {
    let html = "<p><graphic xlink:href=\"chart.svg\" alt=\"Chart\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Chart](chart.svg)\n");
}

#[test]
fn test_graphic_with_src() {
    let html = "<p><graphic src=\"fallback.jpg\" alt=\"Fallback\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Fallback](fallback.jpg)\n");
}

#[test]
fn test_graphic_with_filename_fallback() {
    let html = "<p><graphic url=\"image.png\" filename=\"my-image.png\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![my-image.png](image.png)\n");
}

#[test]
fn test_graphic_attribute_priority() {
    // url should take priority over href, xlink:href, src
    let html = "<p><graphic url=\"priority.svg\" href=\"second.svg\" xlink:href=\"third.svg\" src=\"fourth.svg\" alt=\"Priority\" /></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "![Priority](priority.svg)\n");
}

#[test]
fn test_unordered_list() {
    let html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "- Item 1\n- Item 2\n- Item 3\n");
}

#[test]
fn test_ordered_list() {
    let html = "<ol><li>First</li><li>Second</li><li>Third</li></ol>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "1. First\n2. Second\n3. Third\n");
}

#[test]
fn test_nested_lists() {
    let html = "<ul><li>Item 1<ul><li>Nested 1</li><li>Nested 2</li></ul></li><li>Item 2</li></ul>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("- Item 1"));
    assert!(result.contains("  - Nested 1"));
    assert!(result.contains("  - Nested 2"));
}

#[test]
fn test_blockquote() {
    let html = "<blockquote><p>Quoted text</p></blockquote>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "> Quoted text\n");
}

#[test]
fn test_nested_blockquote() {
    let html = "<blockquote><blockquote><p>Nested quote</p></blockquote></blockquote>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "> > Nested quote\n");
}

#[test]
fn test_horizontal_rule() {
    let html = "<hr>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "---\n");
}

#[test]
fn test_hr_after_paragraph_keeps_blank_line() {
    let html = "<p>paragraph</p><hr>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "paragraph\n\n---\n");
}

#[test]
fn test_comment_between_paragraphs() {
    let html = "<p>yes</p><!----><p>but no</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "yes\n\nbut no\n");
}

#[test]
fn test_line_break() {
    let html = "<p>Line 1<br>Line 2</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Line 1  \nLine 2\n");
}

#[test]
fn test_strikethrough() {
    let html = "<p><del>Deleted text</del></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "~~Deleted text~~\n");
}

#[test]
fn test_simple_table() {
    let html = "<table><tr><th>Header</th></tr><tr><td>Cell</td></tr></table>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header |"));
    assert!(result.contains("| --- |"));
    assert!(result.contains("| Cell |"));
}

#[test]
fn test_table_rowspan() {
    let html = r#"<table>
<tr><th>Header 1</th><th>Header 2</th></tr>
<tr><td rowspan="2">Spanning cell</td><td>
    <div>First row content</div>
    <div>Second line</div>
</td></tr>
<tr><td>
    <div>Next row</div>
    <div>More content</div>
</td></tr>
</table>"#;
    let options = ConversionOptions {
        br_in_tables: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();
    let expected = "\n\n| Header 1 | Header 2 |\n| --- | --- |\n| Spanning cell | First row content<br>Second line |\n|  | Next row<br>More content |\n";
    assert_eq!(result, expected);
}

#[test]
fn test_empty_element() {
    let html = "<p></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_whitespace_normalization() {
    let html = "<p>Multiple    spaces    here</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Multiple spaces here\n");
}

#[test]
fn test_unicode_content() {
    let html = "<p>Hello 世界 🌍</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Hello 世界 🌍\n");
}

#[test]
fn test_html_entities() {
    let html = "<p>&lt;div&gt; &amp; &quot;quotes&quot;</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "<div> & \"quotes\"\n");
}

#[test]
fn test_nested_formatting() {
    let html = "<p><strong>Bold <em>and italic</em> text</strong></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "**Bold *and italic* text**\n");
}

#[test]
fn test_link_inside_paragraph() {
    let html = "<p>Check out <a href=\"https://example.com\">this link</a> for more info.</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Check out [this link](https://example.com) for more info.\n");
}

#[test]
fn test_code_with_special_chars() {
    let html = "<code>&lt;html&gt;</code>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "`<html>`\n");
}

#[test]
fn test_empty_link() {
    let html = "<p><a href=\"\">Empty</a></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "[Empty](<>)\n");
}

#[test]
fn test_div_as_block() {
    let html = "<div>Block content</div>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Block content\n");
}

#[test]
fn test_multiple_divs() {
    let html = "<div>First</div><div>Second</div>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "First\n\nSecond\n");
}

#[test]
fn test_span_inline() {
    let html = "<p>Text with <span>span</span> element</p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Text with span element\n");
}

#[test]
fn test_subscript() {
    let html = "<p>H<sub>2</sub>O</p>";
    let opts = ConversionOptions {
        sub_symbol: "~".to_string(),
        ..Default::default()
    };
    let result = convert(html, Some(opts)).unwrap();
    assert_eq!(result, "H~2~O\n");
}

#[test]
fn test_superscript() {
    let html = "<p>x<sup>2</sup></p>";
    let opts = ConversionOptions {
        sup_symbol: "^".to_string(),
        ..Default::default()
    };
    let result = convert(html, Some(opts)).unwrap();
    assert_eq!(result, "x^2^\n");
}

#[test]
fn test_autolink() {
    let html = "<p><a href=\"https://example.com\">https://example.com</a></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "<https://example.com>\n");
}

#[test]
fn test_email_autolink() {
    let html = "<p><a href=\"mailto:test@example.com\">test@example.com</a></p>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "<test@example.com>\n");
}

#[test]
fn test_metadata_extraction() {
    let html = "<html><head><title>Page Title</title></head><body><p>Content</p></body></html>";
    let opts = ConversionOptions {
        extract_metadata: true,
        ..Default::default()
    };
    let result = convert(html, Some(opts)).unwrap();
    assert!(result.contains("Page Title"));
    assert!(result.contains("Content"));
}

#[test]
fn test_metadata_disabled() {
    let html = "<html><head><title>Page Title</title></head><body><p>Content</p></body></html>";
    let opts = ConversionOptions {
        extract_metadata: false,
        ..Default::default()
    };
    let result = convert(html, Some(opts)).unwrap();
    assert!(!result.contains("<!--"));
    assert!(result.contains("Content"));
}

#[test]
fn test_task_list() {
    let html = "<ul><li><input type=\"checkbox\" checked> Done</li><li><input type=\"checkbox\"> Todo</li></ul>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("- [x] Done"));
    assert!(result.contains("- [ ] Todo"));
}

#[test]
fn test_definition_list() {
    let html = "<dl><dt>Term</dt><dd>Definition</dd></dl>";
    let result = convert(html, None).unwrap();
    assert!(result.contains("Term"));
    assert!(result.contains("Definition"));
}

#[test]
fn test_malformed_html() {
    let html = "<p>Unclosed paragraph<p>Another";
    let result = convert(html, None);
    assert!(result.is_ok());
}

#[test]
fn test_deeply_nested_structure() {
    let html = "<div><div><div><div><p>Deeply nested</p></div></div></div></div>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "Deeply nested\n");
}

#[test]
fn test_mixed_content() {
    let html = r#"
        <h1>Title</h1>
        <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
        <ul>
            <li>List item 1</li>
            <li>List item 2</li>
        </ul>
        <p>Link: <a href="https://example.com">Example</a></p>
    "#;
    let result = convert(html, None).unwrap();
    assert!(result.contains("# Title"));
    assert!(result.contains("**bold**"));
    assert!(result.contains("*italic*"));
    assert!(result.contains("- List item 1"));
    assert!(result.contains("[Example](https://example.com)"));
}

#[test]
fn test_ordered_list_with_heading_and_table() {
    let html = r#"
<ol>
  <li>
    <h3>h3</h3>
  </li>
  <li>
    <table>
      <caption>table</caption>
      <tr>
        <td>blah</td>
      </tr>
    </table>
  </li>
</ol>
"#;

    let result = convert(html, None).unwrap();
    let expected = "1. ### h3\n2. *table*\n\n    | blah |\n    | --- |\n";
    assert_eq!(result, expected);
}

#[test]
fn test_heading_wrapped_in_link_issue_115() {
    let html = r#"<a href="https://domain.local"><h2>Heading A</h2></a>"#;
    let result = convert(html, None).unwrap();
    assert_eq!(result, "## [Heading A](https://domain.local)\n");
}

#[test]
fn test_link_text_escaping_issue_114() {
    let html = r#"<a href="https://domain.local">Hi :]</a><br><a href="https://domain.local">1<2</a>"#;
    let result = convert(html, None).unwrap();
    assert_eq!(
        result,
        "[Hi :\\]](https://domain.local)  \n[1<2](https://domain.local)\n"
    );
}

#[test]
fn test_uppercase_tags_issue_113() {
    let html = r#"<B>Foo<Br />Bar</B>"#;
    let result = convert(html, None).unwrap();
    assert_eq!(result, "**Foo  \nBar**\n");
}

#[test]
fn test_breaks_and_newlines_issue_112() {
    let html = "<br>\n1\n2\n<b>3</b>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "\n1\n2\n**3**\n");
}

#[test]
fn test_nested_bold_issue_111() {
    let html = "<b>bold<b>er</b></b>";
    let result = convert(html, None).unwrap();
    assert_eq!(result, "**bolder**\n");
}
