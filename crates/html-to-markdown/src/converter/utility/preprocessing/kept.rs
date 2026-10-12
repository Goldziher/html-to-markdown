//! Which inert elements a choice of `hidden_content` keeps.

use std::borrow::Cow;
use std::ops::Range;

use html5ever::tokenizer::Tag;

use super::head_scan::body_start;
use super::visibility::TagEndScan;
use crate::options::HiddenContent;

/// What the inert-body pass writes in place of ranges of a page: each range with its text, in
/// the order of the page.
pub(super) type Replacements = Vec<(Range<usize>, String)>;

/// The elements of one page whose content a choice of `hidden_content` keeps. The inert-body
/// pass leaves such an element as written and reads its content as markup.
///
/// ~keep The inert-body pass runs in front of the pass that removes the tags of the kept
/// ~keep `<template>` and `<noscript>` elements. It empties the body of each of the two elements,
/// ~keep so the later pass would find no content to keep. The rule here is the rule of that
/// ~keep pass: [`HiddenContent::All`] keeps every `<template>` and `<noscript>` of the body, and
/// ~keep [`HiddenContent::Reachable`] keeps a declarative shadow root of the body. Under
/// ~keep [`HiddenContent::Drop`] the inert-body pass empties every body, as it does with no
/// ~keep option.
///
/// ~keep An element of the head is not kept: the later pass leaves its tags, and the walk drops
/// ~keep it with what the first parse puts in it. Left as written, a `<noscript>` of the head
/// ~keep whose content has an element with no end tag held the text after it.
#[derive(Clone, Copy)]
pub(super) struct Kept {
    hidden_content: HiddenContent,
    /// Where the body starts in the page as written. No element before it is kept.
    body_start: usize,
}

impl Kept {
    /// The choice that keeps no element.
    pub(super) const NOTHING: Self = Self {
        hidden_content: HiddenContent::Drop,
        body_start: usize::MAX,
    };

    /// The elements of `source` that `hidden_content` keeps.
    ///
    /// ~keep The body starts where the later pass starts it. That pass reads the page of this
    /// ~keep pass, so the start is found in the page with every body empty, the page of
    /// ~keep [`HiddenContent::Drop`], and not in `source`: the two read a `<noscript/>` and the
    /// ~keep content of a `<noscript>` differently. A wider choice changes nothing before the
    /// ~keep first element that it keeps, so its page has the same start.
    pub(super) fn of(source: &str, hidden_content: HiddenContent) -> Self {
        if hidden_content == HiddenContent::Drop {
            return Self::NOTHING;
        }
        let (replacements, _) = super::inert::inert_body_replacements(source, false, Self::NOTHING);
        let emptied = rewritten(source, &replacements);
        let start = body_start(&emptied, &mut TagEndScan::new(emptied.as_bytes()));
        Self {
            hidden_content,
            body_start: start.map_or(usize::MAX, |start| offset_as_written(&replacements, start)),
        }
    }

    /// Whether the inert-body pass protects the body of the element that the start tag `tag`
    /// starts. `is_html` says that the element is an HTML element, and `tag_end` is where the
    /// tag ends in the page as written.
    pub(super) fn is_inert(self, is_html: bool, tag: &Tag, tag_end: usize) -> bool {
        is_html && !self.keeps(tag, tag_end)
    }

    fn keeps(self, tag: &Tag, tag_end: usize) -> bool {
        let keeps_the_kind = match self.hidden_content {
            HiddenContent::Drop => false,
            HiddenContent::Reachable => &*tag.name == "template" && declares_shadow_root(tag),
            HiddenContent::All => matches!(&*tag.name, "template" | "noscript"),
        };
        keeps_the_kind && tag_end > self.body_start
    }
}

/// Whether the start tag of a `<template>` declares a shadow root: `shadowrootmode` is `open` or
/// `closed`.
fn declares_shadow_root(tag: &Tag) -> bool {
    tag.attrs.iter().any(|attribute| {
        &*attribute.name.local == "shadowrootmode"
            && (attribute.value.eq_ignore_ascii_case("open") || attribute.value.eq_ignore_ascii_case("closed"))
    })
}

/// HTML ignores a non-void start tag's self-closing flag; the literal parser must contain its body. ~keep
pub(super) fn normalize_inert_opening(tag: &Tag, end: usize, replacements: &mut Replacements) {
    let opening = tag.self_closing.then(|| (end - 2..end, ">".to_owned()));
    replacements.extend(opening);
}

/// `source` with each range of `replacements` replaced by its text. The ranges are in the order
/// of the page and do not overlap.
pub(super) fn rewritten<'a>(source: &'a str, replacements: &[(Range<usize>, String)]) -> Cow<'a, str> {
    if replacements.is_empty() {
        return Cow::Borrowed(source);
    }
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (range, replacement) in replacements {
        output.push_str(&source[cursor..range.start]);
        output.push_str(replacement);
        cursor = range.end;
    }
    output.push_str(&source[cursor..]);
    Cow::Owned(output)
}

/// The offset in the page as written of `offset`, an offset in the page that [`rewritten`]
/// makes of it. An offset in the text of a replacement is the start of the replaced range.
fn offset_as_written(replacements: &[(Range<usize>, String)], offset: usize) -> usize {
    // ~keep `written` is the end of the last range before `offset`, and `rewritten_to` is where
    // ~keep the text of that range ends in the rewritten page.
    let (mut written, mut rewritten_to) = (0, 0);
    for (range, replacement) in replacements {
        let replacement_start = rewritten_to + (range.start - written);
        if offset < replacement_start {
            break;
        }
        if offset < replacement_start + replacement.len() {
            return range.start;
        }
        written = range.end;
        rewritten_to = replacement_start + replacement.len();
    }
    written + (offset - rewritten_to)
}

#[cfg(test)]
mod tests {
    use super::{offset_as_written, rewritten};

    #[test]
    fn should_find_the_offset_as_written_of_each_offset_of_the_rewritten_page() {
        // ~keep "ab[cde]fg[hi]j" with `cde` replaced by `X` and `hi` by `YYYY`: "abXfgYYYYj".
        let source = "abcdefghij";
        let replacements = [(2..5, "X".to_owned()), (7..9, "YYYY".to_owned())];
        assert_eq!(rewritten(source, &replacements), "abXfgYYYYj");
        let as_written: Vec<usize> = (0..=10)
            .map(|offset| offset_as_written(&replacements, offset))
            .collect();
        assert_eq!(as_written, [0, 1, 2, 5, 6, 7, 7, 7, 7, 9, 10]);
        assert_eq!(offset_as_written(&[], 4), 4);
        assert_eq!(offset_as_written(&[(3..3, "</p>".to_owned())], 3), 3);
        assert_eq!(offset_as_written(&[(3..3, "</p>".to_owned())], 7), 3);
    }
}
