use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct PreservedBodies {
    pub(super) originals: HashMap<String, String>,
    occupied: HashSet<usize>,
    next: usize,
}

const BODY_MARKER_PREFIX: &str = "h2m_inert_body_";
const SCRIPT_MARKER_PREFIX: &str = "h2m_script_body_";
const SCRIPT_ATTRIBUTE_PREFIX: &str = "data-h2m-script-";

fn marker_id(marker: &str, prefix: &str) -> Option<usize> {
    marker.strip_prefix(prefix)?.strip_suffix("_end")?.parse().ok()
}

pub(super) fn script_marker_attribute(marker: &str) -> Option<String> {
    marker_id(marker, SCRIPT_MARKER_PREFIX).map(|id| format!("{SCRIPT_ATTRIBUTE_PREFIX}{id}"))
}

fn occupied_marker_ids<'a>(source: &'a str, prefix: &'a str) -> impl Iterator<Item = usize> + 'a {
    source.match_indices(prefix).filter_map(move |(index, _)| {
        let rest = &source[index + prefix.len()..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        rest.get(digits..).filter(|suffix| suffix.starts_with("_end"))?;
        rest[..digits].parse().ok()
    })
}

fn occupied_attribute_ids(source: &[u8]) -> impl Iterator<Item = usize> + '_ {
    memchr::memchr2_iter(b'd', b'D', source).filter_map(|index| {
        let end = index + SCRIPT_ATTRIBUTE_PREFIX.len();
        source
            .get(index..end)
            .filter(|name| name.eq_ignore_ascii_case(SCRIPT_ATTRIBUTE_PREFIX.as_bytes()))?;
        let rest = source.get(end..)?;
        let digits = rest.iter().take_while(|byte| byte.is_ascii_digit()).count();
        std::str::from_utf8(&rest[..digits]).ok()?.parse().ok()
    })
}

impl PreservedBodies {
    pub(super) fn new(source: &str) -> Self {
        let decoded = crate::text::decode_html_entities_cow(source);
        // ~keep Later bogus-comment removal can join authored marker and owner names; reserve that exact spelling too.
        let sanitized = super::visibility::strip_bogus_comments(source);
        let sanitized_decoded = crate::text::decode_html_entities_cow(&sanitized);
        let occupied = [decoded.as_ref(), sanitized_decoded.as_ref()]
            .into_iter()
            .flat_map(|source| {
                [BODY_MARKER_PREFIX, SCRIPT_MARKER_PREFIX]
                    .into_iter()
                    .flat_map(move |prefix| occupied_marker_ids(source, prefix))
                    .chain(occupied_attribute_ids(source.as_bytes()))
            })
            .collect();
        Self {
            occupied,
            ..Self::default()
        }
    }

    pub(super) fn extend_originals(&mut self, originals: HashMap<String, String>) {
        self.occupied.extend(originals.keys().filter_map(|marker| {
            marker_id(marker, BODY_MARKER_PREFIX).or_else(|| marker_id(marker, SCRIPT_MARKER_PREFIX))
        }));
        self.originals.extend(originals);
    }

    pub(super) fn protect(&mut self, body: &str) -> String {
        self.protect_with_prefix(body, BODY_MARKER_PREFIX)
    }

    pub(super) fn protect_script(&mut self, body: &str) -> (String, String) {
        let marker = self.protect_with_prefix(body, SCRIPT_MARKER_PREFIX);
        let attribute = format!("{SCRIPT_ATTRIBUTE_PREFIX}{}", self.next - 1);
        (marker, attribute)
    }

    fn protect_with_prefix(&mut self, body: &str, prefix: &str) -> String {
        while self.occupied.contains(&self.next) {
            self.next += 1;
        }
        let marker = format!("{prefix}{}_end", self.next);
        self.next += 1;
        self.originals.insert(marker.clone(), body.to_owned());
        marker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reserve_sanitized_owner_names_without_joining_quoted_processing_instruction_bytes() {
        let forged =
            r"<<?x?>script data-h2m-script-<?y?>0='h2m_script_body_<?z?>0_end'>h2m_script_body_<?w?>0_end</script>";
        assert_eq!(
            PreservedBodies::new(forged).protect_script("JSON").0,
            "h2m_script_body_1_end"
        );
        let quoted = r"<script data-x='h2m_script_body_<?x?>0_end'></script>";
        assert_eq!(
            PreservedBodies::new(quoted).protect_script("JSON").0,
            "h2m_script_body_0_end"
        );
    }

    #[test]
    fn should_reserve_authored_attribute_names_in_any_case_and_both_marker_kinds() {
        let source = "<script DATA-H2M-SCRIPT-0='author' data-h2m-script-1='second'></script><p>h2m_inert_body_2_end h2m_script_body_&#51;_end</p>";
        let mut preserved = PreservedBodies::new(source);
        assert_eq!(
            preserved.protect_script("JSON"),
            ("h2m_script_body_4_end".into(), "data-h2m-script-4".into())
        );
        assert_eq!(preserved.originals["h2m_script_body_4_end"], "JSON");
        assert_eq!(preserved.protect("fallback"), "h2m_inert_body_5_end");
        assert_eq!(preserved.originals["h2m_inert_body_5_end"], "fallback");
    }
}
