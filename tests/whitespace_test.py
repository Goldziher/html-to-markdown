"""Comprehensive tests for whitespace handling in HTML to Markdown conversion.

Covers whitespace normalization, block element separation, inline spacing,
and various whitespace modes and configurations.
"""

from __future__ import annotations

import pytest

from html_to_markdown import convert_to_markdown


def test_normalized_mode_basic() -> None:
    assert convert_to_markdown("<b>bold</b> text", whitespace_mode="normalized") == "**bold** text"
    assert convert_to_markdown("<b>bold</b>\ntext", whitespace_mode="normalized") == "**bold** text"
    assert convert_to_markdown("text    with    spaces", whitespace_mode="normalized") == "text with spaces"


def test_normalized_mode() -> None:
    html = "<b>bold</b>\n text"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "**bold**" in result


def test_strict_mode_preservation() -> None:
    html = "<b>bold</b>  \n  text"
    result = convert_to_markdown(html, whitespace_mode="strict")
    assert "**bold**" in result
    assert "text" in result


def test_unicode_space_normalization() -> None:
    test_cases = [
        ("\u00a0", " "),
        ("\u1680", " "),
        ("\u2000", " "),
        ("\u2001", " "),
        ("\u2002", " "),
        ("\u2003", " "),
        ("\u2004", " "),
        ("\u2005", " "),
        ("\u2006", " "),
        ("\u2007", " "),
        ("\u2008", " "),
        ("\u2009", " "),
        ("\u200a", " "),
        ("\u202f", " "),
        ("\u205f", " "),
        ("\u3000", " "),
    ]

    for unicode_space, _expected in test_cases:
        html = f"text{unicode_space}with{unicode_space}space"
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert result == "text with space", f"Failed for Unicode {ord(unicode_space):04X}"


def test_block_element_spacing() -> None:
    assert convert_to_markdown("<div>div1</div><div>div2</div>", whitespace_mode="normalized") == "div1\n\ndiv2\n\n"
    assert convert_to_markdown("<p>para1</p><p>para2</p>", whitespace_mode="normalized") == "para1\n\npara2\n\n"
    assert convert_to_markdown("<div>div</div><p>para</p>", whitespace_mode="normalized") == "div\n\npara\n\n"


def test_inline_element_spacing() -> None:
    assert convert_to_markdown("<em>italic</em> text") == "*italic* text"
    assert convert_to_markdown("text <strong>bold</strong>") == "text **bold**"
    assert convert_to_markdown('<a href="#">link</a> text') == "[link](#) text"
    assert convert_to_markdown('text <a href="#">link</a>') == "text [link](#)"


def test_adjacent_inline_elements() -> None:
    html = "<b>bold</b><i>italic</i>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert result == "**bold***italic*"

    html = "<b>bold</b> <i>italic</i>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert result == "**bold** *italic*"


def test_whitespace_in_lists() -> None:
    html = """
    <ul>
        <li>item 1</li>
        <li>item 2</li>
    </ul>
    """
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "* item 1" in result
    assert "* item 2" in result


def test_whitespace_in_nested_structures() -> None:
    html = """
    <div>
        <p>Paragraph in div</p>
        <ul>
            <li>List item</li>
        </ul>
    </div>
    """
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Paragraph in div" in result
    assert "* List item" in result


def test_pre_and_code_whitespace() -> None:
    pre_html = "<pre>  line 1\n    line 2  </pre>"
    pre_result = convert_to_markdown(pre_html, whitespace_mode="normalized")
    assert "  line 1\n    line 2  " in pre_result

    code_html = "<code>  spaced  </code>"
    code_result = convert_to_markdown(code_html, whitespace_mode="normalized")
    assert "`" in code_result
    assert "spaced" in code_result


def test_tab_character_handling() -> None:
    html = "text\twith\ttabs"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert result == "text with tabs"


def test_mixed_whitespace() -> None:
    html = "  \t \n  text  \n\t  "
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert result.strip() == "text"


