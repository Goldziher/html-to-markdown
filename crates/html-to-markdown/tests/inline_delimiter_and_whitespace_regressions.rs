#![allow(missing_docs)]

//! Regression tests for two whole-family Tier-2 inline defects that the issue #481 and
//! #483 fixes repaired for `<em>`/`<strong>` only, plus one tag that was never wired into
//! the Tier-2 walker at all.
//!
//! - **`<strike>` was invisible to Tier-2.** `converter/main.rs`'s inline dispatch arm,
//!   `dispatch_inline_handler` and `inline/semantic/mod.rs` all listed `del`/`s` without it,
//!   so `<strike>` fell through to the generic child walk and its strikethrough was dropped.
//!   Tier-1 (`tier1/tags.rs`), `types/structure_builder.rs` and `roundtrip_fixpoint.rs`'s
//!   module doc all treat `strike` as a strikethrough synonym, so this was also a
//!   Tier-1/Tier-2 divergence.
//! - **Issue #483's delimiter-run adjacency applies to every repeated-character delimiter**,
//!   not just `*`/`**`: `~~` (`del`/`s`/`strike`), `==` (`ins`, `mark`), the
//!   `strong_em_symbol` reused by `var`/`dfn`, and a code span's backtick. Verified against
//!   the comrak oracle: `~~A~~~~B~~` reparses as ONE strikethrough whose text is `A~~~~B`,
//!   and two adjacent single-backtick code spans reparse as ONE code span whose text
//!   carries the two literal backticks that used to separate them -- both corrupt.
//! - **Issue #481's whitespace-only body applies to every inline handler.** A body that is
//!   non-empty but all whitespace stands for a word separator; handlers that tested
//!   `!trimmed.is_empty()` with no `else` branch dropped it, joining the words either side.

use html_to_markdown_rs::convert;
use html_to_markdown_rs::options::ConversionOptions;

