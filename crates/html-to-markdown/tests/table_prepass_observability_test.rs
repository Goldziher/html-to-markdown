//! Regression coverage for the *early-return* half of the "exactly one walk records" rule.
//!
//! `table_cell_metadata_duplication_test.rs` pins the case where `handle_table` renders the
//! table. This file pins what happens when it renders *nothing*: `4bef405a8` detached
//! `structure_collector` and `inline_collector` from the `TableGrid` walk in
//! `block/table/mod.rs::collect_grid_row`, and that walk is the *only* walk of a cell when
//! `handle_table` returns early. So a cell's contents now contribute zero collector records
//! there, where they previously contributed exactly one.
//!
//! It also pins the deliberate exception: `reference_collector` is *not* detached, because it
//! feeds the emitted `[n]: url` definitions and `get_or_insert` is idempotent per (url, title).

#![cfg(feature = "metadata")]

use html_to_markdown_rs::{ConversionOptions, ConversionResult, DocumentNode, LinkStyle, NodeContent, convert};

/// A table whose every cell is empty: `content_summary` sees no non-whitespace text, no `<a>`,
/// no `<th>`, no `<caption>` and no `<img>`, so `has_text == false` and `link_count == 0`.
/// That is the exact precondition for `handle_table`'s `is_blank_table && link_count == 0`
/// early return in `block/table/builder.rs`.
const BLANK_TABLE_WITH_NESTED_BLANK_TABLE_HTML: &str =
    "<table><tr><td><table><tr><td></td></tr></table></td></tr></table>";

// ~keep Only the `visitor`-gated module below consumes this, so every build without that
// ~keep feature -- including the default one -- warned it dead. Gate the allow rather than
// ~keep the constant, so the warning comes back if the last real user goes away.
#[cfg_attr(not(feature = "visitor"), allow(dead_code))]
const SINGLE_CELL_IMAGE_HTML: &str = r#"<table><tr><td><img src="photo.png" alt="a photo"></td></tr></table>"#;

fn options_with_metadata() -> ConversionOptions {
    ConversionOptions {
        extract_metadata: true,
        ..ConversionOptions::default()
    }
}

fn options_with_document_structure() -> ConversionOptions {
    ConversionOptions {
        include_document_structure: true,
        ..options_with_metadata()
    }
}

fn convert_to_result(html: &str, options: ConversionOptions) -> ConversionResult {
    convert(html, Some(options)).expect("conversion must succeed")
}

fn document_nodes(result: &ConversionResult) -> &[DocumentNode] {
    &result
        .document
        .as_ref()
        .expect("include_document_structure must populate result.document")
        .nodes
}

fn count_nodes(nodes: &[DocumentNode], predicate: fn(&NodeContent) -> bool) -> usize {
    nodes.iter().filter(|node| predicate(&node.content)).count()
}

fn content_of(result: &ConversionResult) -> &str {
    result.content.as_deref().expect("convert must populate result.content")
}

/// Every emitted reference definition line, in emission order.
fn reference_definitions(content: &str) -> Vec<&str> {
    content.lines().filter(|line| line.contains("]: ")).collect()
}

// ~keep A blank table is the one early return reachable without a visitor. Note which cell
// ~keep payloads can *not* be used to exercise it: `apply_tag_content` in `scanner.rs` sets
// ~keep `has_text` for any `<img>` carrying `src` or `alt`, and a `<pre><code>` block needs raw
// ~keep text — so neither an image nor a code block can ever live inside a blank table. A
// ~keep nested blank `<table>` remains a blank payload; an SVG now counts as content (#817).

#[test]
fn should_still_report_the_grid_when_a_blank_table_renders_to_nothing() {
    let result = convert_to_result(
        BLANK_TABLE_WITH_NESTED_BLANK_TABLE_HTML,
        options_with_document_structure(),
    );

    assert_eq!(
        content_of(&result).trim(),
        "",
        "a blank table with no links must emit no markdown at all"
    );
    assert_eq!(
        result.tables.len(),
        1,
        "push_table_data runs in handle_table_with_context *after* handle_table returned early, \
         so the outer grid is still reported"
    );
    assert_eq!(result.tables[0].grid.rows, 1);
    assert_eq!(result.tables[0].grid.cols, 1);
    assert_eq!(result.tables[0].grid.cells.len(), 1);
    assert_eq!(
        result.tables[0].markdown, "",
        "the markdown field mirrors the emitted output, which is empty"
    );
}