def test_br_tag_handling() -> None:
    html = "line1<br>line2<br/>line3"

    result = convert_to_markdown(html, newline_style="spaces")
    assert result == "line1  \nline2  \nline3"

    result = convert_to_markdown(html, newline_style="backslash")
    assert result == "line1\\\nline2\\\nline3"


def test_empty_elements() -> None:
    assert convert_to_markdown("<div></div>") == ""
    assert convert_to_markdown("<p></p>") == ""
    assert convert_to_markdown("<span></span>") == ""


def test_whitespace_only_elements() -> None:
    assert convert_to_markdown("<div>   </div>", whitespace_mode="normalized").strip() == ""
    assert "\n\t" in convert_to_markdown("<pre>\n\t</pre>", whitespace_mode="normalized")


def test_complex_real_world_example() -> None:
    html = """
    <article>
        <h1>Title</h1>
        <p>First paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
        <div>
            <h2>Subtitle</h2>
            <ul>
                <li>Item 1</li>
                <li>Item 2 with <a href="#">link</a></li>
            </ul>
        </div>
        <p>Final paragraph.</p>
    </article>
    """
    result = convert_to_markdown(html, whitespace_mode="normalized")

    assert "Title" in result
    assert "First paragraph with **bold** and *italic* text." in result
    assert "Subtitle" in result
    assert "* Item 1" in result
    assert "* Item 2 with [link](#)" in result
    assert "Final paragraph." in result


def test_block_element_newline_separation() -> None:
    html = """<b>test1</b>
 test2

<div>
<ul>
<li>
test3
</li>
</ul></div><div>
test4
</div>
<p>test5</p>"""

    result = convert_to_markdown(html, whitespace_mode="normalized")

    assert "**test1** test2" in result

    lines = result.strip().split("\n")

    non_empty_lines = [line for line in lines if line.strip()]

    assert len(non_empty_lines) >= 4
    assert "**test1** test2" in non_empty_lines[0]
    assert "* test3" in result
    assert "test4" in result
    assert "test5" in result

    div_test = "<div>div1</div><div>div2</div>"
    _ = convert_to_markdown(div_test, whitespace_mode="normalized")

    p_test = "<p>para1</p><p>para2</p>"
    p_result = convert_to_markdown(p_test, whitespace_mode="normalized")
    assert "para1\n\npara2" in p_result


@pytest.mark.parametrize(
    "html,expected",
    [
        ("<b>test</b> after", "**test** after"),
        ("before <b>test</b>", "before **test**"),
        ("<b>test1</b> <b>test2</b>", "**test1** **test2**"),
        ("<div>block</div>text", "block\n\ntext"),
    ],
)
def test_whitespace_patterns(html: str, expected: str) -> None:
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert expected in result


@pytest.mark.parametrize(
    "html,expected_lines,description",
    [
        ("<div>content1</div><div>content2</div>", ["content1", "content2"], "Adjacent div elements"),
        ("<p>para1</p><p>para2</p>", ["para1", "para2"], "Adjacent paragraph elements"),
        ("<section>sec1</section><section>sec2</section>", ["sec1", "sec2"], "Adjacent section elements"),
        ("<article>art1</article><article>art2</article>", ["art1", "art2"], "Adjacent article elements"),
        (
            "<header>head</header><main>main</main><footer>foot</footer>",
            ["head", "main", "foot"],
            "Header main footer",
        ),
        ("text<div>block</div>", ["text", "block"], "Text followed by div"),
        (
            "<b>bold</b> text<p>paragraph</p>",
            ["**bold** text", "paragraph"],
            "Inline content followed by paragraph",
        ),
        ("<span>inline</span><div>block</div>", ["inline", "block"], "Inline span followed by div"),
        ("<p>para</p><div><ul><li>item</li></ul></div>", ["para", "* item"], "Paragraph followed by div with list"),
        ("<div>div</div><blockquote>quote</blockquote>", ["div", "> quote"], "Div followed by blockquote"),
        ("<h1>Heading 1</h1><p>Content</p>", ["Heading 1", "Content"], "Heading followed by paragraph"),
        ("<p>Content</p><h2>Heading 2</h2>", ["Content", "Heading 2"], "Paragraph followed by heading"),
        ("<div>content</div><ul><li>item</li></ul>", ["content", "* item"], "Div followed by list"),
        ("<ul><li>item1</li></ul><div>content</div>", ["* item1", "content"], "List followed by div"),
    ],
)
def test_block_element_separation_comprehensive(html: str, expected_lines: list[str], description: str) -> None:
    result = convert_to_markdown(html, whitespace_mode="normalized")

    blocks = [block.strip() for block in result.split("\n\n") if block.strip()]

    assert len(blocks) >= len(expected_lines), (
        f"Expected at least {len(expected_lines)} blocks for {description}, got {len(blocks)}. Result: {result!r}"
    )

    for expected_line in expected_lines:
        assert any(expected_line in block for block in blocks), (
            f"Expected line '{expected_line}' not found in blocks {blocks} for {description}. Full result: {result!r}"
        )


