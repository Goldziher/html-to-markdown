use html_to_markdown_rs::convert;

fn main() {
    let html = r#"<table>
        <tr>
            <th>Name</th>
            <th>Age</th>
        </tr>
        <tr>
            <td>Alice</td>
            <td>30</td>
        </tr>
        <tr>
            <td>Bob</td>
            <td>25</td>
        </tr>
    </table>"#;

    match convert(html, None) {
        Ok(markdown) => {
            println!("Test - Simple table with header:");
            println!("HTML: {}", html);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected:");
            println!("| Name | Age |");
            println!("| --- | --- |");
            println!("| Alice | 30 |");
            println!("| Bob | 25 |");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html2 = r#"<table>
        <tr>
            <th colspan="2">Full Name</th>
            <th>Age</th>
        </tr>
        <tr>
            <td>Alice</td>
            <td>Smith</td>
            <td>30</td>
        </tr>
    </table>"#;

    match convert(html2, None) {
        Ok(markdown) => {
            println!("Test - Table with colspan:");
            println!("HTML: {}", html2);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected:");
            println!("| Full Name | | Age |");
            println!("| --- | --- | --- |");
            println!("| Alice | Smith | 30 |");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let html3 = r#"<table>
        <thead>
            <tr>
                <th>Product</th>
                <th>Price</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>Widget</td>
                <td>$10</td>
            </tr>
            <tr>
                <td>Gadget</td>
                <td>$15</td>
            </tr>
        </tbody>
    </table>"#;

    match convert(html3, None) {
        Ok(markdown) => {
            println!("Test - Table with thead/tbody:");
            println!("HTML: {}", html3);
            println!("\nMarkdown:\n{}", markdown);
            println!("Expected:");
            println!("| Product | Price |");
            println!("| --- | --- |");
            println!("| Widget | $10 |");
            println!("| Gadget | $15 |");
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
