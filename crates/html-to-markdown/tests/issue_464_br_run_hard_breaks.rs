// ~keep The inner attribute below is a crate-level Rust attribute, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests for issue #464: consecutive and trailing `<br>` were mishandled under
//! `newline_style="backslash"`. Two distinct defects were involved:
//!
//! - A run of N consecutive `<br>` collapsed to a single hard break plus a stray blank
//!   line instead of N hard breaks. `line_break.rs`'s dispatch treated "the previous
//!   sibling already emitted a break" (`output.ends_with('\n')`) the same as "nothing
//!   precedes this `<br>` on the current line" — so the second and later `<br>` in a run
//!   silently no-opped instead of emitting their own marker.
//! - A trailing run of `<br>` at the end of a block left literal, visible backslash
//!   characters behind. `CommonMark`'s hard-break syntax has no effect at the end of a
//!   block (<https://spec.commonmark.org/spec#hard-line-breaks>), but unlike the
//!   two-space style's leftover marker (invisible trailing whitespace), a leftover `\`
//!   is not whitespace, so later normalization passes could not clean it up.
//!
//! Both defects apply equally to the two-space style's *mid-content* run case; the
//! two-space style's *trailing* case was already correct (see
//! `should_keep_a_single_trailing_two_space_break`) because its leftover marker is inert.

use html_to_markdown_rs::{ConversionOptions, NewlineStyle};

