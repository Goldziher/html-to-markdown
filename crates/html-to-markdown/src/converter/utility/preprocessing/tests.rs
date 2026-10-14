use std::borrow::Cow;

use super::markup::second_pass;
use super::raw_text::{FirstPass, PassRead};
use super::{
    find_closing_tag_bytes, find_closing_tag_bytes_nested, find_tag_end, normalize_bogus_comment_endings,
    normalize_menu_elements, normalize_split_closing_tags, normalize_unclosed_list_items,
    restore_preserved_menu_elements, sanitize_markdown_url, strip_bogus_comments, strip_hidden_elements,
    unwrap_kept_inert_elements,
};
use crate::options::HiddenContent;

#[test]
fn should_find_tag_end_outside_quotes_from_the_requested_byte() {
    let cases: &[(&[u8], usize, Option<usize>)] = &[
        (b"", 0, None),
        (b">", 0, Some(1)),
        (b">", 1, None),
        (b">", usize::MAX, None),
        (b"a>b", 0, Some(2)),
        (b"'>' >", 0, Some(5)),
        (b"\"'>'\">", 0, Some(6)),
        (b"'\" >' >", 0, Some(7)),
        (b"\"unterminated >", 0, None),
        (b"\"abc\">", 1, None),
        (b"\\\">", 0, None),
        (b"\xff'>\0'\xfe>", 0, Some(7)),
    ];
    for &(input, start, expected) in cases {
        assert_eq!(find_tag_end(input, start), expected, "input: {input:?}, start: {start}");
    }
}

#[test]
fn should_borrow_html_without_bogus_comments() {
    for input in [
        "",
        "<",
        "é日 text<",
        "<p title='a<?b>c'>é日</p><",
        "<!--[if gte mso 9]><?ignored?><![endif]--><p>kept</p>",
        "<![CDATA[<?not markup>]]><p>kept</p>",
        "<!DOCTYPE html><p>kept</p>",
        "<!-- unterminated <?ignored>",
        "<![CDATA[unterminated <?ignored>",
    ] {
        let actual = strip_bogus_comments(input);
        assert_eq!(actual.as_ref(), input, "input: {input}");
        assert!(matches!(actual, Cow::Borrowed(_)), "input: {input}");
    }
}

#[test]
fn should_remove_bogus_comments_at_utf8_and_markup_boundaries() {
    for (input, expected) in [
        ("é<?a><!b></3>日<", "é日<"),
        ("<p title='a<?b>c'>é</p><?drop>日", "<p title='a<?b>c'>é</p>日"),
        ("<!-- <?kept> --><?drop>日", "<!-- <?kept> -->日"),
        ("<![CDATA[<?kept>]]><?drop>日", "<![CDATA[<?kept>]]>日"),
        ("é<?'quoted>日", "é日"),
        ("é<?", "é"),
        ("é<!", "é"),
        ("é</", "é"),
    ] {
        let actual = strip_bogus_comments(input);
        assert_eq!(actual.as_ref(), expected, "input: {input}");
        assert!(matches!(actual, Cow::Owned(_)), "input: {input}");
    }
}

#[test]
fn should_distinguish_bogus_markers_from_recognized_prefixes() {
    for (input, expected) in [
        ("é /!? 日", "é /!? 日"),
        ("<!DOC>é", "é"),
        ("<!DOCTYPE", "<!DOCTYPE"),
        ("<!dOcTyPe html><p>é</p>", "<!dOcTyPe html><p>é</p>"),
        ("<![cdata[x]]>é", "é"),
        ("<![CDATA[<?x>]]>é", "<![CDATA[<?x>]]>é"),
        ("<!--<?x>-->é", "<!--<?x>-->é"),
        ("</é>日", "日"),
        ("</a>", "</a>"),
        ("</A>", "</A>"),
        ("<p title='<!bad><?bad></3>'>é</p>", "<p title='<!bad><?bad></3>'>é</p>"),
    ] {
        let actual = strip_bogus_comments(input);
        assert_eq!(actual.as_ref(), expected, "input: {input}");
        assert_eq!(matches!(actual, Cow::Borrowed(_)), input == expected, "input: {input}");
    }
}

#[test]
fn normalize_bogus_comment_endings_leaves_well_formed_comment_unchanged() {
    let input = "<p>A</p><!-- foo --><p>B</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_bogus_comment_endings_rewrites_triple_dash_close() {
    let input = "<!-- foo --->";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<!-- foo -->");
}

#[test]
fn normalize_bogus_comment_endings_rewrites_four_dash_close() {
    let input = "<!-- foo ---->";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<!-- foo -->");
}

