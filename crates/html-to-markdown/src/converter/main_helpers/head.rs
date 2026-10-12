use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::options::ConversionOptions;
use crate::rcdom::{Handle, NodeData};

/// Format metadata as YAML frontmatter.
///
/// ~keep The `title` key is present whenever a title element was seen, even an empty one, so
/// ~keep `extract_document_metadata` can tell "no title element" from "an empty one" (#527). An
/// ~keep empty title carries no frontmatter line either way, so it is the one key skipped here.
///
/// Keys and values come from the page, so each is written as one YAML scalar (#544): a
/// newline, `: ` or a leading indicator would otherwise end the line or change what YAML reads.
///
/// `None` when no key writes a line: a page whose only metadata is an empty title gets no
/// frontmatter block.
pub fn format_metadata_frontmatter(metadata: &BTreeMap<String, String>) -> Option<String> {
    const OPEN: &str = "---\n";
    let mut result = String::from(OPEN);
    for (key, value) in metadata {
        if key == "title" && value.is_empty() {
            continue;
        }
        push_yaml_scalar(&mut result, key);
        result.push_str(": ");
        push_yaml_scalar(&mut result, value);
        result.push('\n');
    }
    if result.len() == OPEN.len() {
        return None;
    }
    result.push_str("---\n");
    Some(result)
}

/// Append `value` as a plain YAML scalar when it reads back unchanged, else as a double-quoted
/// scalar with every non-printable character escaped.
fn push_yaml_scalar(out: &mut String, value: &str) {
    if is_plain_yaml_scalar(value) {
        out.push_str(value);
        return;
    }
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if is_yaml_printable(c) => out.push(c),
            c => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
        }
    }
    out.push('"');
}

/// A conservative subset of the YAML plain scalar in block context: no leading indicator, no
/// `: ` or ` #`, no leading or trailing space, only printable characters, and a value a YAML
/// reader resolves to a string (#552).
fn is_plain_yaml_scalar(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    !matches!(
        first,
        '-' | '?'
            | ':'
            | ','
            | '['
            | ']'
            | '{'
            | '}'
            | '#'
            | '&'
            | '*'
            | '!'
            | '|'
            | '>'
            | '\''
            | '"'
            | '%'
            | '@'
            | '`'
    ) && !value.starts_with(' ')
        && !value.ends_with([' ', ':'])
        && !value.contains(": ")
        && !value.contains(" #")
        && value.chars().all(|c| c != '\t' && is_yaml_printable(c))
        && !yaml_non_string_scalar().is_match(value)
}

/// Matches a plain scalar that a YAML reader resolves to null, a boolean, a number or a timestamp:
/// the YAML 1.2 core schema, plus the YAML 1.1 forms that readers such as PyYAML still apply
/// (`yes`/`no`/`on`/`off`, `0b`, leading-zero octal, `_` separators, base 60, dates, `=`, `<<`).
fn yaml_non_string_scalar() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(concat!(
            r"^(?:~|null|Null|NULL",
            r"|true|True|TRUE|false|False|FALSE|yes|Yes|YES|no|No|NO|on|On|ON|off|Off|OFF|y|Y|n|N",
            r"|[-+]?(?:0b[01_]+|0o[0-7]+|0x[0-9a-fA-F_]+",
            r"|[0-9][0-9_]*(?::[0-5]?[0-9])*(?:\.[0-9_]*)?(?:[eE][-+]?[0-9]+)?",
            r"|\.[0-9][0-9_]*(?:[eE][-+]?[0-9]+)?|\.(?:inf|Inf|INF))",
            r"|\.(?:nan|NaN|NAN)",
            r"|[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}",
            r"(?:(?:[Tt]|[ \t]+)[0-9]{1,2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]*)?(?:[ \t]*(?:Z|[-+][0-9]{1,2}(?::[0-9]{2})?))?)?",
            r"|=|<<)$",
        ))
        .expect("YAML scalar type regex is well-formed")
    })
}

/// The YAML 1.2 printable set, minus the Unicode line and paragraph separators and the
/// byte-order mark, which YAML 1.1 readers still treat as a line break or a document marker.
const fn is_yaml_printable(c: char) -> bool {
    matches!(c, '\t' | ' '..='~' | '\u{A0}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')
        && !matches!(c, '\u{2028}' | '\u{2029}' | '\u{FEFF}')
}

