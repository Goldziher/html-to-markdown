use html_to_markdown_rs::convert;

fn main() {
    let html = "<p>text    with    multiple    spaces</p>";
    match convert(html, None) {
        Ok(markdown) => {
            println!("Test - Multiple spaces:");
            println!("HTML: {}", html);
            println!("Markdown: {}", markdown);
            println!("Expected: text with multiple spaces");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html2 = "<p>text\nwith\nnewlines</p>";
    match convert(html2, None) {
        Ok(markdown) => {
            println!("Test - Newlines:");
            println!("HTML: {}", html2);
            println!("Markdown: {}", markdown);
            println!("Expected: text with newlines");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
