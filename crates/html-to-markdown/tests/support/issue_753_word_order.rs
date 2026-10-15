// ~keep Included by `issue_753_hidden_content_test.rs`: the tests of the order of the three choices.
// ~keep `drop` writes the least and `all` the most, so every word that a narrower choice writes is
// ~keep written by a wider one, in the same order.

/// The words of an output in the order in which they are written: each run of letters and each
/// run of digits.
///
/// ~keep A number is a word of its own. Kept text can stand directly before a number that was
/// ~keep always written (a hidden price before the shown price), and the two are still two words.
fn words(text: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    let mut previous = None;
    for (position, character) in text.char_indices().chain([(text.len(), ' ')]) {
        let class = (character.is_alphabetic(), character.is_numeric());
        if previous != Some(class) {
            if matches!(previous, Some((true, _) | (_, true))) {
                words.push(&text[start..position]);
            }
            start = position;
            previous = Some(class);
        }
    }
    words
}

/// The first word of `narrow` that `wide` does not hold in the same order, with its position.
fn first_lost_word<'a>(narrow: &[&'a str], wide: &[&str]) -> Option<(usize, &'a str)> {
    let mut rest = wide.iter();
    narrow
        .iter()
        .enumerate()
        .find(|(_, word)| !rest.any(|other| other == *word))
        .map(|(position, word)| (position, *word))
}

/// Check that a wider choice writes every word of a narrower choice, in the same order.
fn check_that_no_output_loses_a_word(failures: &mut Vec<String>, name: &str, [drop, reachable, all]: &[String; 3]) {
    for (narrow, wide, label) in [
        (drop, reachable, "reachable loses a word of drop"),
        (reachable, all, "all loses a word of reachable"),
    ] {
        let lost = first_lost_word(&words(narrow), &words(wide));
        check(failures, lost.is_none(), || format!("{name}: {label}: {lost:?}"));
    }
}

fn check_that_no_choice_loses_a_word(failures: &mut Vec<String>, name: &str, html: &str, mode: Mode) {
    let outputs = CHOICES.map(|choice| convert_in(html, choice, mode));
    check_that_no_output_loses_a_word(failures, &format!("{name} {mode:?}"), &outputs);
}

#[test]
fn should_keep_the_body_when_a_noscript_in_the_head_holds_an_image() {
    // ~keep A tracking pixel in the head: `<img>` is not metadata, and the `<noscript>` that holds
    // ~keep it is still in the head. A browser with scripting reads its content as text.
    let pixel = r#"<html><head><title>t</title><noscript><img src="p.gif" alt="pixel"></noscript><meta name="a" content="b"></head><body><p>Shown</p><template><p>Body template</p></template></body></html>"#;
    let [drop, reachable, all] = all_choices(pixel);
    assert_eq!(drop, "---\nmeta-a: b\ntitle: t\n---\n\nShown\n");
    assert_eq!(reachable, drop);
    assert_eq!(all, format!("{drop}\nBody template\n"));

    let shadow = r#"<html><head><noscript><img src="p.gif" alt="pixel"></noscript></head><body><p>Shown</p><div><template shadowrootmode="open"><p>Shadow text</p></template></div></body></html>"#;
    let [drop, reachable, all] = all_choices(shadow);
    assert_eq!(drop, "Shown\n");
    assert_eq!(reachable, "Shown\n\nShadow text\n");
    assert_eq!(all, reachable);
}

#[test]
fn should_keep_the_body_of_a_recorded_page_with_tracking_pixels_in_its_head() {
    let path = support::corpus_root().join("issues/gh-190/firsteigen.html");
    let html = std::fs::read_to_string(&path).expect("the recorded page is read");
    let head = &html[..html.find("</head>").expect("the page has a head end tag")];
    assert_eq!(head.matches("<noscript><img").count(), 2, "two pixels in the head");
    let mut failures = Vec::new();
    for mode in modes() {
        check_that_no_choice_loses_a_word(&mut failures, "firsteigen", &html, mode);
    }
    report(&failures, modes().len() * 2);
    let drop = convert_with(&html, HiddenContent::Drop);
    assert!(
        words(&drop).len() > 800,
        "the default output holds the page: {} words",
        words(&drop).len()
    );
}

