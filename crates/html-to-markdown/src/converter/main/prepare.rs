//! The passes over a page before a parse reads it.

use std::borrow::Cow;

use super::exclusions;
use crate::converter::main_helpers::{has_custom_element_tags, preserves_menu, repair_with_html5ever_bounded};
#[cfg(feature = "metadata")]
use crate::converter::utility::preprocessing::strip_script_and_style_tags_for_second_pass;
use crate::converter::utility::preprocessing::{
    KeptScripts, normalize_bogus_comment_endings, normalize_menu_elements, normalize_split_closing_tags,
    normalize_unclosed_list_items, owner_mark_numbers, preprocess_html, strip_bogus_comments, strip_hidden_elements,
    strip_html_cdata, strip_script_and_style_tags, strip_script_and_style_tags_with_kept_scripts,
};
use crate::error::Result;
use crate::options::{ConversionOptions, HiddenContent};

/// What [`prepare_html`] gives: the page that the first parse reads, the limit that the repair
/// of custom elements reached, the page as written when that repair did not run (the tree
/// builder has not read it), and the kept scripts of the page for the reader of the structured
/// data.
type Prepared<'a> = (String, Option<crate::types::WarningKind>, Option<&'a str>, KeptScripts);

/// The page that the first parse reads, with what [`Prepared`] names.
pub(super) fn prepare_html<'a>(html: &'a str, options: &ConversionOptions) -> Result<Prepared<'a>> {
    let hidden_content = options.hidden_content;
    let (preprocessed, kept_scripts) = preprocess_initial_html_with_kept_scripts(html, options, hidden_content)?;
    if has_custom_element_tags(&preprocessed) {
        let (repaired, limit) = repair_custom_elements(
            preprocessed,
            preserves_menu(options),
            hidden_content,
            keep_inert_wrappers(options, hidden_content),
        );
        return Ok((repaired, limit, None, kept_scripts));
    }
    Ok((preprocessed, None, Some(html), kept_scripts))
}

/// Remove the elements that the `hidden` attribute or an inline style hides, unless the caller
/// keeps hidden content. A kept element then converts like any other element.
fn strip_hidden(html: &str, hidden_content: HiddenContent) -> Cow<'_, str> {
    match hidden_content {
        HiddenContent::Drop => strip_hidden_elements(html),
        HiddenContent::Reachable | HiddenContent::All => Cow::Borrowed(html),
    }
}

/// The page that the first parse reads under `hidden_content`, before the repair of custom
/// elements.
pub(super) fn preprocess_initial_html(
    html: &str,
    options: &ConversionOptions,
    hidden_content: HiddenContent,
) -> Result<String> {
    Ok(preprocess_initial_html_with_kept_scripts(html, options, hidden_content)?.0)
}

/// Each number that the page of the conversion of `html` has after the name of an owner mark,
/// as the cleanup leaves it.
///
/// ~keep The repair of a script with no end tag writes owner marks before the conversion reads
/// ~keep the page. It asks for these numbers and writes its marks again with none of them, as
/// ~keep the second pass of the kept scripts does: one reservation for the two writers of a mark.
pub fn owner_mark_numbers_after_cleanup(html: &str, options: &ConversionOptions) -> Result<Vec<usize>> {
    let page = preprocess_initial_html(html, options, options.hidden_content)?;
    Ok(owner_mark_numbers(page.as_bytes()).into_iter().collect())
}

/// [`preprocess_initial_html`], with the kept scripts of `html`.
///
/// ~keep The conversion takes the kept scripts of the page of its own choice of `hidden_content`
/// ~keep only. The page of a narrower choice is read to ask whether it takes the tree builder,
/// ~keep and its kept scripts are not used. It is written as the page of the conversion is, with
/// ~keep the second pass, so two choices that keep the same content give equal pages.
fn preprocess_initial_html_with_kept_scripts(
    html: &str,
    options: &ConversionOptions,
    hidden_content: HiddenContent,
) -> Result<(String, KeptScripts)> {
    let stripped = strip_html_cdata(html);
    let (first_pass, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&stripped);
    let page = clean_up(&first_pass, options, hidden_content)?;
    // ~keep The cleanup can join two parts of a name of the author into the name of an owner
    // ~keep mark. A page with a mark is written a second time, with no number that the cleaned
    // ~keep page has: the names of the author are reserved as the cleanup leaves them. A page
    // ~keep with no mark, which is most pages, is read once.
    #[cfg(feature = "metadata")]
    {
        if let Some(reserving) = kept_scripts.reserving_owner_names_of(&page) {
            let (first_pass, kept_scripts) = strip_script_and_style_tags_for_second_pass(&stripped, reserving);
            return Ok((clean_up(&first_pass, options, hidden_content)?, kept_scripts));
        }
    }
    Ok((page, kept_scripts))
}

