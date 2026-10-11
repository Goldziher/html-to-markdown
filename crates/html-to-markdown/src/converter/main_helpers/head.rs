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

/// The title, meta and canonical link fields of the head below `roots`: the children of the
/// [`document_head`], then the head content that follows it before the body starts.
///
/// ~keep A browser's parser puts a `meta` or a `link` that is before the body into the head,
/// ~keep with a `<head>` tag on the page or not (issue #849). The two are void elements, so
/// ~keep this tree holds them as the source has them and the page needs no rebuild.
fn head_element_metadata(
    roots: &[tl::NodeHandle],
    parser: &tl::Parser,
    options: &ConversionOptions,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let search = document_head_search(roots, parser);
    let mut elements = Vec::new();
    if let Some(tl::Node::Tag(head)) = search.head.and_then(|handle| handle.get(parser)) {
        elements.extend(head.children().top().iter().copied());
    }
    elements.extend(search.outside_head);
    let mut seen_meta = HashSet::new();
    let mut seen_title = false;
    for child_handle in &elements {
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
struct HeadSearch {
    /// The `<head>` element.
    head: Option<tl::NodeHandle>,
    /// The head content before the body that is in no `<head>` element, in source order.
    outside_head: Vec<tl::NodeHandle>,
    /// True when a browser's parser builds another page than this tree holds. Head content is
    /// in front of the `<head>` tag (issue #592). A `title` is before the body and in no
    /// `<head>` element, with a `<head>` tag on the page or not (issue #816). The `<head>`
    /// element holds content of the body in front of its end tag (issue #856).
    content_outside_head: bool,
}

/// Walks from the start of the document to the start of the body. The walk does not stop at
/// the `<head>` element: a `title` between `</head>` and the body is in the head for a browser.
fn document_head_search(roots: &[tl::NodeHandle], parser: &tl::Parser) -> HeadSearch {
    let mut head = None;
    let mut outside_head = Vec::new();
    let mut content_outside_head = false;
    let mut saw_head_content = false;
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
                            content_outside_head |= saw_head_content || ends_after_body_content(tag, parser);
                        }
                    }
                    b"html" => {
                        let first = work.len();
                        work.extend(tag.children().top().iter().copied());
                        work[first..].reverse();
                    }
                    name if starts_body(name) => break,
                    // ~keep This tree holds the text of a `title` in the body until the page is
                    // ~keep rebuilt, so the readers of the head do not read it here (issue #809).
                    b"title" => {
                        saw_head_content = true;
                        content_outside_head = true;
                    }
                    _ => {
                        saw_head_content = true;
                        outside_head.push(handle);
                    }
                }
            }
            _ => {}
        }
    }
    HeadSearch {
        head,
        outside_head,
        content_outside_head,
    }
}

