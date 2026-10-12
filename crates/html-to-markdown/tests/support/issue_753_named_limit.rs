// ~keep Included by `issue_753_hidden_content_test.rs`: the pages of the named limit of the
// ~keep order of the three choices.

/// The generated documents on which a wider choice writes less than a narrower one: the page,
/// then the output of `drop`, of `reachable` and of `all` with no metadata.
const NAMED_LIMIT_PAGES: &[[&str; 4]] = &[
    [
        r#"<head><img src="p.gif" alt="A1"><template></head><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><TEMPLATE shadowrootmode="open"></head><p>End</p>"#,
        "![A1](p.gif)\n",
        "End\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><noscript></head><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><template><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><TEMPLATE shadowrootmode="open"><p>End</p>"#,
        "![A1](p.gif)\n",
        "End\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><noscript><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head></NOSCRIPT><head><img src="p.gif" alt="A3"><noscript><p>W5</p></head></head><img src="p.gif" alt="A8"><p>End</p>"#,
        "![A3](p.gif)\n",
        "![A3](p.gif)\n",
        "![A8](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><template><noscript/><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head></NOSCRIPT><img src="p.gif" alt="A2"><meta name="m3" content="c"><head><noscript><template></div></head><noscript/><noscript></head><p>End</p>"#,
        "![A2](p.gif)\n",
        "![A2](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></div><template></head><img src="p.gif" alt="A5"><p>W6</p></div></div></NOSCRIPT><noscript><img src="p.gif" alt="A11"><template/><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "![A5](p.gif)\n\nW6\n\n![A11](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></NOSCRIPT><template><template><noscript><template></head></head><img src="p.gif" alt="A9"><body><noscript/><meta name="m12" content="c"><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "![A9](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head><title>T1</title><img src="p.gif" alt="A2"><noscript><meta name="m4" content="c"></head><p>End</p>"#,
        "![A2](p.gif)\n",
        "![A2](p.gif)\n",
        "End\n",
    ],
    [
        r#"</NOSCRIPT><meta name="m1" content="c"><body><body><head><title>T5</title></head><template><TEMPLATE shadowrootmode="open"><p>End</p>"#,
        "T5\n",
        "T5\n",
        "End\n",
    ],
    [
        r#"<head><noscript/><meta name="m2" content="c"></div><head><img src="p.gif" alt="A5"><noscript><p>W7</p></head></head><p>End</p>"#,
        "![A5](p.gif)\n",
        "![A5](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head></head><noscript/><meta name="m3" content="c"></template><head><img src="p.gif" alt="A6"></head></NOSCRIPT></NOSCRIPT></template><meta name="m11" content="c"><template><p>End</p>"#,
        "![A6](p.gif)\n",
        "![A6](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><template/><img src="p.gif" alt="A2"></template><head></NOSCRIPT></template></head></head><TEMPLATE shadowrootmode="open"><TEMPLATE shadowrootmode="open"><template/></head><p>End</p>"#,
        "![A2](p.gif)\n",
        "End\n",
        "End\n",
    ],
    [
        r#"<TEMPLATE shadowrootmode="open"><body><head></NOSCRIPT></template><title>T5</title><template></head><img src="p.gif" alt="A8"><p>End</p>"#,
        "",
        "T5\n",
        "![A8](p.gif)\n\nEnd\n",
    ],
    [
        r#"<TEMPLATE shadowrootmode="open"><noscript><noscript><div hidden>H3</div></NOSCRIPT></template></NOSCRIPT><head><div hidden>H8</div><p>End</p>"#,
        "End\n",
        "",
        "H3\n\nH8\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><TEMPLATE shadowrootmode="open"><head><template/><TEMPLATE shadowrootmode="open"></head><noscript></head><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><noscript/><img src="p.gif" alt="A2"><template><head><noscript/></head></head><div hidden>H8</div><p>End</p>"#,
        "![A2](p.gif)\n",
        "![A2](p.gif)\n",
        "H8\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><noscript></head><body><div hidden>H5</div><div hidden>H6</div><img src="p.gif" alt="A7"><noscript><noscript/><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "H5\n\nH6\n\n![A7](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head></div><title>T2</title></NOSCRIPT><img src="p.gif" alt="A4"></head></head></NOSCRIPT></template><noscript><meta name="m10" content="c"></head><template/><p>End</p>"#,
        "![A4](p.gif)\n",
        "![A4](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><template/></template><TEMPLATE shadowrootmode="open"><noscript/></head><img src="p.gif" alt="A7"><TEMPLATE shadowrootmode="open"></NOSCRIPT></NOSCRIPT><template><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)![A7](p.gif)\n",
        "![A7](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><noscript><img src="p.gif" alt="A3"></head><template/><noscript><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "End\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"><noscript></template><template><meta name="m5" content="c"></template></head><p>W8</p><div hidden>H9</div><div hidden>H10</div><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "W8\n\nH9\n\nH10\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><noscript/><template/><img src="p.gif" alt="A5"><template><template/><TEMPLATE shadowrootmode="open"><p>W9</p><template><p>W11</p><p>End</p>"#,
        "![A1](p.gif)![A5](p.gif)\n",
        "![A1](p.gif)![A5](p.gif)\n",
        "![A5](p.gif)\n\nW9\n\nW11\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><template/><template><TEMPLATE shadowrootmode="open"></NOSCRIPT></div></div><title>T9</title><template/><p>End</p>"#,
        "![A1](p.gif)\n",
        "![A1](p.gif)\n",
        "T9\n\nEnd\n",
    ],
    [
        r#"<noscript/></head><TEMPLATE shadowrootmode="open"><noscript/><title>T4</title></div><body></head><div><template/><noscript><p>End</p>"#,
        "",
        "T4\n",
        "End\n",
    ],
    [
        r#"<TEMPLATE shadowrootmode="open"><noscript><noscript/></div><template/><div></template><noscript/><img src="p.gif" alt="A8"><p>W9</p><meta name="m10" content="c"><p>End</p>"#,
        "![A8](p.gif)\n\nW9\n\nEnd\n",
        "",
        "![A8](p.gif)\n\nW9\n\nEnd\n",
    ],
    [
        r#"<p>W0</p></div><TEMPLATE shadowrootmode="open"><title>T3</title><noscript><meta name="m5" content="c"><noscript/></template><div hidden>H8</div><p>End</p>"#,
        "W0\n\nEnd\n",
        "W0\n\nT3\n",
        "W0\n\nT3\n\nH8\n\nEnd\n",
    ],
    [
        r#"<head></template><img src="p.gif" alt="A2"><img src="p.gif" alt="A3"></head></NOSCRIPT><template><img src="p.gif" alt="A7"><noscript/></template><TEMPLATE shadowrootmode="open"><img src="p.gif" alt="A11"></template><p>End</p>"#,
        "![A2](p.gif)![A3](p.gif)\n\nEnd\n",
        "![A11](p.gif)\n\nEnd\n",
        "![A7](p.gif)![A11](p.gif)\n\nEnd\n",
    ],
    [
        r#"<head><meta name="m1" content="c"><img src="p.gif" alt="A2"></head><p>W4</p><template></NOSCRIPT><img src="p.gif" alt="A7"></head><template><template><template/><p>End</p>"#,
        "![A2](p.gif)\n\nW4\n",
        "![A2](p.gif)\n\nW4\n",
        "W4\n\n![A7](p.gif)\n\nEnd\n",
    ],
    [
        r#"</head><head><noscript/><title>T3</title><head></div><img src="p.gif" alt="A6"><meta name="m7" content="c"><img src="p.gif" alt="A8"><title>T9</title><template><div><p>End</p>"#,
        "T3![A6](p.gif)![A8](p.gif)T9\n",
        "T3![A6](p.gif)![A8](p.gif)T9\n",
        "![A6](p.gif)![A8](p.gif)T9\n\nEnd\n",
    ],
    [
        r#"<head><img src="p.gif" alt="A1"></head><p>W3</p></head><TEMPLATE shadowrootmode="open"><title>T6</title><meta name="m7" content="c"><div hidden>H8</div><p>End</p>"#,
        "![A1](p.gif)\n\nW3\n",
        "W3\n\nT6\n\nH8\n\nEnd\n",
        "W3\n\nT6\n\nH8\n\nEnd\n",
    ],
];

// ~keep The named limit of issue #753: a wider choice can write less than a narrower one when a
// ~keep `<template>` or a `<noscript>` start tag has no end tag, or when the head holds an element
// ~keep that is not metadata. The generated-document test leaves the two page classes out
// ~keep (`is_under_the_named_limit`). These are the 34 pages on which it failed, with what each
// ~keep writes today: 35 checks, one page fails both. When the limit goes (the follow-up issue of
// ~keep #753 for these two classes), this test and the exclusion go with it.
#[test]
fn should_write_what_the_pages_of_the_named_limit_write_today() {
    let mut failures = Vec::new();
    let mut pages_that_lose_a_word = 0;
    for [html, expected @ ..] in NAMED_LIMIT_PAGES {
        let outputs = CHOICES.map(|hidden_content| {
            let options = ConversionOptions {
                hidden_content,
                extract_metadata: false,
                ..ConversionOptions::default()
            };
            convert(html, Some(options))
                .expect("conversion should succeed")
                .content
                .unwrap_or_default()
        });
        check(
            &mut failures,
            outputs.each_ref().map(String::as_str) == *expected,
            || format!("{html}: {outputs:?}"),
        );
        let mut lost = Vec::new();
        check_that_no_output_loses_a_word(&mut lost, html, &outputs);
        pages_that_lose_a_word += usize::from(!lost.is_empty());
    }
    assert_eq!(
        pages_that_lose_a_word,
        NAMED_LIMIT_PAGES.len(),
        "every page here is one on which a wider choice loses a word"
    );
    report(&failures, NAMED_LIMIT_PAGES.len());
}
