//! Recover visible markup after structured data with no script end tag.

use std::borrow::Cow;

use super::inert::scan_markup_end;
use super::markup::is_json_ld_script_open_tag;
use super::preserved_bodies::PreservedBodies;
use super::raw_text::{
    MAX_CLOSING_TAG_SCAN, find_closing_tag_bytes, find_closing_tag_bytes_nested, is_self_closing_tag, opens_a_tag,
};

/// Recover a missing JSON-LD script end tag without consuming following markup.
pub fn repair_unclosed_structured_data_scripts<'a>(
    input: &'a str,
    originals: Option<&mut std::collections::HashMap<String, String>>,
) -> Cow<'a, str> {
    let Some(originals) = originals else {
        return repair_scripts(input, None, &mut ScriptEndSearch::default());
    };
    let mut preserved = PreservedBodies::new(input);
    preserved.extend_originals(std::mem::take(originals));
    let repaired = repair_scripts(input, Some(&mut preserved), &mut ScriptEndSearch::default());
    *originals = preserved.originals;
    repaired
}

fn repair_scripts<'a>(
    input: &'a str,
    mut preserved: Option<&mut PreservedBodies>,
    search: &mut ScriptEndSearch,
) -> Cow<'a, str> {
    let bytes = input.as_bytes();
    if !memchr::memchr_iter(b'+', bytes).any(|at| {
        bytes
            .get(at.saturating_sub(2)..at + 5)
            .is_some_and(|value| value.eq_ignore_ascii_case(b"ld+json"))
    }) {
        return Cow::Borrowed(input);
    }
    let mut cursor = 0;
    let mut copied = 0;
    let mut output = None;
    let mut unclosed_plain_scripts = Vec::new();
    while let Some(offset) = memchr::memchr(b'<', &bytes[cursor..]) {
        cursor += offset;
        if !opens_a_tag(bytes, cursor) && !matches!(bytes.get(cursor + 1), Some(b'!' | b'?' | b'/')) {
            cursor += 1;
            continue;
        }
        let Some(end) = scan_markup_end(bytes, cursor) else {
            break;
        };
        let name_end = crate::converter::main_helpers::scan_tag_name_end(bytes, cursor + 1);
        let name = &bytes[cursor + 1..name_end];
        if !name.eq_ignore_ascii_case(b"script") {
            cursor = opaque_element_end(bytes, cursor, end, name).unwrap_or(end);
            continue;
        }
        let close = search.find(bytes, end);
        // ~keep A capped search proves no close only when it covered the entire remainder.
        if close.is_none() && bytes.len() - end > MAX_CLOSING_TAG_SCAN {
            break;
        }
        let json_end = is_json_ld_script_open_tag(&input[cursor..end])
            .then(|| complete_json_end(&input[end..]).map_or(end, |length| end + length));
        let repair_end = json_end.filter(|body_end| {
            close.is_none()
                || (*body_end > end && next_script_open(bytes, *body_end, close.unwrap_or(bytes.len())).is_some())
        });
        if let Some(body_end) = repair_end {
            // ~keep Synthesized closes must not capture earlier plain scripts that had no real close.
            for (start, open_end) in std::mem::take(&mut unclosed_plain_scripts) {
                write_repaired_script(input, copied, start..open_end, open_end, None, &mut output);
                copied = open_end;
            }
            write_repaired_script(
                input,
                copied,
                cursor..end,
                body_end,
                preserved.as_deref_mut(),
                &mut output,
            );
            copied = body_end;
            cursor = body_end;
        } else {
            if close.is_none() {
                unclosed_plain_scripts.push((cursor, end));
            }
            cursor = close.unwrap_or(end);
        }
    }
    output.map_or(Cow::Borrowed(input), |mut output| {
        output.push_str(&input[copied..]);
        Cow::Owned(output)
    })
}

#[derive(Default)]
struct ScriptEndSearch {
    end: Option<usize>,
    exhausted: bool,
    #[cfg(test)]
    searches: usize,
}

impl ScriptEndSearch {
    fn find(&mut self, bytes: &[u8], start: usize) -> Option<usize> {
        if self.exhausted || self.end.is_some_and(|end| end >= start) {
            return self.end;
        }
        #[cfg(test)]
        {
            self.searches += 1;
        }
        self.end = find_closing_tag_bytes(bytes, start, b"script");
        self.exhausted = self.end.is_none() && bytes.len() - start <= MAX_CLOSING_TAG_SCAN;
        self.end
    }
}

