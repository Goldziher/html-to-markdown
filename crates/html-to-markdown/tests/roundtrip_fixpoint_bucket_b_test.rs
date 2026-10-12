// ~keep The inner attribute below is a crate-level Rust attribute, not a shell shebang.
#![allow(missing_docs)]

//! Regression tests for three of the six real-world round-trip "Bucket B" fixtures in
//! `tests/roundtrip_fixpoint.rs` (`KNOWN_DIVERGENCES`), each minimized in isolation:
//!
//! - **Text-spacing collapse near a closing `**`** (`mdream/wikipedia-small.html`):
//!   `text_node.rs`'s whitespace-only-text-node branch collapsed a run of more than one
//!   character to a single space unconditionally, without first checking whether `output`
//!   already ended with a space. A single-character whitespace-only node right next to it
//!   hit a *different* sibling branch that also pushed unconditionally. Two independently
//!   correct single spaces (one a preprocessing-inserted replacement for a removed
//!   `<style>`/`<script>` element, one the real inter-element gap) then stacked into a
//!   literal double space that only survived until the first Markdown -> HTML -> Markdown
//!   hop, when a compliant reparse folded them back into one — a real fixpoint bug, not
//!   file content.
//! - **Table row/cell-count shrinkage on reparse** (`real-world/wikipedia/small_html.html`,
//!   `large_rust.html`, `medium_python.html`): a `<table>` nested inside another table's
//!   cell (the common Wikipedia navbox shape) had its own row/separator markdown rendered
//!   straight into the outer cell's text by `render_cell_text` (`block/table/cell.rs`),
//!   bypassing `text_node.rs`'s per-text-node pipe escaping entirely. Once the newline-to-
//!   space fold flattened the inner table onto the outer cell's single line, its unescaped
//!   `|` delimiters were read as *the outer row's* cell boundaries on reparse — silently
//!   widening the row, and, worse, causing a GFM table parser to truncate a row to the
//!   header's column count on a subsequent parse: genuine content loss, not a cosmetic
//!   diff.
//! - **nbsp-run collapse between an `<img>` and an `<a>`**
//!   (`real-world/wikipedia/tables_countries.html`): the same whitespace-only-node branch
//!   above also collapsed a run of decoded `&nbsp;` characters to one plain ASCII space
//!   whenever it sat directly between two inline siblings, because `str::trim`'s
//!   Unicode-aware definition (used to decide "is this text node empty") treats U+00A0 as
//!   insignificant, exactly the class of bug `main_helpers::is_ascii_whitespace_only` (see
//!   its own doc comment) exists to avoid — but this particular collapse path had not yet
//!   been updated to use it. A run of nbsp characters that a page uses for deliberate
//!   visual spacing (e.g. between a flag icon and a country name) must survive verbatim.

use html_to_markdown_rs::{ConversionOptions, convert};

fn convert_default(html: &str) -> String {
    convert(html, Some(ConversionOptions::default()))
        .expect("conversion should succeed")
        .content
        .unwrap_or_default()
}

fn render_markdown_to_html(md: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.render.r#unsafe = true;
    comrak::markdown_to_html(md, &options)
}

/// Round-trips `html` through `convert -> render -> convert` and asserts the two Markdown
/// passes are byte-identical, mirroring `roundtrip_fixpoint.rs`'s own oracle.
fn assert_roundtrip_fixpoint(html: &str) -> String {
    let md1 = convert_default(html);
    let html2 = render_markdown_to_html(&md1);
    let md2 = convert_default(&html2);
    assert_eq!(
        md1, md2,
        "round-trip fixpoint failed for {html:?}: md1={md1:?} md2={md2:?}"
    );
    md1
}

// ---------------------------------------------------------------------------
// Cause 1: text-spacing collapse near a closing `**`
// ---------------------------------------------------------------------------

