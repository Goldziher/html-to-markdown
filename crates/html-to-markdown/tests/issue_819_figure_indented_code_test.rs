#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

fn assert_all_tiers(html: &str, expected: &str, code_block_style: CodeBlockStyle) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let options = ConversionOptions {
            tier_strategy,
            code_block_style,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(output, expected, "unexpected output for {tier_strategy:?}: {html}");
    }
}

#[test]
fn should_preserve_first_line_indentation_in_a_figure() {
    for html in [
        "<figure><pre>line one\nline two</pre></figure>",
        "<div><pre>line one\nline two</pre></div>",
    ] {
        assert_all_tiers(html, "    line one\n    line two\n", CodeBlockStyle::Indented);
    }
}

#[test]
fn should_preserve_indented_code_before_a_figure_caption() {
    assert_all_tiers(
        "<figure><pre>line one\nline two</pre><figcaption>Caption</figcaption></figure>",
        "    line one\n    line two\n\n*Caption*\n",
        CodeBlockStyle::Indented,
    );
}

#[test]
fn should_preserve_indented_code_after_a_figure_caption() {
    assert_all_tiers(
        "<figure><figcaption>Caption</figcaption><pre>line one\nline two</pre></figure>",
        "*Caption*\n\n    line one\n    line two\n",
        CodeBlockStyle::Indented,
    );
}

#[test]
fn should_keep_fenced_code_in_a_figure() {
    assert_all_tiers(
        "<figure><pre>line one\nline two</pre></figure>",
        "```\nline one\nline two\n```\n",
        CodeBlockStyle::Backticks,
    );
}

#[test]
fn should_trim_source_whitespace_around_a_figure_image() {
    assert_all_tiers(
        "<figure> \n<img src='x.png' alt='X'> \n<figcaption>Caption</figcaption> </figure>",
        "![X](x.png)\n\n*Caption*\n",
        CodeBlockStyle::Indented,
    );
}

#[test]
fn should_trim_source_whitespace_around_plain_figure_text() {
    for html in ["<figure> foo</figure>", "<figure>\tfoo</figure>"] {
        assert_all_tiers(html, "foo\n", CodeBlockStyle::Indented);
    }
}
