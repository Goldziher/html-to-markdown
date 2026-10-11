#![allow(missing_docs, clippy::print_stdout)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::{ConversionOptions, TierStrategy, convert};

#[test]
fn omitted_head_end_should_preserve_body_without_repeating_title() {
    let inputs = [
        "<html><head><meta charset=utf-8><title>t</title><h1>Shown</h1>",
        "<html><head><title>t</title><h1>Shown</h1>",
        "<head><meta charset=utf-8><title>t</title><h1>Shown</h1>",
        "<html><head><meta charset=utf-8><title>t</title><h1>Shown</h1></html>",
        "<html><head><meta charset=utf-8><title>t</title></head><h1>Shown</h1>",
        "<html><head><meta charset=utf-8><title>t</title><body><h1>Shown</h1>",
        "<!doctype html><html><head><meta charset=utf-8><title>t</title><h1>Shown</h1><p>Text</p>",
        "<html><head><meta charset=utf-8><h1>Shown</h1><p>Text</p>",
    ];
    for html in inputs {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                },
            )
            .expect("document converts");
            let content = result.content.expect("Markdown exists");
            assert!(content.contains("# Shown\n"), "{tier:?}: {html}: {content:?}");
            assert!(!content.lines().any(|line| line == "t"), "{tier:?}: {content:?}");
            if html.contains("<title>") {
                assert!(content.contains("title: t\n"), "{tier:?}: {content:?}");
            }
            if html.contains("<p>Text") {
                assert!(content.contains("Text\n"), "{tier:?}: {content:?}");
            }
        }
    }
}

#[test]
fn parent_close_before_paragraph_close_should_keep_browser_boundaries() {
    let cases = [
        (
            "<!doctype html><div><p>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<!doctype html><div><p>one</div><div>two</p></div>tail",
            "one\n\ntwo\n\ntail\n",
        ),
        ("<!doctype html><div><p>one</div>two</p>three", "one\n\ntwo\n\nthree\n"),
    ];
    for whitespace in ["\n", "\t", "\r", "\u{000c}", "/"] {
        let html = format!("<div><p{whitespace}class=x>one</div>two</p>rest</div>tail");
        let result = convert(&html, ConversionOptions::default()).expect("document converts");
        assert_eq!(result.content.as_deref(), Some("one\n\ntwo\n\nresttail\n"), "{html}");
    }
    for (html, expected) in cases {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let result = convert(
                html,
                ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                },
            )
            .expect("document converts");
            assert_eq!(result.content.as_deref(), Some(expected), "{tier:?}: {html}");
        }
    }
}

/// The output of the full converter, and the output of the fast scanner called with no fallback.
/// A page that the fast scanner hands over is printed with the reason and gives `None`.
fn both_converters(html: &str, options: &ConversionOptions) -> (Option<String>, Option<String>) {
    let full = ConversionOptions {
        tier_strategy: TierStrategy::Tier2,
        ..options.clone()
    };
    let full = convert(html, full).expect("document converts").content;
    let report = html_to_markdown_rs::prescan::PrescanReport::default();
    let fast = match html_to_markdown_rs::tier1::run(html, &report, options) {
        Ok(markdown) => Some(markdown),
        Err(reason) => {
            println!("BAIL fast scanner hands over {html:?}: {reason}");
            None
        }
    };
    (full, fast)
}

