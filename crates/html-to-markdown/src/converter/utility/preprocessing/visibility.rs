use std::borrow::Cow;

use super::markup::{find_tag_end, matches_tag_start};
use super::raw_text::{find_closing_tag_bytes_nested, is_self_closing_tag, matching_tag_ends, skip_opaque_region};

/// Sanitize malformed markdown-like URLs in HTML attributes.
///
/// Handles cases like: `//[domain.com/path](http://domain.com/path)`
/// Extracts the actual URL from parentheses.
///
/// This is an internal function used during preprocessing to extract valid URLs
/// from malformed HTML that contains markdown-like syntax.
///
/// # Arguments
/// * `url` - The URL string to sanitize
///
/// # Returns
/// * `Cow<str>` - Either the borrowed original URL or an owned sanitized version
pub fn sanitize_markdown_url(url: &str) -> Cow<'_, str> {
    let Some(mid) = url.find("](") else {
        return Cow::Borrowed(url);
    };

    if !url[..mid].contains('[') {
        return Cow::Borrowed(url);
    }

    let paren_start = mid + 2;
    let Some(rel_end) = url[paren_start..].find(')') else {
        return Cow::Borrowed(url);
    };
    let paren_end = paren_start + rel_end;
    if paren_start >= paren_end {
        return Cow::Borrowed(url);
    }

    Cow::Owned(url[paren_start..paren_end].to_string())
}

/// Strip elements that are never rendered from HTML: those carrying the `hidden`
/// attribute, and those hidden via an inline `style="display:none"`,
/// `style="visibility:hidden"` or `style="font-size:0"` declaration.
///
/// `font-size: 0` is the one conditional case — it is also a spacing hack whose children
/// restore a readable size — so its subtree is kept when a descendant re-declares a non-zero
/// `font-size` or contains an image, whose intrinsic size is independent of font size (issues #468, #476).
///
/// Scans for opening tags matching either condition, finds their matching
/// closing tag, and removes the entire element (tag + content, so nested
/// content never leaks out). Self-closing tags are also removed.
/// Find `needle` in `haystack` at or after `from`, returning its start offset.
fn find_subslice(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    memchr::memmem::find(&haystack[from..], needle).map(|off| from + off)
}

/// If `<` at `idx` (with `next`/`rest` already read) opens a real HTML comment (`<!--`) or a
/// CDATA section (`<![CDATA[`), return the index just past its terminator (or `len` if
/// unterminated). Returns `None` for anything else, in which case `idx` is unaffected.
///
/// Extracted from `strip_bogus_comments`'s main scan loop — identical terminator lookups,
/// unchanged.
fn skip_real_comment_or_cdata(bytes: &[u8], idx: usize, len: usize, next: u8, rest: &[u8]) -> Option<usize> {
    // ~keep Step over a real comment as one unit. Its interior is not markup, and a
    // ~keep downlevel conditional comment (`<!--[if gte mso 9]> … <![endif]-->`, which
    // ~keep Microsoft Word emits by the dozen) contains a `<![endif]` that looks exactly
    // ~keep like a bogus comment. Stripping that would delete the `-->` closing the real
    // ~keep comment, leaving it unterminated and swallowing the rest of the document.
    if next == b'!' && rest.starts_with(b"--") {
        return Some(find_subslice(bytes, idx + 4, b"-->").map_or(len, |end| end + 3));
    }
    // ~keep Likewise CDATA: its interior is character data, not markup.
    if next == b'!' && rest.starts_with(b"[CDATA[") {
        return Some(find_subslice(bytes, idx + 9, b"]]>").map_or(len, |end| end + 3));
    }
    None
}

