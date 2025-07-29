"""Test for issue #50 - Paragraphs embedded in <li> not being indented correctly."""

from html_to_markdown import convert_to_markdown


def test_list_with_multiple_paragraphs() -> None:
    """Test that multiple paragraphs in list items are properly indented."""
    html = """<ul>
    <li>
        <p>This is an item.</p>
        <p>This is a second paragraph</p>
    </li>
    <li>This is a second item.</li>
    </ul>"""

    result = convert_to_markdown(html)

    # The second paragraph should be indented with 4 spaces
    assert "* This is an item." in result
    assert "    This is a second paragraph" in result
    assert "* This is a second item." in result


def test_ordered_list_with_multiple_paragraphs() -> None:
    """Test that multiple paragraphs in ordered list items are properly indented."""
    html = """<ol>
    <li>
        <p>This is the first step.</p>
        <p>It's important to remember to nibble the cheese first!</p>
    </li>
    <li>This is the second step.</li>
    </ol>"""

    result = convert_to_markdown(html)

    # The second paragraph should be indented with 4 spaces
    assert "1. This is the first step." in result
    assert "    It's important to remember to nibble the cheese first!" in result
    assert "2. This is the second step." in result


def test_nested_list_with_paragraphs() -> None:
    """Test that paragraphs in nested lists are properly handled."""
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

    # Should have proper indentation for both the paragraph and nested list
    assert "    Second paragraph of first item" in result
    assert "Nested item" in result


def test_list_with_blockquote() -> None:
    """Test list items containing blockquotes."""
    html = """<ul>
    <li>
        <p>First paragraph</p>
        <blockquote>This is a quote</blockquote>
        <p>Third paragraph</p>
    </li>
    </ul>"""

    result = convert_to_markdown(html)

    # Blockquote and subsequent paragraph should be indented
    assert "    > This is a quote" in result
    assert "    Third paragraph" in result


def test_mixed_content_list() -> None:
    """Test list with mixed inline and block content."""
    html = """<ul>
    <li>Simple item</li>
    <li>
        <p>Paragraph item</p>
        <p>Second paragraph</p>
    </li>
    <li>Another simple item</li>
    </ul>"""

    result = convert_to_markdown(html)

    # Simple items should remain simple, complex items should have proper indentation
    assert "* Simple item" in result
    assert "* Paragraph item" in result
    assert "    Second paragraph" in result
    assert "* Another simple item" in result
