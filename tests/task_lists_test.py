from html_to_markdown import convert_to_markdown


def test_unchecked_task_item() -> None:
    html = '<ul><li><input type="checkbox"> Unchecked task</li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] Unchecked task\n"


def test_checked_task_item() -> None:
    html = '<ul><li><input type="checkbox" checked> Checked task</li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [x] Checked task\n"


def test_checked_task_item_with_value() -> None:
    html = '<ul><li><input type="checkbox" checked="checked"> Checked task</li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [x] Checked task\n"


def test_multiple_task_items() -> None:
    html = '<ul><li><input type="checkbox"> First task</li><li><input type="checkbox" checked> Second task</li><li><input type="checkbox"> Third task</li></ul>'
    result = convert_to_markdown(html)
    expected = "- [ ] First task\n- [x] Second task\n- [ ] Third task\n"
    assert result == expected


def test_mixed_regular_and_task_items() -> None:
    html = '<ul><li>Regular item</li><li><input type="checkbox"> Task item</li><li>Another regular item</li></ul>'
    result = convert_to_markdown(html)
    expected = "* Regular item\n- [ ] Task item\n* Another regular item\n"
    assert result == expected


def test_nested_task_lists() -> None:
    html = '<ul><li><input type="checkbox"> Parent task<ul><li><input type="checkbox" checked> Child task 1</li><li><input type="checkbox"> Child task 2</li></ul></li></ul>'
    result = convert_to_markdown(html)
    expected = "- [ ] Parent task\n\n    \n    \n    - [x] Child task 1\n    - [ ] Child task 2\n"
    assert result == expected


def test_task_with_inline_formatting() -> None:
    html = '<ul><li><input type="checkbox"> Task with <strong>bold</strong> and <em>italic</em> text</li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] Task with **bold** and *italic* text\n"


def test_task_with_links() -> None:
    html = '<ul><li><input type="checkbox"> Task with <a href="https://example.com">link</a></li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] Task with [link](https://example.com)\n"


def test_task_with_code() -> None:
    html = '<ul><li><input type="checkbox"> Task with <code>code</code></li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] Task with `code`\n"


def test_ordered_list_with_tasks() -> None:
    html = '<ol><li><input type="checkbox"> Task in ordered list</li><li><input type="checkbox" checked> Another task</li></ol>'
    result = convert_to_markdown(html)
    expected = "- [ ] Task in ordered list\n- [x] Another task\n"
    assert result == expected


def test_checkbox_without_task_text() -> None:
    html = '<ul><li><input type="checkbox"></li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] \n"


def test_checkbox_with_only_whitespace() -> None:
    html = '<ul><li><input type="checkbox">   </li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] \n"


def test_multiple_checkboxes_in_one_item() -> None:
    html = '<ul><li><input type="checkbox"> First <input type="checkbox" checked> Second</li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] First  Second\n"


def test_checkbox_with_complex_content() -> None:
    html = '<ul><li><input type="checkbox"> Complex task with:<p>Paragraph content</p><blockquote>Quote content</blockquote></li></ul>'
    result = convert_to_markdown(html)
    expected = "- [ ] Complex task with:\n\nParagraph content\n\n    > Quote content\n"
    assert result == expected


def test_non_checkbox_input_ignored() -> None:
    html = '<ul><li><input type="text" value="text input"> Regular item</li><li><input type="checkbox"> Task item</li></ul>'
    result = convert_to_markdown(html)
    expected = "* Regular item\n- [ ] Task item\n"
    assert result == expected


def test_checkbox_input_attributes() -> None:
    html = '<ul><li><input type="checkbox" id="task1" class="task-checkbox" data-id="1"> Task with attributes</li><li><input type="checkbox" checked disabled> Disabled checked task</li></ul>'
    result = convert_to_markdown(html)
    expected = "- [ ] Task with attributes\n- [x] Disabled checked task\n"
    assert result == expected


def test_checkbox_in_div_within_li() -> None:
    html = '<ul><li><div><input type="checkbox"> Task in div</div></li></ul>'
    result = convert_to_markdown(html)
    assert result == "- [ ] Task in div\n"


def test_deep_nested_task_lists() -> None:
    html = '<ul><li><input type="checkbox"> Level 1<ul><li><input type="checkbox" checked> Level 2<ul><li><input type="checkbox"> Level 3</li></ul></li></ul></li></ul>'
    result = convert_to_markdown(html)
    expected = "- [ ] Level 1\n\n    \n    \n    - [x] Level 2\n    \n    \n        \n        \n        - [ ] Level 3\n"
    assert result == expected


def test_task_list_edge_cases() -> None:
    html = '<ul><li><input type="checkbox" checked=""> Checked with empty value</li><li><input type="checkbox" checked="false"> Checked with false value</li><li><input type="checkbox" checked="true"> Checked with true value</li></ul>'
    result = convert_to_markdown(html)

    expected = "- [x] Checked with empty value\n- [x] Checked with false value\n- [x] Checked with true value\n"
    assert result == expected
