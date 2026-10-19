//! Regression tests for issue #772: an element with no end tag ends where the HTML standard says.
//!
//! Each expected output matches the tree that Google Chrome 155 builds for the input.

#![cfg(feature = "testkit")]

use std::fmt::Write;

use html_to_markdown_rs::{ConversionOptions, ConversionResult, TierStrategy, WarningKind, convert};

fn convert_with(html: &str, tier_strategy: TierStrategy) -> ConversionResult {
    convert_with_depth(html, tier_strategy, None)
}

fn convert_with_depth(html: &str, tier_strategy: TierStrategy, max_depth: Option<usize>) -> ConversionResult {
    let options = ConversionOptions {
        tier_strategy,
        extract_metadata: false,
        max_depth,
        ..ConversionOptions::default()
    };
    convert(html, Some(options)).expect("conversion must succeed")
}

fn assert_all_routes(html: &str, expected: &str) {
    let default = convert(html, None).expect("conversion must succeed");
    assert_eq!(default.content.as_deref(), Some(expected), "default options: {html:?}");
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1] {
        let result = convert_with(html, tier_strategy);
        assert_eq!(result.content.as_deref(), Some(expected), "{tier_strategy:?}: {html:?}");
    }
}

fn issue_page(blocks: usize, paragraph_end_tag: &str) -> String {
    let mut notes = String::new();
    for number in 1..=blocks {
        write!(notes, "<div class='note'><p>Note {number}.{paragraph_end_tag}</div>").expect("write to a string");
    }
    format!("<!doctype html><html><body><main>{notes}</main><footer><p>Built with care.</p></footer></body></html>")
}

fn issue_markdown(blocks: usize) -> String {
    let mut expected = String::new();
    for number in 1..=blocks {
        write!(expected, "Note {number}.\n\n").expect("write to a string");
    }
    expected.push_str("Built with care.\n");
    expected
}

#[test]
fn eighty_paragraphs_with_no_end_tag_keep_every_note_and_the_footer() {
    let html = issue_page(80, "");
    let expected = issue_markdown(80);
    assert_all_routes(&html, &expected);
    let result = convert(&html, None).expect("conversion must succeed");
    assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
}

#[test]
fn a_paragraph_with_no_end_tag_gives_the_output_of_a_closed_paragraph() {
    for blocks in [1, 29, 30, 31] {
        let closed = convert(&issue_page(blocks, "</p>"), None).expect("conversion must succeed");
        let open = convert(&issue_page(blocks, ""), None).expect("conversion must succeed");
        assert_eq!(closed.content.as_deref(), Some(issue_markdown(blocks).as_str()));
        assert_eq!(open.content, closed.content, "{blocks} blocks");
    }
}

#[test]
fn the_end_tag_of_the_parent_ends_an_open_paragraph() {
    assert_all_routes("<!doctype html><div><p>one</div>tail", "one\n\ntail\n");
    assert_all_routes(
        "<!doctype html><section><p>one</section><span>tail</span>",
        "one\n\ntail\n",
    );
    assert_all_routes(
        "<!doctype html><div><p>one</div><h2>Title</h2><ul><li>item</li></ul>",
        "one\n\n## Title\n\n- item\n",
    );
    assert_all_routes(
        "<!doctype html><blockquote><p>quoted</blockquote><p>after</p>",
        "> quoted\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><table><tr><th>h</th></tr><tr><td><p>cell</td></tr></table><p>after</p>",
        "| h    |\n| ---- |\n| cell |\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><details><summary>S</summary><p>one</details><p>two</p>",
        "**S**\n\none\n\ntwo\n",
    );
    assert_all_routes(
        "<!doctype html><div><p>one<!-- </div><div> --></div>tail",
        "one\n\ntail\n",
    );
}