#[test]
fn should_drop_every_template_and_noscript_in_the_document_head() {
    // ~keep The content of a `<template>` or a `<noscript>` in the head is not the content of the
    // ~keep head: an element there that the head cannot hold does not start the body.
    for html in [
        "<html><head><title>t</title><template><p>HEAD one</p></template><template><p>HEAD two</p></template></head><body><p>Shown</p></body></html>",
        "<html><head><title>t</title><template><div>HEAD one</div></template><noscript><p>HEAD two</p></noscript></head><body><p>Shown</p></body></html>",
        r#"<html><head><title>t</title><noscript><img src="p.gif" alt="HEAD"></noscript><noscript><p>HEAD two</p></noscript></head><body><p>Shown</p></body></html>"#,
        "<html><head><title>t</title><template><p>HEAD one</p><template><p>HEAD two</p></template><p>HEAD three</p></template><template><p>HEAD four</p></template></head><body><p>Shown</p></body></html>",
        r#"<HTML><HEAD><TITLE>t</TITLE><TEMPLATE ID="a"><P>HEAD one</P></TEMPLATE><NOSCRIPT CLASS="b"><IMG SRC="p.gif" ALT="HEAD"></NOSCRIPT><TEMPLATE><P>HEAD two</P></TEMPLATE></HEAD><BODY><P>Shown</P></BODY></HTML>"#,
    ] {
        let [drop, reachable, all] = all_choices(html);
        assert_eq!(drop, "---\ntitle: t\n---\n\nShown\n", "{html}");
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, drop, "{html}");
    }
    // ~keep The body after such a head is read as a body: its own template is kept.
    for html in [
        "<html><head><title>t</title><template><p>HEAD one</p><template><p>HEAD two</p></template></template></head><body><p>Shown</p><template><p>BODY</p></template></body></html>",
        r#"<html><head><title>t</title><noscript><img src="p.gif" alt="HEAD"></noscript><template><p>HEAD two</p></template><p>Shown</p><noscript><p>BODY</p></noscript>"#,
    ] {
        let [drop, reachable, all] = all_choices(html);
        assert!(
            drop.ends_with("\n\nShown\n") && !drop.contains("HEAD"),
            "{html}: {drop:?}"
        );
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, format!("{drop}\nBODY\n"), "{html}");
    }
}

#[test]
fn should_read_a_head_tag_in_the_body_as_no_head() {
    // ~keep A browser ignores a `<head>` tag after the first element of the body.
    let html = "<p>Shown</p><head><template><p>BODY</p></template></head><p>After</p>";
    let [drop, reachable, all] = all_choices(html);
    assert_eq!(drop, "Shown\n\nAfter\n");
    assert_eq!(reachable, drop);
    assert_eq!(all, "Shown\n\nBODY\n\nAfter\n");
}

