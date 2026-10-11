use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use crate::options::ConversionOptions;

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

/// Record `<meta name>`/`<meta property>` content into `metadata`, honoring `strip_tags`/
/// `preserve_tags` for `"meta"`. The first tag per key wins, comparing keys in any letter case;
/// `seen` holds the lower-cased keys recorded so far.
fn collect_meta_head_metadata(
    child_tag: &tl::HTMLTag,
    options: &ConversionOptions,
    metadata: &mut BTreeMap<String, String>,
    seen: &mut HashSet<String>,
) {
    if !child_tag.name().as_utf8_str().eq_ignore_ascii_case("meta")
        || options.strip_tags.iter().any(|t| t == "meta")
        || options.preserve_tags.iter().any(|t| t == "meta")
    {
        return;
    }

    // ~keep `content` is a user-visible metadata value and carries character references like
    // ~keep any other attribute, so it is decoded before it reaches the frontmatter (#494).
    // ~keep The key (`name`/`property`) is an identifier, not prose, and stays raw.
    if let (Some(name), Some(content)) = (
        child_tag.attributes().get("name").flatten(),
        crate::converter::utility::attributes::decoded_attribute(child_tag, "content"),
    ) {
        let key = format!("meta-{}", name.as_utf8_str());
        if seen.insert(key.to_ascii_lowercase()) {
            metadata.insert(key, content.into_owned());
        }
    }
    if let (Some(property), Some(content)) = (
        child_tag.attributes().get("property").flatten(),
        crate::converter::utility::attributes::decoded_attribute(child_tag, "content"),
    ) {
        let key = format!("meta-{}", property.as_utf8_str());
        if seen.insert(key.to_ascii_lowercase()) {
            metadata.insert(key, content.into_owned());
        }
    }
}

/// Record the trimmed, decoded `<title>` text into `metadata`, honoring `strip_tags`/
/// `preserve_tags` for `"title"`. The first title wins, as in a browser; an empty one still
/// records the `title` key, empty, so the element's presence survives even though its text does
/// not (#527). `seen_title` is whether a title came before.
fn collect_title_head_metadata(
    child_tag: &tl::HTMLTag,
    parser: &tl::Parser,
    options: &ConversionOptions,
    metadata: &mut BTreeMap<String, String>,
    seen_title: &mut bool,
) {
    if !child_tag.name().as_utf8_str().eq_ignore_ascii_case("title") {
        return;
    }
    let later_title = std::mem::replace(seen_title, true);
    if later_title
        || options.strip_tags.iter().any(|t| t == "title")
        || options.preserve_tags.iter().any(|t| t == "title")
    {
        return;
    }

    let mut title_content = String::new();
    let title_children = child_tag.children();
    for title_child in title_children.top().iter() {
        if let Some(tl::Node::Raw(raw)) = title_child.get(parser) {
            title_content.push_str(raw.as_utf8_str().as_ref());
        }
    }
    // ~keep The title is text and carries character references like any other text (#509).
    let title_content = crate::text::decode_html_entities_cow(title_content.trim()).into_owned();
    metadata.insert("title".to_string(), title_content);
}

/// Record the decoded href of the first `<link rel="canonical">` into `metadata`.
fn collect_link_head_metadata(child_tag: &tl::HTMLTag, metadata: &mut BTreeMap<String, String>) {
    if !child_tag.name().as_utf8_str().eq_ignore_ascii_case("link") {
        return;
    }
    let Some(rel_attr) = child_tag.attributes().get("rel").flatten() else {
        return;
    };
    let rel_str = rel_attr.as_utf8_str();
    if !rel_str.contains("canonical") {
        return;
    }
    let Some(href) = crate::converter::utility::attributes::decoded_attribute(child_tag, "href") else {
        return;
    };
    metadata
        .entry("canonical".to_string())
        .or_insert_with(|| href.into_owned());
}

/// Extract metadata from the head element below `roots`, recording `document_base_href` as
/// `base` whether or not the source has a `<head>` tag.
pub fn extract_head_metadata(
    roots: &[tl::NodeHandle],
    parser: &tl::Parser,
    options: &ConversionOptions,
    document_base_href: Option<&str>,
) -> BTreeMap<String, String> {
    let mut metadata = head_element_metadata(roots, parser, options);
    if let Some(href) = document_base_href {
        metadata.insert("base".to_string(), href.to_string());
    }
    metadata
}

