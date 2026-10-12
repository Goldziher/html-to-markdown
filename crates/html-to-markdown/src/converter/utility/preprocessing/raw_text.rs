//! HTML preprocessing and normalization.
//!
//! Functions for preprocessing HTML before conversion, including script/style stripping,
//! tag repair, and malformed HTML handling.

use std::borrow::Cow;
use std::str;

use super::markup::{find_tag_end, is_json_ld_script_open_tag, matches_end_tag_start, matches_tag_start};
use super::visibility::{after_bogus_comment, is_bogus_comment_marker};

/// If the `<` at `idx` starts a bogus comment (see `strip_bogus_comments`), return the position
/// after it. The caller has read a real comment (`<!--`) before.
///
/// ~keep The first pass over raw-text elements reads a bogus comment with this function, so a
/// ~keep `<textarea>` in `<? ... >` starts nothing there. It reads `<![CDATA[` as a bogus
/// ~keep comment too: it reads no markup inside `<svg>`, and outside foreign content the
/// ~keep tokenizer ends the comment at the first `>`.
fn bogus_comment_end(bytes: &[u8], idx: usize) -> Option<usize> {
    let next = *bytes.get(idx + 1)?;
    is_bogus_comment_marker(bytes, idx, bytes.len(), next, &bytes[idx + 2..]).then(|| after_bogus_comment(bytes, idx))
}

/// If `idx` starts an `<svg`/`</svg` tag, adjust `svg_depth` accordingly and return the index
/// just past the tag. Returns `None` when `idx` does not start an svg tag (or the tag is
/// unterminated), in which case the caller's scan is unaffected.
///
/// Extracted from `strip_script_and_style_tags` — identical `<svg`/`</svg` detection and depth
/// bookkeeping, unchanged.
pub(super) fn track_svg_tag(bytes: &[u8], idx: usize, svg_depth: &mut usize) -> Option<usize> {
    if matches_tag_start(bytes, idx + 1, b"svg") {
        if let Some(open_end) = find_tag_end(bytes, idx + 1 + b"svg".len()) {
            *svg_depth += 1;
            return Some(open_end);
        }
    } else if matches_end_tag_start(bytes, idx + 1, b"svg") {
        if let Some(close_end) = find_tag_end(bytes, idx + 2 + b"svg".len()) {
            if *svg_depth > 0 {
                *svg_depth = svg_depth.saturating_sub(1);
            }
            return Some(close_end);
        }
    }
    None
}

/// What the first pass does with the content of a raw-text element.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RawTextContent {
    /// The element is removed (`style`).
    Removed,
    /// The element is removed, but a structured data script is kept for the metadata.
    RemovedUnlessStructuredData,
    /// The element is kept and its content is text (`textarea`).
    Kept,
}

/// What the first pass did with a raw-text element.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// The element stays as the page has it.
    Unchanged,
    /// The element is removed.
    Removed,
    /// The element is written again, with its content as text.
    Kept,
}

/// The `<` of a kept element, as the first pass writes it.
const ESCAPED_LT: &str = "&lt;";

/// The kept scripts of one page whose text the tree cannot give back as written.
///
/// ~keep `push_kept_element` writes each `<` of a kept script as `&lt;`, and `as_written` takes
/// ~keep that back. No other code knows of the escape: the tree holds the escaped text, and a
/// ~keep writer of HTML writes it as it is. A body that holds `&lt;` itself reads the same in
/// ~keep the tree as a body that holds `<`, so the escaped text does not name a script. The
/// ~keep first pass records each body whose escaped text holds `&lt;`, in the order of the page,
/// ~keep and `match_to_tree` gives each script of the final tree its own record.
#[derive(Debug)]
pub struct KeptScripts {
    /// The bodies whose escaped text holds `&lt;`, as written, in the order of the page, with
    /// no white space at the two ends.
    #[cfg(feature = "metadata")]
    bodies: Vec<Box<str>>,
    /// The id of a script node of the final tree with the index of its body in `bodies`, in
    /// the order of the ids.
    #[cfg(feature = "metadata")]
    of_node: Vec<(u32, usize)>,
    /// Whether the page has a kept script with no end tag.
    ///
    /// ~keep The rest of the page is the text of the first such script, and a later start tag
    /// ~keep is in that text, so the first pass records one body for them: the read stays
    /// ~keep linear on a page of such start tags.
    #[cfg(feature = "metadata")]
    has_no_end_tag: bool,
}

impl KeptScripts {
    const fn new() -> Self {
        Self {
            #[cfg(feature = "metadata")]
            bodies: Vec::new(),
            #[cfg(feature = "metadata")]
            of_node: Vec::new(),
            #[cfg(feature = "metadata")]
            has_no_end_tag: false,
        }
    }

