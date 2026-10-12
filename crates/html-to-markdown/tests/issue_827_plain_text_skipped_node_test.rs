//! Issue 827 in plain output: a node that the plain writer passes over (a structured data
//! script, a template, a comment, `noscript`, `math`) is no part of the text, so the white space
//! on its two sides is one space. An element that is walked and writes nothing keeps both spaces.

#![allow(missing_docs)]
// ~keep The converter is chosen by name, which the crate offers under this feature only.
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

/// The default route and the full converter. In plain output the two write the same text.
const FULL_ROUTES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

const STRUCTURED_DATA: &str = r#"<script type="application/ld+json">{"a":1}</script>"#;
const TEMPLATE: &str = "<template><b>t</b></template>";
const COMMENT: &str = "<!-- c -->";
const NOSCRIPT: &str = "<noscript><b>n</b></noscript>";
const MATH: &str = "<math><mi>m</mi></math>";

/// The nodes that the plain writer passes over.
const PASSED_OVER: [&str; 5] = [STRUCTURED_DATA, TEMPLATE, COMMENT, NOSCRIPT, MATH];

/// The nodes that the fast converter writes nothing for as well.
const PASSED_OVER_ON_THE_FAST_CONVERTER: [&str; 3] = [STRUCTURED_DATA, TEMPLATE, COMMENT];

/// The two sides of a node between two spaces: in a paragraph, a heading, a list item, a table
/// cell and a `div`, and as the first text of the page.
const PLACES_BETWEEN_TWO_SPACES: [(&str, &str); 6] = [
    ("<p>foo ", " bar</p>"),
    ("<h2>foo ", " bar</h2>"),
    ("<ul><li>foo ", " bar</li></ul>"),
    ("<table><tr><td>foo ", " bar</td></tr></table>"),
    ("<div>foo ", " bar</div>"),
    ("<p> ", " foo bar</p>"),
];

/// The plain text of the page, without the line ends at its end.
fn plain(html: &str, tier_strategy: TierStrategy, extract_metadata: bool) -> String {
    let options = ConversionOptions {
        output_format: OutputFormat::Plain,
        tier_strategy,
        extract_metadata,
        ..ConversionOptions::default()
    };
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim_end_matches('\n')
        .to_string()
}

#[test]
fn a_structured_data_script_between_two_spaces_leaves_one_space_in_plain_output() {
    let html = format!("<p>foo {STRUCTURED_DATA} bar</p>");
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier2, TierStrategy::Tier1] {
        for extract_metadata in [false, true] {
            for page in [html.as_str(), "<p>foo  bar</p>"] {
                assert_eq!(
                    plain(page, tier_strategy, extract_metadata),
                    "foo bar",
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {page:?}"
                );
            }
        }
    }
}

#[test]
fn a_node_that_is_passed_over_between_two_spaces_leaves_one_space_in_plain_output() {
    for node in PASSED_OVER {
        let html = format!("<p>foo {node} bar</p>");
        for tier_strategy in FULL_ROUTES {
            for extract_metadata in [false, true] {
                assert_eq!(
                    plain(&html, tier_strategy, extract_metadata),
                    "foo bar",
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {html:?}"
                );
            }
        }
    }
    for node in PASSED_OVER_ON_THE_FAST_CONVERTER {
        let html = format!("<p>foo {node} bar</p>");
        for extract_metadata in [false, true] {
            assert_eq!(
                plain(&html, TierStrategy::Tier1, extract_metadata),
                "foo bar",
                "Tier1, extract_metadata {extract_metadata}: {html:?}"
            );
        }
    }
}

#[test]
fn a_node_that_is_passed_over_leaves_one_space_in_every_place_in_plain_output() {
    let rows = PASSED_OVER
        .into_iter()
        .flat_map(|node| FULL_ROUTES.map(|tier_strategy| (node, tier_strategy)))
        .chain(PASSED_OVER_ON_THE_FAST_CONVERTER.map(|node| (node, TierStrategy::Tier1)));
    for (node, tier_strategy) in rows {
        for (before, after) in PLACES_BETWEEN_TWO_SPACES {
            let html = format!("{before}{node}{after}");
            let page_without_the_node = format!("{before}{after}");
            for extract_metadata in [false, true] {
                let expected = plain(&page_without_the_node, tier_strategy, extract_metadata);
                assert!(
                    expected.contains("foo bar"),
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {expected:?}"
                );
                assert_eq!(
                    plain(&html, tier_strategy, extract_metadata),
                    expected,
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {html:?}"
                );
            }
        }
    }
}

#[test]
fn a_tab_on_each_side_of_a_node_that_is_passed_over_leaves_one_space_in_plain_output() {
    for node in PASSED_OVER {
        let html = format!("<p>foo\t{node}\tbar</p>");
        for tier_strategy in FULL_ROUTES {
            assert_eq!(
                plain(&html, tier_strategy, false),
                "foo bar",
                "{tier_strategy:?}: {html:?}"
            );
        }
    }
}

#[test]
fn an_element_that_is_walked_and_writes_nothing_keeps_both_spaces_in_plain_output() {
    let beside_a_script = format!("<p>foo {STRUCTURED_DATA}<span></span> bar</p>");
    for (html, expected) in [
        (r#"<p>a <svg><path d="M1 1"/></svg> b</p>"#, "a  b"),
        ("<p>foo <span></span> bar</p>", "foo  bar"),
        (beside_a_script.as_str(), "foo  bar"),
    ] {
        for tier_strategy in FULL_ROUTES {
            for extract_metadata in [false, true] {
                assert_eq!(
                    plain(html, tier_strategy, extract_metadata),
                    expected,
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {html:?}"
                );
            }
        }
    }
}

#[test]
fn a_no_break_space_after_a_node_that_is_passed_over_is_kept_in_plain_output() {
    // ~keep The plain writer writes a no-break space as a space, so the text has two spaces.
    for node in PASSED_OVER {
        let html = format!("<p>foo {node}&nbsp;bar</p>");
        for tier_strategy in FULL_ROUTES {
            for extract_metadata in [false, true] {
                assert_eq!(
                    plain(&html, tier_strategy, extract_metadata),
                    "foo  bar",
                    "{tier_strategy:?}, extract_metadata {extract_metadata}: {html:?}"
                );
            }
        }
    }
}
