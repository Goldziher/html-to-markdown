//! HTML cleanup and normalization used before conversion.

mod form_boundaries;
mod head_scan;
mod inert;
mod kept;
mod kept_scripts;
mod lists;
mod markup;
mod menu;
mod preserved_bodies;
mod raw_text;
mod restoration;
mod structured_data;
mod visibility;

pub use form_boundaries::removes_closed_forms;
pub use inert::{html_inert_starts, normalize_inert_bodies_with_preservation, remove_closed_form_bodies};
pub use kept_scripts::{KeptScripts, owner_mark_numbers};
pub use lists::normalize_unclosed_list_items;
#[cfg(feature = "metadata")]
pub use markup::{ScriptType, script_type};
pub use markup::{
    find_html_comment_end, find_tag_end, normalize_bogus_comment_endings, normalize_split_closing_tags,
    preprocess_html, strip_html_cdata,
};
pub use menu::{PRESERVED_MENU_ATTRIBUTE, normalize_menu_elements, restore_preserved_menu_elements};
#[cfg(feature = "metadata")]
pub use raw_text::strip_script_and_style_tags_for_second_pass;
pub use raw_text::{
    find_raw_text_end_tag, skip_opaque_region, strip_script_and_style_tags,
    strip_script_and_style_tags_with_kept_scripts,
};
pub use restoration::restore_inert_bodies;
pub use structured_data::{repair_unclosed_structured_data_scripts, repair_unclosed_structured_data_scripts_reserving};
pub use visibility::{
    HiddenStyleReason, sanitize_markdown_url, strip_bogus_comments, strip_hidden_elements, style_value_hidden_reason,
    tag_has_hidden_attribute, tag_has_hidden_style, unwrap_kept_inert_elements,
};

#[cfg(test)]
use raw_text::{find_closing_tag_bytes, find_closing_tag_bytes_nested};

#[cfg(test)]
mod excluded_tests;
#[cfg(test)]
mod kept_script_tests;
#[cfg(test)]
mod tests;
