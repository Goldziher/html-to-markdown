#![no_main]

//! Fuzz `convert` for panics, hangs, and unbounded output.
//!
//! The oracle is deliberately weak: this target does not know what correct Markdown is, only
//! that converting untrusted bytes must not crash the process. That matches this crate's
//! failure history -- issues #216 and #217 were reported panics from a buffer index going
//! stale across handlers, and the depth guard exists because native recursion could overflow
//! the stack.
//!
//! Options are fuzzed alongside the input, because several past defects were reachable only
//! under a non-default `newline_style`, and issue #749 only with the document structure on.
//!
//! Run with:
//!   cargo +nightly fuzz run convert -- -max_len=65536
//! Seed it from the vendored corpus first:
//!   cp tools/benchmark-harness/fixtures/**/*.html fuzz/corpus/convert/

use html_to_markdown_rs::options::{ConversionOptions, NewlineStyle};
use libfuzzer_sys::fuzz_target;

#[derive(arbitrary::Arbitrary, Debug)]
struct Input<'a> {
    backslash_newlines: bool,
    escape_asterisks: bool,
    escape_underscores: bool,
    wrap: bool,
    include_document_structure: bool,
    html: &'a str,
}

fuzz_target!(|input: Input<'_>| {
    let options = ConversionOptions {
        newline_style: if input.backslash_newlines {
            NewlineStyle::Backslash
        } else {
            NewlineStyle::Spaces
        },
        escape_asterisks: input.escape_asterisks,
        escape_underscores: input.escape_underscores,
        wrap: input.wrap,
        include_document_structure: input.include_document_structure,
        ..Default::default()
    };

    // ~keep An `Err` is an acceptable outcome: refusing malformed input is correct. Only a
    // ~keep panic, a hang, or unbounded growth is a finding, and libFuzzer detects the first
    // ~keep two itself. The growth check is ours because a slow memory blow-up otherwise
    // ~keep surfaces only as an opaque OOM kill.
    if let Ok(result) = html_to_markdown_rs::convert(input.html, Some(options)) {
        let produced = result.content.unwrap_or_default().len();
        let ceiling = input.html.len().saturating_mul(64).max(64 * 1024);
        assert!(
            produced <= ceiling,
            "output grew to {produced} bytes from {} bytes of input",
            input.html.len()
        );
    }
});