fn content(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn content_with(html: &str, options: ConversionOptions) -> String {
    convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
        .trim()
        .to_owned()
}

// ~keep ── <strike> is a strikethrough synonym, like <del> and <s> ────────────────────────

#[test]
fn should_render_strike_as_strikethrough_like_del_and_s() {
    assert_eq!(content("<p><strike>gone</strike></p>"), "~~gone~~");
    assert_eq!(content("<p><del>gone</del></p>"), "~~gone~~");
    assert_eq!(content("<p><s>gone</s></p>"), "~~gone~~");
}

#[test]
fn should_suppress_strike_markers_inside_a_code_span() {
    // ~keep Mirrors <del>/<s>: handle_strikethrough emits no `~~` under `in_code`.
    assert_eq!(content("<p><code>a<strike>b</strike></code></p>"), "`ab`");
}

#[test]
fn should_render_strike_as_djot_when_the_output_format_is_djot() {
    let options = ConversionOptions {
        output_format: html_to_markdown_rs::OutputFormat::Djot,
        ..ConversionOptions::default()
    };
    assert_eq!(content_with("<p><strike>gone</strike></p>", options), "{-gone-}");
}

// ~keep ── Issue #483's delimiter-run adjacency, for every other repeated-character pair ──

#[test]
fn should_merge_adjacent_strikethrough_siblings_into_one_run() {
    assert_eq!(content("<p><del>A</del><del>B</del></p>"), "~~AB~~");
    assert_eq!(content("<p><s>A</s><s>B</s></p>"), "~~AB~~");
    assert_eq!(content("<p><strike>A</strike><strike>B</strike></p>"), "~~AB~~");
}

#[test]
fn should_merge_adjacent_strikethrough_siblings_across_the_del_s_and_strike_synonyms() {
    // ~keep All three tags render the same `~~` pair, so the run they form is the same run
    // ~keep regardless of which synonym each sibling used.
    assert_eq!(content("<p><del>A</del><s>B</s><strike>C</strike></p>"), "~~ABC~~");
}

#[test]
fn should_preserve_plain_content_of_adjacent_inserted_siblings() {
    assert_eq!(content("<p><ins>A</ins><ins>B</ins></p>"), "AB");
}

#[test]
fn should_merge_adjacent_mark_siblings_into_one_run() {
    assert_eq!(content("<p><mark>A</mark><mark>B</mark></p>"), "==AB==");
}

#[test]
fn should_merge_adjacent_variable_and_definition_siblings_into_one_run() {
    assert_eq!(content("<p><var>A</var><var>B</var></p>"), "*AB*");
    assert_eq!(content("<p><dfn>A</dfn><dfn>B</dfn></p>"), "*AB*");
}

#[test]
fn should_merge_adjacent_code_siblings_into_one_span() {
    assert_eq!(content("<p><code>A</code><code>B</code></p>"), "`AB`");
    assert_eq!(content("<p><kbd>A</kbd><kbd>B</kbd></p>"), "`AB`");
    assert_eq!(content("<p><samp>A</samp><samp>B</samp></p>"), "`AB`");
}

#[test]
fn should_not_merge_inline_siblings_separated_by_a_real_space() {
    // ~keep Control: a genuine separator means the delimiters are not adjacent, so each
    // ~keep element must keep its own pair.
    assert_eq!(content("<p><del>A</del> <del>B</del></p>"), "~~A~~ ~~B~~");
    assert_eq!(content("<p><ins>A</ins> <ins>B</ins></p>"), "A B");
    assert_eq!(content("<p><var>A</var> <var>B</var></p>"), "*A* *B*");
    assert_eq!(content("<p><code>A</code> <code>B</code></p>"), "`A` `B`");
}

#[test]
fn should_not_merge_inline_siblings_of_different_kinds() {
    // ~keep Control: `~~A~~` followed by `==B==` shares no delimiter character, so there is
    // ~keep no run to merge and both pairs must survive.
    assert_eq!(content("<p><del>A</del><ins>B</ins></p>"), "~~A~~B");
}

#[test]
fn should_not_merge_a_strikethrough_run_that_is_really_literal_text() {
    // ~keep Control: the preceding `~~` is plain prose, not a sibling element's close
    // ~keep marker, so the DOM-sibling requirement must refuse the merge.
    assert_eq!(content("<p>a~~<del>B</del></p>"), "a~~~~B~~");
}

#[test]
fn should_keep_djot_strikethrough_and_inserted_pairs_unmerged() {
    // ~keep Djot's `{-`/`-}` and `{+`/`+}` are not runs of one character, so there is
    // ~keep nothing to merge and the helper must leave both pairs intact.
    let options = ConversionOptions {
        output_format: html_to_markdown_rs::OutputFormat::Djot,
        ..ConversionOptions::default()
    };
    assert_eq!(
        content_with("<p><del>A</del><del>B</del></p>", options.clone()),
        "{-A-}{-B-}"
    );
    assert_eq!(content_with("<p><ins>A</ins><ins>B</ins></p>", options), "{+A+}{+B+}");
}

// ~keep ── Issue #481's whitespace-only body, for every other inline handler ──────────────

#[test]
fn should_keep_the_word_separator_a_whitespace_only_inline_body_stands_for() {
    for tag in ["ins", "sub", "sup", "var", "dfn", "abbr", "q", "mark"] {
        let html = format!("<p>A<{tag}> </{tag}>B</p>");
        assert_eq!(content(&html), "A B", "whitespace-only <{tag}> dropped the separator");
    }
}

#[test]
fn should_render_a_whitespace_only_code_body_as_a_code_span_rather_than_a_separator() {
    // ~keep `<code> </code>` is a code span whose content IS a space, and CommonMark spells
    // ~keep that exactly as `` ` ` ``: the "strip one space from each end" rule applies only
    // ~keep when the content is NOT entirely spaces, so no delimiter padding is wanted (and
    // ~keep padding it would change one space into three -- spec example 138). Collapsing the
    // ~keep span to a bare separator, as the delimiter-less elements above must do, would
    // ~keep instead lose the code span itself.
    for tag in ["code", "kbd", "samp"] {
        let html = format!("<p>A<{tag}> </{tag}>B</p>");
        assert_eq!(
            content(&html),
            "A` `B",
            "whitespace-only <{tag}> did not survive as a code span"
        );
    }
}

#[test]
fn should_not_double_the_separator_when_a_real_space_already_precedes_it() {
    for tag in ["ins", "sub", "sup", "var", "dfn", "mark"] {
        let html = format!("<p>A <{tag}> </{tag}>B</p>");
        assert_eq!(content(&html), "A B", "whitespace-only <{tag}> doubled the separator");
    }
}

#[test]
fn should_still_drop_a_genuinely_empty_inline_element() {
    // ~keep Control: no bytes at all between the tags is not a word separator.
    for tag in ["ins", "sub", "sup", "var", "dfn", "code", "mark", "del"] {
        let html = format!("<p>A<{tag}></{tag}>B</p>");
        assert_eq!(content(&html), "AB", "empty <{tag}> invented a separator");
    }
}
