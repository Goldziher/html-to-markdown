// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests for lists that must stay apart from what comes before them: an ordered list
//! right after an ordered list writes the `)` delimiter (issue #666), and in Djot output a list
//! after text starts after a blank line (issue #670).

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn tier2_options() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Tier2,
        ..ConversionOptions::default()
    }
}

fn djot_options() -> ConversionOptions {
    ConversionOptions {
        output_format: OutputFormat::Djot,
        ..tier2_options()
    }
}

fn convert_with(html: &str, options: &ConversionOptions) -> String {
    convert(html, Some(options.clone()))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn render(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    comrak::markdown_to_html(markdown, &options)
}

/// Convert `html`, assert the exact markdown, and assert that its rendering opens `lists` lists
/// whose tag starts with `tag`.
fn assert_lists(html: &str, options: &ConversionOptions, expected: &str, tag: &str, lists: usize) {
    let markdown = convert_with(html, options);
    assert_eq!(markdown, expected, "{html}");
    let rendered = render(&markdown);
    assert_eq!(rendered.matches(tag).count(), lists, "{html}: {rendered:?}");
}

#[cfg(feature = "testkit")]
#[test]
fn should_keep_two_ordered_lists_next_to_each_other_apart() {
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto, TierStrategy::Tier1] {
        let options = ConversionOptions {
            tier_strategy: strategy,
            ..tier2_options()
        };
        assert_lists(
            "<ol><li>a</li></ol><ol><li>b</li></ol>",
            &options,
            "1. a\n\n1) b\n",
            "<ol",
            2,
        );
        assert_lists(
            "<ol><li>a</li></ol><ol><li>b</li></ol><ol><li>c</li></ol>",
            &options,
            "1. a\n\n1) b\n\n1. c\n",
            "<ol",
            3,
        );
    }
}

#[test]
fn should_keep_adjacent_ordered_lists_apart_inside_a_list_item_and_a_quote() {
    let options = tier2_options();
    assert_lists(
        "<ul><li>x<ol><li>a</li></ol><ol><li>b</li></ol></li></ul>",
        &options,
        "- x\n  1. a\n  1) b\n",
        "<ol",
        2,
    );
    assert_lists(
        "<ol><li>a<ol><li>n</li></ol></li></ol><ol><li>b</li></ol>",
        &options,
        "1. a\n   1. n\n\n1) b\n",
        "<ol",
        3,
    );
    let markdown = convert_with(
        "<blockquote><ol><li>a</li></ol><ol><li>b</li></ol></blockquote>",
        &options,
    );
    assert!(markdown.contains("> 1) b"), "{markdown:?}");
    assert_eq!(render(&markdown).matches("<ol").count(), 2, "{markdown:?}");
}

#[test]
fn should_keep_a_nested_list_on_the_marker_line_of_a_switched_ordered_item() {
    assert_lists(
        "<ol><li>a</li></ol><ol><li><ul><li>b</li></ul></li></ol>",
        &tier2_options(),
        "1. a\n\n1) - b\n",
        "<ol",
        2,
    );
}

#[test]
fn should_keep_the_default_markers_when_something_separates_the_lists() {
    let options = tier2_options();
    assert_lists(
        "<ol><li>a</li></ol><p>x</p><ol><li>b</li></ol>",
        &options,
        "1. a\n\nx\n\n1. b\n",
        "<ol",
        2,
    );
    assert_eq!(
        convert_with("<ol><li>a</li></ol><!-- c --><ol><li>b</li></ol>", &options),
        "1. a\n\n<!-- -->\n\n1. b\n"
    );
    assert_lists(
        "<ul><li>a</li></ul><ol><li>b</li></ol>",
        &options,
        "- a\n\n1. b\n",
        "<ol",
        1,
    );
    // ~keep The list before ends inside an item, at another column: nothing to continue.
    let markdown = convert_with("<li>x<ol><li>a</li></ol></li><ol><li>b</li></ol>", &options);
    assert!(markdown.ends_with("\n1. b\n"), "{markdown:?}");
}

