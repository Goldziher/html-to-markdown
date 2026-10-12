// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression coverage for deeply nested and malformed markup. `tl` preserves
//! repeated unclosed `<td>` tags as a multi-thousand-level DOM chain, so these
//! tests run conversion on small thread stacks to catch native-stack recursion
//! in whole-subtree helpers.
//!
//! The `<td>` shape reaches hierarchy recording, metadata extraction, and table
//! scanning while the main conversion walker remains bounded by its own depth
//! guard. That keeps failures attributable to the helper traversal under test.

use html_to_markdown_rs::ConversionResult;
use html_to_markdown_rs::convert;
use html_to_markdown_rs::options::{ConversionOptions, OutputFormat};
use std::sync::{Mutex, MutexGuard};
use std::thread;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_MUTEX.lock().expect("deep nesting test mutex poisoned")
}

fn converts_without_overflow(html: String, options: ConversionOptions) -> bool {
    converts_without_overflow_on_stack(html, options, 256 * 1024)
}

fn converts_without_overflow_on_stack(html: String, options: ConversionOptions, stack_size: usize) -> bool {
    thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || convert(&html, Some(options)).is_ok())
        .expect("spawn conversion thread")
        .join()
        .expect("conversion thread overflowed the stack")
}

/// Like [`converts_without_overflow`] but returns the successful result so callers can
/// inspect the (possibly truncated) output instead of only checking `Result::is_ok`.
fn convert_without_overflow_on_stack(html: String, options: ConversionOptions, stack_size: usize) -> ConversionResult {
    thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || convert(&html, Some(options)))
        .expect("spawn conversion thread")
        .join()
        .expect("conversion thread overflowed the stack")
        .expect("conversion should not fail")
}

/// Exercises `record_node_hierarchy` (pre-pass) and `scan_table_node` (table
/// scan); the `<head>` is found without descending the deep chain.
#[test]
fn deep_unclosed_table_cells_do_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><head><title>t</title></head><body><table><tr>");
    for _ in 0..20_000 {
        html.push_str("<td>x");
    }
    html.push_str("</tr></table></body></html>");
    let options = ConversionOptions::builder().max_depth(Some(200)).build();
    assert!(converts_without_overflow(html, options));
}

/// No `<head>`, so metadata extraction must search the entire deep chain.
#[test]
fn deep_markup_without_head_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><body><table><tr>");
    for _ in 0..20_000 {
        html.push_str("<td>x");
    }
    html.push_str("</tr></table></body></html>");
    let options = ConversionOptions::builder().max_depth(Some(200)).build();
    assert!(converts_without_overflow(html, options));
}

#[test]
fn deep_link_descendant_text_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><head><title>t</title></head><body><a href=\"/deep\">");
    for _ in 0..1_000 {
        html.push_str("<span>");
    }
    html.push_str("deep");
    html.push_str("</a></body></html>");

    let options = ConversionOptions::builder().max_depth(Some(200)).build();
    assert!(converts_without_overflow_on_stack(html, options, 8 * 1024 * 1024));
}

#[test]
fn default_depth_uses_stack_safe_limit() {
    let _guard = test_lock();
    let mut html = String::from("<html><body>");
    for _ in 0..1_000 {
        html.push_str("<div>");
    }
    html.push_str("deep");
    for _ in 0..1_000 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    // ~keep MSVC debug frames need more than the helper's 256 KiB budget even when the
    // converter correctly stops at the native stack-safe depth. One MiB remains far below
    // what an unfixed 1,000-frame traversal requires, so the regression still fails.
    assert!(converts_without_overflow_on_stack(
        html,
        ConversionOptions::default(),
        1024 * 1024
    ));
}

#[test]
fn plain_text_output_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><body>");
    for _ in 0..1_000 {
        html.push_str("<div>");
    }
    html.push_str("deep");
    for _ in 0..1_000 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    let options = ConversionOptions {
        output_format: OutputFormat::Plain,
        max_depth: Some(200),
        ..Default::default()
    };
    // ~keep Plain output forces the recursive Tier-2 walk, and issue #434 lets an explicit
    // ~keep max_depth exceed the native stack-safe default (64), so depth 200 genuinely recurses
    // ~keep 200 frames. Raising the ceiling is the caller opting into deeper traversal, which needs
    // ~keep a proportionate stack — mirror the 8 MiB budget used by deep_link_descendant_text.
    assert!(converts_without_overflow_on_stack(html, options, 8 * 1024 * 1024));
}