#[test]
fn normalize_bogus_comment_endings_preserves_content_after_comment() {
    let input = "<h1>One</h1><!-- /// ---><p>Two</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<h1>One</h1><!-- /// --><p>Two</p>");
}

#[test]
fn normalize_bogus_comment_endings_handles_multiple_bogus_comments() {
    let input = "<p>A</p><!-- x ---><p>B</p><!-- y ----><p>C</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<p>A</p><!-- x --><p>B</p><!-- y --><p>C</p>");
}

#[test]
fn normalize_bogus_comment_endings_rewrites_abrupt_closing_of_empty_comment() {
    // ~keep `<!-->` and `<!--->` are HTML5's "abrupt-closing-of-empty-comment"
    // ~keep quirk: legal, empty comments whose open and close share dashes.
    let input = "<p>Before</p><!--><p>After</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<p>Before</p><!----><p>After</p>");

    let input = "<p>Before</p><!---><p>After</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), "<p>Before</p><!----><p>After</p>");
}

#[test]
fn normalize_bogus_comment_endings_handles_no_comments() {
    let input = "<p>Just a paragraph</p>";
    let result = normalize_bogus_comment_endings(input);
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_bogus_comment_endings_empty_input() {
    let result = normalize_bogus_comment_endings("");
    assert_eq!(result.as_ref(), "");
}

#[test]
fn should_preserve_hidden_style_markers_without_false_negative_skips() {
    let cases = [
        ("<ul><li>a<li>b<li>c</ul>\n", "<ul><li>a<li>b<li>c</ul>\n", false),
        ("é<div>visible</div>", "é<div>visible</div>", false),
        ("<b HIDDEN>x</b>", "", true),
        ("<b STYLE='DISPLAY:NONE'>x</b>", "", true),
        ("<b style='display:none'>x</b>", "", true),
        ("<b title='HIDDEN STYLE'>x</b>", "<b title='HIDDEN STYLE'>x</b>", false),
    ];
    for (input, expected, owned) in cases {
        let actual = strip_hidden_elements(input);
        assert_eq!(actual, expected, "input: {input:?}");
        assert_eq!(matches!(actual, Cow::Owned(_)), owned, "input: {input:?}");
    }
}

#[test]
fn should_preserve_hidden_element_scan_boundaries_and_borrowing() {
    let cases = [
        ("é text<", "é text<", false),
        ("<span hidden", "<span hidden", false),
        ("<<span hidden>x</span>終", "<終", true),
        ("< span hidden>x</ span>", "< span hidden>x</ span>", false),
        // ~keep A `<` inside a tag is part of an attribute value, not the start of an element: the
        // ~keep value stays whole. The pass used to cut the "element" out of the value, and the
        // ~keep scan from every such `<` is what made a long tag quadratic (#765).
        (
            "<div title='<span hidden>x</span>'>ok</div>",
            "<div title='<span hidden>x</span>'>ok</div>",
            false,
        ),
        ("<i hidden>x</i><b hidden>y</b>z", "z", true),
        (
            "<div data-hidden='true'>x</div>",
            "<div data-hidden='true'>x</div>",
            false,
        ),
        (
            "<div style='font-size:0'><b style='font-size:12px'>x</b></div>",
            "<div style='font-size:0'><b style='font-size:12px'>x</b></div>",
            false,
        ),
    ];
    for (input, expected, owned) in cases {
        let actual = strip_hidden_elements(input);
        assert_eq!(actual, expected, "input: {input:?}");
        assert_eq!(matches!(actual, Cow::Owned(_)), owned, "input: {input:?}");
    }
}

#[test]
fn should_preserve_split_closing_tag_boundaries_and_borrowing() {
    let cases = [
        ("é\ntext</", "é\ntext</", false),
        ("é\ntext<", "é\ntext<", false),
        ("\n<", "\n<", false),
        ("\n</</a\n>", "\n</</a>", true),
        ("\n</a!></b\r\n >", "\n</a!></b>", true),
        ("é</custom-42\n>終</B\n>", "é</custom-42>終</B>", true),
        ("\n</a ", "\n</a ", false),
        ("</a\r>", "</a\r>", false),
        ("\n</a\r>", "\n</a>", true),
        ("\n</ a\n>", "\n</ a\n>", false),
        ("<!-- </a\n> -->", "<!-- </a> -->", true),
        ("<x a='</b\n>'>", "<x a='</b>'>", true),
    ];
    for (input, expected, owned) in cases {
        let actual = normalize_split_closing_tags(input);
        assert_eq!(actual, expected, "input: {input:?}");
        assert_eq!(matches!(actual, Cow::Owned(_)), owned, "input: {input:?}");
    }
}

#[test]
fn normalize_split_closing_tags_collapses_newline_before_close_bracket() {
    let input = "<a href=\"#x\">text</a\n>";
    let result = normalize_split_closing_tags(input);
    assert_eq!(result.as_ref(), "<a href=\"#x\">text</a>");
}

#[test]
fn normalize_split_closing_tags_collapses_indented_newline_before_close_bracket() {
    let input = "<a href=\"#x\">text</a\n  >";
    let result = normalize_split_closing_tags(input);
    assert_eq!(result.as_ref(), "<a href=\"#x\">text</a>");
}

#[test]
fn normalize_split_closing_tags_leaves_well_formed_closing_tags_unchanged() {
    let input = "<a href=\"#x\">text</a>";
    let result = normalize_split_closing_tags(input);
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_split_closing_tags_handles_multiple_split_closing_tags() {
    let input = "<li><a href=\"#a\">A</a\n  >\n<a href=\"#b\">B</a\n>";
    let result = normalize_split_closing_tags(input);
    assert_eq!(result.as_ref(), "<li><a href=\"#a\">A</a>\n<a href=\"#b\">B</a>");
}

#[test]
fn normalize_split_closing_tags_does_not_collapse_inline_whitespace() {
    let input = "<a href=\"#x\">text</a >";
    let result = normalize_split_closing_tags(input);
    // ~keep A space before > is actually valid HTML and tl handles it fine.
    // ~keep We must not touch it to avoid over-normalising.
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_split_closing_tags_empty_input() {
    let result = normalize_split_closing_tags("");
    assert_eq!(result.as_ref(), "");
}

#[test]
fn normalize_menu_elements_rewrites_open_and_close_tags() {
    let input = "<ul><li><MeNu class='commands'><li>x</li></MENU></li></ul>";
    assert_eq!(
        normalize_menu_elements(input, false),
        "<ul><li><ul class='commands'><li>x</li></ul></li></ul>"
    );
}

#[test]
fn restore_preserved_menu_elements_restores_nested_markers_only() {
    let input = concat!(
        "<ul data-html-to-markdown-preserved-menu=\"\"><li>",
        "<ul><li>plain</li></ul>",
        "<ul data-html-to-markdown-preserved-menu=\"\"><li>nested</li></ul>",
        "</li></ul>"
    );
    assert_eq!(
        restore_preserved_menu_elements(input),
        "<menu><li><ul><li>plain</li></ul><menu><li>nested</li></menu></li></menu>"
    );
}

#[test]
fn normalize_menu_elements_ignores_opaque_and_attribute_content() {
    let input = "<!-- <menu><li>x</li></menu> --><div data-x='<menu><li>x</li></menu>'>x</div>";
    assert!(matches!(normalize_menu_elements(input, false), Cow::Borrowed(_)));
}

#[test]
fn normalize_menu_elements_leaves_text_only_menu_unchanged() {
    let input = "<menu>b</menu>";
    assert!(matches!(normalize_menu_elements(input, false), Cow::Borrowed(_)));
}

#[test]
fn normalize_menu_elements_leaves_standalone_list_menu_unchanged() {
    let input = "<menu><li>x</li></menu>";
    assert!(matches!(normalize_menu_elements(input, false), Cow::Borrowed(_)));
}

#[test]
fn sanitize_markdown_url_extracts_scheme_relative_markdown_like_url() {
    let input = "//[p1.zemanta.com/v2/p/ns/45625/PAGE\\_VIEW/](http://p1.zemanta.com/v2/p/ns/45625/PAGE_VIEW/)";
    let sanitized = sanitize_markdown_url(input);
    assert_eq!(sanitized, "http://p1.zemanta.com/v2/p/ns/45625/PAGE_VIEW/");
}

#[test]
fn sanitize_markdown_url_extracts_standard_markdown_like_url() {
    let input = "[label](https://example.com/path?q=1)";
    let sanitized = sanitize_markdown_url(input);
    assert_eq!(sanitized, "https://example.com/path?q=1");
}

#[test]
fn sanitize_markdown_url_leaves_normal_urls_unchanged() {
    let input = "https://example.com/normal";
    let sanitized = sanitize_markdown_url(input);
    assert_eq!(sanitized, input);
}

#[test]
fn normalize_unclosed_list_items_leaves_well_formed_list_unchanged() {
    let input = "<ul><li>A</li><li>B</li></ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(result.as_ref(), input);
}

#[test]
fn normalize_unclosed_list_items_closes_unclosed_li_before_next_li() {
    let input = "<ul><li>A<li>B</ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(result.as_ref(), "<ul><li>A</li><li>B</li></ul>");
}

#[test]
fn normalize_unclosed_list_items_closes_chain_of_unclosed_li() {
    let input = "<ul><li>A<li>B<li>C</ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(result.as_ref(), "<ul><li>A</li><li>B</li><li>C</li></ul>");
}

#[test]
fn normalize_unclosed_list_items_does_not_modify_input_without_list_items() {
    let input = "<p>Hello</p><div>World</div>";
    let result = normalize_unclosed_list_items(input);
    assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
}

#[test]
fn normalize_unclosed_list_items_handles_nested_list_correctly() {
    let input = "<ul><li>Outer<ul><li>Inner A<li>Inner B</ul><li>Outer B</ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(
        result.as_ref(),
        "<ul><li>Outer<ul><li>Inner A</li><li>Inner B</li></ul></li><li>Outer B</li></ul>"
    );
}

#[test]
fn normalize_unclosed_list_items_handles_dt_and_dd() {
    let input = "<dl><dt>Term A<dd>Def A<dt>Term B<dd>Def B</dl>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(
        result.as_ref(),
        "<dl><dt>Term A</dt><dd>Def A</dd><dt>Term B</dt><dd>Def B</dd></dl>"
    );
}

#[test]
fn normalize_unclosed_list_items_does_not_touch_content_in_pre() {
    let input = "<ul><li>A<pre><li>not-a-list-item</pre><li>B</ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(
        result.as_ref(),
        "<ul><li>A<pre><li>not-a-list-item</pre></li><li>B</li></ul>"
    );
}