/// Whether `<` at `idx` (real comments/CDATA already ruled out by `skip_real_comment_or_cdata`)
/// starts a "bogus comment" per the HTML5 tokenizer.
///
/// Extracted from `strip_bogus_comments`'s main scan loop — identical classification, unchanged.
fn is_bogus_comment_marker(bytes: &[u8], idx: usize, len: usize, next: u8, rest: &[u8]) -> bool {
    if next == b'?' {
        return true;
    }
    if next == b'!' {
        // ~keep Real comments and CDATA already returned above. `<!DOCTYPE` is a doctype,
        // ~keep handled elsewhere; anything else after `<!` is a bogus comment.
        return !(rest.len() >= 7 && rest[..7].eq_ignore_ascii_case(b"DOCTYPE"));
    }
    if next == b'/' {
        // ~keep `</` followed by a letter is a real end tag; anything else -- including
        // ~keep end-of-input -- is a bogus comment.
        return idx + 2 >= len || !bytes[idx + 2].is_ascii_alphabetic();
    }
    false
}

/// The position after the bogus comment that the `<` at `idx` starts.
fn after_bogus_comment(bytes: &[u8], idx: usize) -> usize {
    // ~keep The bogus-comment state ends at the first `>` regardless of quoting, or at
    // ~keep end-of-input if there is none -- unlike a tag, it has no attribute grammar.
    memchr::memchr(b'>', &bytes[idx + 1..]).map_or(bytes.len(), |off| idx + 1 + off + 1)
}

/// If the `<` at `idx` starts a bogus comment (see [`strip_bogus_comments`]), return the
/// position after it. The caller has read a real comment (`<!--`) before.
///
/// ~keep The first pass over raw-text elements reads a bogus comment with this function, so a
/// ~keep `<textarea>` in `<? ... >` starts nothing there. It reads `<![CDATA[` as a bogus
/// ~keep comment too: it reads no markup inside `<svg>`, and outside foreign content the
/// ~keep tokenizer ends the comment at the first `>`.
pub(super) fn bogus_comment_end(bytes: &[u8], idx: usize) -> Option<usize> {
    let next = *bytes.get(idx + 1)?;
    is_bogus_comment_marker(bytes, idx, bytes.len(), next, &bytes[idx + 2..]).then(|| after_bogus_comment(bytes, idx))
}

fn has_bogus_comment_candidate(bytes: &[u8]) -> bool {
    // ~keep Markers inside quotes, comments, or CDATA may produce false positives here;
    // ~keep the full scanner resolves those. Reject only inputs containing no removal marker.
    memchr::memchr3_iter(b'?', b'!', b'/', bytes).any(|idx| {
        if idx == 0 || bytes[idx - 1] != b'<' {
            return false;
        }
        let next = bytes[idx];
        let rest = &bytes[idx + 1..];
        if next == b'!' && (rest.starts_with(b"--") || rest.starts_with(b"[CDATA[")) {
            return false;
        }
        is_bogus_comment_marker(bytes, idx - 1, bytes.len(), next, rest)
    })
}

/// Remove HTML5 *bogus comments* so they do not leak into the output as text.
///
/// The tokenizer enters the bogus-comment state from three places, and in all of them the
/// run through the next `>` becomes a comment token, which renders as nothing:
///
/// - `<?` — a "processing instruction" is not a thing in HTML; `<?php … ?>` is a comment.
/// - `<!` not beginning `--`, `DOCTYPE`, or `[CDATA[`.
/// - `</` followed by anything that is not an ASCII letter, e.g. `</3>` or `</ >`.
///
/// Real comments already convert to nothing, so leaving these as text was inconsistent as
/// well as wrong: `<?php echo 1; ?>` emitted `?php echo 1; ?>`, and `<!bogus>` emitted a
/// stray `>`.
///
/// `<![CDATA[` is deliberately NOT treated as a bogus comment here. It only is one outside
/// foreign content; inside `<svg>` or `<math>` it is real CDATA, and this pass has no
/// element context to tell them apart. Getting that wrong would corrupt SVG, so the narrower
/// behaviour is kept (see `tools/benchmark-harness/fixtures/synthetic/cdata_in_svg.html`).
///
/// Real tags are skipped with the quote-aware [`find_tag_end`] rather than scanned through,
/// so a `<?` or `<!` inside an attribute value cannot be mistaken for a bogus comment.
pub fn strip_bogus_comments(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    // ~keep The shortest bogus comment is two bytes (`<?`, `</`, `<!` at end of input), so
    // ~keep the cheap bail must not be wider than that.
    if len < 2 || !has_bogus_comment_candidate(bytes) {
        return Cow::Borrowed(input);
    }

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;

    while let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) {
        idx += offset;
        if idx + 1 >= len {
            break;
        }

        let next = bytes[idx + 1];
        let rest = if idx + 2 <= len {
            &bytes[idx + 2..]
        } else {
            &bytes[len..]
        };

        if let Some(new_idx) = skip_real_comment_or_cdata(bytes, idx, len, next, rest) {
            idx = new_idx;
            continue;
        }

        if !is_bogus_comment_marker(bytes, idx, len, next, rest) {
            // ~keep Skip a real tag wholesale so a `<?`/`<!` sitting inside a quoted
            // ~keep attribute value is never seen as a bogus comment of its own.
            if next.is_ascii_alphabetic() {
                idx = find_tag_end(bytes, idx + 1).unwrap_or(len);
                continue;
            }
            idx += 1;
            continue;
        }

        let end = after_bogus_comment(bytes, idx);
        let out = output.get_or_insert_with(|| String::with_capacity(len));
        out.push_str(&input[last..idx]);
        last = end;
        idx = end;
    }

    match output {
        Some(mut out) => {
            out.push_str(&input[last..]);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(input),
    }
}

