#![allow(missing_docs)]

use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, InlineDataMedia, LinkStyle, TierStrategy, convert};

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

#[test]
fn should_separate_media_labels_from_text_and_fallback_inside_one_link() {
    for tag in ["video", "audio"] {
        for (body, expected) in [
            (format!("<{tag} src=/media></{tag}>text"), format!("{tag} text")),
            (format!("text<{tag} src=/media></{tag}>"), format!("text {tag}")),
            (format!("<{tag} src=/media>fallback</{tag}>"), format!("{tag} fallback")),
            (
                format!("<{tag} src=/media></{tag}><{tag} src=/other></{tag}>"),
                format!("{tag} {tag}"),
            ),
            (
                format!("<span><{tag} src=/media></{tag}></span>text"),
                format!("{tag} text"),
            ),
            (format!("<{tag} src=/media></{tag}>!"), format!("{tag}!")),
        ] {
            let html = format!("<a href=/x>{body}</a>");
            check_routes(&html, &format!("[{expected}](/x)\n"), CodeBlockStyle::Backticks);
            let markdown = convert(&html, None)
                .expect("convert media link")
                .content
                .unwrap_or_default();
            assert_eq!(
                comrak::markdown_to_html(&markdown, &comrak::Options::default()),
                format!("<p><a href=\"/x\">{expected}</a></p>\n")
            );
        }
    }
}

#[test]
fn should_keep_media_destination_when_an_outer_anchor_has_no_destination() {
    for tag in ["video", "audio"] {
        for href in [" href=''", "", " href=' '"] {
            let html = format!("<a{href}><{tag} src=/media></{tag}></a>");
            let expected = if href == " href=' '" {
                format!("[{tag}](< >)\n")
            } else {
                format!("[{tag}](/media)\n")
            };
            check_routes(&html, &expected, CodeBlockStyle::Backticks);
        }
    }
}

#[test]
fn should_keep_destination_and_data_media_option_boundaries() {
    for tier_strategy in STRATEGIES {
        for tag in ["video", "audio"] {
            for (html, options, expected) in [
                (
                    format!("<a href=/x><{tag} src=/media></{tag}></a>"),
                    ConversionOptions {
                        strip_tags: vec!["a".to_string()],
                        ..ConversionOptions::default()
                    },
                    format!("[{tag}](/media)\n"),
                ),
                (
                    format!("<base href='https://example.com/doc'><a href=''><{tag} src=/media></{tag}></a>"),
                    ConversionOptions {
                        base_url: Some("https://example.com/doc".to_string()),
                        ..ConversionOptions::default()
                    },
                    format!("[{tag}](https://example.com/doc)\n"),
                ),
                (
                    format!("<a href='data:text/plain,outer'><{tag} src=/media></{tag}></a>"),
                    ConversionOptions {
                        inline_data_media: InlineDataMedia::AltTextOnly,
                        ..ConversionOptions::default()
                    },
                    format!("{tag}\n"),
                ),
                (
                    format!("<a href='/x'><{tag} src='data:video/mp4;base64,AAAA'>fallback</{tag}></a>"),
                    ConversionOptions {
                        inline_data_media: InlineDataMedia::AltTextOnly,
                        ..ConversionOptions::default()
                    },
                    "[fallback](/x)\n".to_string(),
                ),
                (
                    format!("<a href='/x'><{tag} src='data:video/mp4;base64,AAAA'></{tag}></a>"),
                    ConversionOptions {
                        inline_data_media: InlineDataMedia::DropElement,
                        ..ConversionOptions::default()
                    },
                    String::new(),
                ),
                (
                    format!("<a href=''><{tag} src=/media></{tag}></a>"),
                    ConversionOptions {
                        link_style: LinkStyle::Reference,
                        ..ConversionOptions::default()
                    },
                    format!("[{tag}][1]\n\n[1]: /media\n"),
                ),
            ] {
                let output = convert(
                    &html,
                    Some(ConversionOptions {
                        tier_strategy,
                        extract_metadata: false,
                        ..options
                    }),
                )
                .expect("convert option control")
                .content
                .unwrap_or_default();
                assert_eq!(output, expected, "{tier_strategy:?}: {html}");
            }
        }
    }
}

