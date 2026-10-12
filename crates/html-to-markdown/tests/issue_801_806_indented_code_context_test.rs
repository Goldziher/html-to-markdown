#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, TierStrategy, convert};

fn check(html: &str, expected: &str, options: &ConversionOptions) {
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy,
                ..options.clone()
            }),
        )
        .expect("conversion must succeed");
        let markdown = result.content.unwrap_or_default();
        assert_eq!(markdown, expected, "{tier_strategy:?}: {html:?}");
        if html.contains("<pre>a\nb\n</pre>") {
            let rendered = comrak::markdown_to_html(&markdown, &comrak::Options::default());
            assert!(rendered.contains("<pre><code>a\nb\n</code></pre>"), "{rendered}");
        }
    }
}

#[test]
fn should_end_a_list_before_a_following_indented_code_block() {
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..Default::default()
    };
    for (list, marker) in [("ul", "-"), ("ol", "1.")] {
        check(
            &format!("<{list}><li>t</li></{list}><pre>a\nb\n</pre>"),
            &format!("{marker} t\n\n<!-- -->\n\n    a\n    b\n"),
            &options,
        );
        check(
            &format!("<blockquote><{list}><li>t</li></{list}><pre>a\nb\n</pre></blockquote>"),
            &format!("> {marker} t\n>\n> <!-- -->\n>\n>     a\n>     b\n"),
            &options,
        );
    }
    check("<p>t</p><pre>a\nb\n</pre>", "t\n\n    a\n    b\n", &options);
}

#[test]
fn should_preserve_definition_code_indentation_inside_list_items() {
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..Default::default()
    };
    for (list, marker, indent) in [("ul", "-", "      "), ("ol", "1.", "       ")] {
        check(
            &format!("<{list}><li><dl><dt>t</dt><dd><pre>a\nb\n</pre></dd></dl></li></{list}>"),
            &format!("{marker} t\n\n{indent}a\n{indent}b\n"),
            &options,
        );
    }
}

#[test]
fn should_keep_nested_definition_code_blocks_inside_the_item() {
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..Default::default()
    };
    check(
        "<ul><li>outer<ul><li><dl><dt>t</dt><dd><pre>a\nb\n</pre></dd></dl></li></ul></li></ul>",
        "- outer\n  * t\n\n        a\n        b\n",
        &options,
    );
    check(
        "<blockquote><ul><li><dl><dt>t</dt><dd><pre>a\nb\n</pre></dd></dl></li></ul></blockquote>",
        "> - t\n>\n>       a\n>       b\n",
        &options,
    );
}

#[test]
fn should_keep_blank_code_lines_after_a_terminated_list() {
    let options = ConversionOptions {
        code_block_style: CodeBlockStyle::Indented,
        ..Default::default()
    };
    check(
        "<ul><li>t</li></ul><pre>a\n\n\nb\n</pre>",
        "- t\n\n<!-- -->\n\n    a\n\n\n    b\n",
        &options,
    );
}

#[test]
fn should_keep_fenced_definition_code_and_code_after_a_paragraph_inside_the_item() {
    for (list, marker, content_indent) in [("ul", "-", "  "), ("ol", "1.", "   ")] {
        let definition = format!("<{list}><li><dl><dt>t</dt><dd><pre>a\nb\n</pre></dd></dl></li></{list}>");
        for (code_block_style, fence) in [(CodeBlockStyle::Backticks, "```"), (CodeBlockStyle::Tildes, "~~~")] {
            let options = ConversionOptions {
                code_block_style,
                ..Default::default()
            };
            check(
                &definition,
                &format!(
                    "{marker} t\n\n{content_indent}{fence}\n{content_indent}a\n{content_indent}b\n{content_indent}{fence}\n"
                ),
                &options,
            );
        }
        let options = ConversionOptions {
            code_block_style: CodeBlockStyle::Indented,
            ..Default::default()
        };
        check(
            &format!("<{list}><li><p>t</p><pre>a\nb\n</pre></li></{list}>"),
            &format!("{marker} t\n\n{content_indent}    a\n{content_indent}    b\n"),
            &options,
        );
    }
}
