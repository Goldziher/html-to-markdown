//! Integration tests for the CLI flags that expose `ConversionOptions` fields which
//! were previously hardcoded in `convert.rs`: `--exclude-selectors`,
//! `--url-escape-style`, `--max-image-size`, `--capture-svg`, `--no-infer-dimensions`
//! and `--tier-strategy`.
//!
//! Split out of `cli_test.rs` to keep both files under the 1000-line lint limit.

use assert_cmd::Command;
use predicates::prelude::*;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_html-to-markdown"))
}

#[test]
fn should_omit_the_abbreviation_title_with_the_flag() {
    let html = "<p><abbr title=\"last-in, first-out\">LIFO</abbr></p>";
    cli()
        .write_stdin(html)
        .assert()
        .success()
        .stdout("LIFO (last-in, first-out)\n");
    cli()
        .arg("--no-expand-abbreviations")
        .write_stdin(html)
        .assert()
        .success()
        .stdout("LIFO\n");
}

#[test]
fn should_omit_the_blockquote_citation_with_the_flag() {
    let html = "<blockquote cite=\"https://example.com/src\"><p>quoted words</p></blockquote>";
    cli()
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicates::str::contains("https://example.com/src"));
    cli()
        .arg("--no-blockquote-citations")
        .write_stdin(html)
        .assert()
        .success()
        .stdout("> quoted words\n");
}

// A well-known 1x1 transparent PNG, 70 bytes when base64-decoded.
const TINY_PNG_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[test]
fn should_remove_matching_element_and_descendants_when_single_exclude_selector_is_given() {
    cli()
        .arg("--exclude-selectors")
        .arg(".ad")
        .write_stdin(r#"<p>Before</p><div class="ad"><p>Ad content</p></div><p>After</p>"#)
        .assert()
        .success()
        .stdout("Before\n\nAfter\n");
}

#[test]
fn should_remove_all_matching_elements_when_comma_separated_exclude_selectors_are_given() {
    cli()
        .arg("--exclude-selectors")
        .arg(".ad,#sidebar")
        .write_stdin(
            r#"<p>Before</p><div class="ad"><p>Ad content</p></div><div id="sidebar"><p>Sidebar content</p></div><p>After</p>"#,
        )
        .assert()
        .success()
        .stdout("Before\n\nAfter\n");
}

#[test]
fn should_wrap_destination_with_space_in_angle_brackets_by_default() {
    cli()
        .write_stdin(r#"<p><a href="/a path/page.html">Link</a></p>"#)
        .assert()
        .success()
        .stdout("[Link](</a path/page.html>)\n");
}

#[test]
fn should_percent_encode_destination_with_space_when_url_escape_style_is_percent() {
    cli()
        .arg("--url-escape-style")
        .arg("percent")
        .write_stdin(r#"<p><a href="/a path/page.html">Link</a></p>"#)
        .assert()
        .success()
        .stdout("[Link](/a%20path/page.html)\n");
}

#[test]
fn should_write_the_alt_text_for_a_data_image_when_inline_data_media_is_alt_text_only() {
    cli()
        .arg("--inline-data-media")
        .arg("alt-text-only")
        .write_stdin(r#"<p><img src="data:image/png;base64,AAAA" alt="icon"></p>"#)
        .assert()
        .success()
        .stdout("icon\n");
}

#[test]
fn should_reject_invalid_inline_data_media_value() {
    cli()
        .arg("--inline-data-media")
        .arg("strip")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn should_drop_a_hidden_element_by_default() {
    cli()
        .write_stdin("<p>visible</p><div hidden>tab two body</div><template><p>row</p></template>")
        .assert()
        .success()
        .stdout("visible\n");
}

#[test]
fn should_keep_a_hidden_element_when_hidden_content_is_reachable() {
    cli()
        .arg("--hidden-content")
        .arg("reachable")
        .write_stdin("<p>visible</p><div hidden>tab two body</div><template><p>row</p></template>")
        .assert()
        .success()
        .stdout("visible\n\ntab two body\n");
}

#[test]
fn should_keep_template_content_when_hidden_content_is_all() {
    cli()
        .arg("--hidden-content")
        .arg("all")
        .write_stdin("<p>visible</p><div hidden>tab two body</div><template><p>row</p></template>")
        .assert()
        .success()
        .stdout("visible\n\ntab two body\n\nrow\n");
}

#[test]
fn should_reject_invalid_hidden_content_value() {
    cli()
        .arg("--hidden-content")
        .arg("keep")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn should_reject_invalid_url_escape_style_value() {
    cli()
        .arg("--url-escape-style")
        .arg("bogus")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn should_include_image_in_json_output_when_max_image_size_is_default() {
    let html = format!(r#"<img src="{TINY_PNG_DATA_URI}" alt="Tiny">"#);

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"format\": \"png\""));
}

#[test]
fn should_produce_empty_images_array_when_max_image_size_is_smaller_than_decoded_image() {
    let html = format!(r#"<img src="{TINY_PNG_DATA_URI}" alt="Tiny">"#);

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .arg("--max-image-size")
        .arg("10")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"images\": []"));
}

#[test]
fn should_not_capture_inline_svg_when_capture_svg_is_unset() {
    let html = r#"<p>text</p><svg width="10" height="10"><rect width="10" height="10"/></svg>"#;

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"images\": []"));
}

#[test]
fn should_capture_inline_svg_as_image_when_capture_svg_is_set() {
    let html = r#"<p>text</p><svg width="10" height="10"><rect width="10" height="10"/></svg>"#;

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .arg("--capture-svg")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"source\": \"svg_element\""));
}

#[test]
fn should_populate_dimensions_by_default_when_extracting_inline_image() {
    let html = format!(r#"<img src="{TINY_PNG_DATA_URI}" alt="Tiny">"#);

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"dimensions\": {\n        \"height\": 1,\n        \"width\": 1\n      }",
        ));
}

