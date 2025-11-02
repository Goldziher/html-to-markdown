"""Comprehensive tests for whitespace handling in HTML to Markdown conversion.

Covers whitespace normalization, block element separation, inline spacing,
and various whitespace modes and configurations.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

import pytest


def test_normalized_mode_basic(convert: Callable[..., str]) -> None:
    assert convert("<b>bold</b> text", whitespace_mode="normalized") == "**bold** text\n"
    assert convert("<b>bold</b>\ntext", whitespace_mode="normalized") == "**bold** text\n"
    assert convert("text    with    spaces", whitespace_mode="normalized") == "text with spaces\n"


def test_normalized_mode(convert: Callable[..., str]) -> None:
    html = "<b>bold</b>\n text"
    result = convert(html, whitespace_mode="normalized")
    assert "**bold**" in result


def test_strict_mode_preservation(convert: Callable[..., str]) -> None:
    html = "<b>bold</b>  \n  text"
    result = convert(html, whitespace_mode="strict")
    assert "**bold**" in result
    assert "text" in result


def test_unicode_space_normalization(convert: Callable[..., str]) -> None:
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
        result = convert(html, whitespace_mode="normalized")
        assert result == "text with space\n", f"Failed for Unicode {ord(unicode_space):04X}"


def test_block_element_spacing(convert: Callable[..., str]) -> None:
    assert convert("<div>div1</div><div>div2</div>", whitespace_mode="normalized") == "div1\n\ndiv2\n"
    assert convert("<p>para1</p><p>para2</p>", whitespace_mode="normalized") == "para1\n\npara2\n"
    assert convert("<div>div</div><p>para</p>", whitespace_mode="normalized") == "div\n\npara\n"


def test_inline_element_spacing(convert: Callable[..., str]) -> None:
    assert convert("<em>italic</em> text") == "*italic* text\n"
    assert convert("text <strong>bold</strong>") == "text **bold**\n"
    assert convert('<a href="#">link</a> text') == "[link](#) text\n"
    assert convert('text <a href="#">link</a>') == "text [link](#)\n"


def test_adjacent_inline_elements(convert: Callable[..., str]) -> None:
    html = "<b>bold</b><i>italic</i>"
    result = convert(html, whitespace_mode="normalized")
    assert result == "**bold***italic*\n"

    html = "<b>bold</b> <i>italic</i>"
    result = convert(html, whitespace_mode="normalized")
    assert result == "**bold** *italic*\n"


def test_whitespace_in_lists(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>item 1</li>
        <li>item 2</li>
    </ul>
    """
    result = convert(html, whitespace_mode="normalized")
    assert "- item 1" in result
    assert "- item 2" in result


def test_whitespace_in_nested_structures(convert: Callable[..., str]) -> None:
    html = """
    <div>
        <p>Paragraph in div</p>
        <ul>
            <li>List item</li>
        </ul>
    </div>
    """
    result = convert(html, whitespace_mode="normalized")
    assert "Paragraph in div" in result
    assert "- List item" in result


def test_pre_and_code_whitespace(convert: Callable[..., str]) -> None:
    pre_html = "<pre>  line 1\n    line 2  </pre>"
    pre_result = convert(pre_html, whitespace_mode="normalized")
    assert "  line 1\n    line 2  " in pre_result

    code_html = "<code>  spaced  </code>"
    code_result = convert(code_html, whitespace_mode="normalized")
    assert "`" in code_result
    assert "spaced" in code_result


def test_tab_character_handling(convert: Callable[..., str]) -> None:
    html = "text\twith\ttabs"
    result = convert(html, whitespace_mode="normalized")
    assert result == "text with tabs\n"


def test_mixed_whitespace(convert: Callable[..., str]) -> None:
    html = "  \t \n  text  \n\t  "
    result = convert(html, whitespace_mode="normalized")
    assert result.strip() == "text"


def test_br_tag_handling(convert: Callable[..., str]) -> None:
    html = "line1<br>line2<br/>line3"

    result = convert(html, newline_style="spaces")
    assert result == "line1  \nline2  \nline3\n"

    result = convert(html, newline_style="backslash")
    assert result == "line1\\\nline2\\\nline3\n"


def test_empty_elements(convert: Callable[..., str]) -> None:
    assert convert("<div></div>") == ""
    assert convert("<p></p>") == ""
    assert convert("<span></span>") == ""


def test_whitespace_only_elements(convert: Callable[..., str]) -> None:
    assert convert("<div>   </div>", whitespace_mode="normalized").strip() == ""
    assert "\n\t" in convert("<pre>\n\t</pre>", whitespace_mode="normalized")


def test_complex_real_world_example(convert: Callable[..., str]) -> None:
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
    result = convert(html, whitespace_mode="normalized")

    assert "Title" in result
    assert "First paragraph with **bold** and *italic* text." in result
    assert "Subtitle" in result
    assert "- Item 1" in result
    assert "- Item 2 with [link](#)" in result
    assert "Final paragraph." in result


