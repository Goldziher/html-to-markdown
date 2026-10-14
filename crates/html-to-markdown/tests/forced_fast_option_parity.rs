//! Forced fast conversion preserves running anchor text and honors selector and image options. ~keep
#![cfg(all(feature = "visitor", feature = "testkit"))]
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::dbg_macro)]

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1::{self, bail::BailReason};
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, convert};
use std::sync::{Arc, Mutex};

const ROUTES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2];
const MARKUP: [OutputFormat; 2] = [OutputFormat::Markdown, OutputFormat::Djot];
const FORMATS: [OutputFormat; 3] = [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain];
const SELECTOR_SOURCE: &str =
    "<p>Bread is baked dough.</p><div class=\"ad\">Buy cheap watches now.</div><p>Cheese is pressed curd.</p>";
const SELECTOR_OUTPUT: &str = "Bread is baked dough.\n\nCheese is pressed curd.\n";

fn options(format: OutputFormat, metadata: bool) -> ConversionOptions {
    ConversionOptions {
        output_format: format,
        extract_metadata: metadata,
        ..Default::default()
    }
}

fn compare(actual: &str, expected: &str, context: &str, failures: &mut Vec<String>) {
    if actual != expected {
        failures.push(format!("{context}: expected {expected:?}, got {actual:?}"));
    }
}

fn public_rows(source: &str, expected: &str, config: &ConversionOptions, failures: &mut Vec<String>) -> usize {
    for route in ROUTES {
        let mut config = config.clone();
        config.tier_strategy = route;
        let actual = convert(source, config)
            .expect("public conversion")
            .content
            .expect("content");
        compare(&actual, expected, &format!("{route:?}: {source}"), failures);
    }
    ROUTES.len()
}

fn fast_row(source: &str, expected: &str, config: &ConversionOptions, failures: &mut Vec<String>) -> usize {
    match tier1::run(source, &PrescanReport::default(), config) {
        Ok(actual) => compare(&actual, expected, &format!("direct scanner: {source}"), failures),
        Err(reason) => failures.push(format!(
            "direct scanner unexpectedly fell back for {source}: {reason:?}"
        )),
    }
    1
}

fn selector_fallback_row(source: &str, config: &ConversionOptions, failures: &mut Vec<String>) -> usize {
    if !matches!(
        tier1::run(source, &PrescanReport::default(), config),
        Err(BailReason::Classifier)
    ) {
        failures.push(format!("nonempty selectors must request fallback: {source}"));
    }
    1
}

fn finish_rows(label: &str, rows: usize, expected: usize, failures: &[String]) {
    println!("ROW_COUNT {label} actual={rows} expected={expected}");
    assert_eq!(rows, expected, "nonzero route coverage");
    assert!(expected > 0);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn list_rows(addressed: bool, failures: &mut Vec<String>) -> usize {
    let mut rows = 0;
    for (payload, label) in [
        ("<strong>rye</strong>", "**rye**"),
        ("<code>rye</code>", "`rye`"),
        ("<img src=\"/rye.png\" alt=\"rye\">", "![rye](/rye.png)"),
    ] {
        let href = if addressed { " href=\"/rye\"" } else { "" };
        let source = format!("<ul><li>Bread<a{href}><br>{payload}</a>today</li></ul>");
        for metadata in [false, true] {
            for newline in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                let mut config = options(OutputFormat::Markdown, metadata);
                config.newline_style = newline;
                let marker = if newline == NewlineStyle::Spaces {
                    "  \n"
                } else {
                    "\\\n"
                };
                let expected = if addressed {
                    format!("- Bread[{marker}  {label}](/rye)today\n")
                } else {
                    format!("- Bread{marker}{label}today\n")
                };
                rows += public_rows(&source, &expected, &config, failures);
                rows += fast_row(&source, &expected, &config, failures);
            }
        }
    }
    rows
}

#[test]
fn should_preserve_unaddressed_list_anchor_children_after_break() {
    let mut failures = Vec::new();
    let rows = list_rows(false, &mut failures);
    finish_rows("unaddressed_break", rows, 48, &failures);
}

#[test]
fn should_keep_addressed_link_continuation_indent() {
    let mut failures = Vec::new();
    let rows = list_rows(true, &mut failures);
    finish_rows("addressed_break", rows, 48, &failures);
}

#[test]
fn should_keep_running_text_indent_and_unbroken_anchor_children() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for (payload, label) in [
        ("<br>rye", None),
        ("<strong>rye</strong>", Some("**rye**")),
        ("<code>rye</code>", Some("`rye`")),
    ] {
        let source = format!("<ul><li>Bread<a>{payload}</a>today</li></ul>");
        for metadata in [false, true] {
            for newline in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                let mut config = options(OutputFormat::Markdown, metadata);
                config.newline_style = newline;
                let marker = if newline == NewlineStyle::Spaces {
                    "  \n"
                } else {
                    "\\\n"
                };
                let expected = label.map_or_else(
                    || format!("- Bread{marker}  ryetoday\n"),
                    |label| format!("- Bread{label}today\n"),
                );
                rows += public_rows(&source, &expected, &config, &mut failures);
                rows += fast_row(&source, &expected, &config, &mut failures);
            }
        }
    }
    finish_rows("running_text", rows, 48, &failures);
}

