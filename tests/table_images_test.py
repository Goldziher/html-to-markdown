from __future__ import annotations

from html_to_markdown import convert_to_markdown


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


def test_original_issue_example() -> None:
    html = """<table class="normal">
        <tr>
            <td><img class="icon" src="../../icons/back16.gif">&nbsp;Go one page back</td>
            <td>&Opens the last page</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)
    assert "![](../../icons/back16.gif)" in result
    assert "Go one page back" in result
    assert "Opens the last page" in result


def test_keep_inline_images_in_with_tables() -> None:
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