def test_block_element_newline_separation(convert: Callable[..., str]) -> None:
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

    result = convert(html, whitespace_mode="normalized")

    assert "**test1** test2" in result

    lines = result.strip().split("\n")

    non_empty_lines = [line for line in lines if line.strip()]

    assert len(non_empty_lines) >= 4
    assert "**test1** test2" in non_empty_lines[0]
    assert "- test3" in result
    assert "test4" in result
    assert "test5" in result

    div_test = "<div>div1</div><div>div2</div>"
    _ = convert(div_test, whitespace_mode="normalized")

    p_test = "<p>para1</p><p>para2</p>"
    p_result = convert(p_test, whitespace_mode="normalized")
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
def test_whitespace_patterns(html: str, expected: str, convert: Callable[..., str]) -> None:
    result = convert(html, whitespace_mode="normalized")
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
        ("<p>para</p><div><ul><li>item</li></ul></div>", ["para", "- item"], "Paragraph followed by div with list"),
        ("<div>div</div><blockquote>quote</blockquote>", ["div\n> quote"], "Div followed by blockquote"),
        ("<h1>Heading 1</h1><p>Content</p>", ["Heading 1", "Content"], "Heading followed by paragraph"),
        ("<p>Content</p><h2>Heading 2</h2>", ["Content", "Heading 2"], "Paragraph followed by heading"),
        ("<div>content</div><ul><li>item</li></ul>", ["content", "- item"], "Div followed by list"),
        ("<ul><li>item1</li></ul><div>content</div>", ["- item1", "content"], "List followed by div"),
    ],
)
def test_block_element_separation_comprehensive(
    html: str, expected_lines: list[str], description: str, convert: Callable[..., str]
) -> None:
    result = convert(html, whitespace_mode="normalized")

    blocks = [block.strip() for block in result.split("\n\n") if block.strip()]

    assert len(blocks) >= len(expected_lines), (
        f"Expected at least {len(expected_lines)} blocks for {description}, got {len(blocks)}. Result: {result!r}"
    )

    for expected_line in expected_lines:
        assert any(expected_line in block for block in blocks), (
            f"Expected line '{expected_line}' not found in blocks {blocks} for {description}. Full result: {result!r}"
        )


def test_carriage_return_normalization(convert: Callable[..., str]) -> None:
    html = "<p>Line 1\rLine 2\r\nLine 3</p>"
    result = convert(html)
    assert "Line 1\nLine 2\nLine 3" in result

    html = "<p>Text\rCarriage</p>"
    result = convert(html, whitespace_mode="normalized")
    assert "Text\nCarriage" in result or "Text Carriage" in result


def test_empty_text_processing(convert: Callable[..., str]) -> None:
    html = "<p></p>"
    result = convert(html)
    assert result.strip() == ""

    html = "<p></p>"
    result = convert(html, whitespace_mode="strict")
    assert result.strip() == ""

    html = "<p>   </p>"
    result = convert(html)
    assert result.strip() == ""


def test_strict_mode_text_preservation(convert: Callable[..., str]) -> None:
    html = "<pre>  Text  with   spaces  </pre>"
    result = convert(html, whitespace_mode="strict")
    assert "  Text  with   spaces  " in result


def test_strict_whitespace_mode(convert: Callable[..., str]) -> None:
    html = "<p>First paragraph</p><p>Second paragraph</p>"
    result = convert(html, whitespace_mode="strict")
    assert result


def test_block_spacing_combinations(convert: Callable[..., str]) -> None:
    html = "<div>Div content</div><blockquote>Quote content</blockquote>"
    result = convert(html)
    assert "Div content" in result
    assert "Quote content" in result

    html = "<ul><li>Item 1</li><li>Item 2</li></ul>"
    result = convert(html)
    assert "Item 1" in result
    assert "Item 2" in result

    html = "<h1>Title</h1><p>Content</p>"
    result = convert(html)
    assert "Title" in result
    assert "Content" in result


def test_mixed_block_and_inline_elements(convert: Callable[..., str]) -> None:
    html = "<p>Text with <strong>inline</strong> element</p><div>Block element</div>"
    result = convert(html)
    assert "Text with **inline** element" in result
    assert "Block element" in result


def test_whitespace_trailing_with_inline_sibling(convert: Callable[..., str]) -> None:
    html = "Text\n<span>inline</span>"
    result = convert(html, whitespace_mode="normalized")
    assert "Textinline" in result

    html = "Text\t<em>emphasized</em>"
    result = convert(html, whitespace_mode="normalized")
    assert "Text *emphasized*" in result


def test_unicode_whitespace_strict_mode(convert: Callable[..., str]) -> None:
    html = "<p>Text\u00a0with\u2003unicode\u00a0spaces</p>"

    result_strict = convert(html, whitespace_mode="strict")
    assert "\u00a0" in result_strict
    assert "\u2003" in result_strict

    result_normalized = convert(html, whitespace_mode="normalized")
    assert "\u00a0" not in result_normalized
    assert "\u2003" not in result_normalized
    assert "Text with unicode spaces" in result_normalized


def test_strict_mode_block_spacing(convert: Callable[..., str]) -> None:
    html = "<p>First paragraph</p><p>Second paragraph</p>"
    result = convert(html, whitespace_mode="strict")
    assert "First paragraph" in result
    assert "Second paragraph" in result


