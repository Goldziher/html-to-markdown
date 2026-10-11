//! HTML cleanup and normalization used before conversion.

mod lists;
mod markup;
mod menu;
mod raw_text;
mod visibility;

pub use lists::normalize_unclosed_list_items;
pub use markup::{find_tag_end, normalize_bogus_comment_endings, normalize_split_closing_tags, preprocess_html};
pub use menu::{PRESERVED_MENU_ATTRIBUTE, normalize_menu_elements, restore_preserved_menu_elements};
pub use raw_text::{skip_opaque_region, strip_script_and_style_tags};
pub use visibility::{
    HiddenStyleReason, sanitize_markdown_url, strip_bogus_comments, strip_hidden_elements, style_value_hidden_reason,
    tag_has_hidden_attribute, tag_has_hidden_style,
};

#[cfg(test)]
use raw_text::{find_closing_tag_bytes, find_closing_tag_bytes_nested};

#[cfg(test)]
mod tests;
