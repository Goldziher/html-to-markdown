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
fn should_write_the_caption_as_a_block_before_the_rows() {
    assert_eq!(
        plain(
            r#"<p>The table shows the count.</p><table class="wikitable sortable"><caption>Population by year</caption><tbody><tr><th>Year</th><th>People</th></tr><tr><td>1990</td><td>4,100</td></tr><tr><td>2000</td><td>5,300</td></tr></tbody></table><p>Source: the town office.</p>"#,
            &[]
        ),
        "The table shows the count.\n\nPopulation by year\n\nYear\tPeople\n1990\t4,100\n2000\t5,300\n\nSource: the town office.\n"
    );
}

#[test]
fn should_write_the_caption_of_a_table_with_no_row() {
    assert_eq!(
        plain("<table><caption>No data this year</caption></table>", &[]),
        "No data this year\n"
    );
}

#[test]
fn should_keep_the_words_of_inline_markup_in_a_caption() {
    assert_eq!(
        plain(
            "<table><caption><strong>Table 2.</strong> Harvest totals by <em>region</em></caption><tr><td>North</td><td>40 tonnes</td></tr></table>",
            &[]
        ),
        "Table 2. Harvest totals by region\n\nNorth\t40 tonnes\n"
    );
}

#[test]
fn should_not_write_a_caption_that_a_selector_excludes() {
    let html =
        r#"<table><caption class="note">Draft figures</caption><tr><td>North</td><td>40 tonnes</td></tr></table>"#;
    let expected = "North\t40 tonnes\n";
    assert_eq!(plain(html, &[".note"]), expected);
    assert_eq!(plain(html, &["caption"]), expected);
    assert_eq!(
        plain("<table><tr><td>North</td><td>40 tonnes</td></tr></table>", &[]),
        expected,
        "the page without the caption"
    );
}

#[test]
fn should_not_write_the_part_of_a_caption_that_a_selector_excludes() {
    assert_eq!(
        plain(
            r#"<table><caption>Harvest totals <span class="note">(draft)</span></caption><tr><td>North</td><td>40 tonnes</td></tr></table>"#,
            &[".note"]
        ),
        "Harvest totals\n\nNorth\t40 tonnes\n"
    );
}

#[test]
fn should_write_the_caption_of_a_table_in_a_cell_in_that_cell() {
    assert_eq!(
        plain(
            "<table><caption>Bakeries</caption><tr><td>Corner Bakery</td><td><table><caption>Opening hours</caption><tr><td>Monday</td><td>Seven to five</td></tr></table></td></tr></table>",
            &[]
        ),
        "Bakeries\n\nCorner Bakery\tOpening hours\n\nMonday\tSeven to five\n"
    );
}

#[test]
fn should_write_the_caption_before_the_rows_when_the_head_comes_first() {
    assert_eq!(
        plain(
            "<table><thead><tr><th>Year</th><th>People</th></tr></thead><caption>Population by year</caption><tbody><tr><td>1990</td><td>4,100</td></tr></tbody></table>",
            &[]
        ),
        "Population by year\n\nYear\tPeople\n1990\t4,100\n"
    );
}

#[test]
fn should_not_write_the_white_space_around_the_text_of_a_caption() {
    assert_eq!(
        plain(
            "<table>\n<caption>\n  Population by year\n</caption>\n<tr><td>1990</td><td>4,100</td></tr>\n</table>",
            &[]
        ),
        "Population by year\n\n1990\t4,100\n"
    );
}

#[test]
fn should_write_nothing_for_a_caption_with_no_text() {
    assert_eq!(
        plain(
            "<p>Counts follow.</p><table><caption> </caption><tr><td>1990</td><td>4,100</td></tr></table>",
            &[]
        ),
        "Counts follow.\n\n1990\t4,100\n"
    );
}