#[cfg(feature = "visitor")]
mod review_visitors {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Overrides {
        media_action: VisitResult,
        link_action: VisitResult,
        calls: Vec<String>,
    }

    impl HtmlVisitor for Overrides {
        fn visit_video(&mut self, _ctx: &NodeContext<'_>, src: Option<&str>) -> VisitResult {
            self.calls.push(format!("video {}", src.unwrap_or_default()));
            self.media_action.clone()
        }
        fn visit_audio(&mut self, _ctx: &NodeContext<'_>, src: Option<&str>) -> VisitResult {
            self.calls.push(format!("audio {}", src.unwrap_or_default()));
            self.media_action.clone()
        }
        fn visit_link(&mut self, _ctx: &NodeContext<'_>, href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            self.calls.push(format!("link {href} {text}"));
            self.link_action.clone()
        }
    }

    fn observe(
        html: &str,
        media_action: VisitResult,
        link_action: VisitResult,
    ) -> (Result<String, String>, Vec<String>) {
        let recorder = Arc::new(Mutex::new(Overrides {
            media_action,
            link_action,
            calls: Vec::new(),
        }));
        let visitor: VisitorHandle = recorder.clone();
        let options = ConversionOptions {
            visitor: Some(visitor),
            extract_metadata: false,
            ..ConversionOptions::default()
        };
        let output = convert(html, Some(options))
            .map(|result| result.content.unwrap_or_default())
            .map_err(|error| error.to_string());
        let calls = std::mem::take(&mut recorder.lock().expect("recorder mutex").calls);
        (output, calls)
    }

    #[test]
    fn should_leave_custom_and_skipped_media_output_under_visitor_control() {
        for tag in ["video", "audio"] {
            for (action, expected) in [
                (VisitResult::Skip, "beforeafter"),
                (VisitResult::Custom("CUSTOM".to_string()), "beforeCUSTOM after"),
            ] {
                let html = format!("<a href=/x>before<{tag} src=/media>fallback</{tag}>after</a>");
                let (output, calls) = observe(&html, action, VisitResult::Continue);
                assert_eq!(output.expect("convert visitor control"), format!("[{expected}](/x)\n"));
                assert_eq!(calls, [format!("{tag} /media"), format!("link /x {expected}")]);
            }
        }
    }

    #[test]
    fn should_keep_media_html_and_errors_under_visitor_control() {
        for tag in ["video", "audio"] {
            let html = format!("<a href=/x><{tag} src=/media>fallback</{tag}></a>");
            let (output, calls) = observe(&html, VisitResult::PreserveHtml, VisitResult::Continue);
            let preserved = format!("<{tag} src=\"/media\">fallback</{tag}>");
            assert_eq!(output.expect("preserve media"), format!("[{preserved}](/x)\n"));
            assert_eq!(calls, [format!("{tag} /media"), format!("link /x {preserved}")]);
            let (output, calls) = observe(
                &html,
                VisitResult::Error("media failed".to_string()),
                VisitResult::Continue,
            );
            assert!(
                output
                    .expect_err("visitor error must propagate")
                    .contains("media failed")
            );
            assert_eq!(calls.first(), Some(&format!("{tag} /media")));
        }
    }