#[test]
fn should_keep_the_text_after_a_template_or_a_noscript_of_every_shape() {
    // ~keep The two tags of one element are removed together or not at all. A start tag that
    // ~keep stays with its end tag removed holds the rest of the page, and the walk drops it.
    let elements = [
        "<template><p>IN</p></template>",
        "<noscript><p>IN</p></noscript>",
        r#"<TEMPLATE ID="a"><P>IN</P></TEMPLATE>"#,
        r#"<NoScript class="a"><img src="p.gif" alt="IN"></NoScript>"#,
        "<template><template><p>IN</p></template></template>",
        "<noscript><noscript><p>IN</p></noscript></noscript>",
        "<template><noscript><p>IN</p></noscript></template><noscript><template><p>IN</p></template></noscript>",
        "<template><p>IN</p></template><template><p>IN</p></template><noscript><p>IN</p></noscript><noscript><p>IN</p></noscript>",
        "</noscript></template><template><p>IN</p></template>",
        // ~keep A start tag that is written as self-closing, before an element with two tags.
        "<template/><p>Mid</p><template><p>IN</p></template>",
        "<noscript/><p>Mid</p><noscript><p>IN</p></noscript>",
        r#"<template id="a" /><noscript class="b" /><p>Mid</p><noscript><template><p>IN</p></template></noscript>"#,
    ];
    let contexts = [
        (
            "<html><head><title>t</title>",
            "</head><body><p>Shown</p></body></html>",
        ),
        ("<html><head>", "<p>Shown</p>"),
        ("<html><head></head><body><p>First</p>", "<p>Shown</p></body></html>"),
        ("<p>First</p>", "<p>Shown</p>"),
        ("<table><tr><td>First</td></tr>", "</table><p>Shown</p>"),
    ];
    let mut failures = Vec::new();
    let mut total = 0;
    let mut shown = 0;
    for (open, close) in contexts {
        for element in elements {
            let html = format!("{open}{element}{close}");
            for mode in modes() {
                let outputs = CHOICES.map(|choice| convert_in(&html, choice, mode));
                for (choice, output) in CHOICES.iter().zip(&outputs) {
                    total += 1;
                    // ~keep Where `drop` has no text at all (#807), no choice is asked for it.
                    check(
                        &mut failures,
                        output.contains("Shown") || !outputs[0].contains("Shown"),
                        || format!("{html} {mode:?} {choice:?}: {output:?}"),
                    );
                }
                shown += usize::from(outputs[0].contains("Shown"));
                total += 2;
                check_that_no_choice_loses_a_word(&mut failures, &html, &html, mode);
            }
        }
    }
    // ~keep An element with no end tag holds the rest of the input under `drop`. A wider choice
    // ~keep that removes its start tag writes more, never less.
    for html in [
        "<p>Shown</p><template><p>IN</p>",
        "<p>Shown</p><noscript><p>IN</p>",
        "<head><title>t</title><template><p>IN</p></head><body><p>Shown</p>",
        "<head><title>t</title><noscript><p>IN</p></head><body><p>Shown</p>",
    ] {
        for mode in modes() {
            total += 2;
            check_that_no_choice_loses_a_word(&mut failures, html, html, mode);
        }
    }
    assert!(
        shown >= 2 * contexts.len() * elements.len(),
        "`drop` writes the text after the element in only {shown} runs"
    );
    report(&failures, total);
}

#[test]
fn should_write_every_word_of_a_narrower_choice_on_the_recorded_pages() {
    // ~keep A wider choice that loses a word of a narrower one has dropped content that was never
    // ~keep hidden. Markdown from each converter; the generated documents cover the plain format.
    //
    // ~keep The Wikipedia pages are not read. They take most of the time of the whole set, and each
    // ~keep hidden form that they hold (a `<noscript>` in the body, the `hidden` attribute,
    // ~keep `display: none`) is on a smaller page. The two counts below prove that the pages that
    // ~keep are read exercise both steps of the order.
    let root = support::corpus_root();
    let mut pages = Vec::new();
    support::collect_html_files(&root, &mut pages);
    pages.retain(|page| !page.starts_with(root.join("wikipedia")));
    pages.sort();
    let markdown_modes: Vec<Mode> = modes()
        .into_iter()
        .filter(|mode| matches!(mode.output_format, OutputFormat::Markdown))
        .collect();
    let mut failures = Vec::new();
    let mut total = 0;
    let mut reachable_is_wider = 0;
    let mut all_is_wider = 0;
    for page in &pages {
        let html = std::fs::read_to_string(page).expect("the recorded page is read");
        let name = page.strip_prefix(&root).unwrap_or(page).display().to_string();
        for mode in &markdown_modes {
            let outputs = CHOICES.map(|choice| convert_in(&html, choice, *mode));
            total += 2;
            check_that_no_output_loses_a_word(&mut failures, &format!("{name} {mode:?}"), &outputs);
            let [drop, reachable, all] = outputs.each_ref().map(|output| words(output).len());
            reachable_is_wider += usize::from(reachable > drop);
            all_is_wider += usize::from(all > reachable);
        }
    }
    assert!(
        pages.len() >= 60,
        "the recorded pages were not found: {} pages",
        pages.len()
    );
    assert!(
        reachable_is_wider >= 12,
        "`reachable` writes more than `drop` in only {reachable_is_wider} runs"
    );
    assert!(
        all_is_wider >= 10,
        "`all` writes more than `reachable` in only {all_is_wider} runs"
    );
    report(&failures, total);
}

