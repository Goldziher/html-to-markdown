//! Full-main empty-boundary policy is shared by every public conversion route. ~keep
#![cfg(feature = "testkit")]
use html_to_markdown_rs::{ConversionOptions, TierStrategy, WhitespaceMode, convert};
const EMPTY_BOUNDARIES: [(&str, &str, &str); 13] = [
    (
        "option_span",
        "<p><option>a</option>\n<option><span></span></option>&nbsp;b</p>",
        "a b\n",
    ),
    (
        "option_empty",
        "<p><option>a</option>\n<option></option>&nbsp;b</p>",
        "a b\n",
    ),
    ("optgroup_empty", "<p>a\n<optgroup></optgroup>&nbsp;b</p>", "a b\n"),
    ("heading_option", "<h2>a\n<option></option>&nbsp;b</h2>", "## a b\n"),
    (
        "heading_optgroup",
        "<h2>a\n<optgroup></optgroup>&nbsp;b</h2>",
        "## a b\n",
    ),
    ("code_newline", "<p>a\n<span><code>\n</code></span>&nbsp;b</p>", "a b\n"),
    ("del_newline", "<p>a\n<span><del>\n</del></span>&nbsp;b</p>", "a b\n"),
    ("nonflow_q", "<p>a \n<span><q></q></span>&nbsp;b</p>", "a b\n"),
    (
        "nonflow_textarea",
        "<p>a \n<span><textarea></textarea></span>&nbsp;b</p>",
        "a b\n",
    ),
    ("nonflow_math", "<p>a \n<span><math></math></span>&nbsp;b</p>", "a b\n"),
    (
        "nonflow_table",
        "<p>a \n<span><table></table></span>&nbsp;b</p>",
        "a b\n",
    ),
    (
        "nonflow_ul",
        "<ul><li>a \n<span><ul></ul></span>&nbsp;b</li></ul>",
        "- a b\n",
    ),
    ("empty_span_split", "<p>a \n<span></span>&nbsp;b</p>", "a b\n"),
];
const PRESERVED_OUTPUT: [(&str, &str, &str, &str); 15] = [
    (
        "pre_blank",
        "<p>a\n<span><pre>\n</pre></span>&nbsp;b</p>",
        "a\n\n```\n\n```\n\nb\n",
        "default",
    ),
    (
        "pre_nested_code",
        "<p>a\n<span><pre><code>\n\n</code></pre></span>&nbsp;b</p>",
        "a\n\n```\n\n```\n\nb\n",
        "default",
    ),
    (
        "outer_pre",
        "<pre> a\n<span><code>\n\n</code></span>&nbsp;b </pre>",
        "```\n a\n\n\n b \n```\n",
        "default",
    ),
    (
        "inline_code",
        "<p><code>a\n\n b</code>&nbsp;c</p>",
        "`a   b` c\n",
        "default",
    ),
    (
        "deep_break",
        "<p>a\n<span><b><br></b></span>&nbsp;b</p>",
        "a  \n b\n",
        "default",
    ),
    (
        "blank_nbsp",
        "<p>a <span>&nbsp;</span>&nbsp;b</p>",
        "a   b\n",
        "default",
    ),
    (
        "nbsp_line_before",
        "<p>a&nbsp;\n<span></span>&nbsp;b</p>",
        "a  b\n",
        "default",
    ),
    (
        "nbsp_run_before",
        "<p>a&nbsp; \n<span></span>&nbsp;b</p>",
        "a  b\n",
        "default",
    ),
    (
        "strict_code",
        "<p>a \n<span><code>\n</code></span>&nbsp;b</p>",
        "a\n b\n",
        "strict",
    ),
    (
        "preserve_code",
        "<p>a\n<span><code>\n</code></span>&nbsp;b</p>",
        "a <code>\n</code> b\n",
        "code",
    ),
    (
        "preserve_del",
        "<p>a\n<span><del>\n</del></span>&nbsp;b</p>",
        "a <del>\n</del> b\n",
        "del",
    ),
    (
        "preserve_span",
        "<p>a \n<span><q></q></span>&nbsp;b</p>",
        "a <span><q></q></span> b\n",
        "span",
    ),
    (
        "code_space",
        "<p>a\n<span><code> </code></span>&nbsp;b</p>",
        "a ` ` b\n",
        "default",
    ),
    ("space_empty_span", "<p>a <span></span>&nbsp;b</p>", "a  b\n", "default"),
    (
        "visible_span",
        "<p>a\n<span>x </span>&nbsp;b</p>",
        "a x  b\n",
        "default",
    ),
];
fn assert_routes(source: &str, expected: &str, mode: &str) -> usize {
    assert_routes_with_fast(source, expected, mode, None)
}
fn assert_routes_with_fast(source: &str, expected: &str, mode: &str, fast: Option<&str>) -> usize {
    let mut count = 0;
    for tier_strategy in [TierStrategy::Auto, TierStrategy::Tier1, TierStrategy::Tier2] {
        for extract_metadata in [false, true] {
            let mut options = ConversionOptions {
                tier_strategy,
                extract_metadata,
                ..Default::default()
            };
            if mode == "strict" {
                options.whitespace_mode = WhitespaceMode::Strict;
            }
            if ["code", "del", "span", "p"].contains(&mode) {
                options.preserve_tags = vec![mode.into()];
            }
            let expected = if tier_strategy == TierStrategy::Tier1 {
                fast.unwrap_or(expected)
            } else {
                expected
            };
            assert_eq!(
                convert(source, options)
                    .expect("empty boundary converts")
                    .content
                    .as_deref(),
                Some(expected),
                "{source:?}, {tier_strategy:?}, metadata={extract_metadata}, mode={mode}"
            );
            count += 1;
        }
    }
    count
}
fn assert_boundaries_where(select: fn(&str) -> bool) -> usize {
    let mut count = 0;
    for (label, source, expected) in EMPTY_BOUNDARIES {
        if select(label) {
            for tail in ["&nbsp;", "\u{a0}"] {
                count += assert_routes(&source.replace("&nbsp;", tail), expected, "default");
            }
        }
    }
    count
}
#[test]
fn should_use_full_main_spacing_for_empty_options_on_every_route() {
    assert_eq!(
        assert_boundaries_where(|label| label.contains("option") || label.contains("optgroup")),
        60
    );
}
#[test]
fn should_join_nbsp_to_a_line_end_after_empty_code_or_del() {
    assert_eq!(
        assert_boundaries_where(|label| label == "code_newline" || label == "del_newline"),
        24
    );
}
#[test]
fn should_use_full_main_spacing_after_empty_nonflow_descendants() {
    assert_eq!(
        assert_boundaries_where(|label| label.starts_with("nonflow_") || label == "empty_span_split"),
        72
    );
}
#[test]
fn should_preserve_literal_whitespace_and_existing_output_bearing_boundaries() {
    let mut count = 0;
    for (_, source, expected, mode) in PRESERVED_OUTPUT {
        count += assert_routes(source, expected, mode);
    }
    assert_eq!(count, 90);
}

