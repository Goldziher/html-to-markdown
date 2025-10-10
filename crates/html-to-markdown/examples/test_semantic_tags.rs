use html_to_markdown_rs::convert;

fn main() {
    let html = r#"<article>
        <header><h1>Title</h1></header>
        <section><p>Content here</p></section>
        <footer><p>Footer</p></footer>
    </article>"#;
    match convert(html, None) {
        Ok(markdown) => {
            println!("Test - Semantic blocks:");
            println!("HTML: {}", html);
            println!("\nMarkdown:\n{}", markdown);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html2 = r#"<figure>
        <img src="image.jpg" alt="Diagram">
        <figcaption>Figure 1: Example diagram</figcaption>
    </figure>"#;
    match convert(html2, None) {
        Ok(markdown) => {
            println!("Test - Figure/Figcaption:");
            println!("HTML: {}", html2);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected: ![Diagram](image.jpg)");
            println!("          *Figure 1: Example diagram*");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html3 = r#"<p>As <cite>Shakespeare</cite> said, <q>To be or not to be</q></p>"#;
    match convert(html3, None) {
        Ok(markdown) => {
            println!("Test - Cite/Quote:");
            println!("HTML: {}", html3);
            println!("Markdown: {}", markdown);
            println!("Expected: As *Shakespeare* said, \"To be or not to be\"");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html4 = r#"<dl>
        <dt>Term 1</dt>
        <dd>Definition 1</dd>
        <dt>Term 2</dt>
        <dd>Definition 2</dd>
    </dl>"#;
    match convert(html4, None) {
        Ok(markdown) => {
            println!("Test - Definition list:");
            println!("HTML: {}", html4);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected:");
            println!("**Term 1**");
            println!("Definition 1");
            println!();
            println!("**Term 2**");
            println!("Definition 2");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html5 = r#"<hgroup>
        <h1>Main Title</h1>
        <h2>Subtitle</h2>
    </hgroup>"#;
    match convert(html5, None) {
        Ok(markdown) => {
            println!("Test - Hgroup:");
            println!("HTML: {}", html5);
            println!("\nMarkdown:\n{}", markdown);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
