//! Coverage for `exclude_selectors` under each choice of `hidden_content` (issue #753): what an
//! exclusion removes does not depend on the choice.

#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, HiddenContent, OutputFormat, convert};

const CHOICES: [HiddenContent; 3] = [HiddenContent::Drop, HiddenContent::Reachable, HiddenContent::All];

/// The pages whose excluded element has no end tag, or holds an element with an omitted end tag:
/// the page, what it sets in the preprocessing options, and the output under each choice of
/// [`CHOICES`]. A token of a page starts with `V` for visible text, `X` for excluded text, `H`
/// for hidden text and `I` for the text of a `<template>` or a `<noscript>`.
const OMITTED_END_TAG_CASES: [(&str, &str, [&str; 3]); 21] = [
    (
        r#"<p>Va</p><div class="x"><p>Xb</div><p>Vc</p>"#,
        "",
        ["Va\n\nVc\n", "Va\n\nVc\n", "Va\n\nVc\n"],
    ),
    (r#"<p>Va</p><div class="x">Xb"#, "", ["Va\n", "Va\n", "Va\n"]),
    (
        r#"<p>Va</p><ul class="x"><li>Xb<li>Xc</ul><p>Vd</p>"#,
        "",
        ["Va\n\nVd\n", "Va\n\nVd\n", "Va\n\nVd\n"],
    ),
    (r#"<ul><li class="x">Xa<li>Vb</ul>"#, "", ["- Vb\n", "- Vb\n", "- Vb\n"]),
    (r#"<p class="x">Xa<p>Vb"#, "", ["Vb\n", "Vb\n", "Vb\n"]),
    (
        r#"<p>Va</p><div class="x"><span>Xb</div><p>Qc</p>"#,
        "",
        ["Va\n\nQc\n", "Va\n\nQc\n", "Va\n\nQc\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><b>Xb</div><p>Qc</p>"#,
        "",
        ["Va\n\n**Qc**\n", "Va\n\n**Qc**\n", "Va\n\n**Qc**\n"],
    ),
    (
        r#"<p>Va</p><div class="x">Xb</DIV><p>Vc</p>"#,
        "",
        ["Va\n\nVc\n", "Va\n\nVc\n", "Va\n\nVc\n"],
    ),
    (
        r#"<p>Va</p><div class="x">Xb</div ><p>Vc</p>"#,
        "",
        ["Va\n\nVc\n", "Va\n\nVc\n", "Va\n\nVc\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><my-el>Xb</div><p>Qc</p>"#,
        "",
        ["Va\n\nQc\n", "Va\n\nQc\n", "Va\n\nQc\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><p>Xb<p>Xc</div><div hidden>Hd</div><p>Ve</p>"#,
        "",
        ["Va\n\nVe\n", "Va\n\nHd\n\nVe\n", "Va\n\nHd\n\nVe\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><ul><li>Xb<li>Xc</ul></div><p>Vd</p>"#,
        "",
        ["Va\n\nVd\n", "Va\n\nVd\n", "Va\n\nVd\n"],
    ),
    (
        r#"<p>Va</p><nav class="x"><p>Xb</nav><p>Vc</p>"#,
        "preoff",
        ["Va\n\nVc\n", "Va\n\nVc\n", "Va\n\nVc\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><dl><dt>Xb<dd>Xc</dl></div><p>Vd</p>"#,
        "",
        ["Va\n\nVd\n", "Va\n\nVd\n", "Va\n\nVd\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><table><tr><td>Xb<td>Xc</table></div><p>Vd</p>"#,
        "",
        ["Va\n\nVd\n", "Va\n\nVd\n", "Va\n\nVd\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><select><option>Xb<option>Xc</select>Xd</div><p>Ve</p>"#,
        "",
        ["Va\n\nVe\n", "Va\n\nVe\n", "Va\n\nVe\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><select><option>Xb<option>Xc</select>Xd</div><p>Ve</p>"#,
        "keepforms",
        ["Va\n\nVe\n", "Va\n\nVe\n", "Va\n\nVe\n"],
    ),
    (r#"<p>Va</p><span class="x">Xb<p>Vc</p>"#, "", ["Va\n", "Va\n", "Va\n"]),
    (
        "<p>Va</p><aside><p>Xb<p>Xc</aside><p>Vd</p>",
        "",
        ["Va\n\nVd\n", "Va\n\nVd\n", "Va\n\nVd\n"],
    ),
    (
        r#"<p>Va</p><div class="x"><p>Xbé</div>é<p>Vc</p>"#,
        "",
        ["Va\n\né\n\nVc\n", "Va\n\né\n\nVc\n", "Va\n\né\n\nVc\n"],
    ),
    (
        r#"<div class="x"><p>Xa</div><div hidden>Hb</div><noscript>Ic</noscript><p>Vd</p>"#,
        "",
        ["Vd\n", "Hb\n\nVd\n", "Hb\n\nIc\n\nVd\n"],
    ),
];

/// The pages with an excluded `<template>` or `<noscript>`, and their selectors. The excluded
/// element is closed, has no end tag, is written as self-closing, holds an omitted end tag, holds
/// a kept element, or is in a kept or an excluded element.
const KEPT_ELEMENT_CASES: [(&str, &[&str]); 28] = [
    ("<p>Va</p><template><p>Xb</p>", &["template"]),
    (r#"<p>Va</p><noscript class="x"><p>Xb</p>"#, &[".x"]),
    (
        r#"<p>Va</p><template class="x"/><p>Xb</p></template><p>Vc</p>"#,
        &[".x"],
    ),
    (r#"<p>Va</p><noscript id="x"/>Xb</noscript><p>Vc</p>"#, &["#x"]),
    (
        "<p>Va</p><template data-x><ul><li>Xb<li>Xc</ul></template><p>Vd</p>",
        &["[data-x]"],
    ),
    (
        r#"<p>Va</p><noscript class="x"><p>Xb<div>Xc</noscript><p>Vd</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template class="x" data-a="a>b"><p>Xb</p></template><p>Vc</p>"#,
        &[".x"],
    ),
    (r#"<p>Va</p><template class="x"><my-el>Xb</template><p>Vc</p>"#, &[".x"]),
    (
        r#"<div><template shadowrootmode="open" class="x"><p>Xa<slot></slot></template><p>Vb</p></div>"#,
        &[".x"],
    ),
    (
        r#"<div><template shadowrootmode="closed"><ul><li>Xa<li>Xb</ul></template><p>Vc</p></div>"#,
        &["[shadowrootmode]"],
    ),
    (
        r#"<p>Va</p><template class="x">Xb<template>Xc</template>Xd</template><p>Ve</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template>Ib<noscript class="x">Xc<template>Xd</template>Xe</noscript>If</template><p>Vg</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template>Ib<template class="x">Xc</template>Id</template><p>Ve</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><noscript><template class="x">Xb</template>Ic</noscript><template><noscript class="x">Xd</noscript>Ie</template><p>Vf</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template><template class="x"><p>Xb</template>Ic</template><p>Vd</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><noscript><template class="x"><p>Xb</template>Ic</noscript><p>Vd</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template class="x">Xb</template><template>Xc</template><p>Vd</p>"#,
        &[".x", "template"],
    ),
    (
        r#"<p>Va</p><template class="x">Xb</template><noscript id="y">Xc</noscript><span class="x">Xd</span><p>Ve</p>"#,
        &[".x", "#y"],
    ),
    (
        r#"<p>Va</p><noscript class="x"><p>Xb</p></noscript><template><p>Ic</p></template><p>Vd</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template><p>Ib</p></template><noscript class="x">Xc"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><div hidden>Hb<template class="x">Xc</template>Hd</div><p>Ve</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><template class="x"><p>Xb</p></template><div class="x">Xc"#,
        &[".x"],
    ),
    (
        r#"<table><tr><td>Va</td></tr><template class="x"><tr><td>Xb</td></tr></template></table>"#,
        &[".x"],
    ),
    (r#"<p>Vaé</p><noscript class="x">Xbé</noscript>éVc"#, &[".x"]),
    (
        r#"<p>Va</p><template><div class="x">Xb</div><p>Ic</p></template><p>Vd</p>"#,
        &[".x"],
    ),
    (
        r#"<p>Va</p><div class="x">Xb<template><p>Xc</p></template><noscript>Xd</noscript></div><p>Ve</p>"#,
        &[".x"],
    ),
    (r#"<p>Va</p><div class="x">Xb<template><p>Xc</p></template>"#, &[".x"]),
    (
        r#"<p>Va</p><div class="x">Xb<span hidden>Xc</span><template>Xd</template><noscript>Xe</noscript></div><p>Vf</p>"#,
        &[".x"],
    ),
];

fn convert_with(
    html: &str,
    selectors: &[&str],
    preprocessing: &str,
    choice: HiddenContent,
    format: OutputFormat,
) -> String {
    let mut options = ConversionOptions {
        hidden_content: choice,
        exclude_selectors: selectors.iter().map(|selector| (*selector).to_owned()).collect(),
        output_format: format,
        extract_metadata: false,
        ..ConversionOptions::default()
    };
    match preprocessing {
        "preoff" => options.preprocessing.enabled = false,
        "keepforms" => options.preprocessing.remove_forms = false,
        _ => {}
    }
    convert(html, options)
        .expect("conversion succeeds")
        .content
        .unwrap_or_default()
}

/// The tokens of `text`: a capital letter of `kinds` and the small letter after it.
fn tokens(text: &str, kinds: &str) -> Vec<String> {
    text.as_bytes()
        .windows(2)
        .filter(|pair| kinds.as_bytes().contains(&pair[0]) && pair[1].is_ascii_lowercase())
        .map(|pair| String::from_utf8_lossy(pair).into_owned())
        .collect()
}

#[test]
fn should_exclude_an_element_with_an_omitted_end_tag_under_each_choice() {
    for (html, preprocessing, expected) in OMITTED_END_TAG_CASES {
        // ~keep One element carries the class, the id and the attribute, for each kind of selector.
        let page = html.replace(r#"class="x""#, r#"class="x" id="x" data-x"#);
        let selectors: &[&str] = if html.contains("<aside>") {
            &["aside"]
        } else {
            &[".x", "#x", "[data-x]"]
        };
        for &selector in selectors {
            for (choice, expected) in CHOICES.into_iter().zip(expected) {
                assert_eq!(
                    convert_with(&page, &[selector], preprocessing, choice, OutputFormat::Markdown),
                    expected,
                    "{html}, {selector}, {choice:?}"
                );
            }
        }
    }
}

#[test]
fn should_keep_tag_shaped_text_that_a_selector_matches_under_each_choice() {
    // ~keep The text of a `<textarea>` and the label of a graphic are no elements of the page.
    for (html, preprocessing, format, expected) in [
        (
            r#"<p>Va</p><textarea><div class="x">Qb</div></textarea><p>Vc</p>"#,
            "keepforms",
            OutputFormat::Markdown,
            "Va\n\n<div class=\"x\">Qb</div>\n\nVc\n",
        ),
        (
            r#"<p>Va</p><textarea><p class="x">Qb</textarea><p>Vc</p>"#,
            "keepforms",
            OutputFormat::Markdown,
            "Va\n\n<p class=\"x\">Qb\n\nVc\n",
        ),
        (
            r#"<p>Va</p><svg><text class="x">Xb</text><text>Qc</text></svg>"#,
            "",
            OutputFormat::Plain,
            "Va\n\nXb Qc\n",
        ),
    ] {
        for choice in CHOICES {
            assert_eq!(
                convert_with(html, &[".x"], preprocessing, choice, format),
                expected,
                "{html}, {choice:?}"
            );
        }
    }
}

#[test]
fn should_write_no_excluded_text_of_a_template_or_a_noscript_under_each_choice() {
    for (html, selectors) in KEPT_ELEMENT_CASES {
        for choice in CHOICES {
            for format in [OutputFormat::Markdown, OutputFormat::Plain] {
                let output = convert_with(html, selectors, "", choice, format);
                let written = tokens(&output, "VXHI");
                for token in tokens(html, "VXHI") {
                    let wanted = match token.as_bytes()[0] {
                        b'V' => true,
                        b'H' => choice != HiddenContent::Drop,
                        b'I' => choice == HiddenContent::All,
                        _ => false,
                    };
                    assert_eq!(
                        written.contains(&token),
                        wanted,
                        "{token} in {output:?}: {html}, {choice:?}, {format:?}"
                    );
                }
            }
        }
    }
}
