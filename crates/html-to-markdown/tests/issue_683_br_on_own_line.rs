// ~keep The inner attributes below are crate-level Rust attributes, not a shell shebang.
#![allow(missing_docs)]
#![cfg(feature = "testkit")]

//! Regression tests for issue #683: a `<br>` on its own source line became a paragraph
//! break. The text node before it (`"First\n"`) turned its trailing source newline into a
//! `'\n'` outside a paragraph, so the `<br>`'s hard-break marker landed on a line of its
//! own (`"First\n  \n"`). Cleanup then reduced that whitespace-only line to a blank line.
//! Under the backslash style, the `\` was left stranded on its own line instead.

use std::time::{Duration, Instant};

use html_to_markdown_rs::prescan::{self, PrescanReport};
use html_to_markdown_rs::{ConversionOptions, NewlineStyle, TierStrategy, convert, tier1};

fn options(newline_style: NewlineStyle, tier_strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        extract_metadata: false,
        newline_style,
        tier_strategy,
        ..ConversionOptions::default()
    }
}

/// The full converter's output.
fn full(html: &str, newline_style: NewlineStyle) -> String {
    convert(html, Some(options(newline_style, TierStrategy::Tier2)))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

/// The fast converter's own output: an `Err` is a bail, never a fallback to the full converter.
/// It writes only the spaces style (the router sends the backslash style to the full converter).
fn fast(html: &str) -> String {
    tier1::run(
        html,
        &PrescanReport::default(),
        &options(NewlineStyle::Spaces, TierStrategy::Tier1),
    )
    .unwrap_or_else(|reason| panic!("the fast converter must not bail on {html:?}: {reason:?}"))
}

/// The output of the default route, which picks the converter.
fn auto(html: &str) -> String {
    convert(html, Some(options(NewlineStyle::Spaces, TierStrategy::Auto)))
        .expect("conversion must succeed")
        .content
        .unwrap_or_default()
}

fn fast_after_prescan(html: &str) -> String {
    let (cleaned, report) = prescan::run(html);
    tier1::run(&cleaned, &report, &options(NewlineStyle::Spaces, TierStrategy::Tier1))
        .unwrap_or_else(|reason| panic!("the fast converter must not bail on {html:?}: {reason:?}"))
}

/// `own_line` gives what `inline`, the same markup without the source newlines, gives, in both
/// converters.
fn assert_matches_inline_form(own_line: &str, inline: &str) {
    assert_eq!(
        full(own_line, NewlineStyle::Spaces),
        full(inline, NewlineStyle::Spaces),
        "full converter: {own_line:?} vs {inline:?}"
    );
    assert_eq!(
        fast(own_line),
        fast(inline),
        "fast converter: {own_line:?} vs {inline:?}"
    );
}

#[test]
fn should_keep_a_top_level_br_on_its_own_line_as_a_hard_break() {
    let html = "First\n<br>\nSecond";
    assert_eq!(full(html, NewlineStyle::Spaces), "First  \nSecond\n");
    assert_eq!(fast(html), "First  \nSecond\n");
}

#[test]
fn should_attach_the_backslash_marker_to_the_preceding_line() {
    for html in [
        "First\n<br>\nSecond",
        "First\n<!-- c --><br>Second",
        "<span>First\n</span><br>Second",
        "<span>First\n<!-- c --></span><br>Second",
        "<small>First\n</small><br>Second",
    ] {
        assert_eq!(full(html, NewlineStyle::Backslash), "First\\\nSecond\n", "{html:?}");
    }
    assert_eq!(
        full("<ul><li><span>a\n</span><br>b</li></ul>", NewlineStyle::Backslash),
        "- a\\\n  b\n"
    );
}

#[test]
fn should_keep_a_br_on_its_own_line_in_a_block_as_a_hard_break() {
    assert_matches_inline_form("<div>First\n<br>\nSecond</div>", "<div>First<br>Second</div>");
    assert_matches_inline_form("<ul><li>a\n<br>\nb</li></ul>", "<ul><li>a<br>b</li></ul>");
    assert_matches_inline_form("<blockquote>a\n<br>\nb</blockquote>", "<blockquote>a<br>b</blockquote>");
}

#[test]
fn should_keep_a_br_preceded_only_by_a_source_newline_as_a_hard_break() {
    assert_matches_inline_form("First\n<br>Second", "First<br>Second");
}

#[test]
fn should_decode_a_character_reference_newline_before_deciding_the_br_break_kind() {
    let html = "First&#10;<br>Second";
    assert_eq!(full(html, NewlineStyle::Spaces), "First  \nSecond\n");
    assert_eq!(fast(html), "First  \nSecond\n");
    assert_eq!(auto(html), "First  \nSecond\n");
}

#[test]
fn should_recognize_crlf_blank_lines_before_a_br_in_both_converters() {
    for html in ["First\n\r\n<br>Second", "First\n<script>x</script>\r\n<br>Second"] {
        assert_eq!(full(html, NewlineStyle::Spaces), "First\n\nSecond\n", "full: {html:?}");
        assert_eq!(fast_after_prescan(html), "First\n\nSecond\n", "fast: {html:?}");
    }
}

#[test]
fn should_look_past_a_comment_before_the_br() {
    assert_matches_inline_form("First\n<!-- c -->\n<br>\nSecond", "First<br>Second");
}

#[test]
fn should_match_the_inline_form_for_a_br_run_on_their_own_lines() {
    assert_matches_inline_form("A\n<br>\n<br>\nB", "A<br><br>B");
    assert_matches_inline_form("A\n<br>\n<br>\n<br>\nB", "A<br><br><br>B");
    assert_eq!(
        full("A\n<br>\n<br>\nB", NewlineStyle::Backslash),
        full("A<br><br>B", NewlineStyle::Backslash)
    );
}

#[test]
fn should_keep_a_br_after_text_that_ends_inside_a_span_as_a_hard_break() {
    assert_matches_inline_form("<span>First\n</span><br>Second", "<span>First</span><br>Second");
    assert_matches_inline_form(
        "<div><span>First\n</span>\n<br>\nSecond</div>",
        "<div><span>First</span><br>Second</div>",
    );
    assert_matches_inline_form(
        "<span><span>First\n</span>\n</span>\n<br>Second",
        "<span><span>First</span></span><br>Second",
    );
    assert_matches_inline_form(
        "<ul><li><span>a\n</span><br>b</li></ul>",
        "<ul><li><span>a</span><br>b</li></ul>",
    );
    assert_matches_inline_form(
        "<span>First\n<!-- c --></span >\n<br>Second",
        "<span>First</span><br>Second",
    );
}

#[test]
fn should_keep_a_br_after_text_that_ends_inside_other_elements_as_a_hard_break() {
    for tag in ["b", "i", "em", "small", "u", "del", "sub", "abbr", "cite", "time"] {
        assert_matches_inline_form(
            &format!("<{tag}>First\n</{tag}><br>Second"),
            &format!("<{tag}>First</{tag}><br>Second"),
        );
    }
    assert_matches_inline_form(
        "<a href=\"u\">First\n</a><br>Second",
        "<a href=\"u\">First</a><br>Second",
    );
    assert_matches_inline_form(
        "<b><span>First\n</span></b><br>Second",
        "<b><span>First</span></b><br>Second",
    );
    assert_matches_inline_form("<x-y>First\n</x-y><br>Second", "<x-y>First</x-y><br>Second");
    // ~keep The fast converter hands these two to the full converter.
    for tag in ["font", "mark"] {
        assert_eq!(
            full(&format!("<{tag}>First\n</{tag}><br>Second"), NewlineStyle::Spaces),
            full(&format!("<{tag}>First</{tag}><br>Second"), NewlineStyle::Spaces),
            "{tag}"
        );
    }
    assert_matches_inline_form(
        "<span><b>First\n</b></span><br>Second",
        "<span><b>First</b></span><br>Second",
    );
}

#[test]
fn should_keep_the_source_newline_when_no_br_ends_the_line() {
    for html in [
        "<span>First\n</span>Second",
        "<span>First\n</span>more<br>Second",
        "<div><span>First\n</span></div><br>Second",
        "<span>First\n</span><em>Second</em>",
        "<div><div>First\n</div><br>Second</div>",
        "<ul><li>First\n</li><br>Second</ul>",
        "First\n<b></b><br>Second",
        "<span>First\n</span><i></i><br>Second",
        "<label>First\n</label><br>Second",
        "<span>First\n</span><span><br>Second</span>",
    ] {
        assert_eq!(fast(html), full(html, NewlineStyle::Spaces), "{html:?}");
    }
    // ~keep The fast converter hands a select list to the full converter (issue 752).
    let select = "<select><option>First\n</option></select><br>Second";
    assert_eq!(auto(select), full(select, NewlineStyle::Spaces), "{select:?}");
    assert_eq!(full(select, NewlineStyle::Spaces), "First  \nSecond\n");
    assert_eq!(
        full("<span>First\n</span>Second", NewlineStyle::Spaces),
        "First\nSecond\n"
    );
    assert_eq!(
        full("<span>First\n</span><em>Second</em>", NewlineStyle::Spaces),
        "First\n*Second*\n"
    );
}

#[test]
fn should_give_the_same_break_in_both_converters_after_a_form_element() {
    for tag in ["progress", "meter", "output"] {
        let html = format!("<{tag}>First\n</{tag}><br>Second");
        assert_eq!(fast(&html), full(&html, NewlineStyle::Spaces), "{html:?}");
    }
}

#[test]
fn should_keep_what_a_closing_tag_writes_after_the_newline_before_a_br() {
    let html = "<abbr title=\"t\">First\n</abbr><br>Second";
    assert!(full(html, NewlineStyle::Spaces).contains("(t)"));
    let fast_output = fast(html);
    assert!(fast_output.contains("(t)"), "{fast_output:?}");
}

#[test]
fn should_keep_a_blank_source_line_before_a_br_as_a_paragraph_break() {
    let html = "<blockquote>a\n\n<br>b</blockquote>";
    assert!(full(html, NewlineStyle::Spaces).starts_with("> a\n>\n"));
    let fast_output = fast(html);
    assert!(fast_output.starts_with("> a\n>\n"), "{fast_output:?}");
}

#[test]
fn should_keep_the_hard_break_past_markup_the_full_converter_does_not_see() {
    for html in [
        "a\n<script>var x;</script><br>y",
        "a\n<style>p { color: red }</style><br>y",
        "<span>a\n</span foo><br>y",
        "<span>a\n</span><script>x</script>\n<br>y",
    ] {
        assert_eq!(full(html, NewlineStyle::Spaces), "a  \ny\n", "full converter: {html:?}");
        assert_eq!(fast(html), "a  \ny\n", "fast converter: {html:?}");
    }
}

#[test]
fn should_keep_a_blank_line_across_a_script_or_style_before_a_br_as_a_paragraph_break() {
    for html in [
        "First\n<script>x</script>\n<br>Second",
        "First\n<style>p { color: red }</style>\n<br>Second",
        "First\r\n<script>x</script>\n<br>Second",
        "First\n<script>x</script>\n \n<br>Second",
        "<div>First\n<style>x</style>\n<br>Second</div>",
        "<span>First\n<script>x</script>\n<br>Second</span>",
        "First\n<script>a</script>\n<style>b</style>\n<br>Second",
    ] {
        assert_eq!(
            full(html, NewlineStyle::Spaces),
            "First\n\nSecond\n",
            "full converter: {html:?}"
        );
        assert_eq!(fast(html), "First\n\nSecond\n", "fast converter: {html:?}");
        assert_eq!(auto(html), "First\n\nSecond\n", "auto route: {html:?}");
    }
    for html in [
        "First\n<script>x</script> \n<br>Second",
        "First\n<style>x</style>\t\n<br>Second",
    ] {
        assert_eq!(
            full(html, NewlineStyle::Spaces),
            "First  \nSecond\n",
            "full converter: {html:?}"
        );
        assert_eq!(fast(html), "First  \nSecond\n", "fast converter: {html:?}");
        assert_eq!(auto(html), "First  \nSecond\n", "auto route: {html:?}");
    }
}

/// ~keep `<canvas>` is inline in the fast converter's tag table and a block in the full
/// ~keep converter's block list. Its close must keep the join, and the `<br>` must remove it before
/// ~keep the fast converter separates inline content after a block, as the full converter does.
#[test]
fn should_keep_a_br_after_text_that_ends_inside_a_canvas_as_a_hard_break() {
    assert_matches_inline_form("<canvas>First\n</canvas><br>Second", "<canvas>First</canvas><br>Second");
    let html = "<canvas>First\n</canvas><br>Second";
    assert_eq!(full(html, NewlineStyle::Spaces), "First  \nSecond\n");
    assert_eq!(fast(html), "First  \nSecond\n");
}

/// ~keep Lines of markup between the text and the `<br>`: enough that a scan of that markup for
/// ~keep each text node in it takes seconds, while one scan takes a few milliseconds.
const HOSTILE_LINES: usize = 20_000;

/// ~keep One conversion against one plain pass over the same bytes, both timed in the same
/// ~keep process, so a loaded machine slows both. Measured at 20k lines: the linear scan takes 18
/// ~keep times the pass in a debug build and 14 times in release; the scan that looked ahead from
/// ~keep each text node took about 27,000 times the pass.
const MAX_TIME_RATIO: f64 = 200.0;

/// Text ending in a source newline, comments each followed by a form feed and a newline, then a
/// `<br>`: markup between the text and the `<br>` that the fast converter must not scan again for
/// each text node in it.
fn form_feed_runs() -> [(&'static str, String); 2] {
    let run = "<!--c-->\x0C\n".repeat(HOSTILE_LINES);
    [
        ("top level", format!("a\n{run}<br>b")),
        ("in a span", format!("<span>a\n{run}</span><br>b")),
    ]
}

/// The fastest of three runs of `work`.
fn fastest_of_three(mut work: impl FnMut()) -> Duration {
    (0..3)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .min()
        .expect("at least one run")
}

/// A linear pass over the bytes of `html` that copies each character, escaping the markup ones.
fn plain_pass(html: &str) -> String {
    let mut out = String::with_capacity(html.len() * 2);
    for character in std::hint::black_box(html).chars() {
        if matches!(character, '<' | '>' | '&') {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

#[test]
fn should_scan_the_markup_before_a_br_in_linear_time() {
    for (name, html) in form_feed_runs() {
        let conversion = fastest_of_three(|| {
            let output = fast(&html);
            assert!(output.ends_with("b\n"), "{output:?}");
        });
        let pass = fastest_of_three(|| {
            std::hint::black_box(plain_pass(&html));
        });
        let ratio = conversion.as_secs_f64() / pass.as_secs_f64().max(1e-9);
        assert!(
            ratio < MAX_TIME_RATIO,
            "{name}: the conversion took {ratio:.0} times as long as a plain pass over the same \
             bytes ({:.4}s vs {:.6}s), expected under {MAX_TIME_RATIO}",
            conversion.as_secs_f64(),
            pass.as_secs_f64()
        );
    }
}
