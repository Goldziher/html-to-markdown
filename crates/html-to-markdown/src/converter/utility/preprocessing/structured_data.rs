//! Recover visible markup after structured data with no script end tag.

use std::borrow::Cow;

use super::inert::scan_markup_end;
use super::markup::is_json_ld_script_open_tag;
use super::raw_text::{
    MAX_CLOSING_TAG_SCAN, find_closing_tag_bytes, find_closing_tag_bytes_nested, is_self_closing_tag, opens_a_tag,
};

/// Recover a missing JSON-LD script end tag without consuming following markup.
pub fn repair_unclosed_structured_data_scripts(input: &str) -> Cow<'_, str> {
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
    let mut no_script_close = false;
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
        if name.eq_ignore_ascii_case(b"script") {
            let close = (!no_script_close)
                .then(|| find_closing_tag_bytes(bytes, end, b"script"))
                .flatten();
            // ~keep A capped search proves no close only when it covered the entire remainder.
            if close.is_none() && bytes.len() - end > MAX_CLOSING_TAG_SCAN {
                break;
            }
            no_script_close |= close.is_none();
            if close.is_none() && is_json_ld_script_open_tag(&input[cursor..end]) {
                let body_end = complete_json_end(&input[end..]).map_or(end, |length| end + length);
                write_repaired_script(input, copied, cursor, end, body_end, &mut output);
                copied = body_end;
                cursor = body_end;
            } else {
                cursor = close.unwrap_or(end);
            }
        } else {
            cursor = opaque_element_end(bytes, cursor, end, name).unwrap_or(end);
        }
    }
    output.map_or(Cow::Borrowed(input), |mut output| {
        output.push_str(&input[copied..]);
        Cow::Owned(output)
    })
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
    start: usize,
    open_end: usize,
    body_end: usize,
    output: &mut Option<String>,
) {
    let output = output.get_or_insert_with(|| String::with_capacity(input.len()));
    output.push_str(&input[copied..start]);
    // ~keep tl needs matching tag-name case; this changes only a synthesized missing close.
    output.push_str("<script");
    output.push_str(&input[start + "<script".len()..open_end]);
    // ~keep Script references are literal. One metadata decode restores escaped < and & bytes.
    let mut body = &input[open_end..body_end];
    while let Some(at) = memchr::memchr2(b'<', b'&', body.as_bytes()) {
        output.push_str(&body[..at]);
        output.push_str(if body.as_bytes()[at] == b'<' { "&lt;" } else { "&amp;" });
        body = &body[at + 1..];
    }
    output.push_str(body);
    output.push_str("</script>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_borrow_explicitly_closed_structured_script_bytes_without_reencoding() {
        let input = r#"<SCRIPT type='application/ld+json'>{"name":"<script> &lt; &amp; 雪"}</SCRIPT><p>tail</p>"#;
        assert!(matches!(
            repair_unclosed_structured_data_scripts(input),
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
                repair_unclosed_structured_data_scripts(&input),
                input,
                "opaque source stays literal"
            );
        }
    }

    #[test]
    fn should_close_a_repaired_script_after_the_valid_json_prefix() {
        let input = r#"<SCRIPT type='APPLICATION/LD+JSON; charset=utf-8'>{"a":"<script> &amp;"} text<p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(input),
            r#"<script type='APPLICATION/LD+JSON; charset=utf-8'>{"a":"&lt;script> &amp;amp;"}</script> text<p>tail</p>"#
        );
    }

    #[test]
    fn should_repair_a_structured_script_after_a_literal_less_than_sign() {
        let input = r#"a < b <script type='application/ld+json'>{"name":"n"}<p>tail</p>"#;
        assert_eq!(
            repair_unclosed_structured_data_scripts(input),
            r#"a < b <script type='application/ld+json'>{"name":"n"}</script><p>tail</p>"#
        );
    }

    #[test]
    fn should_repair_structured_data_after_self_closing_foreign_elements() {
        for name in ["svg", "math"] {
            let input = format!(r#"<{name}/><script type='application/ld+json'>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input),
                format!(r#"<{name}/><script type='application/ld+json'>{{"name":"n"}}</script><p>tail</p>"#)
            );
        }
    }

    #[test]
    fn should_repair_real_scripts_after_abrupt_and_bang_closed_comments() {
        for comment in ["<!-->", "<!--->", "<!--safe--!>"] {
            let input = format!(r#"{comment}<script type='application/ld+json'>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input),
                format!(r#"{comment}<script type='application/ld+json'>{{"name":"n"}}</script><p>tail</p>"#)
            );
        }
    }

    #[test]
    fn should_ignore_structured_script_shapes_in_bogus_comments() {
        for opener in ["</ ", "<! "] {
            let input = format!(r#"{opener}<script type='application/ld+json'>>{{"name":"n"}}<p>tail</p>"#);
            assert_eq!(repair_unclosed_structured_data_scripts(&input), input);
        }
        let unterminated = r#"<!--!><script type='application/ld+json'>{"name":"n"}<p>tail</p>"#;
        assert_eq!(repair_unclosed_structured_data_scripts(unterminated), unterminated);
    }

    #[test]
    fn should_repair_scripts_after_the_html_bogus_comment_end_of_cdata_shaped_markup() {
        for suffix in ["<p>tail</p>", "<p>tail</p>]]><p>after</p>"] {
            let input = format!(r#"<![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}{suffix}"#);
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input),
                format!(r#"<![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}</script>{suffix}"#)
            );
        }
        for name in ["svg", "math"] {
            let input = format!(
                r#"<{name}><![CDATA[x > <script type='application/ld+json'>{{"name":"n"}}]]></{name}><p>tail</p>"#
            );
            assert_eq!(repair_unclosed_structured_data_scripts(&input), input);
        }
    }

    #[test]
    fn should_leave_invalid_json_after_an_empty_structured_script() {
        for body in ["not JSON", "{invalid}", "[1,]", "{\"key\":"] {
            let input = format!("<script type='application/ld+json'>{body}<p>tail</p>");
            assert_eq!(
                repair_unclosed_structured_data_scripts(&input),
                format!("<script type='application/ld+json'></script>{body}<p>tail</p>")
            );
        }
    }
}
