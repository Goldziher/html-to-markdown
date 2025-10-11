from __future__ import annotations

import pytest

from html_to_markdown import convert_to_markdown

# Suppress deprecation warnings for v1 compatibility tests
pytestmark = pytest.mark.filterwarnings("ignore::DeprecationWarning")


class TestV1CompatBasic:
    def test_basic_conversion(self) -> None:
        html = "<p>Hello <strong>world</strong>!</p>"
        result = convert_to_markdown(html)
        assert result == "Hello **world**!\n"

    def test_heading_style_atx(self) -> None:
        html = "<h1>Title</h1>"
        result = convert_to_markdown(html, heading_style="atx")
        assert result == "# Title\n"

    def test_heading_style_underlined(self) -> None:
        html = "<h1>Title</h1>"
        result = convert_to_markdown(html, heading_style="underlined")
        assert result == "Title\n=====\n"

    def test_list_indent_width(self) -> None:
        html = "<ul><li>a<ul><li>b</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=2)
        assert "  +" in result

    def test_bullets_custom(self) -> None:
        html = "<ul><li>1<ul><li>a</li></ul></li></ul>"
        result = convert_to_markdown(html, bullets="-")
        assert result == "- 1\n    - a\n"

    def test_strong_em_symbol_underscore(self) -> None:
        html = "<strong>bold</strong> <em>italic</em>"
        result = convert_to_markdown(html, strong_em_symbol="_")
        assert result == "__bold__ _italic_\n"

    def test_escape_asterisks_false(self) -> None:
        html = "*asterisks*"
        result = convert_to_markdown(html, escape_asterisks=False)
        assert result == "*asterisks*\n"

    def test_escape_asterisks_true(self) -> None:
        html = "*asterisks*"
        result = convert_to_markdown(html, escape_asterisks=True)
        assert result == r"\*asterisks\*" + "\n"

    def test_code_language(self) -> None:
        html = "<pre><code>print('hello')</code></pre>"
        result = convert_to_markdown(html, code_language="python")
        assert "```python" in result

    def test_autolinks_true(self) -> None:
        html = '<a href="https://example.com">https://example.com</a>'
        result = convert_to_markdown(html, autolinks=True)
        assert result == "<https://example.com>\n"

    def test_autolinks_false(self) -> None:
        html = '<a href="https://example.com">https://example.com</a>'
        result = convert_to_markdown(html, autolinks=False)
        assert result == "[https://example.com](https://example.com)\n"


class TestV1CompatOptions:
    def test_extract_metadata_true(self) -> None:
        html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>"
        result = convert_to_markdown(html, extract_metadata=True)
        # v2 uses YAML frontmatter instead of HTML comments
        assert "---" in result
        assert "title: Test" in result

    def test_extract_metadata_false(self) -> None:
        html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>"
        result = convert_to_markdown(html, extract_metadata=False)
        assert "---" not in result
        assert "title:" not in result

    def test_wrap_enabled(self) -> None:
        html = "<p>123456789 123456789</p>"
        result = convert_to_markdown(html, wrap=True, wrap_width=10)
        assert "123456789" in result
        assert "123456789" in result

    def test_convert_as_inline(self) -> None:
        html = "<p>Test</p>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Test\n"

    def test_sub_sup_symbols(self) -> None:
        html = "<sub>2</sub> <sup>3</sup>"
        result = convert_to_markdown(html, sub_symbol="~", sup_symbol="^")
        assert result == "~2~ ^3^\n"

    def test_newline_style_backslash(self) -> None:
        html = "a<br />b"
        result = convert_to_markdown(html, newline_style="backslash")
        assert result == "a\\\nb\n"

    def test_newline_style_spaces(self) -> None:
        html = "a<br />b"
        result = convert_to_markdown(html, newline_style="spaces")
        assert result == "a  \nb\n"

    def test_highlight_style_double_equal(self) -> None:
        html = "<mark>highlighted</mark>"
        result = convert_to_markdown(html, highlight_style="double-equal")
        assert result.strip() == "==highlighted=="

    def test_highlight_style_bold(self) -> None:
        html = "<mark>highlighted</mark>"
        result = convert_to_markdown(html, highlight_style="bold")
        assert result.strip() == "**highlighted**"


