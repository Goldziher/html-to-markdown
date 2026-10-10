//! Source-level omissions that a literal nesting parser cannot repair.

use std::cell::{Cell, RefCell};

use html5ever::LocalName;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};

use super::main_helpers::is_html5_void_element;

const MAX_TRACKED_DEPTH: usize = super::anchor_origin::MAX_REPAIR_DEPTH;

#[derive(Default)]
struct RepairDetector {
    stack: RefCell<Vec<LocalName>>,
    head_open: Cell<bool>,
    omitted_head_end: Cell<bool>,
    paragraph_ended_by_parent: Cell<bool>,
    needed: Cell<bool>,
}

impl RepairDetector {
    fn start(&self, tag: &Tag) {
        let name = &*tag.name;
        let mut stack = self.stack.borrow_mut();
        if self.head_open.get()
            && !stack.iter().any(|name| &**name == "template")
            && !matches!(
                name,
                "html"
                    | "head"
                    | "base"
                    | "basefont"
                    | "bgsound"
                    | "link"
                    | "meta"
                    | "noframes"
                    | "noscript"
                    | "script"
                    | "style"
                    | "template"
                    | "title"
            )
        {
            self.omitted_head_end.set(true);
        }
        if name == "head" {
            self.head_open.set(true);
        }
        if !is_html5_void_element(name.as_bytes()) && stack.len() < MAX_TRACKED_DEPTH {
            stack.push(tag.name.clone());
        }
    }

    fn close(&self, tag: &Tag) {
        if &*tag.name == "head" {
            self.head_open.set(false);
            self.omitted_head_end.set(false);
        }
        let mut stack = self.stack.borrow_mut();
        let Some(position) = stack.iter().rposition(|name| name == &tag.name) else {
            // ~keep The end tag of a parent ended this paragraph already, as in valid HTML
            // ~keep (`<li><p>x</li>`). Only this later `</p>` makes the literal tree differ
            // ~keep from the browser's: it would end the paragraph a second time (#810).
            if &*tag.name == "p" && self.paragraph_ended_by_parent.get() {
                self.needed.set(true);
            }
            return;
        };
        if &*tag.name != "p"
            && !stack.iter().any(|name| matches!(&**name, "pre" | "code"))
            && stack[position + 1..].iter().any(|name| &**name == "p")
        {
            self.paragraph_ended_by_parent.set(true);
        }
        stack.truncate(position);
    }
}

impl TokenSink for RepairDetector {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        if matches!(token, Token::EOFToken) {
            if self.head_open.get() && self.omitted_head_end.get() {
                self.needed.set(true);
            }
            return TokenSinkResult::Continue;
        }
        if let Token::TagToken(tag) = token {
            if tag.kind == TagKind::StartTag {
                self.start(&tag);
                if self.needed.get() {
                    return TokenSinkResult::Script(());
                }
                return match &*tag.name {
                    "title" | "textarea" => TokenSinkResult::RawData(RawKind::Rcdata),
                    "script" => TokenSinkResult::RawData(RawKind::ScriptData),
                    "style" | "xmp" | "iframe" | "noembed" | "noframes" | "noscript" => {
                        TokenSinkResult::RawData(RawKind::Rawtext)
                    }
                    "plaintext" => TokenSinkResult::Plaintext,
                    _ => TokenSinkResult::Continue,
                };
            }
            self.close(&tag);
        }
        if self.needed.get() {
            TokenSinkResult::Script(())
        } else {
            TokenSinkResult::Continue
        }
    }
}

pub(super) fn needs_source_tree_repair(html: &str) -> bool {
    let bytes = html.as_bytes();
    if !bytes.windows(3).any(|window| {
        window[..2].eq_ignore_ascii_case(b"<p") && (window[2].is_ascii_whitespace() || matches!(window[2], b'>' | b'/'))
    }) && !bytes.windows(5).any(|window| window.eq_ignore_ascii_case(b"<head"))
    {
        return false;
    }
    let tokenizer = Tokenizer::new(RepairDetector::default(), TokenizerOpts::default());
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(html));
    while !tokenizer.sink.needed.get() {
        if matches!(tokenizer.feed(&input), html5ever::TokenizerResult::Done) {
            tokenizer.end();
            break;
        }
    }
    tokenizer.sink.needed.get()
}

#[cfg(test)]
mod tests {
    use super::needs_source_tree_repair;

    #[test]
    fn source_tree_repair_should_ignore_markup_in_comments_and_script_data() {
        for html in [
            "<!--<head><h1>fake</h1>--><p>visible</p>",
            "<div><p>before<script><!--<script></script></div></p>--></script>after</p></div>",
            "<head><title>&lt;h1>fake&lt;/h1></title></head><p>visible</p>",
        ] {
            assert!(!needs_source_tree_repair(html), "{html}");
        }
    }

    #[test]
    fn source_tree_repair_should_wait_for_a_second_paragraph_end() {
        for html in [
            "<ul><li><p>x</li></ul>",
            "<div><p>x</div>",
            "<div><p>one</div><p>two</p>",
        ] {
            assert!(!needs_source_tree_repair(html), "{html}");
        }
        for html in [
            "<div><p>one</div>two</p>rest</div>tail",
            "<div><p>one</div><div>two</p></div>",
        ] {
            assert!(needs_source_tree_repair(html), "{html}");
        }
    }
}
