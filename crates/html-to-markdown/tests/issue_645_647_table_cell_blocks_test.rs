// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issues #645 and #647: a block in a table cell is separated from the cell
//! content before and after it, and both converters write the same cell.

use html_to_markdown_rs::prescan::PrescanReport;
use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert, tier1};

fn tier2_options(br_in_tables: bool) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        br_in_tables,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn tier2(html: &str, br_in_tables: bool) -> String {
    convert(html, Some(tier2_options(br_in_tables)))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn tier1_run(html: &str, br_in_tables: bool) -> Result<String, tier1::BailReason> {
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..tier2_options(br_in_tables)
    };
    tier1::run(html, &PrescanReport::default(), &options)
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    comrak::markdown_to_html(markdown, &options)
}

fn table(cell: &str) -> String {
    format!("<table><tr><td>{cell}</td></tr></table>")
}

/// The cell each tier writes, as `(cell html, br_in_tables off, br_in_tables on)`.
fn check(cases: &[(&str, &str, &str)]) {
    let mut failures = Vec::new();
    for (cell, off, on) in cases {
        let html = table(cell);
        for (br_in_tables, expected) in [(false, off), (true, on)] {
            let tier2_out = tier2(&html, br_in_tables);
            let tier1_out = tier1_run(&html, br_in_tables);
            let cell_line = tier2_out.lines().next().unwrap_or_default().to_string();
            if cell_line != format!("| {expected} |") || tier1_out.as_deref().ok() != Some(tier2_out.as_str()) {
                failures.push(format!(
                    "{cell:?} br_in_tables={br_in_tables}: tier2 {tier2_out:?} tier1 {tier1_out:?}, want | {expected} |"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "cell differs:\n{}", failures.join("\n"));
}

/// The cell Tier-2 writes, as `(cell html, br_in_tables off, br_in_tables on)`, for a cell the
/// fast converter leaves to Tier-2.
fn check_tier2(cases: &[(&str, &str, &str)]) {
    let mut failures = Vec::new();
    for (cell, off, on) in cases {
        let html = table(cell);
        for (br_in_tables, expected) in [(false, off), (true, on)] {
            let tier2_out = tier2(&html, br_in_tables);
            let tier1_out = tier1_run(&html, br_in_tables);
            if tier2_out.lines().next() != Some(format!("| {expected} |").as_str())
                || !matches!(tier1_out, Err(tier1::BailReason::TableBlockChildInCell))
            {
                failures.push(format!(
                    "{cell:?} br_in_tables={br_in_tables}: tier2 {tier2_out:?} tier1 {tier1_out:?}, want | {expected} |"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "cell differs:\n{}", failures.join("\n"));
}

#[test]
fn should_separate_text_after_a_paragraph_or_heading_in_a_cell() {
    check(&[
        ("<p>a</p>b", "a b", "a<br>b"),
        ("<h2>a</h2>b", "a b", "a<br>b"),
        ("<h2>a</h2><h2>b</h2>", "a b", "a<br>b"),
        ("<div>a</div>b", "a b", "a<br>b"),
        ("<ul><li>a</li></ul>b", "a b", "a<br>b"),
        ("<pre>a</pre>b", "`a` b", "`a`<br>b"),
    ]);
}

#[test]
fn should_render_the_words_after_a_paragraph_in_a_cell_apart() {
    let markdown = tier2(&table("<p>a</p>b"), false);
    assert!(render(&markdown).contains("<th>a b</th>"), "{markdown:?}");
    let markdown = tier2(&table("<h2>a</h2>b"), false);
    assert!(render(&markdown).contains("<th>a b</th>"), "{markdown:?}");
}

#[test]
fn should_separate_text_before_a_heading_or_code_block_in_a_cell() {
    check(&[
        ("a<h2>b</h2>c", "a b c", "a<br>b<br>c"),
        ("a<pre>b</pre>c", "a `b` c", "a<br>`b`<br>c"),
        ("a<pre> b</pre>", "a `b`", "a<br>`b`"),
    ]);
}

#[test]
fn should_write_no_break_after_a_break_or_around_an_empty_block() {
    check(&[
        ("<p>a</p> b", "a b", "a<br> b"),
        ("<p>a</p><br>b", "a b", "a<br>b"),
        ("<p>a</p>", "a", "a"),
        ("<h2></h2>b", "b", "b"),
        ("a<h2> </h2>b", "a b", "a<br>b"),
        ("a<h2> </h2>", "a", "a"),
        ("a<blockquote> </blockquote>", "a", "a"),
        ("<p>a<br></p>b", "a b", "a<br>b"),
        ("<section>a</section>b", "a b", "a<br>b"),
    ]);
}

/// Both converters write a heading in a cell into a buffer of its own, so a block at its start
/// has no content to break from.
#[test]
fn should_write_no_break_before_a_block_at_the_start_of_a_heading() {
    check(&[("x<h2><div>b</div></h2>", "x b", "x<br>b")]);
}

/// A cell break never trims the space before a quote: the quote's content starts after it.
#[test]
fn should_keep_the_cell_break_whole_before_a_quote_in_a_cell() {
    check(&[
        ("a <blockquote><p>b</p></blockquote>", "a b", "a<br>b"),
        (
            "<section>- x <blockquote><div></div></blockquote></section>",
            "- x",
            "- x",
        ),
        ("<div><hr><blockquote><pre><li></pre></blockquote></div>", "---", "---"),
        ("a <blockquote><br>b</blockquote>", "a b", "a<br><br>b"),
        ("a <blockquote><hr></blockquote>", "a ---", "a<br>---"),
        ("x <pre><p>b</p></pre>", "x `b`", "x<br>`b`"),
        ("x<pre><li>b</li></pre>", "x `b`", "x<br>`b`"),
        (
            "a <blockquote><details><summary>s<p>x</p></summary></details></blockquote>",
            "a **s x**",
            "a<br>**s<br>x**",
        ),
    ]);
}

/// A quote or a table next to it outside the cell does not change where the cell's content starts.
#[test]
fn should_write_the_same_cell_in_a_table_inside_a_quote_in_both_tiers() {
    for (html, cell) in [
        (
            "abc<blockquote><table><tr><td>a<p>b</p></td></tr></table></blockquote>",
            "| a b |",
        ),
        (
            "<table><tr><td>a <blockquote><table><tr><td>x</td></tr></table></blockquote></td><td>z</td></tr></table>",
            r"| a \| x \| \| --- \| | z |",
        ),
    ] {
        for br_in_tables in [false, true] {
            let tier2_out = tier2(html, br_in_tables);
            let tier1_out = tier1_run(html, br_in_tables).expect("tier 1 must not bail");
            let row = |markdown: &str| markdown.lines().find(|line| line.contains("| ")).map(str::to_string);
            assert_eq!(row(&tier1_out), row(&tier2_out), "{html:?}");
            if !br_in_tables {
                assert!(tier2_out.contains(cell), "{html:?}: {tier2_out:?}");
            }
        }
    }
}

/// ~keep A real break in a preformatted cell stays outside the code spans that surround it.
#[test]
fn should_keep_cell_break_outside_preformatted_code_spans() {
    check(&[
        ("<pre>a<br><p>b</p></pre>", "`a b`", "`a`<br>`b`"),
        ("<pre>a<br><div>b</div></pre>", "`a b`", "`a`<br>`b`"),
    ]);
}

#[test]
fn should_write_no_break_for_a_block_inside_code_or_a_heading() {
    check(&[
        ("<h2>a<blockquote>b</blockquote></h2>", "ab", "ab"),
        ("<code>b c<code><br></code></code>", "`b c `", "`b c `"),
    ]);
}

#[test]
fn should_write_a_quote_in_a_cell_without_its_marker_in_both_tiers() {
    check(&[
        ("<blockquote>a</blockquote>b", "a b", "a<br>b"),
        ("<blockquote><hr></blockquote>", "---", "---"),
        ("a<blockquote>b</blockquote>c", "a b c", "a<br>b<br>c"),
        ("<blockquote><p>a</p><p>b</p></blockquote>", "a b", "a<br>b"),
        ("<blockquote><blockquote>a</blockquote>b</blockquote>", "a b", "a<br>b"),
    ]);
}

#[test]
fn should_keep_the_citation_of_a_quote_in_a_cell() {
    let html = table(r#"<blockquote cite="http://x/">a</blockquote>b"#);
    assert_eq!(tier2(&html, false).lines().next(), Some("| a — <http://x/> b |"));
    assert_eq!(tier2(&html, true).lines().next(), Some("| a<br>— <http://x/><br>b |"));
}

#[test]
fn should_write_the_same_paragraph_break_in_a_cell_in_both_tiers() {
    check(&[
        ("a<p>b</p>c", "a b c", "a<br>b<br>c"),
        ("a<p>b</p>", "a b", "a<br>b"),
        ("<p>a</p><p>b</p>", "a b", "a<br>b"),
        ("<h2>a</h2><p>b</p>", "a b", "a<br>b"),
    ]);
}

/// Inside a code block Tier-2 writes bold into the code, so a block in the bold breaks from the
/// code before it.
#[test]
fn should_break_before_a_block_in_bold_inside_a_code_block_in_a_cell() {
    check(&[("<pre>a<b><p>y</p></b>q</pre>", "`a yq`", "`a`<br>`yq`")]);
}

/// Tier-2 still sees a navigation block that preprocessing drops, so the text after it breaks.
#[test]
fn should_separate_text_after_a_dropped_navigation_block_in_a_cell() {
    check(&[("a<nav>x <p>b</p></nav>c", "a c", "a<br>c")]);
}

/// The text after a nested table folded into a cell joins the table's last row in both tiers.
#[test]
fn should_join_text_after_a_folded_nested_table_in_both_tiers() {
    let html = "<table><tr><td><table><tr><td>x</td></tr></table>b</td><td>z</td></tr></table>";
    for (br_in_tables, cell) in [
        (false, r"| \| x \| \| --- \|b | z |"),
        (true, r"| \| x \|<br>\| --- \|b | z |"),
    ] {
        let tier2_out = tier2(html, br_in_tables);
        assert_eq!(tier2_out.lines().next(), Some(cell), "br_in_tables={br_in_tables}");
        let tier1_out = tier1_run(html, br_in_tables).expect("tier 1 must not bail");
        assert_eq!(tier1_out.lines().next(), Some(cell), "br_in_tables={br_in_tables}");
    }
}

/// Tier-2 separates a section from the cell content before it with a blank line, which the cell
/// folds into two spaces, so the fast converter leaves that cell to Tier-2.
#[test]
fn should_leave_a_section_after_cell_content_to_tier2() {
    for cell in ["a<section>b</section>c", "<code>a<section></section></code>"] {
        for br_in_tables in [false, true] {
            assert!(
                matches!(
                    tier1_run(&table(cell), br_in_tables),
                    Err(tier1::BailReason::TableBlockChildInCell)
                ),
                "{cell:?} br_in_tables={br_in_tables}"
            );
        }
    }
    let auto = ConversionOptions {
        tier_strategy: TierStrategy::Auto,
        ..tier2_options(false)
    };
    let html = table("a<section>b</section>c");
    let auto_out = convert(&html, Some(auto)).expect("conversion must succeed").content;
    assert_eq!(auto_out, Some(tier2(&html, false)));
}

/// Tier-2 lays out a block inside bold, code, sub, sup or abbr, a block or line break at the start
/// of one of these or of a definition term or definition, an inline element after a block, and a
/// legend, figure caption or label, with whitespace the fast converter does not reproduce in a
/// cell, so the fast converter leaves the cell to Tier-2.
#[test]
fn should_leave_a_block_in_inline_markup_or_a_label_in_a_cell_to_tier2() {
    check_tier2(&[
        ("<b><h2>a</h2>b</b>", "**a b**", "**a<br>b**"),
        ("a<b><h2>b</h2></b>", "a**b**", "a**b**"),
        ("a<sub><pre>b</pre></sub>c", "a`b` c", "a`b`<br>c"),
        ("a<sup><p>b</p></sup>c", "ab c", "ab<br>c"),
        ("a<abbr><p>b</p></abbr>c", "ab c", "ab<br>c"),
        ("a<kbd><p>b</p></kbd>c", "a`b` c", "a`b`<br>c"),
        ("a<em><p>b</p></em>c", "a*b* c", "a*b*<br>c"),
        ("<em><p>a</p>b</em>", "*a b*", "*a<br>b*"),
        ("a<b><div>b</div></b>", "a**b**", "a**b**"),
        ("a<b><ul><li>x</li></ul></b>", "a**x**", "a**x**"),
        ("a<b><li>x</li></b>", "a**x**", "a**x**"),
        ("a <sub><hr></sub>", "a ---", "a ---"),
        ("<li>a</li><li><code><br></code></li>b", "a ` ` b", "a<br>` `<br>b"),
        ("<code>a<h2>b</h2></code>", "`ab`", "`ab`"),
        ("<code>a<pre>b</pre></code>", "`` a`b` ``", "`` a`b` ``"),
        ("<code>a<blockquote> </blockquote></code>", "`a`", "`a`"),
        ("<code><blockquote>a</blockquote>b</code>", "`a`   `b`", "`a`   `b`"),
        ("<code>a<b><p>y</p></b>q</code>", "`a yq`", "`a<br>yq`"),
        ("a<dt><label><hr></label></dt>c", "a  --- c", "a  ---<br>c"),
        ("a<dt><sub><hr></sub></dt>c", "a  --- c", "a  ---<br>c"),
        ("a <sub> <p>b</p></sub>c", "a b c", "a <br>b<br>c"),
        ("<p>a</p><sub> <ul><li>b</li></ul></sub>c", "a b c", "a<br> b<br>c"),
        ("a <abbr> <dt>b</dt></abbr>c", "a b c", "a b<br>c"),
        ("a <abbr> <dd>b</dd></abbr>c", "a b c", "a b<br>c"),
        ("a <sub> <label><p>b</p></label></sub>c", "a b c", "a b<br>c"),
        ("a <sub> <dl><dt>b</dt></dl></sub>c", "a b c", "a b<br>c"),
        ("a <sub><dt> <ul><li>b</li></ul></dt></sub>c", "a b c", "a b<br>c"),
        ("a <abbr><dd> <ul><li>b</li></ul></dd></abbr>c", "a b c", "a b<br>c"),
        ("a <sub><label> <ul><li>b</li></ul></label></sub>c", "a b c", "a b<br>c"),
        (
            "x<code>a<blockquote>q</blockquote>b</code>",
            "x`a`   `q`   `b`",
            "x`a`   `q`   `b`",
        ),
        ("<hr><code><h2></h2>b c</code>", "--- `b c`", "---<br>`b c`"),
        ("a <label><p>b</p></label>c", "a b c", "a b<br>c"),
        ("a <sub> <br>b</sub>c", "a bc", "a <br>bc"),
        ("<legend><hr></legend>c", "**---** c", "**---**<br>c"),
        ("a<figcaption><p>b</p></figcaption>c", "a  *b* c", "a  *b*<br>c"),
        ("a<label> b</label>c", "abc", "abc"),
        ("<p>a</p><b>b</b>", "a **b**", "a<br>**b**"),
        ("a <dt><p>b</p></dt>c", "a b c", "a b<br>c"),
        ("a <dd><p>b</p></dd>c", "a b c", "a b<br>c"),
        ("a<dt><hr></dt>c", "a  --- c", "a  ---<br>c"),
        ("a<dd> <hr></dd>c", "a  --- c", "a  ---<br>c"),
        ("a<dt><blockquote><hr></blockquote></dt>c", "a  --- c", "a  ---<br>c"),
        ("<p>a</p><span><dt><hr></dt></span>c", "a  --- c", "a<br>  ---<br>c"),
    ]);
    let html = "<table><tr><td>a<sub><table><tr><td>x</td></tr></table></sub></td><td>z</td></tr></table>";
    assert!(matches!(
        tier1_run(html, false),
        Err(tier1::BailReason::TableBlockChildInCell)
    ));
}

/// Tier-2 trims the start of a definition term or definition, so a space at its start after
/// other cell content is dropped in both tiers.
#[test]
fn should_trim_the_start_of_a_definition_after_cell_text_in_both_tiers() {
    check(&[("a<dt> b </dt>", "ab", "ab"), ("a<dd> b </dd>", "ab", "ab")]);
}

/// Tier-2 separates a definition from a definition before it in the same cell, and separates a
/// rule, but not `---` text, at the start of a definition. The fast converter leaves both cells
/// to Tier-2.
#[test]
fn should_leave_a_second_definition_or_a_dash_led_definition_in_a_cell_to_tier2() {
    check_tier2(&[
        ("<b>a</b><dt><!--x-->b</dt><dt> c</dt>", "**a**b c", "**a**b c"),
        ("<dt> - b</dt><dt> c</dt>", "- b c", "- b c"),
        ("<dd> b </dd><dd> c</dd>", "b  c", "b  c"),
        ("<b>a</b><dd> --- b</dd>", "**a**--- b", "**a**--- b"),
        ("a<dt> <!--x--> ---</dt>", "a  ---", "a  ---"),
        ("<p>a</p><dd> <!--x--> ---</dd>", "a  ---", "a  ---"),
    ]);
}

/// Whitespace between two inline elements inside a quote or heading is not at its start, so the
/// fast converter writes that cell itself.
#[test]
fn should_write_a_quote_or_heading_with_inner_whitespace_on_the_fast_path() {
    check(&[
        (
            "a<blockquote><b>x</b> <i>y</i></blockquote>",
            "a **x** *y*",
            "a<br>**x** *y*",
        ),
        ("a<h3><b>x</b> <i>y</i></h3>", "a **x** *y*", "a<br>**x** *y*"),
    ]);
}

/// Tier-2 keeps whitespace at the start of a quote or heading in a cell and breaks the cell twice
/// before its first block, so the fast converter leaves that cell to Tier-2.
#[test]
fn should_leave_a_quote_or_heading_that_starts_with_whitespace_in_a_cell_to_tier2() {
    check_tier2(&[
        ("a <blockquote> <p>b</p></blockquote>c", "a b c", "a<br><br>b<br>c"),
        ("a <h3> <p>b</p></h3>c", "a b c", "a<br><br>b<br>c"),
        ("a <span><h3> <p>b</p></h3></span>", "a b", "a<br><br>b"),
        ("a<blockquote> <hr></blockquote> c", "a --- c", "a<br>---<br> c"),
        (
            "a<blockquote><blockquote>\n<p>b</p></blockquote></blockquote>c",
            "a b c",
            "a<br>b<br>c",
        ),
    ]);
}
