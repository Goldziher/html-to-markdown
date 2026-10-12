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

/// Whether a generated document is of one of the two page classes of the named limit.
///
/// ~keep On such a page a wider choice can write less than a narrower one: the tree builder,
/// ~keep which rebuilds the page, puts the text after an open `<template>` or `<noscript>` and an
/// ~keep element of the head where the walk does not write it. The two classes are a
/// ~keep `<template>` or a `<noscript>` start tag that no later end tag of its name closes, and
/// ~keep an element that is not metadata between a `<head>` tag and the tag that ends that head.
/// ~keep The generated-document test leaves both out, and
/// ~keep `should_write_what_the_pages_of_the_named_limit_write_today` holds the pages that it
/// ~keep found. Both go when the limit goes (the follow-up issue of #753 for these two classes).
fn is_under_the_named_limit(pieces: &[usize]) -> bool {
    let mut open = [0_usize; 2];
    let mut in_head = false;
    let mut other_element_in_head = false;
    for piece in pieces {
        match PIECES[*piece] {
            "<template>" | r#"<TEMPLATE shadowrootmode="open">"# => open[0] += 1,
            "</template>" => open[0] = open[0].saturating_sub(1),
            "<noscript>" => open[1] += 1,
            "</NOSCRIPT>" => open[1] = open[1].saturating_sub(1),
            "<head>" => in_head = true,
            "</head>" | "<body>" => in_head = false,
            other => {
                let is_metadata = !["<p>", "<img", "<div"].iter().any(|start| other.starts_with(start));
                other_element_in_head |= in_head && !is_metadata;
            }
        }
    }
    other_element_in_head || open != [0, 0]
}

#[test]
fn should_write_every_word_of_a_narrower_choice_on_generated_documents() {
    let mut failures = Vec::new();
    let mut total = 0;
    let mut left_out = 0;
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
    // ~keep Every document of up to three pieces, after nothing and after an open head, but the
    // ~keep documents under the named limit.
    for prefix in [&[][..], &[0][..]] {
        for len in 0..=3u32 {
            for mut code in 0..PIECES.len().pow(len) {
                let mut pieces = prefix.to_vec();
                for _ in 0..len {
                    pieces.push(code % PIECES.len());
                    code /= PIECES.len();
                }
                if is_under_the_named_limit(&pieces) {
                    left_out += 1;
                    continue;
                }
                let html = generated_document(&pieces);
                total += 2;
                check_that_no_output_loses_a_word(&mut failures, &html, &outputs(&html));
            }
        }
    }
    // ~keep Longer documents from a fixed sequence of numbers, half of them after an open head:
    // ~keep the first 9000 of the sequence that are not under the named limit.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = |bound: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).unwrap_or(0) % bound
    };
    let mut longer = 0;
    for document in 0.. {
        if longer == 9000 {
            break;
        }
        let mut pieces = if document % 2 == 0 { vec![0] } else { Vec::new() };
        for _ in 0..4 + next(9) {
            pieces.push(next(PIECES.len()));
        }
        if is_under_the_named_limit(&pieces) {
            left_out += 1;
            continue;
        }
        longer += 1;
        let html = generated_document(&pieces);
        total += 2;
        check_that_no_output_loses_a_word(&mut failures, &html, &outputs(&html));
    }
    assert!(total > 25_000, "only {total} checks");
    println!(
        "{} generated documents are checked and {left_out} are left out",
        total / 2
    );
    report(&failures, total);
}
