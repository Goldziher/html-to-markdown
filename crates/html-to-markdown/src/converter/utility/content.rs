//! Content extraction and manipulation utilities.
//!
//! Functions for extracting and processing element content, including text collection
//! and empty element detection.

use crate::converter::utility::escaping::is_block_level_name;
use crate::converter::utility::white_space::{FORM_FEED, is_collapsible, mark_edges, owed_part};
use crate::options::HiddenContent;
use crate::text;
use std::borrow::Cow;
#[cfg(feature = "visitor")]
use std::collections::BTreeMap;

pub use crate::converter::DomContext;

/// Collect all attributes from an HTML tag as a `BTreeMap<String, String>`.
///
/// Boolean attributes (those with `None` as the value) are skipped; only
/// attributes that carry an explicit value are included.
#[cfg(feature = "visitor")]
pub fn collect_tag_attributes(tag: &tl::HTMLTag) -> BTreeMap<String, String> {
    tag.attributes()
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|val| (k.to_string(), val.to_string())))
        .collect()
}

/// Chomp whitespace from inline element content, preserving line breaks.
///
/// Similar to `text::chomp` but handles line breaks from `<br>` tags specially.
/// Line breaks are extracted as suffix to be placed outside formatting.
/// Returns (prefix, suffix, `trimmed_text`).
pub fn chomp_inline(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let has_trailing_linebreak = text.ends_with("  \n") || text.ends_with("\\\n");

    if text.trim().is_empty() && !has_trailing_linebreak {
        // ~keep A whitespace-only body (e.g. `<i> </i>`) is ONE space in the source, not
        // ~keep two: the starts_with/ends_with checks below would treat the very same run
        // ~keep as both prefix AND suffix, and every caller's "push prefix, then
        // ~keep append_inline_suffix(suffix)" shape then emits it twice (issue #481).
        // ~keep Represented once, as a prefix (with an empty suffix so
        // ~keep `append_inline_suffix` is a no-op), every existing caller naturally
        // ~keep collapses back to a single space. A hard trailing linebreak is excluded
        // ~keep above and keeps the full logic below -- it is not interchangeable with a
        // ~keep plain space. A body that is only a newline counts too: that is what a
        // ~keep `<br>` with nothing before it leaves in a wrapper's scratch buffer
        // ~keep (`line_break.rs`'s block-start arm), and `<b>Alpha</b><b><br></b><b>Beta</b>`
        // ~keep joined its words when it was worth nothing (issue #502).
        // ~keep A form feed is a character: a body with one is written as it is, with no mark.
        return (if text.contains(FORM_FEED) { text } else { " " }, "", "");
    }

    // ~keep A form feed at an edge goes outside the marks with the white space of that edge: a
    // ~keep mark beside a form feed is no mark (issue #811).
    let (start_run, end_run) = mark_edges(text);
    let prefix = if start_run.contains(FORM_FEED) {
        start_run
    } else if text.starts_with(&[' ', '\t'][..]) {
        " "
    } else {
        ""
    };

    let suffix = if has_trailing_linebreak {
        if text.ends_with("  \n") { "  \n" } else { "\\\n" }
    } else if end_run.contains(FORM_FEED) {
        end_run
    } else if text.ends_with(&[' ', '\t'][..]) {
        " "
    } else {
        ""
    };

    let trimmed = if has_trailing_linebreak {
        text.strip_suffix("  \n").map_or_else(
            || text.strip_suffix("\\\n").map_or_else(|| text.trim(), |s| s.trim()),
            |s| s.trim(),
        )
    } else {
        text.trim()
    };

    (prefix, suffix, trimmed)
}

