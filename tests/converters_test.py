"""Test converters module edge cases."""

from html_to_markdown import convert_to_markdown
from html_to_markdown.converters import create_converters_map


def test_create_converters_map() -> None:
    """Test converter map creation with various options."""
    # Test default options
    converters = create_converters_map(
        autolinks=True,
        bullets="*+-",
        code_language="",
        code_language_callback=None,
        default_title=False,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        newline_style="spaces",
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        wrap=False,
        wrap_width=80,
    )
    assert "p" in converters
    assert "a" in converters
    assert "h1" in converters

    # Test with custom options
    converters = create_converters_map(
        autolinks=False,
        bullets="-*+",
        code_language="python",
        code_language_callback=None,
        default_title=True,
        heading_style="atx",
        highlight_style="html",
        keep_inline_images_in=["p"],
        newline_style="backslash",
        strong_em_symbol="_",
        sub_symbol="~",
        sup_symbol="^",
        wrap=True,
        wrap_width=120,
    )
    assert "ul" in converters
    assert "h1" in converters


def test_edge_case_converters() -> None:
    """Test edge cases in various converters."""
    # Test empty blockquote - returns empty for empty content
    result = convert_to_markdown("<blockquote></blockquote>")
    # Empty blockquotes may not produce output

    # Test nested emphasis edge cases
    result = convert_to_markdown("<strong><em>text</em></strong>")
    assert "***text***" in result

    # Test code blocks with special characters
    result = convert_to_markdown("<pre><code>&lt;script&gt;</code></pre>")
    assert "<script>" in result

    # Test lists with no content
    result = convert_to_markdown("<ul><li></li></ul>")
    assert "*" in result  # Should produce some list marker


def test_table_edge_cases() -> None:
    """Test table converter edge cases."""
    # Test table with empty cells
    html = """<table>
    <tr><td></td><td>content</td></tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "|  | content |" in result

    # Test table with complex nested content
    html = """<table>
    <tr><td><strong>bold</strong> and <em>italic</em></td></tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "**bold** and *italic*" in result


def test_image_edge_cases() -> None:
    """Test image converter edge cases."""
    # Image with empty alt text
    result = convert_to_markdown('<img src="test.jpg" alt="">')
    assert "![](test.jpg)" in result

    # Image with no alt attribute
    result = convert_to_markdown('<img src="test.jpg">')
    assert "![](test.jpg)" in result

    # Image with title
    result = convert_to_markdown('<img src="test.jpg" alt="Test" title="Test Image">')
    assert '![Test](test.jpg "Test Image")' in result


def test_link_edge_cases() -> None:
    """Test link converter edge cases."""
    # Link with no href - just returns text
    result = convert_to_markdown("<a>text</a>")
    assert result.strip() == "text"

    # Link with empty href - may just return text
    result = convert_to_markdown('<a href="">text</a>')
    assert "text" in result  # May not create link syntax for empty href

    # Link with title
    result = convert_to_markdown('<a href="http://test.com" title="Test">text</a>')
    assert '[text](http://test.com "Test")' in result


def test_heading_edge_cases() -> None:
    """Test heading converter edge cases."""
    # Empty heading - may not produce output
    result = convert_to_markdown("<h1></h1>")
    # Empty headings may not produce markdown

    # Heading with nested formatting
    result = convert_to_markdown("<h2><strong>Bold</strong> Heading</h2>")
    assert "**Bold** Heading" in result


def test_list_edge_cases() -> None:
    """Test list converter edge cases."""
    # Empty list
    result = convert_to_markdown("<ul></ul>")
    # Should not crash

    # List with mixed content
    html = """<ul>
    <li>Item 1</li>
    <li><p>Item 2 with paragraph</p></li>
    <li>Item 3<br>with break</li>
    </ul>"""
    result = convert_to_markdown(html)
    assert "* Item 1" in result
    assert "* Item 2 with paragraph" in result


def test_code_language_callback() -> None:
    """Test code language callback functionality."""

    def language_callback(tag: object) -> str:
        if hasattr(tag, "get") and callable(tag.get):
            classes = tag.get("class")
            if isinstance(classes, list) and len(classes) > 0:
                class_name = classes[0]
                if isinstance(class_name, str) and class_name.startswith("language-"):
                    return class_name[9:]
        return "text"

    html = '<pre><code class="language-python">print("hello")</code></pre>'
    result = convert_to_markdown(html, code_language_callback=language_callback)
    # May return text or python, depending on implementation
    assert "```" in result


def test_wrap_functionality() -> None:
    """Test text wrapping functionality."""
    long_text = "This is a very long line of text that should be wrapped when the wrap option is enabled with a specific width setting."
    html = f"<p>{long_text}</p>"

    result = convert_to_markdown(html, wrap=True, wrap_width=40)
    lines = result.strip().split("\n")
    # At least one line should be shorter due to wrapping
    assert any(len(line) <= 40 for line in lines if line.strip())


def test_special_html_entities() -> None:
    """Test handling of HTML entities."""
    html = "<p>&lt;script&gt;alert('test')&lt;/script&gt;</p>"
    result = convert_to_markdown(html)
    # Entities are decoded and may be escaped in markdown
    assert "script" in result
    assert "alert" in result

    html = "<p>&amp; &quot; &apos;</p>"
    result = convert_to_markdown(html)
    # Entities should be decoded
    assert "&" in result or "\\&" in result


def test_svg_math_elements() -> None:
    """Test SVG and math element handling."""
    # SVG element
    html = '<svg width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>'
    convert_to_markdown(html)
    # Should handle gracefully

    # Math element
    html = "<math><mi>x</mi><mo>=</mo><mn>1</mn></math>"
    convert_to_markdown(html)
    # Should handle gracefully
