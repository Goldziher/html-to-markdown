//! Phase-isolated timing for deeply nested `<div>` input.
//!
//! Times `tl::parse`, `build_dom_context`, and `has_inline_block_misnest` independently
//! for synthetic `<div>*n x </div>*n` input at several nesting depths, to localize which
//! stage scales quadratically with nesting depth. The input has no scripts, styles,
//! comments, or unclosed list items, so `convert_html_impl`'s byte-level preprocessing
//! passes are no-ops for it and are skipped here — parsing the raw HTML directly.
//!
//! Requires the `testkit` feature (exposes `build_dom_context`/`has_inline_block_misnest`
//! for direct timing).
//!
//! Run with: `cargo run -p html-to-markdown-bench --features testkit \
//!   --example profile_deep_nesting_phases`

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use std::time::Instant;

use html_to_markdown_rs::testkit::{build_dom_context, has_inline_block_misnest};

fn deep_nested_divs(depth: usize) -> String {
    let mut html = String::with_capacity(depth * 5 * 2 + 1);
    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push('x');
    for _ in 0..depth {
        html.push_str("</div>");
    }
    html
}

fn time_phase_for_depth(depth: usize) {
    let html = deep_nested_divs(depth);

    let start = Instant::now();
    let dom = tl::parse(&html, tl::ParserOptions::default()).expect("parse deep-nested fixture");
    let parse_elapsed = start.elapsed();

    let parser = dom.parser();
    let start = Instant::now();
    // ~keep The predicate reads the DOM now, so keep the separate context-build measurement observable.
    let _dom_ctx = std::hint::black_box(build_dom_context(&dom, parser, html.len()));
    let dom_ctx_elapsed = start.elapsed();

    let start = Instant::now();
    let misnest = has_inline_block_misnest(&dom, parser);
    let misnest_elapsed = start.elapsed();

    eprintln!(
        "depth={depth:>6}  tl_parse={parse_elapsed:>10.2?}  build_dom_context={dom_ctx_elapsed:>10.2?}  \
         has_inline_block_misnest={misnest_elapsed:>10.2?}  (misnest_detected={misnest})"
    );
}

fn main() {
    eprintln!("phase-isolated timing for deeply nested <div> input (manual Instant timing, no sampling profiler)");
    for depth in [2_000, 5_000, 10_000, 20_000] {
        time_phase_for_depth(depth);
    }
}
