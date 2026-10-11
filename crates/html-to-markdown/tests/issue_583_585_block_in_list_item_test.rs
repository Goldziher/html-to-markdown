// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issues #583 and #585.
//!
//! #583: a block inside a list item, followed by text in the same item, left the item: a heading
//! joined the item's text (`- A### H`), a rule broke out of the list, and the text after a div,
//! table, blockquote, code block or nested list became a lazy continuation or left the list.
//! `CommonMark` keeps a block in a list item only when its lines start at the item's content column.
//!
//! #585: text after a list or table at the end of an inline wrapper (`<span>`) continued the
//! list's last item or the table's last row.

use html_to_markdown_rs::options::{ListIndentType, PreprocessingOptions, WhitespaceMode};
use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert, tier1};

const QUOTE_IN_ITEM: &str = "- A\n  > q\n  >\n  > ### H\n  >\n  > r\n  >\n  > ```\n  > c\n  > ```\n  >\n  > s\n";

const FIGURE_THEN: &str = r#"<ol start="10"><li>X<figure><figcaption>F</figcaption></figure>{}</li></ol>"#;

const GFM_TABLE: &str = "<table><tr><th>h</th></tr><tr><td>c</td></tr></table>";

const LAYOUT_TABLE: &str = r#"<table><tr><td><a href="1">a</a> <a href="2">b</a></td></tr></table>"#;

/// Blocks inside a list item, with a fragment the block renders to.
const BLOCKS: [(&str, &str); 11] = [
    ("<h3>H</h3>", "<h3>H</h3>"),
    ("<hr>", "<hr />"),
    ("<div>d</div>", "<p>d</p>"),
    ("<p>p</p>", "<p>p</p>"),
    ("<ul><li>B</li></ul>", "<ul>\n<li>B</li>\n</ul>"),
    ("<blockquote>q</blockquote>", "<blockquote>\n<p>q</p>\n</blockquote>"),
    (GFM_TABLE, "<td>c</td>"),
    (LAYOUT_TABLE, r#"<a href="2">b</a>"#),
    ("<pre>code</pre>", "<pre><code>code\n</code></pre>"),
    ("<dl><dt>T</dt><dd>D</dd></dl>", "<p>T\nD</p>"),
    ("<span><ul><li>B</li></ul></span>", "<ul>\n<li>B</li>\n</ul>"),
];

/// List item shapes: the HTML around the block, and the rendered end of the item that holds it.
const ITEMS: [(&str, &str); 4] = [
    ("<ul><li>A{}tail</li></ul>", "<p>tail</p>\n</li>\n</ul>\n"),
    ("<ol><li>A{}tail</li></ol>", "<p>tail</p>\n</li>\n</ol>\n"),
    (
        "<ul><li>X<ul><li>A{}tail</li></ul></li></ul>",
        "<p>tail</p>\n</li>\n</ul>\n</li>\n</ul>\n",
    ),
    ("<ul><li><div>A{}tail</div></li></ul>", "<p>tail</p>\n</li>\n</ul>\n"),
];

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn convert_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn tier2(html: &str) -> String {
    convert_with(html, tier2_options())
}

/// Tier 1 output, or `None` when Tier 1 hands the input to Tier 2.
fn tier1(html: &str) -> Option<String> {
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    tier1::run(html, &PrescanReport::default(), &options).ok()
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    comrak::markdown_to_html(markdown, &options)
}

/// Whether the rendered item holds the block and the text after it, in that order. After a
/// heading the list stays tight, so the text is not wrapped in a paragraph.
fn keeps_block_and_text_in_item(rendered: &str, block: &str, item_end: &str) -> bool {
    let tight_end = item_end.replacen("<p>tail</p>\n", "tail", 1);
    let end = if rendered.ends_with(item_end) {
        item_end.len()
    } else if rendered.ends_with(&tight_end) {
        tight_end.len()
    } else {
        return false;
    };
    rendered.find(block).is_some_and(|at| at < rendered.len() - end)
}

#[test]
fn should_render_the_heading_on_its_own_line_inside_the_item() {
    let out = tier2("<ul><li>A<h3>H</h3>tail</li></ul>");
    assert_eq!(
        out, "- A\n  ### H\n  tail\n",
        "the heading must start at the content column"
    );
    assert_eq!(render(&out), "<ul>\n<li>A\n<h3>H</h3>\ntail</li>\n</ul>\n");
}

#[test]
fn should_keep_a_rule_and_the_text_after_it_inside_the_item() {
    let out = tier2("<ul><li>A<hr>tail</li></ul>");
    assert_eq!(
        out, "- A\n\n  ---\n\n  tail\n",
        "the rule must start at the content column"
    );
    assert_eq!(
        render(&out),
        "<ul>\n<li>\n<p>A</p>\n<hr />\n<p>tail</p>\n</li>\n</ul>\n"
    );
}

#[test]
fn should_keep_every_block_and_the_text_after_it_inside_the_list_item() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for (item, item_end) in ITEMS {
        for (block, fragment) in BLOCKS {
            let html = item.replace("{}", block);
            let out = tier2(&html);
            let rendered = render(&out);
            if !keeps_block_and_text_in_item(&rendered, fragment, item_end) {
                failures.push(format!("{html:?}: {out:?} renders {rendered:?}"));
            }
            checked += 1;
        }
    }
    assert!(
        failures.is_empty(),
        "the block or the text after it left the list item:\n{}",
        failures.join("\n")
    );
    assert_eq!(checked, ITEMS.len() * BLOCKS.len());
}

