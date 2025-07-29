"""Tests for task list functionality (GitHub-flavored Markdown)."""

from html_to_markdown import convert_to_markdown


class TestTaskLists:
    """Test task list (checkbox) conversion."""

    def test_unchecked_task_item(self) -> None:
        """Test unchecked task list item."""
        html = '<ul><li><input type="checkbox"> Unchecked task</li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] Unchecked task\n"

    def test_checked_task_item(self) -> None:
        """Test checked task list item."""
        html = '<ul><li><input type="checkbox" checked> Checked task</li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [x] Checked task\n"

    def test_checked_task_item_with_value(self) -> None:
        """Test checked task list item with checked='checked'."""
        html = '<ul><li><input type="checkbox" checked="checked"> Checked task</li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [x] Checked task\n"

    def test_multiple_task_items(self) -> None:
        """Test multiple task list items."""
        html = '<ul><li><input type="checkbox"> First task</li><li><input type="checkbox" checked> Second task</li><li><input type="checkbox"> Third task</li></ul>'
        result = convert_to_markdown(html)
        expected = "- [ ] First task\n- [x] Second task\n- [ ] Third task\n"
        assert result == expected

    def test_mixed_regular_and_task_items(self) -> None:
        """Test list with both regular and task items."""
        html = '<ul><li>Regular item</li><li><input type="checkbox"> Task item</li><li>Another regular item</li></ul>'
        result = convert_to_markdown(html)
        expected = "* Regular item\n- [ ] Task item\n* Another regular item\n"
        assert result == expected

    def test_nested_task_lists(self) -> None:
        """Test nested task lists."""
        html = '<ul><li><input type="checkbox"> Parent task<ul><li><input type="checkbox" checked> Child task 1</li><li><input type="checkbox"> Child task 2</li></ul></li></ul>'
        result = convert_to_markdown(html)
        expected = "- [ ] Parent task\n\t- [x] Child task 1\n\t- [ ] Child task 2\n"
        assert result == expected

    def test_task_with_inline_formatting(self) -> None:
        """Test task item with inline formatting."""
        html = '<ul><li><input type="checkbox"> Task with <strong>bold</strong> and <em>italic</em> text</li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] Task with **bold** and *italic* text\n"

    def test_task_with_links(self) -> None:
        """Test task item with links."""
        html = '<ul><li><input type="checkbox"> Task with <a href="https://example.com">link</a></li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] Task with [link](https://example.com)\n"

    def test_task_with_code(self) -> None:
        """Test task item with code."""
        html = '<ul><li><input type="checkbox"> Task with <code>code</code></li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] Task with `code`\n"

    def test_ordered_list_with_tasks(self) -> None:
        """Test ordered list should not become tasks."""
        html = '<ol><li><input type="checkbox"> Task in ordered list</li><li><input type="checkbox" checked> Another task</li></ol>'
        result = convert_to_markdown(html)
        expected = "- [ ] Task in ordered list\n- [x] Another task\n"
        assert result == expected

    def test_checkbox_without_task_text(self) -> None:
        """Test checkbox without any text."""
        html = '<ul><li><input type="checkbox"></li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] \n"

    def test_checkbox_with_only_whitespace(self) -> None:
        """Test checkbox with only whitespace text."""
        html = '<ul><li><input type="checkbox">   </li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] \n"

    def test_multiple_checkboxes_in_one_item(self) -> None:
        """Test multiple checkboxes in one list item (should only process first)."""
        html = '<ul><li><input type="checkbox"> First <input type="checkbox" checked> Second</li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] First  Second\n"

    def test_checkbox_with_complex_content(self) -> None:
        """Test checkbox with complex nested content."""
        html = '<ul><li><input type="checkbox"> Complex task with:<p>Paragraph content</p><blockquote>Quote content</blockquote></li></ul>'
        result = convert_to_markdown(html)
        expected = "- [ ] Complex task with:Paragraph content\n\n    > Quote content\n"
        assert result == expected

    def test_non_checkbox_input_ignored(self) -> None:
        """Test that non-checkbox input types are ignored."""
        html = '<ul><li><input type="text" value="text input"> Regular item</li><li><input type="checkbox"> Task item</li></ul>'
        result = convert_to_markdown(html)
        expected = "* Regular item\n- [ ] Task item\n"
        assert result == expected

    def test_checkbox_input_attributes(self) -> None:
        """Test checkbox with various attributes."""
        html = '<ul><li><input type="checkbox" id="task1" class="task-checkbox" data-id="1"> Task with attributes</li><li><input type="checkbox" checked disabled> Disabled checked task</li></ul>'
        result = convert_to_markdown(html)
        expected = "- [ ] Task with attributes\n- [x] Disabled checked task\n"
        assert result == expected

    def test_checkbox_in_div_within_li(self) -> None:
        """Test checkbox inside div within list item."""
        html = '<ul><li><div><input type="checkbox"> Task in div</div></li></ul>'
        result = convert_to_markdown(html)
        assert result == "- [ ] Task in div\n"

    def test_deep_nested_task_lists(self) -> None:
        """Test deeply nested task lists."""
        html = '<ul><li><input type="checkbox"> Level 1<ul><li><input type="checkbox" checked> Level 2<ul><li><input type="checkbox"> Level 3</li></ul></li></ul></li></ul>'
        result = convert_to_markdown(html)
        expected = "- [ ] Level 1\n\t- [x] Level 2\n\t\t- [ ] Level 3\n"
        assert result == expected

    def test_task_list_edge_cases(self) -> None:
        """Test various edge cases for task lists."""
        html = '<ul><li><input type="checkbox" checked=""> Checked with empty value</li><li><input type="checkbox" checked="false"> Checked with false value</li><li><input type="checkbox" checked="true"> Checked with true value</li></ul>'
        result = convert_to_markdown(html)

        expected = "- [x] Checked with empty value\n- [x] Checked with false value\n- [x] Checked with true value\n"
        assert result == expected