def test_block_spacing_with_double_newline_elements(convert: Callable[..., str]) -> None:
    html = "<div>Content</div><p>Paragraph</p>"
    result = convert(html, whitespace_mode="normalized")
    assert "Content\n\nParagraph" in result

    html = "<blockquote>Quote</blockquote><div>Content</div>"
    result = convert(html, whitespace_mode="normalized")
    assert "> Quote\n\nContent" in result

    html = "<table><tr><td>Cell</td></tr></table><p>After table</p>"
    result = convert(html, whitespace_mode="normalized")
    assert "Cell" in result
    assert "After table" in result


def test_block_spacing_with_single_newline_elements(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1</li><li>Item 2</li></ul>"
    result = convert(html, whitespace_mode="normalized")
    assert "- Item 1\n- Item 2" in result

    html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
    result = convert(html, whitespace_mode="normalized")
    assert "Term" in result
    assert "Definition" in result

    html = "<table><tr><td>Cell 1</td></tr><tr><td>Cell 2</td></tr></table>"
    result = convert(html, whitespace_mode="normalized")
    assert "Cell 1" in result
    assert "Cell 2" in result


def test_block_spacing_heading_elements(convert: Callable[..., str]) -> None:
    html = "<h1>Heading 1</h1><p>Content</p>"
    result = convert(html, whitespace_mode="normalized", heading_style="atx")
    assert "# Heading 1\n\nContent" in result

    html = "<h2>Heading 2</h2><div>Content</div>"
    result = convert(html, whitespace_mode="normalized", heading_style="atx")
    assert "## Heading 2\n\nContent" in result

    for i in range(3, 7):
        html = f"<h{i}>Heading {i}</h{i}><p>Content</p>"
        result = convert(html, whitespace_mode="normalized", heading_style="atx")
        assert f"{'#' * i} Heading {i}\n\nContent" in result


def test_block_spacing_non_block_next_sibling(convert: Callable[..., str]) -> None:
    html = "<div>Content</div>plain text"
    result = convert(html, whitespace_mode="normalized")
    assert "Content\n\nplain text" in result

    html = "<p>Paragraph</p><span>inline span</span>"
    result = convert(html, whitespace_mode="normalized")
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
            "**test1** test2\n\n- test3\n\ntest4\n\ntest5\n",
            None,
        ),
        (
            """test1

test2

<a href="https://example.com">example.com</a>

<a href="https://example.org">example.org</a>""",
            "test1\n\ntest2\n\n[example.com](https://example.com)[example.org](https://example.org)\n",
            None,
        ),
        (
            """test1

test2

<a href="https://example.com">example.com</a>

<a href="https://example.org">example.org</a>""",
            "test1\n\ntest2\n\n[example.com](https://example.com)\n[example.org](https://example.org)\n",
            "strict",
        ),
        ("<b>bold</b><i>italic</i><code>code</code>", "**bold***italic*`code`\n", None),
        ("<p>Para 1</p><b>bold text</b><p>Para 2</p>", "Para 1\n\n**bold text**\n\nPara 2\n", None),
        (
            "<div>Text content</div><div><ul><li>List item</li></ul></div><div>More text</div>",
            "Text content\n\n- List item\n\nMore text\n",
            None,
        ),
        ("<h1>Header</h1>inline text<p>Paragraph</p>", "# Header\n\ninline text\n\nParagraph\n", None),
        (
            """<div>
    <p>Paragraph in div</p>
    <span>Inline span</span>
    <div>Nested div</div>
</div>
<p>Following paragraph</p>""",
            "Paragraph in div\n\nInline span\n\nNested div\n\nFollowing paragraph\n",
            None,
        ),
        ('<a href="url1">Link1</a><a href="url2">Link2</a>', "[Link1](url1)[Link2](url2)\n", None),
        ('<a href="url1">Link1</a> <a href="url2">Link2</a>', "[Link1](url1) [Link2](url2)\n", None),
        (
            """<p>Para 1</p>

<p>Para 2</p>

<div>Div content</div>""",
            "Para 1\n\nPara 2\n\nDiv content\n",
            "strict",
        ),
        ("""text    with    multiple    spaces""", "text with multiple spaces\n", "normalized"),
        ("<p>Content 1</p><div></div><p>Content 2</p>", "Content 1\n\nContent 2\n", None),
    ],
)
def test_whitespace_and_spacing_issues(
    html: str, expected: str, whitespace_mode: str | None, convert: Callable[..., str], parser: str
) -> None:
    result = convert(html, whitespace_mode=whitespace_mode) if whitespace_mode else convert(html)

    if (
        parser == "html5lib"
        and whitespace_mode == "strict"
        and '<a href="https://example.com">example.com</a>\n\n<a href="https://example.org">example.org</a>' in html
    ):
        expected_html5lib = expected.replace(
            "[example.com](https://example.com)\n[example.org](https://example.org)",
            "[example.com](https://example.com)\n\n[example.org](https://example.org)",
        )
        assert result == expected_html5lib
    else:
        assert result == expected
