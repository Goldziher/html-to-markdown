use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn footer_without_navigation_hint_is_preserved() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
  <body>
    <main>
      <h1>Simple Webpage</h1>
      <p>This is a simple webpage without external images.</p>
    </main>
    <footer>
      <p>Test page for processors validation</p>
    </footer>
  </body>
</html>"#;

    let markdown = convert(html, None).unwrap();
    assert!(
        markdown.contains("Test page for processors validation"),
        "footer content should be retained in markdown:\n{}",
        markdown
    );
}

#[test]
fn footer_with_navigation_hint_is_removed() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
  <body>
    <main>
      <h1>Simple Webpage</h1>
    </main>
    <footer class="site-footer">
      <p>Test page for processors validation</p>
      <nav><a href="/about">About</a></nav>
    </footer>
  </body>
</html>"#;

    let mut options = ConversionOptions::default();
    options.preprocessing.enabled = true;
    let markdown = convert(html, Some(options)).unwrap();
    assert!(
        !markdown.contains("processors validation"),
        "navigational footers should still be stripped entirely:\n{}",
        markdown
    );
}
