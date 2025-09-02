"""Test for issue #55 - Links in rowspan-affected table cells not converting."""

from html_to_markdown import convert_to_markdown


def test_links_in_rowspan_cells() -> None:
    html = """<table>
    <tr>
        <td rowspan="2">Cell A</td>
        <td><a href="https://example.com">Link B</a></td>
    </tr>
    <tr>
        <td><a href="https://example.com">Link C</a></td>
    </tr>
    </table>"""

    result = convert_to_markdown(html)

    assert "[Link B](https://example.com)" in result
    assert "[Link C](https://example.com)" in result
    assert "Link C |" not in result or "[Link C]" in result


def test_complex_table_with_rowspan_and_links() -> None:
    html = """<table>
    <tr>
        <th>Header 1</th>
        <th>Header 2</th>
    </tr>
    <tr>
        <td rowspan="2">Spanning Cell</td>
        <td><a href="https://test.com">First Link</a></td>
    </tr>
    <tr>
        <td><a href="https://test.com">Second Link</a></td>
    </tr>
    <tr>
        <td rowspan="2">Another Span</td>
        <td><p><a href="https://test.com">Third Link</a></p></td>
    </tr>
    <tr>
        <td><p><a href="https://test.com">Fourth Link</a></p></td>
    </tr>
    </table>"""

    result = convert_to_markdown(html)

    assert "[First Link](https://test.com)" in result
    assert "[Second Link](https://test.com)" in result
    assert "[Third Link](https://test.com)" in result
    assert "[Fourth Link](https://test.com)" in result


def test_issue_55_exact_case() -> None:
    html = """<table>
    <tbody>
    <tr>
        <td rowspan="2"><p>EDA</p></td>
        <td><p><a href="https://www.temp.com" target="_blank">EDB</a></p></td>
    </tr>
    <tr>
        <td><p><a href="https://www.temp.com" target="_blank">EDC</a></p><p><a href="https://www.temp.com" target="_blank">EDD</a></p></td>
    </tr>
    </tbody>
    </table>"""

    result = convert_to_markdown(html)

    assert "[EDB](https://www.temp.com)" in result
    assert "[EDC](https://www.temp.com)" in result
    assert "[EDD](https://www.temp.com)" in result

    assert "EDCEDD" not in result or ("[EDC]" in result and "[EDD]" in result)


def test_multiple_rowspan_levels() -> None:
    html = """<table>
    <tr>
        <td rowspan="3">A</td>
        <td><a href="https://example.com">B</a></td>
    </tr>
    <tr>
        <td><a href="https://example.com">C</a></td>
    </tr>
    <tr>
        <td><a href="https://example.com">D</a></td>
    </tr>
    </table>"""

    result = convert_to_markdown(html)

    assert "[B](https://example.com)" in result
    assert "[C](https://example.com)" in result
    assert "[D](https://example.com)" in result
