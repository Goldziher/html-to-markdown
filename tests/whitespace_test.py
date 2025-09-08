"""Comprehensive tests for whitespace handling in HTML to Markdown conversion.

Includes tests for issue #63: Missing newlines and spaces in many cases.
"""

from __future__ import annotations

import pytest

from html_to_markdown import convert_to_markdown


class TestWhitespaceHandling:
    def test_normalized_mode_basic(self) -> None:
        assert convert_to_markdown("<b>bold</b> text", whitespace_mode="normalized") == "**bold** text"

        assert convert_to_markdown("<b>bold</b>\ntext", whitespace_mode="normalized") == "**bold** text"

        assert convert_to_markdown("text    with    spaces", whitespace_mode="normalized") == "text with spaces"

    def test_normalized_mode(self) -> None:
        html = "<b>bold</b>\n text"
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert "**bold**" in result

    def test_strict_mode_preservation(self) -> None:
        html = "<b>bold</b>  \n  text"
        result = convert_to_markdown(html, whitespace_mode="strict")
        assert "**bold**" in result
        assert "text" in result

    def test_unicode_space_normalization(self) -> None:
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

    def test_block_element_spacing(self) -> None:
        assert convert_to_markdown("<div>div1</div><div>div2</div>", whitespace_mode="normalized") == "div1div2"

        assert convert_to_markdown("<p>para1</p><p>para2</p>", whitespace_mode="normalized") == "para1\n\npara2\n\n"

        assert convert_to_markdown("<div>div</div><p>para</p>", whitespace_mode="normalized") == "divpara\n\n"

    def test_inline_element_spacing(self) -> None:
        assert convert_to_markdown("<em>italic</em> text") == "*italic* text"
        assert convert_to_markdown("text <strong>bold</strong>") == "text **bold**"

        assert convert_to_markdown('<a href="#">link</a> text') == "[link](#) text"
        assert convert_to_markdown('text <a href="#">link</a>') == "text [link](#)"

    def test_whitespace_in_lists(self) -> None:
        html = """
        <ul>
            <li>item 1</li>
            <li>item 2</li>
        </ul>
        """
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert "* item 1" in result
        assert "* item 2" in result

    def test_whitespace_in_nested_structures(self) -> None:
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

    def test_pre_and_code_whitespace(self) -> None:
        pre_html = "<pre>  line 1\n    line 2  </pre>"
        pre_result = convert_to_markdown(pre_html, whitespace_mode="normalized")
        assert "  line 1\n    line 2  " in pre_result

        code_html = "<code>  spaced  </code>"
        code_result = convert_to_markdown(code_html, whitespace_mode="normalized")
        assert "`" in code_result
        assert "spaced" in code_result

    def test_tab_character_handling(self) -> None:
        html = "text\twith\ttabs"
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert result == "text with tabs"

    def test_mixed_whitespace(self) -> None:
        html = "  \t \n  text  \n\t  "
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert result.strip() == "text"

    def test_br_tag_handling(self) -> None:
        html = "line1<br>line2<br/>line3"

        result = convert_to_markdown(html, newline_style="spaces")
        assert result == "line1  \nline2  \nline3"

        result = convert_to_markdown(html, newline_style="backslash")
        assert result == "line1\\\nline2\\\nline3"

    def test_empty_elements(self) -> None:
        assert convert_to_markdown("<div></div>") == ""
        assert convert_to_markdown("<p></p>") == ""
        assert convert_to_markdown("<span></span>") == ""

    def test_whitespace_only_elements(self) -> None:
        assert convert_to_markdown("<div>   </div>", whitespace_mode="normalized").strip() == ""

        assert "\n\t" in convert_to_markdown("<pre>\n\t</pre>", whitespace_mode="normalized")

    def test_adjacent_inline_elements(self) -> None:
        html = "<b>bold</b><i>italic</i>"
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert result == "**bold***italic*"

        html = "<b>bold</b> <i>italic</i>"
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert result == "**bold** *italic*"

    def test_complex_real_world_example(self) -> None:
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

    @pytest.mark.parametrize(
        "html,expected",
        [
            ("<b>test</b> after", "**test** after"),
            ("before <b>test</b>", "before **test**"),
            ("<b>test1</b> <b>test2</b>", "**test1** **test2**"),
            ("<div>block</div>text", "blocktext"),
        ],
    )
    def test_parametrized_cases(self, html: str, expected: str) -> None:
        result = convert_to_markdown(html, whitespace_mode="normalized")
        assert expected in result
