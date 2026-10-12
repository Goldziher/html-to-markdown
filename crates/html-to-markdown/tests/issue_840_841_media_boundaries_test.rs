#![allow(missing_docs)]

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, LinkStyle, TierStrategy, convert};

#[cfg(feature = "testkit")]
const STRATEGIES: [TierStrategy; 3] = [TierStrategy::Auto, TierStrategy::Tier2, TierStrategy::Tier1];
#[cfg(not(feature = "testkit"))]
const STRATEGIES: [TierStrategy; 2] = [TierStrategy::Auto, TierStrategy::Tier2];

fn check_routes(html: &str, expected: &str, code_block_style: CodeBlockStyle) {
    for tier_strategy in STRATEGIES {
        for extract_metadata in [false, true] {
            let options = ConversionOptions {
                tier_strategy,
                extract_metadata,
                code_block_style,
                ..ConversionOptions::default()
            };
            let output = convert(html, Some(options))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            assert_eq!(
                output, expected,
                "{tier_strategy:?}, metadata={extract_metadata}: {html}"
            );
        }
    }
}

fn media_forms(tag: &str) -> Vec<(String, String)> {
    vec![
        (format!("<{tag} src=/media></{tag}>"), tag.to_ascii_lowercase()),
        (
            format!("<{tag}><source src=/media><source src=/other></{tag}>"),
            tag.to_ascii_lowercase(),
        ),
        (
            format!("<{tag} src=/media title='Clip &amp; sound'></{tag}>"),
            "Clip & sound".to_string(),
        ),
        (format!("<{tag} src=/media title=''></{tag}>"), tag.to_ascii_lowercase()),
        (
            format!("<{tag} src=/media title='  '></{tag}>"),
            tag.to_ascii_lowercase(),
        ),
    ]
}

#[test]
fn should_write_a_plain_media_label_inside_an_outer_link() {
    for tag in ["video", "audio", "VIDEO", "Audio"] {
        for (media, label) in media_forms(tag) {
            for wrapper in ["", "span"] {
                let body = if wrapper.is_empty() {
                    media.clone()
                } else {
                    format!("<{wrapper}>{media}</{wrapper}>")
                };
                let html = format!("<a href=/x>{body}</a>");
                check_routes(&html, &format!("[{label}](/x)\n"), CodeBlockStyle::Backticks);
            }
        }
    }
}

#[test]
fn should_not_write_media_or_player_spacing_inside_a_code_block() {
    for tag in ["video", "audio", "VIDEO", "Audio"] {
        let mut forms = media_forms(tag);
        forms.push((format!("<{tag}></{tag}>"), String::new()));
        forms.push((format!("<{tag} src=/media>fallback text</{tag}>"), String::new()));
        for (media, _) in forms {
            for body in [
                media.clone(),
                format!("<span>{media}</span>"),
                format!("<code>{media}</code>"),
                format!("<a href=/x>{media}</a>"),
            ] {
                let html = format!("<pre>a {body} b</pre>");
                check_routes(&html, "```\na  b\n```\n", CodeBlockStyle::Backticks);
                check_routes(&html, "    a  b\n", CodeBlockStyle::Indented);
            }
        }
    }
}

#[test]
fn should_keep_standalone_media_labels_titles_and_first_source_selection() {
    for tag in ["video", "audio", "VIDEO", "Audio"] {
        for (media, label) in media_forms(tag) {
            check_routes(&media, &format!("[{label}](/media)\n"), CodeBlockStyle::Backticks);
        }
    }
    check_routes(
        "<iframe src=/frame title=Map></iframe>",
        "[Map](/frame)\n",
        CodeBlockStyle::Backticks,
    );
}

#[test]
fn should_keep_media_rendering_in_inline_code() {
    for tag in ["video", "audio"] {
        let html = format!("<p><code>a <{tag} src=/media></{tag}> b</code></p>");
        check_routes(&html, &format!("`a [{tag}](/media) b`\n"), CodeBlockStyle::Backticks);
    }
}

