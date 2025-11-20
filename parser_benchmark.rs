//! Benchmark comparing astral-tl vs html5ever for HTML parsing
//!
//! Tests:
//! 1. Parse speed
//! 2. DOM traversal speed
//! 3. Whitespace preservation

use std::time::Instant;

// Test HTML samples
const SIMPLE_HTML: &str = r#"<p>Hello <strong>world</strong>!</p>"#;

const MEDIUM_HTML: &str = r#"
<html>
<head><title>Test</title></head>
<body>
    <h1>Main Title</h1>
    <p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
    <pre><code>let x = 42;</code></pre>
</body>
</html>
"#;

const WHITESPACE_TEST: &str = r#"<p>  Multiple   spaces   here  </p>"#;
const PRESERVED_WHITESPACE: &str = r#"<pre>  Preserved   spaces  </pre>"#;

fn benchmark_html5ever() {
    println!("\n=== HTML5EVER BENCHMARKS ===\n");

    // Simple parse
    let start = Instant::now();
    let iterations = 10000;
    for _ in 0..iterations {
        use html5ever::parse_document;
        use html5ever::tendril::TendrilSink;
        use markup5ever_rcdom::RcDom;
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut SIMPLE_HTML.as_bytes())
            .unwrap();
        std::hint::black_box(dom);
    }
    let duration = start.elapsed();
    println!(
        "Simple HTML parse ({}x): {:?} ({:.2} µs/iter)",
        iterations,
        duration,
        duration.as_micros() as f64 / iterations as f64
    );

    // Medium parse
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        use html5ever::parse_document;
        use html5ever::tendril::TendrilSink;
        use markup5ever_rcdom::RcDom;
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut MEDIUM_HTML.as_bytes())
            .unwrap();
        std::hint::black_box(dom);
    }
    let duration = start.elapsed();
    println!(
        "Medium HTML parse ({}x): {:?} ({:.2} µs/iter)",
        iterations,
        duration,
        duration.as_micros() as f64 / iterations as f64
    );

    // Test whitespace handling
    println!("\n--- Whitespace Handling ---");
    test_html5ever_whitespace();
}

fn test_html5ever_whitespace() {
    use html5ever::parse_document;
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::{Handle, NodeData, RcDom};

    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut WHITESPACE_TEST.as_bytes())
        .unwrap();

    fn extract_text(handle: &Handle) -> String {
        let mut text = String::new();
        match handle.data {
            NodeData::Text { ref contents } => {
                text.push_str(&contents.borrow());
            }
            NodeData::Element { .. } => {
                for child in handle.children.borrow().iter() {
                    text.push_str(&extract_text(child));
                }
            }
            _ => {
                for child in handle.children.borrow().iter() {
                    text.push_str(&extract_text(child));
                }
            }
        }
        text
    }

    let text = extract_text(&dom.document);
    println!("Whitespace test: {:?}", text);
    println!("  Original: {:?}", WHITESPACE_TEST);

    // Test <pre> preservation
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut PRESERVED_WHITESPACE.as_bytes())
        .unwrap();
    let text = extract_text(&dom.document);
    println!("Pre whitespace: {:?}", text);
    println!("  Original: {:?}", PRESERVED_WHITESPACE);
}

fn benchmark_astral_tl() {
    println!("\n=== ASTRAL-TL BENCHMARKS ===\n");

    // Simple parse
    let start = Instant::now();
    let iterations = 10000;
    for _ in 0..iterations {
        let dom = astral_tl::parse(SIMPLE_HTML, astral_tl::ParserOptions::default()).unwrap();
        std::hint::black_box(dom);
    }
    let duration = start.elapsed();
    println!(
        "Simple HTML parse ({}x): {:?} ({:.2} µs/iter)",
        iterations,
        duration,
        duration.as_micros() as f64 / iterations as f64
    );

    // Medium parse
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let dom = astral_tl::parse(MEDIUM_HTML, astral_tl::ParserOptions::default()).unwrap();
        std::hint::black_box(dom);
    }
    let duration = start.elapsed();
    println!(
        "Medium HTML parse ({}x): {:?} ({:.2} µs/iter)",
        iterations,
        duration,
        duration.as_micros() as f64 / iterations as f64
    );

    // Test whitespace handling
    println!("\n--- Whitespace Handling ---");
    test_astral_tl_whitespace();
}

fn test_astral_tl_whitespace() {
    let dom = astral_tl::parse(WHITESPACE_TEST, astral_tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    // Extract all text
    let mut text = String::new();
    for node in dom.nodes() {
        if let Some(tag) = node.as_tag() {
            text.push_str(&tag.inner_text(parser));
        } else if let Some(raw_text) = node.as_raw() {
            text.push_str(raw_text.as_utf8_str());
        }
    }

    println!("Whitespace test: {:?}", text);
    println!("  Original: {:?}", WHITESPACE_TEST);

    // Test <pre> preservation
    let dom = astral_tl::parse(PRESERVED_WHITESPACE, astral_tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let mut text = String::new();
    for node in dom.nodes() {
        if let Some(tag) = node.as_tag() {
            text.push_str(&tag.inner_text(parser));
        } else if let Some(raw_text) = node.as_raw() {
            text.push_str(raw_text.as_utf8_str());
        }
    }
    println!("Pre whitespace: {:?}", text);
    println!("  Original: {:?}", PRESERVED_WHITESPACE);
}

fn main() {
    println!("HTML Parser Benchmark: astral-tl vs html5ever\n");
    println!("Testing performance and whitespace handling...\n");

    benchmark_html5ever();
    benchmark_astral_tl();

    println!("\n=== SUMMARY ===");
    println!("Run this benchmark to compare:");
    println!("1. Parse speed (iterations per second)");
    println!("2. Whitespace handling (preservation vs normalization)");
    println!("\nDecision criteria:");
    println!("- Migrate if: astral-tl is significantly faster OR preserves whitespace better");
    println!("- Stay if: html5ever is comparable and meets requirements");
}
