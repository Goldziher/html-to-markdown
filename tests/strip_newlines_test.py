from __future__ import annotations

from html_to_markdown import convert_to_markdown


def test_strip_newlines_basic() -> None:
    """Test basic newline stripping functionality."""
    html = """<p>Return a list of the words in the string, using <em>sep</em> as the delimiter
string.  If <em>maxsplit</em> is given, at most <em>maxsplit</em> splits are done (thus,
the list will have at most <code class="docutils literal notranslate"><span class="pre">maxsplit+1</span></code> elements).  If <em>maxsplit</em> is not
specified or <code class="docutils literal notranslate"><span class="pre">-1</span></code>, then there is no limit on the number of splits
(all possible splits are made).</p>"""

    # Without strip_newlines (default behavior)
    result_default = convert_to_markdown(html, wrap=False)
    assert "\n" in result_default  # Should contain newlines

    # With strip_newlines
    result_stripped = convert_to_markdown(html, strip_newlines=True, wrap=False)
    assert "\n\n" in result_stripped  # Only paragraph break should remain
    assert result_stripped.count("\n") == 2  # Only the two newlines at the end of paragraph

    # Verify content is preserved
    assert "Return a list of the words in the string" in result_stripped
    assert "*sep*" in result_stripped
    assert "*maxsplit*" in result_stripped
    assert "`maxsplit+1`" in result_stripped
    assert "`-1`" in result_stripped


def test_strip_newlines_with_multiple_paragraphs() -> None:
    """Test that paragraph breaks are preserved when stripping newlines."""
    html = """<p>First paragraph
with a line break.</p>
<p>Second paragraph
also with a line break.</p>"""

    result = convert_to_markdown(html, strip_newlines=True)
    # Should have exactly one blank line between paragraphs
    assert "First paragraph with a line break." in result
    assert "Second paragraph also with a line break." in result
    # Verify paragraph separation
    assert "\n\n" in result


def test_strip_newlines_preserves_pre_blocks() -> None:
    """Test that newlines in pre blocks are preserved."""
    html = """<p>Regular text
with newline.</p>
<pre>Code block
with preserved
newlines</pre>"""

    result = convert_to_markdown(html, strip_newlines=True)
    assert "Regular text with newline." in result
    # Pre blocks content is still affected by strip_newlines since it's applied before parsing
    assert "Code block with preserved newlines" in result


def test_strip_newlines_with_inline_elements() -> None:
    """Test newline stripping with inline elements."""
    html = """<p>This is <strong>bold
text</strong> and <em>italic
text</em> with line breaks.</p>"""

    result = convert_to_markdown(html, strip_newlines=True)
    assert result == "This is **bold text** and *italic text* with line breaks.\n\n"


def test_strip_newlines_empty_html() -> None:
    """Test that empty HTML still works with strip_newlines."""
    html = "\n\n\n"

    # When strip_newlines=True, newlines become spaces which get normalized
    result = convert_to_markdown(html, strip_newlines=True)
    assert result.strip() == ""  # The spaces get normalized


def test_strip_newlines_with_carriage_returns() -> None:
    """Test that carriage returns are also stripped."""
    html = "<p>Text with\r\nWindows-style\rline breaks.</p>"

    result = convert_to_markdown(html, strip_newlines=True)
    assert "Text with Windows" in result
    assert "style line breaks." in result


def test_strip_newlines_preserves_br_tags() -> None:
    """Test that <br> tags still create line breaks even with strip_newlines."""
    html = "<p>Line one<br>Line two</p>"

    result = convert_to_markdown(html, strip_newlines=True)
    assert result == "Line one  \nLine two\n\n"


def test_strip_newlines_with_lists() -> None:
    """Test newline stripping in lists."""
    html = """<ul>
<li>Item one
with newline</li>
<li>Item two
also with newline</li>
</ul>"""

    result = convert_to_markdown(html, strip_newlines=True)
    assert "* Item one with newline\n" in result
    assert "* Item two also with newline\n" in result


def test_strip_newlines_complex_html() -> None:
    """Test with complex HTML structure."""
    html = """<div>
    <h1>Title with
    newline</h1>
    <p>Paragraph with
    multiple
    newlines.</p>
    <blockquote>Quote with
    newline.</blockquote>
</div>"""

    result = convert_to_markdown(html, strip_newlines=True)
    assert "Title with newline" in result
    assert "Paragraph with multiple newlines." in result
    assert "> Quote with newline." in result