#[test]
fn should_take_the_tree_builder_under_a_wider_choice_when_a_narrower_choice_takes_it() {
    // ~keep The head repair moves an authored image into the body before each hidden choice.
    let image = "![A](p.gif)\n";
    let both = "![A](p.gif)\n\nEnd\n";
    for (html, expected) in [
        (
            r#"<head><img src="p.gif" alt="A"><template></head><p>End</p>"#,
            [image, image, both],
        ),
        (
            r#"<head><img src="p.gif" alt="A"><TEMPLATE shadowrootmode="open"></head><p>End</p>"#,
            [image, both, both],
        ),
        (
            r#"<head><img src="p.gif" alt="A"><noscript></head><p>End</p>"#,
            [image, image, both],
        ),
        // ~keep The image remains visible even when the following template has no end tag.
        (
            r#"<head><img src="p.gif" alt="A"></head><template><p>End</p>"#,
            [image, image, both],
        ),
        (
            r#"<head><img src="p.gif" alt="A"></head><div><template></div></template><p>End</p>"#,
            [both, both, both],
        ),
    ] {
        assert_eq!(all_choices(html), expected, "{html}");
    }
    let html = r#"<head><img src="p.gif" alt="A"></head><p>End</p><template><p>IN</p></template>"#;
    assert_eq!(all_choices(html), [both, both, "![A](p.gif)\n\nEnd\n\nIN\n"]);
}

#[test]
fn should_start_the_body_where_a_narrower_choice_starts_it() {
    // ~keep The tree builder puts a `<title>` that precedes the body in the head, and a title in
    // ~keep the body is text. A kept element can be what starts the body: a `<noscript>` after the
    // ~keep end tag of the head, the first `<template>` of a fragment, a hidden element. A wider
    // ~keep choice removes the tags of that element, and the body still starts there.
    //
    // ~keep Each page has an element with no end tag, so every choice takes the tree builder. The
    // ~keep first parse holds each title in another element, so the head repair leaves the page.
    // ~keep The outputs have no front matter: a title of the head is then written nowhere.
    //
    // ~keep Each row names the narrowest choice that writes the title. The inert-body pass reads
    // ~keep `<template/>` and `<noscript/>` as start tags, so the first three pages are the
    // ~keep content of one such element: `drop` and `reachable` write nothing, and `all` writes
    // ~keep the title.
    let (reachable, all) = (1, 2);
    let pages = [
        // ~keep Three of the generated documents.
        (
            r#"<template/></div></head><noscript/><TEMPLATE shadowrootmode="open"><title>T5</title><template><div hidden>H7</div><noscript><img src="p.gif" alt="A9"><p>End</p>"#,
            "T5",
            all,
        ),
        (
            r#"<noscript/></head><TEMPLATE shadowrootmode="open"><noscript/><title>T4</title></div><body></head><div><template/><noscript><p>End</p>"#,
            "T4",
            all,
        ),
        (
            r#"</head><head><noscript/><title>T3</title><head></div><img src="p.gif" alt="A6"><meta name="m7" content="c"><img src="p.gif" alt="A8"><title>T9</title><template><div><p>End</p>"#,
            "T3",
            all,
        ),
        // ~keep A `<noscript>` after the end tag of the head, before a title of the page and
        // ~keep before a title of a shadow root.
        (
            "</head><head><noscript></noscript><title>T1</title><div><p>End</p>",
            "T1",
            reachable,
        ),
        (
            r#"</head><noscript></noscript><template shadowrootmode="open"><title>T2</title></template><div><p>End</p>"#,
            "T2",
            reachable,
        ),
        // ~keep The first `<template>` of a fragment, and a shadow root in that place.
        (
            r#"<template></template></head><noscript></noscript><template shadowrootmode="open"><title>T6</title></template><div><p>End</p>"#,
            "T6",
            reachable,
        ),
        (
            r#"<template shadowrootmode="open"><title>T7</title></template><div><p>End</p>"#,
            "T7",
            reachable,
        ),
        // ~keep A hidden element, which keeps its tags under each choice that keeps it.
        (
            "</head><head><div hidden></div><title>T8</title><div><p>End</p>",
            "T8",
            reachable,
        ),
    ];
    let mut failures = Vec::new();
    for (html, title, narrowest) in pages {
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
        check(&mut failures, outputs[narrowest].contains(title), || {
            format!(
                "{html}: {:?} does not write {title}: {:?}",
                CHOICES[narrowest], outputs[narrowest]
            )
        });
        check(
            &mut failures,
            narrowest == reachable || outputs[reachable].is_empty(),
            || format!("{html}: `reachable` writes {:?}", outputs[reachable]),
        );
        check_that_no_output_loses_a_word(&mut failures, html, &outputs);
    }
    report(&failures, pages.len() * 4);
}