/// Compute the end index of the hidden element starting at `idx` whose opening tag spans
/// `idx..tag_end` — either just past the opening tag itself (self-closing) or past its matching
/// closing tag.
fn hidden_element_remove_end(
    bytes: &[u8],
    idx: usize,
    tag_end: usize,
    ends: &mut Option<ahash::AHashMap<usize, Option<usize>>>,
) -> usize {
    let name_start = idx + 1;
    let name_len = bytes[name_start..]
        .iter()
        .take_while(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
        .count();
    let name = &bytes[name_start..name_start + name_len];
    if is_self_closing_tag(&bytes[idx..tag_end], name) {
        return tag_end;
    }
    if let Some(end) = ends.get_or_insert_with(|| matching_tag_ends(bytes)).get(&idx) {
        return end.unwrap_or(tag_end);
    }
    // ~keep Preserve the legacy scan of tag-looking text inside an unrelated quoted attribute.
    find_closing_tag_bytes_nested(bytes, tag_end, name).unwrap_or(tag_end)
}

/// Where the removal of the element whose open tag spans `idx..tag_end` ends, or `None` when
/// the element must be kept.
///
/// The `hidden` attribute and a definitive `display`/`visibility` declaration remove the
/// subtree outright. `font-size: 0` must preserve images and descendants that restore a readable size.
fn hidden_element_removal_end(
    input: &str,
    bytes: &[u8],
    idx: usize,
    tag_end: usize,
    ends: &mut Option<ahash::AHashMap<usize, Option<usize>>>,
) -> Option<usize> {
    let tag_slice = &input[idx..tag_end];
    if tag_has_hidden_attribute(tag_slice) {
        return Some(hidden_element_remove_end(bytes, idx, tag_end, ends));
    }
    match hidden_style_reason(tag_slice)? {
        HiddenStyleReason::Definitive => Some(hidden_element_remove_end(bytes, idx, tag_end, ends)),
        HiddenStyleReason::FontSizeZero => {
            if matches_tag_start(bytes, idx + 1, b"img") {
                return None;
            }
            let remove_end = hidden_element_remove_end(bytes, idx, tag_end, ends);
            // ~keep Scan past the element's own open tag so its `font-size: 0` is not re-read.
            let subtree = input.get(tag_end..remove_end).unwrap_or("");
            (!region_restores_visible_content(subtree)).then_some(remove_end)
        }
    }
}

/// What [`find_tag_end`] returns from every position of a page, computed in one pass.
///
/// `find_tag_end` walks from a start over quoted values to the next `>`. Its result depends only
/// on the first quote or `>` at or after the start, so one answer for each of those bytes is the
/// answer for every start. The answers are filled from the end of the page: a `>` ends the tag
/// after itself, and a quote has the answer of the byte after its partner.
struct TagEnds {
    positions: Vec<usize>,
    ends: Vec<Option<usize>>,
}

impl TagEnds {
    fn new(bytes: &[u8]) -> Self {
        let positions: Vec<usize> = memchr::memchr3_iter(b'"', b'\'', b'>', bytes).collect();
        let mut ends = vec![None; positions.len()];
        let (mut next_double, mut next_single) = (None, None);
        for index in (0..positions.len()).rev() {
            let position = positions[index];
            let next_same: &mut Option<usize> = match bytes[position] {
                b'>' => {
                    ends[index] = Some(position + 1);
                    continue;
                }
                b'"' => &mut next_double,
                _ => &mut next_single,
            };
            let end_after_partner = (*next_same).and_then(|partner| ends.get(partner + 1).copied().flatten());
            ends[index] = end_after_partner;
            *next_same = Some(index);
        }
        Self { positions, ends }
    }

    /// The result of `find_tag_end(bytes, from)`.
    fn tag_end(&self, from: usize) -> Option<usize> {
        let index = self.positions.partition_point(|&position| position < from);
        self.ends.get(index).copied().flatten()
    }
}

pub fn strip_hidden_elements(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len == 0 || !bytes.contains(&b'<') {
        return Cow::Borrowed(input);
    }

    // ~keep Every removal requires a `hidden` or `style` attribute; without either
    // ~keep initial ASCII letter, even malformed or quoted tag-like text cannot match.
    if memchr::memchr2(b'h', b'H', bytes).is_none() && memchr::memchr2(b's', b'S', bytes).is_none() {
        return Cow::Borrowed(input);
    }

    // ~keep A tag start with no end makes `find_tag_end` read to the end of the page: no `>` is
    // ~keep left, or a quote before it has no partner. A page with many such starts made this
    // ~keep loop quadratic (#765). After the first scan that fails, the ends come from a table
    // ~keep that reads the page once.
    let mut tag_ends: Option<TagEnds> = None;

    let mut idx = 0;
    let mut last = 0;
    let mut output: Option<String> = None;
    let mut ends = None;

    while idx < len {
        let Some(offset) = memchr::memchr(b'<', &bytes[idx..]) else {
            break;
        };
        idx += offset;
        // ~keep A `<` not immediately followed by an ASCII letter can never start a real
        // ~keep HTML tag name (HTML5 tokenizer "tag open state"), so it is never worth a
        // ~keep `find_tag_end` scan. Without this, a run like `<<<<<` treats every `<` as a
        // ~keep candidate tag start, and each failing scan re-walks the same suffix.
        let starts_tag_name = idx + 1 < len && bytes[idx + 1].is_ascii_alphabetic();
        if starts_tag_name {
            let tag_end = if let Some(table) = &tag_ends {
                table.tag_end(idx + 1)
            } else {
                let tag_end = find_tag_end(bytes, idx + 1);
                if tag_end.is_none() {
                    tag_ends = Some(TagEnds::new(bytes));
                }
                tag_end
            };
            if let Some(tag_end) = tag_end {
                if let Some(remove_end) = hidden_element_removal_end(input, bytes, idx, tag_end, &mut ends) {
                    let out = output.get_or_insert_with(|| String::with_capacity(len));
                    out.push_str(&input[last..idx]);
                    last = remove_end;
                    idx = remove_end;
                    continue;
                }
                // ~keep The tag is read once, as a whole. A `<` inside it is part of an attribute,
                // ~keep not a tag start, and a scan from each one read the same tag again, so a
                // ~keep page with many tag starts before one `>` took quadratic time (#765).
                idx = tag_end;
                continue;
            }
        }
        idx += 1;
    }

    if let Some(mut out) = output {
        if last < len {
            out.push_str(&input[last..]);
        }
        Cow::Owned(out)
    } else {
        Cow::Borrowed(input)
    }
}

/// Consume an optional `=value` following an attribute name, starting at `i` (already past the
/// name and any whitespace). Returns the index just past the value (quoted or bare), or `i`
/// unchanged if there is no `=`.
///
/// Extracted from `tag_has_hidden_attribute` — identical quote-aware value scan, unchanged.
fn skip_attribute_value(bytes: &[u8], mut i: usize, len: usize) -> usize {
    if i >= len || bytes[i] != b'=' {
        return i;
    }
    i += 1;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < len && (bytes[i] == b'"' || bytes[i] == b'\'') {
        let quote = bytes[i];
        i += 1;
        while i < len && bytes[i] != quote {
            i += 1;
        }
        i += 1;
    } else {
        while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
            i += 1;
        }
    }
    i
}