    #[test]
    fn should_keep_no_address_media_overrides_and_report_the_surviving_fallback_once() {
        for tag in ["video", "audio"] {
            let html = format!("<a href=/x>before<{tag}>fallback</{tag}>after</a>");
            for (action, expected) in [
                (VisitResult::Continue, "before fallback after"),
                (VisitResult::Skip, "beforeafter"),
                (VisitResult::Custom("CUSTOM".to_string()), "beforeCUSTOM after"),
                (VisitResult::Custom(String::new()), "before after"),
            ] {
                let (output, calls) = observe(&html, action.clone(), VisitResult::Continue);
                assert_eq!(
                    output.expect("media override"),
                    format!("[{expected}](/x)\n"),
                    "{tag}, {action:?}"
                );
                assert_eq!(calls, [format!("{tag} "), format!("link /x {expected}")]);
            }
            let html = format!("<a href=/x><{tag}>fallback</{tag}></a>");
            let (output, calls) = observe(&html, VisitResult::PreserveHtml, VisitResult::Continue);
            let preserved = format!("<{tag}>fallback</{tag}>");
            assert_eq!(output.expect("preserve media"), format!("[{preserved}](/x)\n"));
            assert_eq!(calls, [format!("{tag} "), format!("link /x {preserved}")]);
            let (output, calls) = observe(
                &html,
                VisitResult::Error("media failed".to_string()),
                VisitResult::Continue,
            );
            assert!(output.expect_err("media error").contains("media failed"));
            assert_eq!(calls.first(), Some(&format!("{tag} ")));
        }
    }

    #[test]
    fn should_give_outer_link_visitors_the_separated_media_label_and_keep_overrides() {
        for tag in ["video", "audio"] {
            let html = format!("<a href=/x><{tag} src=/media></{tag}>after</a>");
            for (action, expected) in [
                (VisitResult::Skip, ""),
                (VisitResult::Custom("CUSTOM".to_string()), "CUSTOM\n"),
            ] {
                let (output, calls) = observe(&html, VisitResult::Continue, action);
                assert_eq!(output.expect("link visitor override"), expected);
                assert_eq!(calls, [format!("{tag} /media"), format!("link /x {tag} after")]);
            }
            let (output, calls) = observe(
                &html,
                VisitResult::Continue,
                VisitResult::Error("link failed".to_string()),
            );
            assert!(output.expect_err("link visitor error").contains("link failed"));
            assert_eq!(calls, [format!("{tag} /media"), format!("link /x {tag} after")]);
        }
    }
}

#[test]
fn should_not_move_synthetic_media_spacing_outside_the_anchor() {
    for tag in ["video", "audio"] {
        let html = format!("<p>a<a href=/x><{tag} src=/media> </{tag}></a>b</p>");
        check_routes(&html, &format!("a[{tag}](/x)b\n"), CodeBlockStyle::Backticks);
    }
}

#[test]
fn should_separate_media_words_through_empty_inline_neighbors_without_separating_punctuation() {
    for tag in ["video", "audio"] {
        for empty in ["<span></span>", "<!-- gap -->", "<span><b></b><!-- gap --></span>"] {
            for player in [
                format!("<{tag} src=/media></{tag}>"),
                format!("<span><{tag} src=/media></{tag}></span>"),
            ] {
                for (tail, separator) in [("text", " "), ("!", "")] {
                    let html = format!("<a href=/x>{player}{empty}{tail}</a>");
                    check_routes(
                        &html,
                        &format!("[{tag}{separator}{tail}](/x)\n"),
                        CodeBlockStyle::Backticks,
                    );
                }
            }
        }
    }
}

#[test]
fn should_keep_each_wrapped_player_as_one_word_when_players_have_no_source_text() {
    for tag in ["video", "audio"] {
        for player in [
            format!("<span><{tag} src=/media></{tag}></span>"),
            format!("<span><span><{tag} src=/media></{tag}></span></span>"),
        ] {
            let players = player.repeat(100);
            let html = format!("<a href=/x>{players}text</a>");
            let expected = format!("[{} text](/x)\n", vec![tag; 100].join(" "));
            check_routes(&html, &expected, CodeBlockStyle::Backticks);
        }
    }
}