#[test]
fn should_null_dimensions_when_no_infer_dimensions_is_set() {
    let html = format!(r#"<img src="{TINY_PNG_DATA_URI}" alt="Tiny">"#);

    cli()
        .arg("--json")
        .arg("--extract-inline-images")
        .arg("--no-infer-dimensions")
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dimensions\": null"));
}

#[test]
fn should_reject_capture_svg_without_extract_inline_images() {
    cli()
        .arg("--json")
        .arg("--capture-svg")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--extract-inline-images"));
}

#[test]
fn should_reject_max_image_size_without_extract_inline_images() {
    cli()
        .arg("--json")
        .arg("--max-image-size")
        .arg("1024")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--extract-inline-images"));
}

#[test]
fn should_reject_no_infer_dimensions_without_extract_inline_images() {
    cli()
        .arg("--json")
        .arg("--no-infer-dimensions")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--extract-inline-images"));
}

#[test]
fn should_produce_identical_output_for_tier_strategy_auto_and_tier2() {
    let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>\
        <ul><li>Item 1</li><li>Item 2</li></ul>\
        <table><tr><td>A</td><td>B</td></tr></table>";

    let auto_output = cli()
        .arg("--tier-strategy")
        .arg("auto")
        .write_stdin(html)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tier2_output = cli()
        .arg("--tier-strategy")
        .arg("tier2")
        .write_stdin(html)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        auto_output, tier2_output,
        "tier-strategy auto and tier2 must produce identical output for the same input"
    );
}

#[test]
fn should_reject_tier1_as_a_tier_strategy_cli_value() {
    cli()
        .arg("--tier-strategy")
        .arg("tier1")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn every_flag_in_the_help_is_listed_in_the_plugin_cli_reference() {
    let words = |text: &str| -> std::collections::BTreeSet<String> {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .filter(|word| {
                word.strip_prefix("--")
                    .is_some_and(|name| name.starts_with(|c: char| c.is_ascii_alphabetic()))
            })
            .map(str::to_string)
            .collect()
    };
    let help = cli().arg("--help").output().expect("run --help");
    assert!(help.status.success());
    let flags = words(&String::from_utf8(help.stdout).expect("utf-8 help"));
    assert!(flags.len() > 50, "too few flags parsed from --help: {flags:?}");

    let reference_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../plugin/.ai-rulez/references/cli-reference.md"
    );
    let listed = words(&std::fs::read_to_string(reference_path).expect("read the plugin CLI reference"));
    let missing: Vec<&String> = flags.difference(&listed).collect();
    assert!(missing.is_empty(), "flags missing from {reference_path}: {missing:?}");
}
