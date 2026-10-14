//! Text-escaping helpers for Markdown link/label rendering.
//!
//! Split out of `content.rs` to stay under the 1000-line quality gate; these
//! functions evolve independently of the extraction helpers that remain there.

use std::borrow::Cow;

mod block;

#[cfg(test)]
use block::block_opener_offset;
use block::escape_block_openers_on_continuation_lines;
pub use block::{
    atx_closing_sequence_offset, code_fence, ends_with_hard_break, escape_block_start, escape_continuation_line_start,
    escape_djot_continuation_line_start, escape_djot_list_item_start, escape_paragraph_start, first_ordered_marker_len,
    is_block_level_name, is_heading_underline, is_rule, leading_indent, line_opens_block, opens_block,
};

/// Escape a link label or image alt text so it cannot break out of the `[...]` / `![...]`
/// it is about to be wrapped in.
///
/// Two independent escapes, in order:
///
/// 1. [`escape_label_brackets`] -- a `]` with no local opener, or a matched pair that is
///    itself link- or reference-link-shaped.
/// 2. [`escape_block_openers_on_continuation_lines`] -- a `CommonMark` block-structure
///    marker opening a line after the first (issue #496).
///
/// Both are unconditional, unlike the `escape_misc`/`escape_asterisks` family: those decide
/// whether text that merely *looks* like Markdown is emitted verbatim, whereas these two
/// decide whether the link or image survives at all.
pub fn escape_link_label(text: &str) -> Cow<'_, str> {
    match escape_label_brackets(text) {
        Cow::Borrowed(bracketed) => escape_block_openers_on_continuation_lines(bracketed),
        Cow::Owned(bracketed) => Cow::Owned(escape_block_openers_on_continuation_lines(&bracketed).into_owned()),
    }
}

/// Escape raw image-alt text before placing it inside a Markdown image label. ~keep
pub fn escape_image_alt(text: &str) -> Cow<'_, str> {
    if !contains_blank_line(text) {
        return match crate::text::escape(text, false, false, false, false) {
            Cow::Borrowed(escaped) => escape_link_label(escaped),
            Cow::Owned(escaped) => Cow::Owned(escape_link_label(&escaped).into_owned()),
        };
    }

    let mut normalized = String::with_capacity(text.len() + 8);
    let mut lines = text.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        if lines.peek().is_some_and(|next| is_blank_line(next)) {
            normalized.push_str(line.strip_suffix('\n').unwrap_or(line));
            normalized.push_str("&#10;");
        } else {
            normalized.push_str(line);
        }
    }

    let escaped = crate::text::escape(&normalized, false, false, false, false);
    Cow::Owned(escape_link_label(&escaped).into_owned())
}

fn contains_blank_line(text: &str) -> bool {
    let mut lines = text.split_inclusive('\n').peekable();
    while lines.next().is_some() {
        if lines.peek().is_some_and(|line| is_blank_line(line)) {
            return true;
        }
    }
    false
}

fn is_blank_line(line: &str) -> bool {
    line.strip_suffix('\n')
        .is_some_and(|content| content.bytes().all(|byte| matches!(byte, b' ' | b'\t')))
}

