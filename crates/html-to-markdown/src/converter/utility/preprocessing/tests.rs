use std::borrow::Cow;

use super::{
    find_closing_tag_bytes, find_closing_tag_bytes_nested, find_tag_end, normalize_bogus_comment_endings,
    normalize_menu_elements, normalize_split_closing_tags, normalize_unclosed_list_items,
    restore_preserved_menu_elements, sanitize_markdown_url, strip_bogus_comments, strip_hidden_elements,
};

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
