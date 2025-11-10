//! Criterion benchmarks for html-to-markdown conversion
//!
//! Measures throughput (ops/sec, MB/sec) and provides performance baselines
//! for the core Rust conversion engine.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, HeadingStyle, convert};
use std::hint::black_box;
use std::time::Duration;

/// Generate HTML with varying complexity
fn generate_test_html(paragraphs: usize, has_tables: bool, has_lists: bool) -> String {
    let mut html = String::from("<html><body>");

    for i in 0..paragraphs {
        html.push_str(&format!(
            "<p>This is paragraph {} with <strong>bold text</strong>, \
             <em>italic text</em>, and <a href='https://example.com'>links</a>. \
             It also has <code>inline code</code> and <mark>highlighted text</mark>.</p>",
            i
        ));
    }

    if has_lists {
        html.push_str("<ul>");
        for i in 0..20 {
            html.push_str(&format!("<li>List item {} with <strong>formatting</strong></li>", i));
        }
        html.push_str("</ul>");

        html.push_str("<ol>");
        for i in 0..15 {
            html.push_str(&format!("<li>Numbered item {}</li>", i));
        }
        html.push_str("</ol>");
    }

    if has_tables {
        html.push_str(
            "<table>
            <thead>
                <tr><th>Column 1</th><th>Column 2</th><th>Column 3</th><th>Column 4</th></tr>
            </thead>
            <tbody>",
        );
        for i in 0..30 {
            html.push_str(&format!(
                "<tr><td>Data {}</td><td>Value {}</td><td>Info {}</td><td>Result {}</td></tr>",
                i, i, i, i
            ));
        }
        html.push_str("</tbody></table>");
    }

    html.push_str("</body></html>");
    html
}

fn load_wikipedia_document(filename: &str) -> Option<String> {
    let path = format!("../../tests/benchmark_documents/wikipedia/{}", filename);
    std::fs::read_to_string(path).ok()
}

/// Benchmark small documents (< 10KB)
fn bench_small_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_documents");

    let html = generate_test_html(10, false, false);
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("simple_10_paragraphs", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    let html = generate_test_html(5, true, true);
    let size = html.len();
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("mixed_content", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    group.finish();
}

/// Benchmark medium documents (10KB - 100KB)
fn bench_medium_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_documents");

    let html = generate_test_html(100, true, true);
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("100_paragraphs_mixed", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    group.finish();
}

/// Benchmark large documents (> 100KB)
fn bench_large_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_documents");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let html = generate_test_html(500, true, true);
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("500_paragraphs_mixed", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    group.finish();
}

/// Benchmark real Wikipedia documents
fn bench_wikipedia_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("wikipedia_real_world");
    group.measurement_time(Duration::from_secs(10));

    if let Some(html) = load_wikipedia_document("lists_timeline.html") {
        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("timeline_lists", format!("{}KB", size / 1024)),
            &html,
            |b, html| b.iter(|| convert(black_box(html), None)),
        );
    }

    if let Some(html) = load_wikipedia_document("tables_countries.html") {
        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("countries_tables", format!("{}KB", size / 1024)),
            &html,
            |b, html| b.iter(|| convert(black_box(html), None)),
        );
    }

    if let Some(html) = load_wikipedia_document("medium_python.html") {
        let size = html.len();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("python_article", format!("{}KB", size / 1024)),
            &html,
            |b, html| b.iter(|| convert(black_box(html), None)),
        );
    }

    group.finish();
}

/// Benchmark specific features
fn bench_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("features");

    let mut html = String::from("<html><body>");
    for _ in 0..20 {
        html.push_str(
            "<table>
            <tr><th>H1</th><th>H2</th><th>H3</th><th>H4</th></tr>",
        );
        for i in 0..20 {
            html.push_str(&format!(
                "<tr><td>D{}-1</td><td>D{}-2</td><td>D{}-3</td><td>D{}-4</td></tr>",
                i, i, i, i
            ));
        }
        html.push_str("</table>");
    }
    html.push_str("</body></html>");
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("tables_heavy", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    let mut html = String::from("<html><body>");
    for _ in 0..50 {
        html.push_str("<ul>");
        for i in 0..30 {
            html.push_str(&format!(
                "<li>Item {} with <strong>bold</strong> and <a href='#'>link</a></li>",
                i
            ));
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("lists_heavy", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    let mut html = String::from("<html><body>");
    for i in 0..500 {
        html.push_str(&format!(
            "<p>Paragraph {} with <strong>bold</strong>, <em>italic</em>, \
             <code>code</code>, <mark>highlight</mark>, <del>strike</del>, \
             and <a href='https://example.com/page{}'>links</a>.</p>",
            i, i
        ));
    }
    html.push_str("</body></html>");
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("inline_formatting_heavy", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    group.finish();
}

/// Benchmark different configuration options
fn bench_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let html = generate_test_html(100, true, true);
    let size = html.len();

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("default_commonmark", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), None))
    });

    let options = ConversionOptions {
        heading_style: HeadingStyle::Atx,
        ..Default::default()
    };
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("atx_headings", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), Some(options.clone())))
    });

    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Backticks,
        ..Default::default()
    };
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("fenced_code_backticks", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), Some(options.clone())))
    });

    let options = ConversionOptions {
        escape_asterisks: true,
        escape_underscores: true,
        escape_misc: true,
        ..Default::default()
    };
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(BenchmarkId::new("aggressive_escaping", size), &html, |b, html| {
        b.iter(|| convert(black_box(html), Some(options.clone())))
    });

    group.finish();
}

/// Benchmark hOCR table extraction
fn bench_hocr(c: &mut Criterion) {
    let mut group = c.benchmark_group("hocr");
    group.measurement_time(Duration::from_secs(10));

    let hocr_files = [
        ("german_pdf", "../../tests/test_data/hocr/german_pdf_german.hocr"),
        ("english_pdf", "../../tests/test_data/hocr/english_pdf_default.hocr"),
        ("valid_file", "../../tests/test_data/hocr/comprehensive/valid_file.hocr"),
    ];

    for (name, path) in &hocr_files {
        if let Ok(html) = std::fs::read_to_string(path) {
            let size = html.len();

            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_auto_tables", name), size),
                &html,
                |b, html| b.iter(|| convert(black_box(html), None)),
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_small_documents,
    bench_medium_documents,
    bench_large_documents,
    bench_wikipedia_documents,
    bench_features,
    bench_configurations,
    bench_hocr
);
criterion_main!(benches);
