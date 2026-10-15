//! Front matter extracted from the source prefix before body content.

use crate::converter::main_helpers::{extract_head_metadata, format_metadata_frontmatter};
use crate::options::ConversionOptions;

/// Extract front matter from metadata before the body, including an implicit document head.
pub fn extract_frontmatter(
    html: &str,
    options: &ConversionOptions,
    document_base_href: Option<&str>,
) -> Option<String> {
    if !options.extract_metadata {
        return None;
    }

    let prefix_end = crate::converter::utility::preprocessing::metadata_prefix_end(html);
    let prefix = crate::converter::utility::preprocessing::normalize_bogus_comment_endings(&html[..prefix_end]);

    let dom = match tl::parse(&prefix, tl::ParserOptions::default()) {
        Ok(d) => d,
        Err(_) => return None,
    };

    // ~keep Delegate to the canonical Tier-2 extractor so the two tiers agree on
    // ~keep key names / casing / value normalisation byte-for-byte.
    let metadata = extract_head_metadata(dom.children(), dom.parser(), options, document_base_href, prefix.len());

    format_metadata_frontmatter(&metadata)
}
