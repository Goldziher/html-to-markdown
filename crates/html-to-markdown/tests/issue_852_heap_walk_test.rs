#![allow(missing_docs, clippy::print_stderr)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

const DEPTH: usize = 1_000;
const STACK_BYTES: usize = 2 * 1024 * 1024;
const CHILD_MARKER: &str = "HTML_TO_MARKDOWN_SMALL_STACK_CHILD";

fn nested(open: &str, close: &str) -> String {
    format!("{}payload{}", open.repeat(DEPTH), close.repeat(DEPTH))
}

#[test]
fn should_preserve_deep_dom_content_on_a_two_megabyte_stack() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        std::thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn(|| {
                let cases = [
                    (nested("<fieldset>", "</fieldset>"), "payload\n"),
                    (nested("<details>", "</details>"), "payload\n"),
                    (nested("<figure>", "</figure>"), "payload\n"),
                    (nested("<label>", "</label>"), "payload\n"),
                    (nested("<div>", "</div>"), "payload\n"),
                    (nested("<span>", "</span>"), "payload\n"),
                    (nested("<strong>", "</strong>"), "**payload**\n"),
                    (
                        format!("<a href=\"/deep\">{}</a>", nested("<span>", "</span>")),
                        "[payload](/deep)\n",
                    ),
                    (nested("<ruby>", "</ruby>"), "payload\n"),
                    (nested("<code>", "</code>"), "`payload`\n"),
                    (
                        format!("<ul><li>{}</li></ul>", nested("<span>", "</span>")),
                        "- payload\n",
                    ),
                    (
                        format!("<table><tr><td>{}</td></tr></table>", nested("<span>", "</span>")),
                        "| payload |\n| ------- |\n",
                    ),
                ];
                assert_eq!(cases.len(), 12);
                for (html, expected) in cases {
                    let result = convert(
                        &html,
                        ConversionOptions {
                            max_depth: Some(1_024),
                            extract_metadata: false,
                            tier_strategy: TierStrategy::Tier2,
                            ..Default::default()
                        },
                    )
                    .expect("deep conversion succeeds");
                    assert_eq!(
                        result.content.as_deref(),
                        Some(expected),
                        "input prefix: {:?}",
                        &html[..32]
                    );
                    assert!(result.warnings.is_empty(), "the raised limit preserves every node");
                }
            })
            .expect("conversion thread starts")
            .join()
            .expect("conversion finishes without a panic");
        return;
    }
    // ~keep Stack overflow aborts the process; isolation makes it an ordinary failing regression.
    let output = std::process::Command::new(std::env::current_exe().expect("test executable exists"))
        .args([
            "--exact",
            "should_preserve_deep_dom_content_on_a_two_megabyte_stack",
            "--nocapture",
        ])
        .env(CHILD_MARKER, "1")
        .output()
        .expect("isolated test starts");
    assert!(
        output.status.success(),
        "child status: {}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn should_preserve_nested_container_bytes_on_a_two_megabyte_stack() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        let cases = [
            nested("<section>", "</section>"),
            nested("<blockquote>", "</blockquote>"),
            format!("{}payload{}", "<ul><li>".repeat(400), "</li></ul>".repeat(400)),
            format!(
                "{}payload{}",
                "<ul><li><input type=\"checkbox\">".repeat(400),
                "</li></ul>".repeat(400)
            ),
            format!("{}payload{}", "<dl><dd>".repeat(400), "</dd></dl>".repeat(400)),
            format!(
                "{}payload{}",
                "<table><tr><td>".repeat(200),
                "</td></tr></table>".repeat(200)
            ),
        ];
        assert_eq!(cases.len(), 6);
        for (case, html) in cases.into_iter().enumerate() {
            eprintln!("nested container case {case}");
            let options = ConversionOptions {
                max_depth: Some(1_024),
                extract_metadata: false,
                tier_strategy: TierStrategy::Tier2,
                ..Default::default()
            };
            let expected_html = html.clone();
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
                .stack_size(STACK_BYTES)
                .spawn(move || convert(&html, options).expect("small-stack conversion succeeds"))
                .expect("small-stack thread starts")
                .join()
                .expect("small-stack returns");
            assert_eq!(actual.content, expected.content);
            assert!(actual.warnings.is_empty());
        }
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable exists"))
        .args([
            "--exact",
            "should_preserve_nested_container_bytes_on_a_two_megabyte_stack",
            "--nocapture",
        ])
        .env(CHILD_MARKER, "1")
        .output()
        .expect("isolated test starts");
    assert!(
        output.status.success(),
        "child status: {}; stdout: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn should_preserve_remaining_container_bytes_on_a_two_megabyte_stack() {
    const CASE_MARKER: &str = "HTML_TO_MARKDOWN_CONTAINER_CASE";
    let cases = [
        ("<form>", "</form>"),
        ("<summary>", "</summary>"),
        ("<legend>", "</legend>"),
        ("<pre>", "</pre>"),
    ];
    assert_eq!(cases.len(), 4);
    if let Some(case) = std::env::var_os(CASE_MARKER) {
        let case: usize = case
            .to_str()
            .expect("case is UTF-8")
            .parse()
            .expect("case index exists");
        let (open, close) = cases[case];
        let html = nested(open, close);
        let mut options = ConversionOptions {
            max_depth: Some(1_024),
            extract_metadata: false,
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        };
        options.preprocessing.remove_forms = false;
        let reference_html = html.clone();
        let reference_options = options.clone();
        let expected = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(move || convert(&reference_html, reference_options).expect("reference conversion succeeds"))
            .expect("reference thread starts")
            .join()
            .expect("reference returns");
        assert!(expected.warnings.is_empty(), "{open}: {:?}", expected.warnings);
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
            .stack_size(STACK_BYTES)
            .spawn(move || convert(&html, options).expect("small-stack conversion succeeds"))
            .expect("small-stack thread starts")
            .join()
            .expect("small-stack returns");
        assert_eq!(actual.content, expected.content);
        assert!(actual.warnings.is_empty());
        return;
    }
    let mut failures = Vec::new();
    for (case, (open, _)) in cases.into_iter().enumerate() {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable exists"))
            .args([
                "--exact",
                "should_preserve_remaining_container_bytes_on_a_two_megabyte_stack",
                "--nocapture",
            ])
            .env(CASE_MARKER, case.to_string())
            .output()
            .expect("isolated test starts");
        if !output.status.success() {
            failures.push(format!(
                "{open}: {}; stdout: {}; stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn should_preserve_recursive_specialized_containers_on_a_two_megabyte_stack() {
    const CASE_MARKER: &str = "HTML_TO_MARKDOWN_SPECIALIZED_CASE";
    let cases = ["h1", "figcaption", "menu", "option"];
    assert_eq!(cases.len(), 4);
    if let Some(case) = std::env::var_os(CASE_MARKER) {
        let case: usize = case.to_str().expect("UTF-8 case").parse().expect("case index");
        let name = cases[case];
        let html = nested(&format!("<{name}>"), &format!("</{name}>"));
        let options = ConversionOptions {
            max_depth: Some(1_024),
            extract_metadata: false,
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        };
        let reference_html = html.clone();
        let reference_options = options.clone();
        let expected = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(move || convert(&reference_html, reference_options).expect("reference conversion"))
            .expect("reference thread")
            .join()
            .expect("reference result");
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
            .stack_size(STACK_BYTES)
            .spawn(move || convert(&html, options).expect("small-stack conversion"))
            .expect("small-stack thread")
            .join()
            .expect("small-stack result");
        assert_eq!(actual.content, expected.content);
        assert!(actual.warnings.is_empty());
        return;
    }
    let mut failures = Vec::new();
    for (case, name) in cases.into_iter().enumerate() {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "should_preserve_recursive_specialized_containers_on_a_two_megabyte_stack",
                "--nocapture",
            ])
            .env(CASE_MARKER, case.to_string())
            .output()
            .expect("isolated regression starts");
        if !output.status.success() {
            failures.push(format!(
                "{name}: {}; {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