/// Check if an opening tag string contains the `hidden` attribute.
///
/// Handles: `hidden`, `hidden=""`, `hidden="hidden"`, `hidden="true"`.
/// Does NOT match attributes like `data-hidden` or `aria-hidden`.
///
/// ~keep `pub(crate)`: also called from `tier1::scanner` so the Tier-1 byte
/// ~keep scanner bails on the same hidden-attribute condition this pass strips,
/// ~keep rather than re-implementing the scan a second time.
pub fn tag_has_hidden_attribute(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        i += 1;
    }

    // ~keep Walk name=value pairs rather than scanning the raw tag text for the word
    // ~keep `hidden`: a plain substring scan also matches inside quoted VALUES, so
    // ~keep `<div title="... hidden from search engines">` read as hidden and the whole
    // ~keep visible element was stripped.
    while i < len {
        while i < len && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        if i >= len || bytes[i] == b'>' {
            return false;
        }

        let name_start = i;
        while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' && bytes[i] != b'>' && bytes[i] != b'/' {
            i += 1;
        }
        let name = &bytes[name_start..i];

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        i = skip_attribute_value(bytes, i, len);

        if name.eq_ignore_ascii_case(b"hidden") {
            return true;
        }
    }
    false
}

/// Remove `/* ... */` comments from a CSS declaration.
///
/// ~keep A comment before the property name shifts the first `:` so that
/// ~keep `split_once(':')` yields `"/* note */ display"` as the property, which never
/// ~keep matches — silently defeating the hidden-element check and leaking the content.
fn strip_css_comments(declaration: &str) -> Cow<'_, str> {
    if !declaration.contains("/*") {
        return Cow::Borrowed(declaration);
    }
    let mut out = String::with_capacity(declaration.len());
    let mut rest = declaration;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start + 2..].find("*/") else {
            rest = "";
            break;
        };
        rest = &rest[start + 2 + end + 2..];
    }
    out.push_str(rest);
    Cow::Owned(out)
}

