//! Tests of the removal of a kept element that the caller excludes.

use super::unwrap_kept_inert_elements;
use crate::options::HiddenContent;

/// Where the start tags that carry the attribute `x` start in `page`.
fn starts_marked_x(page: &str) -> Vec<usize> {
    page.match_indices('<')
        .map(|(start, _)| start)
        .filter(|&start| {
            ["<template x", "<noscript x"]
                .iter()
                .any(|tag| page[start..].starts_with(tag))
        })
        .collect()
}

#[test]
fn should_remove_whole_a_kept_element_that_the_caller_excludes() {
    let (reachable, all) = (HiddenContent::Reachable, HiddenContent::All);
    for (page, choice, expected) in [
        ("<p>a</p><template x><p>b</p></template>c", all, "<p>a</p>c"),
        ("<p>a</p><noscript x><p>b</p></noscript>c", all, "<p>a</p>c"),
        // ~keep The end tag of an element in the excluded one closes that element only.
        (
            "<p>a</p><template x>b<template>c</template>d</template>e",
            all,
            "<p>a</p>e",
        ),
        (
            "<p>a</p><noscript x>b<template>c</template>d</noscript>e",
            all,
            "<p>a</p>e",
        ),
        // ~keep An excluded element in a kept one, and a kept one after an excluded one.
        (
            "<p>a</p><template>b<noscript x>c</noscript>d</template>e",
            all,
            "<p>a</p>bde",
        ),
        (
            "<p>a</p><template x>b</template><template>c</template>d",
            all,
            "<p>a</p>cd",
        ),
        // ~keep An omitted end tag in the excluded element, and a start tag written as self-closing.
        ("<p>a</p><template x><ul><li>b<li>c</ul></template>d", all, "<p>a</p>d"),
        ("<p>a</p><template x/>b</template>c", all, "<p>a</p>c"),
        // ~keep With no end tag the excluded element holds the rest of the page.
        ("<p>a</p><noscript x>b<p>c", all, "<p>a</p>"),
        ("<p>a</p><template>b</template><template x>c<p>d", all, "<p>a</p>b"),
        (
            r#"<div><template x shadowrootmode="open">b</template>c</div>"#,
            reachable,
            "<div>c</div>",
        ),
        // ~keep An excluded element whose tags stay is left as written.
        (
            "<p>a</p><template x>b</template><noscript x>c</noscript>d",
            reachable,
            "<p>a</p><template x>b</template><noscript x>c</noscript>d",
        ),
        (
            "<head><noscript x>a</noscript></head><p>b</p>",
            all,
            "<head><noscript x>a</noscript></head><p>b</p>",
        ),
        // ~keep A tag in the text of a text element is text.
        (
            "<p>a</p><textarea><template x>b</template></textarea>c",
            all,
            "<p>a</p><textarea><template x>b</template></textarea>c",
        ),
    ] {
        let excluded = starts_marked_x(page);
        assert!(!excluded.is_empty(), "{page:?}");
        assert_eq!(
            unwrap_kept_inert_elements(page, choice, &excluded),
            expected,
            "{page:?}"
        );
        assert_eq!(
            unwrap_kept_inert_elements(page, HiddenContent::Drop, &excluded),
            page,
            "{page:?}"
        );
    }
}

#[test]
fn should_write_an_end_tag_of_the_body_for_the_excluded_element_that_started_the_body() {
    let page = "<template x><p>a</p></template><title>t</title>";
    assert_eq!(
        unwrap_kept_inert_elements(page, HiddenContent::All, &starts_marked_x(page)),
        "</body><title>t</title>"
    );
}