def test_carriage_return_normalization() -> None:
    html = "<p>Line 1\rLine 2\r\nLine 3</p>"
    result = convert_to_markdown(html)
    assert "Line 1\nLine 2\nLine 3" in result

    html = "<p>Text\rCarriage</p>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Text\nCarriage" in result or "Text Carriage" in result


def test_empty_text_processing() -> None:
    html = "<p></p>"
    result = convert_to_markdown(html)
    assert result.strip() == ""

    html = "<p></p>"
    result = convert_to_markdown(html, whitespace_mode="strict")
    assert result.strip() == ""

    html = "<p>   </p>"
    result = convert_to_markdown(html)
    assert result.strip() == ""


def test_strict_mode_text_preservation() -> None:
    html = "<pre>  Text  with   spaces  </pre>"
    result = convert_to_markdown(html, whitespace_mode="strict")
    assert "  Text  with   spaces  " in result


def test_strict_whitespace_mode() -> None:
    html = "<p>First paragraph</p><p>Second paragraph</p>"
    result = convert_to_markdown(html, whitespace_mode="strict")
    assert result


def test_block_spacing_combinations() -> None:
    html = "<div>Div content</div><blockquote>Quote content</blockquote>"
    result = convert_to_markdown(html)
    assert "Div content" in result
    assert "Quote content" in result

    html = "<ul><li>Item 1</li><li>Item 2</li></ul>"
    result = convert_to_markdown(html)
    assert "Item 1" in result
    assert "Item 2" in result

    html = "<h1>Title</h1><p>Content</p>"
    result = convert_to_markdown(html)
    assert "Title" in result
    assert "Content" in result


def test_mixed_block_and_inline_elements() -> None:
    html = "<p>Text with <strong>inline</strong> element</p><div>Block element</div>"
    result = convert_to_markdown(html)
    assert "Text with **inline** element" in result
    assert "Block element" in result


def test_whitespace_trailing_with_inline_sibling() -> None:
    html = "Text\n<span>inline</span>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Text inline" in result

    html = "Text\t<em>emphasized</em>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Text *emphasized*" in result


def test_unicode_whitespace_strict_mode() -> None:
    html = "<p>Text\u00a0with\u2003unicode\u00a0spaces</p>"

    result_strict = convert_to_markdown(html, whitespace_mode="strict")
    assert "\u00a0" in result_strict
    assert "\u2003" in result_strict

    result_normalized = convert_to_markdown(html, whitespace_mode="normalized")
    assert "\u00a0" not in result_normalized
    assert "\u2003" not in result_normalized
    assert "Text with unicode spaces" in result_normalized


def test_strict_mode_block_spacing() -> None:
    html = "<p>First paragraph</p><p>Second paragraph</p>"
    result = convert_to_markdown(html, whitespace_mode="strict")
    assert "First paragraph" in result
    assert "Second paragraph" in result


def test_block_spacing_with_double_newline_elements() -> None:
    html = "<div>Content</div><p>Paragraph</p>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Content\n\nParagraph" in result

    html = "<blockquote>Quote</blockquote><div>Content</div>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "> Quote\n\nContent" in result

    html = "<table><tr><td>Cell</td></tr></table><p>After table</p>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Cell" in result
    assert "After table" in result


