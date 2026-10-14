use super::extract_plain_text;
use crate::options::{ConversionOptions, OutputFormat};

#[test]
fn should_walk_deep_plain_text_on_a_256_kib_stack() {
    // ~keep A subprocess contains the fatal stack-overflow regression in the original walker.
    const NAME: &str = "converter::plain_text::stack_tests::should_walk_deep_plain_text_on_a_256_kib_stack";
    if std::env::var_os("PLAIN_DIRECT_STACK_CHILD").is_some() {
        let text = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                for (open, close, depth, expected) in [
                    ("<div>", "</div>", 1000, "deep\n".to_owned()),
                    ("<span>", "</span>", 1000, "deep\n".to_owned()),
                    ("<em>", "</em>", 1000, "deep\n".to_owned()),
                    ("<ruby>", "</ruby>", 1000, "deep\n".to_owned()),
                    ("<code>", "</code>", 1000, "deep\n".to_owned()),
                    ("<output>", "</output>", 1000, "deep\n".to_owned()),
                    ("<a href='/x'>", "</a>", 1000, "deep\n".to_owned()),
                    ("<table><tr><td>", "</td></tr></table>", 300, "deep\n".to_owned()),
                    ("<ul><li>", "</li></ul>", 400, format!("{}- deep\n", "-\n".repeat(399))),
                ] {
                    let html = format!("{}deep{}", open.repeat(depth), close.repeat(depth));
                    let dom = tl::parse(&html, tl::ParserOptions::default()).expect("parse nested markup");
                    let options = ConversionOptions {
                        output_format: OutputFormat::Plain,
                        max_depth: Some(1024),
                        ..Default::default()
                    };
                    assert_eq!(extract_plain_text(&dom, dom.parser(), &options), expected, "{open}");
                }
                "deep\n".to_owned()
            })
            .expect("spawn small-stack thread")
            .join()
            .expect("join conversion");
        assert_eq!(text, "deep\n");
        return;
    }
    let child = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", NAME, "--nocapture"])
        .env("PLAIN_DIRECT_STACK_CHILD", "1")
        .output()
        .expect("execute child");
    assert!(child.status.success(), "{}", String::from_utf8_lossy(&child.stderr));
}

#[cfg(feature = "visitor")]
#[test]
fn should_keep_plain_capture_output_and_callbacks_at_deep_nesting() {
    use crate::visitor::{HtmlVisitor, NodeContext, VisitResult};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Recorder(Vec<(String, String)>);
    impl HtmlVisitor for Recorder {
        fn visit_element_end(&mut self, context: &NodeContext, text: &str) -> VisitResult {
            if context.tag_name != "span" {
                self.0.push((context.tag_name.to_string(), text.to_string()));
            }
            VisitResult::Continue
        }
    }

    let leaf = "<table><caption>C</caption><tr><td>x<button>y</button>z</td><td><select><option>A</option><option>B</option></select></td></tr></table><ol start='3'><li>one</li><li>two</li></ol>";
    let run = |html: &str| {
        let recorder = Arc::new(Mutex::new(Recorder::default()));
        let options = ConversionOptions {
            output_format: OutputFormat::Plain,
            max_depth: Some(1024),
            visitor: Some(recorder.clone()),
            ..Default::default()
        };
        let dom = tl::parse(html, tl::ParserOptions::default()).expect("parse captures");
        let text = extract_plain_text(&dom, dom.parser(), &options);
        let callbacks = recorder.lock().expect("callback recorder").0.clone();
        (text, callbacks)
    };
    let expected = run(leaf);
    assert_eq!(expected.0, "C\n\nx y z\tA B\n\n3. one\n4. two\n");
    assert_eq!(expected.1.len(), 9);
    let deep = format!("{}{leaf}{}", "<span>".repeat(1000), "</span>".repeat(1000));
    assert_eq!(run(&deep), expected);
}