/// The white space that the content of a link label gives up at its two ends, and the label
/// without it: (prefix, suffix, label). Every writer of a link with a label asks here, in both
/// converters, with the bytes of the label as that converter wrote them.
///
/// ~keep The white space at an edge of the label goes outside the link, as it does for bold
/// ~keep and italic text: `[Go ](/p)now` shows as `Go now`, and a trimmed label joins the words.
/// ~keep White space here is `char::is_whitespace`, the one definition of both converters: a
/// ~keep no-break space, a narrow one or an em space at an edge is the one space of that edge, as
/// ~keep it is at the edge of a text ([`white_space::edges`]) and of an autolink
/// ~keep ([`link_text_edges`]). A form feed is a character (issue #811): the white space of an
/// ~keep edge that holds one goes outside the link as it is, as [`chomp_inline`] gives it for
/// ~keep bold and italic text. A line end is not white space of an edge: a line break there is
/// ~keep content of the label (issue #497), and that side is given back whole, with the white
/// ~keep space around the break for [`normalize_link_label`] to drop. Each edge is decided on its
/// ~keep own: `[  \nGo ](/p)now` shows `Go now` after the break, so the end still gives its space.
///
/// [`white_space::edges`]: crate::converter::utility::white_space::edges
pub fn link_label_edges(content: &str) -> (&str, &str, &str) {
    let is_edge_space = |character: char| character.is_whitespace() && character != '\n';
    let after_start = content.trim_start_matches(is_edge_space);
    let starts_at_break = after_start.starts_with('\n') || after_start.starts_with("\\\n");
    let start = if starts_at_break {
        0
    } else {
        content.len() - after_start.len()
    };
    let before_end = content[start..].trim_end_matches(is_edge_space);
    let end = if before_end.ends_with('\n') {
        content.len()
    } else {
        start + before_end.len()
    };
    let prefix = edge_space(&content[..start]);
    let suffix = edge_space(&content[end..]);
    (prefix, suffix, &content[start..end])
}

/// The white space `run` of a label edge as it is written outside the link: one space, or the
/// run as it is when it holds a form feed.
fn edge_space(run: &str) -> &str {
    if run.contains(FORM_FEED) {
        run
    } else {
        space_if(!run.is_empty())
    }
}

/// The white space at the two ends of the text of a link, as one space for each end:
/// (prefix, suffix).
///
/// ~keep A link in the `<address>` form writes its address, not its content. The text that
/// ~keep made it that form says what white space it has at its ends, so the content is not
/// ~keep walked: an element in it writes nothing and is not collected. A `<br>` is no text.
/// ~keep A no-break space is white space here, as it is where the text is trimmed.
/// ~keep White space with a `<br>` between it and the end of the link is no white space of that
/// ~keep end: a label drops the white space around a line break at its edge, and so does this.
pub fn link_text_edges(link_text: &LinkText) -> (&'static str, &'static str) {
    let text = link_text.text.as_str();
    let starts_at_break = link_text
        .breaks
        .is_some_and(|(first, _)| first <= text.len() - text.trim_start().len());
    let ends_at_break = link_text.breaks.is_some_and(|(_, last)| last >= text.trim_end().len());
    let is_space = |edge: Option<char>| edge.is_some_and(char::is_whitespace);
    (
        space_if(!starts_at_break && is_space(text.chars().next())),
        space_if(!ends_at_break && is_space(text.chars().next_back())),
    )
}

const fn space_if(owed: bool) -> &'static str {
    if owed { " " } else { "" }
}

/// The text of a link, as the walk of its descendants reads it.
#[derive(Debug, Clone, Default)]
pub struct LinkText {
    /// The text of the text nodes, in document order.
    pub text: String,
    /// Where the first and the last `<br>` stand in `text`, as byte offsets. A `<br>` is no
    /// text: it adds nothing to `text`.
    pub breaks: Option<(usize, usize)>,
}

/// Merge a newly-opening emphasis delimiter into the matching close marker `output` already
/// ends with, instead of emitting a second, textually-adjacent delimiter run.
///
/// `CommonMark` parses `*A**B*` as `A` in emphasis followed by a literal `**B*` — NOT as two
/// consecutive emphasis runs — because a closing delimiter run immediately followed by an
/// opening one of the same character forms a single, longer run (issue #483). Sibling
/// `<i>`/`<b>` elements that each independently wrap their own content in `*…*`/`**…**`
/// therefore corrupt on reparse unless the second element's open marker is suppressed and the
/// previous element's close marker is removed, letting the merged run share one pair of
/// delimiters (`<i>A</i><i>B</i>` -> `*AB*`, not `*A**B*`).
///
/// Pops exactly `count` trailing copies of `symbol` from `output` and returns `true` only when:
/// - `output` ends with a run of `symbol` whose length is EXACTLY `count` (not more, not
///   fewer) -- so `***x***` (a real 3-run) is left alone rather than half-eaten, and
/// - the character immediately preceding that run (if any) is neither `symbol` nor `\` -- so
///   a longer run one byte further back, or a backslash-escaped literal (`\*`), is left alone.
///
/// Returns `false` and leaves `output` untouched otherwise, including when `count == 0` (no
/// delimiter to merge, e.g. a `<strong>` nested inside another `<strong>`, which emits no
/// marker of its own).
pub fn merge_adjacent_emphasis(output: &mut String, symbol: char, count: usize) -> bool {
    if count == 0 {
        return false;
    }

    let mut rev = output.chars().rev();
    for _ in 0..count {
        match rev.next() {
            Some(c) if c == symbol => {}
            _ => return false,
        }
    }
    if let Some(preceding) = rev.next() {
        if preceding == symbol || preceding == '\\' {
            return false;
        }
    }

    let new_len = output.len() - symbol.len_utf8() * count;
    output.truncate(new_len);
    true
}

