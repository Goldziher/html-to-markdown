use html_to_markdown_rs::convert;

fn main() {
    let html = r#"<ul>
        <li><input type="checkbox"> Unchecked task</li>
        <li><input type="checkbox" checked> Checked task</li>
        <li>Regular list item</li>
    </ul>"#;

    match convert(html, None) {
        Ok(markdown) => {
            println!("Test - Task list:");
            println!("HTML: {}", html);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected:");
            println!("- [ ] Unchecked task");
            println!("- [x] Checked task");
            println!("- Regular list item");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html2 = r#"<ul>
        <li><input type="checkbox"> Parent task
            <ul>
                <li><input type="checkbox" checked> Child task</li>
            </ul>
        </li>
    </ul>"#;

    match convert(html2, None) {
        Ok(markdown) => {
            println!("Test - Nested task list:");
            println!("HTML: {}", html2);
            println!("\nMarkdown:\n{}", markdown);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html3 = r#"<ruby>東京<rt>とうきょう</rt></ruby>"#;

    match convert(html3, None) {
        Ok(markdown) => {
            println!("Test - Ruby annotation:");
            println!("HTML: {}", html3);
            println!("Markdown: {}", markdown);
            println!("Expected: 東京 (とうきょう)");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
