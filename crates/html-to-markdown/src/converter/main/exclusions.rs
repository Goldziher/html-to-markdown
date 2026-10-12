//! The removal of the kept `<template>` and `<noscript>` elements that the caller excludes.

use std::borrow::Cow;

use super::parse::parse_html;
use crate::converter::utility::preprocessing::unwrap_kept_inert_elements;
use crate::error::Result;
use crate::options::HiddenContent;

/// `html` with the tags of the `<template>` and `<noscript>` elements that `hidden_content` keeps
/// removed, and with each such element that a selector of `selectors` matches removed whole.
pub(super) fn remove_kept_tags<'a>(
    html: &'a str,
    hidden_content: HiddenContent,
    selectors: &[String],
) -> Result<Cow<'a, str>> {
    let excluded = excluded_starts(html, hidden_content, selectors)?;
    Ok(unwrap_kept_inert_elements(html, hidden_content, &excluded))
}

/// Where each start tag that a selector of `selectors` matches starts in `html`, in the order
/// of the page. [`HiddenContent::Drop`] keeps no `<template>` and no `<noscript>`, so it asks
/// for none.
///
/// ~keep The pass that removes the tags of the kept `<template>` and `<noscript>` elements takes
/// ~keep these starts and removes such an element whole: with its tags removed, the element is
/// ~keep no element of the tree, and the exclusion that reads the tree finds nothing to remove.
/// ~keep Every other element that a selector matches is an element of the tree under each choice
/// ~keep of `hidden_content`, and only the exclusion that reads the tree removes it.
///
/// ~keep Only the start of a match is read. `tl` gives the end of an element only when its end
/// ~keep tag arrives while it is the innermost open element: an element with no end tag, or
/// ~keep with an omitted end tag in its content, ends at its start tag there.
fn excluded_starts(html: &str, hidden_content: HiddenContent, selectors: &[String]) -> Result<Vec<usize>> {
    if hidden_content == HiddenContent::Drop || selectors.is_empty() {
        return Ok(Vec::new());
    }
    let dom = parse_html(html)?;
    let parser = dom.parser();
    let mut starts = Vec::new();
    for selector in selectors {
        if let Some(matches) = dom.query_selector(selector) {
            starts
                .extend(matches.filter_map(|handle| handle.get(parser)?.as_tag().map(|tag| tag.boundaries(parser).0)));
        }
    }
    starts.sort_unstable();
    Ok(starts)
}
