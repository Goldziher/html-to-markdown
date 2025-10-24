//! Integration tests for the html-to-markdown CLI.
//!
//! These tests verify the CLI works correctly with various options and edge cases.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cli() -> Command {
    Command::cargo_bin("html-to-markdown").unwrap()
}

#[test]
fn test_basic_stdin() {
    cli()
        .write_stdin("<h1>Title</h1><p>Content</p>")
        .assert()
        .success()
        .stdout("# Title\n\nContent\n");
}

#[test]
fn test_file_input() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.html");
    fs::write(&input_path, "<p>Test content</p>").unwrap();

    cli()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success()
        .stdout("Test content\n");
}

#[test]
fn test_file_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.md");

    cli()
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .write_stdin("<p>Output test</p>")
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    assert_eq!(output, "Output test\n");
}

#[test]
fn test_dash_reads_stdin() {
    cli()
        .arg("-")
        .write_stdin("<p>Dash test</p>")
        .assert()
        .success()
        .stdout("Dash test\n");
}

#[test]
fn test_heading_style_atx() {
    cli()
        .arg("--heading-style")
        .arg("atx")
        .write_stdin("<h1>H1</h1><h2>H2</h2>")
        .assert()
        .success()
        .stdout("# H1\n\n## H2\n");
}

#[test]
fn test_heading_style_underlined() {
    cli()
        .arg("--heading-style")
        .arg("underlined")
        .write_stdin("<h1>H1</h1><h2>H2</h2>")
        .assert()
        .success()
        .stdout(predicate::str::contains("H1\n=="))
        .stdout(predicate::str::contains("H2\n--"));
}

#[test]
fn test_heading_style_atx_closed() {
    cli()
        .arg("--heading-style")
        .arg("atx-closed")
        .write_stdin("<h1>H1</h1>")
        .assert()
        .success()
        .stdout("# H1 #\n");
}

#[test]
fn test_list_indent_width() {
    cli()
        .arg("--list-indent-width")
        .arg("4")
        .write_stdin("<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>")
        .assert()
        .success()
        .stdout(predicate::str::contains("    - Nested"));
}

#[test]
fn test_bullets_option() {
    cli()
        .arg("--bullets")
        .arg("*")
        .write_stdin("<ul><li>Item</li></ul>")
        .assert()
        .success()
        .stdout("* Item\n");
}

#[test]
fn test_strong_em_symbol_asterisk() {
    cli()
        .arg("--strong-em-symbol")
        .arg("*")
        .write_stdin("<p><strong>Bold</strong> <em>italic</em></p>")
        .assert()
        .success()
        .stdout("**Bold** *italic*\n");
}

#[test]
fn test_strong_em_symbol_underscore() {
    cli()
        .arg("--strong-em-symbol")
        .arg("_")
        .write_stdin("<p><strong>Bold</strong> <em>italic</em></p>")
        .assert()
        .success()
        .stdout("__Bold__ _italic_\n");
}

#[test]
fn test_strong_em_symbol_invalid() {
    cli()
        .arg("--strong-em-symbol")
        .arg("x")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("strong_em_symbol must be '*' or '_'"));
}

#[test]
fn test_escape_asterisks() {
    cli()
        .arg("--escape-asterisks")
        .write_stdin("<p>Text with * asterisk</p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\\*"));
}

#[test]
fn test_escape_underscores() {
    cli()
        .arg("--escape-underscores")
        .write_stdin("<p>Text with _ underscore</p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\\_"));
}

#[test]
fn test_escape_misc() {
    cli()
        .arg("--escape-misc")
        .write_stdin("<p>Text with [brackets]</p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\\["));
}

#[test]
fn test_sub_symbol() {
    cli()
        .arg("--sub-symbol")
        .arg("~")
        .write_stdin("<p>H<sub>2</sub>O</p>")
        .assert()
        .success()
        .stdout("H~2~O\n");
}

#[test]
fn test_sup_symbol() {
    cli()
        .arg("--sup-symbol")
        .arg("^")
        .write_stdin("<p>x<sup>2</sup></p>")
        .assert()
        .success()
        .stdout("x^2^\n");
}

#[test]
fn test_newline_style_spaces() {
    cli()
        .arg("--newline-style")
        .arg("spaces")
        .write_stdin("<p>Line 1<br>Line 2</p>")
        .assert()
        .success()
        .stdout("Line 1  \nLine 2\n");
}

#[test]
fn test_newline_style_backslash() {
    cli()
        .arg("--newline-style")
        .arg("backslash")
        .write_stdin("<p>Line 1<br>Line 2</p>")
        .assert()
        .success()
        .stdout("Line 1\\\nLine 2\n");
}