#[test]
fn should_honor_selectors_in_all_public_routes_and_formats() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in FORMATS {
        for metadata in [false, true] {
            let mut config = options(format, metadata);
            config.exclude_selectors = vec![".ad".into()];
            for (source, expected) in [
                (SELECTOR_SOURCE, SELECTOR_OUTPUT),
                (
                    "<p>Read the <a class=\"ad\" href=\"/buy\">cheap watches</a> guide.</p>",
                    if format == OutputFormat::Plain {
                        "Read the  guide.\n"
                    } else {
                        "Read the guide.\n"
                    },
                ),
                (
                    "<p>Read the <span class=\"ad\">cheap watches </span>guide.</p>",
                    if format == OutputFormat::Plain {
                        "Read the guide.\n"
                    } else {
                        "Read theguide.\n"
                    },
                ),
            ] {
                rows += public_rows(source, expected, &config, &mut failures);
            }
        }
    }
    finish_rows("selectors_public", rows, 54, &failures);
}

#[derive(Debug, Default)]
struct Recorder {
    paragraphs: usize,
}
impl HtmlVisitor for Recorder {
    fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
        self.paragraphs += usize::from(context.tag_name == "p");
        VisitResult::Continue
    }
}

#[test]
fn should_request_selector_fallback_and_run_dom_callbacks() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in FORMATS {
        for metadata in [false, true] {
            let mut config = options(format, metadata);
            config.exclude_selectors = vec![".ad".into()];
            rows += selector_fallback_row(SELECTOR_SOURCE, &config, &mut failures);
            let recorder = Arc::new(Mutex::new(Recorder::default()));
            config.tier_strategy = TierStrategy::Tier1;
            config.visitor = Some(recorder.clone());
            let actual = convert(SELECTOR_SOURCE, config)
                .expect("selector conversion")
                .content
                .expect("content");
            compare(&actual, SELECTOR_OUTPUT, "forced selector fallback", &mut failures);
            let calls = recorder.lock().expect("callback counts").paragraphs;
            let expected_calls = if format == OutputFormat::Plain { 4 } else { 2 };
            if calls != expected_calls {
                failures.push(format!(
                    "{format:?} fallback must match {expected_calls} full-converter paragraph callbacks, got {calls}"
                ));
            }
            rows += 1;
        }
    }
    finish_rows("selector_fallback", rows, 12, &failures);
}

#[test]
fn should_keep_empty_selector_options_on_scanner() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in FORMATS {
        for metadata in [false, true] {
            let config = options(format, metadata);
            assert!(config.exclude_selectors.is_empty());
            rows += public_rows(
                "<p>Bread.</p><p>Cheese.</p>",
                "Bread.\n\nCheese.\n",
                &config,
                &mut failures,
            );
            rows += fast_row(
                "<p>Bread.</p><p>Cheese.</p>",
                "Bread.\n\nCheese.\n",
                &config,
                &mut failures,
            );
        }
    }
    finish_rows("empty_selectors", rows, 24, &failures);
}

#[test]
fn should_skip_images_in_normal_link_and_heading_text() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in MARKUP {
        for metadata in [false, true] {
            for skip in [false, true] {
                let mut config = options(format, metadata);
                config.skip_images = skip;
                for (source, skipped, retained) in [
                    (
                        "<p>a<img src=\"/i.png\" alt=\"pic\">b</p>",
                        "ab\n",
                        "a![pic](/i.png)b\n",
                    ),
                    (
                        "<p>a <img src=\"/i.png\" alt=\"pic\"> b</p>",
                        "a b\n",
                        "a ![pic](/i.png) b\n",
                    ),
                    (
                        "<p><a href=\"/x\">t<img src=\"/i.png\" alt=\"pic\"></a></p>",
                        "[t](/x)\n",
                        "[t![pic](/i.png)](/x)\n",
                    ),
                    ("<h2>a<img src=\"/i.png\" alt=\"pic\">b</h2>", "## ab\n", "## apicb\n"),
                ] {
                    let expected = if skip { skipped } else { retained };
                    rows += public_rows(source, expected, &config, &mut failures);
                    rows += fast_row(source, expected, &config, &mut failures);
                }
            }
        }
    }
    finish_rows("skip_images", rows, 128, &failures);
}