#[test]
fn should_empty_a_kept_element_of_the_head_like_a_narrower_choice() {
    // ~keep The inert-body pass leaves a kept element as written only in the body. The later
    // ~keep pass leaves the tags of an element of the head, so the content of that element, read
    // ~keep as markup, held the text after it. The first three pages are generated documents.
    for html in [
        "<head><noscript><head></NOSCRIPT><p>End</p>",
        "<head><noscript><body></NOSCRIPT><p>End</p>",
        "<head><noscript><div></NOSCRIPT><p>End</p>",
        "<head><noscript><div>IN</NOSCRIPT><p>End</p>",
        "<head><template><div>IN</template><p>End</p>",
        r#"<head><TEMPLATE shadowrootmode="open"><div>IN</template><p>End</p>"#,
        "<head><iframe><div>IN</iframe><p>End</p>",
    ] {
        assert_eq!(all_choices(html), ["End\n", "End\n", "End\n"], "{html}");
    }
    // ~keep In the body each choice writes the content that it keeps. No choice keeps the
    // ~keep content of an `<iframe>`.
    let (dropped, kept) = ("First\n\nEnd\n", "First\n\nIN\n\nEnd\n");
    for (html, expected) in [
        (
            "<p>First</p><noscript><div>IN</NOSCRIPT><p>End</p>",
            [dropped, dropped, kept],
        ),
        (
            "<p>First</p><template><div>IN</template><p>End</p>",
            [dropped, dropped, kept],
        ),
        (
            r#"<p>First</p><TEMPLATE shadowrootmode="open"><div>IN</template><p>End</p>"#,
            [dropped, kept, kept],
        ),
        (
            "<p>First</p><iframe><div>IN</iframe><p>End</p>",
            [dropped, dropped, dropped],
        ),
    ] {
        assert_eq!(all_choices(html), expected, "{html}");
    }
}

/// The output of each choice with no front matter: a title of the head is then written nowhere.
fn without_front_matter(html: &str) -> [String; 3] {
    CHOICES.map(|hidden_content| {
        let options = ConversionOptions {
            hidden_content,
            extract_metadata: false,
            ..ConversionOptions::default()
        };
        convert(html, Some(options))
            .expect("conversion should succeed")
            .content
            .unwrap_or_default()
    })
}

