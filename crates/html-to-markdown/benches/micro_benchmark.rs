//! Micro-benchmarks for specific operations

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use html_to_markdown_rs::{ConversionOptions, convert};
use std::hint::black_box;

/// Benchmark text-heavy documents
fn bench_text_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_operations");

    let html = (0..1000)
        .map(|i| {
            format!(
                "<p>Text {} with * asterisks * and _ underscores _ and [brackets]</p>",
                i
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let options_no_escape = ConversionOptions {
        escape_asterisks: false,
        escape_underscores: false,
        escape_misc: false,
        ..Default::default()
    };

    group.bench_function("no_escaping", |b| {
        b.iter(|| convert(black_box(&html), Some(options_no_escape.clone())))
    });

    let options_escape = ConversionOptions {
        escape_asterisks: true,
        escape_underscores: true,
        escape_misc: true,
        ..Default::default()
    };

    group.bench_function("with_escaping", |b| {
        b.iter(|| convert(black_box(&html), Some(options_escape.clone())))
    });

    group.finish();
}

/// Benchmark list operations
fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");

    let shallow_html = format!(
        "<ul>{}</ul>",
        (0..500)
            .map(|i| format!("<li>Item {}</li>", i))
            .collect::<Vec<_>>()
            .join("")
    );

    group.bench_function("shallow_list_500_items", |b| {
        b.iter(|| convert(black_box(&shallow_html), None))
    });

    let mut deep_html = String::from("<ul>");
    for i in 0..10 {
        deep_html.push_str(&format!("<li>Level {} <ul>", i));
        for j in 0..5 {
            deep_html.push_str(&format!("<li>Item {}</li>", j));
        }
        deep_html.push_str("</ul></li>");
    }
    deep_html.push_str("</ul>");

    group.bench_function("deeply_nested_10_levels", |b| {
        b.iter(|| convert(black_box(&deep_html), None))
    });

    group.finish();
}

/// Benchmark table operations
fn bench_table_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_operations");

    for (name, rows, cols) in [
        ("small_10x5", 10, 5),
        ("medium_50x10", 50, 10),
        ("large_100x20", 100, 20),
    ] {
        let mut html = String::from("<table><thead><tr>");
        for c in 0..cols {
            html.push_str(&format!("<th>Header {}</th>", c));
        }
        html.push_str("</tr></thead><tbody>");
        for r in 0..rows {
            html.push_str("<tr>");
            for c in 0..cols {
                html.push_str(&format!("<td>Cell {},{}</td>", r, c));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");

        group.bench_with_input(BenchmarkId::from_parameter(name), &html, |b, html| {
            b.iter(|| convert(black_box(html), None))
        });
    }

    group.finish();
}

/// Benchmark inline formatting
fn bench_inline_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_formatting");

    let heavy_formatting = (0..500)
        .map(|i| {
            format!(
                "<p>Paragraph {} with <strong>bold</strong> and <em>italic</em> and <strong><em>both</em></strong></p>",
                i
            )
        })
        .collect::<Vec<_>>()
        .join("");

    group.bench_function("bold_italic_500p", |b| {
        b.iter(|| convert(black_box(&heavy_formatting), None))
    });

    let heavy_links = (0..500)
        .map(|i| {
            format!(
                "<p>Paragraph {} with <a href='https://example.com/{}'>link {}</a> and more text</p>",
                i, i, i
            )
        })
        .collect::<Vec<_>>()
        .join("");

    group.bench_function("links_500p", |b| b.iter(|| convert(black_box(&heavy_links), None)));

    let heavy_code = (0..500)
        .map(|i| format!("<p>Paragraph {} with <code>code {}</code> and more text</p>", i, i))
        .collect::<Vec<_>>()
        .join("");

    group.bench_function("code_spans_500p", |b| b.iter(|| convert(black_box(&heavy_code), None)));

    group.finish();
}

/// Benchmark DOM traversal
fn bench_dom_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("dom_traversal");

    let mut html = String::from("<html><body>");
    for _ in 0..20 {
        html.push_str("<div>");
    }
    html.push_str("<p>Content</p>");
    for _ in 0..20 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    group.bench_function("deep_nesting_20_levels", |b| b.iter(|| convert(black_box(&html), None)));

    let wide_html = format!(
        "<html><body>{}</body></html>",
        (0..500)
            .map(|i| format!("<p>Para {}</p>", i))
            .collect::<Vec<_>>()
            .join("")
    );

    group.bench_function("wide_tree_500_siblings", |b| {
        b.iter(|| convert(black_box(&wide_html), None))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_text_operations,
    bench_list_operations,
    bench_table_operations,
    bench_inline_formatting,
    bench_dom_traversal
);
criterion_main!(benches);