#[test]
fn a_block_start_tag_ends_an_open_paragraph() {
    assert_all_routes("<!doctype html><p>one<div>two</div>three", "one\n\ntwo\n\nthree\n");
    assert_all_routes(
        "<!doctype html><p>one<ul><li>two</li></ul>three",
        "one\n\n- two\n\nthree\n",
    );
    assert_all_routes(
        "<!doctype html><p>one<table><tr><th>h</th></tr><tr><td>two</td></tr></table>three",
        "one\n\n| h   |\n| --- |\n| two |\n\nthree\n",
    );
    assert_all_routes("<!doctype html><p>one<h2>two</h2>three", "one\n\n## two\n\nthree\n");
    assert_all_routes(
        "<!doctype html><p>one<pre>two</pre>three",
        "one\n\n```\ntwo\n```\n\nthree\n",
    );
    assert_all_routes("<!doctype html><p>one<p>two<p>three", "one\n\ntwo\n\nthree\n");
}

#[test]
fn list_items_and_definitions_with_no_end_tag_stay_siblings() {
    assert_all_routes("<!doctype html><ul><li>one<li>two</ul>after", "- one\n- two\n\nafter\n");
    assert_all_routes("<!doctype html><ol><li>one</ol><p>after</p>", "1. one\n\nafter\n");
    assert_all_routes(
        "<!doctype html><ul><li><p>one<li><p>two</ul><p>after</p>",
        "- one\n\n- two\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><dl><dt>term<dd>desc<dt>term2<dd>desc2</dl>after",
        "term\n\ndesc\n\nterm2\n\ndesc2\n\nafter\n",
    );
}

#[test]
fn table_parts_with_no_end_tag_keep_every_row() {
    let expected = "| a | b |\n| --- | --- |\n| c | d |\n\nafter\n";
    assert_all_routes(
        "<!doctype html><table><tr><th>a<th>b<tr><td>c<td>d</table>after",
        expected,
    );
    assert_all_routes(
        "<!doctype html><table><thead><tr><th>a<th>b<tbody><tr><td>c<td>d</table>after",
        expected,
    );
}

#[test]
fn a_document_with_no_end_tag_for_head_and_body_keeps_its_body() {
    let html = "<!doctype html><html><head><title>T</title><body><p>one<p>two";
    for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1] {
        let result = convert_with(html, tier_strategy);
        assert_eq!(result.content.as_deref(), Some("one\n\ntwo\n"), "{tier_strategy:?}");
    }
    let default = convert(html, None).expect("conversion must succeed");
    assert_eq!(default.content.as_deref(), Some("---\ntitle: T\n---\n\none\n\ntwo\n"));
}

#[test]
fn an_end_tag_with_no_start_tag_changes_nothing() {
    assert_all_routes("<!doctype html><p>one</div>two</p><p>three</p>", "onetwo\n\nthree\n");
    assert_all_routes("<!doctype html>one</div>two</span>three", "onetwothree\n");
}

/// For an input where the fast converter writes other Markdown for the closed form too.
fn assert_default_and_tier2(html: &str, expected: &str) {
    let default = convert(html, None).expect("conversion must succeed");
    assert_eq!(default.content.as_deref(), Some(expected), "default options: {html:?}");
    let tier2 = convert_with(html, TierStrategy::Tier2);
    assert_eq!(tier2.content.as_deref(), Some(expected), "Tier2: {html:?}");
}

#[test]
fn a_block_in_an_open_list_item_or_section_ends_with_its_parent() {
    assert_all_routes(
        "<!doctype html><ul><li><div>one</div><li><div>two</ul>after",
        "- one\n\n- two\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><ol><li>one<p>two<li>three</ol>after",
        "1. one\n\n   two\n\n2. three\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><article><header><p>h</header><p>one</article><footer><p>f</footer>tail",
        "h\n\none\n\nf\n\ntail\n",
    );
    // ~keep A definition term is a paragraph of its own (issue #779).
    assert_all_routes("<!doctype html><dl><dt>a<dt>b<dd>c</dl>after", "a\n\nb\n\nc\n\nafter\n");
    assert_default_and_tier2(
        "<!doctype html><dl><dt>t<dd><p>d1<p>d2</dl>after",
        "t\n\nd1\n\nd2\n\nafter\n",
    );
}

