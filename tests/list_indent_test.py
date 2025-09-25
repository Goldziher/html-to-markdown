from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

import pytest


def test_default_list_indent_4_spaces(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
    result = convert(html)
    assert "    + Nested item" in result


def test_custom_spaces_indent_2_spaces(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    assert "  + Nested item" in result
    assert "    + Nested item" not in result


def test_custom_spaces_indent_6_spaces(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
    result = convert(html, list_indent_width=6, list_indent_type="spaces")
    assert "      + Nested item" in result


def test_tabs_indent(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
    result = convert(html, list_indent_type="tabs")
    assert "\t+ Nested item" in result


def test_tabs_ignore_width(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
    result1 = convert(html, list_indent_type="tabs", list_indent_width=2)
    result2 = convert(html, list_indent_type="tabs", list_indent_width=8)
    assert result1 == result2
    assert "\t+ Nested item" in result1


def test_deeply_nested_lists(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>Level 1
            <ul>
                <li>Level 2
                    <ul>
                        <li>Level 3</li>
                    </ul>
                </li>
            </ul>
        </li>
    </ul>
    """
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    lines = result.split("\n")

    level1_line = next(line for line in lines if "Level 1" in line)
    level2_line = next(line for line in lines if "Level 2" in line)
    level3_line = next(line for line in lines if "Level 3" in line)

    assert level1_line.startswith("* Level 1")
    assert "  + Level 2" in level2_line
    assert "    - Level 3" in level3_line


def test_mixed_list_types_with_custom_indent(convert: Callable[..., str]) -> None:
    html = """
    <ol>
        <li>First ordered
            <ul>
                <li>First unordered</li>
            </ul>
        </li>
    </ol>
    """
    result = convert(html, list_indent_width=3, list_indent_type="spaces")
    assert "1. First ordered" in result
    assert "   * First unordered" in result


def test_blockquote_in_list_with_custom_indent(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>
            <p>Item with quote</p>
            <blockquote>This is a quote</blockquote>
        </li>
    </ul>
    """
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    assert "  > This is a quote" in result


def test_paragraph_in_list_with_custom_indent(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>
            <p>First paragraph</p>
            <p>Second paragraph</p>
        </li>
    </ul>
    """
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    lines = [line for line in result.split("\n") if line.strip()]

    first_para_line = next(line for line in lines if "First paragraph" in line)
    assert first_para_line.startswith("* First paragraph")

    second_para_line = next(line for line in lines if "Second paragraph" in line)
    assert "  Second paragraph" in second_para_line


def test_code_block_in_list_preserves_formatting(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li>Item with code
            <pre><code>def hello():
    print("world")</code></pre>
        </li>
    </ul>
    """
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    assert "def hello():" in result
    assert '    print("world")' in result


def test_task_list_with_custom_indent(convert: Callable[..., str]) -> None:
    html = """
    <ul>
        <li><input type="checkbox" checked> Completed task
            <ul>
                <li><input type="checkbox"> Subtask</li>
            </ul>
        </li>
    </ul>
    """
    result = convert(html, list_indent_width=2, list_indent_type="spaces")
    assert "- [x] Completed task" in result
    assert "  - [ ] Subtask" in result


def test_backward_compatibility_default_behavior(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
    result1 = convert(html)
    result2 = convert(html, list_indent_width=4, list_indent_type="spaces")
    assert result1 == result2
    assert "    + Nested" in result1


@pytest.mark.parametrize("indent_width", [1, 2, 3, 4, 5, 6, 8])
def test_various_indent_widths(indent_width: int, convert: Callable[..., str]) -> None:
    html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
    result = convert(html, list_indent_width=indent_width, list_indent_type="spaces")
    expected_spaces = " " * indent_width
    assert f"{expected_spaces}+ Nested" in result


def test_edge_case_zero_width_spaces(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
    result = convert(html, list_indent_width=0, list_indent_type="spaces")
    assert "+ Nested" in result
    assert " + Nested" not in result


def test_very_large_indent_width(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
    result = convert(html, list_indent_width=20, list_indent_type="spaces")
    expected_spaces = " " * 20
    assert f"{expected_spaces}+ Nested" in result


def test_list_indent_type_spaces(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested Item</li></ul></li></ul>"
    result = convert(html, list_indent_type="spaces", list_indent_width=2)
    assert "  + Nested Item" in result


def test_list_indent_type_tabs(convert: Callable[..., str]) -> None:
    html = "<ul><li>Item 1<ul><li>Nested Item</li></ul></li></ul>"
    result = convert(html, list_indent_type="tabs")
    assert "\t+ Nested Item" in result

    html = "<ul><li>Level 1<ul><li>Level 2<ul><li>Level 3</li></ul></li></ul></li></ul>"
    result = convert(html, list_indent_type="tabs")
    assert "\t+ Level 2" in result
    assert "\t\t- Level 3" in result
