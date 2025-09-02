from html_to_markdown import convert_to_markdown


def test_colgroup_removed_from_output() -> None:
    html = """<table class="wrapped confluenceTable">
        <colgroup>
            <col />
            <col />
            <col />
            <col />
        </colgroup>
        <tbody>
            <tr>
                <th class="confluenceTh" scope="col">Header 1</th>
                <th class="confluenceTh" scope="col">Header 2</th>
                <th class="confluenceTh" scope="col">Header 3</th>
                <th class="confluenceTh" scope="col">Header 4</th>
            </tr>
            <tr>
                <td class="confluenceTd">Data 1</td>
                <td class="confluenceTd">Data 2</td>
                <td class="confluenceTd">Data 3</td>
                <td class="confluenceTd">Data 4</td>
            </tr>
        </tbody>
    </table>"""

    result = convert_to_markdown(html)

    assert "<colgroup>" not in result
    assert "</colgroup>" not in result
    assert "<col" not in result
    assert "<col />" not in result

    assert "| Header 1 | Header 2 | Header 3 | Header 4 |" in result
    assert "| --- | --- | --- | --- |" in result
    assert "| Data 1 | Data 2 | Data 3 | Data 4 |" in result


def test_colgroup_with_attributes_removed() -> None:
    html = """<table>
        <colgroup span="2">
            <col style="background-color: red;" />
            <col width="50%" />
        </colgroup>
        <tr>
            <td>Cell 1</td>
            <td>Cell 2</td>
        </tr>
    </table>"""

    result = convert_to_markdown(html)

    assert "colgroup" not in result.lower()
    assert "col" not in result.lower()
    assert "background-color" not in result
    assert "width" not in result
    assert "50%" not in result

    assert "| Cell 1 | Cell 2 |" in result