fn convert(html: &str, options: ConversionOptions) -> String {
    html_to_markdown_rs::convert(html, Some(options))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn backslash_options() -> ConversionOptions {
    ConversionOptions {
        newline_style: NewlineStyle::Backslash,
        ..Default::default()
    }
}

// ~keep `newline_style: Spaces` is the default, and this markup has nothing else that
// ~keep forces Tier-2 (see `tier1/router.rs`), so it would otherwise run through the
// ~keep Tier-1 byte scanner — a separate implementation from the `line_break.rs` /
// ~keep `paragraph.rs` code this fix touches. `debug: true` is the router's documented
// ~keep Tier-2-forcing gate and does not itself affect rendered output, so it pins these
// ~keep cases onto the exact code path this fix changes.
fn spaces_options_on_tier2() -> ConversionOptions {
    ConversionOptions {
        newline_style: NewlineStyle::Spaces,
        debug: true,
        ..Default::default()
    }
}

// ~keep ── The three reporter cases, verbatim (backslash style) ────────────────────────

#[test]
fn should_emit_one_backslash_break_per_br_in_a_two_br_run() {
    assert_eq!(convert("<p>A<br/><br/>B</p>", backslash_options()), "A\\\n\\\nB\n");
}

#[test]
fn should_emit_one_backslash_break_per_br_in_a_three_br_run() {
    assert_eq!(
        convert("<p>A<br/><br/><br/>B</p>", backslash_options()),
        "A\\\n\\\n\\\nB\n"
    );
}

#[test]
fn should_drop_a_trailing_br_run_entirely() {
    assert_eq!(convert("<p>A<br/><br/><br/></p>", backslash_options()), "A\n");
}

// ~keep ── Equivalent cases, two-space style ───────────────────────────────────────────

#[test]
fn a_two_br_run_collapses_under_the_two_space_style_because_a_blank_line_cannot_carry_a_marker() {
    // ~keep NOT the same expectation as the backslash case above, and deliberately so. The
    // ~keep second break's marker would have to live on an otherwise-empty line, and this
    // ~keep crate strips trailing whitespace from blank lines by design -- see the locked
    // ~keep `whitespace_only_output_test.rs::should_keep_hard_break_after_content_but_not_on_a_blank_line`.
    // ~keep So the two-space style CANNOT express consecutive hard breaks; that limitation is
    // ~keep exactly why the backslash style exists and why issue #464 was reported against it.
    // ~keep Pinned here so a future change to the blank-line rule shows up as this test failing.
    assert_eq!(convert("<p>A<br/><br/>B</p>", spaces_options_on_tier2()), "A  \n\nB\n");
}

#[test]
fn a_three_br_run_collapses_the_same_way_under_the_two_space_style() {
    // ~keep Byte-identical to the two-<br> case: every marker after the first would need a
    // ~keep blank line to sit on. See the two-<br> test above for why that is by design.
    assert_eq!(
        convert("<p>A<br/><br/><br/>B</p>", spaces_options_on_tier2()),
        "A  \n\nB\n"
    );
}

#[test]
fn should_keep_a_single_trailing_two_space_break() {
    // ~keep Unlike the backslash style, a trailing "  \n" is invisible whitespace rather
    // ~keep than a visible artifact, so a trailing run is left with the same single
    // ~keep marker a lone trailing <br> already produces (matches
    // ~keep `whitespace_only_output_test.rs`'s "Only  \n") instead of being dropped.
    assert_eq!(convert("<p>A<br/><br/><br/></p>", spaces_options_on_tier2()), "A  \n");
}

// ~keep ── Boundary cases a naive per-style fix could still get wrong ──────────────────

#[test]
fn should_keep_a_single_mid_paragraph_br_working() {
    // ~keep Regression guard: a lone <br> with real content on both sides must still
    // ~keep emit exactly one break.
    assert_eq!(convert("<p>A<br/>B</p>", backslash_options()), "A\\\nB\n");
}

#[test]
fn should_emit_a_break_for_each_run_when_runs_are_separated_by_text() {
    assert_eq!(
        convert("<p>A<br/><br/>B<br/>C</p>", backslash_options()),
        "A\\\n\\\nB\\\nC\n"
    );
}

#[test]
fn should_drop_a_leading_br_run_inside_a_paragraph() {
    assert_eq!(convert("<p><br/><br/>A</p>", backslash_options()), "A\n");
}

#[test]
fn should_produce_no_output_when_a_block_is_only_a_br_run() {
    // ~keep Nothing precedes and nothing follows the run: the leading-run rule empties
    // ~keep the block before any content is ever appended, so the block contributes no
    // ~keep paragraph separator either.
    assert_eq!(convert("<p><br/><br/></p>", backslash_options()), "");
}

// ~keep ── Follow-up: a <br> terminated by a BLOCK boundary, not just by the end of a <p> ─
// ~keep
// ~keep The first fix stripped a trailing backslash-break run inside `paragraph::handle`
// ~keep only, so it covered `<p>...<br></p>` and nothing else. The reporter found the gap
// ~keep with `A<br><p>B` (issue #464 follow-up comment): the <br> is top-level inline
// ~keep content terminated by a following block, `paragraph::handle` never runs for it, and
// ~keep the marker survives as a visible stray backslash. The same hole existed at the end
// ~keep of a <div>, a list item, a blockquote, and at the end of the document.
// ~keep
// ~keep CommonMark's rule is about the end of a BLOCK, not the end of a paragraph element:
// ~keep <https://spec.commonmark.org/spec#hard-line-breaks>.

#[test]
fn should_drop_a_br_terminated_by_a_following_paragraph() {
    // ~keep The reporter's follow-up case, verbatim.
    assert_eq!(convert("A<br><p>B", backslash_options()), "A\n\nB\n");
}

#[test]
fn should_drop_a_br_run_terminated_by_a_following_paragraph() {
    assert_eq!(convert("A<br><br><p>B", backslash_options()), "A\n\nB\n");
}

#[test]
fn should_drop_a_trailing_br_at_the_end_of_the_document() {
    assert_eq!(convert("A<br>", backslash_options()), "A\n");
}

#[test]
fn should_drop_a_br_before_every_kind_of_following_block() {
    // ~keep The terminator is a block boundary, so which block follows must not matter.
    // ~keep Table-driven because a fix that special-cases <p> would pass the case above
    // ~keep and still leak a backslash before a heading, a list, or a rule.
    for (html, expected) in [
        ("A<br><h2>B</h2>", "A\n\n## B\n"),
        ("A<br><ul><li>B</li></ul>", "A\n\n- B\n"),
        ("A<br><blockquote>B</blockquote>", "A\n\n> B\n"),
        ("A<br><hr>", "A\n\n---\n"),
        ("A<br><div>B</div>", "A\n\nB\n"),
        ("A<br><table><tr><td>B</td></tr></table>", "A\n\n| B |\n| --- |\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_br_separated_from_the_next_block_by_whitespace() {
    // ~keep Source whitespace between the <br> and the next block must not hide the
    // ~keep boundary from the strip.
    assert_eq!(convert("A<br>\n<p>B</p>", backslash_options()), "A\n\nB\n");
}

#[test]
fn should_drop_a_trailing_br_at_the_end_of_a_container_block() {
    for (html, expected) in [
        ("<div>A<br></div><p>B</p>", "A\n\nB\n"),
        ("<div>A<br><br></div>", "A\n"),
        ("<ul><li>A<br></li></ul>", "- A\n"),
        ("<blockquote>A<br></blockquote>", "> A\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_br_before_a_block_nested_inside_a_list_item() {
    // ~keep The paragraph after the item's text is its own paragraph in the item (issue #583).
    assert_eq!(
        convert("<ul><li>A<br><p>B</p></li></ul>", backslash_options()),
        "- A\n\n  B\n"
    );
}

#[test]
fn should_drop_a_trailing_br_through_nested_containers() {
    // ~keep The <br> ends an inner <div> that ends an outer <div>; the strip has to happen
    // ~keep at whichever level actually terminates the inline run, not only the outermost.
    assert_eq!(
        convert("<div><div>A<br></div></div><p>B</p>", backslash_options()),
        "A\n\nB\n"
    );
}

#[test]
fn should_keep_a_br_that_precedes_real_inline_content() {
    // ~keep The counterweight to every test above: these breaks have a next line to reach,
    // ~keep so a fix that strips on any block *element* boundary rather than on an empty
    // ~keep trailing run would silently eat them.
    for (html, expected) in [
        ("<p>A<br><em>B</em></p>", "A\\\n*B*\n"),
        ("A<br><span>B</span>", "A\\\nB\n"),
        ("<p>A<br/>B</p>", "A\\\nB\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_leave_the_two_space_style_unchanged_at_block_boundaries() {
    // ~keep The two-space marker is invisible trailing whitespace, so it is deliberately
    // ~keep left in place (see `should_keep_a_single_trailing_two_space_break`). Pinned for
    // ~keep the block-boundary cases too, so the follow-up fix cannot quietly widen into
    // ~keep the style it was never meant to touch.
    for (html, expected) in [
        ("A<br><p>B", "A  \n\nB\n"),
        ("A<br>", "A  \n"),
        ("<div>A<br></div><p>B</p>", "A  \n\nB\n"),
        ("<ul><li>A<br></li></ul>", "- A  \n"),
    ] {
        assert_eq!(convert(html, spaces_options_on_tier2()), expected, "input: {html}");
    }
}

#[test]
fn should_still_open_a_line_for_a_leading_top_level_br() {
    // ~keep Issue #112. The follow-up fix strips TRAILING runs; a leading break must
    // ~keep survive, and it is the case most likely to be caught by an over-broad strip.
    assert_eq!(convert("<br>A", backslash_options()), "\nA\n");
}

// ~keep ── Nested and container-terminated cases ────────────────────────────────────────
// ~keep
// ~keep Several handlers walk their children into a FRESH String and splice the trimmed
// ~keep result back (blockquote, sectioning, details/summary, figure, dl/dt/dd, list item).
// ~keep `content.trim()` cannot remove a trailing "\\\n" -- trim takes the newline and
// ~keep leaves the backslash -- so each of these was its own instance of the same defect,
// ~keep reachable without a <p> anywhere in the markup.

#[test]
fn should_drop_a_trailing_br_inside_a_blockquote_however_nested() {
    for (html, expected) in [
        ("<blockquote><div>A<br></div></blockquote>", "> A\n"),
        ("<blockquote><ul><li>A<br></li></ul></blockquote>", "> - A\n"),
        ("<div><blockquote>A<br></blockquote></div><p>B</p>", "> A\n\nB\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_br_at_the_end_of_a_list_item_that_has_siblings() {
    // ~keep The next <li> is a block boundary just as much as a following <p> is.
    assert_eq!(
        convert("<ul><li>A<br></li><li>B</li></ul>", backslash_options()),
        "- A\n- B\n"
    );
}

#[test]
fn should_drop_a_br_before_a_nested_list() {
    for (html, expected) in [
        ("<div>A<br><ul><li>B</li></ul></div>", "A\n\n- B\n"),
        ("<ul><li>A<br><ul><li>B</li></ul></li></ul>", "- A\n  * B\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_trailing_br_in_sectioning_and_disclosure_containers() {
    for (html, expected) in [
        ("<section>A<br></section><p>B</p>", "A\n\nB\n"),
        ("<article>A<br></article>", "A\n"),
        ("<details><summary>S</summary>A<br></details>", "**S**\n\nA\n"),
        ("<div>A<br></div>", "A\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}

#[test]
fn should_drop_a_br_at_the_end_of_a_definition_term() {
    assert_eq!(
        convert("<dl><dt>A<br></dt><dd>B</dd></dl>", backslash_options()),
        "A\n\nB\n"
    );
}

#[test]
fn should_not_let_a_trailing_br_escape_a_closing_emphasis_delimiter() {
    // ~keep The worst instance of this defect. In a figcaption the caption text is wrapped
    // ~keep in emphasis AFTER the break marker is appended, so the leftover backslash ends
    // ~keep up between the text and the closing "*" -- producing `*A\*`, where the `\`
    // ~keep escapes the delimiter and the emphasis never closes. That is corrupted Markdown,
    // ~keep not merely a visible stray character like the other cases here.
    assert_eq!(
        convert("<figure><figcaption>A<br></figcaption></figure>", backslash_options()),
        "*A*\n"
    );
    assert_eq!(
        convert("<div><strong>A<br></strong></div>", backslash_options()),
        "**A**\n"
    );
}

#[test]
fn should_drop_a_whole_trailing_br_run_from_a_container_before_a_following_block() {
    assert_eq!(
        convert("<div>A<br><br><br></div><p>B</p>", backslash_options()),
        "A\n\nB\n"
    );
}

#[test]
fn should_keep_a_mid_block_br_while_dropping_the_trailing_one() {
    // ~keep Both breaks are in the same block; only the second has nothing after it.
    assert_eq!(
        convert("<p>A<br><strong>B</strong><br></p>", backslash_options()),
        "A\\\n**B**\n"
    );
}

#[test]
fn should_leave_already_correct_nested_cases_alone() {
    // ~keep These were already right before the follow-up fix. Pinned so a broad strip
    // ~keep cannot regress them into dropping a legitimate break or a blank line.
    for (html, expected) in [
        ("<ol><li><p>A<br></p><p>B</p></li></ol>", "1. A\n\n   B\n"),
        ("<p>A<br></p><ul><li>B</li></ul>", "A\n\n- B\n"),
    ] {
        assert_eq!(convert(html, backslash_options()), expected, "input: {html}");
    }
}
