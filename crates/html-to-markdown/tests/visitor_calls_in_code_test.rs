// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]
#![allow(clippy::significant_drop_tightening)]
#![cfg(feature = "visitor")]

//! The calls a visitor gets for a link, a highlight and an image in code.
//!
//! Code writes a link and a highlight as their text. The visitor still gets the calls it gets for
//! the same markup outside code, in the same order: none for a link that is left out (an empty
//! anchor into its own page, which a highlighter writes for each line of code), none for a link in
//! the `<address>` form, none for a link around one heading.

use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult, VisitorHandle};
use html_to_markdown_rs::{CodeBlockStyle, ConversionOptions, InlineDataMedia, convert};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Recorder {
    calls: Vec<String>,
    link_texts: Vec<String>,
    code: Vec<String>,
}

impl HtmlVisitor for Recorder {
    fn visit_link(&mut self, _ctx: &NodeContext<'_>, href: &str, text: &str, _title: Option<&str>) -> VisitResult {
        self.calls.push(format!("link {href}"));
        self.link_texts.push(text.to_string());
        VisitResult::Continue
    }

    fn visit_mark(&mut self, _ctx: &NodeContext<'_>, text: &str) -> VisitResult {
        self.calls.push(format!("mark {text}"));
        VisitResult::Continue
    }

    fn visit_image(&mut self, _ctx: &NodeContext<'_>, src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        self.calls.push(format!("image {src}"));
        VisitResult::Continue
    }

    fn visit_code_block(&mut self, _ctx: &NodeContext<'_>, _lang: Option<&str>, code: &str) -> VisitResult {
        self.calls.push("code_block".to_string());
        self.code.push(code.to_string());
        VisitResult::Continue
    }

    fn visit_code_inline(&mut self, _ctx: &NodeContext<'_>, code: &str) -> VisitResult {
        self.calls.push("code_inline".to_string());
        self.code.push(code.to_string());
        VisitResult::Continue
    }
}

fn record(html: &str, style: CodeBlockStyle) -> (Recorder, String) {
    record_with(
        html,
        ConversionOptions {
            code_block_style: style,
            ..ConversionOptions::default()
        },
    )
}

fn record_with(html: &str, options: ConversionOptions) -> (Recorder, String) {
    let recorder = Arc::new(Mutex::new(Recorder::default()));
    let handle: VisitorHandle = recorder.clone();
    let options = ConversionOptions {
        extract_metadata: false,
        visitor: Some(handle),
        ..options
    };
    let content = convert(html, Some(options))
        .expect("the conversion succeeds")
        .content
        .unwrap_or_default();
    let seen = std::mem::take(&mut *recorder.lock().expect("the visitor lock"));
    (seen, content)
}

