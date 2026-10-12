//! Head boundaries for self-closing raw-text elements (issue #878). ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, OutputFormat, TierStrategy, convert};

fn options_matrix() -> Vec<ConversionOptions> {
    let mut options = Vec::new();
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain] {
            for extract_metadata in [false, true] {
                options.push(ConversionOptions {
                    tier_strategy,
                    output_format,
                    extract_metadata,
                    ..ConversionOptions::default()
                });
            }
        }
    }
    options
}

#[test]
fn should_keep_self_closing_head_title_in_metadata_without_body_content() {
    for options in options_matrix() {
        #[cfg(feature = "metadata")]
        let wants_metadata = options.extract_metadata;
        let metadata = options.extract_metadata && options.output_format == OutputFormat::Markdown;
        let expected = if metadata {
            "---\ntitle: s</head><body><p>after</p></body></html>\n---\n"
        } else {
            ""
        };
        let result = convert("<html><head><title/>s</head><body><p>after</p></body></html>", options)
            .expect("convert head title");
        assert_eq!(result.content.as_deref(), Some(expected));
        #[cfg(feature = "metadata")]
        assert_eq!(
            result.metadata.document.title.as_deref(),
            wants_metadata.then_some("s</head><body><p>after</p></body></html>")
        );
    }
}

#[test]
fn should_contain_preserved_iframe_body_and_drop_inert_head_elements() {
    for tag in ["title", "iframe", "noscript", "template"] {
        for mut options in options_matrix() {
            options.preserve_tags = vec![tag.to_owned()];
            let route = format!(
                "{:?} {:?} metadata={}",
                options.tier_strategy, options.output_format, options.extract_metadata
            );
            // ~keep A browser moves iframe to the body; preservation keeps its container and raw fallback text.
            let expected = if tag == "iframe" && options.output_format == OutputFormat::Markdown {
                "<iframe>s</head><body><p>after</p></body></html></iframe>\n"
            } else {
                ""
            };
            let result = convert(
                &format!("<html><head><{tag}/>s</head><body><p>after</p></body></html>"),
                options,
            )
            .expect("convert inert head");
            assert_eq!(result.content.as_deref(), Some(expected), "{tag} {route}");
        }
    }
}

#[test]
fn should_keep_browser_visible_xmp_text_after_an_implicit_head_close() {
    for mut options in options_matrix() {
        options.extract_metadata = false;
        let result =
            convert("<html><head><xmp/>s</head><body><p>after</p></body></html>", options).expect("convert xmp");
        assert_eq!(
            result.content.as_deref(),
            Some("s</head><body><p>after</p></body></html>\n")
        );
    }
}

#[cfg(feature = "visitor")]
#[test]
fn should_not_leak_self_closing_head_bodies_with_a_continue_visitor() {
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitorHandle};
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    struct ContinueVisitor;
    impl HtmlVisitor for ContinueVisitor {}
    for tag in ["title", "iframe", "noscript", "template"] {
        for mut options in options_matrix() {
            options.extract_metadata = false;
            let visitor: VisitorHandle = Arc::new(Mutex::new(ContinueVisitor));
            options.visitor = Some(visitor);
            let result = convert(&format!("<head><{tag}/></head>C"), options).expect("convert head with visitor");
            assert_eq!(result.content.as_deref(), Some(""), "{tag}");
        }
    }
}

#[test]
fn should_keep_closed_title_metadata_and_visible_tail() {
    for mut options in options_matrix() {
        options.extract_metadata = false;
        let html = concat!(
            "<!doctype html><html xmlns='http://www.w3.org/1999/xhtml'>",
            "<head><title/>s</title></head><body><p>after</p></body></html>"
        );
        let result = convert(html, options).expect("convert closed title");
        assert_eq!(result.content.as_deref(), Some("after\n"));
    }
}

#[test]
fn should_keep_unclosed_xmp_literal_text_from_the_head() {
    for mut options in options_matrix() {
        options.extract_metadata = false;
        let result = convert(
            "<html><head><xmp>a <b>x</b> y</head><body><p>after</p></body></html>",
            options,
        )
        .expect("convert open xmp");
        assert_eq!(
            result.content.as_deref(),
            Some("a <b>x</b> y</head><body><p>after</p></body></html>\n")
        );
    }
}

#[test]
fn should_drop_the_smallest_unclosed_head_title_without_losing_metadata() {
    for options in options_matrix() {
        let expected = if options.extract_metadata && options.output_format == OutputFormat::Markdown {
            "---\ntitle: </head>C\n---\n"
        } else {
            ""
        };
        let result = convert("<head><title/></head>C", options).expect("convert minimal title");
        assert_eq!(result.content.as_deref(), Some(expected));
    }
}

#[test]
fn should_keep_self_closing_foreign_title_boundaries() {
    for mut options in options_matrix() {
        options.extract_metadata = false;
        options.output_format = OutputFormat::Markdown;
        let expected = "![](data:image/svg+xml;base64,PHN2Zz48dGl0bGUgLz48L3N2Zz4=)\n\nafter\n";
        let result = convert("<svg><title/></svg><p>after</p>", options).expect("convert foreign title");
        assert_eq!(result.content.as_deref(), Some(expected));
    }
}

#[test]
fn should_match_body_iframe_when_the_browser_closes_the_head_implicitly() {
    let head = "<html><head><iframe/>s</head><body><p>after</p></body></html>";
    let body = "<body><iframe/>s</head><body><p>after</p></body></html>";
    for options in options_matrix() {
        for preserve in [false, true] {
            let mut options = options.clone();
            if preserve {
                options.preserve_tags = vec!["iframe".to_owned()];
            }
            let route = format!(
                "{:?} {:?} metadata={} preserve={preserve}",
                options.tier_strategy, options.output_format, options.extract_metadata
            );
            let preserved_markdown = preserve && options.output_format == OutputFormat::Markdown;
            let from_head = convert(head, options.clone()).expect("convert head-origin iframe");
            let from_body = convert(body, options).expect("convert body iframe");
            assert_eq!(from_head.content, from_body.content, "{route}");
            let expected = if preserved_markdown {
                "<iframe>s</head><body><p>after</p></body></html></iframe>\n"
            } else {
                ""
            };
            assert_eq!(from_head.content.as_deref(), Some(expected), "{route}");
        }
    }
}
