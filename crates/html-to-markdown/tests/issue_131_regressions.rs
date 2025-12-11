use html_to_markdown_rs::options::WhitespaceMode;
use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn link_flattens_block_children_issue_131() {
    let html = r#"<a href="https://www.google.com">
              <h3>MWD08 - الابيض</h3>
              <p><span class="money" data-cbb-price-processed="true" style="display: none;">70.00 SAR</span><span class="currency-converter-wrapper-amount-box" style="display: inline-block; white-space: nowrap; margin: 0px; padding: 0px; text-decoration: inherit;"><span class="currency-converter-amount-box" style="display: inline-block; white-space: nowrap; padding: 0px; line-height: initial; color: rgb(28, 28, 28); font-size: 14px; font-style: normal; font-weight: 400; text-decoration: rgb(28, 28, 28);"><span style="text-decoration: inherit; display: inline-block; margin-right: 3px; font-family: Nunito, sans-serif; font-size: 14px; font-weight: 400; color: inherit; float: none;" class="currency-converter-amount cbb-price-currency-USD"><span class="cbb-price-symbol" style="padding: 0px 1px; color: inherit; float: none; margin-right: 0px;">$</span><span class="cbb-price-digits" style="padding: 0px 1px; color: inherit; float: none;">18.66</span><span class="cbb-price-code" style="margin-left: 3px; padding: 0px 1px; color: inherit; float: none;">USD</span></span></span></span></p>
            </a>"#;

    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Normalized,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert_eq!(result, "[MWD08 - الابيض 70.00 SAR$18.66USD](https://www.google.com)\n");
}

#[test]
fn link_label_newlines_are_collapsed() {
    let html = r#"<p><a href="https://example.com">line 1
line 2</a></p>"#;

    let options = ConversionOptions {
        whitespace_mode: WhitespaceMode::Normalized,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();
    assert_eq!(result, "[line 1 line 2](https://example.com)\n");
}
