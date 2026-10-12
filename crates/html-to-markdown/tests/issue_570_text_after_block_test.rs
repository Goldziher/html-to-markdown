// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issues #570 and #571: inline content right after a list or a table, in
//! the same container, continued the block's last line. After a list it became a lazy
//! continuation of the last item (`- A\npara`); after a table it became another table row.
//!
//! In HTML, inline content that follows a block in the same container starts a new block of its
//! own, so every block type followed by every kind of inline content must render that content as
//! a paragraph after the block.

use html_to_markdown_rs::options::{NewlineStyle, WhitespaceMode};
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert, tier1};

const LAYOUT_TABLE: &str = r#"<table><tr><td><a href="1">a</a> <a href="2">b</a> <a href="3">c</a></td></tr></table>"#;

const GFM_TABLE: &str = "<table><tr><th>h</th><th>g</th></tr><tr><td>c</td><td>d</td></tr></table>";

/// Blocks that end their last line with a single newline, then the ones that already end with a
/// blank line, as controls.
const BLOCKS: [&str; 13] = [
    "<ul><li>A</li></ul>",
    "<ul><li>A<br></li></ul>",
    "<ol><li>A</li></ol>",
    "<ul><li>A<ul><li>B</li></ul></li></ul>",
    LAYOUT_TABLE,
    GFM_TABLE,
    "<hr>",
    "<blockquote>A</blockquote>",
    "<pre>A</pre>",
    "<h2>A</h2>",
    "<dl><dt>T</dt><dd>D</dd></dl>",
    "<p>A</p>",
    "<div>A</div>",
];

