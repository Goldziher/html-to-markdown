//! Which inert elements a choice of `hidden_content` keeps.

use html5ever::tokenizer::Tag;

use crate::options::HiddenContent;

/// Whether `hidden_content` keeps the content of the element that the start tag `tag` starts.
/// The inert-body pass leaves such an element as written and reads its content as markup.
///
/// ~keep The inert-body pass runs in front of the pass that removes the tags of the kept
/// ~keep `<template>` and `<noscript>` elements. It empties the body of each of the two elements,
/// ~keep so the later pass would find no content to keep. The rule here is the rule of that
/// ~keep pass: [`HiddenContent::All`] keeps every `<template>` and `<noscript>`, and
/// ~keep [`HiddenContent::Reachable`] keeps a declarative shadow root. Under
/// ~keep [`HiddenContent::Drop`] the inert-body pass empties every body, as it does with no
/// ~keep option.
///
/// ~keep The rule does not ask where the head ends. An element of the two in the head stays as
/// ~keep written under a choice that keeps its kind, the later pass leaves its tags, and the
/// ~keep walk drops it.
pub(super) fn keeps_inert_body(hidden_content: HiddenContent, tag: &Tag) -> bool {
    match hidden_content {
        HiddenContent::Drop => false,
        HiddenContent::Reachable => &*tag.name == "template" && declares_shadow_root(tag),
        HiddenContent::All => matches!(&*tag.name, "template" | "noscript"),
    }
}

/// Whether the start tag of a `<template>` declares a shadow root: `shadowrootmode` is `open` or
/// `closed`.
fn declares_shadow_root(tag: &Tag) -> bool {
    tag.attrs.iter().any(|attribute| {
        &*attribute.name.local == "shadowrootmode"
            && (attribute.value.eq_ignore_ascii_case("open") || attribute.value.eq_ignore_ascii_case("closed"))
    })
}