/// Escape the brackets in a link label or image alt text that would otherwise terminate it.
///
/// One of the two halves of [`escape_link_label`]; see there for the other.
/// Tracks matched bracket pairs and escapes a closing bracket that has no local opener
/// (it would otherwise close the caller's own wrapping `[`/`![` early), an opening
/// bracket that is never matched by a later closer (it would otherwise open a
/// link/image span that swallows the caller's own closing `]`/`)` on reparse), and
/// escapes a matched pair outright when it is itself link- or reference-link-shaped.
///
/// # Examples
/// ```text
/// Input:  "]"
/// Output: "\\]"
///
/// Input:  "[A;B)"
/// Output: "\\[A;B)"
///
/// Input:  "[outer [inner]]"
/// Output: "[outer [inner]]"
///
/// Input:  "[foo](uri2)"
/// Output: "\\[foo\\](uri2)"
/// ```
///
/// Scan `text` for `[`/`]` bytes that must be escaped and return a per-byte marker vector
/// (`true` at a byte offset that needs a preceding `\`). Split out of
/// [`escape_label_brackets`] to keep that function under the line-count quality gate.
fn mark_bracket_escapes(text: &str) -> Vec<bool> {
    let mut escape_at = vec![false; text.len()];
    // ~keep Byte offsets (into `text`) of unescaped `[` openers not yet matched by a `]`,
    // ~keep innermost last. A bare depth counter cannot tell "is there a local opener"
    // ~keep from "which one", and the `](`/`][` check below needs the specific opener.
    let mut open_positions: Vec<usize> = Vec::new();
    let mut backslash_count = 0usize;
    let mut chars = text.char_indices().peekable();

    while let Some((byte_pos, ch)) = chars.next() {
        if ch == '\\' {
            backslash_count += 1;
            continue;
        }

        let is_escaped = backslash_count % 2 == 1;
        backslash_count = 0;

        match ch {
            '[' if !is_escaped => open_positions.push(byte_pos),
            ']' if !is_escaped => match open_positions.pop() {
                None => {
                    // ~keep No local opener: left alone, this `]` closes the caller's own
                    // ~keep wrapping `[`/`![` early, truncating the label.
                    escape_at[byte_pos] = true;
                }
                Some(open_pos) => {
                    // ~keep A `]` immediately followed by `(` or `[` completes an inline
                    // ~keep link/image (`](dest)`) or a reference link (`][ref]`) on
                    // ~keep reparse: `CommonMark` parses link/image label content as full
                    // ~keep inline markdown, so a literal `[foo](uri2)` inside it silently
                    // ~keep becomes a real nested link and the destination is lost --
                    // ~keep escaping only this closer is not enough and is actively worse:
                    // ~keep the still-open outer `[`/`![` is then free to be captured by a
                    // ~keep *later* unescaped `]` instead (verified against comrak), so the
                    // ~keep image/link fails to form at all. Escaping both ends of the
                    // ~keep matched pair together is the only shape that round-trips.
                    //
                    // ~keep Exempted when the opener is itself preceded by `!`: that is a
                    // ~keep real, intentional `![alt](src)` this converter already emitted
                    // ~keep while walking a nested `<img>` inside link text (CommonMark
                    // ~keep permits images, just not links, inside link text), not literal
                    // ~keep text that merely looks link-shaped -- escaping it would corrupt
                    // ~keep the nested image instead of protecting the label (caught by
                    // ~keep `test_commonmark_compliance`'s `[![moon](moon.jpg)](/uri)`).
                    let is_link_shaped = matches!(chars.peek(), Some((_, '(' | '[')));
                    let is_nested_image = open_pos > 0 && text.as_bytes()[open_pos - 1] == b'!';
                    if is_link_shaped && !is_nested_image {
                        escape_at[open_pos] = true;
                        escape_at[byte_pos] = true;
                    }
                }
            },
            _ => {}
        }
    }

    // ~keep Whatever is left in `open_positions` once the scan ends is a `[` with no
    // ~keep matching `]` anywhere in the label. Left alone, it starts a link/image span
    // ~keep that CommonMark's inline parser will happily extend into the *caller's own*
    // ~keep closing `]`/`)` on reparse -- the exact failure in issue #499, where
    // ~keep `![[A;B)](S)` reparses as dangling `!` text followed by a real
    // ~keep `[A;B)](S)` link, silently dropping the image. Escaping every such opener
    // ~keep is the mirror of the unmatched-`]` case above.
    for open_pos in open_positions {
        escape_at[open_pos] = true;
    }

    escape_at
}

/// Returns `Cow::Borrowed` when `text` contains neither `[` nor `]` (escaping is then
/// necessarily a no-op), or `Cow::Owned` with the escaped text otherwise.
fn escape_label_brackets(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed("");
    }

    // ~keep Escapes are only ever inserted at `[`/`]` byte positions (see the match below),
    // ~keep so when neither appears the two-pass logic further down is guaranteed to be a
    // ~keep no-op -- skip both its allocations (the `escape_at` vec and the output String).
    if memchr::memchr2(b'[', b']', text.as_bytes()).is_none() {
        return Cow::Borrowed(text);
    }

    // ~keep Two linear passes rather than one pass that mutates the output string as it
    // ~keep goes: escaping a matched bracket pair needs to touch the *opener*, which by
    // ~keep the time its `]` is found has already been written. Retroactively
    // ~keep `String::insert`-ing it back in is O(remaining length) per escape, so a label
    // ~keep built of many link-shaped pairs (`[a](b)[a](b)...`) would make the whole
    // ~keep function O(n^2) -- the same denial-of-service shape `bare_lt_complexity.rs`
    // ~keep exists to catch elsewhere in this crate. Precomputing which byte offsets need
    // ~keep an escape first keeps the second pass a single linear append.
    let escape_at = mark_bracket_escapes(text);

    let mut result = String::with_capacity(text.len() + 2);
    for (byte_pos, ch) in text.char_indices() {
        if escape_at[byte_pos] {
            result.push('\\');
        }
        result.push(ch);
    }

    Cow::Owned(result)
}