/// What the HTML tree builder says about the head of a page. Both converters read it, and
/// neither reads the head of the page by itself.
#[derive(Debug, Default)]
pub struct HeadFacts {
    /// The page that the converters read: the doctype, the `html` element and the head as the
    /// tree builder built them, then the body of the input as the input wrote it.
    pub page: String,
    /// The title, the `meta` values and the canonical link of the page. The first one in tree
    /// order wins. The `title` key is there for an empty title too (#527).
    pub metadata: BTreeMap<String, String>,
    /// The `href` of the first `<base>` in the tree of the start of the page.
    tree_base_href: Option<String>,
    /// The `lang` of the `html` element, or else of the `body` element.
    pub language: Option<String>,
    /// The `dir` of the `html` element, or else of the `body` element.
    pub direction: Option<String>,
    /// The limit that stopped the tree builder, if it reached one.
    pub limit: Option<crate::types::WarningKind>,
    /// True when the tree builder read the whole input and `page` is its tree.
    pub whole_page: bool,
}

impl HeadFacts {
    /// The `href` of the first `<base>` of the document `input`, which these facts are of.
    ///
    /// ~keep A `<base>` can stand in the body. The tree of the start of the page does not hold
    /// ~keep that one, so the parse of the base goes on into the body for it.
    pub fn base_href(&self, input: &str) -> Option<String> {
        if self.tree_base_href.is_some() || self.whole_page {
            return self.tree_base_href.clone();
        }
        crate::converter::url_resolve::document_base_href(input)
    }

    /// The metadata of the page for `options`, with `base` as the `base` key.
    pub fn metadata_for(&self, options: &ConversionOptions, base: Option<&str>) -> BTreeMap<String, String> {
        let excluded = |name: &str| {
            options.strip_tags.iter().any(|tag| tag == name) || options.preserve_tags.iter().any(|tag| tag == name)
        };
        let (no_meta, no_title) = (excluded("meta"), excluded("title"));
        let mut metadata: BTreeMap<String, String> = self
            .metadata
            .iter()
            .filter(|(key, _)| !(no_title && *key == "title") && !(no_meta && key.starts_with("meta-")))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        if let Some(base) = base {
            metadata.insert("base".to_owned(), base.to_owned());
        }
        metadata
    }
}

/// Read the head of `input` with the HTML tree builder.
///
/// The tree builder reads the page to the start of its body. The result holds what its tree
/// says, and the page for the converters: that tree, then the rest of the input as written.
///
/// ~keep Input with no tag, no comment and no doctype is text and stays as written. At the
/// ~keep node budget the page is the input as written (issue #809). At the depth limit the
/// ~keep input after the last tag of the tree follows the tree (issue #808).
pub fn read_head(input: &str) -> HeadFacts {
    use crate::converter::anchor_origin::{HeadTree, RepairTree, collapse_split_anchors, parse_to_body_start};

    let mut facts = HeadFacts::default();
    let HeadTree { tree, body_start } = parse_to_body_start(input, |document| read_tree(document, &mut facts));
    let RepairTree {
        dom,
        limit,
        unparsed,
        saw_markup,
    } = tree;
    facts.limit = limit;
    // ~keep A page that the tree builder gave no body (a frameset page) stays as written too:
    // ~keep the tree builder drops what follows `</frameset>`, and no text may be lost.
    let no_body = body_start.is_none() && !dom.made_body.get();
    if !saw_markup || no_body || limit == Some(crate::types::WarningKind::TruncatedInput) {
        input.clone_into(&mut facts.page);
        return facts;
    }
    collapse_split_anchors(&dom.document);
    let Some(mut page) = super::serialize_document(dom.document, input.len()) else {
        input.clone_into(&mut facts.page);
        return facts;
    };
    // ~keep The tree ends with the end tags of the `html` and `body` elements that the input
    // ~keep wrote a start tag for. The rest of the input goes in front of them, or takes their
    // ~keep place when the parse stopped at the body: the rest then holds the end tags.
    write_cdata_as_the_source(&mut page);
    let content_end = ["</html>", "</body>"]
        .iter()
        .fold(page.len(), |end, tag| end - page[..end].strip_suffix(tag).map_or(0, |_| tag.len()));
    if let Some(tag) = body_start {
        page.truncate(content_end);
        page.push_str(&tag);
        page.push_str(&unparsed);
    } else {
        page.insert_str(content_end, &unparsed);
        facts.whole_page = limit.is_none();
    }
    facts.page = page;
    facts
}

/// Write each comment of `page` that the source wrote as `<![CDATA[..]]>` in that form again.
///
/// ~keep Outside SVG and MathML the tokenizer gives a CDATA section as a comment with the text
/// ~keep `[CDATA[..]]`. The converters read the section as the source wrote it (the text of a
/// ~keep CommonMark HTML block), so the tree of the head must not turn it into a comment.
fn write_cdata_as_the_source(page: &mut String) {
    const OPEN: &str = "<!--[CDATA[";
    const CLOSE: &str = "]]-->";
    let mut from = 0;
    while let Some(start) = page[from..].find(OPEN).map(|at| from + at) {
        let Some(end) = page[start..].find("-->").map(|at| start + at + "-->".len()) else {
            return;
        };
        if page[start..end].ends_with(CLOSE) {
            page.replace_range(end - CLOSE.len()..end, "]]>");
            page.replace_range(start..start + OPEN.len(), "<![CDATA[");
        }
        from = start + OPEN.len() - 2;
    }
}

