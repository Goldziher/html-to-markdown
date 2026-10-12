//! Protect HTML raw-text and inert bodies from the literal DOM parser. ~keep

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use super::form_boundaries::ClosedForms;
use super::kept::{Kept, Replacements, normalize_inert_opening, rewritten};
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
    kept: Kept,
    replacements: RefCell<Vec<(std::ops::Range<usize>, String)>>,
    token_boundary: Cell<usize>,
    fed_end: Cell<usize>,
    fed_start: Cell<usize>,
    opaque: RefCell<Option<LocalName>>,
    remove_closed_forms: bool,
    rewrite_bodies: bool,
    closed_forms: RefCell<ClosedForms>,
    html_starts: Option<&'a RefCell<Vec<usize>>>,
    graphic_roots: RefCell<NamespaceStack>,
}

impl BodyNormalizer<'_> {
    fn position(&self) -> usize {
        // ~keep The absolute segment endpoint locates tokens even when ordinary HTML
        // ~keep between protected bodies never enters the tokenizer.
        self.fed_end.get() - self.input.peek_front_chunk_mut().map_or(0, |chunk| chunk.len())
    }

    fn finish_body(&self, body: Body, end: usize, closed: bool) {
        if !self.rewrite_bodies {
            self.namespaces.borrow_mut().truncate(body.namespace_depth);
            return;
        }
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

    fn observe_closed_form(&self, tag: &Tag, is_html: bool, html: bool, inert: bool) {
        if self.remove_closed_forms
            && self.body.borrow().is_none()
            && self.namespaces.borrow().is_empty()
            && (is_html || (html && tag.kind == TagKind::EndTag))
            && !(&*tag.name == "template" && tag.kind == TagKind::StartTag && inert)
        {
            self.closed_forms
                .borrow_mut()
                .observe(tag, self.fed_start.get(), self.position());
        }
    }

    fn observe_graphic_root(&self, tag: &Tag) {
        if self.html_starts.is_none() || !matches!(&*tag.name, "svg" | "math") {
            return;
        }
        let mut roots = self.graphic_roots.borrow_mut();
        if tag.kind == TagKind::EndTag {
            if let Some(index) = roots.matching_index(&tag.name) {
                roots.truncate(index);
            }
        } else if !tag.self_closing {
            let namespace = if &*tag.name == "svg" {
                Namespace::Svg
            } else {
                Namespace::Math
            };
            roots.push(ForeignContext {
                name: tag.name.clone(),
                namespace,
                children_namespace: namespace,
            });
        }
    }

    fn observe_html_start(&self, tag: &Tag, is_html: bool) {
        if is_html
            && self.graphic_roots.borrow().is_empty()
            && tag.kind == TagKind::StartTag
            && matches!(&*tag.name, "template" | "noscript")
            && let Some(starts) = self.html_starts
        {
            starts.borrow_mut().push(self.fed_start.get());
        }
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
        self.observe_graphic_root(&tag);
        let is_html = self.track_namespace(&tag);
        self.observe_html_start(&tag, is_html);
        let inert = self.kept.is_inert(is_html, &tag, self.position());
        self.observe_closed_form(&tag, is_html, html, inert);
        if !inert {
            return TokenSinkResult::Continue;
        }
        if self.body.borrow().is_none() && matches!(name, "title" | "xmp" | "iframe" | "noscript" | "template") {
            normalize_inert_opening(&tag, self.position(), &mut self.replacements.borrow_mut());
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
                if self.rewrite_bodies
                    && let Some(body) = self.body.borrow_mut().as_mut()
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

fn needs_normalization(source: &str, remove_closed_forms: bool) -> bool {
    let bytes = source.as_bytes();
    let mut consumed = 0;
    for index in memchr::memchr_iter(b'<', bytes) {
        if index < consumed {
            continue;
        }
        let Some(prefix) = bytes.get(index..index + 4) else {
            continue;
        };
        let Some(name) = ["title", "xmp", "iframe", "noscript", "template", "form"]
            .into_iter()
            .filter(|name| *name != "form" || remove_closed_forms)
            .find(|name| prefix[0] == b'<' && prefix[1..].eq_ignore_ascii_case(&name.as_bytes()[..3]))
        else {
            continue;
        };
        if !matches_body_name(bytes, index + 1, name.as_bytes()) {
            continue;
        }
        if name == "form" {
            return true;
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
    if !needs_normalization(source, false) {
        return (Cow::Borrowed(source), HashMap::new());
    }
    let (replacements, originals) = inert_body_replacements(source, preserve, Kept::of(source, hidden_content));
    (rewritten(source, &replacements), originals)
}

/// Keep empty form tags while removing their content through the real source end tag.
pub fn remove_closed_form_bodies(source: &str, hidden_content: crate::options::HiddenContent) -> Cow<'_, str> {
    if !needs_normalization(source, true) {
        return Cow::Borrowed(source);
    }
    let (replacements, _) = body_replacements(source, false, Kept::of(source, hidden_content), true, false, None);
    rewritten(source, &replacements)
}

/// The text for each body of `source` that `kept` does not keep, and the preserved bodies.
pub(super) fn inert_body_replacements(
    source: &str,
    preserve: bool,
    kept: Kept,
) -> (Replacements, HashMap<String, String>) {
    body_replacements(source, preserve, kept, false, true, None)
}

/// Locate page-owned HTML template/noscript starts outside serialized SVG/Math roots.
/// ~keep Graphics own their descendants even at HTML integration points and foreign breakouts;
/// ~keep their wrappers/labels follow the existing graphic conversion contract.
pub fn html_inert_starts(source: &str, hidden_content: crate::options::HiddenContent) -> Vec<usize> {
    let starts = RefCell::new(Vec::new());
    let _ = body_replacements(
        source,
        false,
        Kept::of(source, hidden_content),
        false,
        false,
        Some(&starts),
    );
    starts.into_inner()
}

fn body_replacements(
    source: &str,
    preserve: bool,
    kept: Kept,
    remove_closed_forms: bool,
    rewrite_bodies: bool,
    html_starts: Option<&RefCell<Vec<usize>>>,
) -> (Replacements, HashMap<String, String>) {
    let input = BufferQueue::default();
    let tokenizer = Tokenizer::new(
        BodyNormalizer {
            source,
            input: &input,
            body: RefCell::new(None),
            namespaces: RefCell::new(NamespaceStack::default()),
            preserved: preserve.then(|| RefCell::new(PreservedBodies::new(source))),
            kept,
            replacements: RefCell::new(Vec::new()),
            token_boundary: Cell::new(0),
            fed_end: Cell::new(0),
            fed_start: Cell::new(0),
            opaque: RefCell::new(None),
            remove_closed_forms,
            rewrite_bodies,
            closed_forms: RefCell::new(ClosedForms::default()),
            html_starts,
            graphic_roots: RefCell::new(NamespaceStack::default()),
        },
        TokenizerOpts::default(),
    );
    feed_protected_regions(source, &input, &tokenizer);
    tokenizer.end();
    let originals = tokenizer
        .sink
        .preserved
        .map_or_else(HashMap::new, |preserved| preserved.into_inner().originals);
    let replacements = tokenizer
        .sink
        .closed_forms
        .into_inner()
        .apply(source, tokenizer.sink.replacements.into_inner());
    (replacements, originals)
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
                || (tokenizer.sink.remove_closed_forms
                    && [b"form".as_slice(), b"table"]
                        .iter()
                        .any(|candidate| name.eq_ignore_ascii_case(candidate)))
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
        tokenizer.sink.fed_start.set(cursor);
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
mod tests;
