//! Compare mdream output vs h2m output side-by-side on a single fixture.
//!
//! Usage: cargo run --release --example mdream_diff -p html-to-markdown-bench -- <fixture-rel-path>

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use std::path::PathBuf;

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
use mdream::{HtmlToMarkdownOptions, html_to_markdown as mdream_convert};

fn main() -> anyhow::Result<()> {
    let arg = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: mdream_diff <fixture>"))?;
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let path = fixtures_dir.join(&arg);
    let html = std::fs::read_to_string(&path)?;

    let h2m_opts = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    let h2m = convert(&html, Some(h2m_opts))?.content.unwrap_or_default();
    let md = mdream_convert(&html, HtmlToMarkdownOptions::default());

    println!("=== h2m ({} bytes) ===", h2m.len());
    println!("{}", &h2m[..h2m.len().min(2000)]);
    println!("\n=== mdream ({} bytes) ===", md.len());
    println!("{}", &md[..md.len().min(2000)]);

    Ok(())
}
