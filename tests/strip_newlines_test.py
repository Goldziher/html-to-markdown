from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def test_strip_newlines_basic(convert: Callable[[str, ...], str]) -> None:
    html = """<p>Return a list of the words in the string, using <em>sep</em> as the delimiter
string.  If <em>maxsplit</em> is given, at most <em>maxsplit</em> splits are done (thus,
the list will have at most <code class="docutils literal notranslate"><span class="pre">maxsplit+1</span></code> elements).  If <em>maxsplit</em> is not
specified or <code class="docutils literal notranslate"><span class="pre">-1</span></code>, then there is no limit on the number of splits
(all possible splits are made).</p>"""

    result_default = convert(html, wrap=False)
    assert "\n" in result_default

    result_stripped = convert(html, strip_newlines=True, wrap=False)
    assert "\n\n" in result_stripped
    assert result_stripped.count("\n") == 2

    assert "Return a list of the words in the string" in result_stripped


def test_strip_newlines_with_carriage_returns(convert: Callable[[str, ...], str]) -> None:
    html_with_cr = "Text with\r\nnewlines and\rcarriage returns"
    result = convert(html_with_cr, strip_newlines=True)
    assert "Text with newlines and carriage returns" in result


def test_strip_newlines_with_multiple_paragraphs(convert: Callable[[str, ...], str]) -> None:
    html = """<p>First paragraph
with a line break.</p>
<p>Second paragraph
also with a line break.</p>"""

    result = convert(html, strip_newlines=True)

    assert "First paragraph with a line break." in result
    assert "Second paragraph also with a line break." in result

    assert "\n\n" in result


def test_strip_newlines_preserves_pre_blocks(convert: Callable[[str, ...], str]) -> None:
    html = """<p>Regular text
with newline.</p>
<pre>Code block
with preserved
newlines</pre>"""

    result = convert(html, strip_newlines=True)
    assert "Regular text with newline." in result

    assert "Code block with preserved newlines" in result


def test_strip_newlines_with_inline_elements(convert: Callable[[str, ...], str]) -> None:
    html = """<p>This is <strong>bold
text</strong> and <em>italic
text</em> with line breaks.</p>"""

    result = convert(html, strip_newlines=True)
    assert result == "This is **bold text** and *italic text* with line breaks.\n\n"


def test_strip_newlines_empty_html(convert: Callable[[str, ...], str]) -> None:
    html = "\n\n"

    result = convert(html, strip_newlines=True)
    assert result.strip() == ""


def test_strip_newlines_preserves_br_tags(convert: Callable[[str, ...], str]) -> None:
    html = "<p>Line one<br>Line two</p>"

    result = convert(html, strip_newlines=True)
    assert result == "Line one  \nLine two\n\n"


def test_strip_newlines_with_lists(convert: Callable[[str, ...], str]) -> None:
    html = """<ul>
<li>Item one
with newline</li>
<li>Item two
also with newline</li>
</ul>"""

    result = convert(html, strip_newlines=True)
    assert "* Item one with newline\n" in result
    assert "* Item two also with newline\n" in result


def test_strip_newlines_complex_html(convert: Callable[[str, ...], str]) -> None:
    html = """<div>
    <h1>Title with
    newline</h1>
    <p>Paragraph with
    multiple
    newlines.</p>
    <blockquote>Quote with
    newline.</blockquote>
</div>"""

    result = convert(html, strip_newlines=True)
    assert "Title with newline" in result
    assert "Paragraph with multiple newlines." in result
    assert "> Quote with newline." in result


def test_strip_newlines_with_only_carriage_returns(convert: Callable[[str, ...], str]) -> None:
    html = "Text\rwith\rcarriage\rreturns"
    result = convert(html, strip_newlines=True)
    assert "Text with carriage returns" in result
