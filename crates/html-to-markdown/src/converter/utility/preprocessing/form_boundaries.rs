use std::ops::Range;

use html5ever::tokenizer::{Tag, TagKind};

use crate::options::{ConversionOptions, PreprocessingPreset};

/// Whether source removal can preserve the caller's DOM visitation and serialization rules.
pub fn removes_closed_forms(source: &str, options: &ConversionOptions, has_visitor: bool) -> bool {
    options.preprocessing.enabled
        && options.preprocessing.remove_forms
        && options.preprocessing.preset != PreprocessingPreset::Minimal
        && !has_visitor
        // ~keep A preserved ancestor serializes its form before walking children. Preserve
        // ~keep that contract conservatively; void tags cannot be such ancestors (#879).
        && !options.preserve_tags.iter().any(|name| {
            !name.eq_ignore_ascii_case("form")
                && !crate::converter::main_helpers::is_html5_void_element(name.as_bytes())
        })
        && memchr::memchr_iter(b'<', source.as_bytes()).any(|at| {
            source.as_bytes().get(at + 1..at + 5).is_some_and(|name| name.eq_ignore_ascii_case(b"form"))
        })
}

#[derive(Default)]
pub(super) struct ClosedForms {
    start: Option<(usize, usize)>,
    templates: Vec<TemplateForms>,
    table_depth: usize,
    ranges: Vec<ClosedForm>,
}

struct TemplateForms {
    starts: Vec<(usize, usize)>,
    table_depth: usize,
}

struct ClosedForm {
    range: Range<usize>,
    open_end: usize,
    close_start: usize,
}

impl ClosedForms {
    pub(super) fn observe(&mut self, tag: &Tag, start: usize, end: usize) {
        let opening = tag.kind == TagKind::StartTag;
        match &*tag.name {
            "table" => {
                self.table_depth = if opening {
                    self.table_depth.saturating_add(1)
                } else {
                    self.table_depth.saturating_sub(1)
                };
            }
            "template" if opening => {
                self.templates.push(TemplateForms {
                    starts: Vec::new(),
                    table_depth: self.table_depth,
                });
                self.table_depth = 0;
            }
            "template" => {
                if let Some(template) = self.templates.pop() {
                    self.table_depth = template.table_depth;
                }
            }
            "form" if self.table_depth == 0 => self.observe_form(opening, start, end),
            _ => {}
        }
    }

    fn observe_form(&mut self, opening: bool, start: usize, end: usize) {
        if opening {
            if let Some(template) = self.templates.last_mut() {
                // ~keep HTML templates permit nested forms instead of using the body form pointer.
                template.starts.push((start, end));
            } else {
                self.start.get_or_insert((start, end));
            }
            return;
        }
        let opened = self
            .templates
            .last_mut()
            .map_or_else(|| self.start.take(), |template| template.starts.pop());
        if let Some((open_start, open_end)) = opened {
            self.ranges.push(ClosedForm {
                range: open_start..end,
                open_end,
                close_start: start,
            });
        }
    }

    pub(super) fn apply(self, source: &str, mut replacements: super::kept::Replacements) -> super::kept::Replacements {
        if self.ranges.is_empty() {
            return replacements;
        }
        replacements.extend(self.ranges.into_iter().map(|form| {
            // ~keep Empty original form tags retain their paragraph/body boundaries while
            // ~keep removing open descendants that would otherwise capture following text.
            let mut replacement = source[form.range.start..form.open_end].to_owned();
            replacement.push_str(&source[form.close_start..form.range.end]);
            (form.range, replacement)
        }));
        // ~keep A removed form supersedes closed forms in kept templates wholly inside it.
        // ~keep Tokenized spans are disjoint or nested, never crossing.
        replacements.sort_by(|(a, _), (b, _)| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));
        let mut end = 0;
        replacements.retain(|(range, _)| {
            if range.start < end {
                return false;
            }
            end = range.end;
            true
        });
        replacements
    }
}

#[cfg(test)]
mod tests {
    use super::removes_closed_forms;
    use crate::converter::utility::preprocessing::remove_closed_form_bodies;
    use crate::options::{ConversionOptions, HiddenContent};
    use std::borrow::Cow;

    #[test]
    fn should_leave_unclosed_form_source_and_opaque_fake_forms_as_written() {
        for html in [
            "<form>a<span>b<p>tail",
            "<script><form>x</form></script><p>tail</p>",
            "<textarea><form>x</form></textarea><p>tail</p>",
            "<template><form>x</form></template><p>tail</p>",
            "<svg><foreignObject><form>x</form></foreignObject></svg><p>tail</p>",
        ] {
            assert!(
                matches!(remove_closed_form_bodies(html, HiddenContent::Drop), Cow::Borrowed(value) if value == html),
                "{html}"
            );
        }
    }

    #[test]
    fn should_keep_the_first_body_form_start_when_a_nested_start_is_ignored() {
        assert_eq!(
            remove_closed_form_bodies("<form>a<form>b</form>c</form>d", HiddenContent::Drop),
            "<form></form>c</form>d"
        );
    }

    #[test]
    fn should_ignore_an_opaque_template_end_when_choosing_the_real_form_boundary() {
        let html = "<form>a<template></form></template><span>b</form>c";
        assert_eq!(remove_closed_form_bodies(html, HiddenContent::Drop), "<form></form>c");
    }

    #[test]
    fn should_supersede_nested_template_form_ranges_without_reordering_source() {
        let html = "é<form data-x='雪'>a<template><form>b</form></template>c</form>d<form>e</form>f";
        assert_eq!(
            remove_closed_form_bodies(html, HiddenContent::All),
            "é<form data-x='雪'></form>d<form></form>f"
        );
    }

    #[test]
    fn should_restore_outer_table_fallback_when_a_template_table_closes() {
        let html = "<table><template><table></table></template><form>row</form></table><form>a<span>b</form>c";
        assert_eq!(
            remove_closed_form_bodies(html, HiddenContent::All),
            "<table><template><table></table></template><form>row</form></table><form></form>c"
        );
    }

    #[test]
    fn should_only_enable_source_form_removal_when_serialization_and_visitation_allow_it() {
        let html = "<form>a<span>b</form>c";
        assert!(removes_closed_forms(html, &ConversionOptions::default(), false));
        assert!(!removes_closed_forms(html, &ConversionOptions::default(), true));
        for name in ["form", "img", "IMG"] {
            assert!(removes_closed_forms(
                html,
                &ConversionOptions {
                    preserve_tags: vec![name.into()],
                    ..Default::default()
                },
                false
            ));
        }
        assert!(!removes_closed_forms(
            html,
            &ConversionOptions {
                preserve_tags: vec!["div".into()],
                ..Default::default()
            },
            false
        ));
        assert!(!removes_closed_forms(
            "<p>no form</p>",
            &ConversionOptions::default(),
            false
        ));
    }
}
