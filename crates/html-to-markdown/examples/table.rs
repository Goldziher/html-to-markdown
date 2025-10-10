use html_to_markdown_rs::convert;

fn main() {
    let html = r#"<table>
        <tr><th>Name</th><th>Age</th></tr>
        <tr><td>Alice</td><td>30</td></tr>
        <tr><td>Bob</td><td>25</td></tr>
    </table>"#;

    match convert(html, None) {
        Ok(markdown) => {
            println!("Markdown:\n{}", markdown);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
