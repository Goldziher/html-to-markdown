//! Protect HTML raw-text and inert bodies from the literal DOM parser. ~keep

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use super::markup::find_tag_end;

use html5ever::LocalName;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};

struct Body {
    name: LocalName,
    start: usize,
    text: String,
    template_depth: usize,
    namespace_depth: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Namespace {
    Html,
    Svg,
    Math,
}

struct ForeignContext {
    name: LocalName,
    namespace: Namespace,
    children_namespace: Namespace,
}

#[derive(Default)]
struct NamespaceStack {
    contexts: Vec<ForeignContext>,
    indices: HashMap<LocalName, Vec<usize>>,
}

impl NamespaceStack {
    fn last(&self) -> Option<&ForeignContext> {
        self.contexts.last()
    }

    const fn len(&self) -> usize {
        self.contexts.len()
    }

    const fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    fn matching_index(&self, name: &LocalName) -> Option<usize> {
        #[cfg(test)]
        NAMESPACE_VISITS.with(|visits| visits.set(visits.get() + 1));
        self.indices.get(name).and_then(|indices| indices.last().copied())
    }

    fn push(&mut self, context: ForeignContext) {
        let index = self.len();
        self.indices.entry(context.name.clone()).or_default().push(index);
        self.contexts.push(context);
    }

    fn pop(&mut self) {
        if let Some(context) = self.contexts.pop() {
            #[cfg(test)]
            NAMESPACE_VISITS.with(|visits| visits.set(visits.get() + 1));
            if let Some(indices) = self.indices.get_mut(&context.name) {
                indices.pop();
                if indices.is_empty() {
                    self.indices.remove(&context.name);
                }
            }
        }
    }

    fn truncate(&mut self, length: usize) {
        while self.len() > length {
            self.pop();
        }
    }
}

#[derive(Default)]
struct PreservedBodies {
    originals: HashMap<String, String>,
    occupied: HashSet<usize>,
    next: usize,
}

const BODY_MARKER_PREFIX: &str = "h2m_inert_body_";

impl PreservedBodies {
    fn new(source: &str) -> Self {
        let decoded = crate::text::decode_html_entities_cow(source);
        let source = decoded.as_ref();
        let occupied = source
            .match_indices(BODY_MARKER_PREFIX)
            .filter_map(|(index, _)| {
                let rest = &source[index + BODY_MARKER_PREFIX.len()..];
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                rest.get(digits..).filter(|suffix| suffix.starts_with("_end"))?;
                rest[..digits].parse().ok()
            })
            .collect();
        Self {
            occupied,
            ..Self::default()
        }
    }

    fn protect(&mut self, body: &str) -> String {
        while self.occupied.contains(&self.next) {
            self.next += 1;
        }
        let marker = format!("{BODY_MARKER_PREFIX}{}_end", self.next);
        self.next += 1;
        self.originals.insert(marker.clone(), body.to_owned());
        marker
    }
}

struct BodyNormalizer<'a> {
    source: &'a str,
    input: &'a BufferQueue,
    body: RefCell<Option<Body>>,
    namespaces: RefCell<NamespaceStack>,
    preserved: Option<RefCell<PreservedBodies>>,
    hidden_content: crate::options::HiddenContent,
    replacements: RefCell<Vec<(std::ops::Range<usize>, String)>>,
    token_boundary: Cell<usize>,
    fed_end: Cell<usize>,
    opaque: RefCell<Option<LocalName>>,
}

