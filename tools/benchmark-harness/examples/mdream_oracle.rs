//! Byte-equality oracle: mdream vs h2m Tier-2.
//!
//! For each bench fixture compute:
//!   - h2m Tier-2 output (authoritative)
//!   - mdream output
//!   - byte-equal? if not, edit-similarity (LCS-line ratio) and first-diff line
//!
//! A fixture is "safely routable to mdream" only if byte-equal. Anything else
//! is a divergence we'd ship as an output regression.
//!
//! Run:
//!   cargo run --release --example mdream_oracle -p html-to-markdown-bench

#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use std::path::PathBuf;

use html_to_markdown_bench::fixture::Loader;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
use mdream::{HtmlToMarkdownOptions, html_to_markdown as mdream_convert};

fn main() -> anyhow::Result<()> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let loader = Loader::new(fixtures_dir);
    let fixtures = loader.load(None)?;

    let h2m_opts = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    let md_opts = HtmlToMarkdownOptions::default();

    println!(
        "{:<48} {:>10} {:>10} {:>9} {:>8} {:>9}",
        "fixture", "h2m bytes", "md bytes", "byte_eq", "sim%", "first_div_line"
    );
    println!("{:-<98}", "");

    let mut equal = 0usize;
    let mut diverge = 0usize;
    let mut details: Vec<(String, f64, usize, String)> = Vec::new();

    for fix in &fixtures {
        let html = std::fs::read_to_string(&fix.path)?;
        let h2m_out = convert(&html, Some(h2m_opts.clone()))?.content.unwrap_or_default();
        let md_out = mdream_convert(&html, md_opts.clone());

        let byte_eq = h2m_out == md_out;
        let sim = line_similarity(&h2m_out, &md_out);
        let (first_div, snippet) = first_diff(&h2m_out, &md_out);
        let label = if byte_eq { "Y" } else { "N" };

        println!(
            "{:<48} {:>10} {:>10} {:>9} {:>7.0}% {:>9}",
            fix.rel_path,
            h2m_out.len(),
            md_out.len(),
            label,
            sim * 100.0,
            first_div.map_or_else(|| "-".to_string(), |n| n.to_string())
        );

        if byte_eq {
            equal += 1;
        } else {
            diverge += 1;
            details.push((fix.rel_path.clone(), sim, first_div.unwrap_or(0), snippet));
        }
    }

    println!();
    println!("=== Summary ===");
    println!("byte-equal:  {}/{}", equal, equal + diverge);
    println!("diverge:     {}/{}", diverge, equal + diverge);

    if !details.is_empty() {
        println!();
        println!("=== Divergence detail (top 10 by similarity, highest first) ===");
        details.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (name, sim, line, snippet) in details.iter().take(10) {
            println!("\n--- {} (sim {:.0}%, first diff line {}) ---", name, sim * 100.0, line);
            println!("{}", snippet);
        }

        println!();
        println!("=== Routable-if-perfect candidates (sim >= 95%) ===");
        let near = details.iter().filter(|d| d.1 >= 0.95).count();
        println!(
            "{} additional fixtures within 5% of Tier-2 — would need normalization to be drop-in",
            near
        );
    }

    Ok(())
}

fn line_similarity(a: &str, b: &str) -> f64 {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    if al.is_empty() && bl.is_empty() {
        return 1.0;
    }
    let max = al.len().max(bl.len());
    if max == 0 {
        return 1.0;
    }
    let lcs = lcs_len(&al, &bl);
    lcs as f64 / max as f64
}

fn lcs_len(a: &[&str], b: &[&str]) -> usize {
    let cap = 2000;
    let a = &a[..a.len().min(cap)];
    let b = &b[..b.len().min(cap)];
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..a.len() {
        for j in 0..b.len() {
            dp[i + 1][j + 1] = if a[i] == b[j] {
                dp[i][j] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    dp[a.len()][b.len()]
}

fn first_diff(a: &str, b: &str) -> (Option<usize>, String) {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    for (i, (x, y)) in al.iter().zip(bl.iter()).enumerate() {
        if x != y {
            let snippet = format!("  T2: {:?}\n  MD: {:?}", truncate(x, 120), truncate(y, 120));
            return (Some(i + 1), snippet);
        }
    }
    if al.len() != bl.len() {
        let i = al.len().min(bl.len());
        let snippet = format!("  line count differs (T2={} MD={})", al.len(), bl.len());
        return (Some(i + 1), snippet);
    }
    (None, String::new())
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}
