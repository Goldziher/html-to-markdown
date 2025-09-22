from __future__ import annotations

import importlib.util
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable

import pytest
from bs4 import BeautifulSoup

import html_to_markdown.processing
from html_to_markdown import convert_to_markdown
from html_to_markdown.exceptions import ConflictingOptionsError, EmptyHtmlError, MissingDependencyError
from html_to_markdown.processing import _as_optional_set, _get_list_indent, convert_to_markdown_stream


def test_get_list_indent_tabs(convert: Callable[[str, ...], str]) -> None:
    result = _get_list_indent("tabs", 4)
    assert result == "\t"


def test_get_list_indent_spaces(convert: Callable[[str, ...], str]) -> None:
    result = _get_list_indent("spaces", 2)
    assert result == "  "


def test_convert_as_inline_strips_trailing_newlines(convert: Callable[[str, ...], str]) -> None:
    html = "<p>Test content\n</p>"
    result = convert(html, convert_as_inline=True)
    assert not result.endswith("\n")
    assert result == "Test content"


def test_empty_html_error(convert: Callable[[str, ...], str]) -> None:
    with pytest.raises(EmptyHtmlError):
        convert("")

    convert("   \n\n  ")


def test_conflicting_options_error(convert: Callable[[str, ...], str]) -> None:
    with pytest.raises(ConflictingOptionsError):
        convert("<p>test</p>", strip=["p"], convert=["p"])


def test_missing_dependency_error(monkeypatch: Any) -> None:
    monkeypatch.setattr(html_to_markdown.processing, "LXML_AVAILABLE", False)

    with pytest.raises(MissingDependencyError):
        convert_to_markdown("<p>test</p>", parser="lxml")


def test_beautifulsoup_input(convert: Callable[[str, ...], str]) -> None:
    soup = BeautifulSoup("<p>test</p>", "html.parser")
    result = convert(soup)
    assert "test" in result


def test_metadata_extraction(convert: Callable[[str, ...], str]) -> None:
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

    result = convert(html, extract_metadata=True)
    assert "title: Test Page" in result
    assert "meta-description: Test description" in result
    assert "meta-og-title: OG Title" in result
    assert "canonical: https://example.com" in result
    assert "base-href: https://example.com/" in result

    result_no_meta = convert(html, extract_metadata=False)
    assert "title:" not in result_no_meta
    assert "Content" in result_no_meta


def test_stream_processing(convert: Callable[[str, ...], str]) -> None:
    html = "<p>" + "test " * 1000 + "</p>"

    chunks = list(convert_to_markdown_stream(html, chunk_size=100))
    combined = "".join(chunks)

    regular_result = convert(html)
    assert combined.strip() == regular_result.strip()
    assert len(chunks) > 1


def test_progress_callback(convert: Callable[[str, ...], str]) -> None:
    html = "<p>" + "test " * 1000 + "</p>"
    progress_calls = []

    def progress_callback(processed: int, total: int) -> None:
        progress_calls.append((processed, total))

    list(convert_to_markdown_stream(html, progress_callback=progress_callback))
    assert len(progress_calls) > 0


def test_strip_newlines(convert: Callable[[str, ...], str]) -> None:
    html = "<p>Line 1\nLine 2\rLine 3</p>"
    result = convert(html, strip_newlines=True)
    assert "Line 1 Line 2 Line 3" in result


def test_convert_as_inline(convert: Callable[[str, ...], str]) -> None:
    html = "<p>Paragraph text</p>"
    result = convert(html, convert_as_inline=True)
    assert not result.endswith("\n")


def test_parser_selection() -> None:
    html = "<p>test</p>"

    result = convert_to_markdown(html, parser="html.parser")
    assert "test" in result

    result = convert_to_markdown(html, parser=None)
    assert "test" in result


def test_whitespace_handling(convert: Callable[[str, ...], str]) -> None:
    html = "  <p>text</p>"
    result = convert(html)

    html = "<p>Para 1</p>   <p>Para 2</p>"
    result = convert(html)
    assert "Para 1" in result
    assert "Para 2" in result


