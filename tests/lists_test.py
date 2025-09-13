"""Tests for list conversion including nested lists and special cases."""

from __future__ import annotations

from html_to_markdown import convert_to_markdown


def test_basic_unordered_list() -> None:
    html = """<ul>
    <li>Item 1</li>
    <li>Item 2</li>
    <li>Item 3</li>
    </ul>"""

    result = convert_to_markdown(html)
    assert "* Item 1" in result
    assert "* Item 2" in result
    assert "* Item 3" in result


def test_basic_ordered_list() -> None:
    html = """<ol>
    <li>First</li>
    <li>Second</li>
    <li>Third</li>
    </ol>"""

    result = convert_to_markdown(html)
    assert "1. First" in result
    assert "2. Second" in result
    assert "3. Third" in result


def test_list_first_item_indent_with_strip_newlines() -> None:
    html = """
    <p>Above</p>
    <ul>
    <li>First</li>
    <li>Second</li>
    </ul>
    """

    result = convert_to_markdown(html, strip_newlines=True)

    lines = result.split("\n")
    list_lines = [line for line in lines if line.strip().startswith("*")]

    if list_lines:
        first_item = list_lines[0]
        assert not first_item.startswith("    *"), "First item should not have extra indent"
        assert first_item.startswith("*"), "First item should start with bullet"


def test_list_indentation_consistency() -> None:
    html = """
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
    """

    result_normal = convert_to_markdown(html)
    result_stripped = convert_to_markdown(html, strip_newlines=True)

    for result in [result_normal, result_stripped]:
        lines = result.split("\n")
        list_lines = [line for line in lines if line.strip().startswith("*")]

        if len(list_lines) > 1:
            first_indent = len(list_lines[0]) - len(list_lines[0].lstrip())
            for line in list_lines[1:]:
                indent = len(line) - len(line.lstrip())
                assert indent == first_indent, f"Inconsistent indentation: {indent} != {first_indent}"


def test_list_with_multiple_paragraphs() -> None:
    html = """<ul>
    <li>
        <p>First paragraph</p>
        <p>Second paragraph</p>
    </li>
    <li>
        <p>Another item</p>
    </li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* First paragraph" in result
    assert "Second paragraph" in result

    lines = result.split("\n")
    for line in lines:
        if "Second paragraph" in line:
            assert line.startswith(("    ", "\t")), "Second paragraph should be indented"


def test_list_with_nested_paragraphs_complex() -> None:
    html = """<ol>
    <li>
        <p>Item 1 first paragraph</p>
        <p>Item 1 second paragraph</p>
    </li>
    <li>Simple item</li>
    <li>
        <p>Item 3 with paragraph</p>
    </li>
    </ol>"""

    result = convert_to_markdown(html)

    assert "1. Item 1 first paragraph" in result
    assert "Item 1 second paragraph" in result
    assert "2. Simple item" in result
    assert "3. Item 3 with paragraph" in result


def test_nested_list_not_inside_li() -> None:
    html = "<ul><li>a</li><li>b</li><ul><li>c</li><li>d</li></ul></ul>"

    result = convert_to_markdown(html)

    expected = "* a\n* b\n    + c\n    + d\n"
    assert result == expected


def test_nested_list_not_inside_li_with_multiple_levels() -> None:
    html = """<ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <ul>
            <li>Subitem 2.1</li>
            <li>Subitem 2.2</li>
            <ul>
                <li>Sub-subitem</li>
            </ul>
        </ul>
        <li>Item 3</li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* Item 1" in result
    assert "* Item 2" in result
    assert "    + Subitem 2\\.1" in result
    assert "    + Subitem 2\\.2" in result
    assert "        - Sub\\-subitem" in result
    assert "* Item 3" in result


def test_mixed_correct_and_incorrect_nesting() -> None:
    html = """<ul>
        <li>Item 1
            <ul>
                <li>Correctly nested 1.1</li>
                <li>Correctly nested 1.2</li>
            </ul>
        </li>
        <li>Item 2</li>
        <ul>
            <li>Incorrectly nested 2.1</li>
            <li>Incorrectly nested 2.2</li>
        </ul>
        <li>Item 3</li>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* Item 1" in result
    assert "    + Correctly nested 1\\.1" in result
    assert "    + Correctly nested 1\\.2" in result
    assert "* Item 2" in result
    assert "    + Incorrectly nested 2\\.1" in result
    assert "    + Incorrectly nested 2\\.2" in result
    assert "* Item 3" in result


def test_ordered_list_incorrectly_nested() -> None:
    html = "<ol><li>First</li><li>Second</li><ol><li>Nested first</li><li>Nested second</li></ol></ol>"

    result = convert_to_markdown(html)

    expected_lines = ["1. First", "2. Second", "    1. Nested first", "    2. Nested second"]

    for line in expected_lines:
        assert line in result


def test_deeply_incorrect_nesting() -> None:
    html = """<ul>
        <li>Level 1</li>
        <ul>
            <li>Level 2</li>
            <ul>
                <li>Level 3</li>
                <ul>
                    <li>Level 4</li>
                </ul>
            </ul>
        </ul>
    </ul>"""

    result = convert_to_markdown(html)

    assert "* Level 1" in result
    assert "    + Level 2" in result
    assert "        - Level 3" in result
    assert "            * Level 4" in result


def test_list_after_paragraph_with_empty_lines() -> None:
    """Test list with empty lines after paragraph in list item."""
    html = """<ul>
        <li>
            <p>First paragraph</p>
            <ul>
                <li>Item with content

                and empty line</li>
                <li>Second item</li>
            </ul>
        </li>
    </ul>"""

    result = convert_to_markdown(html)
    assert "First paragraph" in result
    assert "Item with content" in result
    assert "Second item" in result


def test_nested_list_without_preceding_paragraph() -> None:
    """Test nested list without preceding paragraph."""
    html = """<ul>
        <li>
            <ul>
                <li>Direct nested item</li>
            </ul>
        </li>
    </ul>"""

    result = convert_to_markdown(html)
    assert "Direct nested item" in result


def test_empty_line_handling_in_nested_list() -> None:
    """Test empty line handling in nested lists."""
    html = """<ul>
        <li>
            <p>Paragraph before</p>
            <ol>
                <li>First item</li>
                <li></li>
                <li>Third item</li>
            </ol>
        </li>
    </ul>"""

    result = convert_to_markdown(html)
    assert "Paragraph before" in result
    assert "First item" in result
    assert "Third item" in result