#[test]
fn should_keep_two_ordered_lists_apart_in_djot() {
    assert_eq!(
        convert_with("<ol><li>a</li></ol><ol><li>b</li></ol>", &djot_options()),
        "1. a\n\n1) b\n"
    );
}

#[cfg(feature = "testkit")]
#[test]
fn should_hand_an_ordered_list_after_an_ordered_list_to_the_full_converter() {
    use html_to_markdown_rs::prescan::PrescanReport;
    use html_to_markdown_rs::tier1;
    use html_to_markdown_rs::tier1::BailReason;

    let options = tier2_options();
    for html in [
        "<ol><li>a</li></ol><ol><li>b</li></ol>",
        "<ol><li>a</li></ol>\n<ol><li>b</li></ol>",
        "<ol><li>a</li></ol><span></span><ol><li>b</li></ol>",
    ] {
        let result = tier1::run(html, &PrescanReport::default(), &options);
        assert!(
            matches!(result, Err(BailReason::OrderedListAfterOrderedList)),
            "{html}: {result:?}"
        );
    }
    for html in [
        "<ol><li>a</li></ol><p>x</p><ol><li>b</li></ol>",
        "<ol><li>a</li></ol><ul><li>b</li></ul>",
        "<ul><li>a</li></ul><ul><li>b</li></ul>",
        "<ul><li>a</li></ul><ol><li>b</li></ol>",
    ] {
        let result = tier1::run(html, &PrescanReport::default(), &options);
        assert!(result.is_ok(), "{html}: {result:?}");
    }
}

// ~keep Djot needs a blank line before a list that follows text (djot syntax.md, "List item"):
// ~keep djot.js renders `- a\n  * b` as one paragraph `a * b`, and `- a\n\n  * b` as a nested
// ~keep list in a tight item.
#[test]
fn should_start_a_nested_list_after_a_blank_line_in_djot() {
    let options = djot_options();
    for (html, expected) in [
        (
            r#"<ol start="9"><li><input type="checkbox">a</li><li><input type="checkbox">b<ul><li>n</li></ul></li></ol>"#,
            "- [ ] a\n- [ ] b\n\n  - n\n",
        ),
        ("<ul><li>a<ul><li>b</li></ul></li></ul>", "- a\n\n  * b\n"),
        ("<ol><li>a<ol><li>b</li></ol></li></ol>", "1. a\n\n   1. b\n"),
        ("<ul><li><ul><li>b</li></ul></li></ul>", "- * b\n"),
        ("<ul><li><p>a</p><ul><li>b</li></ul></li></ul>", "- a\n\n  * b\n"),
    ] {
        assert_eq!(convert_with(html, &options), expected, "{html}");
    }
}

#[test]
fn should_keep_a_nested_bullet_list_under_its_text_in_markdown() {
    assert_eq!(
        convert_with("<ul><li>a<ul><li>b</li></ul></li></ul>", &tier2_options()),
        "- a\n  * b\n"
    );
}

#[cfg(feature = "visitor")]
#[test]
fn should_give_the_visitor_the_marker_each_item_writes() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Markers(Vec<(String, String)>);

    impl HtmlVisitor for Markers {
        fn visit_list_item(&mut self, _ctx: &NodeContext, _ordered: bool, marker: &str, text: &str) -> VisitResult {
            self.0.push((marker.to_string(), text.to_string()));
            VisitResult::Continue
        }
    }

    for (html, expected) in [
        ("<ol><li>a</li></ol><ol><li>b</li></ol>", [("1.", "a"), ("1)", "b")]),
        ("<ul><li>a</li></ul><ol><li>b</li></ol>", [("-", "a"), ("1.", "b")]),
    ] {
        let visitor = Arc::new(Mutex::new(Markers::default()));
        let options = ConversionOptions {
            visitor: Some(visitor.clone()),
            ..tier2_options()
        };
        convert_with(html, &options);
        let seen = visitor.lock().expect("visitor lock").0.clone();
        let expected: Vec<(String, String)> = expected
            .iter()
            .map(|(marker, text)| ((*marker).to_string(), (*text).to_string()))
            .collect();
        assert_eq!(seen, expected, "{html}");
    }
}