/// Minimized from `mdream/wikipedia-small.html`'s citation backlink (`cite_note-1`): a
/// `<style>` element removed by preprocessing leaves behind a single synthetic space
/// (neither side of the removed tag had whitespace of its own), landing right next to a
/// real single space from an actual inter-element text node one level up. Before the fix,
/// `md1` carried the double space (`"**  Tobin"`) and only `md2` collapsed it, so `md1 !=
/// md2`; the reparse should never observe anything `md1` did not already settle on.
#[test]
fn nested_style_removal_adjacent_to_bold_link_does_not_double_space() {
    let html = concat!(
        r##"<ol><li id="cite_note-1"><span class="mw-cite-backlink">"##,
        r##"<b><a href="#cite_ref-1">^</a></b></span> "##,
        r##"<span class="reference-text">"##,
        r##"<style data-mw-deduplicate="TemplateStyles:r1">.x{color:red}</style>"##,
        r##"<cite id="CITEREFTobin">Tobin, Allan J.</cite></span></li></ol>"##,
    );

    let md1 = assert_roundtrip_fixpoint(html);
    assert_eq!(md1, "1. **[↑](#cite_ref-1)** Tobin, Allan J.\n");
}

// ---------------------------------------------------------------------------
// Cause 2: table row/cell-count shrinkage from an unescaped nested-table flatten
// ---------------------------------------------------------------------------

/// Minimized from the Wikipedia navbox shape shared by `small_html.html`, `large_rust.html`,
/// and `medium_python.html`: a `<table>` with a `<th>` header (so it is never classified as
/// a layout table, regardless of nested-table count) holds a second `<table>` inside one
/// data cell. Before the fix, `render_cell_text` flattened the inner table's own `|`/`-`
/// syntax into the outer cell unescaped, so a compliant GFM reparse read those bytes as
/// *additional* cell boundaries in the outer row and truncated it to the header's column
/// count -- silently dropping "Inner 1", "Inner 2", "Inner 3", and "Inner 4" outright. Now
/// every unescaped pipe from the flattened inner table is backslash-escaped, so the row's
/// column count -- and every word of content -- survives the reparse; the inner table
/// itself has no representation that survives a compliant reparse (GFM table cells cannot
/// hold a nested table), so it is preserved as escaped, inert text instead of restructured.
///
/// This does not assert the stricter `md1 == md2` fixpoint: a separate, pre-existing,
/// unrelated divergence (issue #406's width-measurement pre-pass skips nested-table
/// rendering to stay linear, so it pads a column narrower than the full render does)
/// still changes the separator row's padding width between passes for this exact shape.
/// That is a cosmetic width difference, not content loss, and is out of scope here.
#[test]
fn nested_table_in_header_table_cell_does_not_lose_content_on_reparse() {
    let html = concat!(
        "<table><tr><th>Head1</th><th>Head2</th></tr>",
        "<tr><td>Outer A</td><td><table>",
        "<tr><td>Inner 1</td><td>Inner 2</td></tr>",
        "<tr><td>Inner 3</td><td>Inner 4</td></tr>",
        "</table></td></tr></table>",
    );

    let md1 = convert_default(html);
    let html2 = render_markdown_to_html(&md1);
    let md2 = convert_default(&html2);

    for md in [&md1, &md2] {
        for word in ["Outer A", "Inner 1", "Inner 2", "Inner 3", "Inner 4"] {
            assert!(md.contains(word), "expected {word:?} to survive in {md:?}");
        }
        // ~keep Flattened inner cells omit table syntax so a reparse cannot mistake it
        // ~keep for additional outer-row cell boundaries (#760).
        assert!(
            md.contains("Inner 1 Inner 2 Inner 3 Inner 4"),
            "expected the inner cells to remain as text in {md:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Cause 3: nbsp-run collapse between an `<img>` and an `<a>`
// ---------------------------------------------------------------------------

/// Minimized from `tables_countries.html`'s flag-icon-and-country-name cells (e.g. Nepal,
/// Switzerland): a run of three `&#160;` (nbsp) characters sits directly between an
/// `<img>` and an `<a>` inside a table cell. Before the fix this hit the same
/// whitespace-only-text-node collapse as Cause 1's ASCII-whitespace case, discarding the
/// nbsp run down to a single plain space on the very first conversion (not just on
/// reparse) -- real, first-pass content loss for a run a page uses as deliberate visual
/// spacing.
#[test]
fn nbsp_run_between_image_and_link_in_table_cell_survives() {
    let html =
        "<table><tr><td><img src=\"flag.png\" alt=\"\">\u{a0}\u{a0}\u{a0}<a href=\"/x\">Nepal</a></td></tr></table>";

    let md1 = assert_roundtrip_fixpoint(html);
    assert!(
        md1.contains("![](flag.png)\u{a0}\u{a0}\u{a0}[Nepal](/x)"),
        "expected the nbsp run to survive verbatim between image and link in {md1:?}"
    );
}