fn next_script_open(bytes: &[u8], mut cursor: usize, limit: usize) -> Option<usize> {
    while cursor < limit {
        cursor += memchr::memchr(b'<', &bytes[cursor..limit])?;
        if !opens_a_tag(bytes, cursor) && !matches!(bytes.get(cursor + 1), Some(b'!' | b'?' | b'/')) {
            cursor += 1;
            continue;
        }
        let end = scan_markup_end(bytes, cursor)?;
        let name_end = crate::converter::main_helpers::scan_tag_name_end(bytes, cursor + 1);
        let name = &bytes[cursor + 1..name_end];
        if name.eq_ignore_ascii_case(b"script") {
            return Some(cursor);
        }
        cursor = opaque_element_end(bytes, cursor, end, name).unwrap_or(end);
    }
    None
}

fn opaque_element_end(bytes: &[u8], start: usize, end: usize, name: &[u8]) -> Option<usize> {
    if name.eq_ignore_ascii_case(b"plaintext") {
        return Some(bytes.len());
    }
    if [b"svg".as_slice(), b"math"]
        .iter()
        .any(|foreign| name.eq_ignore_ascii_case(foreign))
        && is_self_closing_tag(&bytes[start..end], name)
    {
        return Some(end);
    }
    if [b"svg".as_slice(), b"math", b"template"]
        .iter()
        .any(|opaque| name.eq_ignore_ascii_case(opaque))
    {
        return Some(find_closing_tag_bytes_nested(bytes, end, name).unwrap_or(bytes.len()));
    }
    if [
        b"style".as_slice(),
        b"textarea",
        b"title",
        b"xmp",
        b"iframe",
        b"noscript",
        b"noembed",
        b"noframes",
    ]
    .iter()
    .any(|opaque| name.eq_ignore_ascii_case(opaque))
    {
        return Some(find_closing_tag_bytes(bytes, end, name).unwrap_or(bytes.len()));
    }
    None
}

fn complete_json_end(body: &str) -> Option<usize> {
    let mut values = serde_json::Deserializer::from_str(body).into_iter::<&serde_json::value::RawValue>();
    values.next()?.ok()?;
    Some(values.byte_offset())
}

fn write_repaired_script(
    input: &str,
    copied: usize,
    open_tag: std::ops::Range<usize>,
    body_end: usize,
    preserved: Option<&mut PreservedBodies>,
    output: &mut Option<String>,
) {
    let output = output.get_or_insert_with(|| String::with_capacity(input.len()));
    output.push_str(&input[copied..open_tag.start]);
    // ~keep tl needs matching tag-name case; this changes only a synthesized missing close.
    output.push_str("<script");
    let mut body = &input[open_tag.end..body_end];
    if let Some(preserved) = preserved {
        // ~keep Bind the marker to its repaired script so parser-normalized authored text cannot claim its JSON.
        let (marker, attribute) = preserved.protect_script(body);
        output.push(' ');
        output.push_str(&attribute);
        output.push_str("=\"");
        output.push_str(&marker);
        output.push('"');
        let attributes = &input[open_tag.start + "<script".len()..open_tag.end];
        // ~keep Non-void script slash tokens must not strand an owned body in tl; attribute bytes stay authored.
        let mut copied = 0;
        for slash in standalone_script_slashes(attributes.as_bytes()) {
            output.push_str(&attributes[copied..slash]);
            copied = slash + 1;
        }
        output.push_str(&attributes[copied..]);
        output.push_str(&marker);
        output.push_str("</script>");
        return;
    }
    output.push_str(&input[open_tag.start + "<script".len()..open_tag.end]);
    // ~keep Script references are literal. One metadata decode restores escaped < and & bytes.
    while let Some(at) = memchr::memchr2(b'<', b'&', body.as_bytes()) {
        output.push_str(&body[..at]);
        output.push_str(if body.as_bytes()[at] == b'<' { "&lt;" } else { "&amp;" });
        body = &body[at + 1..];
    }
    output.push_str(body);
    output.push_str("</script>");
}

fn standalone_script_slashes(attributes: &[u8]) -> Vec<usize> {
    let mut cursor = 0;
    let mut slashes = Vec::new();
    while cursor < attributes.len() {
        while attributes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if attributes.get(cursor) == Some(&b'/') {
            slashes.push(cursor);
            cursor += 1;
            continue;
        }
        while attributes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>'))
        {
            cursor += 1;
        }
        while attributes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if attributes.get(cursor) == Some(&b'>') {
            break;
        }
        if attributes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        let Some(end) = script_attribute_value_end(attributes, cursor) else {
            break;
        };
        cursor = end;
    }
    slashes
}