#[test]
fn should_flatten_adjacent_ordered_lists_into_heading_labels() {
    let html = "<h2><ol><li>a</li></ol><ol><li>b</li></ol></h2>";
    let markdown = convert_with(html, &tier2_options());
    assert_eq!(markdown, "## a b\n");
    assert_eq!(render(&markdown), "<h2>a b</h2>\n");
}

// ~keep An ordered list written as text continues nothing, so it keeps the `.` delimiter.
#[test]
fn should_keep_the_dot_delimiter_of_ordered_lists_written_as_text() {
    let options = tier2_options();
    for (html, expected) in [
        (
            "<details><summary><ol><li>a</li></ol><ol><li>b</li></ol></summary></details>",
            "**1. a\n\n1. b**\n",
        ),
        (
            "<ul><li><b>x<ol><li>a</li></ol><ol><li>b</li></ol></b></li></ul>",
            "- **x\n  1. a\n  1. b**\n",
        ),
        (
            "<ul><li><mark>x<ol><li>a</li></ol><ol><li>b</li></ol></mark></li></ul>",
            "- ==x\n  1. a\n  1. b==\n",
        ),
    ] {
        assert_eq!(convert_with(html, &options), expected, "{html}");
    }
}

#[test]
fn should_keep_the_dot_delimiter_after_an_ordered_list_that_a_block_closed() {
    assert_eq!(
        convert_with("<ol><li>a</li><h2>h</h2></ol><ol><li>b</li></ol>", &tier2_options()),
        "1. a\n\n## h\n\n1. b\n"
    );
}

// ~keep Sectioning and grouping elements write into a buffer of their own and append it to
// ~keep their parent's: the list before still ends the output, so the next list switches.
const WRAPPERS: [&str; 9] = [
    "section", "article", "main", "aside", "header", "footer", "figure", "details", "fieldset",
];

fn wrapped(tag: &str, inner: &str) -> String {
    format!("<{tag}>{inner}</{tag}>")
}

/// The two adjacent lists `<ol>a</ol><ol>b</ol>` with `tag` around both, only the first, only
/// the second, and both inside a second wrapper.
fn wrapped_pairs(tag: &str) -> [String; 4] {
    let (a, b) = ("<ol><li>a</li></ol>", "<ol><li>b</li></ol>");
    [
        format!("{}{}", wrapped(tag, a), wrapped(tag, b)),
        format!("{}{b}", wrapped(tag, a)),
        format!("{a}{}", wrapped(tag, b)),
        format!(
            "{}{}",
            wrapped(tag, &wrapped("section", a)),
            wrapped("div", &wrapped(tag, b))
        ),
    ]
}

#[test]
fn should_keep_adjacent_ordered_lists_apart_when_an_element_wraps_either_list() {
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let options = ConversionOptions {
            tier_strategy: strategy,
            ..tier2_options()
        };
        for tag in WRAPPERS {
            for html in wrapped_pairs(tag) {
                let markdown = convert_with(&html, &options);
                assert!(markdown.contains("1. a\n\n1) b\n"), "{strategy:?} {html}: {markdown:?}");
                assert_eq!(render(&markdown).matches("<ol").count(), 2, "{html}: {markdown:?}");
            }
        }
    }
}

#[test]
fn should_keep_wrapped_adjacent_ordered_lists_apart_in_djot() {
    for tag in WRAPPERS {
        for html in wrapped_pairs(tag) {
            let markdown = convert_with(&html, &djot_options());
            assert!(markdown.contains("1. a\n\n1) b\n"), "{html}: {markdown:?}");
        }
    }
}