impl BodyNormalizer<'_> {
    fn position(&self) -> usize {
        // ~keep The absolute segment endpoint locates tokens even when ordinary HTML
        // ~keep between protected bodies never enters the tokenizer.
        self.fed_end.get() - self.input.peek_front_chunk_mut().map_or(0, |chunk| chunk.len())
    }

    fn finish_body(&self, body: Body, end: usize, closed: bool) {
        let close_start = closed.then(|| self.closing_start(&body, end)).flatten();
        let content_end = close_start.unwrap_or(end);
        let mut replacement = if matches!(&*body.name, "iframe" | "noscript" | "template")
            && let Some(preserved) = &self.preserved
        {
            preserved.borrow_mut().protect(&self.source[body.start..content_end])
        } else {
            let mut escaped = String::new();
            for character in body.text.chars() {
                match character {
                    '&' => escaped.push_str("&amp;"),
                    '<' => escaped.push_str("&lt;"),
                    _ => escaped.push(character),
                }
            }
            escaped
        };
        if let Some(close_start) = close_start {
            replacement.push_str(&self.source[close_start..end]);
        } else {
            replacement.push_str("</");
            replacement.push_str(&body.name);
            replacement.push('>');
        }
        self.namespaces.borrow_mut().truncate(body.namespace_depth);
        if self.source[body.start..end] != replacement {
            self.replacements.borrow_mut().push((body.start..end, replacement));
        }
    }

    fn closing_start(&self, body: &Body, end: usize) -> Option<usize> {
        // ~keep The previous tag/comment token ends before this closing token starts;
        // ~keep tag-shaped strings inside attributes and comments are already behind it.
        let start = self.token_boundary.get().max(body.start);
        let bytes = self.source.as_bytes();
        memchr::memchr_iter(b'<', &bytes[start..end])
            .map(|index| start + index)
            .find(|index| {
                bytes.get(index + 1) == Some(&b'/') && matches_body_name(bytes, index + 2, body.name.as_bytes())
            })
    }

    fn track_namespace(&self, tag: &Tag) -> bool {
        let mut contexts = self.namespaces.borrow_mut();
        if tag.kind == TagKind::EndTag {
            if let Some(index) = contexts.matching_index(&tag.name) {
                contexts.truncate(index);
            }
            return false;
        }
        let name = &*tag.name;
        let mut namespace = contexts
            .last()
            .map_or(Namespace::Html, |context| context.children_namespace);
        if namespace != Namespace::Html && is_foreign_breakout(tag) {
            while contexts
                .last()
                .is_some_and(|context| context.children_namespace != Namespace::Html)
            {
                contexts.pop();
            }
            namespace = Namespace::Html;
        }
        if matches!(name, "mglyph" | "malignmark")
            && contexts
                .last()
                .is_some_and(|context| context.namespace == Namespace::Math && is_math_text_integration(&context.name))
        {
            namespace = Namespace::Math;
        }
        let html = namespace == Namespace::Html
            || (name == "svg"
                && contexts
                    .last()
                    .is_some_and(|context| context.namespace == Namespace::Math && &*context.name == "annotation-xml"));
        if html {
            namespace = match name {
                "svg" => Namespace::Svg,
                "math" => Namespace::Math,
                _ => Namespace::Html,
            };
        }
        let children_namespace = integration_namespace(namespace, tag);
        let skip_context = tag.self_closing
            || (namespace == Namespace::Html && (is_void(name) || contexts.is_empty() || is_protected_or_raw(name)));
        if !skip_context {
            contexts.push(ForeignContext {
                name: tag.name.clone(),
                namespace,
                children_namespace,
            });
        }
        html && namespace == Namespace::Html
    }

    fn process_tag(&self, tag: Tag) -> TokenSinkResult<()> {
        let name = &*tag.name;
        if tag.kind == TagKind::EndTag && self.opaque.borrow().as_ref() == Some(&tag.name) {
            self.opaque.replace(None);
            return TokenSinkResult::Script(());
        }
        let html = self
            .namespaces
            .borrow()
            .last()
            .is_none_or(|context| context.children_namespace == Namespace::Html);
        let foreign_template_close = name == "template"
            && tag.kind == TagKind::EndTag
            && self
                .namespaces
                .borrow()
                .matching_index(&tag.name)
                .is_some_and(|index| self.namespaces.borrow().contexts[index].namespace != Namespace::Html);
        if let Some(body) = self.body.borrow_mut().as_mut() {
            if name == "template"
                && &*body.name == "template"
                && ((tag.kind == TagKind::StartTag && html) || (tag.kind == TagKind::EndTag && !foreign_template_close))
            {
                if tag.kind == TagKind::StartTag {
                    body.template_depth += 1;
                } else {
                    body.template_depth = body.template_depth.saturating_sub(1);
                }
            }
        }
        let closes_body = self.body.borrow().as_ref().is_some_and(|body| {
            tag.kind == TagKind::EndTag && !foreign_template_close && body.name == tag.name && body.template_depth == 0
        });
        if closes_body {
            if let Some(body) = self.body.borrow_mut().take() {
                self.finish_body(body, self.position(), true);
            }
            return TokenSinkResult::Script(());
        }
        if !self.track_namespace(&tag) || super::kept::keeps_inert_body(self.hidden_content, &tag) {
            return TokenSinkResult::Continue;
        }
        if self.body.borrow().is_none() && matches!(name, "title" | "xmp" | "iframe" | "noscript" | "template") {
            self.body.replace(Some(Body {
                name: tag.name.clone(),
                start: self.position(),
                text: String::new(),
                template_depth: usize::from(name == "template"),
                namespace_depth: self.namespaces.borrow().len(),
            }));
        }
        if matches!(
            name,
            "textarea" | "script" | "style" | "noembed" | "noframes" | "plaintext"
        ) {
            self.opaque.replace(Some(tag.name.clone()));
        }
        match name {
            "title" | "textarea" => TokenSinkResult::RawData(RawKind::Rcdata),
            "script" => TokenSinkResult::RawData(RawKind::ScriptData),
            "style" | "xmp" | "iframe" | "noscript" | "noembed" | "noframes" => {
                TokenSinkResult::RawData(RawKind::Rawtext)
            }
            "plaintext" => TokenSinkResult::Plaintext,
            _ => TokenSinkResult::Continue,
        }
    }
}