    /// Record the body of a kept script when its escaped text holds `&lt;`.
    #[cfg(feature = "metadata")]
    fn record(&mut self, body: &str) {
        if body.contains('<') || body.contains(ESCAPED_LT) {
            self.bodies.push(body.trim().into());
        }
    }

    /// Whether the first pass recorded no body.
    #[cfg(feature = "metadata")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// Give each kept script of the final tree its record. `in_tree` has the id of the node
    /// and the text of each kept script whose text holds `&lt;`, in the order of the tree; the
    /// text has no white space at the two ends.
    ///
    /// ~keep A script is one of the scripts with its escaped text, and its place among them is
    /// ~keep its identity: the first of them in the tree is the first of them in the page. A
    /// ~keep script never gets the record of a script with another escaped text, so a pass that
    /// ~keep drops, moves or adds such a script changes nothing here. When the tree and the
    /// ~keep page do not have the same count of scripts with one escaped text, the place names
    /// ~keep no script and none of them gets a record; when all their bodies are equal the
    /// ~keep place is not needed. The tree builder writes a line end of the page as LF, so a
    /// ~keep CR is not compared.
    #[cfg(feature = "metadata")]
    pub fn match_to_tree(&mut self, in_tree: &[(u32, String)]) {
        let mut recorded: ahash::AHashMap<Vec<u8>, Vec<usize>> = ahash::AHashMap::new();
        for (index, body) in self.bodies.iter().enumerate() {
            let escaped = body.replace('<', ESCAPED_LT);
            recorded.entry(without_cr(&escaped)).or_default().push(index);
        }
        let mut found: ahash::AHashMap<Vec<u8>, Vec<u32>> = ahash::AHashMap::new();
        for (node, text) in in_tree {
            found.entry(without_cr(text)).or_default().push(*node);
        }
        for (text, scripts) in found {
            let Some(written) = recorded.get(&text) else {
                continue;
            };
            let one_body = written
                .iter()
                .all(|index| self.bodies[*index] == self.bodies[written[0]]);
            let named = scripts.len() == written.len() || (one_body && scripts.len() < written.len());
            if !named {
                continue;
            }
            for (place, node) in scripts.into_iter().enumerate() {
                let index = if one_body { written[0] } else { written[place] };
                self.of_node.push((node, index));
            }
        }
        self.of_node.sort_unstable();
    }

    /// The text of a kept script as the page has it. `node` is the id of the script in the
    /// final tree and `text` is the text that the tree has for it, with no white space at the
    /// two ends.
    ///
    /// ~keep A script with no record reads with each `&lt;` as `<`: right for a body that
    /// ~keep holds no `&lt;` of its own.
    #[cfg(feature = "metadata")]
    pub fn as_written<'a>(&'a self, node: u32, text: &'a str) -> Cow<'a, str> {
        if let Ok(at) = self.of_node.binary_search_by_key(&node, |(id, _)| *id) {
            let body: &str = &self.bodies[self.of_node[at].1];
            return Cow::Borrowed(body);
        }
        if text.contains(ESCAPED_LT) {
            Cow::Owned(text.replace(ESCAPED_LT, "<"))
        } else {
            Cow::Borrowed(text)
        }
    }
}

/// The bytes of `text` that `KeptScripts` compares: all but each CR.
#[cfg(feature = "metadata")]
fn without_cr(text: &str) -> Vec<u8> {
    text.bytes().filter(|byte| *byte != b'\r').collect()
}

/// A candidate raw-text element: `idx` is its `<`, `last` is the end of the copied input and
/// `name` is the tag name in lower case.
struct RawTextElement<'a> {
    input: &'a str,
    idx: usize,
    last: usize,
    name: &'a str,
    content: RawTextContent,
}

