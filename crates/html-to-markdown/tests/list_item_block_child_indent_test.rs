// ~keep Rust inner attributes below are crate-level attributes, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests: a block child of a list item that is not a paragraph must have every
//! one of its own physical output lines indented to the item's continuation width.
//!
//! Before this fix, `add_list_continuation_indent` (`block/paragraph.rs`) was the only call
//! site that indented list-item continuation content, and it only indented a single leading
//! position. A fenced code block, a blockquote, or trailing sibling text after a heading
//! spans (or starts) a physical line of its own with no indent, so it fell out of the list
//! item entirely when the emitted Markdown was re-parsed (`CommonMark` spec examples 263, 273,
//! 274, 300, 318, 324): `<ol><li></li></ol><pre>...` instead of the code block staying nested
//! inside the `<li>`.

fn convert(html: &str) -> String {
    html_to_markdown_rs::convert(html, None)
        .unwrap()
        .content
        .unwrap_or_default()
}

/// A fenced code block as the sole child of an ordered list item: every line of the fence
/// (including the closing fence) must be indented to the marker's own width ("1. " = 3).
#[test]
fn should_indent_every_fence_line_when_code_block_is_only_child_of_list_item() {
    let html = "<ol><li><pre><code>foo\n</code></pre></li></ol>";
    let result = convert(html);
    assert_eq!(result, "1. ```\n   foo\n   ```\n", "actual: {result:?}");
}

/// A fenced code block as a *continuation* block (after a paragraph) inside a list item:
/// the blank-line separator before the fence, every fence line, and the following paragraph
/// must all carry the continuation indent (`CommonMark` spec example 324).
#[test]
fn should_indent_fence_and_following_paragraph_when_code_block_continues_a_list_item() {
    let html = "<ol><li><pre><code>foo\n</code></pre><p>bar</p></li></ol>";
    let result = convert(html);
    assert_eq!(result, "1. ```\n   foo\n   ```\n\n   bar\n", "actual: {result:?}");
}

/// A code block containing a blank line (`CommonMark` spec example 318): the blank line stays
/// unindented (blank lines match any container depth), but the non-blank lines around it and
/// the closing fence still carry the indent.
///
/// ~keep The middle item's bare `<pre>` (no `<p>`) still needs a blank-line separator from
/// ~keep its neighbors to keep item boundaries unambiguous, and once that blank line exists
/// ~keep ANYWHERE between two items, a CommonMark-compliant reparse concludes the whole list
/// ~keep is loose and wraps every item's content in `<p>` -- including the plain-text "a" and
/// ~keep "c" that had none originally. The previous expectation here (no blank line before the
/// ~keep first fenced code item) was not itself a fixpoint: rendering it back to HTML and
/// ~keep reconverting produced exactly this fully-loose form instead of reproducing itself. This is the
/// ~keep `commonmark_spec_fixpoint` oracle's own verified-stable form for example 318.
/// ~keep
/// ~keep The blank line is between two lines of code. Blank lines at the end of the code are
/// ~keep dropped in a list item as they are outside of one (issue #769).
#[test]
fn should_leave_blank_interior_lines_unindented_but_indent_surrounding_fence_lines() {
    let html = "<ul><li>a</li><li><pre><code>b\n\nd\n</code></pre></li><li>c</li></ul>";
    let result = convert(html);
    assert_eq!(result, "- a\n\n- ```\n  b\n\n  d\n  ```\n\n- c\n", "actual: {result:?}");
}

/// The blank lines at the end of the code of spec example 318 are not lines of the block.
#[test]
fn should_drop_blank_lines_at_the_end_of_a_code_block_in_a_list_item() {
    let html = "<ul><li>a</li><li><pre><code>b\n\n\n</code></pre></li><li>c</li></ul>";
    let result = convert(html);
    assert_eq!(result, "- a\n\n- ```\n  b\n  ```\n\n- c\n", "actual: {result:?}");
}

/// A heading followed by trailing sibling text directly inside the same `<li>` (`CommonMark`
/// spec example 300): the heading's own single trailing newline leaves the next physical
/// line unindented unless the text node itself adds the continuation indent.
#[test]
fn should_indent_trailing_text_after_a_heading_inside_a_list_item() {
    let html = "<ul><li><h2>Bar</h2>baz</li></ul>";
    let result = convert(html);
    assert_eq!(result, "- ## Bar\n  baz\n", "actual: {result:?}");
}

/// A blockquote as a continuation block inside a list item (`CommonMark` spec example 263):
/// every quoted line needs the item's indent in addition to its own "> " prefix.
#[test]
fn should_indent_every_quoted_line_when_blockquote_continues_a_list_item() {
    let html = "<ol><li><p>foo</p><blockquote><p>bam</p></blockquote></li></ol>";
    let result = convert(html);
    assert_eq!(result, "1. foo\n   > bam\n", "actual: {result:?}");
}

/// A code block nested two list levels deep must accumulate both ancestors' marker widths
/// (`list_indent_columns`), not a fixed per-depth constant.
#[test]
fn should_accumulate_ancestor_marker_widths_for_a_nested_list_code_block() {
    let html = "<ul><li>a<ul><li><pre><code>x\n</code></pre></li></ul></li></ul>";
    let result = convert(html);
    assert_eq!(result, "- a\n  * ```\n    x\n    ```\n", "actual: {result:?}");
}