#[cfg(feature = "visitor")]
mod rendered_neighbor_visitors {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Observer {
        target: &'static str,
        start: VisitResult,
        end: VisitResult,
        calls: Vec<String>,
    }

    impl HtmlVisitor for Observer {
        fn visit_element_start(&mut self, ctx: &NodeContext<'_>) -> VisitResult {
            if ctx.tag_name == self.target {
                self.calls.push("start".to_string());
                self.start.clone()
            } else {
                VisitResult::Continue
            }
        }
        fn visit_element_end(&mut self, ctx: &NodeContext<'_>, output: &str) -> VisitResult {
            if ctx.tag_name == self.target {
                self.calls.push(format!("end {output}"));
                self.end.clone()
            } else {
                VisitResult::Continue
            }
        }
        fn visit_video(&mut self, _ctx: &NodeContext<'_>, _src: Option<&str>) -> VisitResult {
            self.calls.push("video".to_string());
            VisitResult::Continue
        }
        fn visit_link(&mut self, _ctx: &NodeContext<'_>, _href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            self.calls.push(format!("link {text}"));
            VisitResult::Continue
        }
    }

    fn observe(html: &str, start: VisitResult, end: VisitResult) -> (String, Vec<String>) {
        let observer = Arc::new(Mutex::new(Observer {
            target: "span",
            start,
            end,
            calls: Vec::new(),
        }));
        let visitor: VisitorHandle = observer.clone();
        let output = convert(
            html,
            Some(ConversionOptions {
                visitor: Some(visitor),
                extract_metadata: false,
                ..Default::default()
            }),
        )
        .expect("render neighbor visitor")
        .content
        .unwrap_or_default();
        let calls = std::mem::take(&mut observer.lock().expect("observer mutex").calls);
        (output, calls)
    }

    #[test]
    fn should_resolve_skipped_neighbor_spacing_from_the_next_emitted_word_or_punctuation() {
        for (tail, expected) in [("!", "video!"), ("text", "video text")] {
            let html = format!("<a href=/x><video src=/media></video><span>word</span>{tail}</a>");
            let (output, calls) = observe(&html, VisitResult::Skip, VisitResult::Continue);
            assert_eq!(output, format!("[{expected}](/x)\n"));
            assert_eq!(
                calls,
                ["video".to_string(), "start".to_string(), format!("link {expected}")]
            );
        }
    }

    #[test]
    fn should_only_mark_a_fallback_that_survives_child_callbacks() {
        for (start, end, expected, end_output) in [
            (VisitResult::Skip, VisitResult::Continue, "beforeafter", None),
            (
                VisitResult::Continue,
                VisitResult::Skip,
                "beforeafter",
                Some("fallback"),
            ),
            (
                VisitResult::Continue,
                VisitResult::Custom(String::new()),
                "beforeafter",
                Some("fallback"),
            ),
            (
                VisitResult::Continue,
                VisitResult::Custom("replacement".to_string()),
                "before replacement after",
                Some("fallback"),
            ),
            (
                VisitResult::Continue,
                VisitResult::Continue,
                "before fallback after",
                Some("fallback"),
            ),
        ] {
            let html = "<a href=/x>before<video><span>fallback</span></video>after</a>";
            let (output, calls) = observe(html, start, end);
            assert_eq!(output, format!("[{expected}](/x)\n"));
            let mut expected_calls = vec!["video".to_string(), "start".to_string()];
            if let Some(end_output) = end_output {
                expected_calls.push(format!("end {end_output}"));
            }
            expected_calls.push(format!("link {expected}"));
            assert_eq!(calls, expected_calls);
        }
    }