/// Why an element's inline `style` marks it as never rendered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HiddenStyleReason {
    /// `display: none` or `visibility: hidden` — neither the element nor its subtree renders,
    /// whatever a descendant declares.
    Definitive,
    /// `font-size: 0` — the element's own text is invisible, but the same declaration is also
    /// the classic inline-block/email spacing hack, where a descendant sets its own non-zero
    /// `font-size` and *does* render. Callers must check the subtree before removing it.
    FontSizeZero,
}

/// Last-declaration-wins scan of an inline `style` value for the three properties that decide
/// whether an element renders.
///
/// Returns `(display_hides, visibility_hides, font_size_zero)`, where the third element is
/// `None` when `font-size` is not declared at all, and `Some(is_zero)` when it is.
fn scan_visibility_declarations(style_value: &str) -> (bool, bool, Option<bool>) {
    // ~keep CSS cascade: within one declaration block the LAST declaration for a property
    // ~keep wins, so `display:none; display:block` is VISIBLE. Matching with `.any()`
    // ~keep stripped it.
    let mut display_hides = false;
    let mut visibility_hides = false;
    let mut font_size_zero = None;
    for declaration in style_value.split(';') {
        let cleaned = strip_css_comments(declaration);
        let Some((property, value)) = cleaned.split_once(':') else {
            continue;
        };
        let property = property.trim();
        // ~keep `!important` (any casing, with or without a preceding space) is a CSS
        // ~keep priority flag, not part of the value — drop everything from `!` onward.
        let value = value.split('!').next().unwrap_or("").trim();
        if property.eq_ignore_ascii_case("display") {
            display_hides = value.eq_ignore_ascii_case("none");
        } else if property.eq_ignore_ascii_case("visibility") {
            visibility_hides = value.eq_ignore_ascii_case("hidden");
        } else if property.eq_ignore_ascii_case("font-size") {
            font_size_zero = Some(css_length_is_zero(value));
        }
    }
    (display_hides, visibility_hides, font_size_zero)
}

