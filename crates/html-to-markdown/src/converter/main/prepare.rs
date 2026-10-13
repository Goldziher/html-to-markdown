//! The passes over a page before a parse reads it.

use std::borrow::Cow;

use super::exclusions;
use crate::converter::main_helpers::{has_custom_element_tags, preserves_menu, repair_with_html5ever_bounded};
use crate::converter::utility::preprocessing::{
    KeptScripts, normalize_bogus_comment_endings, normalize_menu_elements, normalize_split_closing_tags,
    normalize_unclosed_list_items, preprocess_html, strip_bogus_comments, strip_hidden_elements, strip_html_cdata,
    strip_script_and_style_tags, strip_script_and_style_tags_with_kept_scripts,
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
        let (repaired, limit) = repair_custom_elements(preprocessed, preserves_menu(options), hidden_content);
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

/// [`preprocess_initial_html`], with the kept scripts of `html`.
///
/// ~keep The conversion takes the kept scripts of the page of its own choice of `hidden_content`
/// ~keep only. The page of a narrower choice is read to ask whether it takes the tree builder,
/// ~keep and its kept scripts are not used.
fn preprocess_initial_html_with_kept_scripts(
    html: &str,
    options: &ConversionOptions,
    hidden_content: HiddenContent,
) -> Result<(String, KeptScripts)> {
    let stripped = strip_html_cdata(html);
    let (stripped, kept_scripts) = strip_script_and_style_tags_with_kept_scripts(&stripped);
    // ~keep Bogus comments must be removed before tag-shaped preprocessing examines them.
    let stripped = strip_bogus_comments(&stripped);
    let stripped = strip_hidden(&stripped, hidden_content);
    let stripped = exclusions::remove_kept_tags(&stripped, hidden_content, &options.exclude_selectors)?;
    let stripped = normalize_bogus_comment_endings(&stripped);
    let stripped = normalize_split_closing_tags(&stripped);
    // ~keep Implicit list-item closes prevent deeply nested parser output on large changelogs.
    let stripped = normalize_unclosed_list_items(&stripped);
    let stripped = normalize_menu_elements(&stripped, preserves_menu(options));
    Ok((preprocess_html(&stripped).into_owned(), kept_scripts))
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

fn repair_custom_elements(
    preprocessed: String,
    preserve_menu: bool,
    hidden_content: HiddenContent,
) -> (String, Option<crate::types::WarningKind>) {
    let Some((repaired, warning)) = repair_with_html5ever_bounded(&preprocessed) else {
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
    use super::{HiddenContent, preprocess_repaired_html};

    /// The pass reads a repaired page too: the tree builder writes each `<` of a kept script
    /// as `<` again, and the parser gets it as text only when the pass writes it as `&lt;`.
    #[test]
    fn a_lt_of_a_kept_script_of_a_repaired_page_is_written_as_text() {
        let html = r#"<p>t</p><script type="application/ld+json">{"a":"<b>"}</script><p>after</p>"#;
        let page = preprocess_repaired_html(html, false, HiddenContent::Drop);
        assert!(page.contains(r#"{"a":"&lt;b>"}"#), "{page}");
    }
}