#[test]
fn options_with_no_end_tag_give_the_output_of_closed_options() {
    let pairs = [
        (
            "<!doctype html><select><option>one<option>two</select>after",
            "<!doctype html><select><option>one</option><option>two</option></select>after",
            "one two after\n",
        ),
        (
            "<!doctype html><p>before <select><option>one<option>two</select> after<p>next",
            "<!doctype html><p>before <select><option>one</option><option>two</option></select> after</p><p>next</p>",
            "before one two after\n\nnext\n",
        ),
        (
            "<!doctype html><select><optgroup label=a><option>x<optgroup label=b><option>y</select>after",
            "<!doctype html><select><optgroup label=a><option>x</option></optgroup>\
             <optgroup label=b><option>y</option></optgroup></select>after",
            "x y after\n",
        ),
    ];
    for (open, closed, expected) in pairs {
        assert_all_routes(open, expected);
        assert_all_routes(closed, expected);
    }
}

#[test]
fn ruby_text_with_no_end_tag_ends_at_the_next_ruby_part() {
    assert_all_routes(
        "<!doctype html><p><ruby>base<rt>top</ruby> after</p>",
        "base(top) after\n",
    );
    assert_all_routes("<!doctype html><p><ruby>a<rt>b<rt>c</ruby> after", "a(b)(c) after\n");
    assert_all_routes(
        "<!doctype html><p><ruby>kan<rp>(<rt>ji<rp>)</ruby> after<p>next",
        "kan(ji) after\n\nnext\n",
    );
}

#[test]
fn each_table_part_with_no_end_tag_keeps_every_row() {
    let two_columns = "| a | b |\n| --- | --- |\n| c | d |\n\nafter\n";
    assert_all_routes(
        "<!doctype html><table><tr><th>a</th><th>b</th><tr><td>c</td><td>d</td></table>after",
        two_columns,
    );
    assert_all_routes(
        "<!doctype html><table><tr><th>a<th>b</tr><tr><td>c</td><td>d</td></tr></table>after",
        two_columns,
    );
    assert_all_routes(
        "<!doctype html><table><tr><th>a</th><th>b</th></tr><tr><td>c<td>d</tr></table>after",
        two_columns,
    );
    assert_all_routes(
        "<!doctype html><table><thead><tr><th>a</th></tr><tbody><tr><td>b</td></tr></tbody></table>after",
        "| a |\n| --- |\n| b |\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><table><thead><tr><th>h</th></tr></thead><tbody><tr><td>a</td></tr>\
         <tbody><tr><td>b</td></tr><tfoot><tr><td>f</td></tr></table>after",
        "| h |\n| --- |\n| a |\n| b |\n| f |\n\nafter\n",
    );
}

#[test]
fn a_head_with_no_end_tag_ends_at_the_first_body_content() {
    for (html, body) in [
        (
            "<!doctype html><html><head><title>T</title><body><p>one</p></body></html>",
            "one\n",
        ),
        (
            "<!doctype html><html><head><title>T</title><meta charset=utf-8><p>one<p>two",
            "one\n\ntwo\n",
        ),
        (
            "<!doctype html><html><head><title>T</title><div><p>one</div>tail",
            "one\n\ntail\n",
        ),
    ] {
        for tier_strategy in [TierStrategy::Tier2, TierStrategy::Tier1] {
            let result = convert_with(html, tier_strategy);
            assert_eq!(result.content.as_deref(), Some(body), "{tier_strategy:?}: {html:?}");
        }
        let default = convert(html, None).expect("conversion must succeed");
        let expected = format!("---\ntitle: T\n---\n\n{body}");
        assert_eq!(default.content.as_deref(), Some(expected.as_str()), "{html:?}");
    }
    assert_all_routes("<!doctype html><body><p>one</body><p>two", "one\n\ntwo\n");
}

#[test]
fn an_end_tag_with_no_start_tag_ends_no_other_element() {
    assert_all_routes(
        "<!doctype html><ul><li>one</li></li><li>two</li></ul></ul>after",
        "- one\n- two\n\nafter\n",
    );
    assert_all_routes(
        "<!doctype html><table><tr><th>a</th></th><th>b</th></tr></tr><tr><td>c</td><td>d</td></tr></table></table>after",
        "| a | b |\n| --- | --- |\n| c | d |\n\nafter\n",
    );
    assert_all_routes("<!doctype html></div><p>one</p></section>", "one\n");
    assert_all_routes(
        "<!doctype html><p>one</b> two</em> three</p><p>four</p>",
        "one two three\n\nfour\n",
    );
    assert_all_routes("<!doctype html><p>one</p></body></html>", "one\n");
}

