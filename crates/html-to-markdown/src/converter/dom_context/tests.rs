#[test]
fn should_keep_ordinary_negative_break_memo_when_no_owner_was_omitted() {
    let source = "<b><span> </span></b>";
    let dom = tl::parse(source, tl::ParserOptions::default()).expect("blank nodes");
    let context = crate::converter::utility::caching::build_dom_context(&dom, dom.parser(), source.len());
    let root = dom.children()[0].get_inner();
    let child = context.children_of(root).expect("one span")[0].get_inner();
    super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
    assert!(!context.holds_authored_break(root, dom.parser()));
    super::HOLDS_TEXT_NODE_READS.with(|reads| assert_eq!(reads.get(), 2));
    context.record_absent_breaks(child, false);
    super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
    assert!(!context.holds_authored_break(root, dom.parser()));
    super::HOLDS_TEXT_NODE_READS.with(|reads| assert_eq!(reads.get(), 0));
}

#[cfg(feature = "visitor")]
#[test]
fn should_refresh_cached_break_ancestors_without_forgetting_siblings() {
    let source = "<b><span><br></span><i><br></i></b>";
    for order in [[0, 1], [1, 0]] {
        let dom = tl::parse(source, tl::ParserOptions::default()).expect("break siblings");
        let parser = dom.parser();
        let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
        let root = dom.children()[0].get_inner();
        let breaks = dom
            .query_selector("br")
            .expect("break selector")
            .map(|handle| handle.get_inner())
            .collect::<Vec<_>>();
        assert_eq!(breaks.len(), 2);
        assert!(context.holds_authored_break(root, parser));
        context.record_absent_breaks(breaks[order[0]], true);
        assert!(context.holds_authored_break(root, parser));
        context.record_absent_breaks(breaks[order[1]], true);
        assert!(!context.holds_authored_break(root, parser));
        context.record_absent_breaks(breaks[order[0]], false);
        assert!(context.holds_authored_break(root, parser));
        context.record_absent_breaks(breaks[order[0]], true);
        assert!(!context.holds_authored_break(root, parser));
    }
}

#[test]
fn should_invalidate_omitted_subtree_ancestors_and_keep_retained_siblings() {
    let source = "<b><span><br></span><i><br></i></b>";
    for order in [[0, 1], [1, 0]] {
        let dom = tl::parse(source, tl::ParserOptions::default()).expect("break subtrees");
        let parser = dom.parser();
        let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
        let root = dom.children()[0].get_inner();
        let children = context.children_of(root).expect("two subtrees");
        assert_eq!(children.len(), 2);
        assert!(context.holds_authored_break(root, parser));
        context.record_absent_breaks(children[order[0]].get_inner(), true);
        assert!(context.holds_authored_break(root, parser));
        context.record_absent_breaks(children[order[1]].get_inner(), true);
        assert!(!context.holds_authored_break(root, parser));
        context.record_absent_breaks(children[order[0]].get_inner(), false);
        assert!(context.holds_authored_break(root, parser));
    }
}