/// Escape every `|` in a Markdown table cell that no backslash escapes yet.
///
/// GFM splits a row on each unescaped `|` before it reads any inline syntax, so a pipe in a
/// code span, a link destination or title, an image description or a flattened nested table
/// ends the cell like a pipe in plain text does, and the row no longer matches the delimiter
/// row. `\|` is a literal pipe everywhere in a cell, code spans included. A pipe after an odd
/// run of backslashes is escaped already and stays as it is.
pub fn escape_cell_pipes(text: &str) -> Cow<'_, str> {
    if !text.contains('|') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 4);
    let mut backslashes = 0usize;
    for c in text.chars() {
        if c == '|' && backslashes.is_multiple_of(2) {
            out.push('\\');
        }
        backslashes = if c == '\\' { backslashes + 1 } else { 0 };
        out.push(c);
    }
    Cow::Owned(out)
}

/// Escape every unescaped backtick and dash in literal text emitted into a Djot table cell. ~keep
///
/// Djot treats even an unmatched backtick as the start of a verbatim span, which then
/// consumes the rest of the row and its delimiter row, and parses dash runs as en/em dashes.
/// Callers pass only fragments in which these characters came from literal text, before
/// combining them with generated markup delimiters, so generated syntax remains unchanged.
/// A character after an odd run of backslashes is already escaped.
pub fn escape_djot_table_cell_literal(
    text: &str,
    output_format: crate::options::OutputFormat,
    in_table_cell: bool,
) -> Cow<'_, str> {
    if output_format != crate::options::OutputFormat::Djot
        || !in_table_cell
        || (!text.contains('`') && !text.contains('-'))
    {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 4);
    let mut backslashes = 0usize;
    for c in text.chars() {
        if matches!(c, '`' | '-') && backslashes.is_multiple_of(2) {
            out.push('\\');
        }
        backslashes = if c == '\\' { backslashes + 1 } else { 0 };
        out.push(c);
    }
    Cow::Owned(out)
}

