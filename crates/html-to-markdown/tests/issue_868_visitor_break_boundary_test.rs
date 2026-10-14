//! Visitor omissions cannot turn strict source line endings into authored breaks. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

fn markup_options() -> Vec<ConversionOptions> {
    let mut options = Vec::new();
    for output_format in [OutputFormat::Markdown, OutputFormat::Djot] {
        for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                options.push(ConversionOptions {
                    output_format,
                    newline_style,
                    tier_strategy,
                    whitespace_mode: WhitespaceMode::Strict,
                    extract_metadata: false,
                    ..Default::default()
                });
            }
        }
    }
    options
}

#[test]
fn should_keep_strict_source_separators_without_a_visitor() {
    let mut rows = 0;
    for options in markup_options() {
        let result = convert("<div>a<b>\n</b>z</div>", options).expect("unobserved strict separator");
        assert_eq!(result.content.as_deref(), Some("a z\n"));
        rows += 1;
    }
    assert_eq!(rows, 12);
}

#[cfg(feature = "visitor")]
mod observed {
    use super::*;
    use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Copy, Debug)]
    enum Mode {
        Spaces,
        SkipEmphasis,
        SkipStart,
        Empty,
        SkipBreak,
        Newline,
        Text,
        Continue,
        SpacesAt(usize),
        CustomWhitespace(&'static str),
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct Counts {
        breaks: usize,
        starts: usize,
        emphasis: usize,
    }
    #[derive(Debug)]
    struct Hooks {
        mode: Mode,
        counts: Counts,
    }
    impl HtmlVisitor for Hooks {
        fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
            let index = self.counts.breaks;
            self.counts.breaks += 1;
            match self.mode {
                Mode::Spaces => VisitResult::Custom(" ".into()),
                Mode::Empty => VisitResult::Custom(String::new()),
                Mode::SkipBreak => VisitResult::Skip,
                Mode::Newline => VisitResult::Custom("\n".into()),
                Mode::Text => VisitResult::Custom("V".into()),
                Mode::CustomWhitespace(value) => VisitResult::Custom(value.into()),
                Mode::SpacesAt(at) if index == at => VisitResult::Custom(" ".into()),
                _ => VisitResult::Continue,
            }
        }
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            if context.tag_name != "i" {
                return VisitResult::Continue;
            }
            self.counts.starts += 1;
            if matches!(self.mode, Mode::SkipStart) {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
        fn visit_emphasis(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            if context.tag_name != "i" {
                return VisitResult::Continue;
            }
            self.counts.emphasis += 1;
            if matches!(self.mode, Mode::SkipEmphasis) {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
    }
    fn assert_case(source: &str, mut options: ConversionOptions, mode: Mode, expected: &str, counts: Counts) {
        let hooks = Arc::new(Mutex::new(Hooks {
            mode,
            counts: Counts::default(),
        }));
        options.visitor = Some(hooks.clone());
        let result = convert(source, options).expect("observed strict separator");
        assert_eq!(result.content.as_deref(), Some(expected), "{source:?} {mode:?}");
        assert_eq!(
            hooks.lock().expect("callback count").counts,
            counts,
            "{source:?} {mode:?}"
        );
    }
    fn assert_omitted_routes(source: &str, mode: Mode, counts: Counts) {
        let mut rows = 0;
        for options in markup_options() {
            assert_case(source, options, mode, "a z\n", counts);
            rows += 1;
        }
        assert_eq!(rows, 12);
    }
    #[test]
    fn should_collapse_strict_source_line_endings_after_custom_space_breaks() {
        assert_omitted_routes(
            "<div>a<b>\n<br></b>z</div>",
            Mode::Spaces,
            Counts {
                breaks: 1,
                starts: 0,
                emphasis: 0,
            },
        );
    }
    #[test]
    fn should_collapse_strict_source_line_endings_after_emphasis_skips_break_subtrees() {
        assert_omitted_routes(
            "<div>a<b>\n<i><br></i></b>z</div>",
            Mode::SkipEmphasis,
            Counts {
                breaks: 1,
                starts: 1,
                emphasis: 1,
            },
        );
    }
    #[test]
    fn should_collapse_strict_source_line_endings_after_start_hooks_skip_break_subtrees() {
        assert_omitted_routes(
            "<div>a<b>\n<i><br></i></b>z</div>",
            Mode::SkipStart,
            Counts {
                breaks: 0,
                starts: 1,
                emphasis: 0,
            },
        );
    }
    fn control_options() -> Vec<ConversionOptions> {
        let mut options = Vec::new();
        for output_format in [OutputFormat::Markdown, OutputFormat::Djot, OutputFormat::Plain] {
            for newline_style in [NewlineStyle::Spaces, NewlineStyle::Backslash] {
                for whitespace_mode in [WhitespaceMode::Normalized, WhitespaceMode::Strict] {
                    options.push(ConversionOptions {
                        output_format,
                        newline_style,
                        whitespace_mode,
                        extract_metadata: false,
                        ..Default::default()
                    });
                }
            }
        }
        options
    }
    struct Control {
        source: &'static str,
        mode: Mode,
        expected: [&'static str; 12],
        counts: [Counts; 2],
    }
    const CONTROLS: [Control; 13] = [
        Control {
            source: "<div>a<b>\n<br></b>z</div>",
            mode: Mode::Continue,
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br></b>z</div>",
            mode: Mode::Empty,
            expected: [
                "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a\n\nz\n", "a\n\nz\n",
                "a\n\nz\n", "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br></b>z</div>",
            mode: Mode::Spaces,
            expected: [
                "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a\n\nz\n", "a\n\nz\n",
                "a\n\nz\n", "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br></b>z</div>",
            mode: Mode::Newline,
            expected: [
                "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a\n\nz\n", "a\n\nz\n",
                "a\n\nz\n", "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br></b>z</div>",
            mode: Mode::Text,
            expected: [
                "a **V**z\n",
                "a  \n**V**z\n",
                "a **V**z\n",
                "a\\\n**V**z\n",
                "a *V*z\n",
                "a\\\n*V*z\n",
                "a *V*z\n",
                "a\\\n*V*z\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<i><br></i></b>z</div>",
            mode: Mode::SkipEmphasis,
            expected: [
                "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a\n\nz\n", "a\n\nz\n",
                "a\n\nz\n", "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 1,
                    emphasis: 1,
                },
                Counts {
                    breaks: 1,
                    starts: 2,
                    emphasis: 1,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<i><br></i></b>z</div>",
            mode: Mode::SkipStart,
            expected: [
                "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a z\n", "a\nz\n", "a\nz\n", "a\nz\n",
                "a\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 0,
                    starts: 1,
                    emphasis: 0,
                },
                Counts {
                    breaks: 0,
                    starts: 2,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<i><br></i><br></b>z</div>",
            mode: Mode::SkipEmphasis,
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 2,
                    starts: 1,
                    emphasis: 1,
                },
                Counts {
                    breaks: 2,
                    starts: 2,
                    emphasis: 1,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<i><br></i><br></b>z</div>",
            mode: Mode::SkipStart,
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 1,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 2,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br><i><br></i></b>z</div>",
            mode: Mode::SkipEmphasis,
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 2,
                    starts: 1,
                    emphasis: 1,
                },
                Counts {
                    breaks: 2,
                    starts: 2,
                    emphasis: 1,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br><i><br></i></b>z</div>",
            mode: Mode::SkipStart,
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 1,
                    starts: 1,
                    emphasis: 0,
                },
                Counts {
                    breaks: 1,
                    starts: 2,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br><br></b>z</div>",
            mode: Mode::SpacesAt(0),
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 2,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 2,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
        Control {
            source: "<div>a<b>\n<br><br></b>z</div>",
            mode: Mode::SpacesAt(1),
            expected: [
                "a  \nz\n",
                "a  \nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a \\\nz\n",
                "a\\\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
                "a\n\nz\n",
            ],
            counts: [
                Counts {
                    breaks: 2,
                    starts: 0,
                    emphasis: 0,
                },
                Counts {
                    breaks: 2,
                    starts: 0,
                    emphasis: 0,
                },
            ],
        },
    ];
    #[test]
    fn should_preserve_retained_breaks_and_plain_normalized_callback_contracts() {
        let mut rows = 0;
        for control in CONTROLS {
            for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
                for (index, mut options) in control_options().into_iter().enumerate() {
                    options.tier_strategy = tier_strategy;
                    let plain = usize::from(options.output_format == OutputFormat::Plain);
                    assert_case(
                        control.source,
                        options,
                        control.mode,
                        control.expected[index],
                        control.counts[plain],
                    );
                    rows += 1;
                }
            }
        }
        assert_eq!(rows, 468);
    }
    #[test]
    fn should_keep_existing_skipped_break_separator_behavior() {
        assert_omitted_routes(
            "<div>a<b>\n<br></b>z</div>",
            Mode::SkipBreak,
            Counts {
                breaks: 1,
                starts: 0,
                emphasis: 0,
            },
        );
    }

    #[derive(Debug, Default)]
    struct ReentryHooks {
        emphasis: bool,
        calls: usize,
        starts: usize,
        breaks: usize,
        rows: Vec<Vec<String>>,
    }
    impl ReentryHooks {
        const fn action(&mut self) -> VisitResult {
            self.calls += 1;
            if self.calls == 1 {
                VisitResult::Skip
            } else {
                VisitResult::Continue
            }
        }
        fn is_target(context: &NodeContext) -> bool {
            context.attributes().get("id").map(String::as_str) == Some("flip")
        }
    }
    impl HtmlVisitor for ReentryHooks {
        fn visit_element_start(&mut self, context: &NodeContext) -> VisitResult {
            if !Self::is_target(context) {
                return VisitResult::Continue;
            }
            self.starts += 1;
            if self.emphasis {
                VisitResult::Continue
            } else {
                self.action()
            }
        }
        fn visit_emphasis(&mut self, context: &NodeContext, _: &str) -> VisitResult {
            if Self::is_target(context) && self.emphasis {
                self.action()
            } else {
                VisitResult::Continue
            }
        }
        fn visit_line_break(&mut self, _: &NodeContext) -> VisitResult {
            self.breaks += 1;
            VisitResult::Continue
        }
        fn visit_table_row(&mut self, _: &NodeContext, cells: &[String], _: bool) -> VisitResult {
            self.rows.push(cells.to_vec());
            VisitResult::Continue
        }
    }
    fn assert_reentry(source: &str, emphasis: bool, br_in_tables: bool, expected: &str, first_cell: &str) {
        let hooks = Arc::new(Mutex::new(ReentryHooks {
            emphasis,
            ..Default::default()
        }));
        let result = convert(
            source,
            ConversionOptions {
                tier_strategy: TierStrategy::Tier2,
                whitespace_mode: WhitespaceMode::Strict,
                compact_tables: true,
                br_in_tables,
                extract_metadata: false,
                visitor: Some(hooks.clone()),
                ..Default::default()
            },
        )
        .expect("same-node table replay");
        assert_eq!(result.content.as_deref(), Some(expected));
        let hooks = hooks.lock().expect("table counts");
        assert_eq!(
            (hooks.calls, hooks.starts, hooks.breaks),
            (2, 2, if emphasis { 2 } else { 1 })
        );
        assert_eq!(hooks.rows, vec![vec!["H"], vec![first_cell]]);
        drop(hooks);
    }
    #[test]
    fn should_restore_break_ownership_when_table_nodes_change_from_skip_to_continue() {
        let source = "<table><tr><th>H</th></tr><tr><td>a<b>\n<i id=\"flip\"><br></i></b>z</td></tr></table>";
        for emphasis in [false, true] {
            assert_reentry(source, emphasis, false, "| H |\n| --- |\n| a   z |\n", "a z");
            assert_reentry(source, emphasis, true, "| H |\n| --- |\n| a***<br>***z |\n", "a z");
        }
        let source = "<table><tr><th>H</th></tr><tr><td><b>a<span id=\"flip\"><br></span>z</b></td></tr></table>";
        assert_reentry(source, false, false, "| H |\n| --- |\n| **a z** |\n", "**az**");
        assert_reentry(source, false, true, "| H |\n| --- |\n| **a<br>z** |\n", "**az**");
    }
    #[test]
    fn should_keep_css_omissions_during_repeated_table_visitor_passes() {
        let source = "<table><tr><th>H</th></tr><tr><td>a<b>\n<i id=\"flip\"><br></i></b>z</td></tr></table>";
        for br_in_tables in [false, true] {
            let hooks = Arc::new(Mutex::new(Hooks {
                mode: Mode::Continue,
                counts: Counts::default(),
            }));
            let result = convert(
                source,
                ConversionOptions {
                    tier_strategy: TierStrategy::Tier2,
                    whitespace_mode: WhitespaceMode::Strict,
                    compact_tables: true,
                    br_in_tables,
                    exclude_selectors: vec!["#flip".into()],
                    extract_metadata: false,
                    visitor: Some(hooks.clone()),
                    ..Default::default()
                },
            )
            .expect("CSS omission during table replay");
            assert_eq!(result.content.as_deref(), Some("| H |\n| --- |\n| a z |\n"));
            assert_eq!(
                hooks.lock().expect("CSS callback counts").counts,
                Counts {
                    breaks: 0,
                    starts: 2,
                    emphasis: 0
                }
            );
        }
    }
    #[test]
    fn should_collapse_strict_source_separators_after_tab_and_mixed_ascii_blank_breaks() {
        for value in ["\t", " \t "] {
            assert_omitted_routes(
                "<div>a<b>\n<br></b>z</div>",
                Mode::CustomWhitespace(value),
                Counts {
                    breaks: 1,
                    starts: 0,
                    emphasis: 0,
                },
            );
        }
    }
    #[test]
    fn should_preserve_existing_nonbreaking_custom_whitespace_output() {
        let mut rows = 0;
        for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
            for mut options in control_options() {
                options.tier_strategy = tier_strategy;
                let expected = if options.output_format == OutputFormat::Plain {
                    "a\n\nz\n"
                } else {
                    "a z\n"
                };
                assert_case(
                    "<div>a<b>\n<br></b>z</div>",
                    options,
                    Mode::CustomWhitespace("\u{a0}"),
                    expected,
                    Counts {
                        breaks: 1,
                        starts: 0,
                        emphasis: 0,
                    },
                );
                rows += 1;
            }
        }
        assert_eq!(rows, 36);
    }
}