#[test]
fn should_keep_wrapped_adjacent_ordered_lists_apart_inside_a_list_item_and_a_quote() {
    let options = tier2_options();
    for tag in WRAPPERS {
        for html in wrapped_pairs(tag) {
            for outer in [
                format!("<ul><li>x{html}</li></ul>"),
                format!("<blockquote>{html}</blockquote>"),
            ] {
                let markdown = convert_with(&outer, &options);
                assert!(markdown.contains("1) b"), "{outer}: {markdown:?}");
                assert_eq!(render(&markdown).matches("<ol").count(), 2, "{outer}: {markdown:?}");
            }
        }
    }
}

#[test]
fn should_keep_the_dot_delimiter_after_a_wrapped_list_that_something_follows() {
    let options = tier2_options();
    for html in [
        "<section><ol><li>a</li></ol></section><p>x</p><section><ol><li>b</li></ol></section>",
        "<section><ol><li>a</li></ol><p>x</p></section><section><ol><li>b</li></ol></section>",
        "<section><ol><li>a</li></ol></section><section><p>x</p><ol><li>b</li></ol></section>",
        "<ol><li>a</li></ol><blockquote><ol><li>b</li></ol></blockquote>",
        "<blockquote><ol><li>a</li></ol></blockquote><ol><li>b</li></ol>",
        "<ol><li>a</li></ol><table><tr><td><ol><li>b</li></ol></td></tr></table>",
        // ~keep The next item's marker comes after the nested list, in the same buffer.
        "<ol><li>a<ol><li>x</li></ol></li><li><ol><li>y</li></ol></li></ol>",
        // ~keep A last line as long as the list's, and one that ends with the list's text.
        "<section><ol><li>a</li></ol></section><p>wxyz</p><section><ol><li>b</li></ol></section>",
        "<ol><li>a</li></ol><p>z1. a</p><ol><li>b</li></ol>",
        // ~keep A no-break space is text, not a blank line: it ends the list.
        "<ol><li>a</li></ol>&nbsp;<ol><li>b</li></ol>",
    ] {
        let markdown = convert_with(html, &options);
        assert!(!markdown.contains("1)"), "{html}: {markdown:?}");
    }
}

// ~keep A wrapper that rewrites the end of its text (a figure moving an image next to the text,
// ~keep strict whitespace trimming a no-break space) still leaves the list at the end.
#[test]
fn should_keep_adjacent_ordered_lists_apart_when_the_wrapper_rewrites_the_list_end() {
    let strict = ConversionOptions {
        whitespace_mode: html_to_markdown_rs::options::WhitespaceMode::Strict,
        ..tier2_options()
    };
    for strategy in [TierStrategy::Tier2, TierStrategy::Auto] {
        let options = ConversionOptions {
            tier_strategy: strategy,
            ..tier2_options()
        };
        for html in [
            r#"<figure><ol><li>a <img src="x.png"></li></ol></figure><ol><li>b</li></ol>"#,
            r#"<figure><ol><li>a<br><img src="x.png"></li></ol></figure><ol><li>b</li></ol>"#,
            r#"<figure><ol><li>a <img src="x.png"></li></ol></figure><figure><ol><li>b</li></ol></figure>"#,
        ] {
            let markdown = convert_with(html, &options);
            assert!(markdown.contains("1) b"), "{strategy:?} {html}: {markdown:?}");
            assert_eq!(render(&markdown).matches("<ol").count(), 2, "{html}: {markdown:?}");
        }
    }
    for html in [
        "<details><ol><li>a&nbsp;</li></ol></details><ol><li>b</li></ol>",
        "<details><ol><li>a&emsp;</li></ol></details><ol><li>b</li></ol>",
        "<fieldset><ol><li>a&nbsp;</li></ol></fieldset><ol><li>b</li></ol>",
    ] {
        let markdown = convert_with(html, &strict);
        assert!(markdown.contains("1) b"), "{html}: {markdown:?}");
        assert_eq!(render(&markdown).matches("<ol").count(), 2, "{html}: {markdown:?}");
    }
}
