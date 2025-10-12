//! Profiling-focused benchmarks for identifying performance bottlenecks
//!
//! Run with: cargo bench --bench profiling_benchmark --profile-time=10
//! Or with flamegraph: cargo flamegraph --bench profiling_benchmark

use html_to_markdown_rs::convert;
use std::hint::black_box;

fn load_wikipedia_document(filename: &str) -> Option<String> {
    let path = format!("../../tests/benchmark_documents/wikipedia/{}", filename);
    std::fs::read_to_string(path).ok()
}

fn load_hocr_document(filename: &str) -> Option<String> {
    let path = format!("../../tests/test_data/hocr/{}", filename);
    std::fs::read_to_string(path).ok()
}

/// Generate complex HTML for stress testing
fn generate_stress_test_html(tables: usize, lists: usize, paragraphs: usize) -> String {
    let mut html = String::with_capacity(1024 * 1024);
    html.push_str("<html><body>");

    for t in 0..tables {
        html.push_str("<table><thead><tr>");
        for c in 0..10 {
            html.push_str(&format!("<th>Header {}-{}</th>", t, c));
        }
        html.push_str("</tr></thead><tbody>");
        for r in 0..50 {
            html.push_str("<tr>");
            for c in 0..10 {
                html.push_str(&format!("<td>Data {}-{}-{}</td>", t, r, c));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
    }

    for l in 0..lists {
        html.push_str("<ul>");
        for i in 0..20 {
            html.push_str(&format!(
                "<li>Item {}-{} with <strong>bold</strong> and <em>italic</em>",
                l, i
            ));
            html.push_str("<ul>");
            for j in 0..10 {
                html.push_str(&format!("<li>Nested {}-{}-{}</li>", l, i, j));
            }
            html.push_str("</ul></li>");
        }
        html.push_str("</ul>");
    }

    for p in 0..paragraphs {
        html.push_str(&format!(
            "<p>Paragraph {} with <strong>bold text</strong>, <em>italic text</em>, \
             <code>inline code</code>, <a href='https://example.com/{}'>links</a>, \
             <mark>highlighted</mark>, and <del>strikethrough</del>.</p>",
            p, p
        ));
    }

    html.push_str("</body></html>");
    html
}

fn main() {
    println!("Starting profiling benchmarks...\n");

    if let Some(html) = load_wikipedia_document("medium_python.html") {
        println!("Profiling: Wikipedia Python article (656KB)");
        for _ in 0..100 {
            let _ = black_box(convert(black_box(&html), None));
        }
        println!("  ✓ Completed 100 iterations\n");
    }

    if let Some(html) = load_wikipedia_document("tables_countries.html") {
        println!("Profiling: Wikipedia Countries (360KB, table-heavy)");
        for _ in 0..100 {
            let _ = black_box(convert(black_box(&html), None));
        }
        println!("  ✓ Completed 100 iterations\n");
    }

    if let Some(html) = load_wikipedia_document("lists_timeline.html") {
        println!("Profiling: Wikipedia Timeline (129KB, list-heavy)");
        for _ in 0..100 {
            let _ = black_box(convert(black_box(&html), None));
        }
        println!("  ✓ Completed 100 iterations\n");
    }

    if let Some(html) = load_hocr_document("german_pdf_german.hocr") {
        println!("Profiling: hOCR German PDF (44KB) - automatic table extraction");
        for _ in 0..500 {
            let _ = black_box(convert(black_box(&html), None));
        }
        println!("  ✓ Completed 500 iterations\n");
    }

    println!("Profiling: Stress test - 50 complex tables");
    let html = generate_stress_test_html(50, 0, 0);
    for _ in 0..50 {
        let _ = black_box(convert(black_box(&html), None));
    }
    println!("  ✓ Completed 50 iterations\n");

    println!("Profiling: Stress test - 100 nested lists");
    let html = generate_stress_test_html(0, 100, 0);
    for _ in 0..50 {
        let _ = black_box(convert(black_box(&html), None));
    }
    println!("  ✓ Completed 50 iterations\n");

    println!("Profiling: Stress test - 1000 paragraphs with heavy inline formatting");
    let html = generate_stress_test_html(0, 0, 1000);
    for _ in 0..50 {
        let _ = black_box(convert(black_box(&html), None));
    }
    println!("  ✓ Completed 50 iterations\n");

    println!("Profiling: Stress test - combined (tables + lists + paragraphs)");
    let html = generate_stress_test_html(20, 30, 500);
    for _ in 0..30 {
        let _ = black_box(convert(black_box(&html), None));
    }
    println!("  ✓ Completed 30 iterations\n");

    println!("Profiling complete!");
}
