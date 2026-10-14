//! Regression coverage for issue #190.

fn convert(
    html: &str,
    opts: Option<html_to_markdown_rs::ConversionOptions>,
) -> html_to_markdown_rs::error::Result<String> {
    html_to_markdown_rs::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions};

fn fixture_path(name: &str) -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "../../test_documents/html/issues/gh-190",
        name,
    ]
    .iter()
    .collect()
}

fn read_fixture_lossy(name: &str) -> String {
    let bytes = fs::read(fixture_path(name)).expect("read issue #190 fixture");
    String::from_utf8_lossy(&bytes).into_owned()
}

fn decode_utf16_without_bom(bytes: &[u8]) -> String {
    let mut even_nul = 0usize;
    let mut odd_nul = 0usize;
    for (idx, &byte) in bytes.iter().enumerate() {
        if byte == 0 {
            if idx % 2 == 0 {
                even_nul += 1;
            } else {
                odd_nul += 1;
            }
        }
    }

    let is_little_endian = odd_nul >= even_nul;
    let mut units = Vec::with_capacity(bytes.len() / 2);
    let mut chunks = bytes.chunks_exact(2);
    for chunk in &mut chunks {
        let unit = if is_little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(unit);
    }

    String::from_utf16_lossy(&units)
}

#[test]
fn test_code_block_keeps_unicode_whitespace_at_line_starts() {
    let nbsp = '\u{00A0}';
    let html = format!("<pre><code> msg = String()\n{nbsp}msg = String()\n</code></pre>");
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Backticks,
        ..Default::default()
    };

    let markdown = convert(&html, Some(options)).expect("conversion should succeed");

    // ~keep The indentation of a line of code is code (issue #782); it was removed here once.
    assert!(markdown.contains("\n msg = String()\n"));
    assert!(markdown.contains(&format!("\n{nbsp}msg = String()\n")));
}

#[test]
fn test_convert_strips_nul_bytes() {
    let html = "a\0b";
    let markdown = convert(html, None).expect("conversion should succeed");

    assert_eq!(markdown, "ab\n");
}

#[test]
fn converts_all_issue_190_fixtures_except_known_utf16_binary() {
    let fixtures = [
        "firsteigen.html",
        "vipaarontours.html",
        "ozonekorea.html",
        "mitrade.html",
        "plusblog.html",
        "maxkim.html",
        "insight.html",
        "flex2025.html",
        "flex2021.html",
        "kimbrain.html",
        "rbloggers.html",
        "sjsu.html",
    ];

    for name in fixtures {
        let html = read_fixture_lossy(name);
        let markdown =
            convert(&html, None).unwrap_or_else(|err| panic!("fixture {name} should convert cleanly: {err}"));
        assert!(!markdown.trim().is_empty(), "fixture {name} produced empty markdown");
    }
}

#[test]
fn converts_sjsu_fixture_when_lossy_utf8_is_auto_decoded() {
    let bytes = fs::read(fixture_path("sjsu.html")).expect("read sjsu fixture bytes");
    let raw_html = String::from_utf8_lossy(&bytes).into_owned();

    let markdown = convert(&raw_html, None).expect("lossy UTF-16 HTML should be recovered and converted");
    assert!(
        markdown.contains("pipeline") || markdown.contains("Pipeline"),
        "auto-decoded sjsu fixture should contain expected content"
    );
}

#[test]
fn converts_sjsu_fixture_when_decoded_as_utf16() {
    let bytes = fs::read(fixture_path("sjsu.html")).expect("read sjsu fixture bytes");
    let decoded_html = decode_utf16_without_bom(&bytes);

    let markdown = convert(&decoded_html, None).expect("decoded UTF-16 HTML should convert");
    assert!(
        markdown.contains("pipeline") || markdown.contains("Pipeline"),
        "decoded sjsu fixture should contain expected content"
    );
}
