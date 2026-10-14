#![allow(missing_docs)]

//! A block quote keeps the lines of the indented code it holds, also after a line of a quote in it.
//!
//! A line that starts with a quote marker ends the paragraph before it, so an indented line after
//! it is code. A block quote reads the lines of a nested quote as that quote wrote them.

use html_to_markdown_rs::options::WhitespaceMode;
use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

/// Converts `html` with the default converter choice and with the full converter by name.
fn assert_indented(html: &str, whitespace_mode: WhitespaceMode, expected: &str) {
    let plain = ConversionOptions {
        whitespace_mode,
        code_block_style: CodeBlockStyle::Indented,
        ..ConversionOptions::default()
    };
    let full = ConversionOptions {
        extract_metadata: false,
        tier_strategy: TierStrategy::Tier2,
        ..plain.clone()
    };
    for (setting, options) in [("the default options", plain), ("the full converter", full)] {
        let output = convert(html, Some(options))
            .expect("conversion must succeed")
            .content
            .unwrap_or_default();
        assert_eq!(output, expected, "{setting}: {html:?}");
    }
}

#[test]
fn should_keep_a_line_of_spaces_in_code_that_follows_a_quote_marker_line_in_strict_white_space_mode() {
    // The line with the marker ends the paragraph: the lines after it are indented code, and the
    // line of one space between them keeps its space.
    assert_indented(
        "<blockquote>6\n &gt;\n    3\n \n\t7</blockquote>",
        WhitespaceMode::Strict,
        "> 6\n>  >\n>     3\n>  \n> \t7\n",
    );
}

#[test]
fn should_write_a_blank_line_between_sibling_quotes_as_the_marker_alone() {
    // The space between the two inner quotes is a blank line of the outer quote, not code.
    assert_indented(
        "<blockquote><blockquote><blockquote><pre>8</pre></blockquote></blockquote> \
         <blockquote><blockquote><pre>0</pre></blockquote></blockquote></blockquote>",
        WhitespaceMode::Normalized,
        "> > >     8\n>\n>\n>\n> > >     0\n",
    );
}