/// The value of the attribute `name` of an element.
fn attribute(attributes: &[html5ever::Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| &*attribute.name.local == name)
        .map(|attribute| attribute.value.to_string())
}

/// Read `document`, the tree of the start of a page, into `facts`, and take each `title` that is
/// a child of the body out of the body: the title of a document is no text of its body.
///
/// ~keep The tree holds the head, and body content only when the body began inside the
/// ~keep `<head>` of the source. Template contents are no children, and a foreign element
/// ~keep (SVG, MathML) is not read.
fn read_tree(document: &Handle, facts: &mut HeadFacts) {
    let mut seen_meta = HashSet::new();
    let mut titles = Vec::new();
    let mut body = None;
    let mut work = vec![Rc::clone(document)];
    while let Some(node) = work.pop() {
        let descend = match &node.data {
            NodeData::Document => true,
            NodeData::Element { name, attrs, .. } if name.ns == html5ever::ns!(html) => {
                let attributes = attrs.borrow();
                match &*name.local {
                    "title" => {
                        let text: String = node
                            .children
                            .borrow()
                            .iter()
                            .filter_map(|child| match &child.data {
                                NodeData::Text { contents } => Some(contents.borrow().to_string()),
                                _ => None,
                            })
                            .collect();
                        let text = text.trim_matches(|character: char| character.is_ascii_whitespace());
                        facts
                            .metadata
                            .entry("title".to_owned())
                            .or_insert_with(|| text.to_owned());
                        titles.push(Rc::clone(&node));
                        false
                    }
                    "meta" => {
                        if let Some(content) = attribute(&attributes, "content") {
                            for key in ["name", "property"] {
                                if let Some(name) = attribute(&attributes, key) {
                                    let key = format!("meta-{name}");
                                    if seen_meta.insert(key.to_ascii_lowercase()) {
                                        facts.metadata.insert(key, content.clone());
                                    }
                                }
                            }
                        }
                        false
                    }
                    "link" => {
                        if attribute(&attributes, "rel").is_some_and(|rel| rel.contains("canonical"))
                            && let Some(href) = attribute(&attributes, "href")
                        {
                            facts.metadata.entry("canonical".to_owned()).or_insert(href);
                        }
                        false
                    }
                    "base" => {
                        if facts.tree_base_href.is_none() {
                            facts.tree_base_href = attribute(&attributes, "href");
                        }
                        false
                    }
                    // ~keep The text of a `noframes` or a `noscript` in the head is raw text that
                    // ~keep a browser shows nowhere. `tl` reads tags in it, and an open one would
                    // ~keep take the body into the head, so the page holds the element empty.
                    "head" => {
                        for child in node.children.borrow().iter() {
                            if matches!(&child.data, NodeData::Element { name, .. }
                                if name.ns == html5ever::ns!(html) && matches!(&*name.local, "noframes" | "noscript"))
                            {
                                child.children.borrow_mut().clear();
                            }
                        }
                        true
                    }
                    name @ ("html" | "body") => {
                        if facts.language.is_none() {
                            facts.language = attribute(&attributes, "lang");
                        }
                        if facts.direction.is_none() {
                            facts.direction = attribute(&attributes, "dir");
                        }
                        if name == "body" {
                            body = Some(Rc::clone(&node));
                        }
                        true
                    }
                    _ => true,
                }
            }
            _ => false,
        };
        if descend {
            work.extend(node.children.borrow().iter().rev().cloned());
        }
    }
    if let Some(body) = body {
        body.children
            .borrow_mut()
            .retain(|child| !titles.iter().any(|title| Rc::ptr_eq(title, child)));
    }
}

#[cfg(test)]
mod read_head_tests {
    use super::*;

    fn page(input: &str) -> String {
        read_head(input).page
    }

    fn metadata(input: &str) -> Vec<(String, String)> {
        read_head(input).metadata.into_iter().collect()
    }

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    #[test]
    fn should_put_head_content_with_no_head_tag_into_the_head() {
        let input = r#"<title>T</title><meta name="description" content="D"><link rel="canonical" href="https://e.org/c"><p>one</p>"#;
        assert_eq!(
            page(input),
            r#"<head><title>T</title><meta name="description" content="D"><link rel="canonical" href="https://e.org/c"></head><p>one</p>"#
        );
        assert_eq!(
            metadata(input),
            [
                pair("canonical", "https://e.org/c"),
                pair("meta-description", "D"),
                pair("title", "T")
            ]
        );
    }