#[test]
fn an_end_tag_in_another_spelling_ends_its_element() {
    assert_all_routes("<!doctype html><DIV><p>one</p></div><p>two</p>", "one\n\ntwo\n");
    assert_all_routes("<!doctype html><div><p>one</p></div ><p>two</p>", "one\n\ntwo\n");
    assert_all_routes(
        "<!doctype html><DIV><P>one</DIV><DIV><P>two</DIV>tail",
        "one\n\ntwo\n\ntail\n",
    );
    assert_all_routes("<HTML><BODY><DIV><P>one</DIV>tail", "one\n\ntail\n");
    assert_all_routes("<HTML><BODY><p>one</p><p>two</p>", "one\n\ntwo\n");
    // ~keep The end tag of any heading ends an open heading.
    assert_default_and_tier2("<!doctype html><h2>one</h3>two", "## one\n\ntwo\n");
}

#[test]
fn a_fragment_with_no_body_tag_keeps_its_header_after_the_second_parse() {
    // ~keep The page header rule drops a `<header>` in the `<body>` of a page. A fragment has no
    // ~keep `<body>` tag, and the tree builder must not give it one.
    assert_all_routes("<header>h</header><div><p>one</div>tail", "h\n\none\n\ntail\n");
    assert_all_routes("<header>h</header><b><p>one</p></b>", "h\n\n**one**\n");
    assert_all_routes("<body><header>h</header><div><p>one</div>tail", "one\n\ntail\n");
    assert_all_routes("<body><header>h</header><b><p>one</p></b>", "**one**\n");
    // ~keep An `<html>` start tag does not make a page: only a `<body>` start tag does.
    assert_all_routes("<html><header>h</header><div><p>one</div>tail", "h\n\none\n\ntail\n");
    assert_all_routes("<html><header>h</header><b><p>one</p></b>", "h\n\n**one**\n");
}

#[test]
fn a_page_that_omits_an_end_tag_converts_as_the_page_with_every_end_tag() {
    for (open, closed, expected) in [
        (
            "<nav><p>one</nav><aside><p>two</aside>tail",
            "<nav><p>one</p></nav><aside><p>two</p></aside>tail",
            "two\n\ntail\n",
        ),
        // ~keep A `<header>` directly in the `<body>` is a page header, and it is dropped.
        (
            "<body><article><p>one</article><header>site</header><p>two</p>",
            "<body><article><p>one</p></article><header>site</header><p>two</p>",
            "one\n\ntwo\n",
        ),
        (
            "<body><main><p>one</main><header>late header</header><p>two</p>",
            "<body><main><p>one</p></main><header>late header</header><p>two</p>",
            "one\n\ntwo\n",
        ),
        (
            "<main><p>one</main><header>late header</header><p>two</p>",
            "<main><p>one</p></main><header>late header</header><p>two</p>",
            "one\n\nlate header\n\ntwo\n",
        ),
        // ~keep A block quote can interrupt a paragraph, so no blank line comes before it.
        (
            "<p>one<blockquote>two</blockquote>three",
            "<p>one</p><blockquote>two</blockquote>three",
            "one\n> two\n\nthree\n",
        ),
    ] {
        assert_all_routes(open, expected);
        assert_all_routes(closed, expected);
    }
}

#[test]
fn a_heading_start_tag_ends_an_open_heading() {
    assert_default_and_tier2("<h2>one<h3>two</h3>three", "## one\n\n### two\n\nthree\n");
    assert_all_routes("<h2>one</h2><h3>two</h3>three", "## one\n\n### two\n\nthree\n");
}

#[test]
fn a_page_that_nests_past_the_limit_is_still_cut_and_reports_it() {
    let html = format!(
        "<!doctype html>{}<p>Deep text.</p>{}<p>Shallow text.</p>",
        "<div>".repeat(70),
        "</div>".repeat(70)
    );
    let default = convert(
        &html,
        ConversionOptions {
            max_depth: Some(64),
            ..Default::default()
        },
    )
    .expect("conversion must succeed");
    let routes = [
        default,
        convert_with_depth(&html, TierStrategy::Tier2, Some(64)),
        convert_with_depth(&html, TierStrategy::Tier1, Some(64)),
    ];
    for result in routes {
        assert_eq!(result.content.as_deref(), Some("Shallow text.\n"));
        assert_eq!(result.warnings.len(), 1, "warnings: {:?}", result.warnings);
        assert!(matches!(result.warnings[0].kind, WarningKind::DepthLimitExceeded));
    }
}