#[test]
fn normalize_unclosed_list_items_skips_html_comments() {
    let input = "<ul><li>A<!-- <li>comment --><li>B</ul>";
    let result = normalize_unclosed_list_items(input);
    assert_eq!(result.as_ref(), "<ul><li>A<!-- <li>comment --></li><li>B</li></ul>");
}

#[test]
fn normalize_unclosed_list_items_empty_input() {
    let result = normalize_unclosed_list_items("");
    assert_eq!(result.as_ref(), "");
}

#[test]
fn strip_hidden_elements_removes_display_none_element() {
    let input = r#"<p>visible</p><div style="display:none">secret</div><p>also visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p><p>also visible</p>");
}

#[test]
fn strip_hidden_elements_removes_visibility_hidden_element() {
    let input = r#"<p>visible</p><span style="visibility:hidden">secret</span><p>also visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p><p>also visible</p>");
}

#[test]
fn strip_hidden_elements_tolerates_whitespace_around_declaration() {
    let input = r#"<div style="display : none">secret</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_tolerates_mixed_case_declaration() {
    let input = r#"<div style="Display:NONE">secret</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_tolerates_important_flag() {
    let input = r#"<div style="display:none !important">secret</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_matches_declaration_among_others() {
    let input = r#"<div style="color:red; display:none; margin:0">secret</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_leaves_visible_style_untouched() {
    let input = r#"<div style="color:red; display:block">visible</div>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), input);
}

