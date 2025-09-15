"""Comprehensive tests for HTML table to Markdown conversion.

Covers basic tables, table sections, captions, colgroups, rowspan/colspan,
and tables containing images and other elements.
"""

from __future__ import annotations

import pytest

from html_to_markdown import convert_to_markdown


def test_table_first_row_in_tbody_without_previous_sibling() -> None:
    html = """<table>
    <tbody>
        <tr><td>Cell 1</td><td>Cell 2</td></tr>
    </tbody>
    </table>"""
    result = convert_to_markdown(html)
    expected = "\n\n| Cell 1 | Cell 2 |\n| --- | --- |\n\n"
    assert result == expected


def test_basic_table() -> None:
    html = """<table>
    <tr><th>Header 1</th><th>Header 2</th></tr>
    <tr><td>Cell 1</td><td>Cell 2</td></tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "| Header 1 | Header 2 |" in result
    assert "| Cell 1 | Cell 2 |" in result


def test_simple_table_structure() -> None:
    html = """<table>
        <tr>
            <th>Header 1</th>
            <th>Header 2</th>
        </tr>
        <tr>
            <td>Data 1</td>
            <td>Data 2</td>
        </tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "| Header 1 | Header 2 |" in result
    assert "| --- | --- |" in result
    assert "| Data 1 | Data 2 |" in result


def test_table_with_sections() -> None:
    html = """<table>
        <thead>
            <tr><th>Name</th><th>Age</th></tr>
        </thead>
        <tbody>
            <tr><td>John</td><td>25</td></tr>
            <tr><td>Jane</td><td>30</td></tr>
        </tbody>
        <tfoot>
            <tr><td>Total</td><td>2</td></tr>
        </tfoot>
    </table>"""
    result = convert_to_markdown(html)
    assert "| Name | Age |" in result
    assert "| John | 25 |" in result
    assert "| Jane | 30 |" in result
    assert "| Total | 2 |" in result


def test_tbody_only() -> None:
    html = "<table><tbody><tr><td>Data</td></tr></tbody></table>"
    result = convert_to_markdown(html)
    assert "| Data |" in result


def test_tfoot_basic() -> None:
    html = "<table><tfoot><tr><td>Footer</td></tr></tfoot><tbody><tr><td>Data</td></tr></tbody></table>"
    result = convert_to_markdown(html)
    assert "| Footer |" in result
    assert "| Data |" in result


def test_table_caption() -> None:
    html = "<table><caption>Table Caption</caption><tr><td>Data</td></tr></table>"
    result = convert_to_markdown(html)
    assert "*Table Caption*" in result
    assert "| Data |" in result


def test_caption_with_formatting() -> None:
    html = "<table><caption>Sales <strong>Report</strong> 2023</caption><tr><td>Data</td></tr></table>"
    result = convert_to_markdown(html)
    assert "*Sales **Report** 2023*" in result


def test_caption_empty() -> None:
    html = "<table><caption></caption><tr><td>Data</td></tr></table>"
    result = convert_to_markdown(html)
    assert "*" not in result
    assert "| Data |" in result


def test_caption_inline_mode() -> None:
    html = "<caption>Inline Caption</caption>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == "Inline Caption"


def test_colgroup_removed() -> None:
    html = """<table>
    <colgroup>
        <col style="width: 50%">
        <col style="width: 50%">
    </colgroup>
    <thead>
        <tr>
            <th>Header 1</th>
            <th>Header 2</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Cell 1</td>
            <td>Cell 2</td>
        </tr>
    </tbody>
    </table>"""

    result = convert_to_markdown(html)

    assert "<colgroup>" not in result
    assert "<col>" not in result
    assert "colgroup" not in result.lower()
    assert "| Header 1 | Header 2 |" in result
    assert "| Cell 1 | Cell 2 |" in result


def test_col_elements_removed() -> None:
    html = """<table>
    <col width="100">
    <tr><td>Cell</td></tr>
    </table>"""

    result = convert_to_markdown(html)

    assert "<col" not in result
    assert "width=" not in result
    assert "| Cell |" in result


def test_colgroup_with_span() -> None:
    html = '<table><colgroup span="3"><col><col></colgroup><tr><td>A</td><td>B</td></tr></table>'
    result = convert_to_markdown(html)
    assert '<colgroup span="3">' not in result
    assert "| A | B |" in result