class TestV1CompatPreprocessing:
    def test_preprocess_enabled(self) -> None:
        html = "<nav>Navigation</nav><p>Content</p>"
        result = convert_to_markdown(html, preprocess=True, remove_navigation=True)
        assert "Content" in result


class TestV1CompatUnsupportedOptions:
    def test_code_language_callback_raises(self) -> None:
        html = "<pre>code</pre>"

        def callback(el: object) -> str:
            return "python"

        with pytest.raises(NotImplementedError, match="code_language_callback was removed"):
            convert_to_markdown(html, code_language_callback=callback)

    def test_strip_option_works(self) -> None:
        html = "<p>text with <a href='#'>link</a> inside</p>"
        result = convert_to_markdown(html, strip=["a"])
        assert "link" in result
        assert "[link]" not in result
        assert "(#)" not in result

    def test_convert_option_raises(self) -> None:
        html = "<a href='#'>link</a>"
        with pytest.raises(NotImplementedError, match="convert option was removed"):
            convert_to_markdown(html, convert=["a"])

    def test_custom_converters_raises(self) -> None:
        html = "<custom>content</custom>"
        with pytest.raises(NotImplementedError, match="custom_converters is not yet implemented"):
            convert_to_markdown(html, custom_converters={"custom": lambda **kw: "converted"})


class TestV1CompatEdgeCases:
    def test_empty_html(self) -> None:
        result = convert_to_markdown("")
        assert result == ""

    def test_all_options_together(self) -> None:
        html = """
        <html>
        <head><title>Test</title></head>
        <body>
            <h1>Heading</h1>
            <p>Text with <strong>bold</strong> and <em>italic</em>.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
            <pre><code>code block</code></pre>
        </body>
        </html>
        """
        result = convert_to_markdown(
            html,
            heading_style="atx",
            bullets="-",
            strong_em_symbol="_",
            code_language="python",
            extract_metadata=True,
        )
        assert "# Heading" in result
        assert "__bold__" in result
        assert "_italic_" in result
        assert "- Item 1" in result
        assert "```python" in result
        assert "title: Test" in result

    def test_complex_tables(self) -> None:
        html = """
        <table>
            <tr><th>Name</th><th>Age</th></tr>
            <tr><td>Alice</td><td>30</td></tr>
            <tr><td>Bob</td><td>25</td></tr>
        </table>
        """
        result = convert_to_markdown(html)
        assert "| Name | Age |" in result
        assert "| Alice | 30 |" in result
        assert "| Bob | 25 |" in result

    def test_nested_lists(self) -> None:
        html = """
        <ul>
            <li>Level 1
                <ul>
                    <li>Level 2</li>
                </ul>
            </li>
        </ul>
        """
        result = convert_to_markdown(html, bullets="*+-")
        assert "* Level 1" in result
        assert "+ Level 2" in result or "    + Level 2" in result


class TestV1CompatIntegration:
    def test_wikipedia_style_content(self) -> None:
        html = """
        <div>
            <h2>History</h2>
            <p>Python was created by <a href="#">Guido van Rossum</a>.</p>
            <h3>Early years</h3>
            <p>Development began in <strong>1989</strong>.</p>
        </div>
        """
        result = convert_to_markdown(html, heading_style="atx")
        assert "## History" in result
        assert "### Early years" in result
        assert "[Guido van Rossum]" in result
        assert "**1989**" in result

    def test_blog_post_style(self) -> None:
        html = """
        <article>
            <h1>My Blog Post</h1>
            <p>This is <em>awesome</em> content!</p>
            <blockquote>
                <p>A wise quote.</p>
            </blockquote>
            <ul>
                <li>Point one</li>
                <li>Point two</li>
            </ul>
        </article>
        """
        result = convert_to_markdown(html, heading_style="atx")
        assert "# My Blog Post" in result
        assert "*awesome*" in result
        assert "> A wise quote" in result
        assert "* Point one" in result