    #[test]
    fn should_resolve_neighbor_spacing_after_continue_custom_and_preserve_html_callbacks_once() {
        let html = "<a href=/x><video src=/media></video><span>word</span>!</a>";
        for (start, end, expected, end_output) in [
            (VisitResult::Continue, VisitResult::Continue, "video word!", "word"),
            (
                VisitResult::Custom("replacement".to_string()),
                VisitResult::Continue,
                "video replacement!",
                "replacement",
            ),
            (
                VisitResult::Continue,
                VisitResult::Custom("!".to_string()),
                "video!!",
                "word",
            ),
            (VisitResult::Continue, VisitResult::Skip, "video!", "word"),
            (
                VisitResult::PreserveHtml,
                VisitResult::PreserveHtml,
                "video word!",
                "word",
            ),
        ] {
            let (output, calls) = observe(html, start, end);
            assert_eq!(output, format!("[{expected}](/x)\n"));
            assert_eq!(
                calls,
                [
                    "video".to_string(),
                    "start".to_string(),
                    format!("end {end_output}"),
                    format!("link {expected}")
                ]
            );
        }
    }
}

#[test]
fn should_use_the_first_rendered_edge_of_inline_wrappers_and_isolate_each_link() {
    for tag in ["video", "audio"] {
        for (body, expected) in [
            ("<b>!</b>", "**!**"),
            ("<i>!</i>", "*!*"),
            ("<code>!</code>", "`!`"),
            ("<span><b>!</b></span>", "**!**"),
        ] {
            let html = format!("<a href=/x><{tag} src=/media></{tag}>{body}</a>");
            check_routes(&html, &format!("[{tag}{expected}](/x)\n"), CodeBlockStyle::Backticks);
        }
        let html = format!("<p>a<a href=/x><{tag} src=/media></{tag}></a>b<a href=/y>word</a>c</p>");
        check_routes(&html, &format!("a[{tag}](/x)b[word](/y)c\n"), CodeBlockStyle::Backticks);
    }
}

#[cfg(feature = "visitor")]
mod replaced_media_wrappers {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Replacement {
        calls: Vec<String>,
    }
    impl HtmlVisitor for Replacement {
        fn visit_video(&mut self, _ctx: &NodeContext<'_>, _src: Option<&str>) -> VisitResult {
            self.calls.push("video".to_string());
            VisitResult::Continue
        }
        fn visit_strong(&mut self, _ctx: &NodeContext<'_>, text: &str) -> VisitResult {
            self.calls.push(format!("strong {text}"));
            VisitResult::Custom("replacement".to_string())
        }
        fn visit_emphasis(&mut self, _ctx: &NodeContext<'_>, text: &str) -> VisitResult {
            self.calls.push(format!("emphasis {text}"));
            VisitResult::Custom("replacement".to_string())
        }
        fn visit_link(&mut self, _ctx: &NodeContext<'_>, _href: &str, text: &str, _title: Option<&str>) -> VisitResult {
            self.calls.push(format!("link {text}"));
            VisitResult::Continue
        }
    }

    #[test]
    fn should_discard_media_boundary_state_when_a_specialized_wrapper_callback_replaces_its_children() {
        for (tag, callback) in [("strong", "strong"), ("em", "emphasis")] {
            let html = format!("<a href=/x><{tag}><video src=/media></video></{tag}>word</a>");
            let recorder = Arc::new(Mutex::new(Replacement::default()));
            let visitor: VisitorHandle = recorder.clone();
            let output = convert(
                &html,
                Some(ConversionOptions {
                    visitor: Some(visitor),
                    extract_metadata: false,
                    ..Default::default()
                }),
            )
            .expect("replace media wrapper")
            .content
            .unwrap_or_default();
            assert_eq!(output, "[replacementword](/x)\n");
            let calls = std::mem::take(&mut recorder.lock().expect("recorder mutex").calls);
            assert_eq!(
                calls,
                [
                    "video".to_string(),
                    format!("{callback} "),
                    "link replacementword".to_string()
                ]
            );
        }
    }
}