def test_wbr_element_handling(convert: Callable[[str, ...], str]) -> None:
    html = "<p>long<wbr>word</p>"
    result = convert(html)
    assert "longword" in result


def test_normalize_spaces_outside_code(convert: Callable[[str, ...], str]) -> None:
    html = """
    <p>Text   with    multiple     spaces</p>
    <pre><code>Code   with    preserved     spaces</code></pre>
    """
    result = convert(html)

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


def test_definition_list_formatting(convert: Callable[[str, ...], str]) -> None:
    html = """
    <dl>
        <dt>Term</dt>
        <dd>Definition with multiple words</dd>
    </dl>
    """
    result = convert(html)
    assert ":   " in result


def test_as_optional_set_function(convert: Callable[[str, ...], str]) -> None:
    assert _as_optional_set(None) is None

    result = _as_optional_set("a,b,c")
    expected = {"a", "b", "c"}
    assert result == expected, f"String splitting failed: expected {expected}, got {result}"

    result = _as_optional_set("single")
    assert result == {"single"}

    result = _as_optional_set("")
    assert result == {""}

    result = _as_optional_set(["a,b", "c,d"])
    expected = {"a", "b", "c", "d"}
    assert result == expected


def test_underlined_heading_conversion(convert: Callable[[str, ...], str]) -> None:
    html = "<h2>Header</h2><p>Next paragraph</p>"
    result = convert(html, heading_style="underlined")

    assert "Header" in result
    assert "------" in result
    assert "Next paragraph" in result

    expected = "Header\n------\n\nNext paragraph\n\n"
    assert result == expected


def test_bytes_input_handling() -> None:
    """Test that bytes input is properly handled - reproduces issue #73"""
    html = b"<html><head><title>Test Title</title></head><body><p>Test content</p></body></html>"

    result = convert_to_markdown(html)
    assert "Test content" in result

    result = convert_to_markdown(html, extract_metadata=True)
    assert "Test content" in result

    complex_html = b"""<!DOCTYPE html>
<html>
<head>
    <meta name="viewport" content="width=device-width" />
    <title>COD FISCAL (A) 08/09/2015</title>
</head>
<body>
    <span class="S_DEN">CODUL FISCAL din 8 septembrie 2015</span>
    <p>Test content for issue #73</p>
</body>
</html>"""

    result = convert_to_markdown(complex_html, extract_metadata=True)
    assert "CODUL FISCAL" in result
    assert "Test content for issue" in result
    assert "73" in result


def test_bytes_input_with_encodings() -> None:
    """Test that bytes input handles different encodings correctly"""
    html_utf8 = "<p>Café naïve résumé 日本語</p>".encode()
    result = convert_to_markdown(html_utf8)
    assert "Café naïve résumé 日本語" in result

    html_latin1 = "<p>Café naïve résumé</p>".encode("latin-1")
    result = convert_to_markdown(html_latin1, source_encoding="latin-1")
    assert "Café naïve résumé" in result

    html_win1252 = '<p>Smart quotes: "Hello"</p>'.encode("windows-1252")
    result = convert_to_markdown(html_win1252, source_encoding="windows-1252")
    assert "Smart quotes" in result

    html_iso = "<p>Español: ñ, Português: ção</p>".encode("iso-8859-1")
    result = convert_to_markdown(html_iso, source_encoding="iso-8859-1")
    assert "Español" in result
    assert "Português" in result

    html_invalid = b"<p>Invalid \xff\xfe bytes</p>"
    result = convert_to_markdown(html_invalid)
    assert "Invalid" in result
    assert "bytes" in result

    html_stream = "<p>Streaming with encoding: café</p>".encode("latin-1")
    chunks = list(convert_to_markdown_stream(html_stream, source_encoding="latin-1", chunk_size=10))
    combined = "".join(chunks)
    assert "café" in combined
