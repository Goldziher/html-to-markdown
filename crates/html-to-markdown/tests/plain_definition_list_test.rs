#![allow(missing_docs)]

use html_to_markdown_rs::{ConversionOptions, OutputFormat, convert};

fn plain(html: &str, selectors: &[&str]) -> String {
    let options = ConversionOptions {
        output_format: OutputFormat::Plain,
        exclude_selectors: selectors.iter().map(|selector| (*selector).to_string()).collect(),
        ..Default::default()
    };
    convert(html, Some(options)).unwrap().content.unwrap_or_default()
}

#[test]
fn should_write_each_term_and_each_description_as_a_block() {
    assert_eq!(
        plain(
            "<dl><dt>Bread</dt><dd>Baked dough.</dd><dt>Cheese</dt><dd>Pressed milk curd.</dd></dl>",
            &[]
        ),
        "Bread\n\nBaked dough.\n\nCheese\n\nPressed milk curd.\n"
    );
}

#[test]
fn should_separate_a_definition_list_from_the_blocks_around_it() {
    assert_eq!(
        plain(
            "<h2>Details</h2><dl><dt>Weight</dt><dd>800 g</dd><dt>Price</dt><dd>4.50</dd></dl><p>Free delivery.</p>",
            &[]
        ),
        "Details\n\nWeight\n\n800 g\n\nPrice\n\n4.50\n\nFree delivery.\n"
    );
}

#[test]
fn should_write_two_descriptions_of_one_term_as_two_blocks() {
    assert_eq!(
        plain("<dl><dt>Flour</dt><dd>Ground wheat.</dd><dd>Ground rye.</dd></dl>", &[]),
        "Flour\n\nGround wheat.\n\nGround rye.\n"
    );
}

#[test]
fn should_start_a_definition_list_on_the_line_of_its_list_item() {
    assert_eq!(
        plain(
            "<ul><li><dl><dt>Bread</dt><dd>Baked dough.</dd></dl></li><li>Cheese</li></ul>",
            &[]
        ),
        "- Bread\n\nBaked dough.\n\n- Cheese\n"
    );
}

#[test]
fn should_separate_a_definition_list_from_the_text_of_its_list_item() {
    assert_eq!(
        plain(
            "<ul><li>Terms<dl><dt>Bread</dt><dd>Baked dough.</dd></dl></li></ul>",
            &[]
        ),
        "- Terms\n\nBread\n\nBaked dough.\n"
    );
}

#[test]
fn should_keep_the_words_of_inline_markup_in_a_term_and_a_description() {
    assert_eq!(
        plain(
            r#"<dl><dt><strong>Rye</strong> bread</dt><dd>Made with <a href="/rye">rye flour</a>.</dd></dl>"#,
            &[]
        ),
        "Rye bread\n\nMade with rye flour.\n"
    );
}

#[test]
fn should_write_nothing_for_an_empty_description() {
    assert_eq!(
        plain(
            "<dl><dt>Salt</dt><dd></dd><dt>Sugar</dt><dd>Sweet crystals.</dd></dl>",
            &[]
        ),
        "Salt\n\nSugar\n\nSweet crystals.\n"
    );
}

#[test]
fn should_write_a_term_and_a_description_outside_a_list_as_blocks() {
    assert_eq!(
        plain(
            "<p>Terms follow.</p><dt>Bread</dt><dd>Baked dough.</dd><p>End of terms.</p>",
            &[]
        ),
        "Terms follow.\n\nBread\n\nBaked dough.\n\nEnd of terms.\n"
    );
}

#[test]
fn should_not_write_a_description_that_a_selector_excludes() {
    let expected = "Bread\n\nCheese\n\nPressed milk curd.\n";
    assert_eq!(
        plain(
            r#"<dl><dt>Bread</dt><dd class="note">Baked dough.</dd><dt>Cheese</dt><dd>Pressed milk curd.</dd></dl>"#,
            &[".note"]
        ),
        expected
    );
    assert_eq!(
        plain("<dl><dt>Bread</dt><dt>Cheese</dt><dd>Pressed milk curd.</dd></dl>", &[]),
        expected,
        "the page without the description"
    );
}

#[test]
fn should_write_the_terms_of_each_group_as_blocks() {
    assert_eq!(
        plain(
            "<dl><div><dt>Author</dt><dd>Ada Miller</dd></div><div><dt>Published</dt><dd>5 January</dd></div></dl>",
            &[]
        ),
        "Author\n\nAda Miller\n\nPublished\n\n5 January\n"
    );
}

#[test]
fn should_write_blocks_when_line_ends_stand_between_the_tags() {
    assert_eq!(
        plain(
            "<p>Terms follow.</p>\n<dl>\n<dt>Bread</dt>\n<dd>Baked dough.</dd>\n</dl>\n",
            &[]
        ),
        "Terms follow.\n\nBread\n\nBaked dough.\n"
    );
}

#[test]
fn should_write_a_description_in_a_description_as_a_block() {
    assert_eq!(
        plain("<dl><dd>First reply.<dl><dd>Second reply.</dd></dl></dd></dl>", &[]),
        "First reply.\n\nSecond reply.\n"
    );
}

#[test]
fn should_keep_the_blocks_of_a_description_that_holds_a_paragraph_and_a_list() {
    assert_eq!(
        plain(
            "<dl><dt>Bread</dt><dd><p>Baked dough.</p><ul><li>Rye</li><li>Wheat</li></ul></dd></dl>",
            &[]
        ),
        "Bread\n\nBaked dough.\n\n- Rye\n- Wheat\n"
    );
}
