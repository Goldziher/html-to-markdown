#![allow(missing_docs)]
use html_to_markdown_rs::{ConversionOptions, HighlightStyle, TierStrategy, convert};

#[test]
fn should_keep_stray_table_cell_text_on_every_conversion_path() {
    for (html, expected) in [
        ("<div><td>One</td>items</div>", "Oneitems\n"),
        ("<td>One</td>items", "Oneitems\n"),
        ("<div><th>One</th>items</div>", "Oneitems\n"),
        ("<p>before <td>One</td> after</p>", "before One after\n"),
        ("<div><span>One</span>items</div>", "Oneitems\n"),
        ("<div><td><em>One</em> &amp; two</td>items</div>", "*One* & twoitems\n"),
    ] {
        for options in [
            ConversionOptions::default(),
            ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                ..ConversionOptions::default()
            },
            ConversionOptions {
                extract_metadata: false,
                highlight_style: HighlightStyle::None,
                ..ConversionOptions::default()
            },
        ] {
            let output = convert(html, Some(options))
                .expect("conversion succeeds")
                .content
                .unwrap_or_default();
            assert_eq!(output, expected, "{html}");
        }
    }
}

#[cfg(feature = "visitor")]
mod table_part_visitors {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct TablePartVisitor {
        events: Arc<Mutex<Vec<(bool, usize)>>>,
    }

    impl HtmlVisitor for TablePartVisitor {
        fn visit_element_start(&mut self, ctx: &NodeContext<'_>) -> VisitResult {
            if ctx.tag_name == "td" {
                self.events.lock().expect("events lock").push((true, ctx.depth));
                match ctx.attributes().get("id").map(String::as_str) {
                    Some("skip") => return VisitResult::Skip,
                    Some("custom") => return VisitResult::Custom("replacement".to_owned()),
                    _ => {}
                }
            }
            VisitResult::Continue
        }

        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, _output: &str) -> VisitResult {
            if ctx.tag_name == "td" {
                self.events.lock().expect("events lock").push((false, ctx.depth));
            }
            VisitResult::Continue
        }
    }

    fn convert_with_events(html: &str, max_depth: usize) -> (String, Vec<(bool, usize)>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let visitor: VisitorHandle = Arc::new(Mutex::new(TablePartVisitor { events: events.clone() }));
        let result = convert(
            html,
            Some(ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                max_depth: Some(max_depth),
                visitor: Some(visitor),
                include_document_structure: true,
                ..ConversionOptions::default()
            }),
        )
        .expect("conversion succeeds");
        let recorded = events.lock().expect("events lock").clone();
        (result.content.unwrap_or_default(), recorded)
    }

    #[test]
    fn should_keep_table_part_visitor_order_and_authored_depth() {
        let (output, events) = convert_with_events("<div><td>a<td><em>b</em></td>c</td>d</div>", 20);
        assert_eq!(output, "a*b*cd\n");
        assert_eq!(events, [(true, 1), (true, 2), (false, 2), (false, 1)]);
    }

    #[test]
    fn should_apply_table_part_visitor_skip_and_custom_output() {
        let (output, events) = convert_with_events(
            "<div><td>a<td id=skip>discard</td><td id=custom>discard</td>z</td></div>",
            20,
        );
        assert_eq!(output, "areplacementz\n");
        assert_eq!(events, [(true, 1), (true, 2), (true, 2), (false, 2), (false, 1)]);
    }

    #[test]
    fn should_stop_table_part_callbacks_at_the_authored_depth_limit() {
        let (output, events) = convert_with_events("<div><td>a<td>b<td>c</td></td></td></div>", 3);
        assert_eq!(output, "a\n");
        assert_eq!(events, [(true, 1), (true, 2), (false, 2), (false, 1)]);
    }
}