/// The content and the number of depth warnings on the default route and on tier 2.
fn content_and_depth_warnings(html: &str) -> [(Option<String>, usize); 2] {
    content_and_depth_warnings_at_depth(html, None)
}

fn content_and_depth_warnings_at_depth(html: &str, max_depth: Option<usize>) -> [(Option<String>, usize); 2] {
    let default = convert(
        html,
        ConversionOptions {
            max_depth,
            ..Default::default()
        },
    )
    .expect("conversion must succeed");
    [default, convert_with_depth(html, TierStrategy::Tier2, max_depth)].map(|result| {
        let depth_warnings = result
            .warnings
            .iter()
            .filter(|warning| matches!(warning.kind, WarningKind::DepthLimitExceeded))
            .count();
        assert_eq!(depth_warnings, result.warnings.len(), "warnings: {:?}", result.warnings);
        (result.content, depth_warnings)
    })
}

#[test]
fn the_deepest_fragment_that_converts_still_converts_with_an_omitted_end_tag() {
    // ~keep The second parse must not make the tree deeper than the input wrote it: the tree
    // ~keep builder gives every document an `<html>` and a `<body>`, and the depth limit counts
    // ~keep each level. An explicit 64-level budget preserves the boundary tested in 3.17.2.
    let nested = |depth: usize| format!("{}<p>x", "<div>".repeat(depth));
    for (content, depth_warnings) in content_and_depth_warnings_at_depth(&nested(62), Some(64)) {
        assert_eq!(content.as_deref(), Some("x\n"));
        assert_eq!(depth_warnings, 0);
    }
    for (content, depth_warnings) in content_and_depth_warnings_at_depth(&nested(63), Some(64)) {
        assert_eq!(content.as_deref(), Some(""));
        assert_eq!(depth_warnings, 1);
    }
}

#[test]
fn a_list_that_nests_past_the_limit_keeps_every_bullet_above_the_limit() {
    // ~keep The output of 3.17.2 with a 64-level budget: one empty bullet for each of 32 lists.
    let bullets: Vec<&str> = ["-", "*", "+"].into_iter().cycle().take(32).collect();
    let expected = format!("{}\n", bullets.join(" "));
    let html = format!("{}x", "<ul><li>".repeat(100));
    for (content, depth_warnings) in content_and_depth_warnings_at_depth(&html, Some(64)) {
        assert_eq!(content.as_deref(), Some(expected.as_str()));
        assert_eq!(depth_warnings, 1);
    }
}

#[test]
fn an_omitted_end_tag_leaves_the_depth_limit_where_the_page_with_every_end_tag_has_it() {
    for document_start in ["", "<body>", "<html>", "<html><body>", "<!doctype html>"] {
        let mut converted_depths = 0;
        for depth in 54..=68 {
            let open = "<div>".repeat(depth);
            let close = "</div>".repeat(depth);
            let closed =
                content_and_depth_warnings_at_depth(&format!("{document_start}{open}<p>x</p>{close}"), Some(64));
            converted_depths += usize::from(closed[0].1 == 0);
            for omitted in [
                format!("{document_start}{open}<p>x"),
                format!("{document_start}{open}<p>x{close}"),
            ] {
                assert_eq!(
                    content_and_depth_warnings_at_depth(&omitted, Some(64)),
                    closed,
                    "{document_start:?} at depth {depth}: {:?}",
                    &omitted[omitted.len() - 12..]
                );
            }
        }
        // ~keep The range holds the limit: some depths convert and some are cut.
        assert!(
            (1..15).contains(&converted_depths),
            "{document_start:?}: {converted_depths}"
        );
    }
}