/// Each case on the full converter, and on the fast scanner when it does not hand the page over.
fn assert_on_both_converters(cases: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (html, expected) in cases {
        let (full, fast) = both_converters(html, &ConversionOptions::default());
        if full.as_deref() != Some(*expected) {
            failures.push(format!("full converter: {html:?}: {full:?}, expected {expected:?}"));
        }
        if fast.as_deref().is_some_and(|fast| fast != *expected) {
            failures.push(format!("fast scanner: {html:?}: {fast:?}, expected {expected:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn discarded_end_tag_should_keep_browser_boundaries_on_both_converters() {
    assert_on_both_converters(&[
        ("<div><p>one</div>two</p>rest</div>tail", "one\n\ntwo\n\nresttail\n"),
        ("<div><p>one</div><div>two</p></div>tail", "one\n\ntwo\n\ntail\n"),
        ("<div><p>one</div>two</p>three", "one\n\ntwo\n\nthree\n"),
        ("<blockquote><p>one</blockquote>after", "> one\n\nafter\n"),
        (
            "<div><p\nclass=x>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<div><p\tclass=x>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<div><p\rclass=x>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<div><p\u{000c}class=x>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
        (
            "<div><p/class=x>one</div>two</p>rest</div>tail",
            "one\n\ntwo\n\nresttail\n",
        ),
    ]);
}

#[test]
fn end_tag_text_in_a_comment_raw_text_or_preformatted_text_should_end_no_element() {
    assert_on_both_converters(&[
        (
            "<div><p>one<!-- </div> -->two</p>rest</div>tail",
            "onetwo\n\nrest\n\ntail\n",
        ),
        (
            "<div><p>one</p><script>var a = '</div>';</script><p>two</p></div>tail",
            "one\n\ntwo\n\ntail\n",
        ),
        (
            "<div><p>one</p><style>p::after { content: '</div>'; }</style><p>two</p></div>tail",
            "one\n\ntwo\n\ntail\n",
        ),
    ]);
    let html = "<div><pre>one &lt;/div&gt; two</pre><p>three</p></div>tail";
    let (full, fast) = both_converters(html, &ConversionOptions::default());
    for (converter, content) in [("full converter", full), ("fast scanner", fast)] {
        if let Some(content) = content {
            assert!(content.contains("one </div> two"), "{converter}: {content:?}");
            assert!(content.ends_with("three\n\ntail\n"), "{converter}: {content:?}");
        }
    }
}

#[test]
fn omitted_head_end_should_keep_plain_text_on_both_converters() {
    for html in ["<html><head><title>t</title>Shown", "<head><title>t</title>Shown"] {
        let (full, fast) = both_converters(html, &ConversionOptions::default());
        for (converter, content) in [("full converter", full), ("fast scanner", fast)] {
            if let Some(content) = content {
                assert!(content.contains("Shown"), "{converter}: {html}: {content:?}");
            }
        }
    }
}

#[test]
fn shallow_tail_after_a_closed_deep_subtree_should_stay_on_both_converters() {
    let html = format!("{}deep{}<p>tail</p>", "<div>".repeat(600), "</div>".repeat(600));
    let options = ConversionOptions {
        max_depth: Some(1000),
        ..Default::default()
    };
    // The walk of a page this deep needs more stack than a test thread has, so the test has a
    // thread of its own, as the tests of `deep_nesting_overflow.rs` do.
    let (full, fast) = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || both_converters(&html, &options))
        .expect("thread starts")
        .join()
        .expect("deep input converts");
    for (converter, content) in [("full converter", full), ("fast scanner", fast)] {
        if let Some(content) = content {
            assert!(content.contains("deep"), "{converter}: {content:?}");
            assert!(content.contains("tail"), "{converter}: {content:?}");
        }
    }
}

#[test]
fn omitted_paragraph_end_should_convert_as_the_written_end_does() {
    let pairs = [
        (
            "<ul><li><p>one</li><li>two</li></ul>after",
            "<ul><li><p>one</p></li><li>two</li></ul>after",
        ),
        (
            "<div><p>one</div><p>two</p>after",
            "<div><p>one</p></div><p>two</p>after",
        ),
        (
            "<blockquote><p>one</blockquote>after",
            "<blockquote><p>one</p></blockquote>after",
        ),
    ];
    let mut differences = Vec::new();
    for (omitted, written) in pairs {
        for tier in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            let content = |html: &str| {
                let options = ConversionOptions {
                    tier_strategy: tier,
                    ..Default::default()
                };
                convert(html, options).expect("document converts").content
            };
            let (left, right) = (content(omitted), content(written));
            if left != right {
                differences.push(format!("{tier:?}: {omitted}: {left:?} != {right:?}"));
            }
        }
    }
    assert!(differences.is_empty(), "{differences:#?}");
}
