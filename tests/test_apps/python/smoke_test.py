"""Smoke tests for html-to-markdown Python package."""

import html_to_markdown
from html_to_markdown import convert_html_to_markdown


def test_package_imports() -> None:
    """Test that package can be imported."""
    assert html_to_markdown is not None  # noqa: S101


def test_basic_conversion() -> None:
    """Test basic HTML to markdown conversion."""
    html = "<p>Hello World</p>"
    result = convert_html_to_markdown(html)
    assert "Hello World" in result  # noqa: S101


def test_with_options() -> None:
    """Test conversion with options."""
    html = "<h1>Title</h1>"
    result = convert_html_to_markdown(html)
    assert result.startswith("#")  # noqa: S101


def test_error_handling() -> None:
    """Test error handling."""
    # Should handle empty string
    result = convert_html_to_markdown("")
    assert result == ""  # noqa: S101