/// Remove or rewrite the raw-text element that starts at `element.idx`, appending the input
/// before it to `*output` (lazily allocated). `tag_end` is the end of its start tag;
/// `close_start` and `close_idx` are the two ends of its end tag.
///
/// A removed element leaves one space when the two characters around it are not white space.
/// A kept element is written by `push_kept_element` and adds no space: the walker reads the
/// white space on its two sides as if the element were not there.
///
/// Returns what it did. A `textarea` with no `<` needs no change and is not written again.
fn strip_raw_text_element(
    element: &RawTextElement<'_>,
    tag_end: usize,
    (close_start, close_idx): (usize, usize),
    output: &mut Option<String>,
) -> Outcome {
    let bytes = element.input.as_bytes();
    let name = element.name.as_bytes();
    let body = &element.input[tag_end..close_start];

    let kept = match element.content {
        RawTextContent::Removed => false,
        RawTextContent::RemovedUnlessStructuredData => is_json_ld_script_open_tag(&element.input[element.idx..tag_end]),
        // ~keep A `textarea` with no `<` in it is already one text for the parser.
        RawTextContent::Kept if !body.contains('<') => return Outcome::Unchanged,
        RawTextContent::Kept => true,
    };

    let out = output.get_or_insert_with(|| String::with_capacity(element.input.len()));
    out.push_str(&element.input[element.last..element.idx]);
    if kept {
        // ~keep A kept element adds no space of its own: the white space around it is the page's.
        let attributes = &element.input[element.idx + 1 + name.len()..tag_end];
        push_kept_element(out, element.name, attributes, body);
        return Outcome::Kept;
    }
    if element.idx > 0
        && close_idx < bytes.len()
        && !bytes[element.idx - 1].is_ascii_whitespace()
        && !bytes[close_idx].is_ascii_whitespace()
    {
        out.push(' ');
    }
    Outcome::Removed
}

/// Write a kept raw-text element in a form the parser reads as one element with one text.
/// `attributes` is the open tag after its name, with the `>`; `body` is the content.
///
/// ~keep The parser has no raw-text rule: a `<p>` in the content would open an element that
/// ~keep holds the rest of the page, so each `<` of the body is written as `&lt;`. Nothing else
/// ~keep of the body changes, so this function changes nothing in a body that it wrote: the
/// ~keep pass reads a repaired page as it reads the page of the author. `KeptScripts` gives the
/// ~keep text of a script as written. The parser also closes an element only with an end tag of
/// ~keep the same spelling, so the two tag names are written in one spelling, and it reads only
/// ~keep white space after the name, so a `/` or a form feed there is written as a space.
fn push_kept_element(out: &mut String, name: &str, attributes: &str, body: &str) {
    out.push('<');
    out.push_str(name);
    if matches!(attributes.as_bytes().first(), Some(b'/' | b'\x0C')) {
        out.push(' ');
        out.push_str(&attributes[1..]);
    } else {
        out.push_str(attributes);
    }
    let mut rest = body;
    while let Some(at) = memchr::memchr(b'<', rest.as_bytes()) {
        out.push_str(&rest[..at]);
        out.push_str(ESCAPED_LT);
        rest = &rest[at + 1..];
    }
    out.push_str(rest);
    out.push_str("</");
    out.push_str(name);
    out.push('>');
}

/// The raw-text elements the first pass reads, each with what happens to its content.
const FIRST_PASS_ELEMENTS: [(&str, RawTextContent); 3] = [
    ("script", RawTextContent::RemovedUnlessStructuredData),
    ("style", RawTextContent::Removed),
    ("textarea", RawTextContent::Kept),
];

/// The name of the raw-text element `FIRST_PASS_ELEMENTS[slot]` when the two passes remove
/// it (`script`, `style`). `None` for an element that is kept.
pub(super) fn removed_element_name(slot: usize) -> Option<&'static str> {
    let (name, content) = FIRST_PASS_ELEMENTS[slot];
    (content != RawTextContent::Kept).then_some(name)
}

/// Strip script and style tags and their content from HTML, and write the content of a
/// structured data script and of a `textarea` as text.
pub fn strip_script_and_style_tags(input: &str) -> Cow<'_, str> {
    strip_script_and_style_tags_with_kept_scripts(input).0
}

/// [`strip_script_and_style_tags`], with the kept scripts that the reader of the structured
/// data asks for the text as written.
pub fn strip_script_and_style_tags_with_kept_scripts(input: &str) -> (Cow<'_, str>, KeptScripts) {
    FirstPass::new(input, |_| {}).run()
}

/// A read of the first pass or of the second pass that a test counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PassRead {
    /// A `<` of the page.
    Markup,
    /// A search for the end tag of a raw-text element. It can read to the end of the page.
    EndTagSearch,
    /// A search for the end of a comment. It can read to the end of the page.
    CommentEndSearch,
    /// A read of a tag to its end, with the count of the bytes it went over. It can read to
    /// the end of the page.
    TagRead(usize),
    /// The count of the places where that read stopped to look at a byte: each `=` and the
    /// `>`. A read that looks at each byte of the tag stops at each byte.
    TagStops(usize),
}

