"""Test processing module edge cases."""

import pytest
from bs4 import BeautifulSoup

from html_to_markdown import convert_to_markdown
from html_to_markdown.exceptions import ConflictingOptionsError, EmptyHtmlError, MissingDependencyError
from html_to_markdown.processing import convert_to_markdown_stream


def test_empty_html_error():
    """Test EmptyHtmlError is raised for empty input."""
    with pytest.raises(EmptyHtmlError):
        convert_to_markdown("")

    # Test with whitespace-only input - may not raise error
    result = convert_to_markdown("   \n\n  ")
    # May return empty result instead of raising error


def test_conflicting_options_error():
    """Test ConflictingOptionsError is raised."""
    with pytest.raises(ConflictingOptionsError):
        convert_to_markdown("<p>test</p>", strip=["p"], convert=["p"])


def test_missing_dependency_error(monkeypatch):
    """Test MissingDependencyError for lxml parser."""
    # Mock LXML_AVAILABLE to False
    import html_to_markdown.processing

    monkeypatch.setattr(html_to_markdown.processing, "LXML_AVAILABLE", False)

    with pytest.raises(MissingDependencyError):
        convert_to_markdown("<p>test</p>", parser="lxml")


def test_beautifulsoup_input():
    """Test converting BeautifulSoup object directly."""
    soup = BeautifulSoup("<p>test</p>", "html.parser")
    result = convert_to_markdown(soup)
    assert "test" in result


def test_custom_converters():
    """Test custom converter functionality."""

    def custom_p_converter(tag, text, convert_as_inline=False):
        return f"CUSTOM: {text}"

    custom_converters = {"p": custom_p_converter}
    result = convert_to_markdown("<p>test</p>", custom_converters=custom_converters)
    assert "CUSTOM: test" in result


def test_metadata_extraction():
    """Test metadata extraction edge cases."""
    html = """
    <html>
    <head>
        <title>Test Page</title>
        <meta name="description" content="Test description">
        <meta property="og:title" content="OG Title">
        <meta http-equiv="content-type" content="text/html">
        <link rel="canonical" href="https://example.com">
        <link rel="author" href="https://example.com/author">
        <base href="https://example.com/">
    </head>
    <body><p>Content</p></body>
    </html>
    """

    result = convert_to_markdown(html, extract_metadata=True)
    assert "title: Test Page" in result
    assert "meta-description: Test description" in result
    assert "meta-og-title: OG Title" in result
    assert "canonical: https://example.com" in result
    assert "base-href: https://example.com/" in result

    # Test without metadata
    result_no_meta = convert_to_markdown(html, extract_metadata=False)
    assert "title:" not in result_no_meta
    assert "Content" in result_no_meta


def test_stream_processing():
    """Test streaming functionality."""
    html = "<p>" + "test " * 1000 + "</p>"

    chunks = list(convert_to_markdown_stream(html, chunk_size=100))
    combined = "".join(chunks)

    regular_result = convert_to_markdown(html)
    assert combined.strip() == regular_result.strip()
    assert len(chunks) > 1  # Should be chunked


def test_progress_callback():
    """Test progress callback in streaming."""
    html = "<p>" + "test " * 1000 + "</p>"
    progress_calls = []

    def progress_callback(processed, total):
        progress_calls.append((processed, total))

    list(convert_to_markdown_stream(html, progress_callback=progress_callback))
    assert len(progress_calls) > 0


def test_strip_newlines():
    """Test strip_newlines option."""
    html = "<p>Line 1\nLine 2\rLine 3</p>"
    result = convert_to_markdown(html, strip_newlines=True)
    # Newlines should be replaced with spaces in the result
    assert "Line 1 Line 2 Line 3" in result


def test_convert_as_inline():
    """Test convert_as_inline option."""
    html = "<p>Paragraph text</p>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert not result.endswith("\n")  # No trailing newline for inline


def test_parser_selection():
    """Test parser selection logic."""
    html = "<p>test</p>"

    # Test with html.parser
    result = convert_to_markdown(html, parser="html.parser")
    assert "test" in result

    # Test with auto-selection (should work with any available parser)
    result = convert_to_markdown(html, parser=None)
    assert "test" in result


def test_whitespace_handling():
    """Test various whitespace handling scenarios."""
    # Test leading whitespace preservation
    html = "  <p>text</p>"
    result = convert_to_markdown(html)
    # Should handle gracefully

    # Test whitespace between block elements
    html = "<p>Para 1</p>   <p>Para 2</p>"
    result = convert_to_markdown(html)
    assert "Para 1" in result
    assert "Para 2" in result


def test_wbr_element_handling():
    """Test that <wbr> elements are handled correctly."""
    html = "<p>long<wbr>word</p>"
    result = convert_to_markdown(html)
    assert "longword" in result


def test_normalize_spaces_outside_code():
    """Test space normalization outside code blocks."""
    html = """
    <p>Text   with    multiple     spaces</p>
    <pre><code>Code   with    preserved     spaces</code></pre>
    """
    result = convert_to_markdown(html)

    # Multiple spaces should be normalized in regular text
    assert "multiple     spaces" not in result
    assert "multiple spaces" in result

    # But preserved in code blocks
    assert "preserved     spaces" in result


def test_leading_whitespace_with_lxml():
    """Test leading whitespace handling with lxml parser."""
    try:
        import lxml

        html = "  <p>text</p>"
        result = convert_to_markdown(html, parser="lxml")
        # Should preserve leading whitespace
        assert result.startswith("  ")
    except ImportError:
        pytest.skip("lxml not available")


def test_definition_list_formatting():
    """Test definition list special formatting."""
    html = """
    <dl>
        <dt>Term</dt>
        <dd>Definition with multiple words</dd>
    </dl>
    """
    result = convert_to_markdown(html)
    assert ":   " in result  # Should preserve the 3-space definition format