/// Each row: a name, the page, the calls in order. The calls are those of a run of the same page
/// before code wrote a link as its text.
const CASES: &[(&str, &str, &[&str])] = &[
    (
        "an empty anchor into its own page for each line (MkDocs)",
        "<pre><code><a id=\"l1\" href=\"#l1\"></a>a = 1\n<a id=\"l2\" href=\"#l2\"></a>  b = 2\n</code></pre>",
        &["code_block"],
    ),
    (
        "the same anchors in a line span",
        "<pre><span></span><code><span id=\"s1\"><a id=\"c1\" name=\"c1\" href=\"#c1\"></a><span class=\"k\">a</span>\n</span><span id=\"s2\"><a id=\"c2\" name=\"c2\" href=\"#c2\"></a>  b\n</span></code></pre>",
        &["code_block"],
    ),
    (
        "an empty anchor into its own page that holds an empty element",
        "<pre><span><a href=\"#s\"><span></span></a>x</span></pre>",
        &["code_block"],
    ),
    (
        "an anchor into its own page that holds one space",
        "<pre>a<a href=\"#x\"> </a>b</pre>",
        &["code_block"],
    ),
    (
        "an empty anchor into its own page that has a name",
        "<pre><a href=\"#n\" aria-label=\"N\" title=\"T\"></a>x</pre>",
        &["code_block"],
    ),
    (
        "an empty anchor in a code span",
        "<p>t <code><a href=\"#t\"></a>x</code></p>",
        &["code_inline"],
    ),
    (
        "a link with text in a pre",
        "<pre>see <a href=\"/x\">link</a> end</pre>",
        &["link /x", "code_block"],
    ),
    (
        "a link with text into its own page",
        "<pre>see <a href=\"#x\">link</a> end</pre>",
        &["link #x", "code_block"],
    ),
    (
        "a link in a code span",
        "<p>t <code>b <a href=\"/y\">two</a></code></p>",
        &["link /y", "code_inline"],
    ),
    (
        "a highlight in a pre",
        "<pre>a <mark>hot</mark> b</pre>",
        &["mark hot", "code_block"],
    ),
    (
        "a highlight in a code span",
        "<p>t <code>a <mark>m</mark></code></p>",
        &["mark m", "code_inline"],
    ),
    (
        "an empty highlight",
        "<pre><mark></mark>a</pre>",
        &["mark ", "code_block"],
    ),
    (
        "a link in a highlight",
        "<pre>a <mark>hot <a href=\"/x\">link</a></mark> b</pre>",
        &["mark hot link", "link /x", "code_block"],
    ),
    (
        "a highlight in a link",
        "<pre>a <a href=\"/x\"><mark>hot</mark> link</a> b</pre>",
        &["mark hot", "link /x", "code_block"],
    ),
    (
        "a link and a highlight in a code element in a pre",
        "<pre><code>see <a href=\"/x\">link</a>\n  <mark>m</mark></code></pre>",
        &["link /x", "mark m", "code_block"],
    ),
    (
        "an empty anchor to another page",
        "<pre><a href=\"/x\"></a>a</pre>",
        &["link /x", "code_block"],
    ),
    (
        "an empty anchor to another page with a title",
        "<pre><a href=\"/x\" title=\"T\"></a>a</pre>",
        &["link /x", "code_block"],
    ),
    (
        "three anchors: empty into its own page, with text, empty to another page",
        "<pre><a href=\"#a\"></a>x <a href=\"#b\">y</a> <a href=\"/c\"></a>z</pre>",
        &["link #b", "link /c", "code_block"],
    ),
    (
        "a pre in a list item",
        "<ul><li>t<pre>a <a href=\"/x\">l</a>\n<a href=\"#e\"></a>b</pre></li></ul>",
        &["link /x", "code_block"],
    ),
    (
        "an image alone in an anchor into its own page",
        "<pre><a href=\"#x\"><img src=\"i.png\" alt=\"p\"></a>z</pre>",
        &["image i.png", "link #x", "code_block"],
    ),
    (
        "an image alone in an anchor",
        "<pre><a href=\"/x\"><img src=\"i.png\" alt=\"p\"></a>z</pre>",
        &["image i.png", "link /x", "code_block"],
    ),
    (
        "an anchor with no address",
        "<pre><a id=\"l1\"></a>a</pre>",
        &["code_block"],
    ),
    (
        "a link whose text is its address",
        "<pre>see <a href=\"https://e.org/x\">https://e.org/x</a> end</pre>",
        &["code_block"],
    ),
    (
        "a mail link whose text is its address",
        "<pre>to <a href=\"mailto:a@e.org\">a@e.org</a></pre>",
        &["code_block"],
    ),
    (
        "a link whose text is its address in a code span",
        "<p>t <code><a href=\"https://e.org/x\">https://e.org/x</a></code></p>",
        &["code_inline"],
    ),
    (
        "a link around one heading",
        "<pre><a href=\"/h\"><h2>t</h2></a>x</pre>",
        &["code_block"],
    ),
    (
        "a link around one empty heading",
        "<pre><a href=\"/h\"><h2></h2></a>x</pre>",
        &["link /h", "code_block"],
    ),
    (
        "a link in `code` in a link, around an image",
        "<pre><a href=\"#x1\"><code><a href=\"#x2\"><code><img src=\"i.png\"></code></a></code></a>z</pre>",
        &["image i.png", "link #x2", "link #x1", "code_block"],
    ),
    (
        "four links, each in `code` in the link around it, around an image",
        "<pre><a href=\"#x1\"><code><a href=\"#x2\"><code><a href=\"#x3\"><code><a href=\"#x4\"><code><img src=\"i.png\"></code></a></code></a></code></a></code></a>z</pre>",
        &[
            "image i.png",
            "link #x4",
            "link #x3",
            "link #x2",
            "link #x1",
            "code_block",
        ],
    ),
    (
        "a link in `kbd` in a link, around an image",
        "<pre><a href=\"#x1\"><kbd><a href=\"#x2\"><kbd><img src=\"i.png\"></kbd></a></kbd></a>z</pre>",
        &["image i.png", "link #x2", "link #x1", "code_block"],
    ),
    (
        "a link in `samp` in a link, around an image",
        "<pre><a href=\"#x1\"><samp><a href=\"#x2\"><samp><img src=\"i.png\"></samp></a></samp></a>z</pre>",
        &["image i.png", "link #x2", "link #x1", "code_block"],
    ),
    (
        "a link into its own page around a line break in `code`",
        "<pre><a href=\"#x\"><code><br></code></a>z</pre>",
        &["code_block"],
    ),
    (
        "a link into its own page around a space in `kbd`, in a code span",
        "<p>t <code><a href=\"#x\"><kbd> </kbd></a>z</code></p>",
        &["code_inline"],
    ),
    (
        "a link into its own page around two line breaks in `samp`",
        "<pre><code><a href=\"#x\"><samp><br><br></samp></a>z</code></pre>",
        &["code_block"],
    ),
    (
        "a link into its own page around a no-break space in `code`, in a code span",
        "<p>t <code><a href=\"#x\"><code>&nbsp;</code></a>z</code></p>",
        &["code_inline"],
    ),
];