/// The reads of one pass over one page that can go to the end of the page: the end of a
/// comment, the end of a tag and the end tag of a raw-text element. The first pass and the
/// second pass read a comment and a tag whole with it, so a tag in a comment or in an
/// attribute value starts nothing in either pass.
///
/// ~keep A read that finds no end is not made again, so the reads of a pass are linear. A later
/// ~keep comment has no end either, and a later start tag of the same name has no end tag
/// ~keep either. A tag that does not end holds the rest of the page, so a later `<` is in it:
/// ~keep it ends at its first `>`, as a tag with a quote that never ends.
pub(super) struct PageReads<C: FnMut(PassRead)> {
    comments_never_end: bool,
    tags_never_end: bool,
    no_end_tag: [bool; FIRST_PASS_ELEMENTS.len()],
    count: C,
}

impl<C: FnMut(PassRead)> PageReads<C> {
    pub(super) const fn new(count: C) -> Self {
        Self {
            comments_never_end: false,
            tags_never_end: false,
            no_end_tag: [false; FIRST_PASS_ELEMENTS.len()],
            count,
        }
    }

    /// The first end tag of the raw-text element `FIRST_PASS_ELEMENTS[slot]` at or after
    /// `from`: the position of its `<` and the position after its `>`.
    pub(super) fn raw_text_end_tag(&mut self, bytes: &[u8], from: usize, slot: usize) -> Option<(usize, usize)> {
        if self.no_end_tag[slot] {
            return None;
        }
        self.count(PassRead::EndTagSearch);
        let found = find_raw_text_end_tag(bytes, from, FIRST_PASS_ELEMENTS[slot].0.as_bytes());
        // ~keep A search that stopped at the scan limit did not read the whole page and proves
        // ~keep nothing.
        self.no_end_tag[slot] = found.is_none() && bytes.len() - from <= MAX_CLOSING_TAG_SCAN;
        found
    }

    /// Count one read of the pass.
    pub(super) fn count(&mut self, read: PassRead) {
        (self.count)(read);
    }

    /// The position after the comment that the `<` at `idx` starts, when it starts a comment
    /// that ends.
    pub(super) fn comment_end(&mut self, bytes: &[u8], idx: usize) -> Option<usize> {
        if self.comments_never_end || !bytes[idx..].starts_with(COMMENT_OPEN) {
            return None;
        }
        self.count(PassRead::CommentEndSearch);
        // ~keep `<!-->` and `<!--->` are whole comments: the search starts in the opening.
        let from = idx + 2;
        let found = memchr::memmem::find(&bytes[from..], COMMENT_CLOSE);
        self.comments_never_end = found.is_none();
        found.map(|offset| from + offset + COMMENT_CLOSE.len())
    }

    /// The position after the tag that the `<` at `idx` starts, read whole with its quoted
    /// attribute values. Returns `None` for a `<` of the text and for a tag with no `>`.
    pub(super) fn tag_end(&mut self, bytes: &[u8], idx: usize) -> Option<usize> {
        if !opens_a_tag(bytes, idx) {
            return None;
        }
        self.tag_end_from(bytes, idx + 1)
    }

    /// If the `<` at `lt` starts a raw-text element of `FIRST_PASS_ELEMENTS`, return its slot
    /// there and the position after its start tag, read whole with its quoted attribute
    /// values.
    pub(super) fn raw_text_start_tag_end(&mut self, bytes: &[u8], lt: usize) -> Option<(usize, usize)> {
        let (slot, after_name) = FIRST_PASS_ELEMENTS.iter().enumerate().find_map(|(slot, (name, _))| {
            let after_name = raw_text_tag_name_end(bytes, lt, name.as_bytes(), TagSide::Start)?;
            Some((slot, after_name))
        })?;
        let tag_end = self.tag_end_from(bytes, after_name)?;
        Some((slot, tag_end))
    }

    /// The position after the `>` that ends the tag read from `from`, a position in the tag
    /// after its `<`.
    fn tag_end_from(&mut self, bytes: &[u8], from: usize) -> Option<usize> {
        if !self.tags_never_end {
            let mut stops = 0;
            let whole = whole_tag_end(bytes, from, &mut stops);
            self.count(PassRead::TagRead(whole.unwrap_or(bytes.len()) - from));
            self.count(PassRead::TagStops(stops));
            self.tags_never_end = whole.is_none();
            if whole.is_some() {
                return whole;
            }
        }
        memchr::memchr(b'>', &bytes[from..]).map(|offset| from + offset + 1)
    }
}

/// The first pass over one page.
///
/// ~keep The pass reads markup as a tokenizer does: a comment whole, a tag to its end with its
/// ~keep quoted attribute values, a raw-text element to its end tag. The next `<` it tries is
/// ~keep after that, so a `<textarea>` in a comment or in an attribute value starts nothing.
/// ~keep A bogus comment is read to its first `>`: a `<textarea>` in `<? ... >` starts nothing.
/// ~keep The pass is linear: a search that finds no end is not made again. A later start tag
/// ~keep of the same name has no end tag either; `PageReads` has the rule of a comment and of
/// ~keep a tag.
pub(super) struct FirstPass<'a, C: FnMut(PassRead)> {
    input: &'a str,
    last: usize,
    output: Option<String>,
    svg_depth: usize,
    reads: PageReads<C>,
    kept_scripts: KeptScripts,
}