#[test]
fn a_deep_page_that_took_the_second_parse_before_keeps_its_text_and_its_warning() {
    // ~keep A cell with no row and a custom element asked for the second parse before issue
    // ~keep #772, and the first parse hides such a cell. They still take the second parse, and
    // ~keep the tree builder stops at its depth limit (issue #808): the page keeps the words in
    // ~keep front of the deep tables and behind them. The `x` is 800 levels deep, past the depth
    // ~keep of the walk: the walk skips it and records its own limit. The warning of the walk
    // ~keep names skipped content, so it is the one warning of the conversion.
    let open = "<table><td>".repeat(400);
    let close = "</td></table>".repeat(400);
    let pages = [
        (format!("<table><td>one<table><td>two{open}x"), "one\n\ntwo\n"),
        (
            format!("<table><td>one{open}x{close}</td></table><p>tail</p>"),
            "one\n\ntail\n",
        ),
        (
            format!("<x-note>note</x-note>{open}x{close}<p>tail</p>"),
            "note\n\ntail\n",
        ),
        (format!("<x-note><p>one</x-note>{open}x"), "one\n"),
    ];
    for (html, expected) in &pages {
        let default = convert(
            html,
            ConversionOptions {
                max_depth: Some(64),
                ..Default::default()
            },
        )
        .expect("conversion must succeed");
        for result in [default, convert_with_depth(html, TierStrategy::Tier2, Some(64))] {
            assert_eq!(result.content.as_deref(), Some(*expected), "{:?}", &html[..40]);
            assert_eq!(result.warnings.len(), 1, "warnings: {:?}", result.warnings);
            assert!(matches!(result.warnings[0].kind, WarningKind::DepthLimitExceeded));
            assert!(result.warnings[0].message.contains("depth limit of 64"));
        }
    }
}

#[test]
fn nested_tables_with_an_omitted_end_tag_convert_as_the_page_with_every_end_tag() {
    // ~keep The tree builder gives a table with a bare row a `<tbody>`: one more level for each
    // ~keep table, so 16 nested tables reached the depth limit that 22 reach with every end tag.
    // ~keep The outputs are those of 3.17.2.
    for document_start in ["", "<!doctype html><html><body>"] {
        for depth in [15, 16, 17, 20, 21, 22, 25] {
            let open = format!("{document_start}<p>lead words</p>{}", "<table><tr><td>".repeat(depth));
            let close = "</td></tr></table>".repeat(depth);
            let closed = content_and_depth_warnings(&format!("{open}<p>inner words</p>{close}<p>tail words</p>"));
            if document_start.is_empty() && depth <= 20 {
                for (content, depth_warnings) in &closed {
                    assert_eq!(
                        content.as_deref(),
                        Some("lead words\n\n| inner words |\n| ----------- |\n\ntail words\n")
                    );
                    assert_eq!(*depth_warnings, 0);
                }
            }
            for omitted in [
                format!("{open}<p>inner words</p>{close}<p>tail words"),
                format!("{open}<p>inner words{close}<p>tail words</p>"),
            ] {
                assert_eq!(
                    content_and_depth_warnings(&omitted),
                    closed,
                    "{document_start:?} at depth {depth}: {:?}",
                    &omitted[omitted.len() - 12..]
                );
            }
            assert_eq!(
                content_and_depth_warnings(&format!("{open}<p>inner words")),
                content_and_depth_warnings(&format!("{open}<p>inner words</p>{close}")),
                "{document_start:?} at depth {depth}: no end tag"
            );
        }
    }
}

#[test]
fn a_body_start_tag_after_content_keeps_the_header_in_front_of_it() {
    let closed = "<header>h</header><p>one</p><body class=b><div><p>two</p></div>tail";
    let omitted = "<header>h</header><p>one</p><body class=b><div><p>two</div>tail";
    for (content, depth_warnings) in content_and_depth_warnings(closed)
        .into_iter()
        .chain(content_and_depth_warnings(omitted))
    {
        assert_eq!(content.as_deref(), Some("h\n\none\n\ntwo\n\ntail\n"));
        assert_eq!(depth_warnings, 0);
    }
}