// ~keep The counterpart of `should_record_a_nested_table_in_document_structure_exactly_once`,
// ~keep which asserts 2 for a *rendering* outer table. Here the answer is 1, not 2, and the
// ~keep difference is the whole behaviour change: the outer table returns early, so the render
// ~keep pass never reaches the nested table, and the TableGrid walk — the only walk left — now
// ~keep runs with `structure_collector` detached. Before `4bef405a8` that walk still held the
// ~keep collector and pushed the nested table, giving 2. The nested table's absence mirrors the
// ~keep emitted output, which contains neither table.
#[test]
fn should_not_record_a_nested_table_when_the_outer_blank_table_renders_to_nothing() {
    let result = convert_to_result(
        BLANK_TABLE_WITH_NESTED_BLANK_TABLE_HTML,
        options_with_document_structure(),
    );
    let nodes = document_nodes(&result);

    assert_eq!(
        count_nodes(nodes, |content| matches!(content, NodeContent::Table { .. })),
        1,
        "only the outer table, pushed by handle_table_with_context; the nested table is reached \
         solely by the TableGrid walk, whose structure_collector is detached"
    );
    assert_eq!(
        nodes.len(),
        1,
        "the outer table node is the document's only node, got {nodes:?}"
    );
    assert_eq!(
        result.tables.len(),
        1,
        "result.tables and the Table nodes are filled by the same push_table_data call"
    );
}

// ~keep reference_collector is deliberately NOT detached from the internal walks (unlike the
// ~keep metadata/structure/inline collectors) because it feeds emitted bytes: the render pass
// ~keep must be able to resolve `[label][n]`. Sharing it is safe only because `get_or_insert`
// ~keep is idempotent per (url, title). These two tests are what makes a future "consistency"
// ~keep cleanup that detaches it — or a change that makes insertion non-idempotent — visible as
// ~keep duplicated or renumbered definitions in the output.

#[test]
fn should_emit_one_reference_definition_when_a_table_cell_link_is_walked_repeatedly() {
    let html = r#"<table><tr><td><a href="https://example.com">L</a></td></tr></table>"#;
    let result = convert_to_result(
        html,
        ConversionOptions {
            link_style: LinkStyle::Reference,
            ..options_with_document_structure()
        },
    );
    let content = content_of(&result);

    assert_eq!(
        reference_definitions(content),
        vec!["[1]: https://example.com"],
        "the width pre-pass, the render pass and the TableGrid walk each call get_or_insert for \
         the same (url, title); it is idempotent, so exactly one definition is emitted. Full \
         output was:\n{content}"
    );
    assert!(
        content.contains("[L][1]"),
        "the cell must reference the single definition, got:\n{content}"
    );
}

#[test]
fn should_number_table_cell_references_by_distinct_url_not_by_walk_count() {
    let html = concat!(
        "<table><tr>",
        r#"<td><a href="https://a.example">A</a></td>"#,
        r#"<td><a href="https://b.example">B</a></td>"#,
        "</tr></table>"
    );
    let result = convert_to_result(
        html,
        ConversionOptions {
            link_style: LinkStyle::Reference,
            ..options_with_document_structure()
        },
    );
    let content = content_of(&result);

    assert_eq!(
        reference_definitions(content),
        vec!["[1]: https://a.example", "[2]: https://b.example"],
        "three walks over two cells must still number the references 1 and 2, in cell order. \
         Full output was:\n{content}"
    );
    assert!(
        content.contains("[A][1]") && content.contains("[B][2]"),
        "each cell must carry the number its own url was assigned, got:\n{content}"
    );
}

#[cfg(feature = "visitor")]
mod visitor_early_returns {
    use super::{
        ConversionOptions, ConversionResult, NodeContent, SINGLE_CELL_IMAGE_HTML, content_of, convert_to_result,
        count_nodes, document_nodes, options_with_document_structure,
    };

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use html_to_markdown_rs::NodeContext;
    use html_to_markdown_rs::visitor::{HtmlVisitor, VisitResult};

    /// Visitor that answers `visit_table_start` with a fixed result and counts how often it was
    /// asked, so a test can prove the early-return branch actually ran.
    #[derive(Debug)]
    struct TableStartVisitor {
        result: VisitResult,
        calls: Arc<AtomicUsize>,
    }

    impl HtmlVisitor for TableStartVisitor {
        fn visit_table_start(&mut self, _ctx: &NodeContext) -> VisitResult {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result.clone()
        }
    }

    fn convert_with_table_start_result(html: &str, result: VisitResult) -> (ConversionResult, usize) {
        let calls = Arc::new(AtomicUsize::new(0));
        let visitor = TableStartVisitor {
            result,
            calls: Arc::clone(&calls),
        };
        let conversion = convert_to_result(
            html,
            ConversionOptions {
                visitor: Some(Arc::new(Mutex::new(visitor))),
                ..options_with_document_structure()
            },
        );
        let observed = calls.load(Ordering::SeqCst);
        (conversion, observed)
    }