/// True when the source ends `head` with an end tag after content of the body: a child that
/// [`starts_body`], or text that is not white space. A browser's parser ends the head in front
/// of that content (issue #856).
///
/// ~keep A `<head>` with no end tag holds the rest of the page in this tree. The converter
/// ~keep gives that page to the HTML tree builder itself (issue #772).
fn ends_after_body_content(head: &tl::HTMLTag, parser: &tl::Parser) -> bool {
    crate::converter::preprocessing_helpers::has_end_tag(head)
        && head.children().top().iter().any(|child| match child.get(parser) {
            Some(tl::Node::Tag(tag)) => starts_body(&tag.name().as_bytes().to_ascii_lowercase()),
            Some(tl::Node::Raw(text)) => !is_ignorable_before_head(&text.as_utf8_str(), false),
            _ => false,
        })
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

/// Rebuild a document whose head and body a browser's parser builds from other content than
/// the source nesting holds: head content precedes the `<head>` tag, a `<title>` precedes the
/// body and is in no `<head>` element, or the `<head>` element holds content of the body in
/// front of its end tag.
///
/// The HTML tree builder places that content in an implicit head and ignores a later head
/// start tag. The lightweight parser preserves source nesting, so its tree needs this targeted
/// repair before either conversion tier reads metadata or renders the body.
///
/// ~keep A `<meta>` or a `<link>` outside the `<head>` element starts no rebuild: the readers
/// ~keep of the head read it where it is (issue #849).
pub fn repair_head_content_outside_head(input: &str) -> Option<(String, Option<crate::types::WarningKind>)> {
    if !may_hold_head_content_outside_head(input.as_bytes()) {
        return None;
    }
    let dom = tl::parse(input, tl::ParserOptions::default()).ok()?;
    let search = document_head_search(dom.children(), dom.parser());
    if !search.content_outside_head {
        return None;
    }
    let page = match search.head.and_then(|handle| handle.get(dom.parser())) {
        Some(tl::Node::Tag(head)) => with_head_metadata_first(input, head, dom.parser()),
        _ => None,
    };
    super::repair_with_html5ever_bounded(page.as_deref().unwrap_or(input))
}

/// The byte range of `part` in `input`, when `part` is a slice of it.
fn source_range(input: &str, part: &[u8]) -> Option<std::ops::Range<usize>> {
    let start = (part.as_ptr() as usize).checked_sub(input.as_ptr() as usize)?;
    let end = start.checked_add(part.len())?;
    (end <= input.len()).then_some(start..end)
}

/// The source with the metadata elements of `head` in front of the first body content of that
/// head. `None` when the head has no end tag, or no metadata element after body content.
///
/// ~keep The tree builder ends the head at body content, so a `title`, `meta`, `link` or `base`
/// ~keep that the source has after it in the head goes into the body. For a browser it is
/// ~keep metadata of the document all the same: the title of a document is its first `title`
/// ~keep element. Only a direct child of the head moves: a `title` in an `svg` is no page title.
fn with_head_metadata_first(input: &str, head: &tl::HTMLTag, parser: &tl::Parser) -> Option<String> {
    if !crate::converter::preprocessing_helpers::has_end_tag(head) {
        return None;
    }
    let mut body_start = None;
    let mut moved = Vec::new();
    for child in head.children().top().iter() {
        match child.get(parser) {
            Some(tl::Node::Tag(tag)) => {
                let name = tag.name().as_bytes().to_ascii_lowercase();
                let range = source_range(input, tag.raw().as_bytes())?;
                if starts_body(&name) {
                    body_start = body_start.or(Some(range.start));
                } else if body_start.is_some() && matches!(name.as_slice(), b"title" | b"meta" | b"link" | b"base") {
                    moved.push(range);
                }
            }
            Some(tl::Node::Raw(text)) if !is_ignorable_before_head(&text.as_utf8_str(), false) => {
                body_start = body_start.or(Some(source_range(input, text.as_bytes())?.start));
            }
            _ => {}
        }
    }
    let body_start = body_start?;
    if moved.is_empty() {
        return None;
    }
    let mut page = String::with_capacity(input.len());
    page.push_str(input.get(..body_start)?);
    for range in &moved {
        page.push_str(input.get(range.clone())?);
    }
    let mut rest = body_start;
    for range in &moved {
        page.push_str(input.get(rest..range.start)?);
        rest = range.end;
    }
    page.push_str(input.get(rest..)?);
    Some(page)
}

/// A cheap check of the source in front of the parse of [`repair_head_content_outside_head`];
/// the parsed tree decides.
///
/// ~keep One read of the tags from the start of the source to the tag that starts the body. A
/// ~keep comment, a CDATA section and the text of a `<script>` or a `<style>` hold no tag (issue
/// ~keep #848). The markup in a `<title>`, a `<noscript>`, a `<noframes>` or a `<template>` is
/// ~keep the content of that element, not of the head.
fn may_hold_head_content_outside_head(bytes: &[u8]) -> bool {
    use crate::converter::utility::preprocessing::{find_closing_tag_bytes, find_tag_end, skip_opaque_region};

    let is_text = |range: std::ops::Range<usize>| bytes[range].iter().any(|byte| !byte.is_ascii_whitespace());
    // ~keep Body content in a `<head>` asks for the rebuild only when the head has its end tag
    // ~keep (issue #856). The converter rebuilds a head with no end tag itself.
    let head_ends_after = |from: usize| find_closing_tag_bytes(bytes, from, b"head").is_some();
    let mut in_head = false;
    let mut saw_head = false;
    let mut saw_head_content = false;
    let mut text_start = 0;
    let mut idx = 0;
    while let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) {
        idx += offset;
        let is_end_tag = bytes.get(idx + 1) == Some(&b'/');
        let name_start = idx + 1 + usize::from(is_end_tag);
        let is_tag = bytes.get(name_start).is_some_and(u8::is_ascii_alphabetic);
        let is_declaration = !is_end_tag && matches!(bytes.get(idx + 1), Some(b'!' | b'?'));
        if !is_tag && !is_declaration {
            // ~keep A `<` that starts no tag is text.
            idx += 1;
            continue;
        }
        if in_head && is_text(text_start..idx) {
            return head_ends_after(idx);
        }
        let name_end = super::scan_tag_name_end(bytes, name_start);
        let name = bytes[name_start..name_end].to_ascii_lowercase();
        let Some(tag_end) = skip_opaque_region(bytes, idx).or_else(|| find_tag_end(bytes, name_end)) else {
            return false;
        };
        idx = tag_end;
        text_start = tag_end;
        if is_end_tag {
            in_head &= name != b"head";
        }
        if is_end_tag || is_declaration {
            continue;
        }
        match name.as_slice() {
            b"html" => {}
            b"head" => {
                if !saw_head {
                    if saw_head_content {
                        return true;
                    }
                    saw_head = true;
                    in_head = true;
                }
            }
            name if starts_body(name) => return in_head && head_ends_after(idx),
            name => {
                if !in_head && name == b"title" {
                    return true;
                }
                saw_head_content = true;
                if matches!(name, b"title" | b"noscript" | b"noframes" | b"template") {
                    idx = find_closing_tag_bytes(bytes, tag_end, name).unwrap_or(bytes.len());
                    text_start = idx;
                }
            }
        }
    }
    // ~keep The source ended in the head or before the body: no head has an end tag after body content.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rebuilds_the_head(html: &str) -> bool {
        repair_head_content_outside_head(html).is_some()
    }

    #[test]
    fn should_not_rebuild_a_page_for_a_metadata_element_outside_the_head() {
        for html in [
            r#"<meta name="description" content="D"><p>one</p>"#,
            r#"<link rel="canonical" href="https://e.org/"><p>one</p>"#,
            r#"<head></head><meta name="description" content="D"><p>one</p>"#,
            r#"<meta charset="utf-8"><p>one</p>"#,
            r#"<meta name="description"><p>one</p>"#,
            r#"<link rel="stylesheet" href="s.css"><p>one</p>"#,
            "<style>p{}</style><p>one</p>",
            "<script>var a;</script><p>one</p>",
            r#"<p>one</p><meta name="description" content="D">"#,
        ] {
            assert!(!rebuilds_the_head(html), "{html:?}");
        }
    }

    #[test]
    fn should_put_the_metadata_of_a_head_in_front_of_its_body_content() {
        let page = |html: &str| {
            let dom = tl::parse(html, tl::ParserOptions::default()).expect("a tree");
            let head = document_head(dom.children(), dom.parser()).expect("a head");
            let Some(tl::Node::Tag(head)) = head.get(dom.parser()) else {
                panic!("the head is a tag");
            };
            with_head_metadata_first(html, head, dom.parser())
        };
        assert_eq!(
            page("<head><o:p></o:p><meta name=a content=b>x<title>T</title></head><p>one").as_deref(),
            Some("<head><meta name=a content=b><title>T</title><o:p></o:p>x</head><p>one")
        );
        assert_eq!(page("<head><title>T</title><img src=i.png></head>"), None);
        assert_eq!(page("<head><svg><title>X</title></svg></head>"), None);
        assert_eq!(page("<head><img src=i.png><title>T</title><p>one"), None);
    }

    #[test]
    fn should_rebuild_a_page_whose_head_ends_after_body_content() {
        assert!(rebuilds_the_head("<head><img src=\"p.gif\"></head><p>End</p>"));
        assert!(rebuilds_the_head("<head>text</head><p>End</p>"));
        for html in [
            "<head><title>T</title></head><p>End</p>",
            "<head><title>T</title><noscript><img src=\"p.gif\"></noscript></head><p>End</p>",
            "<head><!-- <img> --><title>T</title></head><p>End</p>",
            // ~keep The converter gives a head with no end tag to the tree builder itself.
            "<head><title>T</title><p>End</p>",
        ] {
            assert!(!rebuilds_the_head(html), "{html:?}");
        }
    }

    #[test]
    fn should_read_no_tag_in_a_comment_or_in_the_content_of_a_head_element() {
        let may_hold = |html: &str| may_hold_head_content_outside_head(html.as_bytes());
        for html in [
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>a <b>x</b></title></head><body><p>one</p>",
            "<head><noscript><img src=\"p.gif\"></noscript><template><p>x</p></template></head><body><title>T</title>",
            "<head><script>var a = \"<p>\";</script><style>p > b {}</style><!-- <img> --></head><p>one</p>",
            "<!-- <title>X</title> --><p>one</p><title>T</title>",
            "<header>h</header><title>T</title>",
            "<head><title>T</title><img src=\"p.gif\"><p>one</p>",
            "<head><title>T</title>text<p>one</p>",
        ] {
            assert!(!may_hold(html), "{html:?}");
        }
        for html in [
            "<!-- <head> --><title>T</title><p>one</p>",
            "<head></head><!-- <body> --><title>T</title><p>one</p>",
            "<script>var a;</script><head></head>",
            "<head><title>T</title><img src=\"p.gif\"></head>",
            "<head>text</head>",
        ] {
            assert!(may_hold(html), "{html:?}");
        }
    }
}
