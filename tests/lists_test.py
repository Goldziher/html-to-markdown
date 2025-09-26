"""Tests for list conversion including nested lists and special cases."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

import pytest


def test_basic_unordered_list(convert: Callable[..., str]) -> None:
    html = """<ul>
    <li>Item 1</li>
    <li>Item 2</li>
    <li>Item 3</li>
    </ul>"""

    result = convert(html)
    assert "* Item 1" in result
    assert "* Item 2" in result
    assert "* Item 3" in result


def test_basic_ordered_list(convert: Callable[..., str]) -> None:
    html = """<ol>
    <li>First</li>
    <li>Second</li>
    <li>Third</li>
    </ol>"""

    result = convert(html)
    assert "1. First" in result
    assert "2. Second" in result
    assert "3. Third" in result


def test_list_first_item_indent_with_strip_newlines(convert: Callable[..., str]) -> None:
    html = """
    <p>Above</p>
    <ul>
    <li>First</li>
    <li>Second</li>
    </ul>
    """

    result = convert(html, strip_newlines=True)

    lines = result.split("\n")
    list_lines = [line for line in lines if line.strip().startswith("*")]

    if list_lines:
        first_item = list_lines[0]
        assert not first_item.startswith("    *"), "First item should not have extra indent"
        assert first_item.startswith("*"), "First item should start with bullet"


def test_list_indentation_consistency(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
    """

    result_normal = convert(html)
    result_stripped = convert(html, strip_newlines=True)

    for result in [result_normal, result_stripped]:
        lines = result.split("\n")
        list_lines = [line for line in lines if line.strip().startswith("*")]

        if len(list_lines) > 1:
            first_indent = len(list_lines[0]) - len(list_lines[0].lstrip())
            for line in list_lines[1:]:
                indent = len(line) - len(line.lstrip())
                assert indent == first_indent, f"Inconsistent indentation: {indent} != {first_indent}"


def test_list_with_multiple_paragraphs(convert: Callable[..., str]) -> None:
    html = """<ul>
    <li>
        <p>First paragraph</p>
        <p>Second paragraph</p>
    </li>
    <li>
        <p>Another item</p>
    </li>
    </ul>"""

    result = convert(html)

    assert "* First paragraph" in result
    assert "Second paragraph" in result

    lines = result.split("\n")
    for line in lines:
        if "Second paragraph" in line:
            assert line.startswith(("    ", "\t")), "Second paragraph should be indented"


def test_list_with_nested_paragraphs_complex(convert: Callable[..., str]) -> None:
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

    result = convert(html)

    assert "1. Item 1 first paragraph" in result
    assert "Item 1 second paragraph" in result
    assert "2. Simple item" in result
    assert "3. Item 3 with paragraph" in result


def test_nested_list_not_inside_li(convert: Callable[..., str]) -> None:
    html = "<ul><li>a</li><li>b</li><ul><li>c</li><li>d</li></ul></ul>"

    result = convert(html)

    expected = "* a\n* b\n    + c\n    + d\n"
    assert result == expected


def test_nested_list_not_inside_li_with_multiple_levels(convert: Callable[..., str]) -> None:
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

    result = convert(html)

    assert "* Item 1" in result
    assert "* Item 2" in result
    assert "    + Subitem 2\\.1" in result
    assert "    + Subitem 2\\.2" in result
    assert "        - Sub\\-subitem" in result
    assert "* Item 3" in result


def test_mixed_correct_and_incorrect_nesting(convert: Callable[..., str]) -> None:
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

    result = convert(html)

    assert "* Item 1" in result
    assert "    + Correctly nested 1\\.1" in result
    assert "    + Correctly nested 1\\.2" in result
    assert "* Item 2" in result
    assert "    + Incorrectly nested 2\\.1" in result
    assert "    + Incorrectly nested 2\\.2" in result
    assert "* Item 3" in result


def test_ordered_list_incorrectly_nested(convert: Callable[..., str]) -> None:
    html = "<ol><li>First</li><li>Second</li><ol><li>Nested first</li><li>Nested second</li></ol></ol>"

    result = convert(html)

    expected_lines = ["1. First", "2. Second", "    1. Nested first", "    2. Nested second"]

    for line in expected_lines:
        assert line in result


def test_deeply_incorrect_nesting(convert: Callable[..., str]) -> None:
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

    result = convert(html)

    assert "* Level 1" in result
    assert "    + Level 2" in result
    assert "        - Level 3" in result
    assert "            * Level 4" in result


def test_list_after_paragraph_with_empty_lines(convert: Callable[..., str]) -> None:
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

    result = convert(html)
    assert "First paragraph" in result
    assert "Item with content" in result
    assert "Second item" in result


def test_nested_list_without_preceding_paragraph(convert: Callable[..., str]) -> None:
    html = """<ul>
        <li>
            <ul>
                <li>Direct nested item</li>
            </ul>
        </li>
    </ul>"""

    result = convert(html)
    assert "Direct nested item" in result


def test_empty_line_handling_in_nested_list(convert: Callable[..., str]) -> None:
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

    result = convert(html)
    assert "Paragraph before" in result
    assert "First item" in result
    assert "Third item" in result


@pytest.mark.parametrize(
    "html,expected",
    [
        (
            """<ul>
<li>Item 1</li>
<li>
    <div>Item 2-1</div>
    <div>Item 2-2</div>
</li>
</ul>""",
            "* Item 1\n* Item 2\\-1\n\n    Item 2\\-2\n\n",
        ),
        (
            """<ul>
<li><p>First paragraph</p><p>Second paragraph</p></li>
<li>Simple item</li>
</ul>""",
            "* First paragraph\n\n    Second paragraph\n\n* Simple item\n\n",
        ),
        (
            """<ol>
<li>First item</li>
<li>
    <div>Second item line 1</div>
    <div>Second item line 2</div>
</li>
<li>Third item</li>
</ol>""",
            "1. First item\n2. Second item line 1\n\n    Second item line 2\n\n3. Third item\n\n",
        ),
        (
            """<ul>
<li>
    <div>Main content</div>
    <ul><li>Nested item</li></ul>
    <div>More content</div>
</li>
</ul>""",
            "* Main content\n\n    + Nested item\n\n    More content\n\n",
        ),
        (
            """<ul>
<li>
    <div>
        <p>Deep paragraph 1</p>
        <p>Deep paragraph 2</p>
    </div>
</li>
</ul>""",
            "* Deep paragraph 1\n\n    Deep paragraph 2\n\n",
        ),
        (
            """<ul>
<li>
    <div>Item 1 line 1</div>
    <div>Item 1 line 2</div>
</li>
<li>
    <div>Item 2 line 1</div>
    <div>Item 2 line 2</div>
</li>
</ul>""",
            "* Item 1 line 1\n\n    Item 1 line 2\n\n* Item 2 line 1\n\n    Item 2 line 2\n\n",
        ),
        (
            """<ol>
<li>
    <p>First paragraph</p>
    <div>Middle div</div>
    <p>Last paragraph</p>
</li>
</ol>""",
            "1. First paragraph\n\n    Middle div\n\n    Last paragraph\n\n",
        ),
        (
            """<ul>
<li>Level 1
    <ul>
    <li>Level 2
        <div>Content line 1</div>
        <div>Content line 2</div>
    </li>
    </ul>
</li>
</ul>""",
            "* Level 1\n\n    + Level 2\n            Content line 1\n            Content line 2\n\n",
        ),
    ],
)
def test_multiline_list_item_indentation_issues(html: str, expected: str, convert: Callable[..., str]) -> None:
    result = convert(html)
    assert result == expected
