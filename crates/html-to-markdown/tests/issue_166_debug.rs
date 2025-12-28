//! Debug test for issue #166: Reuters HTML body tag not producing output.
//!
//! This test reads the Reuters HTML files and converts them with debug logging
//! enabled to trace exactly what happens during body tag processing.

use html_to_markdown_rs::{ConversionOptions, convert};
use std::fs;

#[test]
#[ignore] // Run with: cargo test issue_166_debug -- --ignored --nocapture
fn test_reuters_html_with_debug() {
    // Test with body-only Reuters HTML (g-166-tags-reuters-2.html)
    let body_only_html = fs::read_to_string("../../test_documents/html/issues/gh-166-tags-reuters-2.html")
        .expect("Failed to read body-only Reuters HTML");

    println!(
        "\n=== Testing body-only Reuters HTML (size: {} bytes) ===\n",
        body_only_html.len()
    );

    let mut opts = ConversionOptions::default();
    opts.debug = true;

    let result = convert(&body_only_html, Some(opts.clone())).expect("Failed to convert body-only HTML");

    println!("\nResult size: {} bytes\n", result.len());
    println!(
        "Result preview (first 500 chars):\n{}\n",
        if result.len() > 500 { &result[..500] } else { &result }
    );

    // Test with full Reuters HTML (g-166-tags-reuters.html)
    println!(
        "\n\n=== Testing full Reuters HTML (size: {} bytes) ===\n",
        std::fs::metadata("../../test_documents/html/issues/gh-166-tags-reuters.html")
            .map(|m| m.len())
            .unwrap_or(0)
    );

    let full_html = fs::read_to_string("../../test_documents/html/issues/gh-166-tags-reuters.html")
        .expect("Failed to read full Reuters HTML");

    let result_full = convert(&full_html, Some(opts.clone())).expect("Failed to convert full HTML");

    println!("\nResult size: {} bytes\n", result_full.len());
    println!(
        "Result preview (first 500 chars):\n{}\n",
        if result_full.len() > 500 {
            &result_full[..500]
        } else {
            &result_full
        }
    );

    // Compare results
    println!("\n=== COMPARISON ===");
    println!("Body-only result: {} bytes", result.len());
    println!("Full HTML result: {} bytes", result_full.len());
    println!(
        "Difference: {} bytes",
        (result.len() as i64 - result_full.len() as i64).abs()
    );
}

#[test]
#[ignore] // Run with: cargo test reuters_minimal_debug -- --ignored --nocapture
fn test_reuters_minimal_debug() {
    // Create a minimal Reuters-like structure with body tag
    let minimal_html = r#"<html>
<head>
    <title>Test</title>
</head>
<body>
    <div class="article">
        <h1>Test Article</h1>
        <p>This is the first paragraph.</p>
        <p>This is the second paragraph.</p>
    </div>
</body>
</html>"#;

    println!("\n=== Testing minimal HTML with body tag ===\n");
    println!("Input HTML:\n{}\n", minimal_html);

    let mut opts = ConversionOptions::default();
    opts.debug = true;

    let result = convert(minimal_html, Some(opts)).expect("Failed to convert minimal HTML");

    println!("\nResult size: {} bytes\n", result.len());
    println!("Result:\n{}\n", result);
}

#[test]
#[ignore] // Run with: cargo test reuters_body_only_structure -- --ignored --nocapture
fn test_reuters_body_only_structure() {
    // Simulate what happens when only body content is processed
    let html_with_body = r#"<body>
    <main>
        <article>
            <h1>Headline</h1>
            <p>Article content here.</p>
        </article>
    </main>
</body>"#;

    println!("\n=== Testing body-only HTML structure ===\n");
    println!("Input HTML:\n{}\n", html_with_body);

    let mut opts = ConversionOptions::default();
    opts.debug = true;

    let result = convert(html_with_body, Some(opts)).expect("Failed to convert");

    println!("\nResult size: {} bytes\n", result.len());
    println!("Result:\n{}\n", result);
}
