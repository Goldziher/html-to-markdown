#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

fn assert_conversion(html: &str, expected: &str) {
    for tier_strategy in [
        TierStrategy::Auto,
        TierStrategy::Tier2,
        #[cfg(feature = "testkit")]
        TierStrategy::Tier1,
    ] {
        let options = ConversionOptions {
            tier_strategy,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default();
        assert_eq!(output, expected, "{tier_strategy:?}: {html}");
    }
}

#[test]
fn should_keep_stray_text_in_a_table_section() {
    assert_conversion("<table><tbody>stray</tbody></table>", "stray\n");
}

#[test]
fn should_foster_parent_stray_row_text_before_the_table() {
    assert_conversion(
        "<table><tr>stray</tr><tr><td>cell</td></tr></table>",
        "stray\n\n| cell |\n| ---- |\n",
    );
}

#[test]
fn should_keep_entity_encoded_stray_table_text() {
    assert_conversion("<table><tfoot>&#115;tray</tfoot></table>", "stray\n");
}

#[test]
fn should_leave_space_only_rows_empty() {
    assert_conversion("<table><tr>&nbsp; \t</tr></table>", "");
}

#[test]
fn should_keep_a_break_only_code_element_between_words() {
    assert_conversion("<p>a<code><br></code>z</p>", "a  \nz\n");
    assert_conversion("<p>a<code>\n<br></code>z</p>", "a  \nz\n");
}

#[test]
fn should_keep_a_heading_code_break_as_its_existing_whitespace_code_span() {
    assert_conversion("<h1>a<code><br></code>z</h1>", "# a` `z\n");
}

#[test]
fn should_keep_an_empty_code_element_empty() {
    assert_conversion("<p>a<code></code>z</p>", "az\n");
}

#[test]
fn should_keep_the_metadata_only_head_policy() {
    // ~keep Head-only raw text is intentionally suppressed with other metadata (#921).
    assert_conversion("<head>out</head>", "");
}

#[test]
fn should_keep_the_plaintext_markup_policy_and_literal_xmp_control() {
    // ~keep Unlike xmp, plaintext is converted as markup; issue #753 pins this contract.
    assert_conversion("<plaintext><base href=\"https://example.com/\">", "");
    assert_conversion("<xmp><p>", "<p>\n");
}

#[test]
fn should_bound_an_unclosed_noscript_to_its_noframes_parent() {
    assert_conversion("<noframes><noscript></noframes>b", "b\n");
    assert_conversion("<p>a</p><noframes><noscript>hidden</noframes>b", "a\n\nb\n");
    assert_conversion("<noframes><script>1</script><noscript>hidden</noframes>b", "b\n");
}

#[test]
fn should_ignore_noscript_looking_text_inside_raw_bodies_and_comments() {
    assert_conversion("<noframes><!-- <noscript> --></noframes>b", "b\n");
    assert_conversion("<noframes><script>\"<noscript>\"</script></noframes>b", "b\n");
    assert_conversion("<noframes><style>/* <noscript> */</style></noframes>b", "b\n");
}

#[test]
fn should_keep_a_block_ending_inside_a_subscript_apart_from_following_text() {
    assert_conversion("<sub><ul>b</ul></sub>c", "b\n\nc\n");
    assert_conversion("<sup><ul>b</ul></sup>c", "b\n\nc\n");
    assert_conversion("<sub>a<ul>b</ul></sub>c", "a\n\nb\n\nc\n");
}

#[test]
fn should_keep_existing_inline_block_and_repair_guard_policies() {
    // ~keep These outputs have explicit contracts in issues #772, #732 and inline whitespace tests.
    assert_conversion("<noframes>a </noframes><div><p>one</div>tail", "a\n\nonetail\n");
    assert_conversion("<h1><code>x<p>y", "# `xy`\n");
    assert_conversion("<table><td><h2>a<blockquote>x", "| ax |\n| --- |\n");
}

#[cfg(feature = "visitor")]
#[test]
fn should_ignore_breaks_erased_by_visitors_inside_newline_only_code() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct OmittedBreak {
        span_end: bool,
        custom: bool,
    }
    impl HtmlVisitor for OmittedBreak {
        fn visit_element_end(&mut self, context: &NodeContext<'_>, _: &str) -> VisitResult {
            if self.span_end && context.tag_name == "span" {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }

        fn visit_line_break(&mut self, _: &NodeContext<'_>) -> VisitResult {
            if self.span_end {
                VisitResult::Continue
            } else if self.custom {
                VisitResult::Custom("\n".to_owned())
            } else {
                VisitResult::Skip
            }
        }
    }
    for (span_end, custom) in [(false, false), (false, true), (true, false)] {
        let source = if span_end {
            "<p>a<code>\n<span><br></span></code>z</p>"
        } else {
            "<p>a<code>\n<br></code>z</p>"
        };
        let options = ConversionOptions {
            visitor: Some(Arc::new(Mutex::new(OmittedBreak { span_end, custom }))),
            ..Default::default()
        };
        let result = convert(source, options).expect("observed code");
        assert_eq!(result.content.as_deref(), Some("az\n"), "{span_end} {custom}");
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_ignore_direct_breaks_erased_by_the_generic_end_callback() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ReplacedBreak(bool);
    impl HtmlVisitor for ReplacedBreak {
        fn visit_element_end(&mut self, context: &NodeContext<'_>, _: &str) -> VisitResult {
            if context.tag_name != "br" {
                VisitResult::Continue
            } else if self.0 {
                VisitResult::Custom("\n".to_owned())
            } else {
                VisitResult::Skip
            }
        }
    }
    for custom in [false, true] {
        let options = ConversionOptions {
            visitor: Some(Arc::new(Mutex::new(ReplacedBreak(custom)))),
            ..Default::default()
        };
        let result = convert("<p>a<code>\n<br></code>z</p>", options).expect("observed code");
        assert_eq!(result.content.as_deref(), Some("az\n"), "custom={custom}");
    }
}