impl<'a, C: FnMut(PassRead)> FirstPass<'a, C> {
    pub(super) const fn new(input: &'a str, count: C) -> Self {
        Self {
            input,
            last: 0,
            output: None,
            svg_depth: 0,
            reads: PageReads::new(count),
            kept_scripts: KeptScripts::new(),
        }
    }

    pub(super) fn run(mut self) -> (Cow<'a, str>, KeptScripts) {
        let bytes = self.input.as_bytes();
        // ~keep No tag ends after the last `>` of the page: nothing after it is an element,
        // ~keep and no search for the end of a tag starts there.
        let last_gt = memchr::memrchr(b'>', bytes).unwrap_or(0);
        let mut idx = 0;
        while idx < last_gt {
            let Some(offset) = memchr::memchr(b'<', &bytes[idx..last_gt]) else {
                break;
            };
            idx = self.read_markup(idx + offset);
        }
        let page = match self.output {
            Some(mut out) => {
                out.push_str(&self.input[self.last..]);
                Cow::Owned(out)
            }
            None => Cow::Borrowed(self.input),
        };
        (page, self.kept_scripts)
    }

    /// Read the markup that the `<` at `idx` starts. Returns the position of the next read.
    fn read_markup(&mut self, idx: usize) -> usize {
        let bytes = self.input.as_bytes();
        self.reads.count(PassRead::Markup);
        if let Some(after_tag) = track_svg_tag(bytes, idx, &mut self.svg_depth) {
            return after_tag;
        }
        if self.svg_depth > 0 {
            return idx + 1;
        }
        if bytes[idx..].starts_with(COMMENT_OPEN) {
            return self.reads.comment_end(bytes, idx).unwrap_or(idx + 1);
        }
        if let Some(after_comment) = bogus_comment_end(bytes, idx) {
            return after_comment;
        }
        if let Some((slot, tag_end)) = self.reads.raw_text_start_tag_end(bytes, idx) {
            return self.read_raw_text_element(idx, slot, tag_end);
        }
        self.reads.tag_end(bytes, idx).unwrap_or(idx + 1)
    }

    /// Read the raw-text element of `FIRST_PASS_ELEMENTS[slot]` whose start tag is at `idx`
    /// and ends at `tag_end`. Returns the position of the next read.
    fn read_raw_text_element(&mut self, idx: usize, slot: usize, tag_end: usize) -> usize {
        let bytes = self.input.as_bytes();
        let (name, content) = FIRST_PASS_ELEMENTS[slot];
        let Some(close) = self.reads.raw_text_end_tag(bytes, tag_end, slot) else {
            // ~keep The second pass reads an element with no end tag.
            #[cfg(feature = "metadata")]
            {
                if content == RawTextContent::RemovedUnlessStructuredData
                    && !self.kept_scripts.has_no_end_tag
                    && is_json_ld_script_open_tag(&self.input[idx..tag_end])
                {
                    self.kept_scripts.has_no_end_tag = true;
                    self.kept_scripts.record(&self.input[tag_end..]);
                }
            }
            return tag_end;
        };
        let element = RawTextElement {
            input: self.input,
            idx,
            last: self.last,
            name,
            content,
        };
        let outcome = strip_raw_text_element(&element, tag_end, close, &mut self.output);
        if outcome != Outcome::Unchanged {
            self.last = close.1;
        }
        #[cfg(feature = "metadata")]
        {
            if outcome == Outcome::Kept && content == RawTextContent::RemovedUnlessStructuredData {
                self.kept_scripts.record(&self.input[tag_end..close.0]);
            }
        }
        close.1
    }
}

/// Upper bound on how far a closing-tag scan may travel from its start offset.
///
/// ~keep DoS guard: hostile input can contain an unterminated element followed
/// ~keep by hundreds of megabytes of text; the scan gives up rather than walking
/// ~keep the whole document for every such tag.
const MAX_CLOSING_TAG_SCAN: usize = 100_000_000;

/// The side of an element that a tag is on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TagSide {
    /// `<name ...>`
    Start,
    /// `</name ...>`
    End,
}

