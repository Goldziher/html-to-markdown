#![allow(missing_docs)]
#![cfg(feature = "visitor")]
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use html_to_markdown_rs::{ConversionOptions, convert};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct Calls {
    texts: Vec<String>,
    links: Vec<String>,
    images: Vec<String>,
}

#[derive(Debug)]
struct Recorder {
    calls: Arc<Mutex<Calls>>,
}

impl HtmlVisitor for Recorder {
    fn visit_text(&mut self, _ctx: &NodeContext, text: &str) -> VisitResult {
        self.calls.lock().expect("lock").texts.push(text.to_owned());
        VisitResult::Continue
    }

    fn visit_link(&mut self, _ctx: &NodeContext, href: &str, _text: &str, _title: Option<&str>) -> VisitResult {
        self.calls.lock().expect("lock").links.push(href.to_owned());
        VisitResult::Continue
    }

    fn visit_image(&mut self, _ctx: &NodeContext, src: &str, _alt: &str, _title: Option<&str>) -> VisitResult {
        self.calls.lock().expect("lock").images.push(src.to_owned());
        VisitResult::Continue
    }
}

fn record(html: &str) -> (String, Calls) {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let options = ConversionOptions {
        visitor: Some(Arc::new(Mutex::new(Recorder {
            calls: Arc::clone(&calls),
        }))),
        ..ConversionOptions::default()
    };
    let output = convert(html, Some(options))
        .expect("conversion succeeds")
        .content
        .unwrap_or_default();
    let recorded = std::mem::take(&mut *calls.lock().expect("lock"));
    (output, recorded)
}

// The visitor walk of a row and the render pass each call the visitor for a cell with markup.
// The pass that measures the column widths calls no visitor method.
const WALKS_WITH_A_VISITOR: usize = 2;

#[test]
fn should_not_call_the_visitor_for_a_link_or_an_image_from_the_width_pass_of_a_table() {
    let html = "<table><tr><td><a href=\"/x\">l</a></td><td><img src=\"/i.png\" alt=\"pic\"></td></tr></table>";
    let (output, calls) = record(html);
    assert_eq!(
        output.lines().next(),
        Some("| [l](/x) | ![pic](/i.png) |"),
        "{output:?}"
    );
    assert_eq!(calls.links, vec!["/x"; WALKS_WITH_A_VISITOR]);
    assert_eq!(calls.images, vec!["/i.png"; WALKS_WITH_A_VISITOR]);
    assert_eq!(calls.texts, vec!["l"; WALKS_WITH_A_VISITOR]);
}

#[test]
fn should_not_call_the_visitor_for_a_text_from_the_width_pass_of_a_table() {
    let (output, calls) = record("<table><tr><td><p>only</p></td><td>z</td></tr></table>");
    assert!(output.contains("only") && output.contains("| z |"), "{output:?}");
    let only = calls.texts.iter().filter(|text| *text == "only").count();
    assert_eq!(only, WALKS_WITH_A_VISITOR, "{:?}", calls.texts);
}