/// The title, meta and canonical link fields of the [`document_head`] below `roots`.
fn head_element_metadata(
    roots: &[tl::NodeHandle],
    parser: &tl::Parser,
    options: &ConversionOptions,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Some(tl::Node::Tag(head)) = document_head(roots, parser).and_then(|handle| handle.get(parser)) else {
        return metadata;
    };
    let mut seen_meta = HashSet::new();
    let mut seen_title = false;
    for child_handle in head.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            collect_meta_head_metadata(child_tag, options, &mut metadata, &mut seen_meta);
            collect_title_head_metadata(child_tag, parser, options, &mut metadata, &mut seen_title);
            collect_link_head_metadata(child_tag, &mut metadata);
        }
    }
    metadata
}

/// The first `<head>` element below `roots` before the body starts. A browser's parser ignores
/// a `<head>` tag once the body has started, at text or at a tag that [`starts_body`].
pub fn document_head(roots: &[tl::NodeHandle], parser: &tl::Parser) -> Option<tl::NodeHandle> {
    document_head_search(roots, parser).head
}

/// What [`document_head_search`] finds before the body starts.
pub(super) struct HeadSearch {
    /// The `<head>` element.
    head: Option<tl::NodeHandle>,
    /// True when a browser's parser puts an element in the head that this tree holds outside
    /// one: head content in front of the `<head>` tag, or a `title` before the body that is in
    /// no `<head>` element, with a `<head>` tag on the page or not (issue #816).
    pub(super) content_outside_head: bool,
}

/// Walks from the start of the document to the start of the body. The walk does not stop at
/// the `<head>` element: a `title` between `</head>` and the body is in the head for a browser.
pub(super) fn document_head_search(roots: &[tl::NodeHandle], parser: &tl::Parser) -> HeadSearch {
    let mut head = None;
    let mut content_before_head = false;
    let mut saw_head_content = false;
    let mut saw_title = false;
    let mut work: Vec<_> = roots.iter().rev().copied().collect();
    while let Some(handle) = work.pop() {
        match handle.get(parser) {
            Some(tl::Node::Raw(text)) => {
                let next_is_html = matches!(
                    work.last().and_then(|next| next.get(parser)),
                    Some(tl::Node::Tag(tag)) if tag.name().as_bytes().eq_ignore_ascii_case(b"html")
                );
                if !is_ignorable_before_head(&text.as_utf8_str(), next_is_html) {
                    break;
                }
            }
            Some(tl::Node::Tag(tag)) => {
                let name = tag.name().as_bytes().to_ascii_lowercase();
                match name.as_slice() {
                    b"head" => {
                        if head.is_none() {
                            head = Some(handle);
                            content_before_head = saw_head_content;
                        }
                    }
                    b"html" => {
                        let first = work.len();
                        work.extend(tag.children().top().iter().copied());
                        work[first..].reverse();
                    }
                    name if starts_body(name) => break,
                    name => {
                        saw_head_content = true;
                        saw_title |= name == b"title";
                    }
                }
            }
            _ => {}
        }
    }
    HeadSearch {
        head,
        content_outside_head: content_before_head || saw_title,
    }
}

/// Whether a run of text before the head is found should be skipped rather than ending the
/// search: whitespace after character-reference decoding, or any run that sits directly in
/// front of the document's own `<html>`
/// tag. The WHATWG "before html" insertion mode already discards anything ahead of `<html>`
/// itself without letting it block the parser from reaching the real head inside, whether that
/// text is a real byte order mark (stripped earlier, so it never reaches here), one a wrong
/// encoding guess mangled beyond recognition, or ordinary prose: a browser shows the page's
/// title and meta tags either way. Text with nothing named `html` ahead of it, a head-only
/// fragment, still ends the search unchanged; see
/// `should_ignore_a_head_after_implicit_body_content_on_both_tiers`.
pub fn is_ignorable_before_head(text: &str, next_tag_is_html: bool) -> bool {
    next_tag_is_html
        || crate::text::decode_html_entities_cow(text)
            .chars()
            .all(char::is_whitespace)
}

/// Whether a start tag named `name` (lower case) starts the body when no body has started:
/// every tag except the ones the HTML parser's "in head" insertion mode keeps in the head.
pub fn starts_body(name: &[u8]) -> bool {
    !matches!(
        name,
        b"html"
            | b"head"
            | b"base"
            | b"basefont"
            | b"bgsound"
            | b"link"
            | b"meta"
            | b"noframes"
            | b"noscript"
            | b"script"
            | b"style"
            | b"template"
            | b"title"
    )
}