fn is_math_text_integration(name: &str) -> bool {
    matches!(name, "mi" | "mo" | "mn" | "ms" | "mtext")
}

fn integration_namespace(namespace: Namespace, tag: &Tag) -> Namespace {
    let name = &*tag.name;
    let svg_html = namespace == Namespace::Svg && matches!(name, "foreignobject" | "desc" | "title");
    let math_html = namespace == Namespace::Math
        && (is_math_text_integration(name)
            || (name == "annotation-xml"
                && tag.attrs.iter().any(|attribute| {
                    &*attribute.name.local == "encoding"
                        && (attribute.value.eq_ignore_ascii_case("text/html")
                            || attribute.value.eq_ignore_ascii_case("application/xhtml+xml"))
                })));
    if svg_html || math_html {
        Namespace::Html
    } else {
        namespace
    }
}

fn is_protected_or_raw(name: &str) -> bool {
    matches!(
        name,
        "title"
            | "xmp"
            | "iframe"
            | "noscript"
            | "template"
            | "textarea"
            | "script"
            | "style"
            | "noembed"
            | "noframes"
            | "plaintext"
    )
}

fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_foreign_breakout(tag: &Tag) -> bool {
    // ~keep Mirrors html5ever's foreign-content breakout set; an HTML paragraph after
    // ~keep an unclosed SVG changes subsequent iframe/title tokenization back to HTML.
    matches!(
        &*tag.name,
        "b" | "big"
            | "blockquote"
            | "body"
            | "br"
            | "center"
            | "code"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "em"
            | "embed"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "hr"
            | "i"
            | "img"
            | "li"
            | "listing"
            | "menu"
            | "meta"
            | "nobr"
            | "ol"
            | "p"
            | "pre"
            | "ruby"
            | "s"
            | "small"
            | "span"
            | "strong"
            | "strike"
            | "sub"
            | "sup"
            | "table"
            | "tt"
            | "u"
            | "ul"
            | "var"
    ) || (&*tag.name == "font"
        && tag
            .attrs
            .iter()
            .any(|attribute| matches!(&*attribute.name.local, "color" | "face" | "size")))
}

impl TokenSink for BodyNormalizer<'_> {
    type Handle = ();

    fn process_token(&self, token: Token, _: u64) -> TokenSinkResult<()> {
        #[cfg(test)]
        TOKEN_VISITS.with(|visits| visits.set(visits.get() + 1));
        match token {
            Token::TagToken(tag) => {
                let result = self.process_tag(tag);
                self.token_boundary.set(self.position());
                result
            }
            Token::CommentToken(_) | Token::DoctypeToken(_) => {
                self.token_boundary.set(self.position());
                TokenSinkResult::Continue
            }
            Token::CharacterTokens(text) => {
                if let Some(body) = self.body.borrow_mut().as_mut()
                    && matches!(&*body.name, "title" | "xmp")
                {
                    body.text.push_str(&text);
                }
                TokenSinkResult::Continue
            }
            _ => TokenSinkResult::Continue,
        }
    }

    fn end(&self) {
        if let Some(body) = self.body.borrow_mut().take() {
            self.finish_body(body, self.source.len(), false);
        }
    }
}