/// Get the text content of a node and its children.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn get_text_content(node_handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> String {
    dom_ctx.text_content(*node_handle, parser)
}

/// Writes the white space that an inline element starts with (`prefix` of [`chomp_inline`])
/// before the marks of the element, when a space is owed there.
///
/// ~keep `one <b> two</b>` has white space on both sides of the element start. It is one run,
/// ~keep so it is one space, and it goes outside the marks: `** two**` is not strong text.
/// ~keep Where the source white space is kept (`keeps_source_white_space`), the prefix is always written.
pub fn push_inline_prefix(
    output: &mut String,
    prefix: &str,
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::context::Context,
) {
    if keeps_source_white_space(options, ctx) {
        output.push_str(prefix);
    } else {
        // ~keep A form feed in the prefix is a character: it is written where no space is owed too.
        let owed = owed_part(output, prefix);
        output.push_str(owed);
    }
}

/// Whether the white space of the source is kept: in the strict white space mode, and in code
/// (`<pre>`, `<code>`, `<kbd>`, `<samp>`). No rule that collapses white space or moves it out of
/// an element applies there. Every such rule of the full converter asks here.
#[must_use]
pub const fn keeps_source_white_space(
    options: &crate::options::ConversionOptions,
    ctx: &crate::converter::context::Context,
) -> bool {
    ctx.in_code || matches!(options.whitespace_mode, crate::options::WhitespaceMode::Strict)
}

/// Whether an inline element is code (`<code>`, `<kbd>`, `<samp>`): its content is written with
/// [`keeps_source_white_space`], so a rule for running text does not look into it either.
#[must_use]
pub fn is_inline_code(tag_name: &str) -> bool {
    matches!(tag_name, "code" | "kbd" | "samp")
}

/// The element that follows a line end of the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextElement {
    /// A block, or an element that is not inline.
    Block,
    /// An inline element.
    Inline,
    /// An inline element whose content starts with a block (`<a href="/y"><h3>y</h3></a>`).
    InlineAroundBlock,
}

impl NextElement {
    /// The kind of an element from the two facts that each converter reads in its own way.
    #[must_use]
    pub const fn new(is_inline: bool, content_starts_with_block: bool) -> Self {
        match (is_inline, content_starts_with_block) {
            (true, true) => Self::InlineAroundBlock,
            (true, false) => Self::Inline,
            (false, _) => Self::Block,
        }
    }
}

/// What one line end of the source becomes before the element that follows it: a space in
/// running text and before an inline element, a line end before a block. Both converters ask
/// here, so a line end is the same space in a paragraph, a `<div>`, a list item and the root.
///
/// ~keep An inline element whose content starts with a block starts a line of its own in a
/// ~keep browser, so the line end before it stays a line end, as before a block.
#[must_use]
pub const fn line_end_before_element(in_running_text: bool, next: NextElement) -> char {
    if in_running_text || matches!(next, NextElement::Inline) {
        ' '
    } else {
        '\n'
    }
}

/// The source text `raw` without the white space at its end, when that white space holds a
/// line end and other text is before it.
///
/// ~keep Tier-1 gives its text with the references to white space already written as
/// ~keep characters (`with_white_space_references_decoded`), so `one&#10;` and `one\n&#32;`
/// ~keep end with a line end here as `one\n` does. This function decodes nothing: one decoder
/// ~keep reads a reference, and `&amp;#10;` stays the text `&#10;`.
#[must_use]
pub fn without_trailing_line_end_in_source(raw: &str) -> Option<&str> {
    let kept = raw.trim_end_matches(is_collapsible);
    (raw[kept.len()..].contains('\n') && !kept.is_empty()).then_some(kept)
}