// Existing strict paragraph spans differ in forced-fast mode; this repair preserves that split. ~keep
const LEADING_BOUNDARIES: [(&str, &str, &str, Option<&str>); 96] = [
    ("<p>\n<span></span>&nbsp;b</p>", "b\n", "default", None),
    ("<p>\n<span></span> b</p>", "b\n", "default", None),
    ("<p>\n<span></span>&nbsp;b</p>", " b\n", "strict", Some("b\n")),
    ("<p>\n<span></span> b</p>", " b\n", "strict", Some("b\n")),
    ("<p>\n<option></option>&nbsp;b</p>", "b\n", "default", None),
    ("<p>\n<option></option> b</p>", "b\n", "default", None),
    ("<p>\n<option></option>&nbsp;b</p>", " b\n", "strict", None),
    ("<p>\n<option></option> b</p>", " b\n", "strict", None),
    ("<h2>\n<span></span>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2>\n<span></span> b</h2>", "## b\n", "default", None),
    ("<h2>\n<span></span>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2>\n<span></span> b</h2>", "## b\n", "strict", None),
    ("<h2>\n<option></option>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2>\n<option></option> b</h2>", "## b\n", "default", None),
    ("<h2>\n<option></option>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2>\n<option></option> b</h2>", "## b\n", "strict", None),
    ("<ul><li>\n<span></span>&nbsp;b</li></ul>", "- b\n", "default", None),
    ("<ul><li>\n<span></span> b</li></ul>", "- b\n", "default", None),
    ("<ul><li>\n<span></span>&nbsp;b</li></ul>", "-\n   b\n", "strict", None),
    ("<ul><li>\n<span></span> b</li></ul>", "-\n   b\n", "strict", None),
    ("<ul><li>\n<option></option>&nbsp;b</li></ul>", "- b\n", "default", None),
    ("<ul><li>\n<option></option> b</li></ul>", "- b\n", "default", None),
    (
        "<ul><li>\n<option></option>&nbsp;b</li></ul>",
        "-\n   b\n",
        "strict",
        None,
    ),
    ("<ul><li>\n<option></option> b</li></ul>", "-\n   b\n", "strict", None),
    (
        "<table><tr><td>\n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    ("<p> \n<span></span>&nbsp;b</p>", "b\n", "default", None),
    ("<p> \n<span></span> b</p>", "b\n", "default", None),
    ("<p> \n<span></span>&nbsp;b</p>", " b\n", "strict", Some("b\n")),
    ("<p> \n<span></span> b</p>", " b\n", "strict", Some("b\n")),
    ("<p> \n<option></option>&nbsp;b</p>", "b\n", "default", None),
    ("<p> \n<option></option> b</p>", "b\n", "default", None),
    ("<p> \n<option></option>&nbsp;b</p>", " b\n", "strict", None),
    ("<p> \n<option></option> b</p>", " b\n", "strict", None),
    ("<h2> \n<span></span>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2> \n<span></span> b</h2>", "## b\n", "default", None),
    ("<h2> \n<span></span>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2> \n<span></span> b</h2>", "## b\n", "strict", None),
    ("<h2> \n<option></option>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2> \n<option></option> b</h2>", "## b\n", "default", None),
    ("<h2> \n<option></option>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2> \n<option></option> b</h2>", "## b\n", "strict", None),
    ("<ul><li> \n<span></span>&nbsp;b</li></ul>", "- b\n", "default", None),
    ("<ul><li> \n<span></span> b</li></ul>", "- b\n", "default", None),
    (
        "<ul><li> \n<span></span>&nbsp;b</li></ul>",
        "-  \n   b\n",
        "strict",
        None,
    ),
    ("<ul><li> \n<span></span> b</li></ul>", "-  \n   b\n", "strict", None),
    (
        "<ul><li> \n<option></option>&nbsp;b</li></ul>",
        "- b\n",
        "default",
        None,
    ),
    ("<ul><li> \n<option></option> b</li></ul>", "- b\n", "default", None),
    (
        "<ul><li> \n<option></option>&nbsp;b</li></ul>",
        "-  \n   b\n",
        "strict",
        None,
    ),
    (
        "<ul><li> \n<option></option> b</li></ul>",
        "-  \n   b\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td> \n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td> \n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td> \n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td> \n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td> \n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td> \n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td> \n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td> \n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    ("<p>\t\n<span></span>&nbsp;b</p>", "b\n", "default", None),
    ("<p>\t\n<span></span> b</p>", "b\n", "default", None),
    ("<p>\t\n<span></span>&nbsp;b</p>", " b\n", "strict", Some("b\n")),
    ("<p>\t\n<span></span> b</p>", " b\n", "strict", Some("b\n")),
    ("<p>\t\n<option></option>&nbsp;b</p>", "b\n", "default", None),
    ("<p>\t\n<option></option> b</p>", "b\n", "default", None),
    ("<p>\t\n<option></option>&nbsp;b</p>", " b\n", "strict", None),
    ("<p>\t\n<option></option> b</p>", " b\n", "strict", None),
    ("<h2>\t\n<span></span>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2>\t\n<span></span> b</h2>", "## b\n", "default", None),
    ("<h2>\t\n<span></span>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2>\t\n<span></span> b</h2>", "## b\n", "strict", None),
    ("<h2>\t\n<option></option>&nbsp;b</h2>", "## b\n", "default", None),
    ("<h2>\t\n<option></option> b</h2>", "## b\n", "default", None),
    ("<h2>\t\n<option></option>&nbsp;b</h2>", "## b\n", "strict", None),
    ("<h2>\t\n<option></option> b</h2>", "## b\n", "strict", None),
    ("<ul><li>\t\n<span></span>&nbsp;b</li></ul>", "- b\n", "default", None),
    ("<ul><li>\t\n<span></span> b</li></ul>", "- b\n", "default", None),
    (
        "<ul><li>\t\n<span></span>&nbsp;b</li></ul>",
        "-\n   b\n",
        "strict",
        None,
    ),
    ("<ul><li>\t\n<span></span> b</li></ul>", "-\n   b\n", "strict", None),
    (
        "<ul><li>\t\n<option></option>&nbsp;b</li></ul>",
        "- b\n",
        "default",
        None,
    ),
    ("<ul><li>\t\n<option></option> b</li></ul>", "- b\n", "default", None),
    (
        "<ul><li>\t\n<option></option>&nbsp;b</li></ul>",
        "-\n   b\n",
        "strict",
        None,
    ),
    ("<ul><li>\t\n<option></option> b</li></ul>", "-\n   b\n", "strict", None),
    (
        "<table><tr><td>\t\n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\t\n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\t\n<span></span>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\t\n<span></span> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\t\n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\t\n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "default",
        None,
    ),
    (
        "<table><tr><td>\t\n<option></option>&nbsp;b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
    (
        "<table><tr><td>\t\n<option></option> b</td></tr></table>",
        "| b |\n| --- |\n",
        "strict",
        None,
    ),
];
#[test]
fn should_preserve_existing_leading_boundaries_in_normalized_and_strict_modes() {
    let mut count = 0;
    for (source, expected, mode, fast) in LEADING_BOUNDARIES {
        count += assert_routes_with_fast(source, expected, mode, fast);
    }
    assert_eq!(count, 576);
}

const LITERAL_OWNERSHIP: [(&str, &str, Option<&str>); 16] = [
    ("<p>a\n<span><code>&#10;</code></span>&nbsp;b</p>", "a b\n", None),
    ("<p>a\n<span><del>&#10;</del></span>&nbsp;b</p>", "a b\n", None),
    (
        "<p>a\n<span><code>&#13;&#10;</code></span>&nbsp;b</p>",
        "a `\r` b\n",
        None,
    ),
    ("<p>a\n<span><del>&#13;&#10;</del></span>&nbsp;b</p>", "a b\n", None),
    (
        "<p>a\n<span><pre><!--x--></pre></span>&nbsp;b</p>",
        "a b\n",
        Some("a\n\nb\n"),
    ),
    (
        "<p>a\n<span><pre><i></i></pre></span>&nbsp;b</p>",
        "a b\n",
        Some("a\n\nb\n"),
    ),
    (
        "<p>a\n<span><code><span>\n</span></code></span>&nbsp;b</p>",
        "a b\n",
        None,
    ),
    (
        "<p>a\n<span><del><span>\n</span></del></span>&nbsp;b</p>",
        "a b\n",
        None,
    ),
    ("<p>a\n<span><code><b><br></b></code></span>&nbsp;b</p>", "a  b\n", None),
    (
        "<p>a\n<span><del><b><br></b></del></span>&nbsp;b</p>",
        "a  \n b\n",
        None,
    ),
    (
        "<p>a\n<span><code><b><img src='a.png' alt='x'></b></code></span>&nbsp;b</p>",
        "a `x` b\n",
        None,
    ),
    (
        "<p>a\n<span><del><b><img src='a.png' alt='x'></b></del></span>&nbsp;b</p>",
        "a ~~**![x](a.png)**~~ b\n",
        None,
    ),
    ("<p>a\n<span><code>&nbsp;</code></span>&nbsp;b</p>", "a ` ` b\n", None),
    ("<p>a\n<span><del>&nbsp;</del></span>&nbsp;b</p>", "a   b\n", None),
    (
        "<p>a\n<span><pre>&#10;&nbsp;&#13;</pre></span>&nbsp;b</p>",
        "a\n\n```\n \r\n```\n\nb\n",
        None,
    ),
    ("<p>a\n<span>\n</span>&nbsp;b</p>", "a  b\n", None),
];
#[test]
fn should_preserve_output_bearing_and_nested_literal_ownership() {
    let mut count = 0;
    for (source, expected, fast) in LITERAL_OWNERSHIP {
        count += assert_routes_with_fast(source, expected, "default", fast);
    }
    assert_eq!(count, 96);
}

#[test]
fn should_join_nbsp_after_nested_code_and_del_line_ends() {
    let mut rows = 0;
    for owner in ["code", "del"] {
        for body in ["\n", "\r\n", "&#10;"] {
            for depth in [1, 3] {
                for tail in ["&nbsp;", "\u{a0}"] {
                    let inner = format!("{}{}{}", "<span>".repeat(depth), body, "</span>".repeat(depth));
                    let source = format!("<p>a\n<span><{owner}>{inner}</{owner}></span>{tail}b</p>");
                    rows += assert_routes(&source, "a b\n", "default");
                }
            }
        }
    }
    assert_eq!(rows, 144);
}
#[test]
fn should_keep_code_ownership_literal_content_and_unowned_line_ends() {
    let mut rows = 0;
    for (source, expected, fast) in [
        (
            "<p>a\n<span><code><del><span>&#13;</span></del></code></span>&nbsp;b</p>",
            "a `\r` b\n",
            None,
        ),
        (
            "<p>a\n<span><code><del><span>&#13;&#10;</span></del></code></span>&nbsp;b</p>",
            "a `\r` b\n",
            None,
        ),
        (
            "<p>a\n<span><del><code><span>&#13;</span></code></del></span>&nbsp;b</p>",
            "a ~~`\r`~~ b\n",
            Some("a ~~` `~~ b\n"),
        ),
        (
            "<p>a\n<span><code><del><span> </span></del></code></span>&nbsp;b</p>",
            "a ` ` b\n",
            None,
        ),
        (
            "<p>a\n<span><code><del><span>\n</span></del></code></span>&nbsp;b</p>",
            "a b\n",
            None,
        ),
        (
            "<p>a\n<span><del><code><span>\n</span></code></del></span>&nbsp;b</p>",
            "a b\n",
            Some("a ~~` `~~ b\n"),
        ),
        (
            "<p>a\n<span><del><span>&#13;</span></del></span>&nbsp;b</p>",
            "a  b\n",
            None,
        ),
        (
            "<p>a\n<span><pre><code><del><span>\n</span></del></code></pre></span>&nbsp;b</p>",
            "a\n\n```\n\n```\n\nb\n",
            None,
        ),
        (
            "<p>a\n<span><code><span>&nbsp;</span></code></span>&nbsp;b</p>",
            "a `\u{a0}` b\n",
            None,
        ),
        (
            "<p>a\n<span><code><span><br></span></code></span>&nbsp;b</p>",
            "a  b\n",
            None,
        ),
        ("<p>a\n<span>\n</span>&nbsp;b</p>", "a  b\n", None),
    ] {
        rows += assert_routes_with_fast(source, expected, "default", fast);
    }
    for (source, expected, mode) in [
        (
            "<p>a\n<span><code><span>\n</span></code></span>&nbsp;b</p>",
            "a\n\u{a0}b\n",
            "strict",
        ),
        (
            "<p>a\n<span><del><span>\n</span></del></span>&nbsp;b</p>",
            "a\n\n\u{a0}b\n",
            "strict",
        ),
        (
            "<p>a\n<span><code><span>\n</span></code></span>&nbsp;b</p>",
            "a <code><span>\n</span></code> b\n",
            "code",
        ),
        (
            "<p>a\n<span><del><span>\n</span></del></span>&nbsp;b</p>",
            "a <del><span>\n</span></del> b\n",
            "del",
        ),
    ] {
        rows += assert_routes(source, expected, mode);
    }
    assert_eq!(rows, 90);
}

fn assert_default_and_routes(source: &str, expected: &str) -> usize {
    assert_eq!(
        convert(source, None).expect("default list boundary").content.as_deref(),
        Some(expected),
        "{source}"
    );
    1 + assert_routes(source, expected, "default")
}
#[test]
fn should_keep_nbsp_content_indent_after_empty_blocks_in_list_spans() {
    let mut rows = 0;
    for (tag, expected) in [("p", "- a\n\n   b\n"), ("div", "- a\n   b\n"), ("table", "- a\n   b\n")] {
        for tail in ["&nbsp;", "\u{a0}"] {
            for depth in [0, 2, 16] {
                let inner = format!("{}<{tag}></{tag}>{}", "<span>".repeat(depth), "</span>".repeat(depth));
                let source = format!("<ul><li>a\n<span>{inner}</span>{tail}b</li></ul>");
                rows += assert_default_and_routes(&source, expected);
            }
        }
    }
    for tail in ["&nbsp;", "\u{a0}"] {
        rows += assert_default_and_routes(
            &format!("<ol><li>a\n<span><p></p></span>{tail}b</li></ol>"),
            "1. a\n\n    b\n",
        );
    }
    assert_eq!(rows, 140);
}
#[test]
fn should_preserve_visible_list_block_and_nonlist_prefix_ownership() {
    let mut rows = 0;
    for (source, expected) in [
        ("<ul><li>a\n<span></span>&nbsp;b</li></ul>", "- a b\n"),
        ("<ul><li>a\n<span>\n</span>&nbsp;b</li></ul>", "- a  b\n"),
        (
            "<ul><li>a\n<span><ul><li>x</li></ul></span>&nbsp;b</li></ul>",
            "- a\n  * x\n\n  b\n",
        ),
        (
            "<ul><li>a\n<span><p>x</p></span>&nbsp;b</li></ul>",
            "- a\n\n  x\n\n  b\n",
        ),
        ("<p>a\n<span><div></div></span>&nbsp;b</p>", "a\n\nb\n"),
        (
            "<table><tr><td>a\n<span><p></p></span>&nbsp;b</td></tr></table>",
            "| a  b |\n| ---- |\n",
        ),
        ("<p>a\n<span>\n</span>&nbsp;b</p>", "a  b\n"),
    ] {
        rows += assert_default_and_routes(source, expected);
    }
    for (source, expected) in [
        (
            "<ul><li>a\n<span><code><p></p></code></span>&nbsp;b</li></ul>",
            "- a\n\n  b\n",
        ),
        (
            "<ul><li>a\n<span><pre>\n</pre></span>&nbsp;b</li></ul>",
            "- a\n\n  ```\n\n  ```\n\n   b\n",
        ),
        ("<ul><li>a\n<p></p>&nbsp;b</li></ul>", "- a\n\n   b\n"),
        ("<ul><li>a\n<span><p></p></span> b</li></ul>", "- a\n\n  b\n"),
    ] {
        rows += assert_default_and_routes(source, expected);
    }
    rows += assert_routes(
        "<ul><li>a\n<span><p></p></span>&nbsp;b</li></ul>",
        "- a\n\n  <p></p>\n\n   b\n",
        "p",
    );
    rows += assert_routes(
        "<ul><li>a\n<span><p></p></span>&nbsp;b</li></ul>",
        "- a\n\n  \u{a0}b\n",
        "strict",
    );
    assert_eq!(rows, 89);
}