fn script_attribute_value_end(attributes: &[u8], mut cursor: usize) -> Option<usize> {
    while attributes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    let byte = *attributes.get(cursor)?;
    if matches!(byte, b'\'' | b'"') {
        cursor += 1;
        return Some(cursor + memchr::memchr(byte, &attributes[cursor..])? + 1);
    }
    while attributes
        .get(cursor)
        .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
    {
        cursor += 1;
    }
    Some(cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_borrow_explicitly_closed_structured_script_bytes_without_reencoding() {
        let input = r#"<SCRIPT type='application/ld+json'>{"name":"<script> &lt; &amp; 雪"}</SCRIPT><p>tail</p>"#;
        assert!(matches!(
            repair_unclosed_structured_data_scripts(input, None),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn should_ignore_structured_script_shapes_inside_opaque_markup() {
        let script = r#"<script type='application/ld+json'>{"name":"n"}"#;
        for input in [
            format!("<!--{script}--><p>tail</p>"),
            format!("<![CDATA[{script}]]><p>tail</p>"),
            format!("<div title=\"{script}\">tail</div>"),
            format!("<?{script}><p>tail</p>"),
            format!("<plaintext>{script}"),
        ]
        .into_iter()
        .chain(
            [
                "style", "textarea", "title", "xmp", "iframe", "noscript", "noembed", "noframes", "template", "svg",
                "math",
            ]
            .map(|name| format!("<{name}>{script}</{name}><p>tail</p>")),
        ) {
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input, None),
                input,
                "opaque source stays literal"
            );
        }
    }

    #[test]
    fn should_close_a_repaired_script_after_the_valid_json_prefix() {
        let input = r#"<SCRIPT type='APPLICATION/LD+JSON; charset=utf-8'>{"a":"<script> &amp;"} text<p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(input, None),
            r#"<script type='APPLICATION/LD+JSON; charset=utf-8'>{"a":"&lt;script> &amp;amp;"}</script> text<p>tail</p>"#
        );
    }

    #[test]
    fn should_repair_a_structured_script_after_a_literal_less_than_sign() {
        let input = r#"a < b <script type='application/ld+json'>{"name":"n"}<p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(input, None),
            r#"a < b <script type='application/ld+json'>{"name":"n"}</script><p>tail</p>"#
        );
    }

    #[test]
    fn should_repair_structured_data_after_self_closing_foreign_elements() {
        for name in ["svg", "math"] {
            let input = format!(r#"<{name}/><script type='application/ld+json'>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input, None),
                format!(r#"<{name}/><script type='application/ld+json'>{{"name":"n"}}</script><p>tail</p>"#)
            );
        }
    }

    #[test]
    fn should_repair_real_scripts_after_abrupt_and_bang_closed_comments() {
        for comment in ["<!-->", "<!--->", "<!--safe--!>"] {
            let input = format!(r#"{comment}<script type='application/ld+json'>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input, None),
                format!(r#"{comment}<script type='application/ld+json'>{{"name":"n"}}</script><p>tail</p>"#)
            );
        }
    }

    #[test]
    fn should_ignore_structured_script_shapes_in_bogus_comments() {
        for opener in ["</ ", "<! "] {
            let input = format!(r#"{opener}<script type='application/ld+json'>>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(repair_unclosed_structured_data_scripts(&input, None), input);
        }
        let unterminated = r#"<!--!><script type='application/ld+json'>{"name":"n"}<p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(unterminated, None),
            unterminated
        );
    }

    #[test]
    fn should_repair_scripts_after_the_html_bogus_comment_end_of_cdata_shaped_markup() {
        for suffix in ["<p>tail</p>", "<p>tail</p>]]><p>after</p>"] {
            let input = format!(r#"<![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}{suffix}"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input, None),
                format!(r#"<![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}</script>{suffix}"#)
            );
        }
        for name in ["svg", "math"] {
            let input = format!(
                r#"<{name}><![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}]]></{name}><p>tail</p>"#
            );
            assert_eq!(repair_unclosed_structured_data_scripts(&input, None), input);
        }
    }

    #[test]
    fn should_leave_invalid_json_after_an_empty_structured_script() {
        for body in ["not JSON", "{invalid}", "[1,]", "{\"key\":"] {
            let input = format!("<script type='application/ld+json'>{body}<p>tail</p>");
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input, None),
                format!("<script type='application/ld+json'></script>{body}<p>tail</p>")
            );
        }
    }

    #[test]
    fn should_search_an_absent_script_close_only_once_for_many_unclosed_scripts() {
        let input = "<script>".repeat(20_000);
        let mut search = ScriptEndSearch::default();
        for start in (8..input.len()).step_by(8) {
            assert_eq!(search.find(input.as_bytes(), start), None);
        }
        assert_eq!(search.searches, 1);
    }

    #[test]
    fn should_reuse_a_later_script_close_until_the_cursor_passes_it() {
        let input = "<script><script></script><script></script>";
        let mut search = ScriptEndSearch::default();
        assert_eq!(search.find(input.as_bytes(), 8), Some(25));
        assert_eq!(search.find(input.as_bytes(), 16), Some(25));
        assert_eq!(search.searches, 1);
        assert_eq!(search.find(input.as_bytes(), 33), Some(42));
        assert_eq!(search.searches, 2);
    }

    #[test]
    fn should_leave_structured_script_shapes_in_nested_and_unclosed_opaque_elements() {
        for name in ["svg", "math", "template"] {
            let input =
                format!(r#"<{name}><{name}></{name}><script type='application/ld+json'>{{"a":1}}</{name}><p>tail</p>"#);
            assert_eq!(repair_unclosed_structured_data_scripts(&input, None), input);
        }
        for name in [
            "svg", "math", "template", "style", "textarea", "title", "xmp", "iframe", "noscript", "noembed", "noframes",
        ] {
            let input = format!(r#"<{name}><script type='application/ld+json'>{{"a":1}}<p>tail</p>"#);
            assert_eq!(repair_unclosed_structured_data_scripts(&input, None), input);
        }
    }

    #[test]
    fn should_parse_a_deep_json_value_without_native_recursion_or_losing_the_tail() {
        let json = format!("{}0{}", "[".repeat(40_000), "]".repeat(40_000));
        let input = format!("<script type='application/ld+json'>{json}<p>tail</p>");
        assert_eq!(complete_json_end(&json), Some(json.len()));
        assert_eq!(
            repair_unclosed_structured_data_scripts(&input, None),
            format!("<script type='application/ld+json'>{json}</script><p>tail</p>")
        );
    }

    #[test]
    fn should_find_a_later_script_after_literal_less_than_text_but_not_inside_json() {
        let input = r#"<script type='application/ld+json'>{"a":"<script>"} x < y <script>z</script><p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(input, None),
            r#"<script type='application/ld+json'>{"a":"&lt;script>"}</script> x < y <script>z</script><p>tail</p>"#
        );
    }

    #[test]
    fn should_leave_a_missing_plain_script_unchanged_without_any_structured_repair() {
        let input = r"<script>x<p>a</p><div title='ld+json'>tail</div>";
        assert!(matches!(
            repair_unclosed_structured_data_scripts(input, None),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn should_repair_many_plain_script_openers_with_one_end_tag_search() {
        let plain = "<script>".repeat(2_000);
        let input = format!(r#"{plain}<script type='application/ld+json'>{{"a":1}}<p>tail</p>"#);
        let mut search = ScriptEndSearch::default();
        let output = repair_scripts(&input, None, &mut search);
        assert_eq!(
            output,
            format!(
                r#"{}<script type='application/ld+json'>{{"a":1}}</script><p>tail</p>"#,
                "<script></script>".repeat(2_000)
            )
        );
        assert_eq!(search.searches, 1);
    }
    #[test]
    fn should_leave_closed_invalid_structured_script_bytes_opaque() {
        for body in ["not JSON<script>x", "{invalid}<script>x", "{\"bad\":\"<script>"] {
            let input = format!("<script type='application/ld+json'>{body}</script><p>tail</p>");
            assert!(
                matches!(repair_unclosed_structured_data_scripts(&input, None), Cow::Borrowed(_)),
                "{input}"
            );
        }
    }

    #[test]
    fn should_allocate_a_repaired_body_marker_without_overwriting_existing_or_literal_markers() {
        let json = r#"{"a":"<script> &lt; &amp; 雪"}"#;
        let input = format!(
            "<p>h2m_inert_body_0_end h2m_inert_body_&#49;_end</p><script type='application/ld+json'>{json}<p>tail</p>"
        );
        let mut originals = std::collections::HashMap::from([("h2m_inert_body_2_end".into(), "earlier".into())]);
        let repaired = repair_unclosed_structured_data_scripts(&input, Some(&mut originals));
        assert_eq!(originals.len(), 2);
        assert_eq!(originals["h2m_inert_body_2_end"], "earlier");
        assert_eq!(originals["h2m_script_body_3_end"], json);
        assert_eq!(
            repaired,
            "<p>h2m_inert_body_0_end h2m_inert_body_&#49;_end</p><script data-h2m-script-3=\"h2m_script_body_3_end\" type='application/ld+json'>h2m_script_body_3_end</script><p>tail</p>"
        );
    }

    #[test]
    fn should_not_treat_later_script_openers_in_opaque_contexts_as_a_repair_boundary() {
        for name in ["template", "svg", "math", "noscript", "style", "iframe"] {
            let input =
                format!("<script type='application/ld+json'>{{\"a\":1}}<{name}><script>x</script></{name}><p>tail</p>");
            assert!(
                matches!(repair_unclosed_structured_data_scripts(&input, None), Cow::Borrowed(_)),
                "{name}"
            );
        }
    }
}