#[test]
fn test_code_language() {
    cli()
        .arg("--code-language")
        .arg("rust")
        .arg("--code-block-style")
        .arg("backticks")
        .write_stdin("<pre><code>fn main() {}</code></pre>")
        .assert()
        .success()
        .stdout(predicate::str::contains("```rust"));
}

#[test]
fn test_code_block_style_indented() {
    cli()
        .arg("--code-block-style")
        .arg("indented")
        .write_stdin("<pre><code>code</code></pre>")
        .assert()
        .success()
        .stdout("    code\n");
}

#[test]
fn test_code_block_style_backticks() {
    cli()
        .arg("--code-block-style")
        .arg("backticks")
        .write_stdin("<pre><code>code</code></pre>")
        .assert()
        .success()
        .stdout("```\ncode\n```\n");
}

#[test]
fn test_code_block_style_tildes() {
    cli()
        .arg("--code-block-style")
        .arg("tildes")
        .write_stdin("<pre><code>code</code></pre>")
        .assert()
        .success()
        .stdout("~~~\ncode\n~~~\n");
}

#[test]
fn test_autolinks() {
    cli()
        .arg("--autolinks")
        .write_stdin("<p><a href=\"https://example.com\">https://example.com</a></p>")
        .assert()
        .success()
        .stdout("<https://example.com>\n");
}

