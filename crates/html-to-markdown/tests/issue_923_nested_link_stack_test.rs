#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, convert};

const NESTED_PAIRS: usize = 20_000;
const STACK_BYTES: usize = 2 * 1024 * 1024;
const CHILD_MARKER: &str = "HTML_TO_MARKDOWN_ISSUE_923_CHILD";
const TEST_NAME: &str = "should_convert_deep_block_links_on_a_two_megabyte_stack";

fn convert_on_small_stack(max_depth: Option<usize>) {
    let html = format!(
        "<html><body>{}deep{}<p>tail</p></body></html>",
        r##"<div><a href="#x">"##.repeat(NESTED_PAIRS),
        "</a></div>".repeat(NESTED_PAIRS),
    );
    let result = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            convert(
                &html,
                ConversionOptions {
                    max_depth,
                    ..ConversionOptions::default()
                },
            )
            .expect("deep link conversion succeeds")
        })
        .expect("small-stack thread starts")
        .join()
        .expect("small-stack conversion does not panic");
    let content = result.content.expect("content is requested");
    assert_eq!(content.matches("tail").count(), 1);
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn should_convert_deep_block_links_on_a_two_megabyte_stack() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        let cases = [None, Some(1024)];
        assert_eq!(cases.len(), 2);
        for max_depth in cases {
            convert_on_small_stack(max_depth);
        }
        return;
    }
    // ~keep Stack overflow aborts rather than unwinds, so only an isolated child may reproduce it.
    let child = std::process::Command::new(std::env::current_exe().expect("test executable exists"))
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(CHILD_MARKER, "1")
        .output()
        .expect("isolated regression starts");
    let stdout = String::from_utf8_lossy(&child.stdout);
    assert!(
        stdout.contains("running 1 test"),
        "subprocess must execute one regression: {stdout}"
    );
    assert!(
        child.status.success(),
        "child status: {}; stderr: {}",
        child.status,
        String::from_utf8_lossy(&child.stderr)
    );
}

#[test]
fn should_keep_link_labels_and_defer_tables_buried_in_block_children() {
    for (html, expected) in [
        (r#"<a href="/o"><div>Text</div></a>"#, "[Text](/o)\n"),
        (
            r#"<a href="/o"><div><span></span><section><table><tr><td>Cell</td></tr></table></section></div></a>"#,
            "[/o](/o)\n\n| Cell |\n| ---- |\n",
        ),
    ] {
        let actual = convert(html, ConversionOptions::default())
            .expect("shallow link conversion succeeds")
            .content
            .expect("content is requested");
        assert_eq!(actual, expected, "{html}");
    }
}