#[test]
fn should_apply_outer_link_escaping_to_media_titles() {
    for tag in ["video", "audio"] {
        for (title, expected) in [
            ("[Clip]", "[[Clip]](/x)\n"),
            ("Clip]", "[Clip\\]](/x)\n"),
            ("[Clip](/other)", "[\\[Clip\\](/other)](/x)\n"),
        ] {
            let html = format!("<a href=/x><{tag} src=/media title='{title}'></{tag}></a>");
            check_routes(&html, expected, CodeBlockStyle::Backticks);
            let markdown = convert(&html, None)
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            assert_eq!(
                comrak::markdown_to_html(&markdown, &comrak::Options::default()),
                format!("<p><a href=\"/x\">{title}</a></p>\n"),
                "the title stays literal inside one outer link: {html}"
            );
        }
    }
}

#[test]
fn should_collect_only_the_outer_link_address_for_reference_links() {
    for tag in ["video", "audio"] {
        for html in [
            format!("<a href=/x><{tag} src=/media></{tag}></a>"),
            format!("<pre>a <{tag} src=/media></{tag}> b</pre>"),
        ] {
            let options = ConversionOptions {
                link_style: LinkStyle::Reference,
                extract_metadata: false,
                ..ConversionOptions::default()
            };
            let output = convert(&html, Some(options))
                .expect("conversion must succeed")
                .content
                .unwrap_or_default();
            let expected = if html.starts_with("<pre>") {
                "```\na  b\n```\n".to_string()
            } else {
                format!("[{tag}][1]\n\n[1]: /x\n")
            };
            assert_eq!(output, expected, "{html}");
        }
    }
}

#[cfg(feature = "visitor")]
mod visitor {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Recorder {
        calls: Vec<String>,
        custom: bool,
    }

    impl Recorder {
        fn media(&mut self, tag: &str, src: Option<&str>) -> VisitResult {
            self.calls.push(format!("{tag} {}", src.unwrap_or_default()));
            if self.custom {
                VisitResult::Custom("custom\n".to_string())
            } else {
                VisitResult::Continue
            }
        }
    }

    impl HtmlVisitor for Recorder {
        fn visit_video(&mut self, _ctx: &NodeContext<'_>, src: Option<&str>) -> VisitResult {
            self.media("video", src)
        }

        fn visit_audio(&mut self, _ctx: &NodeContext<'_>, src: Option<&str>) -> VisitResult {
            self.media("audio", src)
        }

        fn visit_link(&mut self, _ctx: &NodeContext<'_>, _href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            self.calls.push(format!("link {text}"));
            VisitResult::Continue
        }

        fn visit_code_block(&mut self, _ctx: &NodeContext<'_>, _language: Option<&str>, code: &str) -> VisitResult {
            self.calls.push(format!("code {code}"));
            VisitResult::Continue
        }
    }

    fn record(html: &str, custom: bool) -> (String, Vec<String>) {
        let recorder = Arc::new(Mutex::new(Recorder {
            custom,
            ..Recorder::default()
        }));
        let visitor: VisitorHandle = recorder.clone();
        let options = ConversionOptions {
            visitor: Some(visitor),
            extract_metadata: false,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .expect("conversion must succeed")
            .content
            .unwrap_or_default();
        let calls = std::mem::take(&mut recorder.lock().expect("recorder mutex").calls);
        (output, calls)
    }

    #[test]
    fn should_keep_media_callbacks_and_give_the_link_its_plain_media_label() {
        for tag in ["video", "audio"] {
            let html = format!("<a href=/x><span><{tag} src=/media title=Clip></{tag}></span></a>");
            let (output, calls) = record(&html, false);
            assert_eq!(output, "[Clip](/x)\n");
            assert_eq!(calls, [format!("{tag} /media"), "link Clip".to_string()]);
        }
    }

    #[test]
    fn should_keep_media_callbacks_when_default_code_output_is_suppressed() {
        for tag in ["video", "audio"] {
            let html = format!("<pre>a <span><{tag}><source src=/media></{tag}></span> b</pre>");
            let (output, calls) = record(&html, false);
            assert_eq!(output, "```\na  b\n```\n");
            assert_eq!(calls, [format!("{tag} /media"), "code a  b".to_string()]);
        }
    }

    #[test]
    fn should_keep_custom_media_output_in_code() {
        for tag in ["video", "audio"] {
            let html = format!("<pre>a <{tag} src=/media></{tag}> b</pre>");
            let (output, calls) = record(&html, true);
            assert_eq!(output, "```\na custom\n b\n```\n");
            assert_eq!(calls, [format!("{tag} /media"), "code a custom\n b".to_string()]);
        }
    }
}