/// The cleanup of the page that the first pass wrote, before the repair of custom elements.
fn clean_up(first_pass: &str, options: &ConversionOptions, hidden_content: HiddenContent) -> Result<String> {
    // ~keep Bogus comments must be removed before tag-shaped preprocessing examines them.
    let stripped = strip_bogus_comments(first_pass);
    let stripped = strip_hidden(&stripped, hidden_content);
    let stripped = if keep_inert_wrappers(options, hidden_content) {
        Cow::Borrowed(stripped.as_ref())
    } else {
        exclusions::remove_kept_tags(&stripped, hidden_content, &options.exclude_selectors)?
    };
    let stripped = normalize_bogus_comment_endings(&stripped);
    let stripped = normalize_split_closing_tags(&stripped);
    // ~keep Implicit list-item closes prevent deeply nested parser output on large changelogs.
    let stripped = normalize_unclosed_list_items(&stripped);
    let stripped = normalize_menu_elements(&stripped, preserves_menu(options));
    Ok(preprocess_html(&stripped).into_owned())
}

pub(super) fn preprocess_repaired_html(html: &str, preserve_menu: bool, hidden_content: HiddenContent) -> String {
    let stripped = strip_html_cdata(html);
    let stripped = strip_script_and_style_tags(&stripped);
    let stripped = strip_hidden(&stripped, hidden_content);
    let stripped = normalize_bogus_comment_endings(&stripped);
    let stripped = normalize_split_closing_tags(&stripped);
    let stripped = normalize_unclosed_list_items(&stripped);
    let stripped = normalize_menu_elements(&stripped, preserve_menu);
    preprocess_html(&stripped).into_owned()
}

#[cfg(feature = "visitor")]
pub(super) fn keep_inert_wrappers(options: &ConversionOptions, hidden_content: HiddenContent) -> bool {
    options.visitor.is_some()
        && hidden_content == HiddenContent::All
        && options.output_format != crate::options::OutputFormat::Plain
}

#[cfg(not(feature = "visitor"))]
pub(super) const fn keep_inert_wrappers(_options: &ConversionOptions, _hidden_content: HiddenContent) -> bool {
    false
}

fn repair_custom_elements(
    preprocessed: String,
    preserve_menu: bool,
    hidden_content: HiddenContent,
    keep_templates: bool,
) -> (String, Option<crate::types::WarningKind>) {
    let repair = if keep_templates {
        crate::converter::main_helpers::repair_with_html5ever(&preprocessed, true)
    } else {
        repair_with_html5ever_bounded(&preprocessed)
    };
    let Some((repaired, warning)) = repair else {
        tracing::warn!(
            target: "html_to_markdown::convert",
            "custom element tags detected; html5ever repair failed, proceeding with unrepaired markup"
        );
        return (preprocessed, None);
    };
    tracing::warn!(
        target: "html_to_markdown::convert",
        "custom element tags detected; re-parsed input with html5ever repair fallback"
    );
    (
        preprocess_repaired_html(&repaired, preserve_menu, hidden_content),
        warning,
    )
}

#[cfg(test)]
mod tests {
    use super::{ConversionOptions, HiddenContent, prepare_html, preprocess_initial_html, preprocess_repaired_html};

    #[test]
    fn should_keep_the_initial_inert_wrapper_as_a_metadata_boundary_after_cleanup() {
        let html = r#"<template><p>inert</p></template><meta name="description" content="D"><p>body</p>"#;
        let options = ConversionOptions::default();
        let page = preprocess_initial_html(html, &options, HiddenContent::Drop).expect("cleaned page");
        assert!(page.starts_with("<template>"), "{page}");
        assert_eq!(
            crate::converter::utility::preprocessing::metadata_prefix_end(&page),
            0,
            "{page}"
        );
    }

    /// The page of a choice is the page that the conversion of that choice reads, with the
    /// numbers of the second pass: a narrower page is compared with it byte by byte.
    #[test]
    fn the_page_of_a_choice_is_the_page_that_the_conversion_reads() {
        let html = r#"<p>t</p><script type="application/ld+json">{"a":"<b>"}</script><p>after</p>"#;
        let options = ConversionOptions::default();
        let page = preprocess_initial_html(html, &options, HiddenContent::Drop).expect("the page");
        let (read, _, _, _) = prepare_html(html, &options).expect("the page");
        assert_eq!(page, read);
        #[cfg(feature = "metadata")]
        assert_eq!(page.matches("data-h2m-script-1=").count(), 1, "{page}");
    }

    /// The pass reads a repaired page too: the tree builder writes each `<` of a kept script
    /// as `<` again, and the parser gets it as text only when the pass writes it as `&lt;`.
    #[test]
    fn a_lt_of_a_kept_script_of_a_repaired_page_is_written_as_text() {
        let html = r#"<p>t</p><script type="application/ld+json">{"a":"<b>"}</script><p>after</p>"#;
        let page = preprocess_repaired_html(html, false, HiddenContent::Drop);
        assert!(page.contains(r#"{"a":"&lt;b>"}"#), "{page}");
    }
}