/// ~keep In a frameset document the HTML tree builder drops everything after `</frameset>` and
/// ~keep everything in the frameset but a `<noframes>`. The converter wrote that text before
/// ~keep issue #772, and a second parse that would lose text is not used. Each expected
/// ~keep string is the output before issue #772, not the tree of a browser.
#[test]
fn a_frameset_page_keeps_the_text_that_the_tree_builder_drops() {
    for (html, expected) in [
        (
            "<frameset><frame></frameset><div><p>one</p></div><p>tail</p>",
            "one\n\ntail\n",
        ),
        ("<frameset><frame></frameset>tail", "tail\n"),
        ("<FRAMESET><FRAME></FRAMESET>tail", "tail\n"),
        ("<frameset></frameset><div><p>one</p></div><p>tail</p>", "one\n\ntail\n"),
        (
            "<frameset><frame><noframes><p>one</p></noframes></frameset><div><p>two</p></div>tail",
            "one\n\ntwo\n\ntail\n",
        ),
    ] {
        assert_default_and_tier2(html, expected);
    }
}

/// ~keep Not covered by issue #772: the omitted end tag of a frameset page is not repaired, so
/// ~keep the tail still joins the paragraph, as before.
#[test]
fn a_frameset_page_with_an_omitted_end_tag_converts_as_before() {
    for (html, expected) in [
        ("<frameset><frame></frameset><div><p>one</div>tail", "onetail\n"),
        ("<frameset></frameset><div><p>one</div>tail", "onetail\n"),
        ("<FRAMESET><FRAME></FRAMESET><div><p>one</div>tail", "onetail\n"),
        (
            "<frameset><frame><noframes><p>one</noframes></frameset><div><p>two</div>tail",
            "one\n\ntwotail\n",
        ),
        (
            "<svg><foreignObject><frameset></frameset></foreignObject></svg><div><p>one</div>tail",
            "![](data:image/svg+xml;base64,PHN2Zz48Zm9yZWlnbm9iamVjdD48ZnJhbWVzZXQgLz48L2ZvcmVpZ25vYmplY3Q+PC9zdmc+)\n\nonetail\n",
        ),
        (
            "<math><mtext><frameset></frameset></mtext></math><div><p>one</div>tail",
            "onetail\n",
        ),
        (
            "<div><frameset><frame></frameset></div><div><p>one</div>tail",
            "onetail\n",
        ),
    ] {
        assert_default_and_tier2(html, expected);
    }
}

/// ~keep The tree builder moves a `<noframes>` that starts a page into the head and keeps its
/// ~keep content as text. The first parser reads that content as elements, so the repaired
/// ~keep page comes back with the text of the page in the head, where the converter writes
/// ~keep none of it. Such a repair is not used: the page converts as before issue #772.
#[test]
fn a_page_whose_text_the_second_parse_moves_into_the_head_keeps_its_text() {
    for (html, expected) in [
        ("<noframes><div></noframes><div><p>one</p></div>tail", "one\n\ntail\n"),
        (
            "<noframes><frameset></noframes><div><p>one</p></div>tail",
            "one\n\ntail\n",
        ),
        ("<noframes><frameset></noframes><div><p>one</div>tail", "onetail\n"),
        (
            "<noframes>a <b>x</b></noframes><div><p>one</div>tail",
            "a **x**\n\nonetail\n",
        ),
    ] {
        assert_default_and_tier2(html, expected);
    }
}

/// ~keep The content of an `<iframe>` or of a `<noscript>` is no markup, so a start tag in it
/// ~keep opens nothing (issue #821). The page after the element is read, and its omitted end
/// ~keep tag is repaired.
#[test]
fn a_page_after_an_iframe_or_noscript_that_holds_a_start_tag_is_repaired() {
    for html in [
        "<iframe><frameset></iframe><div><p>one</div>tail",
        "<iframe><frameset></iframe><div><p>one</p></div>tail",
        "<noscript><frameset></noscript><div><p>one</div>tail",
        "<noscript><frameset></noscript><div><p>one</p></div>tail",
    ] {
        assert_default_and_tier2(html, "one\n\ntail\n");
    }
}

/// ~keep The tree builder decodes a character reference and writes the character back, so the
/// ~keep text is counted with every reference decoded: a page with references keeps the repair.
#[test]
fn a_page_with_character_references_is_repaired() {
    assert_default_and_tier2(
        "<div><p>R&amp;D &copy 2020 &#169;&#xA9; &notit; a&nbsp;b</div>tail",
        "R&D © 2020 ©© ¬it; a b\n\ntail\n",
    );
}