/// Exercises `media::svg::serialize_element_at_depth`/`serialize_node_at_depth`
/// (audit #23): the SVG-to-base64 embedding path used to mutually recurse over
/// `<g>` children with no depth bound at all, independent of the main walker's
/// own depth guard (the `<svg>` element is serialized as a single leaf, so its
/// internal children never pass through `walk_node`).
#[test]
fn deep_svg_group_nesting_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><body><svg>");
    for _ in 0..50_000 {
        html.push_str("<g>");
    }
    for _ in 0..50_000 {
        html.push_str("</g>");
    }
    html.push_str("</svg></body></html>");

    let result = convert_without_overflow_on_stack(html, ConversionOptions::default(), 256 * 1024);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(
        content.contains("data:image/svg+xml;base64,"),
        "expected a (truncated) base64 SVG data URI in the output. Got:\n{content}"
    );
}

/// Deep `MathML` text extraction remains safe after dropping the serialized source comment.
#[test]
fn deep_mathml_nesting_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><body><math>");
    for _ in 0..50_000 {
        html.push_str("<mrow>");
    }
    html.push_str("<mi>x</mi>");
    for _ in 0..50_000 {
        html.push_str("</mrow>");
    }
    html.push_str("</math></body></html>");

    let result = convert_without_overflow_on_stack(html, ConversionOptions::default(), 256 * 1024);
    let content = result.content.as_deref().unwrap_or_default();
    assert_eq!(content, "x\n");
}

/// Regression coverage: table-cell conversion used to reset the `walk_node` recursion
/// depth to a literal `0` at every cell boundary (`table/cell.rs`, `table/cells.rs`,
/// `table/mod.rs`, `table/builder.rs`), instead of threading the real depth through as
/// `depth + 1` like every other descent in `converter/`. Because each `<table>` nested
/// inside a `<td>` restarted the counter, `walk_node`'s own depth guard
/// (`crate::converter::main::effective_max_depth`) could never fire for a chain of
/// nested tables, so a deeply nested `<table><tr><td>` chain recursed without bound and
/// could overflow the native stack. The fix threads `depth + 1` through every cell/row
/// descent so the guard now covers nested tables the same way it covers every other
/// element chain.
#[test]
fn deeply_nested_tables_do_not_overflow_stack() {
    // ~keep 2,000 is ~30x the native-stack-safe depth guard (64), which is enough to prove
    // ~keep the guard fires reliably; `scan_table_node` (table/scanner.rs) independently walks
    // ~keep the full remaining nested-table chain from every table's own position (a separate,
    // ~keep pre-existing O(n^2) cost unrelated to this depth-reset fix), so a much larger depth
    // ~keep here would make this test impractically slow without adding overflow coverage.
    const NESTING_DEPTH: usize = 2_000;
    // ~keep One MiB unconditionally. The budget has to clear the cost of the BOUNDED
    // ~keep recursion the depth guard allows (64 frames) in the heaviest build, not just the
    // ~keep default one: enabling any non-default feature -- `visitor` and `inline-images`
    // ~keep were each enough on their own -- adds locals to `walk_node`'s frame, and a debug
    // ~keep `--all-features` build needed just over 256 KiB, so the previous Unix budget
    // ~keep aborted the whole test binary with a stack overflow. It was also already
    // ~keep per-platform for the same underlying reason (MSVC frames are larger), so a
    // ~keep single figure with headroom replaces a `cfg!` that only tracked one cause of it.
    // ~keep One MiB stays an order of magnitude below what the unfixed 2,000-frame recursion
    // ~keep would require, so the regression this test exists for still fires.
    const TEST_STACK_SIZE: usize = 1024 * 1024;

    let _guard = test_lock();

    let mut html = String::from("<html><body>");
    for _ in 0..NESTING_DEPTH {
        html.push_str("<table><tr><td>");
    }
    html.push_str("leaf");
    for _ in 0..NESTING_DEPTH {
        html.push_str("</td></tr></table>");
    }
    html.push_str("</body></html>");

    let result = convert_without_overflow_on_stack(html, ConversionOptions::default(), TEST_STACK_SIZE);
    let content = result.content.as_deref().unwrap_or_default();
    assert!(
        !content.trim().is_empty(),
        "deeply nested table conversion must not silently return empty output"
    );
    assert!(
        content.contains('|'),
        "expected at least the outer table rows to render as pipe-delimited content. Got:\n{content}"
    );
}

#[test]
fn document_structure_builder_does_not_overflow_stack() {
    let _guard = test_lock();
    let mut html = String::from("<html><body>");
    for _ in 0..1_000 {
        html.push_str("<section>");
    }
    html.push_str("<p>deep</p>");
    for _ in 0..1_000 {
        html.push_str("</section>");
    }
    html.push_str("</body></html>");

    assert!(
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let dom = tl::parse(&html, tl::ParserOptions::default()).expect("parse deep html");
                let document = html_to_markdown_rs::types::build_document_structure(&dom);
                !document.nodes.is_empty()
            })
            .expect("spawn structure thread")
            .join()
            .expect("structure thread overflowed the stack")
    );
}