#[test]
fn should_bound_observed_font_registration_and_boundary_reads() {
    let mut jobs = 0;
    for overlapping in [false, true] {
        for depth in [4, 16, 64, 256] {
            let closing = if overlapping {
                "</font>&nbsp;".repeat(depth)
            } else {
                "</font>".repeat(depth) + "&nbsp;"
            };
            let source = format!("<p>a{} {closing}b</p>", "<font>".repeat(depth));
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("font nodes");
            let nodes = dom.nodes().len();
            super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
            super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
            let result = crate::convert(
                &source,
                crate::ConversionOptions {
                    tier_strategy: crate::TierStrategy::Tier2,
                    extract_metadata: false,
                    max_depth: Some(1024),
                    ..Default::default()
                },
            )
            .expect("font nesting");
            let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
            let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
            assert_eq!(result.content.as_deref(), Some("a b\n"));
            assert_eq!(byte_reads, depth);
            assert_eq!(node_reads, if overlapping { depth + 1 } else { 2 * depth });
            assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
            assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0, depth);
            assert!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1 <= 2 * depth);
            assert!(
                byte_reads > 0 && byte_reads <= 4 * nodes,
                "font registration {byte_reads}/{nodes}"
            );
            assert!(
                node_reads > 0 && node_reads <= 4 * nodes,
                "font boundary {node_reads}/{nodes}"
            );
            jobs += 1;
        }
    }
    assert_eq!(jobs, 8);
}
#[cfg(feature = "visitor")]
#[test]
fn should_bound_font_registration_with_growing_custom_child_whitespace() {
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct BlankChild {
        width: usize,
        calls: usize,
    }
    impl crate::visitor::HtmlVisitor for BlankChild {
        fn visit_element_start(&mut self, ctx: &crate::visitor::NodeContext) -> crate::visitor::VisitResult {
            if ctx.tag_name == "span" {
                self.calls += 1;
                crate::visitor::VisitResult::Custom(" ".repeat(self.width))
            } else {
                crate::visitor::VisitResult::Continue
            }
        }
    }
    let mut jobs = 0;
    for depth in [4, 16, 64, 256] {
        let source = format!(
            "<p>a{}<span></span>{}&nbsp;b</p>",
            "<font>".repeat(depth),
            "</font>".repeat(depth)
        );
        let dom = tl::parse(&source, tl::ParserOptions::default()).expect("custom font nodes");
        let nodes = dom.nodes().len();
        let visitor = Arc::new(Mutex::new(BlankChild { width: depth, calls: 0 }));
        super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
        super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
        super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
        let result = crate::convert(
            &source,
            crate::ConversionOptions {
                visitor: Some(visitor.clone()),
                tier_strategy: crate::TierStrategy::Tier2,
                extract_metadata: false,
                max_depth: Some(1024),
                ..Default::default()
            },
        )
        .expect("custom child whitespace");
        let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
        let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
        assert_eq!(
            result.content.as_deref(),
            Some(format!("a{}b\n", " ".repeat(depth + 1)).as_str())
        );
        assert_eq!(visitor.lock().expect("visitor calls").calls, 1);
        assert_eq!(byte_reads, 0);
        assert_eq!(node_reads, depth);
        assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get), (0, 0));
        assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
        assert!(node_reads > 0 && node_reads <= 4 * nodes);
        jobs += 1;
    }
    assert_eq!(jobs, 4);
}
#[test]
#[allow(clippy::print_stdout)]
fn should_bound_observed_unknown_registration_and_boundary_reads() {
    let mut jobs = 0;
    for overlapping in [false, true] {
        for depth in [4, 16, 64, 256] {
            let closing = if overlapping {
                "</x-y>&nbsp;".repeat(depth)
            } else {
                "</x-y>".repeat(depth) + "&nbsp;"
            };
            let source = format!("<p>a{} {closing}b</p>", "<x-y>".repeat(depth));
            let dom = tl::parse(&source, tl::ParserOptions::default()).expect("unknown nodes");
            let nodes = dom.nodes().len();
            assert_eq!(nodes, if overlapping { 2 * depth + 3 } else { depth + 4 });
            super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
            super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
            super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
            let result = crate::convert(
                &source,
                crate::ConversionOptions {
                    tier_strategy: crate::TierStrategy::Tier2,
                    extract_metadata: false,
                    max_depth: Some(1024),
                    ..Default::default()
                },
            )
            .expect("unknown nesting");
            let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
            let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
            assert_eq!(result.content.as_deref(), Some("a b\n"));
            assert_eq!(byte_reads, depth);
            assert_eq!(node_reads, if overlapping { depth + 1 } else { 2 * depth });
            assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
            assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0, depth);
            assert!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1 <= 2 * depth);
            assert!(
                byte_reads > 0 && byte_reads <= 4 * nodes,
                "unknown registration {byte_reads}/{nodes}"
            );
            assert!(
                node_reads > 0 && node_reads <= 4 * nodes,
                "unknown boundary {node_reads}/{nodes}"
            );
            std::println!(
                "unknown-cost overlap={overlapping} depth={depth} nodes={nodes} bytes={byte_reads} reads={node_reads} calls={} entries={} capacity={}",
                super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get),
                super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0,
                super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1
            );
            jobs += 1;
        }
    }
    assert_eq!(jobs, 8);
}
#[cfg(feature = "visitor")]
#[test]
#[allow(clippy::print_stdout)]
fn should_bound_unknown_registration_with_growing_custom_child_whitespace() {
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct BlankChild {
        width: usize,
        calls: usize,
    }
    impl crate::visitor::HtmlVisitor for BlankChild {
        fn visit_element_start(&mut self, ctx: &crate::visitor::NodeContext) -> crate::visitor::VisitResult {
            if ctx.tag_name == "span" {
                self.calls += 1;
                crate::visitor::VisitResult::Custom(" ".repeat(self.width))
            } else {
                crate::visitor::VisitResult::Continue
            }
        }
    }
    let mut jobs = 0;
    for depth in [4, 16, 64, 256] {
        let source = format!(
            "<p>a{}<span></span>{}&nbsp;b</p>",
            "<x-y>".repeat(depth),
            "</x-y>".repeat(depth)
        );
        let dom = tl::parse(&source, tl::ParserOptions::default()).expect("custom unknown nodes");
        let nodes = dom.nodes().len();
        assert_eq!(nodes, depth + 4);
        let visitor = Arc::new(Mutex::new(BlankChild { width: depth, calls: 0 }));
        super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
        super::FONT_RENDER_BYTE_READS.with(|reads| reads.set(0));
        super::FONT_RENDER_GUARD_CALLS.with(|calls| calls.set(0));
        super::FONT_RENDER_SET_STATS.with(|stats| stats.set((0, 0)));
        let result = crate::convert(
            &source,
            crate::ConversionOptions {
                visitor: Some(visitor.clone()),
                tier_strategy: crate::TierStrategy::Tier2,
                extract_metadata: false,
                max_depth: Some(1024),
                ..Default::default()
            },
        )
        .expect("custom child whitespace");
        let node_reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
        let byte_reads = super::FONT_RENDER_BYTE_READS.with(std::cell::Cell::get);
        assert_eq!(
            result.content.as_deref(),
            Some(format!("a{}b\n", " ".repeat(depth + 1)).as_str())
        );
        assert_eq!(visitor.lock().expect("visitor calls").calls, 1);
        assert_eq!(byte_reads, 0);
        assert_eq!(node_reads, depth);
        assert_eq!(super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get), (0, 0));
        assert_eq!(super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get), depth);
        assert!(node_reads > 0 && node_reads <= 4 * nodes);
        std::println!(
            "unknown-cost custom depth={depth} nodes={nodes} bytes={byte_reads} reads={node_reads} calls={} entries={} capacity={}",
            super::FONT_RENDER_GUARD_CALLS.with(std::cell::Cell::get),
            super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).0,
            super::FONT_RENDER_SET_STATS.with(std::cell::Cell::get).1
        );
        jobs += 1;
    }
    assert_eq!(jobs, 4);
}
#[test]
fn should_classify_visible_void_descendants_below_inline_wrappers() {
    for (body, expected) in [
        ("<em><wbr></em>", false),
        ("<em><input></em>", false),
        ("<b><br></b>", true),
        ("<b><img alt='X' src='x'></b>", true),
        ("<b><hr></b>", true),
    ] {
        let source = format!("<span>{body}</span>");
        let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse descendants");
        let parser = dom.parser();
        let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
        let root = dom.children()[0].get_inner();
        assert_eq!(context.holds_content(root, parser), expected, "{source}");
    }
}
#[test]
fn should_keep_literal_preformatted_content_when_inline_line_ends_are_empty() {
    for (body, expected) in [
        ("<code>\n</code>", false),
        ("<del>\n</del>", false),
        ("<code> </code>", true),
        ("<pre>\n</pre>", true),
        ("<pre><code>\n\n</code></pre>", true),
        ("<pre><!-- x --></pre>", false),
        ("<pre><i></i></pre>", false),
        ("<pre><span><code>\n</code></span></pre>", true),
        ("<code>&#13;&#10;</code>", true),
        ("<code><span>\n</span></code>", false),
        ("<del><span>\n</span></del>", false),
        ("<code><del><span>&#13;</span></del></code>", true),
        ("<code><del><span>&#13;&#10;</span></del></code>", true),
        ("<del><code><span>&#13;</span></code></del>", true),
        ("<del><span>&#13;</span></del>", true),
        ("<pre><code><span>\n</span></code></pre>", true),
    ] {
        let source = format!("<span>{body}</span>");
        let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse literal ownership");
        let parser = dom.parser();
        let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
        assert_eq!(
            context.holds_content(dom.children()[0].get_inner(), parser),
            expected,
            "{source}"
        );
        for (index, node) in dom.nodes().iter().enumerate() {
            if matches!(node, tl::Node::Tag(_)) {
                let id = u32::try_from(index).expect("small node arena");
                assert_eq!(context.holds_content(id, parser), expected, "rooted {source}");
            }
        }
    }
}
#[test]
fn should_preserve_deep_pre_ownership_without_changing_text_cache() {
    let depth = 2048;
    let source = format!(
        "<span><pre>{}<code>\n</code>{}</pre></span>",
        "<span>".repeat(depth),
        "</span>".repeat(depth)
    );
    let dom = tl::parse(&source, tl::ParserOptions::default()).expect("parse deep literal ownership");
    let parser = dom.parser();
    let context = crate::converter::utility::caching::build_dom_context(&dom, parser, source.len());
    super::HOLDS_TEXT_NODE_READS.with(|reads| reads.set(0));
    let root = dom.children()[0].get_inner();
    let pre = dom
        .query_selector("pre")
        .expect("pre selector")
        .next()
        .expect("pre exists")
        .get_inner();
    let code = dom
        .query_selector("code")
        .expect("code selector")
        .next()
        .expect("code exists")
        .get_inner();
    for id in [root, pre, code] {
        assert!(context.holds_content(id, parser), "literal query at {id}");
        assert!(!context.holds_text(id, parser), "text query at {id}");
    }
    let reads = super::HOLDS_TEXT_NODE_READS.with(std::cell::Cell::get);
    assert_eq!(reads, 3 * depth + 9);
}