    // ~keep `VisitResult::Skip` from `visit_table_start` returns from `handle_table` before any
    // ~keep row is rendered, so the TableGrid walk in handle_table_with_context is the only walk
    // ~keep that ever reaches the `<img>`. Since `4bef405a8` that walk has structure_collector
    // ~keep detached, so the image contributes zero document nodes — which is correct precisely
    // ~keep because the image contributes zero emitted bytes too. Before the fix it contributed
    // ~keep one node describing content that was nowhere in the output.
    #[test]
    fn should_record_no_image_node_when_a_visitor_skips_the_table() {
        let (result, table_start_calls) = convert_with_table_start_result(SINGLE_CELL_IMAGE_HTML, VisitResult::Skip);

        assert_eq!(
            table_start_calls, 1,
            "the Skip branch must actually have been taken, otherwise this test proves nothing"
        );
        assert_eq!(content_of(&result).trim(), "", "a skipped table emits nothing at all");

        let nodes = document_nodes(&result);
        assert_eq!(
            count_nodes(nodes, |content| matches!(content, NodeContent::Image { .. })),
            0,
            "the image is never rendered, so it must never be recorded either"
        );
        assert_eq!(
            count_nodes(nodes, |content| matches!(content, NodeContent::Table { .. })),
            1,
            "push_table_data still fires after the early return, so the grid is still reported"
        );
        assert_eq!(nodes.len(), 1, "the table node is the document's only node");

        assert_eq!(
            result.metadata.images.len(),
            0,
            "the metadata collector must agree with the structure collector"
        );

        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0].grid.rows, 1);
        assert_eq!(result.tables[0].grid.cols, 1);
        assert_eq!(result.tables[0].grid.cells.len(), 1);
        assert_eq!(
            result.tables[0].grid.cells[0].content, "![a photo](photo.png)",
            "the grid keeps the cell text it built locally; only the collectors were detached"
        );
        assert_eq!(
            result.tables[0].markdown, "",
            "nothing was rendered, so the table's markdown is empty"
        );
    }

    // ~keep `VisitResult::PreserveHtml` is the second reachable early return. The image *is*
    // ~keep present in the output, but only as raw serialized HTML that the converter never
    // ~keep walked — so it still contributes zero document nodes. This is the shape most likely
    // ~keep to tempt a future change into re-attaching the collector "so the structure matches
    // ~keep the output"; the assertion records that the current contract is markdown-walk-based,
    // ~keep not HTML-based.
    #[test]
    fn should_record_no_image_node_when_a_visitor_preserves_the_table_html() {
        let (result, table_start_calls) =
            convert_with_table_start_result(SINGLE_CELL_IMAGE_HTML, VisitResult::PreserveHtml);

        assert_eq!(
            table_start_calls, 1,
            "the PreserveHtml branch must actually have been taken"
        );
        let content = content_of(&result);
        assert!(
            content.contains("<table") && content.contains("<img"),
            "PreserveHtml must emit the serialized table, got:\n{content}"
        );

        let nodes = document_nodes(&result);
        assert_eq!(
            count_nodes(nodes, |content| matches!(content, NodeContent::Image { .. })),
            0,
            "preserved HTML is not walked, so nothing inside it is recorded"
        );
        assert_eq!(
            count_nodes(nodes, |content| matches!(content, NodeContent::Table { .. })),
            1,
            "push_table_data still fires, so the grid is still reported"
        );
        assert_eq!(nodes.len(), 1, "the table node is the document's only node");
        assert_eq!(result.tables.len(), 1);
        assert_eq!(
            result.metadata.images.len(),
            0,
            "the metadata collector must agree with the structure collector"
        );
    }
}

// ~keep SVG-only tables render their graphic (#817); the width and grid passes must not
// ~keep duplicate the inline-image record emitted by the actual cell walk.
#[cfg(feature = "inline-images")]
#[test]
fn should_collect_inline_svg_once_when_a_media_only_table_renders() {
    let html = concat!(
        r#"<table><tr><td><svg xmlns="http://www.w3.org/2000/svg" aria-label="glyph">"#,
        r#"<rect width="1" height="1"></rect></svg></td></tr></table>"#
    );
    let result = convert_to_result(
        html,
        ConversionOptions {
            extract_images: true,
            capture_svg: true,
            compact_tables: true,
            ..options_with_document_structure()
        },
    );

    let image = concat!(
        "![SVG Image](data:image/svg+xml;base64,",
        "PHN2ZyBhcmlhLWxhYmVsPSJnbHlwaCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4",
        "8cmVjdCBoZWlnaHQ9IjEiIHdpZHRoPSIxIiAvPjwvc3ZnPg==)"
    );
    assert_eq!(content_of(&result), format!("| {image} |\n| --- |\n"));
    assert_eq!(result.images.len(), 1, "only the render pass records the SVG");
    assert_eq!(result.images[0].format, html_to_markdown_rs::InlineImageFormat::Svg);
    assert_eq!(
        result.images[0].source,
        html_to_markdown_rs::InlineImageSource::SvgElement
    );
    assert_eq!(result.tables.len(), 1);
    assert_eq!(result.tables[0].grid.rows, 1);
    assert_eq!(result.tables[0].grid.cols, 1);
}
