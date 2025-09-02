
import importlib.util
from typing import Any

import pytest
from bs4 import BeautifulSoup

import html_to_markdown.processing
from html_to_markdown import convert_to_markdown
from html_to_markdown.exceptions import ConflictingOptionsError, EmptyHtmlError, MissingDependencyError
from html_to_markdown.processing import convert_to_markdown_stream


def test_empty_html_error() -> None:
    with pytest.raises(EmptyHtmlError):
        convert_to_markdown("")

    convert_to_markdown("   \n\n  ")


def test_conflicting_options_error() -> None:
    with pytest.raises(ConflictingOptionsError):
        convert_to_markdown("<p>test</p>", strip=["p"], convert=["p"])


def test_missing_dependency_error(monkeypatch: Any) -> None:

    monkeypatch.setattr(html_to_markdown.processing, "LXML_AVAILABLE", False)

    with pytest.raises(MissingDependencyError):
        convert_to_markdown("<p>test</p>", parser="lxml")


def test_beautifulsoup_input() -> None:
    soup = BeautifulSoup("<p>test</p>", "html.parser")
    result = convert_to_markdown(soup)
    assert "test" in result


def test_custom_converters() -> None:


def test_metadata_extraction() -> None:
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

    result_no_meta = convert_to_markdown(html, extract_metadata=False)
    assert "title:" not in result_no_meta
    assert "Content" in result_no_meta


def test_stream_processing() -> None:
    html = "<p>" + "test " * 1000 + "</p>"

    chunks = list(convert_to_markdown_stream(html, chunk_size=100))
    combined = "".join(chunks)

    regular_result = convert_to_markdown(html)
    assert combined.strip() == regular_result.strip()
    assert len(chunks) > 1


def test_progress_callback() -> None:
    html = "<p>" + "test " * 1000 + "</p>"
    progress_calls = []

    def progress_callback(processed: int, total: int) -> None:
        progress_calls.append((processed, total))

    list(convert_to_markdown_stream(html, progress_callback=progress_callback))
    assert len(progress_calls) > 0


def test_strip_newlines() -> None:
    html = "<p>Line 1\nLine 2\rLine 3</p>"
    result = convert_to_markdown(html, strip_newlines=True)
    assert "Line 1 Line 2 Line 3" in result


def test_convert_as_inline() -> None:
    html = "<p>Paragraph text</p>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert not result.endswith("\n")


def test_parser_selection() -> None:
    html = "<p>test</p>"

    result = convert_to_markdown(html, parser="html.parser")
    assert "test" in result

    result = convert_to_markdown(html, parser=None)
    assert "test" in result


def test_whitespace_handling() -> None:
    html = "  <p>text</p>"
    result = convert_to_markdown(html)

    html = "<p>Para 1</p>   <p>Para 2</p>"
    result = convert_to_markdown(html)
    assert "Para 1" in result
    assert "Para 2" in result


def test_wbr_element_handling() -> None:
    html = "<p>long<wbr>word</p>"
    result = convert_to_markdown(html)
    assert "longword" in result


def test_normalize_spaces_outside_code() -> None:
    html = """
    <p>Text   with    multiple     spaces</p>
    <pre><code>Code   with    preserved     spaces</code></pre>
    """
    result = convert_to_markdown(html)

    assert "multiple     spaces" not in result
    assert "multiple spaces" in result

    assert "preserved     spaces" in result


def test_leading_whitespace_with_lxml() -> None:
    try:
        if importlib.util.find_spec("lxml") is not None:
            html = "  <p>text</p>"
            result = convert_to_markdown(html, parser="lxml")
            assert result.startswith("  ")
        else:
            pytest.skip("lxml not available")
    except ImportError:
        pytest.skip("lxml not available")


def test_definition_list_formatting() -> None:
    html = """
    <dl>
        <dt>Term</dt>
        <dd>Definition with multiple words</dd>
    </dl>
    """
    result = convert_to_markdown(html)
    assert ":   " in result
