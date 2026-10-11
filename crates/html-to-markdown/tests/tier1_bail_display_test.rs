//! Characterization of every `BailReason`'s `Display` string.
//!
//! Bail reasons reach users through `tracing` output, so their wording is an observable surface,
//! not an internal detail. This test exists to let `bail.rs` be restructured for the size limits in
//! issue #465 without changing a single rendered message. ~keep

#![allow(missing_docs)]
#![cfg(feature = "testkit")]

use html_to_markdown_rs::tier1::BailReason;

fn all_variants() -> Vec<BailReason> {
    vec![
        BailReason::Classifier,
        BailReason::DepthMismatch {
            tag: "div".to_owned(),
            expected: 2,
            actual: 1,
        },
        BailReason::EofWithOpenBlock { open_count: 3 },
        BailReason::LiteralLt { offset: 11 },
        BailReason::Cdata { offset: 12 },
        BailReason::UnknownCustomElement {
            name: "x-foo".into(),
            offset: 13,
        },
        BailReason::AdjacentRawTextTags { offset: 14 },
        BailReason::TableRowspanColspan,
        BailReason::TableBlockChildInCell,
        BailReason::TableNestedTable,
        BailReason::TableNestedTableInSingleCellRow,
        BailReason::TableCaption,
        BailReason::TableSectionOrder,
        BailReason::DepthLimitExceeded {
            depth: 65,
            max_depth: 64,
        },
        BailReason::UnknownEntity {
            name: "copy".into(),
            offset: 15,
        },
        BailReason::HiddenElement { offset: 16 },
        BailReason::ListNestedOrdered,
        BailReason::OrderedListAfterOrderedList,
        BailReason::EmptyNestedListItem,
        BailReason::ListItemUnsupportedBlockChild,
        BailReason::ListItemCheckbox,
        BailReason::ImageLazyLoadSrc,
        BailReason::LinkAutolinkNestedMarkup,
        BailReason::LinkEmptyLabel,
        BailReason::ImageMissingSource,
        BailReason::AdjacentInlineEmphasis,
        BailReason::WhitespaceOnlyInlineEmphasis,
        BailReason::InlineMarkerNotReproduced,
        BailReason::RuleBetweenInlineMarkers,
    ]
}

#[test]
fn should_render_every_bail_reason_with_its_documented_message() {
    let rendered: Vec<String> = all_variants().iter().map(ToString::to_string).collect();
    let expected = [
        "classifier forced tier-2",
        "depth mismatch for </div>: expected 2 open(s), got 1",
        "EOF with 3 unclosed block element(s)",
        "literal '<' at byte offset 11",
        "CDATA section at byte offset 12",
        "unknown custom element <x-foo> at byte offset 13",
        "adjacent <script>/<style> tags with no separating whitespace at byte offset 14",
        "table cell has rowspan or colspan != 1",
        "block-level element inside table cell",
        "nested <table> inside a table cell",
        "nested <table> inside a data table's single-cell row",
        "<caption> element in table",
        "table sections in unsupported order",
        "open-tag nesting depth 65 reached the effective limit of 64",
        "HTML entity &copy is missing its closing semicolon at byte offset 15",
        "hidden element (hidden attribute or style) at byte offset 16",
        "nested list with an ordered ancestor or ordered self (cumulative indent width)",
        "ordered list right after an ordered list (switched delimiter)",
        "nested list item with nothing on its marker line",
        "block-level child of a list item in a shape this scanner cannot render correctly",
        "checkbox input inside a list item (a task item)",
        "<img> has a lazy-load placeholder src and a fallback src attribute",
        "autolink-eligible <a> href had a nested tag inside the label before close",
        "empty link label requires DOM-based fallback",
        "image has no effective source",
        "adjacent strong/emphasis elements would form one delimiter run",
        "strong/emphasis element with a whitespace-only body",
        "inline element whose tier-2 markers tier-1 does not emit",
        "horizontal rule between inline markers",
    ];
    // ~keep Length first: zipping two iterators of different lengths silently compares only the
    // shorter prefix, so a truncated expectation would "pass" while checking almost nothing.
    assert_eq!(
        rendered.len(),
        expected.len(),
        "a BailReason variant was added or removed without updating this characterization"
    );
    assert_eq!(rendered, expected, "a bail reason message changed");
}

/// #586: the message says which reference reason applies. `&#39` and `&copy` are references
/// Tier-2's decoder knows, so the message says they are missing their `;`, not that they are
/// unknown. `&bogusentityname` is not a reference at all, so the message keeps that wording.
#[test]
fn unknown_entity_message_distinguishes_a_known_reference_from_a_genuinely_unknown_one() {
    let known_numeric = BailReason::UnknownEntity {
        name: "#39".into(),
        offset: 3,
    };
    assert_eq!(
        known_numeric.to_string(),
        "HTML entity &#39 is missing its closing semicolon at byte offset 3"
    );

    let known_named = BailReason::UnknownEntity {
        name: "copy".into(),
        offset: 7,
    };
    assert_eq!(
        known_named.to_string(),
        "HTML entity &copy is missing its closing semicolon at byte offset 7"
    );

    let unknown = BailReason::UnknownEntity {
        name: "bogusentityname".into(),
        offset: 9,
    };
    assert_eq!(
        unknown.to_string(),
        "unknown HTML entity &bogusentityname at byte offset 9"
    );
}
