//! Protect HTML raw-text and inert bodies from the literal DOM parser. ~keep

use std::borrow::Cow;
use std::cell::RefCell;

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

struct BodyNormalizer<'a> {
    source: &'a str,
    input: &'a BufferQueue,
    body: RefCell<Option<Body>>,
    namespaces: RefCell<Vec<ForeignContext>>,
    replacements: RefCell<Vec<(std::ops::Range<usize>, String)>>,
}

impl BodyNormalizer<'_> {
    fn position(&self) -> usize {
        // ~keep One source tendril is queued; its remaining bytes locate tag-token ends
        // ~keep without guessing at quoted attributes or comment-shaped source text.
        self.source.len() - self.input.peek_front_chunk_mut().map_or(0, |chunk| chunk.len())
    }

    fn finish_body(&self, body: Body, end: usize) {
        let mut replacement = String::new();
        for character in body.text.chars() {
            match character {
                '&' => replacement.push_str("&amp;"),
                '<' => replacement.push_str("&lt;"),
                _ => replacement.push(character),
            }
        }
        replacement.push_str("</");
        replacement.push_str(&body.name);
        replacement.push('>');
        self.namespaces.borrow_mut().truncate(body.namespace_depth);
        if self.source[body.start..end] != replacement {
            self.replacements.borrow_mut().push((body.start..end, replacement));
        }
    }

    fn track_namespace(&self, tag: &Tag) -> bool {
        let mut contexts = self.namespaces.borrow_mut();
        if tag.kind == TagKind::EndTag {
            if let Some(index) = contexts.iter().rposition(|context| context.name == tag.name) {
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
                .iter()
                .rev()
                .find(|context| context.name == tag.name)
                .is_some_and(|context| context.namespace != Namespace::Html);
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
                self.finish_body(body, self.position());
            }
            return TokenSinkResult::Continue;
        }
        if !self.track_namespace(&tag) {
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
        match token {
            Token::TagToken(tag) => self.process_tag(tag),
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
            self.finish_body(body, self.source.len());
        }
    }
}

/// Protect text and inert markup before the literal parser can swallow visible siblings. ~keep
pub fn normalize_inert_bodies(source: &str) -> Cow<'_, str> {
    let bytes = source.as_bytes();
    if !memchr::memchr_iter(b'<', bytes).any(|index| {
        bytes.get(index..index + 4).is_some_and(|prefix| {
            prefix[1].is_ascii_alphabetic()
                && [b"<tit", b"<xmp", b"<ifr", b"<nos", b"<tem"]
                    .iter()
                    .any(|target| prefix.eq_ignore_ascii_case(*target))
        })
    }) {
        return Cow::Borrowed(source);
    }
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(source));
    let tokenizer = Tokenizer::new(
        BodyNormalizer {
            source,
            input: &input,
            body: RefCell::new(None),
            namespaces: RefCell::new(Vec::new()),
            replacements: RefCell::new(Vec::new()),
        },
        TokenizerOpts::default(),
    );
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    let replacements = tokenizer.sink.replacements.into_inner();
    if replacements.is_empty() {
        return Cow::Borrowed(source);
    }
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (range, replacement) in replacements {
        output.push_str(&source[cursor..range.start]);
        output.push_str(&replacement);
        cursor = range.end;
    }
    output.push_str(&source[cursor..]);
    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::normalize_inert_bodies;
    use std::borrow::Cow;

    #[test]
    fn should_preserve_source_outside_target_bodies() {
        let source = "<p data-x='a > b'>one</p><IFRAME Src=/f><p></IFRAME><p data-y=two>tail</p>";
        assert_eq!(
            normalize_inert_bodies(source),
            "<p data-x='a > b'>one</p><IFRAME Src=/f></iframe><p data-y=two>tail</p>"
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
        let repaired = crate::converter::main_helpers::repair_with_html5ever(source).expect("repair HTML");
        assert!(
            repaired.contains("<body><svg></svg><p><iframe><p></iframe></p><p>visible</p>"),
            "{repaired}"
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
}