def test_col_with_attributes() -> None:
    html = '<table><colgroup><col width="50%" style="background: yellow;" span="2"></colgroup><tr><td>A</td><td>B</td></tr></table>'
    result = convert_to_markdown(html)
    assert 'width="50%"' not in result
    assert 'style="background: yellow;"' not in result
    assert 'span="2"' not in result
    assert "| A | B |" in result


def test_table_with_colspan() -> None:
    html = """<table>
        <tr>
            <th colspan="2">Merged Header</th>
        </tr>
        <tr>
            <td>Data 1</td>
            <td>Data 2</td>
        </tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "| Merged Header |" in result
    assert "| Data 1 | Data 2 |" in result


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


def test_complex_rowspan_case() -> None:
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


def test_image_in_table_cell() -> None:
    html = """<table>
        <tr>
            <td><img src="test.jpg" alt="Test Image">Cell with image</td>
            <td>Regular cell</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![Test Image](test.jpg)" in result
    assert "Cell with image" in result
    assert "Regular cell" in result


def test_image_with_title_in_table() -> None:
    html = """<table>
        <tr>
            <td><img src="icon.png" alt="Icon" title="An icon">Text</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert '![Icon](icon.png "An icon")' in result
    assert "Text" in result


def test_image_without_alt_in_table() -> None:
    html = """<table>
        <tr>
            <td><img src="image.gif">Content</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![](image.gif)" in result
    assert "Content" in result


def test_multiple_images_in_table_cell() -> None:
    html = """<table>
        <tr>
            <td><img src="img1.jpg" alt="First"> and <img src="img2.jpg" alt="Second"></td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![First](img1.jpg)" in result
    assert "![Second](img2.jpg)" in result
    assert "and" in result


def test_image_in_table_header() -> None:
    html = """<table>
        <tr>
            <th><img src="header.png" alt="Header Icon">Column</th>
        </tr>
        <tr>
            <td>Data</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![Header Icon](header.png)" in result
    assert "Column" in result
    assert "Data" in result


def test_image_with_dimensions_in_table() -> None:
    html = """<table>
        <tr>
            <td><img src="sized.jpg" alt="Sized" width="100" height="50">Text</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "<img src='sized.jpg' alt='Sized' title='' width='100' height='50' />" in result
    assert "Text" in result


def test_keep_inline_images_in_tables() -> None:
    html = """<table>
        <tr>
            <td><img src="table.jpg" alt="Table Image">In table</td>
        </tr>
    </table>
    <h1><img src="heading.jpg" alt="Heading Image">In heading</h1>"""

    result_default = convert_to_markdown(html)
    assert "![Table Image](table.jpg)" in result_default
    assert "Heading Image" in result_default
    assert "![Heading Image](heading.jpg)" not in result_default

    result_with_h1 = convert_to_markdown(html, keep_inline_images_in=["h1"])
    assert "![Table Image](table.jpg)" in result_with_h1
    assert "![Heading Image](heading.jpg)" in result_with_h1


def test_complex_table_with_images() -> None:
    html = """<table>
        <thead>
            <tr>
                <th><img src="col1.png" alt="Column 1">Actions</th>
                <th>Description</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td><img src="icon1.gif" alt="Back">Go back</td>
                <td>Navigate to previous page</td>
            </tr>
            <tr>
                <td><img src="icon2.gif" alt="Forward">Go forward</td>
                <td>Navigate to next page</td>
            </tr>
        </tbody>
    </table>"""

    result = convert_to_markdown(html)
    assert "![Column 1](col1.png)" in result
    assert "![Back](icon1.gif)" in result
    assert "![Forward](icon2.gif)" in result
    assert "Actions" in result
    assert "Go back" in result
    assert "Go forward" in result


def test_table_with_mixed_content() -> None:
    html = """<table>
        <tr>
            <td><img src="test.jpg" alt="Test"> <strong>Bold text</strong> and <em>italic</em></td>
            <td><code>code</code> with <img src="icon.png" alt="Icon"></td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![Test](test.jpg)" in result
    assert "**Bold text**" in result
    assert "*italic*" in result
    assert "`code`" in result
    assert "![Icon](icon.png)" in result


