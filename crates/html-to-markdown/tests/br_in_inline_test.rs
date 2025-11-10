use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_br_inside_bold_tags() {
    let html = "<b>Hello!<br/></b><b>Hola!</b>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("**Hello!**  \n**Hola!**"));
    assert!(!result.contains("**Hello!****Hola!**"));
}

#[test]
fn test_br_inside_strong_tags() {
    let html = "<strong>First<br/></strong><strong>Second</strong>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("**First**  \n**Second**"));
    assert!(!result.contains("**First****Second**"));
}

#[test]
fn test_multiple_bolds_with_br() {
    let html = "<b>Line 1<br/></b><b>Line 2<br/></b><b>Line 3</b>";
    let result = convert(html, None).unwrap();

    assert!(result.matches("**").count() >= 6);
    assert!(result.contains("**Line 1**"));
    assert!(result.contains("**Line 2**"));
    assert!(result.contains("**Line 3**"));
    assert!(result.contains("**Line 1**  \n**Line 2**"));
}

#[test]
fn test_br_inside_em_tags() {
    let html = "<em>First<br/></em><em>Second</em>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("*First*  \n*Second*"));
    assert!(!result.contains("*First**Second*"));
}

#[test]
fn test_br_inside_italic_tags() {
    let html = "<i>Alpha<br/></i><i>Beta</i>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("*Alpha*  \n*Beta*"));
    assert!(!result.contains("*Alpha**Beta*"));
}

#[test]
fn test_br_with_backslash_style() {
    let html = "<b>Hello<br/></b><b>World</b>";
    let mut options = ConversionOptions::default();
    options.newline_style = html_to_markdown_rs::NewlineStyle::Backslash;
    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("**Hello**\\\n**World**"));
    assert!(!result.contains("**Hello****World**"));
}

#[test]
fn test_br_inside_nested_formatting() {
    let html = "<b><i>Bold italic<br/></i></b><b>Just bold</b>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("***Bold italic***  \n**Just bold**"));
}

#[test]
fn test_br_at_end_of_paragraph_with_bold() {
    let html = "<p><b>Line 1<br/></b><b>Line 2</b></p>";
    let result = convert(html, None).unwrap();

    assert!(result.contains("**Line 1**  \n**Line 2**"));
}
