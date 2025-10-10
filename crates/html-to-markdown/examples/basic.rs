use html_to_markdown_rs::convert;

fn main() {
    let html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";

    match convert(html, None) {
        Ok(markdown) => {
            println!("HTML:");
            println!("{}", html);
            println!("\nMarkdown:");
            println!("{}", markdown);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