#[test]
fn test_default_title() {
    cli()
        .arg("--default-title")
        .write_stdin("<p><a href=\"https://example.com\">Link</a></p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("[Link](https://example.com)"));
}

#[test]
fn test_keep_inline_images_in() {
    cli()
        .arg("--keep-inline-images-in")
        .arg("a,strong")
        .write_stdin("<a><img src=\"test.jpg\" alt=\"Alt\"></a>")
        .assert()
        .success()
        .stdout(predicate::str::contains("![Alt](test.jpg)"));
}

#[test]
fn test_br_in_tables() {
    cli()
        .arg("--br-in-tables")
        .write_stdin("<table><tr><td>Line 1<br>Line 2</td></tr></table>")
        .assert()
        .success()
        .stdout(predicate::str::contains("<br>"));
}

#[test]
fn test_highlight_style_double_equal() {
    cli()
        .arg("--highlight-style")
        .arg("double-equal")
        .write_stdin("<p><mark>highlighted</mark></p>")
        .assert()
        .success()
        .stdout("==highlighted==\n");
}

#[test]
fn test_highlight_style_html() {
    cli()
        .arg("--highlight-style")
        .arg("html")
        .write_stdin("<p><mark>highlighted</mark></p>")
        .assert()
        .success()
        .stdout("<mark>highlighted</mark>\n");
}

#[test]
fn test_highlight_style_bold() {
    cli()
        .arg("--highlight-style")
        .arg("bold")
        .write_stdin("<p><mark>highlighted</mark></p>")
        .assert()
        .success()
        .stdout("**highlighted**\n");
}

#[test]
fn test_highlight_style_none() {
    cli()
        .arg("--highlight-style")
        .arg("none")
        .write_stdin("<p><mark>highlighted</mark></p>")
        .assert()
        .success()
        .stdout("highlighted\n");
}

#[test]
fn test_extract_metadata() {
    cli()
        .arg("--extract-metadata")
        .write_stdin("<html><head><title>Page Title</title></head><body><p>Content</p></body></html>")
        .assert()
        .success()
        .stdout(predicate::str::contains("Page Title"));
}

#[test]
fn test_whitespace_mode_normalized() {
    cli()
        .arg("--whitespace-mode")
        .arg("normalized")
        .write_stdin("<p>Multiple    spaces</p>")
        .assert()
        .success()
        .stdout("Multiple spaces\n");
}

#[test]
fn test_strip_newlines() {
    cli()
        .arg("--strip-newlines")
        .write_stdin("<p>\nContent\n</p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("Content"));
}

#[test]
fn test_wrap() {
    cli()
        .arg("--wrap")
        .arg("--wrap-width")
        .arg("20")
        .write_stdin("<p>This is a very long line that should be wrapped at 20 characters</p>")
        .assert()
        .success();
}

#[test]
fn test_wrap_width_validation() {
    cli()
        .arg("--wrap-width")
        .arg("10")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure();
}

#[test]
fn test_convert_as_inline() {
    cli()
        .arg("--convert-as-inline")
        .write_stdin("<div>Block 1</div><div>Block 2</div>")
        .assert()
        .success();
}

#[test]
fn test_strip_tags() {
    cli()
        .arg("--strip-tags")
        .arg("span,div")
        .write_stdin("<p>Text with <span>span content</span> and <div>div content</div></p>")
        .assert()
        .success()
        .stdout(predicate::str::contains("span content"))
        .stdout(predicate::str::contains("div content"));
}

#[test]
fn test_preprocess() {
    cli()
        .arg("--preprocess")
        .write_stdin("<nav>Navigation</nav><article>Content</article>")
        .assert()
        .success();
}

#[test]
fn test_preset_requires_preprocess() {
    cli()
        .arg("--preset")
        .arg("aggressive")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--preprocess"));
}

#[test]
fn test_preprocess_with_preset_minimal() {
    cli()
        .arg("--preprocess")
        .arg("--preset")
        .arg("minimal")
        .write_stdin("<p>Test</p>")
        .assert()
        .success();
}

#[test]
fn test_preprocess_with_preset_aggressive() {
    cli()
        .arg("--preprocess")
        .arg("--preset")
        .arg("aggressive")
        .write_stdin("<nav>Nav</nav><p>Content</p>")
        .assert()
        .success();
}

#[test]
fn test_keep_navigation() {
    cli()
        .arg("--preprocess")
        .arg("--keep-navigation")
        .write_stdin("<nav>Navigation</nav>")
        .assert()
        .success();
}

#[test]
fn test_keep_forms() {
    cli()
        .arg("--preprocess")
        .arg("--keep-forms")
        .write_stdin("<form><input></form>")
        .assert()
        .success();
}

#[test]
fn test_debug_flag() {
    cli().arg("--debug").write_stdin("<p>Test</p>").assert().success();
}

#[test]
fn test_encoding_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.html");
    fs::write(&input_path, "<p>Test UTF-8: 你好</p>").unwrap();

    cli()
        .arg("--encoding")
        .arg("utf-8")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("你好"));
}

#[test]
fn test_encoding_invalid() {
    cli()
        .arg("--encoding")
        .arg("invalid-encoding")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown encoding"));
}

#[test]
fn test_list_indent_width_validation_min() {
    cli()
        .arg("--list-indent-width")
        .arg("0")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure();
}

#[test]
fn test_list_indent_width_validation_max() {
    cli()
        .arg("--list-indent-width")
        .arg("9")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure();
}

#[test]
fn test_bullets_validation_empty() {
    cli()
        .arg("--bullets")
        .arg("")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_bullets_validation_too_long() {
    cli()
        .arg("--bullets")
        .arg("*+-*+-*+-*+")
        .write_stdin("<p>Test</p>")
        .assert()
        .failure()
        .stderr(predicate::str::contains("too long"));
}

#[test]
fn test_nonexistent_file() {
    cli()
        .arg("/nonexistent/file.html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading file"));
}

#[test]
fn test_invalid_html() {
    cli().write_stdin("<p>Unclosed paragraph<p>Another").assert().success();
}

#[test]
fn test_empty_input() {
    cli().write_stdin("").assert().success().stdout("");
}

#[test]
fn test_complex_document() {
    let html = r#"
        <html>
            <head><title>Test Document</title></head>
            <body>
                <h1>Main Title</h1>
                <p>Introduction with <strong>bold</strong> and <em>italic</em>.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2
                        <ul>
                            <li>Nested item</li>
                        </ul>
                    </li>
                </ul>
                <pre><code>fn main() {
    println!("Hello");
}</code></pre>
                <p>Link: <a href="https://example.com">Example</a></p>
            </body>
        </html>
    "#;

    cli()
        .write_stdin(html)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Main Title"))
        .stdout(predicate::str::contains("**bold**"))
        .stdout(predicate::str::contains("*italic*"))
        .stdout(predicate::str::contains("- Item 1"))
        .stdout(predicate::str::contains("[Example](https://example.com)"));
}

#[test]
fn test_version_flag() {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("2.5.0"));
}

#[test]
fn test_help_flag() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn test_generate_completion_bash() {
    cli()
        .arg("--generate-completion")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("_html-to-markdown()"));
}

#[test]
fn test_generate_completion_zsh() {
    cli()
        .arg("--generate-completion")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

#[test]
fn test_generate_man() {
    cli()
        .arg("--generate-man")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"))
        .stdout(predicate::str::contains("html-to-markdown"));
}

#[test]
fn test_multiple_options_combined() {
    cli()
        .arg("--heading-style")
        .arg("atx")
        .arg("--bullets")
        .arg("*")
        .arg("--escape-asterisks")
        .arg("--code-block-style")
        .arg("backticks")
        .write_stdin("<h1>Title</h1><ul><li>Item</li></ul><pre><code>code</code></pre>")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Title"))
        .stdout(predicate::str::contains("* Item"))
        .stdout(predicate::str::contains("```"));
}
