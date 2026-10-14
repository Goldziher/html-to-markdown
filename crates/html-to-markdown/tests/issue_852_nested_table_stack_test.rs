#![allow(missing_docs)]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};
const CHILD: &str = "HTML_TO_MARKDOWN_NESTED_TABLE_CHILD";
#[test]
fn should_preserve_nested_tables_on_a_two_megabyte_stack() {
    if std::env::var_os(CHILD).is_some() {
        let cases = [
            ("<table><tr><td>", "</td></tr></table>"),
            ("<table><tr><th>head</th></tr><tr><td>", "</td></tr></table>"),
            ("<table><tr><td>side</td><td>", "</td></tr></table>"),
            (
                r#"<table border="0"><tr><td colspan="2">"#,
                "</td><td>side</td></tr></table>",
            ),
        ];
        assert_eq!(cases.len(), 4);
        for (open, close) in cases {
            for whitespace_mode in [
                html_to_markdown_rs::WhitespaceMode::Normalized,
                html_to_markdown_rs::WhitespaceMode::Strict,
            ] {
                let html = format!("{}payload{}", open.repeat(200), close.repeat(200));
                let expected_html = html.clone();
                let options = ConversionOptions {
                    max_depth: Some(1024),
                    extract_metadata: false,
                    whitespace_mode,
                    tier_strategy: TierStrategy::Tier2,
                    ..Default::default()
                };
                let expected_options = options.clone();
                let expected = std::thread::Builder::new()
                    .stack_size(32 * 1024 * 1024)
                    .spawn(move || convert(&expected_html, expected_options).expect("reference conversion succeeds"))
                    .expect("reference thread starts")
                    .join()
                    .expect("reference returns");
                assert!(expected.warnings.is_empty());
                assert_eq!(
                    expected
                        .content
                        .as_deref()
                        .unwrap_or_default()
                        .matches("payload")
                        .count(),
                    1
                );
                let actual = std::thread::Builder::new()
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || convert(&html, options).expect("small-stack conversion succeeds"))
                    .expect("small-stack thread starts")
                    .join()
                    .expect("small-stack returns");
                assert_eq!(actual.content, expected.content);
                assert!(actual.warnings.is_empty());
            }
        }
        return;
    }
    // ~keep A native stack overflow aborts, so the regression runs in an isolated process.
    let child = std::process::Command::new(std::env::current_exe().expect("test executable exists"))
        .args([
            "--exact",
            "should_preserve_nested_tables_on_a_two_megabyte_stack",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output()
        .expect("isolated process starts");
    assert!(
        child.status.success(),
        "child status: {}; stderr: {}",
        child.status,
        String::from_utf8_lossy(&child.stderr)
    );
}
