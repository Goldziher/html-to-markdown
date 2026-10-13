use crate::{ConversionOptions, HighlightStyle, OutputFormat, TierStrategy, WhitespaceMode, convert};

#[test]
fn raw_marks_should_preserve_styles_wrapping_and_side_data() {
    let html =
        "<html><head><title>T</title></head><body><div>1. <mark>1. </mark><mark>雪&#49;. </mark>z</div></body></html>";
    let mut rows = 0;
    for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
        for highlight_style in [
            HighlightStyle::None,
            HighlightStyle::DoubleEqual,
            HighlightStyle::Bold,
            HighlightStyle::Html,
        ] {
            for preserved in [false, true] {
                let options = ConversionOptions {
                    output_format,
                    highlight_style,
                    whitespace_mode: WhitespaceMode::Strict,
                    wrap: true,
                    wrap_width: 40,
                    extract_metadata: true,
                    include_document_structure: true,
                    preserve_tags: if preserved { vec!["mark".into()] } else { vec![] },
                    tier_strategy: TierStrategy::Tier2,
                    ..Default::default()
                };
                assert_row(
                    &format!("option/{output_format:?}/{highlight_style:?}/{preserved}"),
                    html,
                    options,
                    None,
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 24);
}

#[test]
fn raw_marks_should_preserve_nested_lists_and_boundary_side_data() {
    let mut rows = 0;
    for (id, html) in [
        (
            "list",
            "<ol start='12'><li><mark><ul><li>b<blockquote><p>c</p></blockquote></li></ul></mark></li></ol>",
        ),
        ("span", "<div>1. <mark><span>1. </span></mark>z</div>"),
        ("hardbreak", "<div>x<br><mark>\r\n1. </mark>z</div>"),
        ("unicode", "<div>\u{2000}1. <mark>雪\r\n&#49;. </mark>z</div>"),
        ("formfeed", "<div>1. <mark>\u{c}1. </mark>z</div>"),
        ("comment", "<div>1. <mark> <!--x-->1. </mark>z</div>"),
        ("code", "<code>1. <mark>1. </mark>z</code>"),
        ("pre", "<pre>1. <mark>1. </mark>z</pre>"),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
            assert_row(
                &format!("policy/{id}/{output_format:?}"),
                html,
                ConversionOptions {
                    output_format,
                    highlight_style: HighlightStyle::None,
                    whitespace_mode: WhitespaceMode::Strict,
                    extract_metadata: true,
                    include_document_structure: true,
                    tier_strategy: TierStrategy::Tier2,
                    ..Default::default()
                },
                None,
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 24);
}

fn assert_row(id: &str, html: &str, options: ConversionOptions, calls: Option<(Vec<String>, Vec<String>)>) {
    crate::converter::context::INLINE_BUFFER_COLUMN_CHAR_READS.with(|v| v.set(0));
    let result = convert(html, options).expect("policy conversion");
    assert_result(id, &result, calls);
}
fn assert_result(id: &str, result: &crate::ConversionResult, calls: Option<(Vec<String>, Vec<String>)>) {
    assert_eq!(result.content.as_deref(), Some(expected_content(id)), "{id}");
    assert_eq!(format!("{:?}", result.document), expected_document(id), "{id}");
    assert_eq!(result.tables.len(), 0, "{id}");
    assert_eq!(result.warnings.len(), 0, "{id}");
    #[cfg(feature = "metadata")]
    {
        let expected = crate::metadata::HtmlMetadata {
            document: crate::metadata::DocumentMetadata {
                title: if id.starts_with("policy/") {
                    None
                } else {
                    Some("T".to_string())
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&result.metadata).expect("metadata"),
            serde_json::to_value(expected).expect("expected metadata"),
            "{id}"
        );
    }
    if id.starts_with("policy/list/") {
        assert_eq!(
            crate::converter::context::INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get),
            4
        );
    }
    #[cfg(feature = "visitor")]
    if let Some(calls) = calls {
        assert_eq!(
            serde_json::to_value(calls).expect("callback JSON"),
            serde_json::from_str::<serde_json::Value>(expected_calls(id)).expect("expected callback JSON"),
            "{id}"
        );
    }
    #[cfg(not(feature = "visitor"))]
    let _ = calls;
}

#[cfg(feature = "visitor")]
#[derive(Debug)]
struct Recorder {
    answer: crate::visitor::VisitResult,
    marks: Vec<String>,
    texts: Vec<String>,
}
#[cfg(feature = "visitor")]
impl crate::visitor::HtmlVisitor for Recorder {
    fn visit_mark(&mut self, _: &crate::visitor::NodeContext<'_>, text: &str) -> crate::visitor::VisitResult {
        self.marks.push(text.into());
        self.answer.clone()
    }
    fn visit_text(&mut self, _: &crate::visitor::NodeContext<'_>, text: &str) -> crate::visitor::VisitResult {
        self.texts.push(text.into());
        crate::visitor::VisitResult::Custom("12. replacement\n- literal".into())
    }
}
#[cfg(feature = "visitor")]
#[test]
fn raw_marks_should_preserve_hook_replacements_and_callback_sequences() {
    use crate::visitor::VisitResult;
    let mut rows = 0;
    for (id, answer) in [
        ("Continue", VisitResult::Continue),
        ("Skip", VisitResult::Skip),
        ("Custom", VisitResult::Custom("C".into())),
        ("PreserveHtml", VisitResult::PreserveHtml),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
            let (result, calls) = convert_recorded_mark(output_format, answer.clone());
            assert_result(&format!("hook/{id}/{output_format:?}"), &result, Some(calls));
            rows += 1;
        }
    }
    assert_eq!(rows, 12);
}

#[cfg(feature = "visitor")]
fn convert_recorded_mark(
    output_format: OutputFormat,
    answer: crate::visitor::VisitResult,
) -> (crate::ConversionResult, (Vec<String>, Vec<String>)) {
    use crate::visitor::VisitorHandle;
    use std::sync::{Arc, Mutex};
    let recorder = Arc::new(Mutex::new(Recorder {
        answer,
        marks: vec![],
        texts: vec![],
    }));
    let handle: VisitorHandle = recorder.clone();
    let result = convert(
        "<html><head><title>T</title></head><body><div>1. <mark>1. </mark><mark>雪&#49;. </mark>z</div></body></html>",
        ConversionOptions {
            output_format,
            highlight_style: HighlightStyle::None,
            whitespace_mode: WhitespaceMode::Strict,
            extract_metadata: true,
            include_document_structure: true,
            tier_strategy: TierStrategy::Tier2,
            visitor: Some(handle),
            ..Default::default()
        },
    )
    .expect("hook conversion");
    let rec = recorder.lock().expect("recorder");
    let calls = (rec.marks.clone(), rec.texts.clone());
    drop(rec);
    (result, calls)
}

const CONTENT_ORACLES: &[(&str, &str)] = &[
    (
        "hook/Continue/Markdown",
        concat!(
            "---\ntitle: T\n---\n\n12\\. replacement\n- literal12. replacement\n- lit",
            "eral12. replacement\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/Continue/Plain",
        concat!(
            "12. replacement\n- literal12. replacement\n- literal12. replacement",
            "\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/Continue/Djot",
        concat!(
            "---\ntitle: T\n---\n\n12. replacement\n- literal12. replacement\n- lite",
            "ral12. replacement\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/Skip/Markdown",
        "---\ntitle: T\n---\n\n12\\. replacement\n- literal12. replacement\n- literal\n",
    ),
    (
        "hook/Skip/Plain",
        concat!(
            "12. replacement\n- literal12. replacement\n- literal12. replacement",
            "\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/Skip/Djot",
        "---\ntitle: T\n---\n\n12. replacement\n- literal12. replacement\n- literal\n",
    ),
    (
        "hook/Custom/Markdown",
        "---\ntitle: T\n---\n\n12\\. replacement\n- literalCC12. replacement\n- literal\n",
    ),
    (
        "hook/Custom/Plain",
        concat!(
            "12. replacement\n- literal12. replacement\n- literal12. replacement",
            "\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/Custom/Djot",
        "---\ntitle: T\n---\n\n12. replacement\n- literalCC12. replacement\n- literal\n",
    ),
    (
        "hook/PreserveHtml/Markdown",
        concat!(
            "---\ntitle: T\n---\n\n12\\. replacement\n- literal<mark>1. </mark><mark",
            ">雪&#49;. </mark>12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/PreserveHtml/Plain",
        concat!(
            "12. replacement\n- literal12. replacement\n- literal12. replacement",
            "\n- literal12. replacement\n- literal\n",
        ),
    ),
    (
        "hook/PreserveHtml/Djot",
        concat!(
            "---\ntitle: T\n---\n\n12. replacement\n- literal<mark>1. </mark><mark>",
            "雪&#49;. </mark>12. replacement\n- literal\n",
        ),
    ),
    ("option/Markdown/None/false", "---\ntitle: T\n---\n\n1\\. 1. 雪1. z\n\n"),
    (
        "option/Markdown/None/true",
        "---\ntitle: T\n---\n\n1\\. <mark>1. </mark><mark>雪&#49;.\n</mark>z\n\n",
    ),
    (
        "option/Markdown/DoubleEqual/false",
        "---\ntitle: T\n---\n\n1\\. ==1.== ==雪1.== z\n\n",
    ),
    (
        "option/Markdown/DoubleEqual/true",
        "---\ntitle: T\n---\n\n1\\. <mark>1. </mark><mark>雪&#49;.\n</mark>z\n\n",
    ),
    (
        "option/Markdown/Bold/false",
        "---\ntitle: T\n---\n\n1\\. **1.** **雪1.** z\n\n",
    ),
    (
        "option/Markdown/Bold/true",
        "---\ntitle: T\n---\n\n1\\. <mark>1. </mark><mark>雪&#49;.\n</mark>z\n\n",
    ),
    (
        "option/Markdown/Html/false",
        "---\ntitle: T\n---\n\n1\\. <mark>1. </mark><mark>雪1. </mark>z\n\n",
    ),
    (
        "option/Markdown/Html/true",
        "---\ntitle: T\n---\n\n1\\. <mark>1. </mark><mark>雪&#49;.\n</mark>z\n\n",
    ),
    ("option/Plain/None/false", "1. 1. 雪1. z\n"),
    ("option/Plain/None/true", "1. 1. 雪1. z\n"),
    ("option/Plain/DoubleEqual/false", "1. 1. 雪1. z\n"),
    ("option/Plain/DoubleEqual/true", "1. 1. 雪1. z\n"),
    ("option/Plain/Bold/false", "1. 1. 雪1. z\n"),
    ("option/Plain/Bold/true", "1. 1. 雪1. z\n"),
    ("option/Plain/Html/false", "1. 1. 雪1. z\n"),
    ("option/Plain/Html/true", "1. 1. 雪1. z\n"),
    ("option/Djot/None/false", "---\ntitle: T\n---\n\n1. 1. 雪1. z\n"),
    (
        "option/Djot/None/true",
        "---\ntitle: T\n---\n\n1. <mark>1. </mark><mark>雪&#49;.\n   </mark>z\n",
    ),
    (
        "option/Djot/DoubleEqual/false",
        "---\ntitle: T\n---\n\n1. {=1.=} {=雪1.=} z\n",
    ),
    (
        "option/Djot/DoubleEqual/true",
        "---\ntitle: T\n---\n\n1. <mark>1. </mark><mark>雪&#49;.\n   </mark>z\n",
    ),
    ("option/Djot/Bold/false", "---\ntitle: T\n---\n\n1. **1.** **雪1.** z\n"),
    (
        "option/Djot/Bold/true",
        "---\ntitle: T\n---\n\n1. <mark>1. </mark><mark>雪&#49;.\n   </mark>z\n",
    ),
    (
        "option/Djot/Html/false",
        "---\ntitle: T\n---\n\n1. <mark>1. </mark><mark>雪1. </mark>z\n",
    ),
    (
        "option/Djot/Html/true",
        "---\ntitle: T\n---\n\n1. <mark>1. </mark><mark>雪&#49;.\n   </mark>z\n",
    ),
    ("policy/list/Markdown", "12. - b\n      > c\n"),
    ("policy/list/Plain", "12.\n- b\n\nc\n"),
    ("policy/list/Djot", "12. - b\n      > c\n"),
    ("policy/span/Markdown", "1\\. 1. z\n"),
    ("policy/span/Plain", "1. 1. z\n"),
    ("policy/span/Djot", "1. 1. z\n"),
    ("policy/hardbreak/Markdown", "x  \n1. z\n"),
    ("policy/hardbreak/Plain", "x\n\n1. z\n"),
    ("policy/hardbreak/Djot", "x\\\n1. z\n"),
    ("policy/unicode/Markdown", "\u{2000}1. 雪\n1. z\n"),
    ("policy/unicode/Plain", " 1. 雪\n1. z\n"),
    ("policy/unicode/Djot", "\u{2000}1. 雪\n1. z\n"),
    ("policy/formfeed/Markdown", "1\\. \u{c}1. z\n"),
    ("policy/formfeed/Plain", "1. \u{c}1. z\n"),
    ("policy/formfeed/Djot", "1. \u{c}1. z\n"),
    ("policy/comment/Markdown", "1\\.  1. z\n"),
    ("policy/comment/Plain", "1.  1. z\n"),
    ("policy/comment/Djot", "1.  1. z\n"),
    ("policy/code/Markdown", "`1. 1. z`\n"),
    ("policy/code/Plain", "1. 1. z\n"),
    ("policy/code/Djot", "`1. 1. z`\n"),
    ("policy/pre/Markdown", "```\n1. 1. z\n```\n"),
    ("policy/pre/Plain", "1. 1. z\n"),
    ("policy/pre/Djot", "```\n1. 1. z\n```\n"),
];

#[test]
fn content_oracles_should_have_sixty_unique_case_ids() {
    let ids: std::collections::BTreeSet<_> = CONTENT_ORACLES.iter().map(|(id, _)| *id).collect();
    assert_eq!(CONTENT_ORACLES.len(), 60);
    assert_eq!(ids.len(), 60);
}

fn expected_content(id: &str) -> &'static str {
    CONTENT_ORACLES
        .iter()
        .find_map(|(key, value)| (*key == id).then_some(*value))
        .expect("known output oracle")
}

fn expected_document(id: &str) -> &'static str {
    match id {
        "policy/list/Markdown" | "policy/list/Plain" | "policy/list/Djot" => concat!(
            "Some(DocumentStructure { nodes: [DocumentNode { id: \"list-02eef3c51aa026ea\", ",
            "content: List { ordered: true }, parent: None, children: [], annotations: [], ",
            "attributes: None }, DocumentNode { id: \"list-464921dfa5d83f29\", content: List { ",
            "ordered: false }, parent: None, children: [3], annotations: [], attributes: None }, ",
            "DocumentNode { id: \"paragraph-d2944f1eeac9d1c2\", content: Paragraph { text: \"c\" }, ",
            "parent: None, children: [], annotations: [], attributes: None }, DocumentNode { id: ",
            "\"list_item-631f7e0c3decaac2\", content: ListItem { text: \"bc\" }, parent: Some(1), ",
            "children: [], annotations: [], attributes: None }], source_format: Some(\"html\") })",
        ),
        "policy/pre/Markdown" | "policy/pre/Plain" | "policy/pre/Djot" => concat!(
            "Some(DocumentStructure { nodes: [DocumentNode { id: \"code-d0b73273c9b72376\", ",
            "content: Code { text: \"1. 1. z\", language: None }, parent: None, children: [], ",
            "annotations: [], attributes: None }], source_format: Some(\"html\") })",
        ),
        _ => "Some(DocumentStructure { nodes: [], source_format: Some(\"html\") })",
    }
}

#[cfg(feature = "visitor")]
fn expected_calls(id: &str) -> &'static str {
    match id {
        "hook/Continue/Markdown" => "[[\"1. \", \"雪1. \"], [\"1. \", \"1. \", \"雪1. \", \"z\"]]",
        "hook/Continue/Plain" => concat!(
            "[[\"1. \", \"雪1. \"], [\"1. \", \"1. \", \"雪1. \", \"z\", \"1. \", \"1. \", \"雪1. ",
            "\", \"z\"]]",
        ),
        "hook/Continue/Djot" => "[[\"1. \", \"雪1. \"], [\"1. \", \"1. \", \"雪1. \", \"z\"]]",
        "hook/Skip/Markdown" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        "hook/Skip/Plain" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\", \"1. \", \"1. \", \"雪1. \", \"z\"]]",
        "hook/Skip/Djot" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        "hook/Custom/Markdown" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        "hook/Custom/Plain" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\", \"1. \", \"1. \", \"雪1. \", \"z\"]]",
        "hook/Custom/Djot" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        "hook/PreserveHtml/Markdown" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        "hook/PreserveHtml/Plain" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\", \"1. \", \"1. \", \"雪1. \", \"z\"]]",
        "hook/PreserveHtml/Djot" => "[[\"1. \", \"雪1. \"], [\"1. \", \"z\"]]",
        _ => panic!("unknown callback oracle {id}"),
    }
}

#[test]
fn raw_mark_text_should_not_count_unused_columns_after_bare_markers() {
    use crate::converter::context::INLINE_BUFFER_COLUMN_CHAR_READS;
    let mut cases = 0;
    for (raw, normalized, plain) in [
        ("1. ", "1. ", "1. "),
        ("2) ", "2) ", "2) "),
        ("- ", "- ", "- "),
        ("x 1. ", "x 1. ", "x 1. "),
        ("\u{2000}- ", "- ", " - "),
        ("&#49;. ", "1. ", "1. "),
        ("\u{c}1. ", "\u{c}1. ", "\u{c}1. "),
        ("雪 ", "雪 ", "雪 "),
    ] {
        for output_format in [OutputFormat::Markdown, OutputFormat::Plain, OutputFormat::Djot] {
            for whitespace_mode in [WhitespaceMode::Strict, WhitespaceMode::Normalized] {
                for n in [100, 400] {
                    INLINE_BUFFER_COLUMN_CHAR_READS.with(|v| v.set(0));
                    let result = convert(
                        &format!("<div>{}</div>", format!("<mark>{raw}</mark>").repeat(n)),
                        ConversionOptions {
                            output_format,
                            whitespace_mode,
                            highlight_style: HighlightStyle::None,
                            tier_strategy: TierStrategy::Tier2,
                            extract_metadata: false,
                            ..Default::default()
                        },
                    )
                    .expect("raw mark conversion");
                    let text = if output_format == OutputFormat::Plain {
                        plain
                    } else {
                        normalized
                    };
                    let expected = format!("{}\n", text.repeat(n).trim_end());
                    assert_eq!(
                        result.content.as_deref(),
                        Some(expected.as_str()),
                        "{raw:?}/{output_format:?}/{whitespace_mode:?}/{n}"
                    );
                    assert_eq!(INLINE_BUFFER_COLUMN_CHAR_READS.with(std::cell::Cell::get), 0);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 96);
}