#[test]
fn should_use_rendered_opening_punctuation_and_words_before_media_inside_formatting() {
    for tag in ["video", "audio"] {
        for (body, expected) in [
            ("<b>(</b>", format!("**(**{tag}")),
            ("<i>(</i>", format!("*(*{tag}")),
            ("<span><b>(</b></span>", format!("**(**{tag}")),
            ("<b>word</b>", format!("**word** {tag}")),
            (
                "<span>word<video src=/other></video></span>",
                format!("word video {tag}"),
            ),
        ] {
            let html = format!("<a href=/x>{body}<{tag} src=/media></{tag}></a>");
            check_routes(&html, &format!("[{expected}](/x)\n"), CodeBlockStyle::Backticks);
        }
    }
}

#[test]
fn should_separate_surviving_media_fallback_without_an_address_from_the_next_word() {
    for tag in ["video", "audio"] {
        for (body, expected) in [
            (format!("<{tag}>fallback</{tag}>text"), "fallback text"),
            (format!("<span><{tag}>fallback</{tag}></span>text"), "fallback text"),
            (format!("<{tag}><b>fallback</b></{tag}>text"), "**fallback** text"),
            (format!("<{tag}>fallback</{tag}><b>text</b>"), "fallback **text**"),
            (
                format!("<{tag}>fallback</{tag}><span></span><!-- gap -->text"),
                "fallback text",
            ),
            (
                format!("<{tag}>fallback</{tag}><{tag}>next</{tag}>text"),
                "fallback next text",
            ),
            (format!("<{tag}><source>fallback</{tag}>text"), "fallback text"),
            (
                format!("<{tag}>fallback</{tag}><video src=/media></video>text"),
                "fallback video text",
            ),
        ] {
            check_routes(
                &format!("<a href=/x>{body}</a>"),
                &format!("[{expected}](/x)\n"),
                CodeBlockStyle::Backticks,
            );
        }
    }
}

#[test]
fn should_keep_empty_fallback_and_rendered_punctuation_boundaries_inside_the_anchor() {
    for tag in ["video", "audio"] {
        for (body, expected) in [
            (format!("before<{tag}></{tag}>after"), "beforeafter"),
            (format!("before<{tag}> </{tag}>after"), "beforeafter"),
            (
                format!("before<{tag}><span></span><!-- gap --></{tag}>after"),
                "beforeafter",
            ),
            (format!("<{tag}>fallback</{tag}>!"), "fallback!"),
            (format!("<{tag}>fallback</{tag}><b>!</b>"), "fallback**!**"),
            (format!("<{tag}><b>(</b></{tag}>text"), "**(**text"),
            (format!("<{tag}><i>(</i></{tag}><b>text</b>"), "*(***text**"),
        ] {
            check_routes(
                &format!("<a href=/x>{body}</a>"),
                &format!("[{expected}](/x)\n"),
                CodeBlockStyle::Backticks,
            );
        }
        check_routes(
            &format!("<p>a<a href=/x><{tag}>fallback</{tag}></a>b</p>"),
            "a[fallback](/x)b\n",
            CodeBlockStyle::Backticks,
        );
    }
}

#[test]
fn should_keep_surviving_fallback_boundaries_when_a_data_destination_is_suppressed() {
    for tag in ["video", "audio"] {
        for tier_strategy in STRATEGIES {
            for (inline_data_media, expected) in [
                (InlineDataMedia::AltTextOnly, "[before fallback after](/x)\n"),
                (InlineDataMedia::DropElement, "[beforeafter](/x)\n"),
            ] {
                let html =
                    format!("<a href=/x>before<{tag} src='data:video/mp4;base64,AAAA'>fallback</{tag}>after</a>");
                let options = ConversionOptions {
                    tier_strategy,
                    inline_data_media,
                    extract_metadata: false,
                    ..Default::default()
                };
                let output = convert(&html, options)
                    .expect("data media control")
                    .content
                    .unwrap_or_default();
                assert_eq!(output, expected, "{tag}, {tier_strategy:?}, {inline_data_media:?}");
            }
        }
    }
}
