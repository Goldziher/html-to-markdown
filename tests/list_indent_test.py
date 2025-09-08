"""Tests for configurable list indentation feature."""

import pytest

from html_to_markdown import convert_to_markdown


class TestListIndentConfiguration:
    """Test configurable list indent functionality."""

    def test_default_list_indent_4_spaces(self) -> None:
        """Test that default list indentation is 4 spaces."""
        html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
        result = convert_to_markdown(html)

        # Should use 4 spaces by default
        assert "    + Nested item" in result

    def test_custom_spaces_indent_2_spaces(self) -> None:
        """Test 2-space indentation (Discord-friendly)."""
        html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")

        # Should use 2 spaces
        assert "  + Nested item" in result
        assert "    + Nested item" not in result

    def test_custom_spaces_indent_6_spaces(self) -> None:
        """Test 6-space indentation."""
        html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=6, list_indent_type="spaces")

        # Should use 6 spaces
        assert "      + Nested item" in result

    def test_tabs_indent(self) -> None:
        """Test tab indentation."""
        html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_type="tabs")

        # Should use 1 tab (width is ignored for tabs)
        assert "\t+ Nested item" in result

    def test_tabs_ignore_width(self) -> None:
        """Test that tabs mode ignores width parameter."""
        html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
        result1 = convert_to_markdown(html, list_indent_type="tabs", list_indent_width=2)
        result2 = convert_to_markdown(html, list_indent_type="tabs", list_indent_width=8)

        # Both should use 1 tab regardless of width
        assert result1 == result2
        assert "\t+ Nested item" in result1

    def test_deeply_nested_lists(self) -> None:
        """Test deeply nested lists with custom indentation."""
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

        # Test with 2 spaces
        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")
        lines = result.split("\n")

        level1_line = next(line for line in lines if "Level 1" in line)
        level2_line = next(line for line in lines if "Level 2" in line)
        level3_line = next(line for line in lines if "Level 3" in line)

        # Level 1 should have no indent (bullet only)
        assert level1_line.startswith("* Level 1")

        # Level 2 should have 2 spaces
        assert "  + Level 2" in level2_line

        # Level 3 should have 4 spaces (2 * 2)
        assert "    - Level 3" in level3_line

    def test_mixed_list_types_with_custom_indent(self) -> None:
        """Test mixed ordered and unordered lists with custom indentation."""
        html = """
        <ol>
            <li>First ordered
                <ul>
                    <li>First unordered</li>
                </ul>
            </li>
        </ol>
        """

        result = convert_to_markdown(html, list_indent_width=3, list_indent_type="spaces")

        assert "1. First ordered" in result
        assert "   * First unordered" in result

    def test_blockquote_in_list_with_custom_indent(self) -> None:
        """Test blockquote inside list with custom indentation."""
        html = """
        <ul>
            <li>
                <p>Item with quote</p>
                <blockquote>This is a quote</blockquote>
            </li>
        </ul>
        """

        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")

        # Blockquote should be indented with 2 spaces + "> "
        assert "  > This is a quote" in result

    def test_paragraph_in_list_with_custom_indent(self) -> None:
        """Test paragraph inside list with custom indentation."""
        html = """
        <ul>
            <li>
                <p>First paragraph</p>
                <p>Second paragraph</p>
            </li>
        </ul>
        """

        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")
        lines = [line for line in result.split("\n") if line.strip()]

        # First paragraph should be part of bullet
        first_para_line = next(line for line in lines if "First paragraph" in line)
        assert first_para_line.startswith("* First paragraph")

        # Second paragraph should be indented
        second_para_line = next(line for line in lines if "Second paragraph" in line)
        assert "  Second paragraph" in second_para_line

    def test_code_block_in_list_preserves_formatting(self) -> None:
        """Test that code blocks in lists preserve their formatting."""
        html = """
        <ul>
            <li>Item with code
                <pre><code>def hello():
    print("world")</code></pre>
            </li>
        </ul>
        """

        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")

        # Code should maintain its internal formatting
        assert "def hello():" in result
        assert '    print("world")' in result

    def test_task_list_with_custom_indent(self) -> None:
        """Test task lists (checkboxes) with custom indentation."""
        html = """
        <ul>
            <li><input type="checkbox" checked> Completed task
                <ul>
                    <li><input type="checkbox"> Subtask</li>
                </ul>
            </li>
        </ul>
        """

        result = convert_to_markdown(html, list_indent_width=2, list_indent_type="spaces")

        assert "- [x] Completed task" in result
        assert "  - [ ] Subtask" in result

    def test_backward_compatibility_default_behavior(self) -> None:
        """Test that not specifying indent parameters maintains backward compatibility."""
        html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"

        # These should be equivalent
        result1 = convert_to_markdown(html)
        result2 = convert_to_markdown(html, list_indent_width=4, list_indent_type="spaces")

        assert result1 == result2
        assert "    + Nested" in result1

    @pytest.mark.parametrize("indent_width", [1, 2, 3, 4, 5, 6, 8])
    def test_various_indent_widths(self, indent_width: int) -> None:
        """Test various indent widths work correctly."""
        html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=indent_width, list_indent_type="spaces")

        expected_spaces = " " * indent_width
        assert f"{expected_spaces}+ Nested" in result

    def test_edge_case_zero_width_spaces(self) -> None:
        """Test edge case of zero width spaces."""
        html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=0, list_indent_type="spaces")

        # Should result in no indentation
        assert "+ Nested" in result
        assert " + Nested" not in result

    def test_very_large_indent_width(self) -> None:
        """Test very large indent width."""
        html = "<ul><li>Item<ul><li>Nested</li></ul></li></ul>"
        result = convert_to_markdown(html, list_indent_width=20, list_indent_type="spaces")

        # Should use 20 spaces
        expected_spaces = " " * 20
        assert f"{expected_spaces}+ Nested" in result