#[test]
fn a_visitor_gets_the_same_calls_for_links_and_highlights_in_code_as_before() {
    let mut failures = Vec::new();
    for (name, html, expected) in CASES {
        for style in [
            CodeBlockStyle::Backticks,
            CodeBlockStyle::Tildes,
            CodeBlockStyle::Indented,
        ] {
            let (seen, content) = record(html, style);
            if seen.calls != *expected {
                failures.push(format!(
                    "{name} ({style:?}): calls {:?}, expected {expected:?}; output {content:?}",
                    seen.calls
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows differ:\n{}",
        failures.len(),
        CASES.len() * 3,
        failures.join("\n")
    );
}

/// How a row of [`IMAGE_LINK_CASES`] sets the options.
#[derive(Clone, Copy, Debug)]
enum Images {
    Default,
    Skipped,
    DataDropped,
    DataAsAltText,
}

const DATA_IMAGE: &str = "data:image/png;base64,iVBORw0KGgo=";

/// Each row: a name, the options, the page (`{data}` stands for [`DATA_IMAGE`]), the calls in
/// order. The calls are those of a run of the same page before code wrote a link as its text.
const IMAGE_LINK_CASES: &[(&str, Images, &str, &[&str])] = &[
    (
        "a link into its own page that holds only an image, images skipped",
        Images::Skipped,
        "<pre><a href=\"#x\"><img src=\"i.png\" alt=\"p\"></a>z</pre>",
        &["image i.png", "code_block"],
    ),
    (
        "the same link in a code span",
        Images::Skipped,
        "<p>t <code><a href=\"#x\"><img src=\"i.png\" alt=\"p\"></a>z</code></p>",
        &["image i.png", "code_inline"],
    ),
    (
        "a link that holds only a graphic into its own page, images skipped",
        Images::Skipped,
        "<pre><a href=\"#x\"><svg width=\"4\" height=\"4\"><rect width=\"4\" height=\"4\"/></svg></a>z</pre>",
        &["code_block"],
    ),
    (
        "a link that holds only a data image, data images removed",
        Images::DataDropped,
        "<pre><a href=\"/x\"><img src=\"{data}\" alt=\"p\"></a>z</pre>",
        &["image {data}", "code_block"],
    ),
    (
        "the same link into its own page",
        Images::DataDropped,
        "<pre><a href=\"#x\"><img src=\"{data}\" alt=\"p\"></a>z</pre>",
        &["image {data}", "code_block"],
    ),
    (
        "the same link in a code span",
        Images::DataDropped,
        "<p>t <code><a href=\"/x\"><img src=\"{data}\" alt=\"p\"></a>z</code></p>",
        &["image {data}", "code_inline"],
    ),
    (
        "a link that holds only a graphic, data images removed",
        Images::DataDropped,
        "<pre><a href=\"/x\"><svg width=\"4\" height=\"4\"><rect width=\"4\" height=\"4\"/></svg></a>z</pre>",
        &["code_block"],
    ),
    (
        "the same link in a code span",
        Images::DataDropped,
        "<p>t <code><a href=\"/x\"><svg width=\"4\" height=\"4\"><rect width=\"4\" height=\"4\"/></svg></a>z</code></p>",
        &["code_inline"],
    ),
    (
        "a link into its own page that holds a picture without an image",
        Images::Default,
        "<pre><a href=\"#x\"><picture><source srcset=\"a.webp\"></picture></a>z</pre>",
        &["code_block"],
    ),
    (
        "a link into its own page that holds a data image with no alt text, data images as alt text",
        Images::DataAsAltText,
        "<pre><a href=\"#x\"><img src=\"{data}\" alt=\"\"></a>z</pre>",
        &["image {data}", "code_block"],
    ),
    (
        "a link to another page that holds only an image, images skipped",
        Images::Skipped,
        "<pre><a href=\"/x\"><img src=\"i.png\" alt=\"p\"></a>z</pre>",
        &["image i.png", "link /x", "code_block"],
    ),
    (
        "a link into its own page that holds an image with no alt text",
        Images::Default,
        "<pre><a href=\"#x\"><img src=\"i.png\" alt=\"\"></a>z</pre>",
        &["image i.png", "link #x", "code_block"],
    ),
    (
        "a link into its own page that holds an image with no alt text in a block",
        Images::Default,
        "<pre><a href=\"#l1\"><div><img src=\"i.png\"></div></a>x</pre>",
        &["image i.png", "code_block"],
    ),
    (
        "a link into its own page that holds an image and then an empty link",
        Images::Default,
        "<pre><a href=\"#o\"><img src=\"i.png\"><a href=\"#i\"></a></a>x</pre>",
        &["image i.png", "link #o", "code_block"],
    ),
    (
        "a link into its own page that holds only an empty link to another page",
        Images::Default,
        "<pre><a href=\"#o\"><a href=\"/q\"></a></a>x</pre>",
        &["link /q", "link #o", "code_block"],
    ),
    (
        "a link around one heading that holds only an image with alt text",
        Images::Default,
        "<pre><a href=\"/x\"><h2><img src=\"i.png\" alt=\"p\"></h2></a>z</pre>",
        &["image i.png", "code_block"],
    ),
    (
        "a link into its own page that holds only a graphic",
        Images::Default,
        "<pre><a href=\"#x\"><svg width=\"4\" height=\"4\"><rect width=\"4\" height=\"4\"/></svg></a>z</pre>",
        &["link #x", "code_block"],
    ),
    (
        "a link that holds a data image with alt text, data images as alt text",
        Images::DataAsAltText,
        "<pre><a href=\"/x\"><img src=\"{data}\" alt=\"p\"></a>z</pre>",
        &["image {data}", "link /x", "code_block"],
    ),
    (
        "a link into its own page that holds an image and a data image, data images removed",
        Images::DataDropped,
        "<pre><a href=\"#x\"><img src=\"i.png\" alt=\"p\"><img src=\"{data}\" alt=\"q\"></a>z</pre>",
        &["image i.png", "image {data}", "link #x", "code_block"],
    ),
];

#[test]
fn a_visitor_gets_the_same_calls_for_a_link_around_an_image_in_code_as_before() {
    let mut failures = Vec::new();
    for (name, images, html, expected) in IMAGE_LINK_CASES {
        let options = ConversionOptions {
            skip_images: matches!(images, Images::Skipped),
            inline_data_media: match images {
                Images::DataDropped => InlineDataMedia::DropElement,
                Images::DataAsAltText => InlineDataMedia::AltTextOnly,
                Images::Default | Images::Skipped => InlineDataMedia::default(),
            },
            ..ConversionOptions::default()
        };
        let expected: Vec<String> = expected.iter().map(|call| call.replace("{data}", DATA_IMAGE)).collect();
        let (seen, content) = record_with(&html.replace("{data}", DATA_IMAGE), options);
        if seen.calls != expected {
            failures.push(format!(
                "{name} ({images:?}): calls {:?}, expected {expected:?}; output {content:?}",
                seen.calls
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows differ:\n{}",
        failures.len(),
        IMAGE_LINK_CASES.len(),
        failures.join("\n")
    );
}

/// How an option takes a node out of the page.
#[derive(Debug, Clone, Copy)]
enum Removed {
    /// `strip_tags` names the tag: the element goes, its content stays.
    Stripped(&'static str),
    /// `exclude_selectors` names the tag: the element goes with its content.
    Excluded(&'static str),
}

/// Each row: a name, the node that an option removes, the calls in order for
/// [`FILTERED_LABEL_PAGE`]. The calls are those of a run before code wrote a link as its text.
const FILTERED_LABEL_CASES: &[(&str, Removed, &[&str])] = &[
    ("the image is stripped", Removed::Stripped("img"), &["code_block"]),
    ("the image is excluded", Removed::Excluded("img"), &["code_block"]),
    (
        "the wrapper of the image is excluded",
        Removed::Excluded("span"),
        &["code_block"],
    ),
    (
        "the wrapper of the image is stripped",
        Removed::Stripped("span"),
        &["image i.png", "link #x", "code_block"],
    ),
];

const FILTERED_LABEL_PAGE: &str = "<pre><a href=\"#x\"><span><img src=\"i.png\"></span></a>z</pre>";

#[test]
fn a_link_in_code_whose_image_the_options_remove_gets_the_same_calls_as_before() {
    let mut failures = Vec::new();
    for (name, removed, expected) in FILTERED_LABEL_CASES {
        let mut options = ConversionOptions::default();
        match removed {
            Removed::Stripped(tag) => options.strip_tags.push((*tag).to_string()),
            Removed::Excluded(selector) => options.exclude_selectors.push((*selector).to_string()),
        }
        let (seen, content) = record_with(FILTERED_LABEL_PAGE, options);
        if seen.calls != *expected {
            failures.push(format!(
                "{name}: calls {:?}, expected {expected:?}; output {content:?}",
                seen.calls
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows differ:\n{}",
        failures.len(),
        FILTERED_LABEL_CASES.len(),
        failures.join("\n")
    );
}

#[test]
fn an_anchor_that_gets_no_call_still_writes_its_text_in_code() {
    for (html, code, output) in [
        ("<pre>a<a href=\"#x\"> </a>b</pre>", "a b", "```\na b\n```\n"),
        (
            "<pre>see <a href=\"https://e.org/x\">https://e.org/x</a> end</pre>",
            "see https://e.org/x end",
            "```\nsee https://e.org/x end\n```\n",
        ),
        (
            "<pre><a id=\"l1\" href=\"#l1\"></a>a = 1\n<a id=\"l2\" href=\"#l2\"></a>  b = 2\n</pre>",
            "a = 1\n  b = 2\n",
            "```\na = 1\n  b = 2\n```\n",
        ),
    ] {
        let (seen, content) = record(html, CodeBlockStyle::Backticks);
        assert_eq!(seen.code, [code], "the code argument for {html:?}");
        assert_eq!(content, output, "the output for {html:?}");
    }
}

#[test]
fn the_text_argument_of_a_link_in_code_is_the_text_the_code_shows() {
    let (seen, content) = record(
        "<pre>a <a href=\"/x\"><mark>hot</mark> link</a> b</pre>",
        CodeBlockStyle::Backticks,
    );
    assert_eq!(seen.link_texts, ["hot link"]);
    assert_eq!(seen.code, ["a hot link b"], "the code argument holds no link marks");
    assert_eq!(content, "```\na hot link b\n```\n");
}

#[test]
fn a_link_in_code_that_gets_no_call_is_written_the_same_without_a_visitor() {
    for (name, html, _) in CASES {
        let (_, with_visitor) = record(html, CodeBlockStyle::Backticks);
        let without = convert(
            html,
            Some(ConversionOptions {
                extract_metadata: false,
                ..ConversionOptions::default()
            }),
        )
        .expect("the conversion succeeds")
        .content
        .unwrap_or_default();
        assert_eq!(
            with_visitor, without,
            "{name}: a visitor that continues changes no output"
        );
    }
}
