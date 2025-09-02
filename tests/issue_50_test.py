from html_to_markdown import convert_to_markdown


def test_list_with_multiple_paragraphs() -> None:
    html = """<ul>
    <li>
        <p>This is an item.</p>
        <p>This is a second paragraph</p>
    </li>
    <li>This is a second item.</li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* This is an item." in result
    assert "    This is a second paragraph" in result
    assert "* This is a second item." in result


def test_ordered_list_with_multiple_paragraphs() -> None:
    html = """<ol>
    <li>
        <p>This is the first step.</p>
        <p>It's important to remember to nibble the cheese first!</p>
    </li>
    <li>This is the second step.</li>
    </ol>"""

    result = convert_to_markdown(html)

    assert "1. This is the first step." in result
    assert "    It's important to remember to nibble the cheese first!" in result
    assert "2. This is the second step." in result


def test_nested_list_with_paragraphs() -> None:
    html = """<ul>
    <li>
        <p>First item</p>
        <p>Second paragraph of first item</p>
        <ul>
            <li>Nested item</li>
        </ul>
    </li>
    <li>Second item</li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "    Second paragraph of first item" in result
    assert "Nested item" in result


def test_list_with_blockquote() -> None:
    html = """<ul>
    <li>
        <p>First paragraph</p>
        <blockquote>This is a quote</blockquote>
        <p>Third paragraph</p>
    </li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "    > This is a quote" in result
    assert "    Third paragraph" in result


def test_mixed_content_list() -> None:
    html = """<ul>
    <li>Simple item</li>
    <li>
        <p>Paragraph item</p>
        <p>Second paragraph</p>
    </li>
    <li>Another simple item</li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* Simple item" in result
    assert "* Paragraph item" in result
    assert "    Second paragraph" in result
    assert "* Another simple item" in result
