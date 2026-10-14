// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Form controls (issues 752 and 757), the checkbox: a checkbox writes task brackets only as the
//! marker of a task item, and writes nothing in every other place.

use html_to_markdown_rs::options::PreprocessingOptions;
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::tier1::{self, BailReason};
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

/// Options that let the fast converter run, with forms kept.
fn options(tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy,
        preprocessing: PreprocessingOptions {
            remove_forms: false,
            ..PreprocessingOptions::default()
        },
        ..ConversionOptions::default()
    }
}

fn markdown(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn fast_converter(html: &str) -> Result<String, BailReason> {
    tier1::run(html, &PrescanReport::default(), &options(TierStrategy::Tier1))
}

/// Asserts `expected` for `html` on every path: the full converter, the fast converter with its
/// fallback, the router, and the default options with forms kept. The fast converter alone must
/// write the same text or hand the input over.
fn assert_all_paths(html: &str, expected: &str) {
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1, TierStrategy::Auto] {
        assert_eq!(
            markdown(html, options(tier_strategy)),
            expected,
            "{tier_strategy:?}: {html}"
        );
    }
    let defaults = ConversionOptions {
        preprocessing: options(TierStrategy::Auto).preprocessing,
        ..ConversionOptions::default()
    };
    assert_eq!(markdown(html, defaults), expected, "default options: {html}");
    if let Ok(fast) = fast_converter(html) {
        assert_eq!(fast, expected, "fast converter alone: {html}");
    }
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    comrak::markdown_to_html(markdown, &options)
}

#[test]
fn should_write_task_brackets_only_for_a_list_item_that_starts_with_a_checkbox() {
    for (html, expected) in [
        (
            r#"<ul><li><input type="checkbox"> open task</li><li><input type="checkbox" checked> done task</li></ul>"#,
            "- [ ] open task\n- [x] done task\n",
        ),
        (
            r#"<ul><li><input type="checkbox"> outer<ul><li><input type="checkbox" checked> inner</li></ul></li></ul>"#,
            "- [ ] outer\n  - [x] inner\n",
        ),
        (
            r#"<ul><li><label><input type="checkbox" checked> Item</label></li></ul>"#,
            "- [x] Item\n",
        ),
        (
            r#"<ol><li><input type="checkbox" checked> first</li><li>second</li></ol>"#,
            "1. [x] first\n2. second\n",
        ),
        // ~keep A second checkbox in a task item, and a checkbox after other content of an
        // ~keep item, are controls in the text.
        (
            r#"<ul><li><input type="checkbox"> a <input type="checkbox" checked> b</li></ul>"#,
            "- [ ] a b\n",
        ),
        (
            r#"<ul><li>Text <input type="checkbox"> more</li></ul>"#,
            "- Text more\n",
        ),
        (r#"<ul><li>Text <input type="checkbox" checked></li></ul>"#, "- Text\n"),
        (
            r#"<ul><li><img src="i.png" alt="pic"><input type="checkbox" checked> shown</li></ul>"#,
            "- ![pic](i.png) shown\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_write_nothing_for_a_checkbox_outside_a_task_item() {
    for (html, expected) in [
        (
            r#"<p>Agree <input type="checkbox" name="a"> to the terms</p>"#,
            "Agree to the terms\n",
        ),
        (
            r#"<input type="checkbox" id="nav-toggle"><label for="nav-toggle">Menu</label><p>text</p>"#,
            "Menu\n\ntext\n",
        ),
        (
            "<input type=\"checkbox\" id=\"nav-toggle\">\n<label for=\"nav-toggle\">Menu</label>\n<p>text</p>",
            "Menu\n\ntext\n",
        ),
        (
            "<p>Agree<input type=\"checkbox\">to the terms</p>",
            "Agreeto the terms\n",
        ),
        (r#"<h2><input type="checkbox" checked> Title</h2>"#, "## Title\n"),
        (
            r#"<a href="/u"><input type="checkbox" checked> label</a>"#,
            "[label](/u)\n",
        ),
        (
            r#"<label><input type="checkbox" checked> Remember me</label>"#,
            "Remember me\n",
        ),
        (
            r#"<label for="c">Remember me</label><input type="checkbox" id="c">"#,
            "Remember me\n",
        ),
        (r#"<p><input type="checkbox"> starts</p>"#, "starts\n"),
        (r#"<p>a <INPUT TYPE="CHECKBOX" CHECKED> b</p>"#, "a b\n"),
        (r#"<p><input type="checkbox"><input type="checkbox" checked></p>"#, ""),
        (
            r#"<blockquote><input type="checkbox"> quoted</blockquote>"#,
            "> quoted\n",
        ),
        (r#"<p>a <input type="checkbox" disabled checked> b</p>"#, "a b\n"),
        (
            r#"<p>Dark mode<input type="checkbox" role="switch" checked>.</p>"#,
            "Dark mode.\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_write_nothing_for_a_checkbox_that_has_the_role_of_a_button() {
    for (html, expected) in [
        (
            r#"<input type="checkbox" id="m" role="button" aria-haspopup="true" aria-label="Main menu"><label for="m">Main menu</label>"#,
            "Main menu\n",
        ),
        (
            r#"<ul><li><input type="checkbox" role="button"> Menu</li></ul>"#,
            "- Menu\n",
        ),
        // ~keep The white space on both sides of a checkbox that writes nothing is one space.
        (r#"<p>a <input type="checkbox" role="button"> b</p>"#, "a b\n"),
        (r#"<p>a <input type="checkbox"><!-- c --> b</p>"#, "a b\n"),
        (
            r#"<pre>a <input type="checkbox" role="button"> b</pre>"#,
            "```\na  b\n```\n",
        ),
        (
            r#"<table><tr><td><input type="checkbox" role="button"></td></tr></table>"#,
            "",
        ),
        (
            r#"<table><tr><td><input type="checkbox" role="checkbox"></td></tr></table>"#,
            "| [ ] |\n| --- |\n",
        ),
    ] {
        assert_all_paths(html, expected);
    }
}

#[test]
fn should_not_turn_a_checkbox_into_a_link() {
    // ~keep With brackets each of these rendered as a link: `[x](optional)` and `[x]` beside
    // ~keep a `[x]:` definition. A checkbox beside text writes nothing.
    for (html, expected, rendered_text) in [
        (
            r#"<p><input type="checkbox" checked>(optional)</p>"#,
            "(optional)\n",
            "<p>(optional)</p>\n",
        ),
        (
            r#"<p><input type="checkbox" checked> on</p><p>[x]: /target</p>"#,
            "on\n\n[x]: /target\n",
            "<p>on</p>\n",
        ),
        (
            r#"<table><tr><th>Done</th></tr><tr><td><input type="checkbox" checked>(late)</td></tr></table>"#,
            "| Done   |\n| ------ |\n| (late) |\n",
            "<table>\n<thead>\n<tr>\n<th>Done</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>(late)</td>\n</tr>\n</tbody>\n</table>\n",
        ),
    ] {
        assert_all_paths(html, expected);
        let rendered = render(expected);
        assert_eq!(rendered, rendered_text, "{html}");
        assert!(!rendered.contains("<a "), "{html}: {rendered}");
    }
    // ~keep The renderer does make a link of the bracket form, so the assertion above can fail.
    assert!(render("[x](optional)\n").contains("<a "));
}
