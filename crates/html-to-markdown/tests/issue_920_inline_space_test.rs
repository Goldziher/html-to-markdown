#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, WhitespaceMode, convert};

fn assert_routes(html: &str, expected: &str, whitespace_mode: WhitespaceMode) {
    for tier_strategy in [
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
        TierStrategy::Auto,
    ] {
        let output = convert(
            html,
            Some(ConversionOptions {
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                tier_strategy,
                whitespace_mode,
                ..ConversionOptions::default()
            }),
        )
        .expect("inline conversion should succeed")
        .content
        .unwrap_or_default();
        assert_eq!(output, expected, "{tier_strategy:?}: {html:?}");
    }
}

#[test]
fn should_keep_the_space_before_an_inline_element_after_a_list_blank_line() {
    for (tag, marked) in [("span", "word"), ("em", "*word*"), ("b", "**word**")] {
        assert_routes(
            &format!("<ul><li>.\n\nisolation is <{tag}>word</{tag}></li></ul>"),
            &format!("- .\n\n    isolation is {marked}\n"),
            WhitespaceMode::Normalized,
        );
    }
}

#[test]
fn should_keep_whitespace_after_a_link_before_an_inline_element() {
    assert_routes(
        "<p><a href=\"/x\">1</a>\n\t<font size=\"1\">Steps one</font></p>",
        "[1](/x) Steps one\n",
        WhitespaceMode::Normalized,
    );
}

#[test]
fn should_preserve_existing_single_spaces_and_intentional_adjacency() {
    for (html, expected) in [
        (
            "<ul><li>isolation is <span>word</span></li></ul>",
            "- isolation is word\n",
        ),
        ("<p>one <span>two</span></p>", "one two\n"),
        ("<p>one<span>two</span></p>", "onetwo\n"),
        (
            "<p><a href=\"/x\">1</a><font>Steps one</font></p>",
            "[1](/x)Steps one\n",
        ),
    ] {
        assert_routes(html, expected, WhitespaceMode::Normalized);
    }
}

#[test]
fn should_keep_source_newlines_in_strict_whitespace_mode() {
    assert_routes(
        "<p><a href=\"/x\">1</a>\n\t<font size=\"1\">Steps one</font></p>",
        "[1](/x)\n\tSteps one\n",
        WhitespaceMode::Strict,
    );
}

#[test]
fn should_keep_the_inline_space_after_other_spellings_of_a_list_blank_line() {
    for blank_line in ["\r\n\r\n", "\n \t\n", "&#10;&#10;"] {
        assert_routes(
            &format!("<ul><li>.{blank_line}isolation is <span>word</span></li></ul>"),
            "- .\n\n    isolation is word\n",
            WhitespaceMode::Normalized,
        );
    }
}

#[cfg(feature = "testkit")]
#[test]
fn should_keep_ordinary_list_continuation_lines_on_the_fast_path() {
    let options = ConversionOptions {
        extract_metadata: false,
        highlight_style: HighlightStyle::None,
        ..ConversionOptions::default()
    };
    let html = "<ul><li>first\nsecond\nthird <span>word</span></li></ul>";
    let expected = "- first\n  second\n  third word\n";
    let output =
        html_to_markdown_rs::tier1::run(html, &html_to_markdown_rs::prescan::PrescanReport::default(), &options)
            .expect("ordinary continuation lines should stay on the fast path");
    assert_eq!(output, expected);
    assert_routes(html, expected, WhitespaceMode::Normalized);
}