/// Whether the source text `raw` ends with a zero-width space, written as the character or as
/// a character reference (`one&#8203;`, `one&#x200B;`, `one&ZeroWidthSpace;`).
///
/// ~keep Tier-1 asks for the character before a line end (rule 2 of `white_space`); Tier-2 reads
/// ~keep the last character of its decoded text. Only the text from the last `&` on is decoded:
/// ~keep a reference ends where the text ends, and `&amp;#8203;` is the text `&#8203;`.
#[must_use]
pub fn ends_with_zero_width_space_in_source(raw: &str) -> bool {
    raw.rfind('&').map_or_else(
        || raw.ends_with(ZERO_WIDTH_SPACE),
        |start| text::decode_html_entities_cow(&raw[start..]).ends_with(ZERO_WIDTH_SPACE),
    )
}

/// Whether an element only wraps its text: it writes no content of its own, so the text after
/// a line end can start inside it or after it when it is empty.
#[must_use]
pub fn is_text_wrapper(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "abbr"
            | "b"
            | "bdi"
            | "bdo"
            | "cite"
            | "code"
            | "data"
            | "del"
            | "dfn"
            | "em"
            | "i"
            | "ins"
            | "kbd"
            | "mark"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strike"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "u"
            | "var"
    )
}

/// The zero-width space: a place where a line can break, not a space.
pub const ZERO_WIDTH_SPACE: char = '\u{200b}';

/// Determine whether a node is block-level, preferring the DOM context's precomputed tag
/// info when available and falling back to a name-based check otherwise.
///
/// ~keep Shared between `collect_link_label_text` (topmost block *descendants*, for
/// ~keep skipping block content out of an inline label) and `handlers/link.rs`'s
/// ~keep direct-children partition (issue #490) -- both need the identical block/inline
/// ~keep classification, or a byte could end up assigned to neither half, or both.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn node_is_block_level(handle: &tl::NodeHandle, parser: &tl::Parser, dom_ctx: &DomContext) -> bool {
    let Some(tl::Node::Tag(tag)) = handle.get(parser) else {
        return false;
    };
    dom_ctx.tag_info(handle.get_inner(), parser).map_or_else(
        || {
            let tag_name = normalized_tag_name(tag.name().as_utf8_str());
            is_block_level_element(tag_name.as_ref())
        },
        |info| info.is_block,
    )
}

/// Collect inline text for link labels, skipping block-level descendants.
pub fn collect_link_label_text(
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    hidden_content: HiddenContent,
) -> (LinkText, Vec<tl::NodeHandle>, bool) {
    let mut block_nodes = Vec::new();
    let text = walk_link_text(children, parser, dom_ctx, hidden_content, |handle| {
        block_nodes.push(handle);
        false
    });
    let saw_block = !block_nodes.is_empty();
    (text, block_nodes, saw_block)
}

/// The text of every descendant of a link, block-level ones included.
pub fn link_text_content(
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    hidden_content: HiddenContent,
) -> LinkText {
    walk_link_text(children, parser, dom_ctx, hidden_content, |_| true)
}

/// The name of a link apart from its content: its `aria-label`, else its `title`. A value of
/// white space only is no name.
///
/// ~keep The order is the order of a browser's accessible name. Both converters ask this one
/// ~keep function: the full one to label a link whose content gives no text, the fast one to
/// ~keep leave such a link to the full one.
pub fn link_accessible_name<'a>(aria_label: Option<&'a str>, title: Option<&'a str>) -> Option<&'a str> {
    [aria_label, title]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|name| !name.is_empty())
}