def test_complete_table_structure() -> None:
    html = """<table>
        <caption>Employee Database</caption>
        <colgroup>
            <col style="width: 40%">
            <col style="width: 30%">
            <col style="width: 30%">
        </colgroup>
        <thead>
            <tr>
                <th>Name</th>
                <th>Department</th>
                <th>Salary</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>John Doe</td>
                <td>Engineering</td>
                <td>$75,000</td>
            </tr>
            <tr>
                <td>Jane Smith</td>
                <td>Marketing</td>
                <td>$65,000</td>
            </tr>
        </tbody>
        <tfoot>
            <tr>
                <td>Total Employees</td>
                <td>2</td>
                <td>$140,000</td>
            </tr>
        </tfoot>
    </table>"""
    result = convert_to_markdown(html)

    assert "*Employee Database*" in result

    assert "<colgroup>" not in result
    assert '<col style="width: 40%" />' not in result
    assert '<col style="width: 30%" />' not in result

    assert "| Name | Department | Salary |" in result
    assert "| John Doe | Engineering | $75,000 |" in result
    assert "| Jane Smith | Marketing | $65,000 |" in result
    assert "| Total Employees | 2 | $140,000 |" in result


def test_nested_colgroups() -> None:
    html = """<table>
        <colgroup span="2">
            <col style="background: red;">
            <col style="background: blue;">
        </colgroup>
        <colgroup>
            <col style="background: green;">
        </colgroup>
        <tr>
            <td>Red</td>
            <td>Blue</td>
            <td>Green</td>
        </tr>
    </table>"""
    result = convert_to_markdown(html)
    assert '<colgroup span="2">' not in result
    assert 'style="background: red;"' not in result
    assert 'style="background: blue;"' not in result
    assert 'style="background: green;"' not in result
    assert "| Red | Blue | Green |" in result


def test_table_with_caption_and_formatting() -> None:
    html = """<table>
        <caption><strong>Q4 2023</strong> Sales Report - <em>Final</em></caption>
        <tr>
            <td>Product A</td>
            <td>$1,000</td>
        </tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "***Q4 2023** Sales Report \\- *Final**" in result
    assert "| Product A | $1,000 |" in result


def test_empty_table_elements() -> None:
    html = """<table>
        <caption></caption>
        <colgroup></colgroup>
        <thead></thead>
        <tbody>
            <tr>
                <td>Only Data</td>
            </tr>
        </tbody>
        <tfoot></tfoot>
    </table>"""
    result = convert_to_markdown(html)

    assert "*" not in result.split("Only Data")[0]
    assert "<colgroup>" not in result
    assert "| Only Data |" in result


def test_mixed_table_elements() -> None:
    html = """<table>
        <caption>Mixed Table</caption>
        <tr>
            <th>Header</th>
        </tr>
        <tbody>
            <tr>
                <td>Body Data</td>
            </tr>
        </tbody>
    </table>"""
    result = convert_to_markdown(html)
    assert "*Mixed Table*" in result
    assert "| Header |" in result
    assert "| Body Data |" in result


def test_table_sections_inline_mode() -> None:
    html = "<thead><tr><th>Header</th></tr></thead>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert "Header" in result


def test_colgroup_inline_mode() -> None:
    html = "<colgroup><col><col></colgroup>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == ""


def test_col_inline_mode() -> None:
    html = '<col width="50%">'
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == ""


@pytest.mark.parametrize(
    "html,should_contain",
    [
        ("<table><tr><th>H</th></tr><tr><td>D</td></tr></table>", ["| H |", "| D |"]),
        ("<table><caption>Cap</caption><tr><td>Data</td></tr></table>", ["*Cap*", "| Data |"]),
        ('<table><colgroup><col width="50%"></colgroup><tr><td>Cell</td></tr></table>', ["| Cell |"]),
        ('<table><col style="color: red;"><tr><td>Cell</td></tr></table>', ["| Cell |"]),
        ("<table><thead><tr><th>H</th></tr></thead><tbody><tr><td>D</td></tr></tbody></table>", ["| H |", "| D |"]),
        ("<table><tbody><tr><td>Body</td></tr></tbody></table>", ["| Body |"]),
        ('<table><tr><td><img src="test.jpg" alt="Test">Text</td></tr></table>', ["![Test](test.jpg)", "Text"]),
    ],
)
def test_table_conversion_patterns(html: str, should_contain: list[str]) -> None:
    result = convert_to_markdown(html)
    for expected in should_contain:
        assert expected in result


@pytest.mark.parametrize(
    "html,should_not_contain",
    [
        ("<table><colgroup><col></colgroup><tr><td>Data</td></tr></table>", ["<colgroup>", "<col", "</colgroup>"]),
        ('<table><col width="100"><tr><td>Data</td></tr></table>', ["<col", "width="]),
        ("<table><caption></caption><tr><td>Data</td></tr></table>", ["*"]),
    ],
)
def test_table_element_removal(html: str, should_not_contain: list[str]) -> None:
    result = convert_to_markdown(html)
    for unwanted in should_not_contain:
        assert unwanted not in result


def test_table_with_tbody_but_no_thead() -> None:
    html = """
    <table>
        <tbody>
            <tr><td>Cell 1</td><td>Cell 2</td></tr>
            <tr><td>Cell 3</td><td>Cell 4</td></tr>
        </tbody>
    </table>
    """
    result = convert_to_markdown(html)
    assert "| Cell 1 | Cell 2 |" in result
    assert "| Cell 3 | Cell 4 |" in result


def test_table_first_row_directly_in_table() -> None:
    html = """<table>
        <tr><td>Cell1</td><td>Cell2</td></tr>
        <tr><td>Cell3</td><td>Cell4</td></tr>
    </table>"""
    result = convert_to_markdown(html)
    assert "| Cell1 | Cell2 |" in result
    assert "| --- | --- |" in result
    assert "| Cell3 | Cell4 |" in result


def test_tbody_inline_mode() -> None:
    html = "<tbody><tr><td>Cell</td></tr></tbody>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert "Cell" in result


def test_tfoot_inline_mode() -> None:
    html = "<tfoot><tr><td>Footer</td></tr></tfoot>"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert "Footer" in result


@pytest.mark.parametrize(
    "html,expected",
    [
        (
            """<table>