fn matches_body_name(bytes: &[u8], start: usize, name: &[u8]) -> bool {
    bytes
        .get(start..start + name.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
        && matches!(
            bytes.get(start + name.len()),
            None | Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
        )
}

fn needs_normalization(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut consumed = 0;
    for index in memchr::memchr_iter(b'<', bytes) {
        if index < consumed {
            continue;
        }
        let Some(prefix) = bytes.get(index..index + 4) else {
            continue;
        };
        let Some(name) = ["title", "xmp", "iframe", "noscript", "template"]
            .into_iter()
            .find(|name| prefix[0] == b'<' && prefix[1..].eq_ignore_ascii_case(&name.as_bytes()[..3]))
        else {
            continue;
        };
        if !matches_body_name(bytes, index + 1, name.as_bytes()) {
            continue;
        }
        #[cfg(test)]
        TAG_SCANS.with(|scans| scans.set(scans.get() + 1));
        let Some(start) = find_tag_end(bytes, index + 1 + name.len()) else {
            return true;
        };
        consumed = start;
        if bytes[index..start].ends_with(b"/>") {
            return true;
        }
        let next = memchr::memchr(b'<', &bytes[start..]).map_or(bytes.len(), |next| start + next);
        if name == "xmp" && bytes[start..next].contains(&b'&') {
            return true;
        }
        if next < bytes.len()
            && !(bytes.get(next + 1) == Some(&b'/') && matches_body_name(bytes, next + 2, name.as_bytes()))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
fn normalize_inert_bodies(source: &str) -> Cow<'_, str> {
    normalize_inert_bodies_with_preservation(source, false, crate::options::HiddenContent::Drop).0
}

/// Keep opaque source bodies available for DOM preservation and visitor serialization. ~keep
pub fn normalize_inert_bodies_with_preservation(
    source: &str,
    preserve: bool,
    hidden_content: crate::options::HiddenContent,
) -> (Cow<'_, str>, HashMap<String, String>) {
    if !needs_normalization(source) {
        return (Cow::Borrowed(source), HashMap::new());
    }
    let input = BufferQueue::default();
    let tokenizer = Tokenizer::new(
        BodyNormalizer {
            source,
            input: &input,
            body: RefCell::new(None),
            namespaces: RefCell::new(NamespaceStack::default()),
            preserved: preserve.then(|| RefCell::new(PreservedBodies::new(source))),
            hidden_content,
            replacements: RefCell::new(Vec::new()),
            token_boundary: Cell::new(0),
            fed_end: Cell::new(0),
            opaque: RefCell::new(None),
        },
        TokenizerOpts::default(),
    );
    feed_protected_regions(source, &input, &tokenizer);
    tokenizer.end();
    let originals = tokenizer
        .sink
        .preserved
        .map_or_else(HashMap::new, |preserved| preserved.into_inner().originals);
    let replacements = tokenizer.sink.replacements.into_inner();
    if replacements.is_empty() {
        return (Cow::Borrowed(source), originals);
    }
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (range, replacement) in replacements {
        output.push_str(&source[cursor..range.start]);
        output.push_str(&replacement);
        cursor = range.end;
    }
    output.push_str(&source[cursor..]);
    (Cow::Owned(output), originals)
}

const TOKENIZER_CHUNK_BYTES: usize = 4096;

fn feed_protected_regions(source: &str, input: &BufferQueue, tokenizer: &Tokenizer<BodyNormalizer<'_>>) {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let inside = tokenizer.sink.body.borrow().is_some() || tokenizer.sink.opaque.borrow().is_some();
        let end = if inside {
            let limit = (cursor + TOKENIZER_CHUNK_BYTES).min(bytes.len());
            let mut end = memchr::memchr(b'>', &bytes[cursor..limit]).map_or(limit, |offset| cursor + offset + 1);
            while !source.is_char_boundary(end) {
                end -= 1;
            }
            end
        } else {
            let Some(offset) = memchr::memchr(b'<', &bytes[cursor..]) else {
                break;
            };
            cursor += offset;
            if !super::raw_text::opens_a_tag(bytes, cursor)
                && !matches!(bytes.get(cursor + 1), Some(b'!' | b'?' | b'/'))
            {
                cursor += 1;
                continue;
            }
            let end = scan_markup_end(bytes, cursor).unwrap_or(bytes.len());
            let name_start = cursor + 1 + usize::from(bytes.get(cursor + 1) == Some(&b'/'));
            let name_end = bytes[name_start..]
                .iter()
                .position(|byte| matches!(byte, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c'))
                .map_or(bytes.len(), |offset| name_start + offset);
            let name = &bytes[name_start..name_end];
            let relevant = !tokenizer.sink.namespaces.borrow().is_empty()
                || [
                    "svg",
                    "math",
                    "title",
                    "xmp",
                    "iframe",
                    "noscript",
                    "template",
                    "textarea",
                    "script",
                    "style",
                    "noembed",
                    "noframes",
                    "plaintext",
                ]
                .iter()
                .any(|candidate| name.eq_ignore_ascii_case(candidate.as_bytes()));
            if !relevant {
                cursor = end;
                continue;
            }
            end
        };
        tokenizer.sink.fed_end.set(end);
        #[cfg(test)]
        TOKEN_BYTES.with(|count| count.set(count.get() + end - cursor));
        input.push_back(StrTendril::from(&source[cursor..end]));
        let _ = tokenizer.feed(input);
        cursor = tokenizer.sink.position();
        // ~keep A closing raw token suspends the feed; its unconsumed suffix is
        // ~keep scanned as ordinary source instead of changing tokenizer context.
        while input.pop_front().is_some() {}
    }
}

#[derive(Clone, Copy)]
enum AttributeScan {
    BeforeName,
    Name,
    AfterName,
    BeforeValue,
    Unquoted,
    Quoted(u8),
}

fn scan_markup_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes[start..].starts_with(b"<!--") {
        let body = start + 4;
        if bytes.get(body) == Some(&b'>') {
            return Some(body + 1);
        }
        if bytes.get(body..body + 2) == Some(b"->") {
            return Some(body + 2);
        }
        for index in memchr::memchr_iter(b'>', &bytes[body..]).map(|offset| body + offset) {
            // ~keep The opener's dashes cannot put the tokenizer into comment-end state.
            if bytes[body..index].ends_with(b"--") || bytes[body..index].ends_with(b"--!") {
                return Some(index + 1);
            }
        }
        return None;
    }
    // ~keep Invalid end-tag openers consume through the first >, without attribute quoting.
    let bogus_end_tag =
        bytes.get(start + 1) == Some(&b'/') && bytes.get(start + 2).is_some_and(|byte| !byte.is_ascii_alphabetic());
    if matches!(bytes.get(start + 1), Some(b'!' | b'?')) || bogus_end_tag {
        return memchr::memchr(b'>', &bytes[start + 2..]).map(|offset| start + offset + 3);
    }
    let name_start = start + 1 + usize::from(bytes.get(start + 1) == Some(&b'/'));
    let name_end = bytes[name_start..]
        .iter()
        .position(|byte| matches!(byte, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c'))
        .map_or(bytes.len(), |offset| name_start + offset);
    let mut state = AttributeScan::BeforeName;
    for (offset, byte) in bytes[name_end..].iter().copied().enumerate() {
        if byte == b'>' && !matches!(state, AttributeScan::Quoted(_)) {
            return Some(name_end + offset + 1);
        }
        state = match state {
            AttributeScan::Quoted(delimiter) if byte != delimiter => state,
            AttributeScan::Quoted(_) => AttributeScan::BeforeName,
            AttributeScan::Unquoted if !byte.is_ascii_whitespace() => state,
            AttributeScan::Unquoted => AttributeScan::BeforeName,
            AttributeScan::BeforeValue if byte.is_ascii_whitespace() => state,
            AttributeScan::BeforeValue if matches!(byte, b'\'' | b'"') => AttributeScan::Quoted(byte),
            AttributeScan::BeforeValue => AttributeScan::Unquoted,
            AttributeScan::Name | AttributeScan::AfterName if byte == b'=' => AttributeScan::BeforeValue,
            AttributeScan::Name if byte.is_ascii_whitespace() => AttributeScan::AfterName,
            _ if byte.is_ascii_whitespace() || byte == b'/' => AttributeScan::BeforeName,
            _ => AttributeScan::Name,
        };
    }
    None
}

/// Restore opaque source bytes after parsing, before metadata or visitor consumers see the DOM. ~keep
pub fn restore_inert_bodies(dom: &mut tl::VDom<'_>, originals: &HashMap<String, String>) -> crate::error::Result<()> {
    for node in dom.nodes_mut() {
        if let Some(raw) = node.as_raw_mut()
            && let Some(original) = originals.get(raw.as_utf8_str().as_ref())
        {
            raw.set(original.as_bytes().to_vec()).map_err(|error| {
                crate::error::ConversionError::Other(format!("restoring inert HTML source bytes: {error:?}"))
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
thread_local! {
    static TOKEN_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TAG_SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TOKEN_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static NAMESPACE_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::normalize_inert_bodies;
    use std::borrow::Cow;

    #[test]
    fn should_skip_bogus_end_tag_comments_before_protected_bodies() {
        for next in [" ", "\t", "\n", "0", "!", "?", "/", "'", "é", " data='"] {
            for tag in ["title", "xmp", "iframe", "noscript", "template", "plaintext", "script"] {
                let before = format!("</{next}<{tag}>>");
                let source = format!("{before}<noscript><p>fallback</noscript><p>visible</p>");
                assert_eq!(
                    normalize_inert_bodies(&source),
                    format!("{before}<noscript></noscript><p>visible</p>"),
                    "{next:?}/{tag}"
                );
            }
        }
    }

    #[test]
    fn should_not_match_a_comment_terminator_across_its_body_boundary() {
        for opener in ["<!--!>", "<!---!>", "<!-->!>", "<!--->!>"] {
            let before = format!("{opener}<plaintext><noscript>fake</noscript></plaintext>-->");
            let source = format!("{before}<xmp><b>literal</b></xmp><noscript><p>fallback</noscript><p>visible</p>");
            let expected = format!("{before}<xmp>&lt;b>literal&lt;/b></xmp><noscript></noscript><p>visible</p>");
            if matches!(opener, "<!--!>" | "<!---!>") {
                assert_eq!(normalize_inert_bodies(&source), expected, "{opener}");
            } else {
                assert_eq!(
                    normalize_inert_bodies(&source),
                    source,
                    "an abrupt empty comment exposes plaintext"
                );
            }
        }
    }

    #[test]
    fn should_tokenize_only_the_inert_body_amid_large_html_gaps() {
        let paragraph = "<p data-note='quoted <noscript> fake'>ordinary text</p>";
        let prefix = paragraph.repeat(20_000);
        let suffix = paragraph.repeat(20_000);
        let source = format!("{prefix}<noscript><p>fallback</noscript>{suffix}");
        super::TOKEN_BYTES.with(|bytes| bytes.set(0));
        let normalized = normalize_inert_bodies(&source);
        assert_eq!(normalized, format!("{prefix}<noscript></noscript>{suffix}"));
        super::TOKEN_BYTES.with(|bytes| {
            assert!(bytes.get() > 0, "the candidate must actually be tokenized");
            assert!(bytes.get() <= 64, "tokenized {} of {} bytes", bytes.get(), source.len());
        });
    }

    #[test]
    fn should_keep_fake_candidates_outside_protected_bodies_opaque() {
        for before in [
            "a < b ",
            "<<",
            "<!bogus data-x='fake> ",
            "<?bogus data-x='fake> ",
            "<p data-x='quoted <noscript> fake'></p>",
            "<!-- <noscript><p>fake</noscript> -->",
            "<!--> ",
            "<!---> ",
            "<!-- <noscript> --!>",
            "<script><!--<script><noscript>fake</noscript></script>--></script>",
            "<style><noscript>fake</noscript></style>",
            "<textarea><noscript>fake</noscript></textarea>",
            "<p data-x=unquoted'quote><span></span></p>",
            "<p ='quote><span></span></p>",
        ] {
            let source = format!("{before}<noscript><p>fallback</noscript><p>visible</p>");
            assert_eq!(
                normalize_inert_bodies(&source),
                format!("{before}<noscript></noscript><p>visible</p>")
            );
        }
    }

    #[test]
    fn should_match_tokenizer_abrupt_doctype_identifier_endings() {
        for keyword in ["PUBLIC", "SYSTEM"] {
            let source = format!("<!DOCTYPE html {keyword} \"x><noscript><p>fallback</noscript>\"><p>visible</p>");
            assert_eq!(
                normalize_inert_bodies(&source),
                format!("<!DOCTYPE html {keyword} \"x><noscript></noscript>\"><p>visible</p>")
            );
        }
    }

    #[test]
    fn should_keep_source_offsets_when_a_raw_body_crosses_unicode_chunks() {
        let text = "é🦀".repeat(2_000);
        let source = format!("<p>prefix</p><xmp>{text}<p>literal</xmp><p>visible</p>");
        assert_eq!(
            normalize_inert_bodies(&source),
            format!("<p>prefix</p><xmp>{text}&lt;p>literal</xmp><p>visible</p>")
        );
    }

    #[test]
    fn should_preserve_source_outside_target_bodies() {
        let source = "<p data-x='a > b'>one</p><IFRAME Src=/f><p></IFRAME><p data-y=two>tail</p>";
        assert_eq!(
            normalize_inert_bodies(source),
            "<p data-x='a > b'>one</p><IFRAME Src=/f></IFRAME><p data-y=two>tail</p>"
        );
    }

    #[test]
    fn should_leave_svg_title_markup_unchanged() {
        let source = "<svg><title>a <p> b</p></title></svg>";
        assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
    }

    #[test]
    fn should_protect_raw_text_inside_foreign_object() {
        assert_eq!(
            normalize_inert_bodies("<svg><foreignObject><iframe><p></iframe></foreignObject></svg>"),
            "<svg><foreignObject><iframe></iframe></foreignObject></svg>"
        );
    }

    #[test]
    fn should_ignore_unmatched_foreign_end_tags() {
        assert_eq!(
            normalize_inert_bodies("<svg></math><title>a <p> b</p></title></svg>"),
            "<svg></math><title>a <p> b</p></title></svg>"
        );
    }

    #[test]
    fn should_restore_outer_namespace_after_dropping_a_template() {
        assert_eq!(
            normalize_inert_bodies("<template><svg></template><xmp><p></xmp>"),
            "<template></template><xmp>&lt;p></xmp>"
        );
    }

    #[test]
    fn should_borrow_already_normalized_target_bodies() {
        assert!(matches!(
            normalize_inert_bodies("<head><title>safe title</title></head>"),
            Cow::Borrowed(_)
        ));
    }
    #[test]
    fn should_not_count_a_foreign_template_as_an_html_template() {
        let source = "<template><svg><template></svg></template><p>visible</p>";
        assert_eq!(normalize_inert_bodies(source), "<template></template><p>visible</p>");
    }

    #[test]
    fn should_break_out_of_svg_on_an_html_paragraph() {
        let source = "<svg><p><iframe><p></iframe><p>visible</p>";
        assert_eq!(
            normalize_inert_bodies(source),
            "<svg><p><iframe></iframe><p>visible</p>"
        );
        let (repaired, limit) =
            crate::converter::main_helpers::repair_with_html5ever_bounded(source).expect("repair HTML");
        assert_eq!(limit, None);
        assert_eq!(
            repaired,
            "<head></head><svg></svg><p><iframe><p></iframe></p><p>visible</p>"
        );
    }

    #[test]
    fn should_not_treat_math_integration_names_as_svg_integration_points() {
        let source = "<svg><mi><xmp>a <b>x</b></xmp></mi></svg>";
        assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
    }

    #[test]
    fn should_not_treat_svg_integration_names_as_math_integration_points() {
        let source = "<math><title><xmp>a <b>x</b></xmp></title></math>";
        assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
    }

    #[test]
    fn should_keep_math_text_integration_exceptions_in_math_namespace() {
        for tag in ["mglyph", "malignmark"] {
            let source = format!("<math><mtext><{tag}><xmp>a &amp; b</xmp></{tag}></mtext></math>");
            assert!(matches!(normalize_inert_bodies(&source), Cow::Borrowed(value) if value == source));
        }
    }

    #[test]
    fn should_treat_html_void_names_as_normal_foreign_elements() {
        let source = "<svg><source><title></source><xmp>a &amp; b</xmp></svg>";
        assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(value) if value == source));
    }

    #[test]
    fn should_enter_svg_inside_math_annotation_without_an_encoding() {
        let source = "<math><annotation-xml><svg><foreignObject><iframe><p></iframe></foreignObject></svg></annotation-xml></math>";
        let expected =
            "<math><annotation-xml><svg><foreignObject><iframe></iframe></foreignObject></svg></annotation-xml></math>";
        assert_eq!(normalize_inert_bodies(source), expected);
    }
    #[test]
    fn should_skip_tokenization_for_bodies_without_literal_markup() {
        super::TOKEN_VISITS.with(|visits| visits.set(0));
        let source = "<TITLE>safe &amp; title</TITLE><noscript>plain</noscript><iframe>plain</iframe><template>plain</template><p>visible</p>";
        assert!(matches!(normalize_inert_bodies(source), Cow::Borrowed(_)));
        super::TOKEN_VISITS.with(|visits| assert_eq!(visits.get(), 0));
    }

    #[test]
    fn should_bound_namespace_work_for_unmatched_foreign_closes() {
        const DEPTH: usize = 10_000;
        let source = "<title><p></title><svg>".to_owned() + &"<g>".repeat(DEPTH) + &"</zz>".repeat(DEPTH);
        super::NAMESPACE_VISITS.with(|visits| visits.set(0));
        super::TOKEN_VISITS.with(|visits| visits.set(0));
        let normalized = normalize_inert_bodies(&source);
        assert!(normalized.starts_with("<title>&lt;p></title><svg>"));
        super::TOKEN_VISITS.with(|visits| assert!(visits.get() >= 2 * DEPTH));
        super::NAMESPACE_VISITS.with(|visits| assert_eq!(visits.get(), DEPTH));
    }

    #[test]
    fn should_skip_nested_candidate_shapes_inside_a_scanned_attribute() {
        let source = "<title a=".to_owned() + &"<title a=".repeat(10_000) + ">safe</title>";
        super::TAG_SCANS.with(|scans| scans.set(0));
        super::TOKEN_VISITS.with(|visits| visits.set(0));
        assert!(matches!(normalize_inert_bodies(&source), Cow::Borrowed(_)));
        super::TAG_SCANS.with(|scans| assert_eq!(scans.get(), 1));
        super::TOKEN_VISITS.with(|visits| assert_eq!(visits.get(), 0));
    }

    #[test]
    fn should_preserve_closing_token_bytes_with_quoted_tag_shapes() {
        let source = "<TEMPLATE><p></TEMPLATE data-x='</TEMPLATE> >'>";
        assert_eq!(
            normalize_inert_bodies(source),
            "<TEMPLATE></TEMPLATE data-x='</TEMPLATE> >'>"
        );
        let source = "<TEMPLATE><div data-x='</template a=\"'></div></TEMPLATE>";
        assert_eq!(normalize_inert_bodies(source), "<TEMPLATE></TEMPLATE>");
        let source = "<NOSCRIPT><p></NOSCRIPT data=x='>";
        assert_eq!(normalize_inert_bodies(source), "<NOSCRIPT></NOSCRIPT data=x='>");
    }

    #[test]
    fn should_bound_unmatched_template_close_lookups_in_foreign_markup() {
        const DEPTH: usize = 10_000;
        let source = "<title><p></title><svg>".to_owned() + &"<g>".repeat(DEPTH) + &"</template>".repeat(DEPTH);
        super::NAMESPACE_VISITS.with(|visits| visits.set(0));
        assert!(normalize_inert_bodies(&source).starts_with("<title>&lt;p></title>"));
        super::NAMESPACE_VISITS.with(|visits| assert_eq!(visits.get(), 2 * DEPTH));
    }

    #[test]
    fn should_avoid_source_and_entity_decoded_body_marker_collisions() {
        let source = "<p>h2m_inert_body_0_end h2m_inert_body_&#49;_end</p><noscript><p>x</noscript>";
        let (normalized, bodies) =
            super::normalize_inert_bodies_with_preservation(source, true, crate::options::HiddenContent::Drop);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies.get("h2m_inert_body_2_end").map(String::as_str), Some("<p>x"));
        assert!(normalized.contains("<noscript>h2m_inert_body_2_end</noscript>"));
        let mut dom = tl::parse(&normalized, tl::ParserOptions::default()).expect("parse protected bodies");
        super::restore_inert_bodies(&mut dom, &bodies).expect("restore original bodies");
        assert_eq!(
            crate::converter::utility::serialization::serialize_tag_to_html(&dom.children()[1], dom.parser()),
            "<noscript><p>x</noscript>"
        );
    }

    #[test]
    fn should_recognize_form_feed_as_html_tag_whitespace() {
        assert_eq!(
            normalize_inert_bodies("<NOSCRIPT\x0c><p></NOSCRIPT\x0c><p>visible</p>"),
            "<NOSCRIPT\x0c></NOSCRIPT\x0c><p>visible</p>"
        );
    }

    #[test]
    fn should_finish_unterminated_protected_bodies_at_eof() {
        for (source, expected) in [
            ("<title><p>", "<title>&lt;p></title>"),
            ("<xmp>a &amp;", "<xmp>a &amp;amp;</xmp>"),
            ("<iframe><p>x", "<iframe></iframe>"),
            ("<noscript><p>x", "<noscript></noscript>"),
            ("<template><p>x", "<template></template>"),
        ] {
            assert_eq!(normalize_inert_bodies(source), expected, "{source}");
        }
    }
}