/// Walks the descendants of a link for their text. `enter_block` gets each topmost block-level
/// element and says whether to read inside it.
///
/// ~keep An inline `<svg>` gives the text a reader gets from it, not its text nodes: those hold its
/// ~keep style sheets and scripts, which became the label of an icon link.
#[allow(clippy::match_wildcard_for_single_variants)]
fn walk_link_text(
    children: &[tl::NodeHandle],
    parser: &tl::Parser,
    dom_ctx: &DomContext,
    hidden_content: HiddenContent,
    mut enter_block: impl FnMut(tl::NodeHandle) -> bool,
) -> LinkText {
    let mut text = String::new();
    let mut breaks: Option<(usize, usize)> = None;
    let mut stack: Vec<_> = children.iter().rev().copied().collect();

    while let Some(handle) = stack.pop() {
        let Some(node) = handle.get(parser) else {
            continue;
        };
        match node {
            tl::Node::Raw(bytes) => {
                let raw = bytes.as_utf8_str();
                let decoded = text::decode_html_entities_cow(raw.as_ref());
                text.push_str(decoded.as_ref());
            }
            tl::Node::Tag(tag) if tag.name().as_utf8_str().eq_ignore_ascii_case("svg") => {
                text.push_str(&crate::converter::media::svg::graphic_text(tag, parser, hidden_content));
            }
            tl::Node::Tag(tag) if tag.name().as_utf8_str().eq_ignore_ascii_case("br") => {
                breaks = Some((breaks.map_or(text.len(), |(first, _)| first), text.len()));
            }
            tl::Node::Tag(tag) => {
                if !node_is_block_level(&handle, parser, dom_ctx) || enter_block(handle) {
                    push_label_children(&mut stack, handle, tag, dom_ctx);
                }
            }
            _ => {}
        }
    }
    LinkText { text, breaks }
}

fn push_label_children(
    stack: &mut Vec<tl::NodeHandle>,
    handle: tl::NodeHandle,
    tag: &tl::HTMLTag<'_>,
    dom_ctx: &DomContext,
) {
    if let Some(children) = dom_ctx.children_of(handle.get_inner()) {
        stack.extend(children.iter().rev().copied());
        return;
    }
    let mut children: Vec<_> = tag.children().top().iter().copied().collect();
    children.reverse();
    stack.extend(children);
}

/// The two hard-line-break markers `block/line_break.rs` can emit for a real `<br>`:
/// `"  \n"` for `NewlineStyle::Spaces`, `"\\\n"` for `NewlineStyle::Backslash`.
const HARD_BREAK_MARKERS: [&str; 2] = ["  \n", "\\\n"];

/// Normalize a link label by collapsing incidental newlines/whitespace while preserving an
/// explicit hard line break (`<br>`) that appears mid-label.
///
/// A hard line break inside a link's visible text is legal `CommonMark` (`[foo  \nbar](url)`),
/// so collapsing it unconditionally is lossy: convert to Markdown, render that back to HTML,
/// and convert again, and the `<br>` that survived the round trip disappears on the second
/// pass. Only the two exact marker shapes `block/line_break.rs` emits for a real `<br>` are
/// preserved; every other newline (soft line breaks from wrapped source text, `\r`) still
/// collapses to a single space, matching the previous behaviour.
///
/// ~keep This scans for the two marker substrings directly and copies everything else through
/// ~keep the ordinary whitespace-collapsing rules, rather than swapping the markers out for a
/// ~keep placeholder character and restoring them afterward. A placeholder scheme is unsound
/// ~keep here: it assumes an injective mapping over a character set the label cannot contain,
/// ~keep and that is false for arbitrary HTML input. An earlier version used Private Use Area
/// ~keep code points as placeholders on the reasoning that "no producer this crate parses
/// ~keep assigns them" -- but icon fonts do exactly that (Bootstrap 3's Glyphicons start at
/// ~keep U+E001), so a label already containing that literal character collided with the
/// ~keep placeholder and reappeared as a spurious hard break after "restoration". Because the
/// ~keep three marker bytes (space, backslash, `\n`) are pure ASCII, they can never occur as
/// ~keep part of a multi-byte UTF-8 sequence, so matching/splitting on them with plain byte
/// ~keep offsets (via `str::find`) is always on a char boundary -- no placeholder needed.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn normalize_link_label(label: &str) -> String {
    let mut segments: Vec<String> = Vec::new();
    let mut markers: Vec<&'static str> = Vec::new();
    let mut rest = label;

    while let Some((marker_pos, marker)) = find_earliest_hard_break_marker(rest) {
        let mut segment = String::new();
        collapse_whitespace_into(&mut segment, &rest[..marker_pos]);
        segments.push(segment);
        markers.push(marker);
        rest = &rest[marker_pos + marker.len()..];
    }
    let mut segment = String::new();
    collapse_whitespace_into(&mut segment, rest);
    segments.push(segment);

    let label = assemble_label(segments, markers);
    protect_adjacent_hard_breaks(&label).into_owned()
}