/// ~keep The tree builder moves a `<title>` that starts a page into the head. The default route
/// ~keep writes that text as metadata; the forced second tier wrote it as body text and stops.
#[test]
fn a_page_that_starts_with_a_title_is_repaired() {
    let html = "<TITLE>My page</TITLE><div><p>one</div>tail<!-- <frameset> -->";
    let default = convert(html, None).expect("conversion must succeed");
    assert_eq!(
        default.content.as_deref(),
        Some("---\ntitle: My page\n---\n\none\n\ntail\n")
    );
    let tier2 = convert_with(html, TierStrategy::Tier2);
    assert_eq!(tier2.content.as_deref(), Some("one\n\ntail\n"));
}

/// ~keep A `<frameset>` start tag makes a frameset document only while the body has no
/// ~keep content, and never in SVG or in `MathML`. The name in a comment, in an attribute
/// ~keep value, in the text of a `<textarea>`, of a script or of a `<noframes>` in the body,
/// ~keep or as the start of a longer tag name makes none either. The repair loses no text of
/// ~keep such a page, so its omitted end tag is repaired.
#[test]
fn a_frameset_tag_that_makes_no_frameset_document_does_not_stop_the_repair() {
    for (html, expected) in [
        (
            "<svg><frameset></frameset></svg><div><p>one</div>tail",
            "![](data:image/svg+xml;base64,PHN2Zz48ZnJhbWVzZXQgLz48L3N2Zz4=)\n\none\n\ntail\n",
        ),
        (
            "<math><frameset></frameset></math><div><p>one</div>tail",
            "one\n\ntail\n",
        ),
        (
            "<p>x</p><frameset></frameset><div><p>one</div>tail",
            "x\n\none\n\ntail\n",
        ),
        (
            "<div>lead</div><frameset><frame></frameset><div><p>one</div>tail",
            "lead\n\none\n\ntail\n",
        ),
        ("<body><frameset></frameset><div><p>one</div>tail", "one\n\ntail\n"),
        (
            "<p>x</p><noframes><frameset></noframes><div><p>one</div>tail",
            "x\n\none\n\ntail\n",
        ),
        (
            "<template><frameset></frameset></template><div><p>one</div>tail",
            "one\n\ntail\n",
        ),
        (
            "<xmp><frameset></xmp><div><p>one</div>tail",
            "<frameset>\n\none\n\ntail\n",
        ),
        ("<!-- <frameset> --><div><p>one</div>tail", "one\n\ntail\n"),
        ("<framesetter></framesetter><div><p>one</div>tail", "one\n\ntail\n"),
        ("<div title=\"<frameset>\"><p>one</div>tail", "one\n\ntail\n"),
        (
            "<script>var s = \"<frameset>\";</script><div><p>one</div>tail",
            "one\n\ntail\n",
        ),
        (
            "<textarea><frameset></textarea><div><p>one</div>tail",
            "<frameset>\n\none\n\ntail\n",
        ),
    ] {
        assert_default_and_tier2(html, expected);
    }
}

/// ~keep An SVG or a `MathML` element named `tbody` or `colgroup` is no part of a table. The
/// ~keep private mark that the repair gives each such start tag must not reach the output.
#[test]
fn a_table_part_name_outside_a_table_keeps_no_private_mark() {
    for (html, expected) in [
        (
            "<svg><tbody>x</tbody></svg><div><p>one</div>tail",
            "![](data:image/svg+xml;base64,PHN2Zz48dGJvZHk+eDwvdGJvZHk+PC9zdmc+)\n\none\n\ntail\n",
        ),
        (
            "<svg><colgroup></colgroup><text>t</text></svg><div><p>one</div>tail",
            "![t](data:image/svg+xml;base64,PHN2Zz48Y29sZ3JvdXAgLz48dGV4dD50PC90ZXh0Pjwvc3ZnPg==)\n\none\n\ntail\n",
        ),
        (
            "<math><tbody>x</tbody></math><div><p>one</div>tail",
            // ~keep A `MathML` element writes its visible text and no comment (issue #754).
            "x\n\none\n\ntail\n",
        ),
    ] {
        assert_default_and_tier2(html, expected);
    }
}
