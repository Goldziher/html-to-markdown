use html_to_markdown::{convert, ConversionOptions};

fn main() {
    // Test mark with default (DoubleEqual)
    let html = "<p>This is <mark>highlighted</mark> text</p>";
    match convert(html, None) {
        Ok(markdown) => {
            println!("Test - Mark (default):");
            println!("HTML: {}", html);
            println!("Markdown: {}", markdown);
            println!("Expected: This is ==highlighted== text");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test del and s tags
    let html2 = "<p>This is <del>deleted</del> and <s>strikethrough</s> text</p>";
    match convert(html2, None) {
        Ok(markdown) => {
            println!("Test - Del/Strike:");
            println!("HTML: {}", html2);
            println!("Markdown: {}", markdown);
            println!("Expected: This is ~~deleted~~ and ~~strikethrough~~ text");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test ins tag
    let html3 = "<p>This is <ins>inserted</ins> text</p>";
    match convert(html3, None) {
        Ok(markdown) => {
            println!("Test - Ins:");
            println!("HTML: {}", html3);
            println!("Markdown: {}", markdown);
            println!("Expected: This is ==inserted== text");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test kbd and samp
    let html4 = "<p>Press <kbd>Ctrl+C</kbd> and see <samp>output</samp></p>";
    match convert(html4, None) {
        Ok(markdown) => {
            println!("Test - Kbd/Samp:");
            println!("HTML: {}", html4);
            println!("Markdown: {}", markdown);
            println!("Expected: Press `Ctrl+C` and see `output`");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test var
    let html5 = "<p>The variable <var>x</var> is defined</p>";
    match convert(html5, None) {
        Ok(markdown) => {
            println!("Test - Var:");
            println!("HTML: {}", html5);
            println!("Markdown: {}", markdown);
            println!("Expected: The variable *x* is defined");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test sub and sup with custom symbols
    let mut opts = ConversionOptions::default();
    opts.sub_symbol = "~".to_string();
    opts.sup_symbol = "^".to_string();

    let html6 = "<p>H<sub>2</sub>O and x<sup>2</sup></p>";
    match convert(html6, Some(opts)) {
        Ok(markdown) => {
            println!("Test - Sub/Sup:");
            println!("HTML: {}", html6);
            println!("Markdown: {}", markdown);
            println!("Expected: H~2~O and x^2^");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test abbr with title
    let html7 = r#"<p>The <abbr title="World Health Organization">WHO</abbr> announced</p>"#;
    match convert(html7, None) {
        Ok(markdown) => {
            println!("Test - Abbr:");
            println!("HTML: {}", html7);
            println!("Markdown: {}", markdown);
            println!("Expected: The WHO (World Health Organization) announced");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // Test u and small (no formatting)
    let html8 = "<p>This is <u>underlined</u> and <small>small</small> text</p>";
    match convert(html8, None) {
        Ok(markdown) => {
            println!("Test - U/Small:");
            println!("HTML: {}", html8);
            println!("Markdown: {}", markdown);
            println!("Expected: This is underlined and small text");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