/// Starts every line of `label` after the first with `indent`, the content column of the list
/// item that holds the link, and nothing else. Both converters indent a normalized label here.
///
/// ~keep Link-label normalization collapses the content-column whitespace after a hard break to
/// ~keep one space. Restore the enclosing item's actual column so the continuation cannot become
/// ~keep a new block and split the link (issue #678). A blank line stays blank.
pub fn indent_continuation_lines(label: &mut String, indent: &str) {
    if !label.contains('\n') {
        return;
    }
    let original = std::mem::take(label);
    let mut lines = original.split('\n');
    let mut indented = lines.next().unwrap_or_default().to_string();
    for line in lines {
        indented.push('\n');
        if !line.is_empty() {
            indented.push_str(indent);
            indented.push_str(line.trim_start_matches([' ', '\t']));
        }
    }
    *label = indented;
}

/// ~keep Keep consecutive hard breaks inside a link label from creating a blank line, which
/// would end the paragraph before the link's closing delimiter is parsed.
pub fn protect_adjacent_hard_breaks(label: &str) -> Cow<'_, str> {
    if !label.contains("  \n  \n") && !label.contains("\\\n  \n") {
        return Cow::Borrowed(label);
    }
    let protected = label.replace("  \n  \n", "  \n\\\n");
    Cow::Owned(protected.replace("\\\n  \n", "\\\n\\\n"))
}

/// Re-join a label's whitespace-collapsed segments and the hard-break markers between them,
/// trimming the label's own outer whitespace without eating a break that sits at either end.
///
/// A `<br>` against the `</a>` is real content: `<a href="H">A<br></a>B` renders as A, a line
/// break, then B, and `[A  \n](H)B` re-parses to exactly that `<a href="H">A<br/></a>B`
/// (issue #497). It was previously dropped because the whole label was `str::trim`-ed, and a
/// `"  \n"` marker is indistinguishable from incidental trailing whitespace once flattened.
///
/// A run of breaks at either end still collapses to one. Two adjacent markers put a blank line
/// in the label, and a blank line ends the paragraph -- so `[  \n  \nA](H)` would destroy the
/// link rather than preserve a second break nobody can see anyway.
pub fn assemble_label(mut segments: Vec<String>, mut markers: Vec<&'static str>) -> String {
    if let Some(first) = segments.first_mut() {
        *first = first.trim_start().to_string();
    }
    if let Some(last) = segments.last_mut() {
        *last = last.trim_end().to_string();
    }

    let mut leading = "";
    while segments.len() > 1 && segments[0].is_empty() {
        segments.remove(0);
        leading = markers.remove(0);
    }
    let mut trailing = "";
    while segments.len() > 1 && segments.last().is_some_and(String::is_empty) {
        segments.pop();
        trailing = markers.pop().unwrap_or("");
    }

    // ~keep Nothing but breaks: a break needs a line on both sides to mean anything, so a
    // ~keep label of only `<br>` collapses to empty exactly as it did before.
    if segments.iter().all(String::is_empty) {
        return String::new();
    }

    let mut result = String::with_capacity(label_capacity(&segments, &markers, leading, trailing));
    result.push_str(leading);
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            result.push_str(markers.get(index - 1).copied().unwrap_or(""));
        }
        result.push_str(segment);
    }
    result.push_str(trailing);
    result
}

/// Exact byte length [`assemble_label`] is about to write.
fn label_capacity(segments: &[String], markers: &[&str], leading: &str, trailing: &str) -> usize {
    segments.iter().map(String::len).sum::<usize>()
        + markers.iter().map(|marker| marker.len()).sum::<usize>()
        + leading.len()
        + trailing.len()
}

/// Find the earliest occurrence of either hard-break marker in `text`, if any.
fn find_earliest_hard_break_marker(text: &str) -> Option<(usize, &'static str)> {
    HARD_BREAK_MARKERS
        .iter()
        .filter_map(|marker| text.find(marker).map(|pos| (pos, *marker)))
        .min_by_key(|(pos, _)| *pos)
}

/// Fold newlines to a space and collapse whitespace runs in a marker-free segment, appending
/// the result to `out`. Mirrors the whitespace handling `normalize_link_label` has always
/// applied outside of a hard-break marker. The fast converter normalizes its label with this
/// function and [`assemble_label`] too.
pub fn collapse_whitespace_into(out: &mut String, segment: &str) {
    if segment.is_empty() {
        return;
    }

    let folded: Cow<'_, str> = if segment.contains(['\n', '\r']) {
        Cow::Owned(segment.replace(['\n', '\r'], " "))
    } else {
        Cow::Borrowed(segment)
    };

    out.push_str(text::normalize_whitespace_cow(folded.as_ref()).as_ref());
}

