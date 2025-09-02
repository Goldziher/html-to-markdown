from html_to_markdown import convert_to_markdown


def test_list_first_item_indent_with_strip_newlines() -> None:
    html = """<p>Text before</p>
<ul>
<li>First item</li>
<li>Second item</li>
</ul>"""

    result = convert_to_markdown(html, strip_newlines=True)

    lines = result.strip().split("\n")

    list_items = [line for line in lines if line.lstrip().startswith("*")]

    assert len(list_items) == 2, f"Expected 2 list items, got {len(list_items)}"

    first_item = list_items[0]
    second_item = list_items[1]

    assert first_item == "* First item", f"First item has incorrect format: {first_item!r}"
    assert second_item == "* Second item", f"Second item has incorrect format: {second_item!r}"

    assert not first_item.startswith(" "), "First item should not have leading space"
    assert not second_item.startswith(" "), "Second item should not have leading space"


def test_nested_list_indent_preserved_with_strip_newlines() -> None:
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

    assert list_items[0] == "* Parent item"

    assert list_items[1].startswith("\t"), f"Nested item should be indented: {list_items[1]!r}"


def test_multiple_blocks_with_strip_newlines() -> None:
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

    for item in list_items:
        if not item.startswith(("\t", "  ")):
            assert not item.startswith(" "), f"List item should not have leading space: {item!r}"
