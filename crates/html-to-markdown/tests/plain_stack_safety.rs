#![allow(missing_docs)]

use html_to_markdown_rs::convert;
use html_to_markdown_rs::options::{ConversionOptions, OutputFormat};
use std::process::Command;

fn nested(open: &str, close: &str, depth: usize, leaf: &str) -> String {
    format!("{}{leaf}{}", open.repeat(depth), close.repeat(depth))
}

fn convert_on_small_stack(html: String) -> String {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            convert(
                &html,
                Some(ConversionOptions {
                    output_format: OutputFormat::Plain,
                    max_depth: Some(1024),
                    ..Default::default()
                }),
            )
            .expect("convert plain text")
            .content
            .expect("plain content")
        })
        .expect("spawn two MiB thread")
        .join()
        .expect("conversion thread succeeds")
}

#[test]
fn should_preserve_deep_plain_text_on_a_two_mib_thread_stack() {
    // ~keep Stack overflow aborts the process, so a child contains a regression without killing other tests.
    if let Ok(case) = std::env::var("PLAIN_STACK_CASE") {
        let (open, close, leaf, expected) = match case.as_str() {
            "div" => ("<div>", "</div>", "deep", "deep\n"),
            "span" => ("<span>", "</span>", "deep", "deep\n"),
            "emphasis" => ("<em>", "</em>", "deep", "deep\n"),
            "link" => ("<a href='/x'><span>", "</span></a>", "deep", "deep\n"),
            "ruby" => ("<ruby>", "</ruby>", "deep", "deep\n"),
            "code" => ("<code>", "</code>", "deep", "deep\n"),
            "captures" => (
                "<span>",
                "</span>",
                "<table><caption>C</caption><tr><td>x<button>y</button>z</td><td><select><option>A</option><option>B</option></select></td></tr></table><ol start='3'><li>one</li><li>two</li></ol>",
                "C\n\nx y z\tA B\n\n3. one\n4. two\n",
            ),
            _ => panic!("unknown case {case}"),
        };
        let depth = if case == "link" { 500 } else { 1000 };
        assert_eq!(convert_on_small_stack(nested(open, close, depth, leaf)), expected);
        return;
    }
    for case in ["div", "span", "emphasis", "link", "ruby", "code", "captures"] {
        let result = Command::new(std::env::current_exe().expect("current test executable"))
            .args([
                "--exact",
                "should_preserve_deep_plain_text_on_a_two_mib_thread_stack",
                "--nocapture",
            ])
            .env("PLAIN_STACK_CASE", case)
            .output()
            .expect("run isolated stack probe");
        assert!(
            result.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