#[test]
fn should_keep_the_block_inside_the_item_with_tab_indentation() {
    let options = ConversionOptions {
        list_indent_type: ListIndentType::Tabs,
        ..tier2_options()
    };
    let mut failures = Vec::new();
    // ~keep A fenced code block indented by a tab keeps the tab's extra columns in its content,
    // ~keep with or without text after it, so the code block is left out here.
    for (block, fragment) in BLOCKS.iter().filter(|(block, _)| !block.starts_with("<pre>")) {
        let html = format!("<ul><li>A{block}tail</li></ul>");
        let out = convert_with(&html, options.clone());
        let rendered = render(&out);
        if !keeps_block_and_text_in_item(&rendered, fragment, "<p>tail</p>\n</li>\n</ul>\n") {
            failures.push(format!("{html:?}: {out:?} renders {rendered:?}"));
        }
    }
    assert!(failures.is_empty(), "with tabs:\n{}", failures.join("\n"));
}

#[test]
fn should_agree_across_tiers_or_leave_the_item_to_tier2() {
    let mut failures = Vec::new();
    for (item, _) in ITEMS {
        for (block, _) in BLOCKS {
            let html = item.replace("{}", block);
            if let Some(tier1_out) = tier1(&html) {
                let tier2_out = tier2(&html);
                if tier1_out != tier2_out {
                    failures.push(format!("{html:?}: tier1 {tier1_out:?} vs tier2 {tier2_out:?}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "Tier 1 must match Tier 2:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_leave_blocks_that_start_the_item_or_need_no_separation_unchanged() {
    for (html, expected) in [
        ("<ul><li><h3>H</h3></li></ul>", "- ### H\n"),
        ("<ul><li><p>A</p><p>B</p></li></ul>", "- A\n\n  B\n"),
        ("<ul><li>A<ul><li>B</li></ul></li><li>C</li></ul>", "- A\n  * B\n- C\n"),
        ("<ul><li>A<blockquote>q</blockquote></li></ul>", "- A\n  > q\n"),
        ("<ul><li>A<pre>code</pre></li></ul>", "- A\n\n  ```\n  code\n  ```\n"),
        ("<ul><li>A<br>B</li></ul>", "- A  \n  B\n"),
    ] {
        assert_eq!(tier2(html), expected, "{html:?} changed");
        if let Some(tier1_out) = tier1(html) {
            assert_eq!(tier1_out, expected, "Tier 1 changed {html:?}");
        }
    }
}

#[test]
fn should_leave_text_between_the_items_of_a_nested_list_alone() {
    // ~keep Text that is a child of the list, not of an item, is not item content.
    let html = "<ul><li>A<ul><li>B</li>x<li>C</li></ul></li></ul>";
    let out = tier2(html);
    assert!(!out.contains("\n\n"), "the nested list must stay tight: {out:?}");
}

/// Wrappers around a block, then the text after the wrapper, with the rendered block end.
const WRAPPED: [(&str, &str); 6] = [
    (
        "<div><span><ul><li>A</li></ul></span>para</div>",
        "</ul>\n<p>para</p>\n",
    ),
    (
        "<div><span><span><ul><li>A</li></ul></span></span>para</div>",
        "</ul>\n<p>para</p>\n",
    ),
    (
        "<div><span><ul><li>A</li></ul> </span><span>para</span></div>",
        "</ul>\n<p>para</p>\n",
    ),
    (
        "<span><ul><li>A</li></ul></span><b>para</b>",
        "</ul>\n<p><strong>para</strong></p>\n",
    ),
    (
        "<div><span><table><tr><th>h</th></tr><tr><td>c</td></tr></table></span>para</div>",
        "</table>\n<p>para</p>\n",
    ),
    ("<div><span><hr></span>para</div>", "<hr />\n<p>para</p>\n"),
];

#[test]
fn should_start_a_paragraph_after_a_block_at_the_end_of_an_inline_wrapper() {
    for (html, rendered_end) in WRAPPED {
        let out = tier2(html);
        let rendered = render(&out);
        assert!(
            rendered.ends_with(rendered_end),
            "{html:?}: the text must start a paragraph after the block: {out:?} renders {rendered:?}"
        );
        let tier1_out = tier1(html).expect("tier1 converts an inline wrapper");
        assert_eq!(tier1_out, out, "Tier 1 must match on {html:?}");
    }
}

#[test]
fn should_not_separate_text_that_continues_the_wrapper_after_its_block() {
    // ~keep The wrapper's own text after the block is the paragraph; the next text joins it.
    let html = "<div><span><ul><li>A</li></ul>x</span>para</div>";
    let out = tier2(html);
    assert_eq!(out, "- A\n\nxpara\n");
    assert_eq!(
        tier1(html).as_deref(),
        Some("- A\n\nxpara\n"),
        "Tier 1 must match on {html:?}"
    );
}

#[test]
fn should_agree_across_tiers_on_a_block_that_ends_the_item() {
    let mut failures = Vec::new();
    for (block, _) in BLOCKS {
        let html = format!("<ul><li>A{block}</li></ul>");
        if let Some(tier1_out) = tier1(&html) {
            let tier2_out = tier2(&html);
            if tier1_out != tier2_out {
                failures.push(format!("{html:?}: tier1 {tier1_out:?} vs tier2 {tier2_out:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "Tier 1 must match Tier 2:\n{}",
        failures.join("\n")
    );
}

#[test]
fn should_leave_blocks_inside_a_quote_in_a_list_item_to_the_quote() {
    // ~keep The quote renders its content on its own and prefixes each line afterwards, so the
    // ~keep list item's content column is written once, outside the quote marker.
    let html = "<ul><li>A<blockquote><p>q</p><h3>H</h3>r<pre>c</pre>s</blockquote></li></ul>";
    let out = tier2(html);
    assert_eq!(out, QUOTE_IN_ITEM, "the quote content changed");
    assert!(
        render(&out).ends_with("<p>r</p>\n<pre><code>c\n</code></pre>\n<p>s</p>\n</blockquote>\n</li>\n</ul>\n"),
        "the quote must keep its blocks inside the item: {:?}",
        render(&out)
    );
}

#[test]
fn should_agree_across_tiers_on_a_block_container_whose_content_is_dropped() {
    // ~keep `<nav>` content is dropped, but it is still a block between the two texts.
    for html in [
        "<ul><li>X<nav>A</nav>ZZ</li></ul>",
        "<ul><li>X<section>A</section></li></ul>",
    ] {
        if let Some(tier1_out) = tier1(html) {
            assert_eq!(tier1_out, tier2(html), "Tier 1 must match Tier 2 on {html:?}");
        }
    }
}

#[test]
fn should_agree_across_tiers_on_a_generic_block_container_in_an_item() {
    // ~keep Tier 2 starts an `<address>` in an item at the content column; Tier 1 does not.
    let html = "<ul><li>X<address>A</address>  </li></ul>";
    assert_eq!(tier2(html), "- X\n\n  A\n");
    if let Some(tier1_out) = tier1(html) {
        assert_eq!(tier1_out, tier2(html), "Tier 1 must match Tier 2 on {html:?}");
    }
}

#[test]
fn should_leave_no_blank_line_for_an_empty_div_in_a_list_item() {
    // ~keep A task item renders its text into a buffer of its own, where a one-letter text is
    // ~keep the whole buffer before the div.
    for (html, expected) in [
        ("<ul><li>X<div></div>ZZ</li></ul>", "- X\n  ZZ\n"),
        (
            r#"<ul><li><input type="checkbox">X<div></div>ZZ</li></ul>"#,
            "- [ ] X\n  ZZ\n",
        ),
    ] {
        assert_eq!(tier2(html), expected, "{html:?}");
    }
}

#[test]
fn should_agree_across_tiers_on_other_block_kinds_that_end_the_item() {
    // ~keep This kind reaches the fast path's catch-all block arm; text after it reaches the
    // ~keep inline-after-block hand-off instead, so it also ends the item here.
    for html in [
        "<ul><li>X<figcaption>A</figcaption></li></ul>",
        "<ul><li>X<figcaption>A</figcaption>ZZ</li></ul>",
    ] {
        if let Some(tier1_out) = tier1(html) {
            assert_eq!(tier1_out, tier2(html), "Tier 1 must match Tier 2 on {html:?}");
        }
    }
}

#[test]
fn should_keep_a_paragraph_in_its_item_in_strict_whitespace_mode() {
    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..tier2_options()
    };
    for html in [
        "<ul><li>\n  <p>A</p></li></ul>",
        "<ul><li> <p>A</p></li></ul>",
        "<ul><li>\n  <div>A</div></li></ul>",
    ] {
        let out = convert_with(html, options.clone());
        assert_eq!(
            render(&out),
            "<ul>\n<li>A</li>\n</ul>\n",
            "strict: the block left the item: {out:?}"
        );
    }
    let out = convert_with("<ul><li>\n<h3>H</h3>\n<hr>\n</li></ul>", options);
    assert!(
        render(&out).ends_with("<hr />\n</li>\n</ul>\n"),
        "strict: the rule left the item: {out:?} renders {:?}",
        render(&out)
    );
}

#[test]
fn should_keep_a_rule_a_rule_in_inline_mode_and_table_cells() {
    let inline = ConversionOptions {
        convert_as_inline: true,
        ..tier2_options()
    };
    let out = convert_with("<ul><li>A<blockquote>q</blockquote><hr></li></ul>", inline);
    assert!(
        !render(&out).contains("<h2>"),
        "inline: the item text became a heading: {out:?} renders {:?}",
        render(&out)
    );
    let out = tier2("<ul><li><table><tr><td>x<hr>y</td></tr></table></li></ul>");
    assert!(
        !out.contains("    ---"),
        "a rule in a table cell got the item's indent: {out:?}"
    );
    let out = tier2("<ul><li><table><tr><td>x<dl><dt>T</dt></dl></td></tr></table></li></ul>");
    assert!(
        !out.contains("x    T"),
        "a definition list in a table cell got the item's indent: {out:?}"
    );
}

#[test]
fn should_not_open_a_code_block_once_earlier_content_left_the_item() {
    let tabs = ConversionOptions {
        list_indent_type: ListIndentType::Tabs,
        ..tier2_options()
    };
    let mut cases = vec![
        (
            r#"<ol start="10"><li>X<section>A</section>ZZ</li></ol>"#.to_string(),
            tier2_options(),
        ),
        (
            r#"<ol start="10"><li><br><dl><dd><hr></dd></dl></li></ol>"#.to_string(),
            tier2_options(),
        ),
        ("<ul><li>X<section>A</section>ZZ</li></ul>".to_string(), tabs.clone()),
        ("<ul><li>X<article>A</article>ZZ</li></ul>".to_string(), tabs),
    ];
    // ~keep A figure still leaves the item, so nothing after it may get the content column.
    for after in [
        "ZZ",
        "<hr>",
        "<p>P</p>",
        "<div>D</div>",
        "<dl><dt>T</dt><dd>D</dd></dl>",
    ] {
        cases.push((FIGURE_THEN.replace("{}", after), tier2_options()));
    }
    for (html, options) in cases {
        let out = convert_with(&html, options);
        assert!(
            !render(&out).contains("<pre>"),
            "{html:?}: text became a code block: {out:?} renders {:?}",
            render(&out)
        );
    }
}

#[test]
fn should_keep_a_sectioning_element_and_the_text_after_it_in_the_item() {
    for (html, item_end) in [
        (
            r#"<ol start="10"><li>X<section>A</section>ZZ</li></ol>"#,
            "<p>ZZ</p>\n</li>\n</ol>\n",
        ),
        ("<ul><li>X<article>A</article>ZZ</li></ul>", "<p>ZZ</p>\n</li>\n</ul>\n"),
        ("<ul><li>X<header>A</header></li></ul>", "<p>A</p>\n</li>\n</ul>\n"),
    ] {
        let out = tier2(html);
        assert!(
            render(&out).ends_with(item_end),
            "{html:?}: {out:?} renders {:?}",
            render(&out)
        );
    }
}

#[test]
fn should_keep_a_paragraph_in_its_item_when_wrapping() {
    let wrap = ConversionOptions {
        wrap: true,
        ..tier2_options()
    };
    for html in [
        "<ul><li>A<p>B</p></li><li>C</li></ul>",
        "<ul><li>A<div>B</div></li><li>C</li></ul>",
        "<ul><li><p>A</p><p>B</p></li><li>C</li></ul>",
        "<ul><li>X<ul><li>A</li></ul>ZZ</li><li>C</li></ul>",
    ] {
        let out = convert_with(html, wrap.clone());
        let rendered = render(&out);
        assert!(
            rendered.matches("<ul>").count() == html.matches("<ul>").count() && rendered.ends_with("</li>\n</ul>\n"),
            "wrap: text left the list: {html:?} gives {out:?} renders {rendered:?}"
        );
    }
}

#[test]
fn should_keep_a_block_that_starts_the_item_on_the_fast_path() {
    // ~keep The full converter writes nothing new for a block at the item's marker line.
    let html = "<ul><li><dt>t</dt></li></ul>";
    assert!(tier1(html).is_some(), "Tier 1 handed {html:?} to Tier 2");
}

#[test]
fn should_keep_a_term_at_the_marker_and_the_rule_after_it_on_the_fast_path() {
    // ~keep The full converter writes a stray `<dt>` at the marker below an empty item, so the
    // ~keep default mode keeps the fast path's `- t` for it.
    let auto = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    for html in [
        "<ul><li><dt>t</dt><dd><hr></dd></li></ul>",
        "<ol><li><dt>t</dt><dd><hr></dd></li></ol>",
        "<ul><li><br><dt>t</dt><dd><hr></dd></li></ul>",
        "<ul><li><dd>xt<hr>B</dd></li></ul>",
    ] {
        let out = tier1(html).unwrap_or_else(|| panic!("Tier 1 handed {html:?} to Tier 2"));
        assert_eq!(out, convert_with(html, auto.clone()), "auto mode differs on {html:?}");
        assert!(
            render(&out).starts_with("<ul>\n<li>t")
                || render(&out).starts_with("<ol>\n<li>t")
                || render(&out).starts_with("<ul>\n<li>xt"),
            "{html:?}: {out:?}"
        );
    }
    // ~keep A rule on the item's text line, and a rule after a hard break, still hand off.
    for html in ["<ul><li>t<hr></li></ul>", "<ul><li>t<br><hr></li></ul>"] {
        assert!(tier1(html).is_none(), "Tier 1 converted {html:?}");
    }
}

#[test]
fn should_keep_rules_out_of_code_blocks_inside_containers() {
    let tabs = ConversionOptions {
        list_indent_type: ListIndentType::Tabs,
        ..tier2_options()
    };
    let wrap = ConversionOptions {
        wrap: true,
        ..tier2_options()
    };
    for (html, options) in [
        (
            r#"<ol start="10"><li>X<figure><p>a</p></figure><dl><dt>t</dt><dd><p>x</p><hr></dd></dl></li></ol>"#,
            tier2_options(),
        ),
        (
            r#"<ol start="10"><li>X<figure><p>a</p></figure><dl><dt>t</dt><dd><p>x</p><hr></dd></dl></li></ol>"#,
            wrap,
        ),
        (
            "<ul><li>X<figure><p>a</p></figure><dl><dt>t</dt><dd><p>x</p><hr></dd></dl></li></ul>",
            tabs.clone(),
        ),
        (
            r#"<ol start="10"><li>X<figure><p>a</p><hr></figure></li></ol>"#,
            tier2_options(),
        ),
        (
            r#"<ol start="10"><li>X<form><p>a</p><hr></form></li></ol>"#,
            ConversionOptions {
                preprocessing: PreprocessingOptions {
                    remove_forms: false,
                    ..PreprocessingOptions::default()
                },
                ..tier2_options()
            },
        ),
        (
            r#"<ol start="10"><li>X<fieldset><p>a</p><hr></fieldset></li></ol>"#,
            tier2_options(),
        ),
        // ~keep A section after the item has ended.
        (
            r#"<ol start="10"><li>X<figure><p>a</p></figure><section><p>a</p><hr></section></li></ol>"#,
            tier2_options(),
        ),
    ] {
        let out = convert_with(html, options);
        let rendered = render(&out);
        assert!(
            !rendered.contains("<pre>") && rendered.contains("<hr />"),
            "{html:?}: the rule was lost inside the container: {out:?} renders {rendered:?}"
        );
    }
    // ~keep A list inside an inline wrapper, a summary or a caption is not a list once the
    // ~keep markers are added, so nothing in it gets a column that a tab would turn into code.
    for (html, expected) in [
        ("<b><ul><li>x<dl><dd><hr></dd></dl></li></ul></b>", "**- x ---**\n"),
        ("<b><ul><li>x<p>p</p>t</li></ul></b>", "**- x\n\np\n\nt**\n"),
        (
            "<details><summary><ul><li>x<dl><dd>d</dd></dl>t</li></ul></summary></details>",
            "**- x\n\nd\n\nt**\n",
        ),
        (
            "<details><summary><ul><li>x<dl><dd><hr></dd></dl></li></ul></summary></details>",
            "**- x ---**\n",
        ),
        (
            "<figure><figcaption><ul><li>x<dl><dd><hr></dd></dl></li></ul></figcaption></figure>",
            "*- x ---*\n",
        ),
        ("<q><ul><li>x<dl><dd><hr></dd></dl></li></ul></q>", "\"- x ---\"\n"),
        (
            "<table><caption><ul><li>x<dl><dd><hr></dd></dl></li></ul></caption></table>",
            "*\\- x \\-\\-\\-*\n",
        ),
        // ~keep A figure preserves the item, so following text and rules keep the content column.
        (
            "<ul><li>X<figure><p>a</p></figure>t<br><hr></li></ul>",
            "- X\n\n\ta\n\n\tt  \n\n\t---\n",
        ),
        // ~keep A rule at the marker stays in the item, so the text and the rule after it do.
        ("<ul><li><hr>t<br><hr></li></ul>", "- ___\n\n\tt  \n\n\t---\n"),
        // ~keep The column written for a definition stays with a rule that starts it.
        (
            "<ul><li><dl><dt>t</dt><dd><hr><hr></dd></dl></li></ul>",
            "- t\n\n\t---\n\n\t---\n",
        ),
    ] {
        assert_eq!(
            convert_with(html, tabs.clone()),
            expected,
            "{html:?}: a wrapped list got the column"
        );
    }
    // ~keep While the item is open, a container's blocks stay at the column.
    let out = tier2(r#"<ol start="10"><li>A<dl><dt>T</dt><dd><p>x</p><hr></dd></dl></li></ol>"#);
    assert!(
        render(&out).ends_with("<hr />\n</li>\n</ol>\n"),
        "the definition's rule left the open item: {out:?}"
    );
}

#[test]
fn should_keep_a_rule_after_a_line_break_at_the_marker_a_rule() {
    for (html, options) in [
        (r#"<ol start="10"><li><br><hr>ZZ</li></ol>"#, tier2_options()),
        (
            "<ul><li><br><hr>ZZ</li></ul>",
            ConversionOptions {
                list_indent_type: ListIndentType::Tabs,
                ..tier2_options()
            },
        ),
    ] {
        let out = convert_with(html, options);
        let rendered = render(&out);
        assert!(
            rendered.contains("<hr />") && !rendered.contains("<pre><code>---"),
            "{html:?}: the rule became a code block: {out:?} renders {rendered:?}"
        );
    }
}

#[test]
fn should_leave_strict_text_that_starts_with_a_line_break_as_a_continuation_line() {
    let strict = ConversionOptions {
        whitespace_mode: WhitespaceMode::Strict,
        ..tier2_options()
    };
    let out = convert_with("<ul><li>X<nav>A</nav>\nZZ</li></ul>", strict);
    assert_eq!(
        render(&out),
        "<ul>\n<li>X\nZZ</li>\n</ul>\n",
        "strict: the text left the item: {out:?}"
    );
}

#[test]
fn should_keep_a_paragraph_in_its_item_when_wrapping_inside_a_quote() {
    let wrap = ConversionOptions {
        wrap: true,
        ..tier2_options()
    };
    let out = convert_with("<blockquote><ul><li>x<p>t<hr>B</p></li></ul></blockquote>", wrap);
    let rendered = render(&out);
    assert!(
        rendered.contains("<p>t</p>\n<hr />\n<p>B</p>\n</li>"),
        "wrap: the paragraph left the item inside the quote: {out:?} renders {rendered:?}"
    );
}

#[test]
fn should_reach_the_content_column_with_the_fewest_tabs() {
    // ~keep A tab is four columns wide, and four or more columns past the item's content column
    // ~keep is a code block; a nested item's column needs one tab, not one per level.
    let tabs = ConversionOptions {
        list_indent_type: ListIndentType::Tabs,
        ..tier2_options()
    };
    let out = convert_with(
        "<ul><li><ul><li>x<dl><dt>t</dt><dd>d</dd></dl></li></ul></li></ul>",
        tabs,
    );
    assert_eq!(
        out, "- * x\n\n\tt\n\td\n",
        "the nested item's definition list got the wrong tab column"
    );
    assert!(!render(&out).contains("<pre>"), "{out:?} renders {:?}", render(&out));
}

#[test]
fn should_keep_the_marker_line_before_a_container_that_starts_with_a_rule() {
    // ~keep `- ---` is a thematic break, which loses the item; the rule goes below the marker.
    for html in [
        "<ul><li><dl><dd><hr></dd></dl></li></ul>",
        "<ul><li><section><hr></section></li></ul>",
        "<ul><li><br><dl><dd><hr></dd></dl></li></ul>",
    ] {
        let out = tier2(html);
        assert!(
            render(&out).starts_with("<ul>\n<li></li>\n</ul>\n<hr />"),
            "{html:?}: the rule swallowed the item: {out:?} renders {:?}",
            render(&out)
        );
    }
}

#[test]
fn should_hand_text_after_a_nested_task_list_to_the_full_converter() {
    // ~keep The checkbox belongs to the nested item, so the outer item is a plain item whose text
    // ~keep after the nested list gets the content column, which only the full converter writes.
    let auto = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options()
    };
    for (html, expected) in [
        (
            r#"<ul><li>X<ul><li><input type="checkbox" checked> A</li></ul>ZZ</li></ul>"#,
            "- X\n  - [x] A\n\n  ZZ\n",
        ),
        (
            r#"<ul><li>X<ul><li><input type="checkbox"> A</li></ul>ZZ</li></ul>"#,
            "- X\n  - [ ] A\n\n  ZZ\n",
        ),
        ("<ul><li>X<ul><li>A</li></ul>ZZ</li></ul>", "- X\n  * A\n\n  ZZ\n"),
    ] {
        assert!(tier1(html).is_none(), "Tier 1 converted {html:?}");
        assert_eq!(convert_with(html, auto.clone()), expected, "auto mode on {html:?}");
    }
}

/// A details converts like a div, so its content and rule stay in the item.
#[test]
fn should_keep_the_content_of_a_details_in_its_list_item_like_a_div() {
    let details = convert_with(
        r#"<ol start="10"><li>X<details><p>a</p><hr></details></li></ol>"#,
        tier2_options(),
    );
    let div = convert_with(
        r#"<ol start="10"><li>X<div><p>a</p><hr></div></li></ol>"#,
        tier2_options(),
    );
    assert_eq!(details, div);
    let rendered = render(&details);
    assert!(
        !rendered.contains("<pre>") && rendered.contains("<hr />\n</li>"),
        "{details:?} renders {rendered:?}"
    );
}