    #[test]
    fn should_keep_the_body_as_the_input_wrote_it() {
        assert_eq!(
            page("<html lang=en><head><title>T</title></head><body dir=rtl><P>one<x-foo a='1&amp;\"'>U</x-foo></body></html>"),
            "<html lang=\"en\"><head><title>T</title></head><body dir=\"rtl\"><P>one<x-foo a='1&amp;\"'>U</x-foo></body></html>"
        );
        assert_eq!(page("text<p>one</p>"), "text<p>one</p>");
        let frameset = "<frameset><frame></frameset><div><p>one</p></div>";
        assert_eq!(page(frameset), frameset);
        assert_eq!(page("no tag &amp; no comment"), "no tag &amp; no comment");
        assert_eq!(page(""), "");
    }

    #[test]
    fn should_write_the_tag_that_starts_the_body_from_its_token() {
        assert_eq!(
            page("<title>T</title><X-Foo A='1&amp;\"2' b>U</x-foo>"),
            "<head><title>T</title></head><x-foo a=\"1&amp;&quot;2\" b=\"\">U</x-foo>"
        );
        assert_eq!(page("<title>T</title><br/>x"), "<head><title>T</title></head><br/>x");
        assert_eq!(page("<title>T</title></br>x"), "<head><title>T</title></head></br>x");
    }

    #[test]
    fn should_read_a_head_tag_in_a_comment_or_in_a_script_as_text() {
        assert_eq!(
            page("<!-- <head> --><title>T</title><p>one</p>"),
            "<!-- <head> --><head><title>T</title></head><p>one</p>"
        );
        let input = r#"text<script>var h="<head>";</script><title>T</title>"#;
        assert_eq!(page(input), input);
        assert_eq!(metadata(input), []);
    }

    #[test]
    fn should_write_a_cdata_section_as_the_source_wrote_it() {
        assert_eq!(page("<![CDATA[x]]>"), "<![CDATA[x]]>");
        assert_eq!(
            page("<!-- c]] --><![CDATA[x]]><title>T</title><p>one</p>"),
            "<!-- c]] --><![CDATA[x]]><head><title>T</title></head><p>one</p>"
        );
    }

    #[test]
    fn should_end_the_head_at_the_end_tag_of_the_source_when_the_body_began_inside_it() {
        let input = "<HEAD><IMG SRC=i.png><TITLE>T</TITLE><meta name=k content=v></HEAD\n><p>one</p>";
        assert_eq!(
            page(input),
            "<head></head><img src=\"i.png\"><meta name=\"k\" content=\"v\"><p>one</p>"
        );
        assert_eq!(metadata(input), [pair("meta-k", "v"), pair("title", "T")]);
    }

    #[test]
    fn should_read_a_page_with_a_head_and_no_head_end_tag_to_its_end() {
        let facts = read_head("<head><title>T</title><div><p>one</div>tail");
        assert!(facts.whole_page);
        assert_eq!(facts.page, "<head><title>T</title></head><div><p>one</p></div>tail");
        assert!(!read_head("<title>T</title><p>one").whole_page);
    }

    #[test]
    fn should_read_the_language_the_direction_and_the_first_base_from_the_tree() {
        let input = "<html lang=en><base target=_blank><base href=/a/><base href=/b/><body dir=rtl lang=fr><p>one";
        let facts = read_head(input);
        assert_eq!(facts.language.as_deref(), Some("en"));
        assert_eq!(facts.direction.as_deref(), Some("rtl"));
        assert_eq!(facts.base_href(input).as_deref(), Some("/a/"));
        let in_body = "<p>one</p><base href=/late/>";
        assert_eq!(read_head(in_body).base_href(in_body).as_deref(), Some("/late/"));
    }

    #[test]
    fn should_leave_out_the_title_and_the_meta_keys_that_the_options_exclude() {
        let facts = read_head("<title>T</title><meta name=a content=1><meta property=A content=2><p>one");
        let all = facts.metadata_for(&ConversionOptions::default(), Some("https://b.org/"));
        assert_eq!(
            all.into_iter().collect::<Vec<_>>(),
            [pair("base", "https://b.org/"), pair("meta-a", "1"), pair("title", "T")]
        );
        let options = ConversionOptions {
            strip_tags: vec!["meta".to_owned()],
            preserve_tags: vec!["title".to_owned()],
            ..ConversionOptions::default()
        };
        assert!(facts.metadata_for(&options, None).is_empty());
    }

    #[test]
    fn should_put_the_rest_of_the_input_after_the_tree_at_the_depth_limit() {
        let deep = format!("<head><title>T</title>{}x", "<div>".repeat(600));
        let facts = read_head(&deep);
        assert_eq!(facts.limit, Some(crate::types::WarningKind::DepthLimitExceeded));
        assert_eq!(facts.page.matches("<div>").count(), 600);
        assert!(facts.page.ends_with("<div>x"), "{}", &facts.page[facts.page.len() - 40..]);
    }
}
