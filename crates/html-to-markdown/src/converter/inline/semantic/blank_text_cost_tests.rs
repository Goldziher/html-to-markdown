use crate::converter::context::{
    INLINE_BUFFER_COLUMN_CHAR_READS, INLINE_BUFFER_CONTEXT_CALLS, INLINE_BUFFER_LINE_SEARCH_CALLS,
};
use crate::{
    ConversionOptions, ConversionResult, DocumentNode, DocumentStructure, HighlightStyle, NodeContent, OutputFormat,
    TierStrategy, WhitespaceMode, convert,
};

fn assert_side_data(result: &ConversionResult, title: Option<&str>, nodes: Vec<DocumentNode>) {
    assert_eq!(
        result.document,
        Some(DocumentStructure {
            nodes,
            source_format: Some("html".to_string())
        })
    );
    assert_eq!(result.tables.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    #[cfg(feature = "metadata")]
    {
        let expected = crate::metadata::HtmlMetadata {
            document: crate::metadata::DocumentMetadata {
                title: title.map(str::to_string),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&result.metadata).expect("metadata JSON"),
            serde_json::to_value(expected).expect("expected metadata JSON")
        );
    }
    #[cfg(not(feature = "metadata"))]
    let _ = title;
}

#[test]
fn blank_marks_should_preserve_styles_tags_wrapping_and_side_data() {
    let cases = [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot]
        .into_iter()
        .flat_map(|format| {
            [
                HighlightStyle::None,
                HighlightStyle::DoubleEqual,
                HighlightStyle::Bold,
                HighlightStyle::Html,
            ]
            .into_iter()
            .flat_map(move |style| [false, true].map(move |preserved| (format, style, preserved)))
        });
    for (output_format, highlight_style, preserved) in cases {
        let options = ConversionOptions {
            output_format,
            highlight_style,
            whitespace_mode: WhitespaceMode::Strict,
            wrap: true,
            wrap_width: 40,
            include_document_structure: true,
            extract_metadata: true,
            preserve_tags: if preserved { vec!["mark".to_string()] } else { vec![] },
            tier_strategy: TierStrategy::Tier2,
            ..Default::default()
        };
        let result = convert("<html><head><title>T</title></head><body><div>\u{2000}<mark>\u{2000}</mark>\u{2000}<mark>\u{2000}</mark>z</div></body></html>", options).expect("convert blank mark options");
        let expected = if output_format == OutputFormat::Plain {
            "    z\n".to_string()
        } else if preserved || highlight_style == HighlightStyle::Html {
            "---\ntitle: T\n---\n\n\u{2000}<mark>\u{2000}</mark>\u{2000}<mark>\u{2000}</mark>z\n\n".to_string()
        } else {
            "---\ntitle: T\n---\n\n\u{2000} \u{2000} z\n\n".to_string()
        };
        assert_eq!(result.content.as_deref(), Some(expected.as_str()));
        assert_side_data(&result, Some("T"), vec![]);
    }
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct BlankVisitor {
    answer: crate::visitor::VisitResult,
    marks: Vec<String>,
    texts: Vec<String>,
}

#[cfg(feature = "visitor")]
impl crate::visitor::HtmlVisitor for BlankVisitor {
    fn visit_mark(&mut self, _ctx: &crate::visitor::NodeContext<'_>, text: &str) -> crate::visitor::VisitResult {
        self.marks.push(text.to_string());
        self.answer.clone()
    }
    fn visit_text(&mut self, _ctx: &crate::visitor::NodeContext<'_>, text: &str) -> crate::visitor::VisitResult {
        self.texts.push(text.to_string());
        if text.trim().is_empty() {
            crate::visitor::VisitResult::Custom("1. visible".to_string())
        } else {
            crate::visitor::VisitResult::Continue
        }
    }
}

#[cfg(feature = "visitor")]
#[test]
fn blank_marks_should_keep_parent_and_plain_text_callback_results() {
    use crate::visitor::{VisitResult, VisitorHandle};
    use std::sync::{Arc, Mutex};
    for (answer, markup) in [
        (VisitResult::Continue, " "),
        (VisitResult::Skip, ""),
        (VisitResult::Custom("C".to_string()), "C"),
        (VisitResult::PreserveHtml, "<mark>\u{2000}</mark>"),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
            let recorder = Arc::new(Mutex::new(BlankVisitor {
                answer: answer.clone(),
                marks: vec![],
                texts: vec![],
            }));
            let handle: VisitorHandle = recorder.clone();
            let options = ConversionOptions {
                output_format,
                whitespace_mode: WhitespaceMode::Strict,
                highlight_style: HighlightStyle::None,
                tier_strategy: TierStrategy::Tier2,
                extract_metadata: true,
                include_document_structure: true,
                visitor: Some(handle),
                ..Default::default()
            };
            let result = convert("<html><head><title>T</title></head><body><div>\u{2000}<mark>\u{2000}</mark>\u{2000}<mark>\u{2000}</mark>z</div></body></html>", options).expect("convert visited blank marks");
            let expected = if output_format == OutputFormat::Plain {
                "1. visible1. visible1. visible1. visiblez\n".to_string()
            } else {
                format!("---\ntitle: T\n---\n\n\u{2000}{markup}\u{2000}{markup}z\n")
            };
            assert_eq!(result.content.as_deref(), Some(expected.as_str()));
            assert_side_data(&result, Some("T"), vec![]);
            let rec = recorder.lock().expect("visitor recorder");
            assert_eq!(rec.marks, ["\u{2000}", "\u{2000}"]);
            let texts = if output_format == OutputFormat::Plain {
                vec!["z", "\u{2000}", "\u{2000}", "\u{2000}", "\u{2000}", "z"]
            } else {
                vec!["z"]
            };
            assert_eq!(rec.texts, texts);
            drop(rec);
        }
    }
}

#[test]
fn blank_marks_should_keep_line_break_code_and_fallback_policies() {
    let cases = [
        ("<div>\u{2000}<mark>\r\n </mark>z</div>", "\u{2000} z\n", 0, false),
        (
            "<div>\u{2000}<mark>\t\u{2000}\u{a0}</mark>z</div>",
            "\u{2000} z\n",
            0,
            false,
        ),
        ("<div>x<br><mark>\r\n </mark>z</div>", "x  \nz\n", 0, false),
        (
            "<pre>\u{2000}<mark>\u{2000}</mark>z</pre>",
            "```\n\u{2000}\u{2000}z\n```\n",
            0,
            true,
        ),
        (
            "<code>\u{2000}<mark>\u{2000}</mark>z</code>",
            "`\u{2000}\u{2000}z`\n",
            0,
            false,
        ),
        (
            "<div>\u{2000}<mark> <!--x-->\u{2000}</mark>z</div>",
            "\u{2000} z\n",
            1,
            false,
        ),
        ("<div>\u{2000}<mark> &nbsp;</mark>z</div>", "\u{2000} z\n", 1, false),
    ];
    for (html, expected, searches, code_block) in cases {
        INLINE_BUFFER_LINE_SEARCH_CALLS.with(|v| v.set(0));
        let options = ConversionOptions {
            whitespace_mode: WhitespaceMode::Strict,
            highlight_style: HighlightStyle::None,
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: true,
            include_document_structure: true,
            ..Default::default()
        };
        let result = convert(html, options).expect("convert blank policy");
        assert_eq!(result.content.as_deref(), Some(expected));
        assert_eq!(INLINE_BUFFER_LINE_SEARCH_CALLS.with(std::cell::Cell::get), searches);
        let nodes = if code_block {
            vec![DocumentNode {
                id: "code-3e253c763b691440".to_string(),
                content: NodeContent::Code {
                    text: "\u{2000}\u{2000}z".to_string(),
                    language: None,
                },
                parent: None,
                children: vec![],
                annotations: vec![],
                attributes: None,
            }]
        } else {
            vec![]
        };
        assert_side_data(&result, None, nodes);
    }
}

#[test]
fn blank_text_eligibility_should_accept_multiple_raw_nodes_and_reject_other_children() {
    let dom = tl::parse(
        "<mark> </mark><mark>\r\n\u{2000}</mark><span>x</span>",
        Default::default(),
    )
    .expect("parse raw node controls");
    let parser = dom.parser();
    let mut tag = dom.children()[0]
        .get(parser)
        .and_then(tl::Node::as_tag)
        .expect("first mark")
        .clone();
    let second = dom.children()[1]
        .get(parser)
        .and_then(tl::Node::as_tag)
        .expect("second mark");
    let raw = second
        .children()
        .top()
        .as_slice()
        .first()
        .copied()
        .expect("second raw node");
    tag.children_mut().top_mut().push(raw);
    assert_eq!(tag.children().top().len(), 2);
    assert!(super::has_only_blank_text_children(&tag, parser));
    tag.children_mut().top_mut().push(dom.children()[2]);
    assert!(!super::has_only_blank_text_children(&tag, parser));
    for html in [
        "<mark>&nbsp;</mark>",
        "<mark>\u{c}</mark>",
        "<mark><!--x--></mark>",
        "<mark><span></span></mark>",
        "<mark>x</mark>",
    ] {
        let dom = tl::parse(html, Default::default()).expect("parse nonblank control");
        let tag = dom.children()[0]
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("mark");
        assert!(!super::has_only_blank_text_children(tag, dom.parser()), "{html:?}");
    }
}

#[test]
fn blank_text_marks_should_skip_unused_native_line_searches() {
    let cases = ['\u{2000}', '\u{a0}', ' ', '\t'].into_iter().flat_map(|space| {
        [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot]
            .into_iter()
            .flat_map(move |format| {
                [WhitespaceMode::Strict, WhitespaceMode::Normalized]
                    .into_iter()
                    .flat_map(move |whitespace| [100, 400].map(move |n| (space, format, whitespace, n)))
            })
    });
    for (space, output_format, whitespace_mode, n) in cases {
        INLINE_BUFFER_CONTEXT_CALLS.with(|v| v.set(0));
        INLINE_BUFFER_LINE_SEARCH_CALLS.with(|v| v.set(0));
        INLINE_BUFFER_COLUMN_CHAR_READS.with(|v| v.set(0));
        let options = ConversionOptions {
            output_format,
            whitespace_mode,
            highlight_style: HighlightStyle::None,
            tier_strategy: TierStrategy::Tier2,
            extract_metadata: false,
            ..Default::default()
        };
        let html = format!("<div>{}z</div>", format!("{space}<mark>{space}</mark>").repeat(n));
        let result = convert(&html, options).expect("convert blank text marks");
        let expected = if output_format == OutputFormat::Plain {
            format!("{}z\n", "  ".repeat(n))
        } else if whitespace_mode == WhitespaceMode::Normalized {
            "z\n".to_string()
        } else if space == ' ' {
            format!("{}z\n", " ".repeat(n))
        } else {
            format!("{}z\n", format!("{space} ").repeat(n))
        };
        assert_eq!(result.content.as_deref(), Some(expected.as_str()));
        assert_eq!(INLINE_BUFFER_LINE_SEARCH_CALLS.with(std::cell::Cell::get), 0);
        assert_eq!(INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get), 0);
        assert_eq!(INLINE_BUFFER_CONTEXT_CALLS.with(std::cell::Cell::get), 0);
    }
}