/// Normalize a tag name to lowercase, preserving borrowed input when possible.
pub fn normalized_tag_name(raw: Cow<'_, str>) -> Cow<'_, str> {
    if raw.as_bytes().iter().any(u8::is_ascii_uppercase) {
        let mut owned = raw.into_owned();
        owned.make_ascii_lowercase();
        Cow::Owned(owned)
    } else {
        raw
    }
}

/// Check if an element is block-level (not inline).
pub fn is_block_level_element(tag_name: &str) -> bool {
    is_block_level_name(tag_name, crate::converter::main_helpers::is_inline_element(tag_name))
}

/// Returns the largest valid char boundary index at or before `index`.
///
/// If `index` is already a char boundary it is returned unchanged.
/// Otherwise it walks backwards to find one.  Returns 0 if no boundary
/// is found before `index`.
pub const fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        s.len()
    } else {
        let mut i = index;
        while i > 0 && !s.is_char_boundary(i) {
            i -= 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LinkText, ends_with_zero_width_space_in_source, link_label_edges, link_text_edges,
        without_trailing_line_end_in_source,
    };
    use crate::converter::utility::white_space::with_white_space_references_decoded;

    #[test]
    fn a_link_label_gives_up_the_white_space_at_its_edges() {
        for (content, edges) in [
            ("Go", ("", "", "Go")),
            (" Go ", (" ", " ", "Go")),
            ("\tGo\t", (" ", " ", "Go")),
            ("  Go", (" ", "", "Go")),
            ("Go\r", ("", " ", "Go")),
            (" ", (" ", "", "")),
            ("", ("", "", "")),
            ("Go  \nnow", ("", "", "Go  \nnow")),
        ] {
            assert_eq!(link_label_edges(content), edges, "{content:?}");
        }
    }

    /// One definition of white space for every label edge: a Unicode space at an edge is the
    /// one space of that edge, as at the edge of a text or of an autolink.
    #[test]
    fn a_unicode_space_at_a_link_label_edge_is_the_space_of_that_edge() {
        for (content, edges) in [
            ("\u{a0}Go\u{a0}", (" ", " ", "Go")),
            ("\u{202f}Go", (" ", "", "Go")),
            ("Go\u{202f}", ("", " ", "Go")),
            ("\u{2003}**Go**", (" ", "", "**Go**")),
            (" \u{a0}Go\u{a0} ", (" ", " ", "Go")),
            ("\u{a0}", (" ", "", "")),
        ] {
            assert_eq!(link_label_edges(content), edges, "{content:?}");
        }
    }

    /// A form feed is a character (issue #811): the white space of an edge that holds one goes
    /// outside the link as it is. The other edge is decided on its own.
    #[test]
    fn a_form_feed_at_a_link_label_edge_is_given_back_as_it_is() {
        for (content, edges) in [
            ("\u{c}Go", ("\u{c}", "", "Go")),
            ("Go\u{c}", ("", "\u{c}", "Go")),
            (" \u{c}Go ", (" \u{c}", " ", "Go")),
            ("\u{a0}Go\u{c} ", (" ", "\u{c} ", "Go")),
            ("\u{c}", ("\u{c}", "", "")),
            ("Go\u{c}now", ("", "", "Go\u{c}now")),
        ] {
            assert_eq!(link_label_edges(content), edges, "{content:?}");
        }
    }

    /// A line end at an edge is a break, not white space: that side is given back whole, with the
    /// white space around the break. The other edge still gives its space.
    #[test]
    fn a_line_break_at_a_link_label_edge_keeps_that_side_whole() {
        for (content, edges) in [
            ("Go  \n", ("", "", "Go  \n")),
            ("Go\n", ("", "", "Go\n")),
            ("Go\\\n", ("", "", "Go\\\n")),
            ("Go  \n\u{a0}", ("", "", "Go  \n\u{a0}")),
            ("  \nGo", ("", "", "  \nGo")),
            ("\\\nGo", ("", "", "\\\nGo")),
            ("   \nGo", ("", "", "   \nGo")),
            ("\u{a0}  \nGo", ("", "", "\u{a0}  \nGo")),
            ("  \nGo  \n", ("", "", "  \nGo  \n")),
            ("  \n", ("", "", "  \n")),
            (" Go  \n", (" ", "", "Go  \n")),
            (" Go  \n\u{a0}", (" ", "", "Go  \n\u{a0}")),
            ("  \nGo ", ("", " ", "  \nGo")),
            ("  \nGo\u{a0}", ("", " ", "  \nGo")),
        ] {
            assert_eq!(link_label_edges(content), edges, "{content:?}");
        }
    }

    #[test]
    fn the_text_of_a_link_gives_one_space_for_the_white_space_at_each_end() {
        for (text, edges) in [
            ("https://e.org/", ("", "")),
            (" https://e.org/", (" ", "")),
            ("https://e.org/\n", ("", " ")),
            ("\t\thttps://e.org/ \r\n", (" ", " ")),
            ("\u{a0}https://e.org/\u{a0}", (" ", " ")),
            ("\u{200b}https://e.org/", ("", "")),
            ("", ("", "")),
        ] {
            let link_text = LinkText {
                text: text.to_owned(),
                breaks: None,
            };
            assert_eq!(link_text_edges(&link_text), edges, "{text:?}");
        }
    }

    #[test]
    fn white_space_beside_a_line_break_at_an_end_of_a_link_gives_no_space() {
        for (text, breaks, edges) in [
            ("https://e.org/\u{a0}", (14, 14), ("", "")),
            ("https://e.org/ ", (15, 15), ("", "")),
            ("\u{a0}https://e.org/ ", (0, 0), ("", " ")),
            (" https://e.org/", (1, 1), ("", "")),
            (" https://e.org/ ", (5, 5), (" ", " ")),
            (" https://e.org/ ", (0, 16), ("", "")),
        ] {
            let link_text = LinkText {
                text: text.to_owned(),
                breaks: Some(breaks),
            };
            assert_eq!(link_text_edges(&link_text), edges, "{text:?} {breaks:?}");
        }
    }

    #[test]
    fn a_source_text_that_ends_with_a_line_end_loses_its_trailing_white_space() {
        for (raw, kept) in [
            ("one\n", "one"),
            ("one \t\r\n ", "one"),
            ("one&#10;", "one"),
            ("one&#xA;", "one"),
            ("one&#Xa;", "one"),
            ("one&NewLine;", "one"),
            ("one&#13;&#10;", "one"),
            ("one\n&#32;", "one"),
            ("one&#32;\n", "one"),
            ("one&#9;&Tab;&#10; &#x20;", "one"),
            ("a &amp; b&#10;", "a &amp; b"),
            ("one&amp;#10;\n", "one&amp;#10;"),
            ("one&nbsp;\n", "one&nbsp;"),
            ("one &\n", "one &"),
            ("é&#10;", "é"),
        ] {
            let text = with_white_space_references_decoded(raw);
            assert_eq!(without_trailing_line_end_in_source(&text), Some(kept), "{raw:?}");
        }
    }

    #[test]
    fn a_source_text_with_no_line_end_at_its_end_or_no_other_text_is_not_changed() {
        for raw in [
            "",
            "one",
            "one ",
            "one&#32;",
            "one&#9;&#13;",
            "one&amp;#10;",
            "one&amp;NewLine;",
            "one&NewLines;",
            "one&NEWLINE;",
            "one&#10;two",
            "one\n&nbsp;",
            "\n",
            " \n ",
            "&#10;",
            "&#32;&#10;\n",
        ] {
            let text = with_white_space_references_decoded(raw);
            assert_eq!(without_trailing_line_end_in_source(&text), None, "{raw:?}");
        }
    }

    #[test]
    fn a_source_text_ends_with_a_zero_width_space_as_a_character_or_a_reference() {
        for raw in [
            "one\u{200b}",
            "one&#8203;",
            "one&#x200B;",
            "one&#x200b;",
            "a &amp; b\u{200b}",
            "a & b&#8203;",
        ] {
            assert!(ends_with_zero_width_space_in_source(raw), "{raw:?}");
        }
        for raw in [
            "one",
            "",
            "one\u{200b}two",
            "one&amp;#8203;",
            "one&#8203;two",
            "one&#82030;",
        ] {
            assert!(!ends_with_zero_width_space_in_source(raw), "{raw:?}");
        }
    }
}