#[test]
fn should_write_a_title_under_a_wider_choice_where_a_narrower_choice_writes_it() {
    // ~keep The walk writes the title of a `<head>` that holds a paragraph as text. The kept
    // ~keep content has an element with no end tag, which asks for the tree builder, and the
    // ~keep tree builder puts the title in the head. No narrower choice takes the tree builder
    // ~keep here, so the wider choice keeps its first parse. The first page of each group is a
    // ~keep generated document.
    for html in [
        r#"<head><title>T1</title><meta name="m2" content="c"></div><p>W4</p></head></NOSCRIPT></div><TEMPLATE shadowrootmode="open"><template/><body><div><noscript/><p>End</p>"#,
        "<head><title>T1</title><p>W4</p></head><template><div><p>End</p></template>",
        "<head><title>T1</title><p>W4</p></head><noscript><div><p>End</p></noscript>",
    ] {
        let [drop, reachable, all] = without_front_matter(html);
        assert_eq!(drop, "T1\n\nW4\n", "{html}");
        assert_eq!(reachable, drop, "{html}");
        assert_eq!(all, "T1\n\nW4\n\nEnd\n", "{html}");
    }
    // ~keep Where `drop` takes the tree builder, each choice takes it: no choice writes the title.
    let html = r#"<head><title>T1</title><p>W4</p></head><div><TEMPLATE shadowrootmode="open"><div><p>End</p></template></div>"#;
    let [drop, reachable, all] = without_front_matter(html);
    assert_eq!(drop, "W4\n", "{html}");
    assert_eq!(reachable, "W4\n\nEnd\n", "{html}");
    assert_eq!(all, reachable, "{html}");
    let html = "<head><title>T1</title><p>W4</p></head><div><p>Mid</p><template><div><p>End</p></template>";
    let [drop, reachable, all] = without_front_matter(html);
    assert_eq!(drop, "W4\n\nMid\n", "{html}");
    assert_eq!(reachable, drop, "{html}");
    assert_eq!(all, "W4\n\nMid\n\nEnd\n", "{html}");
    // ~keep A `<head>` tag in kept content is no head of the page: the tree builder ignores it in
    // ~keep the body. It sends no choice to the head repair that `drop` does not take.
    let html =
        r#"<template/><head><head></template><TEMPLATE shadowrootmode="open"><title>T5</title><noscript/><p>End</p>"#;
    let [drop, reachable, all] = without_front_matter(html);
    assert_eq!(drop, "", "{html}");
    assert_eq!(reachable, "T5\n", "{html}");
    assert_eq!(all, "T5\n\nEnd\n", "{html}");
}

/// The pieces of the generated documents. `{}` takes a word that no other piece of the document has.
const PIECES: &[&str] = &[
    "<head>",
    "</head>",
    "<body>",
    "<title>T{}</title>",
    r#"<meta name="m{}" content="c">"#,
    "<template>",
    "</template>",
    r#"<TEMPLATE shadowrootmode="open">"#,
    "<template/>",
    "<noscript>",
    "</NOSCRIPT>",
    "<noscript/>",
    "<p>W{}</p>",
    r#"<img src="p.gif" alt="A{}">"#,
    "<div hidden>H{}</div>",
    "<div>",
    "</div>",
];

fn generated_document(pieces: &[usize]) -> String {
    let mut html = String::new();
    for (position, piece) in pieces.iter().enumerate() {
        html.push_str(&PIECES[*piece].replace("{}", &position.to_string()));
    }
    html.push_str("<p>End</p>");
    html
}

#[test]
fn should_write_every_word_of_a_narrower_choice_on_generated_documents() {
    let mut failures = Vec::new();
    let mut total = 0;
    // ~keep The front matter is metadata, not text: a kept element before the `<head>` tag makes
    // ~keep the head a part of the body, as the same element does when it is not hidden.
    let outputs = |html: &str| {
        CHOICES.map(|hidden_content| {
            let options = ConversionOptions {
                hidden_content,
                extract_metadata: false,
                ..ConversionOptions::default()
            };
            convert(html, Some(options))
                .expect("conversion should succeed")
                .content
                .unwrap_or_default()
        })
    };
    // ~keep Every document of up to three pieces, after nothing and after an open head.
    for prefix in [&[][..], &[0][..]] {
        for len in 0..=3u32 {
            for mut code in 0..PIECES.len().pow(len) {
                let mut pieces = prefix.to_vec();
                for _ in 0..len {
                    pieces.push(code % PIECES.len());
                    code /= PIECES.len();
                }
                let html = generated_document(&pieces);
                total += 2;
                check_that_no_output_loses_a_word(&mut failures, &html, &outputs(&html));
            }
        }
    }
    // ~keep Longer documents from a fixed sequence of numbers, half of them after an open head.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = |bound: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).unwrap_or(0) % bound
    };
    for document in 0..6000 {
        let mut pieces = if document % 2 == 0 { vec![0] } else { Vec::new() };
        for _ in 0..4 + next(9) {
            pieces.push(next(PIECES.len()));
        }
        let html = generated_document(&pieces);
        total += 2;
        check_that_no_output_loses_a_word(&mut failures, &html, &outputs(&html));
    }
    assert!(total > 25_000, "only {total} checks");
    report(&failures, total);
}