/// Escape any bare pipe left in a nested table's rendered markdown: one that is neither
/// already backslash-escaped nor inside a matched backtick code span (a `CommonMark`-
/// compliant reparse does not treat either as a cell delimiter, so this must not touch
/// them either).
///
/// Scoped to a nested `<table>`'s own rendered text (see the call site in
/// [`render_cell_text`]). Markdown output then escapes the whole cell with
/// [`escape_cell_pipes`], because GFM also splits a row on a pipe in a code span.
///
/// Walks backtick runs the same way a spec-compliant parser does: a run of N backticks
/// opens a code span only if a later run of exactly N backticks closes it; otherwise the
/// backticks are literal text and any pipes among them still need escaping.
pub fn escape_bare_pipes_outside_code_spans(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 4);
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            out.push(c);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '`' {
            let run_start = i;
            while i < chars.len() && chars[i] == '`' {
                i += 1;
            }
            let run_len = i - run_start;
            if let Some(close_start) = find_matching_backtick_run(&chars, i, run_len) {
                out.extend(&chars[run_start..close_start + run_len]);
                i = close_start + run_len;
            } else {
                out.extend(&chars[run_start..i]);
            }
            continue;
        }
        if c == '|' {
            out.push('\\');
            out.push('|');
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

// ~keep Lives here, shared, rather than beside either caller. Both tiers must escape a
// ~keep flattened nested table identically or the output forks: unescaped, these pipes are
// ~keep read as the OUTER row's cell delimiters on reparse, and GFM truncates the row to the
// ~keep header's column count, dropping the inner cells outright. That is content loss, and a
// ~keep silently drifting second copy would reintroduce it on whichever tier fell behind.

/// Find the start index of the next backtick run of exactly `run_len` backticks at or
/// after `start`, treating a longer or shorter run as not matching (mirroring `CommonMark`
/// code span matching, which requires an exact backtick-count match).
pub fn find_matching_backtick_run(chars: &[char], start: usize, run_len: usize) -> Option<usize> {
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '`' {
            let candidate_start = i;
            while i < chars.len() && chars[i] == '`' {
                i += 1;
            }
            if i - candidate_start == run_len {
                return Some(candidate_start);
            }
        } else {
            i += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::content::{chomp_inline, merge_adjacent_emphasis, normalize_link_label};
    use super::*;

    #[test]
    fn escape_cell_pipes_escapes_each_pipe_no_backslash_escapes() {
        assert!(matches!(escape_cell_pipes("a b"), Cow::Borrowed("a b")));
        assert_eq!(escape_cell_pipes("a|b"), r"a\|b");
        assert_eq!(escape_cell_pipes(r"a\|b"), r"a\|b");
        assert_eq!(escape_cell_pipes(r"a\\|b"), r"a\\\|b");
        assert_eq!(escape_cell_pipes("`a|b` [t](u|v)"), r"`a\|b` [t](u\|v)");
        assert_eq!(escape_cell_pipes("||"), r"\|\|");
    }

    #[test]
    fn escape_djot_table_cell_literal_escapes_only_unescaped_backticks_in_scope() {
        use crate::options::OutputFormat;

        assert!(matches!(
            escape_djot_table_cell_literal("a`b", OutputFormat::Markdown, true),
            Cow::Borrowed("a`b")
        ));
        assert!(matches!(
            escape_djot_table_cell_literal("a`b", OutputFormat::Djot, false),
            Cow::Borrowed("a`b")
        ));
        assert_eq!(escape_djot_table_cell_literal("a`b", OutputFormat::Djot, true), r"a\`b");
        assert_eq!(
            escape_djot_table_cell_literal(r"a\`b", OutputFormat::Djot, true),
            r"a\`b"
        );
        assert_eq!(
            escape_djot_table_cell_literal(r"a\\`b", OutputFormat::Djot, true),
            r"a\\\`b"
        );
        assert_eq!(
            escape_djot_table_cell_literal("``a``", OutputFormat::Djot, true),
            r"\`\`a\`\`"
        );
        assert_eq!(
            escape_djot_table_cell_literal("a---b", OutputFormat::Djot, true),
            r"a\-\-\-b"
        );
        assert_eq!(
            escape_djot_table_cell_literal(r"a\-b", OutputFormat::Djot, true),
            r"a\-b"
        );
    }

    #[test]
    fn escape_link_label_leaves_plain_text_unchanged() {
        assert_eq!(escape_link_label("plain text"), "plain text");
    }

    #[test]
    fn escape_link_label_escapes_an_unmatched_closing_bracket() {
        assert_eq!(escape_link_label("]"), "\\]");
    }

    // ~keep A `[...]` pair with a local opener and no trailing `(`/`[` is left unescaped
    // ~keep even though it superficially resembles the unmatched-bracket case above: the
    // ~keep opener/closer here match each other, so unlike a bare `]` they do not close
    // ~keep the caller's own wrapping bracket early.
    #[test]
    fn escape_link_label_leaves_a_matched_non_link_shaped_pair_unchanged() {
        assert_eq!(escape_link_label("[link]"), "[link]");
    }

    #[test]
    fn escape_link_label_leaves_balanced_nested_brackets_unchanged() {
        assert_eq!(escape_link_label("[outer [inner]]"), "[outer [inner]]");
    }

    // ~keep Regression for Cluster B (image alt text losing its nested destination):
    // ~keep `<img alt="[foo](uri2)">` must not let the alt text's own `[foo](uri2)` be
    // ~keep reparsed as a real nested link, or `uri2` is silently dropped on a second
    // ~keep conversion pass (CommonMark parses an image's alt as full inline content).
    #[test]
    fn escape_link_label_escapes_a_link_shaped_bracket_pair() {
        assert_eq!(escape_link_label("[foo](uri2)"), "\\[foo\\](uri2)");
    }

    // ~keep Same hazard, reference-link form: `[foo][ref]` is just as reparsable as
    // ~keep `[foo](uri2)`.
    #[test]
    fn escape_link_label_escapes_a_reference_link_shaped_bracket_pair() {
        assert_eq!(escape_link_label("[foo][ref]"), "\\[foo\\][ref]");
    }

    // ~keep A `[...]` NOT immediately followed by `(` or `[` cannot complete a link on
    // ~keep reparse, so it is left alone -- this is the exact shape the task's proposed
    // ~keep narrower rule ("escape `]` only before `(`") would also leave alone, but this
    // ~keep also confirms plain non-link-shaped bracket text is unaffected.
    #[test]
    fn escape_link_label_leaves_non_link_shaped_brackets_unchanged() {
        assert_eq!(escape_link_label("see [note] here"), "see [note] here");
    }

    // ~keep Regression: a link whose *label* is a real, intentionally-emitted nested
    // ~keep image (`![moon](moon.jpg)`, CommonMark permits images -- just not links --
    // ~keep inside link text) must not be escaped merely because it is link-shaped: it
    // ~keep is not literal text, it is markdown this converter already produced while
    // ~keep walking a nested `<img>`. Caught by `commonmark_compliance_test`'s example
    // ~keep 517 (`[![moon](moon.jpg)](/uri)`) before this exemption was added.
    #[test]
    fn escape_link_label_does_not_escape_a_nested_image() {
        assert_eq!(escape_link_label("![moon](moon.jpg)"), "![moon](moon.jpg)");
    }

    // ~keep A link-shaped bracket pair still nested inside an outer, non-link-shaped
    // ~keep bracket pair: only the inner (dangerous) pair is escaped, and the escape is
    // ~keep inserted at the correct byte offset for the *matching* opener, not just
    // ~keep prepended to the whole string.
    #[test]
    fn escape_link_label_escapes_only_the_link_shaped_inner_pair() {
        assert_eq!(escape_link_label("[a[b](c)]"), "[a\\[b\\](c)]");
    }

    // ~keep Regression for issue #499: an unmatched `[` is the mirror hazard of the
    // ~keep unmatched-`]` case above -- left alone, `![[A;B)](S)` reparses as dangling
    // ~keep `!` text followed by a real `[A;B)](S)` link, silently dropping the image.
    #[test]
    fn escape_link_label_escapes_an_unmatched_opening_bracket() {
        assert_eq!(escape_link_label("[A;B)"), "\\[A;B)");
    }

    #[test]
    fn escape_link_label_escapes_an_unmatched_opening_bracket_before_plain_text() {
        assert_eq!(escape_link_label("[a"), "\\[a");
    }

    // ~keep The outer `[` is unmatched and must be escaped; the inner `[a]` is a
    // ~keep matched, non-link-shaped pair and stays untouched -- the two rules apply
    // ~keep independently at their own byte offsets.
    #[test]
    fn escape_link_label_escapes_an_unmatched_outer_bracket_around_a_matched_inner_pair() {
        assert_eq!(escape_link_label("[[a]"), "\\[[a]");
    }

    #[test]
    fn escape_link_label_escapes_an_unmatched_opening_bracket_after_plain_text() {
        assert_eq!(escape_link_label("a[b"), "a\\[b");
    }

    #[test]
    fn escape_link_label_leaves_an_already_escaped_opening_bracket_unchanged() {
        assert_eq!(escape_link_label("\\[a"), "\\[a");
    }

    #[test]
    fn escape_image_alt_encodes_blank_line_as_html_line_feed() {
        assert_eq!(escape_image_alt("A\n\n B C"), "A&#10;\n B C");
    }

    #[test]
    fn escape_image_alt_encodes_each_blank_line_without_losing_line_feeds() {
        assert_eq!(escape_image_alt("A\n\n\nB"), "A&#10;&#10;\nB");
    }

    #[test]
    fn escape_image_alt_encodes_space_and_tab_only_blank_lines() {
        assert_eq!(escape_image_alt("A\n \nB"), "A&#10; \nB");
        assert_eq!(escape_image_alt("A\n\t\nB"), "A&#10;\t\nB");
    }

    #[test]
    fn escape_image_alt_preserves_single_newline() {
        assert_eq!(escape_image_alt("A\nB"), "A\nB");
    }

    // ~keep Regression for CommonMark spec examples 642/643: a `<br>`-produced hard
    // ~keep line break (`"  \n"`, matching `NewlineStyle::Spaces`) mid-label must
    // ~keep survive, not collapse to a plain space.
    #[test]
    fn normalize_link_label_preserves_a_mid_label_spaces_style_hard_break() {
        assert_eq!(normalize_link_label("foo  \nbar"), "foo  \nbar");
    }

    #[test]
    fn normalize_link_label_preserves_a_mid_label_backslash_style_hard_break() {
        assert_eq!(normalize_link_label("foo\\\nbar"), "foo\\\nbar");
    }

    // ~keep Issue #497: a break at the label's edge is real content, not incidental
    // ~keep whitespace. `<a href="H">A<br></a>B` renders as A, a line break, then B, and
    // ~keep `[A  \n](H)B` re-parses to exactly that -- verified against comrak. These two
    // ~keep previously asserted the opposite (the break dropped), which is where the bug
    // ~keep lived: the whole label was `str::trim`-ed, and a flattened `"  \n"` is
    // ~keep indistinguishable from trailing source whitespace before a `</a>`.
    #[test]
    fn normalize_link_label_keeps_a_leading_hard_break() {
        assert_eq!(normalize_link_label("  \nbar"), "  \nbar");
    }

    #[test]
    fn normalize_link_label_keeps_a_trailing_hard_break() {
        assert_eq!(normalize_link_label("foo  \n"), "foo  \n");
    }

    #[test]
    fn normalize_link_label_keeps_a_boundary_hard_break_past_incidental_whitespace() {
        assert_eq!(normalize_link_label(" \u{a0}foo  \n "), "foo  \n");
    }

    // ~keep A run of breaks at one edge collapses to a single break: two adjacent markers
    // ~keep put a blank line in the label, and a blank line ends the paragraph the link
    // ~keep lives in -- destroying the link rather than preserving a second break nobody
    // ~keep can see.
    #[test]
    fn normalize_link_label_collapses_a_run_of_boundary_hard_breaks_to_one() {
        assert_eq!(normalize_link_label("  \n  \nbar"), "  \nbar");
        assert_eq!(normalize_link_label("foo  \n  \n"), "foo  \n");
    }

    // ~keep A break needs a line on both sides to mean anything, so a label of nothing but
    // ~keep breaks still collapses to empty and the caller's own href fallback takes over.
    #[test]
    fn normalize_link_label_drops_a_label_that_is_only_hard_breaks() {
        assert_eq!(normalize_link_label("  \n"), "");
        assert_eq!(normalize_link_label("  \n  \n"), "");
    }

    // ~keep An ordinary soft newline (no `<br>` behind it, e.g. wrapped source text)
    // ~keep still collapses to a single space -- only the two exact hard-break marker
    // ~keep shapes are preserved.
    #[test]
    fn normalize_link_label_still_collapses_an_incidental_newline_to_a_space() {
        assert_eq!(normalize_link_label("foo\nbar"), "foo bar");
        assert_eq!(normalize_link_label("foo \n bar"), "foo bar");
    }

    #[test]
    fn normalize_link_label_still_collapses_ordinary_whitespace_runs() {
        assert_eq!(normalize_link_label("foo   bar"), "foo bar");
        assert_eq!(normalize_link_label("  foo bar  "), "foo bar");
    }

    // ~keep Regression for a real collision in an earlier version of this function: it used
    // ~keep Private Use Area code points (U+E000/U+E001) as placeholders for the hard-break
    // ~keep markers, reasoning that no producer this crate parses assigns them. That is false
    // ~keep -- icon fonts live in the PUA (Bootstrap 3's Glyphicons start at U+E001) -- so a
    // ~keep label already containing that literal character collided with the placeholder and
    // ~keep reappeared as a spurious hard break once the placeholder was "restored". The
    // ~keep trigger needs both a literal PUA character AND a real hard-break marker in the
    // ~keep same label -- a PUA character alone never entered the placeholder-substitution
    // ~keep branch at all, which is why this was not caught by the other tests above.
    #[test]
    fn normalize_link_label_does_not_confuse_a_literal_pua_character_with_the_spaces_sentinel() {
        assert_eq!(normalize_link_label("a\u{E000}b  \nc"), "a\u{E000}b  \nc");
    }

    #[test]
    fn normalize_link_label_does_not_confuse_a_literal_pua_character_with_the_backslash_sentinel() {
        assert_eq!(normalize_link_label("a\u{E001}b\\\nc"), "a\u{E001}b\\\nc");
    }

    // ~keep The Glyphicon code point itself (U+E001) is exactly the second placeholder this
    // ~keep function used to use, so this pins the specific real-world icon-font byte, not
    // ~keep just "some" PUA character.
    #[test]
    fn normalize_link_label_preserves_a_glyphicon_code_point_alongside_a_spaces_style_hard_break() {
        assert_eq!(normalize_link_label("\u{E001} foo  \nbar"), "\u{E001} foo  \nbar");
    }

    #[test]
    fn normalize_link_label_preserves_a_glyphicon_code_point_alongside_a_backslash_style_hard_break() {
        assert_eq!(normalize_link_label("\u{E001} foo\\\nbar"), "\u{E001} foo\\\nbar");
    }

    // ~keep ── chomp_inline whitespace-only dedup (issue #481) ──────────────────────────

    #[test]
    fn chomp_inline_returns_a_single_space_prefix_for_whitespace_only_content() {
        assert_eq!(chomp_inline(" "), (" ", "", ""));
    }

    #[test]
    fn chomp_inline_returns_a_single_space_prefix_for_a_multi_char_whitespace_only_run() {
        assert_eq!(chomp_inline("   "), (" ", "", ""));
        assert_eq!(chomp_inline("\t "), (" ", "", ""));
    }

    #[test]
    fn chomp_inline_preserves_a_hard_trailing_linebreak_over_the_whitespace_only_collapse() {
        assert_eq!(chomp_inline("  \n"), (" ", "  \n", ""));
        assert_eq!(chomp_inline("\\\n"), ("", "\\\n", ""));
    }

    #[test]
    fn chomp_inline_leaves_non_whitespace_content_unaffected() {
        assert_eq!(chomp_inline(" a "), (" ", " ", "a"));
        assert_eq!(chomp_inline("a"), ("", "", "a"));
    }

    #[test]
    fn chomp_inline_returns_empty_for_empty_input() {
        assert_eq!(chomp_inline(""), ("", "", ""));
    }

    // ~keep ── merge_adjacent_emphasis (issue #483) ─────────────────────────────────────

    #[test]
    fn merge_adjacent_emphasis_pops_a_lone_matching_close_marker() {
        let mut output = String::from("*A*");
        assert!(merge_adjacent_emphasis(&mut output, '*', 1));
        assert_eq!(output, "*A");
    }

    #[test]
    fn merge_adjacent_emphasis_pops_a_double_matching_close_marker() {
        let mut output = String::from("**A**");
        assert!(merge_adjacent_emphasis(&mut output, '*', 2));
        assert_eq!(output, "**A");
    }

    #[test]
    fn merge_adjacent_emphasis_returns_false_when_output_does_not_end_with_the_marker() {
        let mut output = String::from("*A* ");
        assert!(!merge_adjacent_emphasis(&mut output, '*', 1));
        assert_eq!(output, "*A* ", "untouched on refusal");
    }

    #[test]
    fn merge_adjacent_emphasis_refuses_a_run_longer_than_count() {
        // ~keep `***x***` must not be half-eaten: a trailing run of 3 is not "exactly 2".
        let mut output = String::from("***x***");
        assert!(!merge_adjacent_emphasis(&mut output, '*', 2));
        assert_eq!(output, "***x***");
    }

    #[test]
    fn merge_adjacent_emphasis_refuses_when_the_run_is_shorter_than_count() {
        let mut output = String::from("*A*");
        assert!(!merge_adjacent_emphasis(&mut output, '*', 2));
        assert_eq!(output, "*A*");
    }

    #[test]
    fn merge_adjacent_emphasis_refuses_an_escaped_literal_asterisk() {
        // ~keep `\*` immediately before the run: the preceding character is a backslash,
        // ~keep so this is a literal escaped asterisk, not a mergeable close marker.
        let mut output = String::from(r"a\*");
        assert!(!merge_adjacent_emphasis(&mut output, '*', 1));
        assert_eq!(output, r"a\*");
    }

    #[test]
    fn merge_adjacent_emphasis_returns_false_for_zero_count() {
        let mut output = String::from("*A*");
        assert!(!merge_adjacent_emphasis(&mut output, '*', 0));
        assert_eq!(output, "*A*");
    }

    #[test]
    fn merge_adjacent_emphasis_respects_the_underscore_symbol_variant() {
        let mut output = String::from("__A__");
        assert!(merge_adjacent_emphasis(&mut output, '_', 2));
        assert_eq!(output, "__A");
    }

    #[test]
    fn opens_block_reads_an_ordered_marker_by_its_value() {
        for line in ["1. x", "1) x", "01. x", "001) x", "000000001. x", "01.\tx"] {
            assert!(opens_block(line), "{line:?} starts a list at 1");
        }
        for line in [
            "0000000001. x",
            "2. x",
            "02. x",
            "0. x",
            "00. x",
            "10. x",
            "11. x",
            "01.",
            "01. ",
            "01.x",
            "01",
        ] {
            assert!(!opens_block(line), "{line:?} cannot interrupt a paragraph");
        }
        assert_eq!(block_opener_offset("001) x"), Some(3));
        assert_eq!(block_opener_offset("1. x"), Some(1));
    }

    fn escaped_continuation(before: &str, text: &str) -> String {
        let mut buffer = format!("{before}{text}");
        escape_continuation_line_start(&mut buffer, before.len(), false);
        buffer
    }

    #[test]
    fn should_escape_block_prefixes_split_across_text_fragments() {
        for (before, text, in_list_item, expected) in [
            ("", "# x", false, "\\# x"),
            ("1.", " x", false, "1\\. x"),
            ("a\u{c} ", "# x", false, "a\u{c} # x"),
            ("\u{c} ", "# x", false, "\u{c} # x"),
            ("1.", "\u{c} x", false, "1.\u{c} x"),
            ("- ", "# x", true, "- \\# x"),
        ] {
            let mut buffer = format!("{before}{text}");
            escape_block_start(&mut buffer, before.len(), in_list_item, false);
            assert_eq!(buffer, expected, "{before:?} + {text:?}");
        }
    }

    #[test]
    fn escape_continuation_line_start_escapes_a_line_after_text() {
        assert_eq!(escaped_continuation("a  \n", "1) t"), "a  \n1\\) t");
        assert_eq!(escaped_continuation("x\n- a  \n  ", "- t"), "x\n- a  \n  \\- t");
        assert_eq!(escaped_continuation("a\\\n\t", "> t"), "a\\\n\t\\> t");
        assert_eq!(escaped_continuation("a  \n", "-\nx"), "a  \n\\-\nx");
    }

    #[test]
    fn escape_continuation_line_start_uses_the_enclosing_inline_buffer_context() {
        let mut buffer = String::from("- t");
        escape_continuation_line_start(&mut buffer, 0, true);
        assert_eq!(buffer, "\\- t");
    }

    #[test]
    fn escape_continuation_line_start_leaves_other_text_alone() {
        for (before, text) in [
            ("", "1) t"),
            ("a", "1) t"),
            ("a  \nb ", "1) t"),
            ("a\n\n", "1) t"),
            ("a\n  \n", "1) t"),
            ("a  \n", "2. t"),
            ("a  \n", "    1) t"),
            ("a  \n", "1)\n- t"),
        ] {
            assert_eq!(
                escaped_continuation(before, text),
                format!("{before}{text}"),
                "{before:?} {text:?}"
            );
        }
    }

    #[test]
    fn escape_paragraph_start_escapes_a_link_reference_definition() {
        for text in [
            "[a]: b",
            "[a]:",
            "[a]:b \"t\"",
            "[a]:b 't'",
            "[a]: b (t)",
            "[a]: <b c>",
            "[a]: <>",
            "[**a**]: b",
            "[a\\]]: b",
            "[a]: b(c)",
            "[a]: b\"t\"",
        ] {
            assert_eq!(escape_paragraph_start(text, b'='), format!("\\{text}"), "{text:?}");
        }
        assert_eq!(escape_paragraph_start("[a]: b", b'-'), "\\[a]: b");
    }

    #[test]
    fn escape_paragraph_start_leaves_a_line_that_starts_no_definition() {
        for text in [
            "[a]: b c",
            "[a]: b \"t\" x",
            "[a]: b\\ c",
            "[a]: b \"t",
            "[a]: b (t(u))",
            "[a]: b (t(u)",
            "[a]: <b>\"t\"",
            "[a[b]: c",
            "[a]: b(",
            "[a]: <b",
            "[a]: <b<c>",
            "[a] b",
            "[a]",
            "[ ]: b",
            "[a][b]: c",
            "[[a]]: b",
            "a [b]: c",
            "plain text",
        ] {
            assert_eq!(escape_paragraph_start(text, b'='), text, "{text:?}");
        }
        assert_eq!(escape_paragraph_start("[a]:", b'-'), "[a]:");
        let long_label = format!("[{}]: b", "é".repeat(1000));
        assert_eq!(escape_paragraph_start(&long_label, b'='), long_label);
        let longest_label = format!("[{}]: b", "é".repeat(999));
        assert_eq!(
            escape_paragraph_start(&longest_label, b'='),
            format!("\\{longest_label}")
        );
    }

    #[test]
    fn atx_closing_sequence_offset_finds_a_run_the_line_closes_on() {
        assert_eq!(atx_closing_sequence_offset("#"), Some(0));
        assert_eq!(atx_closing_sequence_offset("##"), Some(0));
        assert_eq!(atx_closing_sequence_offset("a #"), Some(2));
        assert_eq!(atx_closing_sequence_offset("a ##"), Some(2));
        assert_eq!(atx_closing_sequence_offset("a\t#"), Some(2));
        assert_eq!(atx_closing_sequence_offset("a # #"), Some(4));
        assert_eq!(atx_closing_sequence_offset("**a** #"), Some(6));
        for text in ["a#", "a \\#", "a \\\\#", "#a", "a # b", "plain text", ""] {
            assert_eq!(atx_closing_sequence_offset(text), None, "{text:?}");
        }
    }

    #[test]
    fn merge_adjacent_emphasis_returns_false_on_empty_output() {
        let mut output = String::new();
        assert!(!merge_adjacent_emphasis(&mut output, '*', 1));
        assert_eq!(output, "");
    }
}
