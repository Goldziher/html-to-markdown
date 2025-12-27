#![cfg(feature = "visitor")]

use html_to_markdown_rs::convert_with_visitor;
use html_to_markdown_rs::visitor::{HtmlVisitor, NodeContext, VisitResult};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct CodeVisitor {
    code_blocks: Vec<String>,
    inline_codes: Vec<String>,
}

impl HtmlVisitor for CodeVisitor {
    fn visit_code_block(&mut self, _ctx: &NodeContext, lang: Option<&str>, code: &str) -> VisitResult {
        let lang_str = lang.unwrap_or("unknown").to_string();
        self.code_blocks.push(format!("[{}] {}", lang_str, code.trim()));
        VisitResult::Custom(format!("```{}\n{}\n```", lang.unwrap_or(""), code))
    }

    fn visit_code_inline(&mut self, _ctx: &NodeContext, code: &str) -> VisitResult {
        self.inline_codes.push(code.to_string());
        VisitResult::Custom(format!("`{}`", code))
    }
}

#[test]
fn test_code_block_visitor() {
    let html = "<pre><code class=\"language-rust\">fn main() {}\n</code></pre>";
    let visitor = Rc::new(RefCell::new(CodeVisitor {
        code_blocks: vec![],
        inline_codes: vec![],
    }));

    let result = convert_with_visitor(html, None, Some(visitor.clone()));
    assert!(result.is_ok());

    let visitor_ref = visitor.borrow();
    assert_eq!(visitor_ref.code_blocks.len(), 1);
    assert!(visitor_ref.code_blocks[0].contains("rust"));
}

#[test]
fn test_inline_code_visitor() {
    let html = "<p>Use <code>println!</code> to print</p>";
    let visitor = Rc::new(RefCell::new(CodeVisitor {
        code_blocks: vec![],
        inline_codes: vec![],
    }));

    let result = convert_with_visitor(html, None, Some(visitor.clone()));
    assert!(result.is_ok());

    let visitor_ref = visitor.borrow();
    assert_eq!(visitor_ref.inline_codes.len(), 1);
    assert_eq!(visitor_ref.inline_codes[0], "println!");
}

#[test]
fn test_code_block_skip() {
    #[derive(Debug)]
    struct SkipCodeVisitor;

    impl HtmlVisitor for SkipCodeVisitor {
        fn visit_code_block(&mut self, _ctx: &NodeContext, _lang: Option<&str>, _code: &str) -> VisitResult {
            VisitResult::Skip
        }
    }

    let html = "<pre><code>skipped code</code></pre>";
    let visitor = Rc::new(RefCell::new(SkipCodeVisitor));

    let result = convert_with_visitor(html, None, Some(visitor));
    assert!(result.is_ok());
    let markdown = result.unwrap();
    assert!(!markdown.contains("skipped code"));
}

#[test]
fn test_code_block_language_detection() {
    let html_patterns = vec![
        (
            "<pre class=\"language-python\"><code>print('hi')</code></pre>",
            "python",
        ),
        (
            "<pre class=\"lang-javascript\"><code>console.log('hi')</code></pre>",
            "javascript",
        ),
        ("<pre><code>no language</code></pre>", "unknown"),
    ];

    for (html, expected_lang) in html_patterns {
        let visitor = Rc::new(RefCell::new(CodeVisitor {
            code_blocks: vec![],
            inline_codes: vec![],
        }));

        let result = convert_with_visitor(html, None, Some(visitor.clone()));
        assert!(result.is_ok(), "Failed to convert: {}", html);

        let visitor_ref = visitor.borrow();
        assert_eq!(visitor_ref.code_blocks.len(), 1);
        if expected_lang != "unknown" {
            assert!(visitor_ref.code_blocks[0].starts_with(&format!("[{}]", expected_lang)));
        }
    }
}