/// If the `<` at `lt` starts the start tag of the raw-text element `name`, return the index
/// after the `>` of the tag. A scan that skips a raw-text element reads its start tag here.
///
/// ~keep A `>` in a quoted attribute value does not end a start tag. A start tag with a quote
/// ~keep that never ends ends at its first `>`. A pass reads many tags of one page: it reads
/// ~keep them with `PageReads`, which has this rule and makes no read twice.
fn raw_text_start_tag_end(bytes: &[u8], lt: usize, name: &[u8]) -> Option<usize> {
    let after_name = raw_text_tag_name_end(bytes, lt, name, TagSide::Start)?;
    let first_bracket = || memchr::memchr(b'>', &bytes[after_name..]).map(|offset| after_name + offset + 1);
    whole_tag_end(bytes, after_name, &mut 0).or_else(first_bracket)
}

/// The one rule for the name of the start tag and of the end tag of a raw-text element. If
/// the `<` at `lt` starts a tag of `name` on `side`, return the position after the name.
///
/// ~keep The HTML rule: the name in any case, then tab, LF, FF, CR, space, `/` or `>`. The
/// ~keep two passes, the scan of hidden regions and the fast converter read a tag with this
/// ~keep function; a second copy of the rule is how they came to disagree on the form feed
/// ~keep and the slash.
fn raw_text_tag_name_end(bytes: &[u8], lt: usize, name: &[u8], side: TagSide) -> Option<usize> {
    if bytes.get(lt) != Some(&b'<') {
        return None;
    }
    let name_start = match side {
        TagSide::Start => lt + 1,
        TagSide::End if bytes.get(lt + 1) == Some(&b'/') => lt + 2,
        TagSide::End => return None,
    };
    let after_name = name_start + name.len();
    if !bytes.get(name_start..after_name)?.eq_ignore_ascii_case(name) {
        return None;
    }
    if !matches!(
        bytes.get(after_name)?,
        b'\t' | b'\n' | b'\x0C' | b'\r' | b' ' | b'/' | b'>'
    ) {
        return None;
    }
    Some(after_name)
}

/// The position after the `>` that ends the tag read from `from`, a position in the tag after
/// its `<`. Returns `None` when a value or the tag does not end. `stops` counts the places
/// where the read stops to look at a byte.
///
/// ~keep The HTML rule: a quote opens a value only as the first byte of the value, after `=`
/// ~keep and white space. A value with no quotes ends at white space or at `>`: the apostrophe
/// ~keep of `title=it's` and the `=` and the apostrophe of `href=/search?q='tis` are bytes of
/// ~keep the value.
/// ~keep Outside a value only `>` and `=` change the read, so the read jumps to the next one of
/// ~keep the two: most bytes of a page are in a tag or between two tags, and a read of each
/// ~keep byte of each tag is a cost on every page.
fn whole_tag_end(bytes: &[u8], from: usize, stops: &mut usize) -> Option<usize> {
    let mut idx = from;
    loop {
        *stops += 1;
        idx += memchr::memchr2(b'>', b'=', bytes.get(idx..)?)?;
        if bytes[idx] == b'>' {
            return Some(idx + 1);
        }
        let after_equals = &bytes[idx + 1..];
        let value = idx + 1 + after_equals.iter().position(|byte| !is_tag_white_space(*byte))?;
        idx = match bytes[value] {
            b'>' => return Some(value + 1),
            quote @ (b'"' | b'\'') => value + 2 + memchr::memchr(quote, &bytes[value + 1..])?,
            _ => value + bytes[value..].iter().copied().position(ends_an_unquoted_value)?,
        };
    }
}

/// Whether `byte` is white space in a tag: tab, LF, FF, CR or space.
const fn is_tag_white_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}

/// Whether `byte` ends an attribute value with no quotes: white space or `>`.
const fn ends_an_unquoted_value(byte: u8) -> bool {
    is_tag_white_space(byte) || byte == b'>'
}

/// Find the position of the FIRST closing tag in bytes, ignoring nesting.
/// Returns the position AFTER the closing tag (including the '>').
/// This is highly optimized for performance and uses a fast-path scan.
///
/// ~keep First-match (not depth-aware) is correct ONLY for the HTML raw-text
/// ~keep elements `<script>` and `<style>`: inside their bodies `<script` is
/// ~keep text, not a tag, so the first `</script>` is the spec terminator.
/// ~keep Counting depth here would treat `var s = "<script>"` as an opening tag
/// ~keep and run past the real close, swallowing the rest of the document.
/// ~keep Elements that can nest must use `find_closing_tag_bytes_nested`; do not
/// ~keep unify the two functions.
#[inline]
pub fn find_closing_tag_bytes(bytes: &[u8], start: usize, tag: &[u8]) -> Option<usize> {
    find_raw_text_end_tag(bytes, start, tag).map(|(_, end)| end)
}