#[test]
fn strip_hidden_elements_removes_nested_content_inside_hidden_parent() {
    let input = r#"<div style="display:none"><p>secret</p><span>also secret</span></div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_removes_parent_that_nests_the_same_tag_name() {
    let input = r#"<p>A<div style="display:none">S1<div>S2</div>LEAKED</div>B</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>AB</p>");
}

#[test]
fn strip_hidden_elements_removes_deeply_nested_same_tag_subtree() {
    let input = r#"<div style="display:none">L0<div>L1<div>L2</div>T2</div>T1</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_matches_closing_tag_case_insensitively() {
    let input = r#"<DIV style="display:none">x<div>y</div>z</DIV><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_does_not_match_a_tag_with_a_longer_name() {
    let input = r#"<div style="display:none">a<divider>b</divider>c</div><p>visible</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>");
}

#[test]
fn strip_hidden_elements_drops_only_the_open_tag_when_never_closed() {
    let input = r#"<p>visible</p><div style="display:none">dangling"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>visible</p>dangling");
}

#[test]
fn find_closing_tag_bytes_nested_skips_inner_same_name_element() {
    let html = b"<div>a<div>b</div>c</div>tail";
    let start = "<div>".len();
    let end = find_closing_tag_bytes_nested(html, start, b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_ignores_self_closing_inner_tag() {
    let html = b"<div>a<div/>b</div>tail";
    let start = "<div>".len();
    let end = find_closing_tag_bytes_nested(html, start, b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_returns_none_for_unbalanced_input() {
    let html = b"<div>a<div>b</div>";
    assert_eq!(find_closing_tag_bytes_nested(html, "<div>".len(), b"div"), None);
}

#[test]
fn find_closing_tag_bytes_nested_ignores_close_tag_inside_a_comment() {
    let html = b"<div>a<!-- </div> -->b</div>tail";
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_ignores_open_tag_inside_a_comment() {
    let html = b"<div>a<!-- <div> -->b</div>tail";
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_ignores_close_tag_inside_a_quoted_attribute() {
    let html = br#"<div>a<span title="</div>">x</span>b</div>tail"#;
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_ignores_close_tag_inside_a_raw_text_body() {
    let html = br#"<div>a<script>var s = "</div>";</script>b</div>tail"#;
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_ignores_close_tag_inside_a_cdata_section() {
    let html = b"<div>a<![CDATA[ </div> ]]>b</div>tail";
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn find_closing_tag_bytes_nested_does_not_skip_over_a_literal_less_than_in_text() {
    let html = b"<div>a < b</div>tail";
    let end = find_closing_tag_bytes_nested(html, "<div>".len(), b"div").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

#[test]
fn strip_hidden_elements_ignores_a_close_tag_written_inside_a_comment() {
    let input = r#"<p>A</p><div style="display:none">SECRET<!-- </div> -->MORE</div><p>B</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), "<p>A</p><p>B</p>");
}

#[test]
fn strip_hidden_elements_keeps_sibling_when_a_comment_holds_an_unbalanced_open_tag() {
    let input = r#"<div id="wrap"><div style="display:none">S<!-- <div> --></div><p>VISIBLE</p></div><p>after</p>"#;
    let result = strip_hidden_elements(input);
    assert_eq!(result.as_ref(), r#"<div id="wrap"><p>VISIBLE</p></div><p>after</p>"#);
}

#[test]
fn find_closing_tag_bytes_stays_first_match_for_raw_text_elements() {
    let html = br#"<script>var s = "<script>";</script>tail"#;
    let start = "<script>".len();
    let end = find_closing_tag_bytes(html, start, b"script").expect("closing tag not found");
    assert_eq!(&html[end..], b"tail");
}

/// The reads of a pass over one page that a test counts.
#[derive(Default)]
struct Reads {
    /// The `<` characters that the first pass read.
    markup: usize,
    /// The searches for an end tag and for the end of a comment: each can read to the end of
    /// the page.
    searches: usize,
    /// The bytes that the reads of a tag went over.
    tag_bytes: usize,
    /// The places where the reads of a tag stopped to look at a byte.
    tag_stops: usize,
}

impl Reads {
    fn count(&mut self, read: PassRead) {
        match read {
            PassRead::Markup => self.markup += 1,
            PassRead::EndTagSearch | PassRead::CommentEndSearch => self.searches += 1,
            PassRead::TagRead(bytes) => self.tag_bytes += bytes,
            PassRead::TagStops(stops) => self.tag_stops += stops,
        }
    }
}

fn first_pass_reads(input: &str) -> Reads {
    let mut reads = Reads::default();
    let (stripped, _) = FirstPass::new(input, |read| reads.count(read)).run();
    assert_eq!(stripped.as_ref(), input, "nothing in the page has an end");
    reads
}

fn second_pass_reads(input: &str) -> Reads {
    let mut reads = Reads::default();
    drop(second_pass(input, |read| reads.count(read)));
    reads
}

#[test]
fn the_first_pass_searches_once_for_an_end_that_the_page_does_not_have() {
    const STARTS: usize = 2_000;
    // ~keep The `>` in the comment unit keeps the comments before the last `>` of the page.
    for unit in ["<textarea>x ", "<script>x ", "<style>x ", "<!-- x> "] {
        let html = format!("<p>start</p>{}", unit.repeat(STARTS));
        let reads = first_pass_reads(&html);
        assert_eq!(reads.searches, 1, "{unit:?}: one search, not one for each start");
        assert!(reads.markup <= STARTS + 2, "{unit:?}: {} reads", reads.markup);
    }
    let mixed = format!("<p>start</p>{}", "<textarea>x <script>x <style>x ".repeat(STARTS));
    assert_eq!(first_pass_reads(&mixed).searches, 3, "one search for each name");
}

#[test]
fn the_first_pass_reads_nothing_after_the_last_tag_end_of_the_page() {
    for unit in ["<textarea x ", "<script x ", "<a x ", "<svg x ", "</svg x "] {
        let html = format!("<p>start</p>{}", unit.repeat(2_000));
        let reads = first_pass_reads(&html);
        assert_eq!((reads.markup, reads.searches), (2, 0), "{unit:?}");
    }
}

#[test]
fn the_second_pass_searches_once_for_an_end_that_the_page_does_not_have() {
    const STARTS: usize = 2_000;
    for unit in ["<script>x ", "<style>x ", "<!-- x> "] {
        let html = format!("<p>start</p>{}", unit.repeat(STARTS));
        let reads = second_pass_reads(&html);
        assert_eq!(reads.searches, 1, "{unit:?}: one search, not one for each start");
    }
    let mixed = format!("<p>start</p>{}", "<script>x <style>x ".repeat(STARTS));
    assert_eq!(second_pass_reads(&mixed).searches, 2, "one search for each name");
}

/// Start tags with a `>` in each attribute value: the read of the first one goes to the end
/// of the page, because the quote after its last `>` has no partner.
const TAGS_THAT_NEVER_END: [&str; 5] = [
    "<a x='>'",
    "<BR x='>'",
    "<textarea x='>'",
    "<script x='>'",
    "<style x='>'",
];

#[test]
fn a_pass_reads_a_tag_that_never_ends_once() {
    for unit in TAGS_THAT_NEVER_END {
        let html = format!("<p>start</p>{}", unit.repeat(2_000));
        for (pass, reads) in [("first", first_pass_reads(&html)), ("second", second_pass_reads(&html))] {
            assert!(
                reads.tag_bytes <= 2 * html.len(),
                "{unit:?}, {pass} pass: the tag reads went over {} bytes of a page of {}",
                reads.tag_bytes,
                html.len()
            );
            assert!(
                reads.searches <= 1,
                "{unit:?}, {pass} pass: {} searches",
                reads.searches
            );
        }
    }
}

#[test]
fn an_end_tag_with_no_bracket_ends_no_raw_text_element() {
    let page = format!("<script>{}", "</script x ".repeat(3));
    assert_eq!(
        find_closing_tag_bytes(page.as_bytes(), "<script>".len(), b"script"),
        None
    );
    let page = format!("<script></scriptx>{}>tail", "</script x ".repeat(3));
    let end = find_closing_tag_bytes(page.as_bytes(), "<script>".len(), b"script").expect("an end tag");
    assert_eq!(&page[end..], "tail");
}

/// The read of a tag ends at the `>` that is in no attribute value. A quote opens a value
/// only after `=` and white space; a value with no quotes ends at white space.
#[test]
fn a_tag_is_read_to_the_bracket_that_is_in_no_value() {
    for tag in [
        "<a x='>' y>",
        "<a x=\">\" y>",
        "<a x=\"a'b>\" y>",
        "<a x = '>' y>",
        "<a x=\t\n\x0C\r '>' y>",
        "<a x=it's y='>'>",
        "<a href=/s?q='tis y='>'>",
        "<a x=y= z='>'>",
        "<a x== y='>'>",
        "<a 'x' y='>'>",
        "<a x' y='>'>",
    ] {
        let html = format!("{tag}z</a>");
        let whole = tag.len() - "<".len() + "/a>".len();
        for (pass, reads) in [("first", first_pass_reads(&html)), ("second", second_pass_reads(&html))] {
            assert_eq!(reads.tag_bytes, whole, "{tag:?}, {pass} pass");
        }
    }
}

/// The read of a tag looks at the bytes that can change it and jumps over the others: it
/// stops at each `=` and at the `>`, not at each byte. The count is of the stops, so a read
/// that goes byte by byte over the same tag fails here.
#[test]
fn the_read_of_a_tag_stops_at_an_equals_sign_and_at_the_bracket_only() {
    let html = format!("<a {}x=1 {}>z</a>", "b ".repeat(500), "c ".repeat(500));
    for (pass, reads) in [("first", first_pass_reads(&html)), ("second", second_pass_reads(&html))] {
        assert!(reads.tag_bytes > 2_000, "{pass} pass: the read went over the tag");
        assert_eq!(
            reads.tag_stops, 3,
            "{pass} pass: the `=`, the `>` of the tag, the `>` of `</a>`"
        );
    }
}

/// A kept script is written with each `<` as `&lt;` and with nothing else changed, so the pass
/// changes nothing in a page that it wrote: a repaired page is read as the page of the author.
#[test]
fn the_first_pass_changes_nothing_in_a_kept_script_that_it_wrote() {
    use super::strip_script_and_style_tags;
    let open = r#"<script type="application/ld+json">"#;
    for body in [
        r#"{"a":"a & b &amp; c < d &lt; e"}"#,
        r#"{"a":"<p>x</p>"}"#,
        "{} <script type=\"application/ld+json\">{} <script type=\"application/ld+json\">{} ",
    ] {
        let once = strip_script_and_style_tags(&format!("<p>a</p>{open}{body}</script><p>b</p>")).into_owned();
        assert_eq!(
            once,
            format!("<p>a</p>{open}{}</script><p>b</p>", body.replace('<', "&lt;")),
            "{body:?}"
        );
        assert_eq!(strip_script_and_style_tags(&once), once, "{body:?}: the second read");
    }
}

/// The one rule of the `type` of a script: the media type before the first `;`, in any case.
/// A longer media type that starts with the name keeps the script and is not structured data.
#[test]
fn the_type_of_a_script_is_read_by_one_rule() {
    use super::markup::{ScriptType, script_type};
    for (value, expected) in [
        ("application/ld+json", ScriptType::StructuredData),
        (" Application/LD+JSON ; charset=utf-8", ScriptType::StructuredData),
        ("application/ld+json;", ScriptType::StructuredData),
        ("application/ld+jsonx", ScriptType::KeptNotRead),
        ("application/ld+json5", ScriptType::KeptNotRead),
        ("application/ld+json, text/plain", ScriptType::KeptNotRead),
        ("x-application/ld+json", ScriptType::Removed),
        ("application/json", ScriptType::Removed),
        ("", ScriptType::Removed),
    ] {
        assert_eq!(script_type(value), expected, "{value:?}");
    }
}

#[test]
fn should_remove_the_end_tag_of_a_removed_start_tag_that_closes_itself() {
    // ~keep The inert-body pass and the tree builder read `<template/>` as a start tag: the
    // ~keep element holds what follows, up to its end tag. So the tag opens an element here too,
    // ~keep and its end tag is removed with it or stays with it.
    for (page, choice, expected) in [
        (
            "<p>a</p><template/><p>b</p></template><p>c</p>",
            HiddenContent::All,
            "<p>a</p><p>b</p><p>c</p>",
        ),
        (
            "<p>a</p><noscript/><p>b</p></noscript><p>c</p>",
            HiddenContent::All,
            "<p>a</p><p>b</p><p>c</p>",
        ),
        (
            "<p>a</p><template><p>b</p></template><p>c</p>",
            HiddenContent::All,
            "<p>a</p><p>b</p><p>c</p>",
        ),
        (
            "<p>a</p><template/><p>b</p></template><p>c</p>",
            HiddenContent::Reachable,
            "<p>a</p><template/><p>b</p></template><p>c</p>",
        ),
        (
            "<div><template shadowrootmode=\"open\"><template/></template><p>x</p></template></div>",
            HiddenContent::Reachable,
            "<div><template/></template><p>x</p></div>",
        ),
        // ~keep The end tag closes the shadow root, so the first `<template>` has no end tag and
        // ~keep holds the rest of the page.
        (
            "<template><p>IN</p><template shadowrootmode=\"open\"/></template><p>X</p>",
            HiddenContent::Reachable,
            "<template><p>IN</p><p>X</p>",
        ),
    ] {
        assert_eq!(unwrap_kept_inert_elements(page, choice, &[]), expected, "{page:?}");
    }
}

#[test]
fn should_write_an_end_tag_of_the_body_for_the_removed_tag_that_started_the_body() {
    let (reachable, all) = (HiddenContent::Reachable, HiddenContent::All);
    for (page, choice, expected) in [
        // ~keep The first `<template>` and the first `<noscript>` of a fragment start its body.
        (
            "<template><p>a</p></template><title>t</title>",
            all,
            "</body><p>a</p><title>t</title>",
        ),
        ("<noscript/><title>t</title>", all, "</body><title>t</title>"),
        (
            r#"<template shadowrootmode="open"><p>a</p></template><noscript>b</noscript>"#,
            reachable,
            "</body><p>a</p><noscript>b</noscript>",
        ),
        // ~keep After the end tag of the head, a `<noscript>` starts the body.
        (
            "<head></head><noscript><p>a</p></noscript><title>t</title>",
            all,
            "<head></head></body><p>a</p><title>t</title>",
        ),
        (
            "<html><template>a</template></head><noscript>b</noscript>",
            all,
            "<html><template>a</template></head></body>b",
        ),
        // ~keep The body of these pages starts at text, at an element, at an end tag and at a
        // ~keep tag that stays: the removed tags leave nothing in their place.
        ("a<noscript>b</noscript>", all, "ab"),
        ("<p>a</p><template>b</template>", all, "<p>a</p>b"),
        ("</br><template>a</template>", all, "</br>a"),
        (
            "<template>a</template><noscript>b</noscript>",
            reachable,
            "<template>a</template><noscript>b</noscript>",
        ),
        (
            r#"<template>a</template><div><template shadowrootmode="open">b</template></div>"#,
            reachable,
            "<template>a</template><div>b</div>",
        ),
    ] {
        assert_eq!(unwrap_kept_inert_elements(page, choice, &[]), expected, "{page:?}");
    }
}

#[test]
fn should_leave_a_tag_in_the_text_of_a_text_element_as_text() {
    let inner = [
        "<template><p>x</p></template>",
        "<noscript><p>x</p></noscript>",
        r#"<template shadowrootmode="open"><p>x</p></template>"#,
    ];
    for choice in [HiddenContent::Reachable, HiddenContent::All] {
        for name in ["textarea", "noembed", "noframes", "plaintext", "TextArea"] {
            for text in inner {
                // ~keep With its end tag, with no end tag, and with an attribute.
                for page in [
                    format!("<p>a</p><{name}>{text}</{name}>"),
                    format!("<p>a</p><{name}>{text}<p>b</p>"),
                    format!("<p>a</p><{name} class=\"c\">{text}</{name}><p>b</p>"),
                ] {
                    assert_eq!(unwrap_kept_inert_elements(&page, choice, &[]), page, "{choice:?}");
                }
            }
        }
    }
    let all = HiddenContent::All;
    for (page, expected) in [
        // ~keep The element ends at its end tag: a template after it is a template.
        (
            "<p>a</p><textarea><template></textarea><template>b</template>",
            "<p>a</p><textarea><template></textarea>b",
        ),
        (
            "<p>a</p><noembed></noscript></noembed><noscript>b</noscript>",
            "<p>a</p><noembed></noscript></noembed>b",
        ),
        // ~keep An end tag in the text closes nothing: the start tag before the element has no
        // ~keep end tag, and the end tag after the element has no start tag.
        (
            "<p>a</p><template><textarea></template></textarea>",
            "<p>a</p><textarea></template></textarea>",
        ),
        (
            "<p>a</p><noframes><noscript></noframes>b</noscript>",
            "<p>a</p><noframes><noscript></noframes>b</noscript>",
        ),
        // ~keep A `<plaintext>` has no end tag: the rest of the page is its text.
        (
            "<template>a</template><p>b</p><plaintext></plaintext><template>c</template>",
            "</body>a<p>b</p><plaintext></plaintext><template>c</template>",
        ),
        // ~keep A name that only starts like one of the elements is another element.
        (
            "<p>a</p><textareas><template>b</template></textareas>",
            "<p>a</p><textareas>b</textareas>",
        ),
    ] {
        assert_eq!(unwrap_kept_inert_elements(page, all, &[]), expected, "{page:?}");
    }
}

#[test]
fn should_strip_only_html_cdata_and_keep_opaque_foreign_and_raw_text_regions() {
    use super::strip_html_cdata;
    for input in [
        "<svg><![CDATA[<path/>]]></svg>",
        "<math><mi><![CDATA[x]]></mi></math>",
        "<p title=\"<![CDATA[attribute]]>\">visible</p>",
        "<textarea><![CDATA[literal]]></textarea>",
        "<!-- <svg> --><math><![CDATA[x]]></math>",
    ] {
        assert_eq!(strip_html_cdata(input), input);
    }
    assert_eq!(
        strip_html_cdata("<svg/><![CDATA[hidden]]><p>visible</p>"),
        "<svg/><p>visible</p>"
    );
    assert_eq!(
        strip_html_cdata("<svg></svg><![CDATA[hidden]]><p>visible</p>"),
        "<svg></svg><p>visible</p>"
    );
    assert_eq!(strip_html_cdata("before<![CDATA[unterminated"), "before");
}

#[test]
fn should_remove_html_cdata_after_foreign_text_with_a_literal_less_than() {
    assert_eq!(
        super::strip_html_cdata("<svg><text>a < b</text></svg><![CDATA[hidden]]>"),
        "<svg><text>a < b</text></svg>"
    );
}

#[test]
fn should_end_an_html_cdata_bogus_comment_at_the_first_greater_than() {
    assert_eq!(super::strip_html_cdata("<![CDATA[hidden>visible"), "visible");
}