/// Whether a CSS length value is an exact zero: `0`, `0px`, `0.0em`, `.0%` and friends, in any
/// casing. A unit is optional (CSS permits a bare `0`) but must be a real length or percentage
/// unit when present, so a function call or keyword never reads as zero.
fn css_length_is_zero(value: &str) -> bool {
    const ZERO_UNITS: [&str; 16] = [
        "px", "pt", "pc", "em", "rem", "ex", "ch", "vw", "vh", "vmin", "vmax", "cm", "mm", "in", "q", "%",
    ];
    let digits_end = value
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(digits_end);
    if number.is_empty() || number == "." || number.matches('.').count() > 1 {
        return false;
    }
    if number.bytes().any(|b| b != b'0' && b != b'.') {
        return false;
    }
    let unit = unit.trim();
    unit.is_empty() || ZERO_UNITS.iter().any(|candidate| unit.eq_ignore_ascii_case(candidate))
}

/// Classify how an opening tag's inline `style` attribute hides the element, if at all.
///
/// This is a targeted declaration scan, not a full CSS parser: it extracts the raw `style`
/// attribute value, splits it on `;`, and inspects each `property: value` pair. Untrusted
/// input is handled defensively — extra whitespace around `:` and `;`, mixed casing, and a
/// trailing `!important` (with or without a preceding space) are all tolerated.
pub fn hidden_style_reason(tag: &str) -> Option<HiddenStyleReason> {
    style_value_hidden_reason(extract_attribute_value(tag, "style")?)
}

/// As [`hidden_style_reason`], for the value of a `style` attribute.
///
/// ~keep An inline `<svg>` reads its `display` and `visibility` attributes through this function,
/// ~keep so text in a graphic and text outside one are hidden by the same decision.
pub fn style_value_hidden_reason(style_value: &str) -> Option<HiddenStyleReason> {
    let (display_hides, visibility_hides, font_size_zero) = scan_visibility_declarations(style_value);
    if display_hides || visibility_hides {
        return Some(HiddenStyleReason::Definitive);
    }
    if font_size_zero == Some(true) {
        return Some(HiddenStyleReason::FontSizeZero);
    }
    None
}

/// Whether an opening tag's inline `style` declares a non-zero `font-size`, which makes the
/// element render even inside a `font-size: 0` ancestor.
fn tag_sets_non_zero_font_size(tag: &str) -> bool {
    let Some(style_value) = extract_attribute_value(tag, "style") else {
        return false;
    };
    scan_visibility_declarations(style_value).2 == Some(false)
}

/// Whether `region` contains an image or re-declares a non-zero `font-size`.
///
/// `font-size: 0` on a wrapper is a well-known inline-block/email spacing hack: the wrapper
/// kills the whitespace between children while each child restores a readable size. Removing
/// such a subtree would delete genuinely visible text, so the removal is skipped when a
/// descendant opts back in. This is a one-level-of-inheritance heuristic, not a cascade — a
/// size restored from a stylesheet is out of reach of a byte-level pass.
fn region_restores_visible_content(region: &str) -> bool {
    let bytes = region.as_bytes();
    let len = bytes.len();
    let mut idx = 0;
    let mut ends = None;
    while idx < len {
        if let Some(end) = skip_opaque_region(bytes, idx) {
            idx = end;
            continue;
        }
        if bytes[idx] == b'<' && idx + 1 < len && bytes[idx + 1].is_ascii_alphabetic() {
            if let Some(tag_end) = find_tag_end(bytes, idx + 1) {
                let tag = &region[idx..tag_end];
                if tag_has_hidden_attribute(tag) || hidden_style_reason(tag) == Some(HiddenStyleReason::Definitive) {
                    idx = hidden_element_remove_end(bytes, idx, tag_end, &mut ends);
                    continue;
                }
                if tag_sets_non_zero_font_size(tag) || matches_tag_start(bytes, idx + 1, b"img") {
                    return true;
                }
                idx = tag_end;
                continue;
            }
        }
        idx += 1;
    }
    false
}

