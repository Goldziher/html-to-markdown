from html_to_markdown import convert_to_markdown
from html_to_markdown.converters import create_converters_map


def test_create_converters_map() -> None:
    converters = create_converters_map(
        autolinks=True,
        bullets="*+-",
        code_language="",
        code_language_callback=None,
        default_title=False,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
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

    converters = create_converters_map(
        autolinks=False,
        bullets="-*+",
        code_language="python",
        code_language_callback=None,
        default_title=True,
        heading_style="atx",
        highlight_style="html",
        keep_inline_images_in=["p"],
        list_indent_type="tabs",
        list_indent_width=2,
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
    result = convert_to_markdown("<blockquote></blockquote>")

    result = convert_to_markdown("<strong><em>text</em></strong>")
    assert "***text***" in result

    result = convert_to_markdown("<pre><code>&lt;script&gt;</code></pre>")
    assert "<script>" in result

    result = convert_to_markdown("<ul><li></li></ul>")
    assert "*" in result


def test_table_edge_cases() -> None:
    html = """<table>
    <tr><td></td><td>content</td></tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "|  | content |" in result

    html = """<table>
    <tr><td><strong>bold</strong> and <em>italic</em></td></tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "**bold** and *italic*" in result


def test_image_edge_cases() -> None:
    result = convert_to_markdown('<img src="test.jpg" alt="">')
    assert "![](test.jpg)" in result

    result = convert_to_markdown('<img src="test.jpg">')
    assert "![](test.jpg)" in result

    result = convert_to_markdown('<img src="test.jpg" alt="Test" title="Test Image">')
    assert '![Test](test.jpg "Test Image")' in result


def test_link_edge_cases() -> None:
    result = convert_to_markdown("<a>text</a>")
    assert result.strip() == "text"

    result = convert_to_markdown('<a href="">text</a>')
    assert "text" in result

    result = convert_to_markdown('<a href="http://test.com" title="Test">text</a>')
    assert '[text](http://test.com "Test")' in result


def test_heading_edge_cases() -> None:
    result = convert_to_markdown("<h1></h1>")

    result = convert_to_markdown("<h2><strong>Bold</strong> Heading</h2>")
    assert "**Bold** Heading" in result


def test_list_edge_cases() -> None:
    result = convert_to_markdown("<ul></ul>")

    html = """<ul>
    <li>Item 1</li>
    <li><p>Item 2 with paragraph</p></li>
    <li>Item 3<br>with break</li>
    </ul>"""
    result = convert_to_markdown(html)
    assert "* Item 1" in result
    assert "* Item 2 with paragraph" in result


def test_code_language_callback() -> None:
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
    assert "```" in result


def test_wrap_functionality() -> None:
    long_text = "This is a very long line of text that should be wrapped when the wrap option is enabled with a specific width setting."
    html = f"<p>{long_text}</p>"

    result = convert_to_markdown(html, wrap=True, wrap_width=40)
    lines = result.strip().split("\n")
    assert any(len(line) <= 40 for line in lines if line.strip())


def test_special_html_entities() -> None:
    html = "<p>&lt;script&gt;alert('test')&lt;/script&gt;</p>"
    result = convert_to_markdown(html)
    assert "script" in result
    assert "alert" in result

    html = "<p>&amp; &quot; &apos;</p>"
    result = convert_to_markdown(html)
    assert "&" in result or "\\&" in result


def test_svg_math_elements() -> None:
    html = '<svg width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>'
    convert_to_markdown(html)

    html = "<math><mi>x</mi><mo>=</mo><mn>1</mn></math>"
    convert_to_markdown(html)