/// Inline content after the block, with the paragraph a `CommonMark` renderer must produce for it.
const FOLLOWERS: [(&str, &str); 7] = [
    ("para", "<p>para</p>"),
    ("\npara", "<p>para</p>"),
    ("<span>para</span>", "<p>para</p>"),
    ("<b>para</b>", "<p><strong>para</strong></p>"),
    (r#"<a href="x">para</a>"#, r#"<p><a href="x">para</a></p>"#),
    (r#"<img src="i.png" alt="x">"#, r#"<p><img src="i.png" alt="x" /></p>"#),
    ("<code>para</code>", "<p><code>para</code></p>"),
];

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn tier2(html: &str) -> String {
    convert(html, Some(tier2_options()))
        .expect("tier2 conversion must succeed")
        .content
        .unwrap_or_default()
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    comrak::markdown_to_html(markdown, &options)
}

/// Every block, then every follower, inside a `<div>`, at the document root, and inside a
/// `<blockquote>`, with the rendered HTML the follower's paragraph must close.
fn cases() -> Vec<(String, String)> {
    let mut cases = Vec::new();
    for block in BLOCKS {
        for (follower, paragraph) in FOLLOWERS {
            cases.push((format!("<div>{block}{follower}</div>"), format!("{paragraph}\n")));
            cases.push((format!("{block}{follower}"), format!("{paragraph}\n")));
            cases.push((
                format!("<blockquote>{block}{follower}</blockquote>"),
                format!("{paragraph}\n</blockquote>\n"),
            ));
        }
    }
    cases
}

#[test]
fn should_render_text_after_a_list_as_a_paragraph_after_the_list() {
    let out = tier2("<div><ul><li>A</li></ul>para</div>");
    assert_eq!(out, "- A\n\npara\n", "the text must start a paragraph after the list");
    assert!(
        render(&out).ends_with("</ul>\n<p>para</p>\n"),
        "para must render outside the list item: {:?}",
        render(&out)
    );
}

#[test]
fn should_render_text_after_a_table_as_a_paragraph_not_a_table_row() {
    let out = tier2(&format!("<div>{GFM_TABLE}para</div>"));
    assert_eq!(
        out, "| h | g |\n| --- | --- |\n| c | d |\n\npara\n",
        "the text must start a paragraph after the table"
    );
    assert!(
        render(&out).ends_with("</table>\n<p>para</p>\n"),
        "para must render after the table, not as a row: {:?}",
        render(&out)
    );
}

#[test]
fn should_render_inline_content_after_every_block_as_its_own_paragraph() {
    let mut failures = Vec::new();
    for (html, paragraph_end) in cases() {
        let out = tier2(&html);
        let rendered = render(&out);
        if !rendered.ends_with(&paragraph_end) {
            failures.push(format!("{html:?}: {out:?} renders {rendered:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "the content after the block is not its own paragraph:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_render_inline_content_after_every_block_as_its_own_paragraph_in_strict_whitespace_mode() {
    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..tier2_options()
    };
    let mut failures = Vec::new();
    for (html, paragraph_end) in cases() {
        let out = convert(&html, Some(options.clone()))
            .expect("strict conversion must succeed")
            .content
            .unwrap_or_default();
        let rendered = render(&out);
        if !rendered.ends_with(&paragraph_end) {
            failures.push(format!("{html:?}: {out:?} renders {rendered:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "the content after the block is not its own paragraph:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_agree_across_tiers_on_inline_content_after_a_block() {
    // ~keep Tier 1 bails on some of these (tables, for one); the count assertion keeps this
    // ~keep from passing when every case bails.
    let report = PrescanReport::default();
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    let mut compared = 0;
    let mut failures = Vec::new();
    for (html, _) in cases() {
        if let Ok(tier1_out) = tier1::run(&html, &report, &options) {
            let tier2_out = tier2(&html);
            if tier1_out != tier2_out {
                failures.push(format!("{html:?}: tier1 {tier1_out:?} vs tier2 {tier2_out:?}"));
            }
            compared += 1;
        }
    }
    assert!(
        failures.is_empty(),
        "Tier 1 and Tier 2 must agree:\n{}",
        failures.join("\n")
    );
    assert!(compared >= 100, "Tier 1 converted only {compared} cases");
}

#[test]
fn should_leave_output_without_inline_content_after_a_block_unchanged() {
    assert_eq!(tier2("<ul><li>A</li><li>B</li></ul>"), "- A\n- B\n");
    assert_eq!(tier2("<div><ul><li>A</li></ul><br>para</div>"), "- A\n\npara\n");
    assert_eq!(tier2("<div><ul><li>A</li></ul></div>"), "- A\n");
    assert_eq!(tier2("<div>A<br>B</div>"), "A  \nB\n");
}

#[test]
fn should_not_add_a_blank_line_after_a_block_that_wrote_nothing() {
    let report = PrescanReport::default();
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    let tier1_out = tier1::run("<div><ul></ul>para</div>", &report, &options).expect("tier1 converts an empty list");
    assert!(
        !tier1_out.starts_with("\n\n"),
        "an empty list must not open a blank line: {tier1_out:?}"
    );
    let strict = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..tier2_options()
    };
    let strict_out = convert("<div><div></div>\n<span> para</span></div>", Some(strict))
        .expect("strict conversion must succeed")
        .content
        .unwrap_or_default();
    assert!(
        !strict_out.starts_with("\n\n"),
        "an empty block must not open a blank line: {strict_out:?}"
    );
}

#[test]
fn should_keep_text_after_a_nested_list_inside_the_outer_list_item() {
    // ~keep Inside a list item the text after a nested list starts a paragraph at the item's
    // ~keep content column (issue #583), so it stays in the outer item and leaves the inner one.
    let html = "<ul><li>A<ul><li>B</li></ul>tail</li></ul>";
    let out = tier2(html);
    let rendered = render(&out);
    assert!(
        rendered.ends_with("</ul>\n<p>tail</p>\n</li>\n</ul>\n") && rendered.contains("<li>B</li>"),
        "tail must be a paragraph of the outer list item: {out:?} renders {rendered:?}"
    );
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    if let Ok(tier1_out) = tier1::run(html, &PrescanReport::default(), &options) {
        assert_eq!(tier1_out, out, "Tier 1 and Tier 2 must agree on {html:?}");
    }
}

/// Tier 2 output, and Tier 1 output wherever Tier 1 converts the input itself.
fn both_tiers(html: &str) -> (String, Option<String>) {
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    (tier2(html), tier1::run(html, &PrescanReport::default(), &options).ok())
}

#[test]
fn should_start_a_paragraph_after_a_list_item_that_ends_in_a_line_break() {
    let html = "<ul><li>A<br></li></ul>ZZ";
    let (tier2_out, tier1_out) = both_tiers(html);
    assert_eq!(tier2_out, "- A  \n\nZZ\n", "ZZ must start a paragraph after the item");
    assert_eq!(
        tier1_out.as_deref(),
        Some("- A  \n\nZZ\n"),
        "Tier 1 must match on {html:?}"
    );
    let backslash = ConversionOptions {
        newline_style: NewlineStyle::Backslash,
        ..tier2_options()
    };
    let out = convert(html, Some(backslash))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default();
    assert!(
        render(&out).ends_with("</ul>\n<p>ZZ</p>\n"),
        "ZZ must render after the list with backslash breaks: {out:?} renders {:?}",
        render(&out)
    );
}

#[test]
fn should_leave_a_line_break_between_inline_siblings_alone() {
    // ~keep The break before ZZ ends a line of the same paragraph, not a block, so no blank line.
    for html in [
        "<div>A<br>ZZ</div>",
        "<blockquote>A<br>ZZ</blockquote>",
        "<div>A<br> <i>ZZ</i></div>",
    ] {
        let (tier2_out, tier1_out) = both_tiers(html);
        assert!(
            !tier2_out.contains("\n\n"),
            "Tier 2 split a paragraph: {html:?} {tier2_out:?}"
        );
        if let Some(tier1_out) = tier1_out {
            assert_eq!(tier1_out, tier2_out, "Tier 1 and Tier 2 must agree on {html:?}");
        }
    }
}

#[test]
fn should_write_the_cell_break_in_a_cell_and_no_line_in_code_or_converted_inline_output() {
    for (html, expected) in [
        (
            "<table><tr><th>h</th></tr><tr><td><ul><li>A</li></ul>ZZ</td></tr></table>",
            "| h    |\n| ---- |\n| A ZZ |\n",
        ),
        (
            "<table><tr><th>h</th></tr><tr><td><hr>ZZ</td></tr></table>",
            "| h      |\n| ------ |\n| --- ZZ |\n",
        ),
        ("<pre><ul><li>A</li></ul>ZZ</pre>", "```\n- A\nZZ\n```\n"),
        ("<p><code><ul><li>A</li></ul>ZZ</code></p>", "`- A`  \n`ZZ`\n"),
    ] {
        let (tier2_out, tier1_out) = both_tiers(html);
        assert_eq!(tier2_out, expected, "Tier 2 changed an excluded context: {html:?}");
        if let Some(tier1_out) = tier1_out {
            assert_eq!(tier1_out, expected, "Tier 1 changed an excluded context: {html:?}");
        }
    }
    // ~keep A list item outside any list sets the list-item context but not the list context;
    // ~keep its text after a nested list still starts a paragraph inside it (issue #583).
    // ~keep Tier 1 renders a stray item differently, so only Tier 2 is pinned here.
    assert_eq!(
        tier2("<div><li>X<ul><li>A</li></ul>ZZ</li></div>"),
        "- X\n\n  - A\n\n  ZZ\n",
        "Tier 2 changed text inside a stray list item"
    );
    let inline = ConversionOptions {
        convert_as_inline: true,
        ..tier2_options()
    };
    let out = convert("<div><ul><li>A</li></ul>ZZ</div>", Some(inline))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default();
    assert!(
        !out.contains("\n\n"),
        "convert_as_inline must stay on one block: {out:?}"
    );
}

#[test]
fn should_agree_across_tiers_on_every_inline_element_after_a_list() {
    let mut compared = 0;
    for follower in [
        r#"<input type="checkbox">ZZ"#,
        "<wbr>ZZ",
        "<button>ZZ</button>",
        "<br>ZZ",
        "<label>ZZ</label>",
        "\n  <b>ZZ</b>",
        "<!-- c --> <i>ZZ</i>",
        "<x-foo>ZZ</x-foo>",
    ] {
        let html = format!("<div><ul><li>A</li></ul>{follower}</div>");
        let (tier2_out, tier1_out) = both_tiers(&html);
        if let Some(tier1_out) = tier1_out {
            assert_eq!(tier1_out, tier2_out, "Tier 1 and Tier 2 must agree on {html:?}");
            compared += 1;
        }
    }
    assert!(compared >= 5, "Tier 1 converted only {compared} cases");
}

#[test]
fn should_not_separate_on_whitespace_between_blocks() {
    // ~keep Whitespace between two blocks is not content: a definition term and its description,
    // ~keep or a list and a quote, keep the spacing their own handlers give them.
    for (html, expected) in [
        ("<dl><dt>T</dt>\n<dd>D</dd></dl>", "T\n\nD\n"),
        (
            "<div><ul><li>A</li></ul>\n<blockquote>Q</blockquote></div>",
            "- A\n\n> Q\n",
        ),
    ] {
        let (tier2_out, tier1_out) = both_tiers(html);
        assert_eq!(
            tier2_out, expected,
            "Tier 2 changed the spacing between blocks: {html:?}"
        );
        if let Some(tier1_out) = tier1_out {
            assert_eq!(
                tier1_out, expected,
                "Tier 1 changed the spacing between blocks: {html:?}"
            );
        }
    }
}

#[test]
fn should_start_a_paragraph_after_a_list_for_a_br_in_backslash_mode() {
    let options = ConversionOptions {
        newline_style: NewlineStyle::Backslash,
        ..tier2_options()
    };
    let out = convert("<div><ul><li>A</li></ul><br>ZZ</div>", Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default();
    assert!(
        !render(&out).contains("<li>A\n<br />"),
        "the br must not become a hard break inside the item: {out:?} renders {:?}",
        render(&out)
    );
}

#[test]
fn should_keep_text_between_list_items_in_a_tight_list() {
    let html = "<ul><li>A</li>ZZ<li>B</li></ul>";
    let (tier2_out, tier1_out) = both_tiers(html);
    assert!(!tier2_out.contains("\n\n"), "the list must stay tight: {tier2_out:?}");
    if let Some(tier1_out) = tier1_out {
        assert_eq!(tier1_out, tier2_out, "Tier 1 and Tier 2 must agree on {html:?}");
    }
}

#[test]
fn should_agree_across_tiers_on_the_block_before_the_text() {
    // ~keep Tier 1 files `<canvas>` as inline; both tiers must still use Tier 2's block test for the
    // ~keep element that closed before the text.
    let html = "<div><canvas><ul><li>A</li></ul></canvas>ZZ</div>";
    let (tier2_out, tier1_out) = both_tiers(html);
    let tier1_out = tier1_out.expect("tier1 converts a canvas");
    assert_eq!(tier1_out, tier2_out, "Tier 1 and Tier 2 must agree on {html:?}");
}