#[test]
fn should_preserve_literal_code_images_and_skip_false() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in MARKUP {
        for metadata in [false, true] {
            for skip in [false, true] {
                let mut config = options(format, metadata);
                config.skip_images = skip;
                let literal = if skip { "ab" } else { "apicb" };
                for (source, expected) in [
                    (
                        "<p><code>a<img src=\"/i.png\" alt=\"pic\">b</code></p>",
                        format!("`{literal}`\n"),
                    ),
                    (
                        "<pre>a<img src=\"/i.png\" alt=\"pic\">b</pre>",
                        format!("```\n{literal}\n```\n"),
                    ),
                ] {
                    rows += public_rows(source, &expected, &config, &mut failures);
                    rows += fast_row(source, &expected, &config, &mut failures);
                }
            }
        }
    }
    finish_rows("code_images", rows, 64, &failures);
}

#[test]
fn should_keep_existing_svg_skip_behavior() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in MARKUP {
        for metadata in [false, true] {
            let mut config = options(format, metadata);
            config.skip_images = true;
            let source = "<p>a<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"1\"/></svg>b</p>";
            rows += public_rows(source, "ab\n", &config, &mut failures);
            rows += fast_row(source, "ab\n", &config, &mut failures);
        }
    }
    finish_rows("svg_images", rows, 16, &failures);
}

#[test]
fn should_preserve_metadata_when_selectors_require_fallback() {
    let mut rows = 0;
    let mut failures = Vec::new();
    let source = "<html><head><title>Bread</title></head><body><p>Keep.</p><p class=\"ad\">Remove.</p></body></html>";
    for format in MARKUP {
        for metadata in [false, true] {
            let mut config = options(format, metadata);
            config.exclude_selectors = vec![".ad".into()];
            let expected = if metadata {
                "---\ntitle: Bread\n---\n\nKeep.\n"
            } else {
                "Keep.\n"
            };
            rows += public_rows(source, expected, &config, &mut failures);
            rows += selector_fallback_row(source, &config, &mut failures);
        }
    }
    finish_rows("selector_metadata", rows, 16, &failures);
}

fn boundary_scanner_row(source: &str, expected: &str, config: &ConversionOptions, failures: &mut Vec<String>) -> usize {
    match tier1::run(source, &PrescanReport::default(), config) {
        Ok(actual) => compare(&actual, expected, &format!("boundary scanner: {source}"), failures),
        Err(BailReason::Classifier) => println!("BOUNDARY_FALLBACK {source}"),
        Err(reason) => failures.push(format!("unexpected boundary fallback for {source}: {reason:?}")),
    }
    1
}

#[test]
fn should_preserve_unaddressed_anchor_owner_edges_and_repeated_breaks() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for metadata in [false, true] {
        for newline in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            let mut config = options(OutputFormat::Markdown, metadata);
            config.newline_style = newline;
            let marker = if newline == NewlineStyle::Spaces {
                "  \n"
            } else {
                "\\\n"
            };
            let repeated = if newline == NewlineStyle::Spaces {
                "- Bread  \n\n**rye**today\n"
            } else {
                "- Bread\\\n  \\\n**rye**today\n"
            };
            for (source, expected, supports_fallback) in [
                (
                    "<ul><li><a><br><strong>rye</strong></a>today</li></ul>",
                    if newline == NewlineStyle::Spaces {
                        "-  \n**rye**today\n".into()
                    } else {
                        "- \\\n**rye**today\n".into()
                    },
                    false,
                ),
                (
                    "<p><a><br><strong>rye</strong></a>today</p>",
                    "**rye**today\n".into(),
                    true,
                ),
                (
                    "<ul><li>Bread<a><br><br><strong>rye</strong></a>today</li></ul>",
                    repeated.into(),
                    true,
                ),
                (
                    "<ul><li>Bread<a><br><strong><em>rye</em></strong></a>today</li></ul>",
                    format!("- Bread{marker}***rye***today\n"),
                    false,
                ),
            ] {
                rows += public_rows(source, &expected, &config, &mut failures);
                rows += if supports_fallback {
                    boundary_scanner_row(source, &expected, &config, &mut failures)
                } else {
                    fast_row(source, &expected, &config, &mut failures)
                };
            }
        }
    }
    finish_rows("anchor_boundaries", rows, 64, &failures);
}

#[test]
fn should_keep_whitespace_between_skipped_images_and_code_neighbors() {
    let mut rows = 0;
    let mut failures = Vec::new();
    for format in MARKUP {
        for metadata in [false, true] {
            for newline in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                let mut config = options(format, metadata);
                config.newline_style = newline;
                config.skip_images = true;
                for (source, expected) in [
                    (
                        "<p>a<img src=\"/i.png\" alt=\"pic\"> <img src=\"/j.png\" alt=\"pic\">b</p>",
                        "a b\n",
                    ),
                    ("<p>a <code><img src=\"/i.png\" alt=\"pic\"></code> b</p>", "a b\n"),
                    (
                        "<p>a <code>x <img src=\"/i.png\" alt=\"pic\"> y</code> b</p>",
                        "a `x  y` b\n",
                    ),
                ] {
                    rows += public_rows(source, expected, &config, &mut failures);
                    rows += fast_row(source, expected, &config, &mut failures);
                }
            }
        }
    }
    finish_rows("image_whitespace", rows, 96, &failures);
}
