use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct PreservedBodies {
    pub(super) originals: HashMap<String, String>,
    occupied: HashSet<usize>,
    next: usize,
}

const BODY_MARKER_PREFIX: &str = "h2m_inert_body_";

impl PreservedBodies {
    pub(super) fn new(source: &str) -> Self {
        let decoded = crate::text::decode_html_entities_cow(source);
        let source = decoded.as_ref();
        let occupied = source
            .match_indices(BODY_MARKER_PREFIX)
            .filter_map(|(index, _)| {
                let rest = &source[index + BODY_MARKER_PREFIX.len()..];
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                rest.get(digits..).filter(|suffix| suffix.starts_with("_end"))?;
                rest[..digits].parse().ok()
            })
            .collect();
        Self {
            occupied,
            ..Self::default()
        }
    }

    pub(super) fn extend_originals(&mut self, originals: HashMap<String, String>) {
        self.occupied.extend(originals.keys().filter_map(|marker| {
            marker
                .strip_prefix(BODY_MARKER_PREFIX)?
                .strip_suffix("_end")?
                .parse::<usize>()
                .ok()
        }));
        self.originals.extend(originals);
    }

    pub(super) fn protect(&mut self, body: &str) -> String {
        while self.occupied.contains(&self.next) {
            self.next += 1;
        }
        let marker = format!("{BODY_MARKER_PREFIX}{}_end", self.next);
        self.next += 1;
        self.originals.insert(marker.clone(), body.to_owned());
        marker
    }
}
