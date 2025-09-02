"""Test for issue #59 - Nested lists are not indented."""

from html_to_markdown import convert_to_markdown


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
    assert "\t+ Correctly nested 1\\.1" in result
    assert "\t+ Correctly nested 1\\.2" in result
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
