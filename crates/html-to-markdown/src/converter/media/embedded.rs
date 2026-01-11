//! Embedded media element handling (iframe, video, audio, source).

use std::borrow::Cow;
use tl::{HTMLTag, NodeHandle, Parser};

/// Extract src attribute from media element (audio, video, iframe).
pub(crate) fn extract_media_src<'a>(tag: &'a HTMLTag<'a>) -> Cow<'a, str> {
    tag.attributes()
        .get("src")
        .flatten()
        .map(|v| v.as_utf8_str())
        .unwrap_or_else(|| Cow::Borrowed(""))
}

/// Try to find source src from nested source element.
///
/// Used by audio and video elements to extract src from child <source> elements
/// when the parent doesn't have a src attribute.
pub(crate) fn find_source_src<'a>(children: &[NodeHandle], parser: &'a Parser) -> Option<Cow<'a, str>> {
    for child_handle in children.iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            if tag_name_eq(child_tag.name().as_utf8_str(), "source") {
                return child_tag.attributes().get("src").flatten().map(|v| v.as_utf8_str());
            }
        }
    }
    None
}

/// Check if tag is a source element.
pub(crate) fn is_source_element(tag: &HTMLTag) -> bool {
    tag_name_eq(tag.name().as_utf8_str(), "source")
}

/// Compare tag name with needle (case-insensitive).
fn tag_name_eq<'a>(name: impl AsRef<str>, needle: &str) -> bool {
    name.as_ref().eq_ignore_ascii_case(needle)
}

/// Determine if media should output source link in markdown.
///
/// Returns true if src is non-empty.
pub(crate) fn should_output_media_link(src: &str) -> bool {
    !src.is_empty()
}
