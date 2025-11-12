use html_to_markdown_rs::{ConversionOptions, HeadingStyle, convert};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = ConversionOptions::default();
    options.heading_style = HeadingStyle::Atx;

    let html = "<h1>Rust Smoke Test</h1><p>Validates the crates.io release.</p>";
    let markdown = convert(html, Some(options))?;

    if !markdown.contains("# Rust Smoke Test") {
        panic!("html-to-markdown-rs did not render the expected heading");
    }

    println!("✓ html-to-markdown-rs produced markdown\n{}", markdown);
    Ok(())
}