/// The first end tag of the raw-text element `name` at or after `start`: the position of its
/// `<` and the position after its `>`. Both converters end a raw-text element here.
pub fn find_raw_text_end_tag(bytes: &[u8], start: usize, name: &[u8]) -> Option<(usize, usize)> {
    let mut idx = start;
    while idx < bytes.len() && (idx - start) < MAX_CLOSING_TAG_SCAN {
        idx += memchr::memchr(b'<', &bytes[idx..])?;
        if let Some(after_name) = raw_text_tag_name_end(bytes, idx, name, TagSide::End) {
            // ~keep An end tag with no `>` ends the search: no later end tag has one either.
            let end = after_name + memchr::memchr(b'>', &bytes[after_name..])? + 1;
            return Some((idx, end));
        }
        idx += 1;
    }
    None
}

/// Void elements that never carry a closing tag, so an occurrence of one inside
/// a subtree must not raise the nesting depth of a closing-tag scan.
const VOID_ELEMENT_NAMES: [&[u8]; 4] = [b"br", b"hr", b"img", b"input"];

/// Whether an opening tag closes itself: XHTML-style `<tag/>` or a void element.
///
/// ~keep Shared by `strip_hidden_elements` and `find_closing_tag_bytes_nested`
/// ~keep so the element the stripper treats as self-closing and the element the
/// ~keep depth counter refuses to count can never disagree.
#[inline]
pub(super) fn is_self_closing_tag(tag_slice: &[u8], tag_name: &[u8]) -> bool {
    tag_slice.ends_with(b"/>")
        || VOID_ELEMENT_NAMES
            .iter()
            .any(|void_name| tag_name.eq_ignore_ascii_case(void_name))
}

/// Byte sequences delimiting the regions of an HTML document whose bytes are
/// character data rather than markup.
const COMMENT_OPEN: &[u8] = b"<!--";
const COMMENT_CLOSE: &[u8] = b"-->";
const CDATA_OPEN: &[u8] = b"<![CDATA[";
const CDATA_CLOSE: &[u8] = b"]]>";

/// Elements whose body is raw text: `<` inside them never starts a tag.
///
/// ~keep The names of `FIRST_PASS_ELEMENTS`: a scan that skips a region and the first pass
/// ~keep read the same elements as raw text.
const RAW_TEXT_ELEMENT_NAMES: [&[u8]; 3] = [b"script", b"style", b"textarea"];

/// Index just past the first occurrence of `terminator` at or after `from`,
/// or the input length when the region is never terminated.
///
/// ~keep An unterminated comment/CDATA region runs to EOF per HTML5, so
/// ~keep returning `len` (rather than `None`) keeps the caller from resuming a
/// ~keep tag scan inside text that will never be markup.
#[inline]
fn find_sequence_end(bytes: &[u8], from: usize, terminator: &[u8]) -> usize {
    let len = bytes.len();
    if from >= len {
        return len;
    }
    match bytes[from..].windows(terminator.len()).position(|w| w == terminator) {
        Some(offset) => from + offset + terminator.len(),
        None => len,
    }
}

/// If a `<` at `idx` opens a region that is NOT markup — an HTML comment, a
/// CDATA section, or the body of a raw-text element — return the index just
/// past that region. Returns `None` when `idx` does not open such a region.
///
/// ~keep A depth-counting close-tag scan MUST consult this before interpreting
/// ~keep a `<`: `<!-- </div> -->` and `<script>{"a":"</div>"}</script>` contain
/// ~keep byte sequences that look like tags but are text. Counting them ends the
/// ~keep scan early (leaking the hidden element's tail into the output) or
/// ~keep inflates the depth (swallowing the following visible sibling).
#[inline]
pub fn skip_opaque_region(bytes: &[u8], idx: usize) -> Option<usize> {
    let len = bytes.len();
    if idx >= len || bytes[idx] != b'<' {
        return None;
    }

    if bytes[idx..].starts_with(COMMENT_OPEN) {
        return Some(find_sequence_end(bytes, idx + COMMENT_OPEN.len(), COMMENT_CLOSE));
    }

    if bytes[idx..].starts_with(CDATA_OPEN) {
        return Some(find_sequence_end(bytes, idx + CDATA_OPEN.len(), CDATA_CLOSE));
    }

    for raw_text_name in RAW_TEXT_ELEMENT_NAMES {
        if let Some(open_end) = raw_text_start_tag_end(bytes, idx, raw_text_name) {
            // ~keep First-match close is the spec terminator for raw text — see
            // ~keep the doc comment on `find_closing_tag_bytes`.
            return Some(find_closing_tag_bytes(bytes, open_end, raw_text_name).unwrap_or(len));
        }
    }

    None
}