/// Check if an opening tag's inline `style` attribute hides the element.
///
/// ~keep `pub`: also called from `tier1::scanner` (see `tag_has_hidden_attribute` above).
/// ~keep Tier-1 bails on any hit, including the `font-size: 0` case whose subtree Tier-2 may
/// ~keep still keep — a conservative bail that falls through to Tier-2 rather than diverging.
pub fn tag_has_hidden_style(tag: &str) -> bool {
    hidden_style_reason(tag).is_some()
}

/// If `i` (already past an attribute name and whitespace) points at `=`, scan its value with
/// `scan_attribute_value` and return `(Some(value), next_i)`; otherwise return `(None, i)`
/// unchanged.
///
/// Extracted from `extract_attribute_value` — identical `=value` handling, unchanged.
fn scan_optional_attribute_value<'a>(bytes: &[u8], tag: &'a str, mut i: usize, len: usize) -> (Option<&'a str>, usize) {
    if i >= len || bytes[i] != b'=' {
        return (None, i);
    }
    i += 1;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let (val, next) = scan_attribute_value(bytes, tag, i);
    (Some(val), next)
}

/// Extract the value of a named attribute from a raw opening-tag string.
///
/// Walks name=value pairs left to right (skipping the leading tag name) and
/// returns the first case-insensitive match. Handles double- and single-quoted
/// values and bare (unquoted) values. Returns `None` when the attribute is
/// absent or is a boolean attribute with no `=value`.
fn extract_attribute_value<'a>(tag: &'a str, attr_name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
        i += 1;
    }

    while i < len {
        while i < len && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        if i >= len || bytes[i] == b'>' {
            break;
        }

        let name_start = i;
        while i < len && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/' {
            i += 1;
        }
        let name = &tag[name_start..i];

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        let (value, next_i) = scan_optional_attribute_value(bytes, tag, i, len);
        i = next_i;

        if name.eq_ignore_ascii_case(attr_name) {
            return value;
        }
    }
    None
}

/// Scan a single attribute value starting at `start`, handling quoted and bare forms.
///
/// Returns the extracted value slice and the byte index immediately following it.
fn scan_attribute_value<'a>(bytes: &[u8], tag: &'a str, start: usize) -> (&'a str, usize) {
    let len = bytes.len();
    if start < len && (bytes[start] == b'"' || bytes[start] == b'\'') {
        let quote = bytes[start];
        let val_start = start + 1;
        let mut end = val_start;
        while end < len && bytes[end] != quote {
            end += 1;
        }
        let value = &tag[val_start..end];
        let next = if end < len { end + 1 } else { end };
        return (value, next);
    }

    let val_start = start;
    let mut end = start;
    while end < len && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
        end += 1;
    }
    (&tag[val_start..end], end)
}

#[cfg(test)]
mod tag_ends_tests {
    use super::{TagEnds, find_tag_end};

    #[test]
    fn should_give_the_tag_end_that_a_scan_from_each_byte_gives() {
        // ~keep Every page of up to eight bytes over the bytes that the scan tells apart.
        let alphabet = [b'"', b'\'', b'>', b'a'];
        let mut compared = 0;
        for len in 0..=8u32 {
            for mut code in 0..4usize.pow(len) {
                let page: Vec<u8> = (0..len)
                    .map(|_| {
                        let byte = alphabet[code % 4];
                        code /= 4;
                        byte
                    })
                    .collect();
                let table = TagEnds::new(&page);
                for from in 0..=page.len() + 1 {
                    assert_eq!(table.tag_end(from), find_tag_end(&page, from), "{page:?} from {from}");
                    compared += 1;
                }
            }
        }
        assert!(compared > 500_000, "compared only {compared} scans");
    }
}
