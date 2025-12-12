use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_basic_unordered_list() {
    let html = r#"<ul>
    <li>Item 1</li>
    <li>Item 2</li>
    <li>Item 3</li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- Item 1"));
    assert!(result.contains("- Item 2"));
    assert!(result.contains("- Item 3"));
}

#[test]
fn test_basic_ordered_list() {
    let html = r#"<ol>
    <li>First</li>
    <li>Second</li>
    <li>Third</li>
    </ol>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("1. First"));
    assert!(result.contains("2. Second"));
    assert!(result.contains("3. Third"));
}

#[test]
fn test_nested_lists() {
    let html = r#"<ul>
    <li>Item 1
        <ul>
            <li>Nested 1</li>
            <li>Nested 2</li>
        </ul>
    </li>
    <li>Item 2</li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- Item 1"));
    assert!(result.contains("- Nested 1"));
    assert!(result.contains("- Nested 2"));
    assert!(result.contains("- Item 2"));
}

#[test]
fn test_ordered_nested_in_unordered() {
    let html = r#"<ul>
    <li>Outer item
        <ol>
            <li>Inner item 1</li>
            <li>Inner item 2</li>
        </ol>
    </li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- Outer item"));
    assert!(result.contains("1. Inner item 1"));
    assert!(result.contains("2. Inner item 2"));
}

#[test]
fn test_list_with_formatting() {
    let html = r#"<ul>
    <li><strong>Bold</strong> item</li>
    <li><em>Italic</em> item</li>
    <li><code>Code</code> item</li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- **Bold** item"));
    assert!(result.contains("- *Italic* item"));
    assert!(result.contains("- `Code` item"));
}

#[test]
fn test_list_with_links() {
    let html = r#"<ul>
    <li><a href="https://example.com">Link 1</a></li>
    <li><a href="https://example.org">Link 2</a></li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("[Link 1](https://example.com)"));
    assert!(result.contains("[Link 2](https://example.org)"));
}

#[test]
fn test_task_list() {
    let html = r#"<ul>
    <li><input type="checkbox" checked> Completed task</li>
    <li><input type="checkbox"> Incomplete task</li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- [x] Completed task"));
    assert!(result.contains("- [ ] Incomplete task"));
}

#[test]
fn test_list_indent_spaces() {
    let html = r#"<ul>
    <li>Parent
        <ul>
            <li>Child</li>
        </ul>
    </li>
    </ul>"#;

    let options = ConversionOptions {
        list_indent_type: html_to_markdown_rs::ListIndentType::Spaces,
        list_indent_width: 2,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert!(result.contains("- Parent"));
    assert!(result.contains("  - Child"));
}

#[test]
fn test_list_indent_tabs() {
    let html = r#"<ul>
    <li>Parent
        <ul>
            <li>Child</li>
        </ul>
    </li>
    </ul>"#;

    let options = ConversionOptions {
        list_indent_type: html_to_markdown_rs::ListIndentType::Tabs,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert!(result.contains("- Parent"));
    assert!(result.contains("\t- Child"));
}

#[test]
fn test_custom_bullet_symbols() {
    let html = r#"<ul>
    <li>Item 1</li>
    <li>Item 2</li>
    </ul>"#;

    let options = ConversionOptions {
        bullets: "*+-".to_string(),
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert!(result.contains("* Item 1") || result.contains("* Item 2"));
}

#[test]
fn test_empty_list_item() {
    let html = r#"<ul>
    <li>Item 1</li>
    <li></li>
    <li>Item 3</li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("- Item 1"));
    assert!(result.contains("- Item 3"));
}

#[test]
fn test_list_with_code_block() {
    let html = r#"<ul>
    <li>
        <p>Item with code:</p>
        <pre><code>fn main() {
    println!("Hello");
}</code></pre>
    </li>
    </ul>"#;

    let result = convert(html, None).unwrap();
    println!("Result:\n{}", result);
    assert!(result.contains("- Item with code:"));
    assert!(result.contains("fn main()"));
}
