//! HTML cleanup and normalization used before conversion.

mod head_scan;
mod inert;
mod kept;
mod lists;
mod markup;
mod menu;
mod raw_text;
mod visibility;

pub use inert::{normalize_inert_bodies_with_preservation, restore_inert_bodies};
pub use lists::normalize_unclosed_list_items;
pub use markup::{
    find_tag_end, normalize_bogus_comment_endings, normalize_split_closing_tags, preprocess_html, strip_html_cdata,
};
pub use menu::{PRESERVED_MENU_ATTRIBUTE, normalize_menu_elements, restore_preserved_menu_elements};
pub use raw_text::{skip_opaque_region, strip_script_and_style_tags};
pub use visibility::{
    HiddenStyleReason, sanitize_markdown_url, strip_bogus_comments, strip_hidden_elements, style_value_hidden_reason,
    tag_has_hidden_attribute, tag_has_hidden_style, unwrap_kept_inert_elements,
};

#[cfg(test)]
use raw_text::{find_closing_tag_bytes, find_closing_tag_bytes_nested};

#[cfg(test)]
mod tests;