/// Whether the `<` at `idx` starts a tag rather than a stray `<` in text.
///
/// ~keep Mirrors the `is_valid_tag` test in `preprocess_html`: without it a
/// ~keep literal `<` in text (`a < b`) would make a scan jump to the next `>`,
/// ~keep stepping over the real closing tag in between.
#[inline]
pub(super) fn opens_a_tag(bytes: &[u8], idx: usize) -> bool {
    match bytes.get(idx + 1) {
        Some(b'/' | b'!') => bytes.get(idx + 2).is_some_and(u8::is_ascii_alphabetic),
        Some(byte) => byte.is_ascii_alphabetic(),
        None => false,
    }
}

/// Find the closing tag that matches an opening `<tag>`, counting nested
/// elements of the same name. Returns the position AFTER the closing tag
/// (including the '>'), or `None` if the element is never closed.
///
/// ~keep Required for elements that can nest (`div`, `span`, ...): the
/// ~keep first-match `find_closing_tag_bytes` stops at the INNER `</tag>`, which
/// ~keep leaves the hidden element's tail in the document. Kept separate from
/// ~keep that function because raw-text elements must NOT count depth — see its
/// ~keep doc comment.
///
/// ~keep Only real markup may move the depth counter, so every `<` is first
/// ~keep offered to `skip_opaque_region`, and a tag that is not the target is
/// ~keep jumped via the quote-aware `find_tag_end` rather than byte-by-byte:
/// ~keep otherwise `</tag>` written inside a comment, a quoted attribute value,
/// ~keep or a raw-text body is mistaken for the element's terminator.
#[inline]
pub fn find_closing_tag_bytes_nested(bytes: &[u8], start: usize, tag: &[u8]) -> Option<usize> {
    let len = bytes.len();
    if tag.is_empty() {
        return None;
    }

    let mut idx = start;
    let mut depth = 1usize;

    while idx < len && (idx - start) < MAX_CLOSING_TAG_SCAN {
        if bytes[idx] != b'<' {
            match memchr::memchr(b'<', &bytes[idx..]) {
                Some(pos) => idx += pos,
                None => break,
            }
        }

        if let Some(region_end) = skip_opaque_region(bytes, idx) {
            idx = region_end;
            continue;
        }

        if matches_end_tag_start(bytes, idx + 1, tag) {
            if let Some(close_end) = find_tag_end(bytes, idx + 2 + tag.len()) {
                depth -= 1;
                if depth == 0 {
                    return Some(close_end);
                }
                idx = close_end;
                continue;
            }
        } else if matches_tag_start(bytes, idx + 1, tag) {
            if let Some(open_end) = find_tag_end(bytes, idx + 1 + tag.len()) {
                if !is_self_closing_tag(&bytes[idx..open_end], tag) {
                    depth += 1;
                }
                idx = open_end;
                continue;
            }
        } else if opens_a_tag(bytes, idx) {
            if let Some(unrelated_tag_end) = find_tag_end(bytes, idx + 1) {
                idx = unrelated_tag_end;
                continue;
            }
        }

        idx += 1;
    }

    None
}

// ~keep Match each same-name nesting stack once; repeated hidden opens without closes
// otherwise search the entire suffix independently (#773).
pub(super) fn matching_tag_ends(bytes: &[u8]) -> ahash::AHashMap<usize, Option<usize>> {
    let mut stacks: ahash::AHashMap<Vec<u8>, Vec<usize>> = ahash::AHashMap::new();
    let mut ends = ahash::AHashMap::new();
    let mut idx = 0;
    let last_gt = bytes.iter().rposition(|&byte| byte == b'>').unwrap_or(0);
    while idx < last_gt {
        let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) else {
            break;
        };
        idx += offset;
        if let Some(end) = skip_opaque_region(bytes, idx) {
            idx = end;
            continue;
        }
        if !opens_a_tag(bytes, idx) {
            idx += 1;
            continue;
        }
        let closing = bytes.get(idx + 1) == Some(&b'/');
        let start = idx + if closing { 2 } else { 1 };
        let name_len = bytes[start..]
            .iter()
            .take_while(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
            .count();
        let name = bytes[start..start + name_len].to_ascii_lowercase();
        let Some(end) = find_tag_end(bytes, start + name_len) else {
            break;
        };
        let stack = stacks.entry(name.clone()).or_default();
        if closing {
            if let Some(open) = stack.pop() {
                ends.insert(open, Some(end));
            }
        } else if !is_self_closing_tag(&bytes[idx..end], &name) {
            ends.insert(idx, None);
            stack.push(idx);
        }
        idx = end;
    }
    ends
}
