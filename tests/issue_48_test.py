"""Tests for issue #48 - First list bullet getting extra indent when strip_newlines=True."""

from html_to_markdown import convert_to_markdown


def test_list_first_item_indent_with_strip_newlines() -> None:
    """Test that first list item doesn't get extra indent when strip_newlines=True."""
    html = """<p>Text before</p>
<ul>
<li>First item</li>
<li>Second item</li>
</ul>"""

    result = convert_to_markdown(html, strip_newlines=True)

    # Split into lines and check indentation
    lines = result.strip().split("\n")

    # Find list items
    list_items = [line for line in lines if line.lstrip().startswith("*")]

    assert len(list_items) == 2, f"Expected 2 list items, got {len(list_items)}"

    # Check that both items have the same indentation (no indent)
    first_item = list_items[0]
    second_item = list_items[1]

    assert first_item == "* First item", f"First item has incorrect format: {first_item!r}"
    assert second_item == "* Second item", f"Second item has incorrect format: {second_item!r}"

    # Both should start at column 0 (no leading spaces)
    assert not first_item.startswith(" "), "First item should not have leading space"
    assert not second_item.startswith(" "), "Second item should not have leading space"


def test_nested_list_indent_preserved_with_strip_newlines() -> None:
    """Test that nested list indentation is preserved when strip_newlines=True."""
    html = """<ul>
<li>Parent item
<ul>
<li>Nested item</li>
</ul>
</li>
</ul>"""

    result = convert_to_markdown(html, strip_newlines=True)

    lines = result.strip().split("\n")
    list_items = [line for line in lines if line.lstrip().startswith(("*", "+", "-"))]

    assert len(list_items) == 2

    # Parent should have no indent
    assert list_items[0] == "* Parent item"

    # Nested item should be indented (with tab)
    assert list_items[1].startswith("\t"), f"Nested item should be indented: {list_items[1]!r}"


def test_multiple_blocks_with_strip_newlines() -> None:
    """Test multiple block elements with lists when strip_newlines=True."""
    html = """<h1>Title</h1>
<p>First paragraph</p>
<ul>
<li>List item 1</li>
<li>List item 2</li>
</ul>
<p>After list</p>"""

    result = convert_to_markdown(html, strip_newlines=True)

    lines = result.strip().split("\n")
    list_items = [line for line in lines if line.lstrip().startswith(("*", "+", "-"))]

    # All top-level list items should have no leading spaces
    for item in list_items:
        if not item.startswith(("\t", "  ")):  # Skip nested items
            assert not item.startswith(" "), f"List item should not have leading space: {item!r}"