def test_block_spacing_with_single_newline_elements() -> None:
    html = "<ul><li>Item 1</li><li>Item 2</li></ul>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "* Item 1\n* Item 2" in result

    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Term" in result
    assert "Definition" in result

    html = "<table><tr><td>Cell 1</td></tr><tr><td>Cell 2</td></tr></table>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Cell 1" in result
    assert "Cell 2" in result


def test_block_spacing_heading_elements() -> None:
    html = "<h1>Heading 1</h1><p>Content</p>"
    result = convert_to_markdown(html, whitespace_mode="normalized", heading_style="atx")
    assert "# Heading 1\n\nContent" in result

    html = "<h2>Heading 2</h2><div>Content</div>"
    result = convert_to_markdown(html, whitespace_mode="normalized", heading_style="atx")
    assert "## Heading 2\n\nContent" in result

    for i in range(3, 7):
        html = f"<h{i}>Heading {i}</h{i}><p>Content</p>"
        result = convert_to_markdown(html, whitespace_mode="normalized", heading_style="atx")
        assert f"{'#' * i} Heading {i}\n\nContent" in result


def test_block_spacing_non_block_next_sibling() -> None:
    html = "<div>Content</div>plain text"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Content\n\nplain text" in result

    html = "<p>Paragraph</p><span>inline span</span>"
    result = convert_to_markdown(html, whitespace_mode="normalized")
    assert "Paragraph\n\ninline span" in result


@pytest.mark.parametrize(
    "html,expected,whitespace_mode",
    [
        (
            """<b>test1</b>
 test2

<div>
<ul>
<li>
test3
</li>
</ul></div><div>
test4
</div>
<p>test5</p>""",
            "**test1** test2\n\n* test3\n\ntest4\n\ntest5\n\n",
            None,
        ),
        (
            """test1

test2

<a href="https://example.com">example.com</a>

<a href="https://example.org">example.org</a>""",
            "test1\n\ntest2 [example.com](https://example.com)[example.org](https://example.org)",
            None,
        ),
        (
            """test1

test2

<a href="https://example.com">example.com</a>

<a href="https://example.org">example.org</a>""",
            """test1

test2

[example.com](https://example.com)
[example.org](https://example.org)""",
            "strict",
        ),
        ("<b>bold</b><i>italic</i><code>code</code>", "**bold***italic*`code`", None),
        ("<p>Para 1</p><b>bold text</b><p>Para 2</p>", "Para 1\n\n**bold text**\n\nPara 2\n\n", None),
        (
            "<div>Text content</div><div><ul><li>List item</li></ul></div><div>More text</div>",
            "Text content\n\n* List item\n\nMore text\n\n",
            None,
        ),
        ("<h1>Header</h1>inline text<p>Paragraph</p>", "Header\n======\n\ninline text\n\nParagraph\n\n", None),
        (
            """<div>
    <p>Paragraph in div</p>
    <span>Inline span</span>
    <div>Nested div</div>
</div>
<p>Following paragraph</p>""",
            """Paragraph in div

Inline span

Nested div

Following paragraph

""",
            None,
        ),
        ('<a href="url1">Link1</a><a href="url2">Link2</a>', "[Link1](url1)[Link2](url2)", None),
        ('<a href="url1">Link1</a> <a href="url2">Link2</a>', "[Link1](url1) [Link2](url2)", None),
        (
            """<p>Para 1</p>

<p>Para 2</p>

<div>Div content</div>""",
            """Para 1

Para 2

Div content

""",
            "strict",
        ),
        ("""text    with    multiple    spaces""", "text with multiple spaces", "normalized"),
        ("<p>Content 1</p><div></div><p>Content 2</p>", "Content 1\n\nContent 2\n\n", None),
    ],
)
def test_whitespace_and_spacing_issues(html: str, expected: str, whitespace_mode: str | None) -> None:
    """Test whitespace and spacing handling - Issue #63."""
    if whitespace_mode:
        result = convert_to_markdown(html, whitespace_mode=whitespace_mode)
    else:
        result = convert_to_markdown(html)
    assert result == expected
