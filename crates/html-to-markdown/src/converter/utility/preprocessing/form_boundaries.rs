use std::ops::Range;

use html5ever::LocalName;
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
    start: Option<FormStart>,
    templates: Vec<TemplateForms>,
    table_depth: usize,
    selects: Vec<SelectScope>,
    ranges: Vec<ClosedForm>,
}

struct TemplateForms {
    starts: Vec<FormStart>,
    table_depth: usize,
    selects: Vec<SelectScope>,
}

struct FormStart {
    start: usize,
    end: usize,
    select_depth: usize,
    released: bool,
}

struct SelectScope {
    blockers: Vec<LocalName>,
    table_depth: usize,
    forms: Vec<(usize, usize)>,
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
            "table" => self.observe_table(opening),
            "template" if opening => {
                self.templates.push(TemplateForms {
                    starts: Vec::new(),
                    table_depth: self.table_depth,
                    selects: std::mem::take(&mut self.selects),
                });
                self.table_depth = 0;
            }
            "template" => {
                if let Some(template) = self.templates.pop() {
                    self.table_depth = template.table_depth;
                    self.selects = template.selects;
                }
            }
            "select" => self.observe_select(opening),
            "input" if opening => self.observe_select(false),
            "object" | "applet" | "marquee" => self.observe_select_blocker(opening, &tag.name),
            "form" if self.table_depth == 0 => self.observe_form(opening, start, end),
            _ => {}
        }
    }

    fn observe_table(&mut self, opening: bool) {
        self.table_depth = if opening {
            self.table_depth.saturating_add(1)
        } else {
            self.table_depth.saturating_sub(1)
        };
        if !opening {
            while self
                .selects
                .last()
                .is_some_and(|scope| scope.table_depth > self.table_depth)
            {
                self.release_select_forms();
                self.selects.pop();
            }
        }
    }

    fn select_in_scope(&self) -> bool {
        #[cfg(test)]
        SELECT_SCOPE_VISITS.with(|counter| counter.set(counter.get() + 1));
        self.selects
            .last()
            .is_some_and(|scope| scope.blockers.is_empty() && self.table_depth <= scope.table_depth)
    }

    fn observe_select(&mut self, opening: bool) {
        if self.select_in_scope() {
            self.release_select_forms();
            self.selects.pop();
        } else if opening {
            self.selects.push(SelectScope {
                blockers: Vec::new(),
                table_depth: self.table_depth,
                forms: Vec::new(),
            });
        }
    }

    fn observe_select_blocker(&mut self, opening: bool, name: &LocalName) {
        #[cfg(test)]
        SELECT_SCOPE_VISITS.with(|counter| counter.set(counter.get() + 1));
        let Some(scope) = self.selects.last_mut() else {
            return;
        };
        if opening {
            scope.blockers.push(name.clone());
        } else if scope.blockers.last() == Some(name) {
            scope.blockers.pop();
        }
    }

    fn release_select_forms(&mut self) {
        // ~keep Scope release leaves the form pointer alive until its end token.
        let select_depth = self.selects.len();
        let forms = self
            .selects
            .last_mut()
            .map_or_else(Vec::new, |scope| std::mem::take(&mut scope.forms));
        if let Some(template) = self.templates.last_mut() {
            for (index, start) in forms {
                #[cfg(test)]
                SELECT_RELEASE_VISITS.with(|counter| counter.set(counter.get() + 1));
                // ~keep LIFO slots can be reused after an end token; the source offset identifies the owned start.
                if let Some(form) = template.starts.get_mut(index)
                    && form.start == start
                    && form.select_depth == select_depth
                {
                    form.released = true;
                }
            }
        } else if let Some(form) = &mut self.start {
            form.released |= form.select_depth == select_depth;
        }
    }

    fn observe_form(&mut self, opening: bool, start: usize, end: usize) {
        if opening {
            let form = FormStart {
                start,
                end,
                select_depth: self.selects.len(),
                released: false,
            };
            if let Some(template) = self.templates.last_mut() {
                // ~keep HTML templates permit nested forms instead of using the body form pointer.
                if let Some(scope) = self.selects.last_mut() {
                    scope.forms.push((template.starts.len(), start));
                }
                template.starts.push(form);
            } else {
                self.start.get_or_insert(form);
            }
            return;
        }
        let opened = self
            .templates
            .last_mut()
            .map_or_else(|| self.start.take(), |template| template.starts.pop());
        if let Some(form) = opened
            && !form.released
        {
            self.ranges.push(ClosedForm {
                range: form.start..end,
                open_end: form.end,
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
thread_local! {
    static SELECT_RELEASE_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static SELECT_SCOPE_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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
    fn should_reset_table_scope_for_forms_at_a_kept_template_start() {
        assert_eq!(
            remove_closed_form_bodies(
                "<table><template><form>a</form>b</template></table>",
                HiddenContent::All
            ),
            "<table><template><form></form>b</template></table>"
        );
    }

    #[test]
    fn should_ignore_a_form_end_in_foreign_content_without_a_foreign_form_start() {
        assert_eq!(
            remove_closed_form_bodies("<form>a<svg></form></svg>b</form>c", HiddenContent::Drop),
            "<form></form>c"
        );
    }

    #[test]
    fn should_leave_forms_started_in_selects_to_the_html_tree_builder() {
        for html in ["<select><option>o<form>a</select>x</form>c", "<select><option>o<form>a"] {
            assert_eq!(remove_closed_form_bodies(html, HiddenContent::Drop), html);
        }
        assert_eq!(
            remove_closed_form_bodies("<select><select><form>a<span>b</form>c", HiddenContent::Drop),
            "<select><select><form></form>c"
        );
        assert_eq!(
            remove_closed_form_bodies("<select><input><form>a<span>b</form>c", HiddenContent::Drop),
            "<select><input><form></form>c"
        );
        assert_eq!(
            remove_closed_form_bodies(
                "<form>a</form><select><form>b</select>x</form><form>c</form>d",
                HiddenContent::Drop
            ),
            "<form></form><select><form>b</select>x</form><form></form>d"
        );
    }

    #[test]
    fn should_bound_select_release_work_for_nested_template_forms() {
        for forms in [100, 400] {
            for selected in [
                "<select></select>",
                "<select><form>x</select>",
                "<select><form>x</form></select>",
            ] {
                let extra_ends = usize::from(selected == "<select><form>x</select>") * forms;
                let html = format!(
                    "<template>{}{}{}</template>",
                    "<form>".repeat(forms),
                    selected.repeat(forms),
                    "</form>".repeat(forms + extra_ends)
                );
                super::SELECT_RELEASE_VISITS.with(|counter| counter.set(0));
                assert_eq!(
                    remove_closed_form_bodies(&html, HiddenContent::All),
                    "<template><form></form></template>"
                );
                let visits = super::SELECT_RELEASE_VISITS.with(std::cell::Cell::get);
                assert!(
                    visits <= 2 * forms,
                    "forms={forms}, selected={selected}, visits={visits}"
                );
                let expected = if selected == "<select></select>" { 0 } else { forms };
                assert_eq!(visits, expected, "forms={forms}, selected={selected}");
            }
        }
    }

    #[test]
    fn should_keep_outer_and_nested_template_form_ownership_when_select_scope_releases() {
        for (html, expected) in [
            ("<form>a<select><form>b</select>x</form>c", "<form></form>c"),
            (
                "<select><form>a<template><form>b</form>c</template></select>x</form>d",
                "<select><form>a<template><form></form>c</template></select>x</form>d",
            ),
            (
                "<template><form>a<select><form>b</form>c</select>d</form>e</template>",
                "<template><form></form>e</template>",
            ),
        ] {
            assert_eq!(remove_closed_form_bodies(html, HiddenContent::All), expected, "{html}");
        }
    }

    #[test]
    fn should_bound_ignored_select_closes_beneath_deep_scope_blockers() {
        for depth in [100, 400] {
            let prefix = format!("<template><select>{}", "<object>".repeat(depth));
            let suffix = format!("tail{}</select></template>", "</object>".repeat(depth));
            let html = format!("{prefix}<form>x{}</form>{suffix}", "</applet></select>".repeat(depth));
            super::SELECT_SCOPE_VISITS.with(|counter| counter.set(0));
            assert_eq!(
                remove_closed_form_bodies(&html, HiddenContent::All),
                format!("{prefix}<form></form>{suffix}")
            );
            assert_eq!(super::SELECT_SCOPE_VISITS.with(std::cell::Cell::get), 4 * depth + 2);
        }
    }

    #[test]
    fn should_keep_nested_select_owners_separate_inside_kept_templates() {
        assert_eq!(
            remove_closed_form_bodies(
                "<template><select><object><form>a<select><form>b</select>x</form>c</form>d</object></select></template>",
                HiddenContent::All
            ),
            "<template><select><object><form></form>d</object></select></template>"
        );
        assert_eq!(
            remove_closed_form_bodies(
                "<select><object><template><select><form>a</select>x</form></template><form>b</select>c</form>d</object></select>",
                HiddenContent::All
            ),
            "<select><object><template><select><form>a</select>x</form></template><form></form>d</object></select>"
        );
    }

    #[test]
    fn should_release_outer_select_forms_beneath_released_inner_forms() {
        let html =
            "<template><select><object><form>a<select><form>b</select>x</object></select>c</form>d</form>e</template>";
        assert_eq!(remove_closed_form_bodies(html, HiddenContent::All), html);
    }

    #[test]
    fn should_leave_reused_template_form_slots_with_their_current_select_owner() {
        assert_eq!(
            remove_closed_form_bodies(
                "<template><select><form>a</form><object><select><form>b</select>x</object></select>c</form>d</template>",
                HiddenContent::All
            ),
            "<template><select><form></form><object><select><form>b</select>x</object></select>c</form>d</template>"
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