<tr><th>Header 1</th><th>Header 2</th></tr>
<tr><td>Cell 3</td><td>
    <div>Cell 4-1</div>
    <div>Cell 4-2</div>
</td></tr>
</table>""",
            "\n\n| Header 1 | Header 2 |\n| --- | --- |\n| Cell 3 | Cell 4-1<br>Cell 4-2 |\n\n",
        ),
        (
            """<table>
<tr><td><p>Paragraph 1</p><p>Paragraph 2</p></td></tr>
</table>""",
            "\n\n| Paragraph 1<br>Paragraph 2 |\n| --- |\n\n",
        ),
        (
            """<table>
<tr><th>
    <div>Header 1</div>
    <div>Sub-header</div>
</th><th>Header 2</th></tr>
<tr><td>Cell 1</td><td>Cell 2</td></tr>
</table>""",
            "\n\n| Header 1<br>Sub-header | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |\n\n",
        ),
        (
            """<table>
<tr><td>
    <div>Text content</div>
    <ul><li>List item</li></ul>
    <div>More text</div>
</td></tr>
</table>""",
            "\n\n| Text content<br>List item<br>More text |\n| --- |\n\n",
        ),
        (
            """<table>
<tr>
<td>
    <div>Cell 1-1</div>
    <div>Cell 1-2</div>
</td>
<td>
    <div>Cell 2-1</div>
    <div>Cell 2-2</div>
</td>
</tr>
</table>""",
            "\n\n| Cell 1-1<br>Cell 1-2 | Cell 2-1<br>Cell 2-2 |\n| --- | --- |\n\n",
        ),
        (
            """<table>
<tr><th>Header 1</th><th>Header 2</th></tr>
<tr><td rowspan="2">Spanning cell</td><td>
    <div>First row content</div>
    <div>Second line</div>
</td></tr>
<tr><td>
    <div>Next row</div>
    <div>More content</div>
</td></tr>
</table>""",
            "\n\n| Header 1 | Header 2 |\n| --- | --- |\n| Spanning cell | First row content<br>Second line |\n| | Next row<br>More content |\n\n",
        ),
        (
            """<table>
<tr><td>
    <div><p>Nested paragraph 1</p></div>
    <div><p>Nested paragraph 2</p></div>
</td></tr>
</table>""",
            "\n\n| Nested paragraph 1<br>Nested paragraph 2 |\n| --- |\n\n",
        ),
        (
            """<table>
<tr><td>
    <h3>Title</h3>
    <p>Content paragraph</p>
</td></tr>
</table>""",
            "\n\n| Title<br>Content paragraph |\n| --- |\n\n",
        ),
    ],
)
def test_table_cell_multiline_content_issues(html: str, expected: str) -> None:
    result = convert_to_markdown(html, br_in_tables=True)
    assert result == expected
