from html_to_markdown import convert_to_markdown


class TestTableCaption:
    """Test table caption element conversion."""

    def test_caption_basic(self) -> None:
        """Test basic caption conversion."""
        html = "<table><caption>Table Caption</caption><tr><td>Data</td></tr></table>"
        result = convert_to_markdown(html)
        assert "*Table Caption*" in result
        assert "| Data |" in result

    def test_caption_empty(self) -> None:
        html = "<table><caption></caption><tr><td>Data</td></tr></table>"
        result = convert_to_markdown(html)
        assert "*" not in result
        assert "| Data |" in result

    def test_caption_with_formatting(self) -> None:
        html = "<table><caption>Sales <strong>Report</strong> 2023</caption><tr><td>Data</td></tr></table>"
        result = convert_to_markdown(html)
        assert "*Sales **Report** 2023*" in result

    def test_caption_inline_mode(self) -> None:
        html = "<caption>Inline Caption</caption>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline Caption"


class TestColgroup:
    def test_colgroup_basic(self) -> None:
        html = "<table><colgroup><col><col></colgroup><tr><td>A</td><td>B</td></tr></table>"
        result = convert_to_markdown(html)
        assert "<colgroup>" not in result
        assert "<col />" not in result
        assert "</colgroup>" not in result
        assert "| A | B |" in result

    def test_colgroup_with_span(self) -> None:
        html = '<table><colgroup span="3"><col><col></colgroup><tr><td>A</td><td>B</td></tr></table>'
        result = convert_to_markdown(html)
        assert '<colgroup span="3">' not in result
        assert "| A | B |" in result

    def test_colgroup_empty(self) -> None:
        html = "<table><colgroup></colgroup><tr><td>Data</td></tr></table>"
        result = convert_to_markdown(html)
        assert "<colgroup>" not in result

    def test_colgroup_inline_mode(self) -> None:
        html = "<colgroup><col><col></colgroup>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ""


class TestCol:
    def test_col_basic(self) -> None:
        html = "<table><colgroup><col></colgroup><tr><td>Data</td></tr></table>"
        result = convert_to_markdown(html)
        assert "<col />" not in result
        assert "| Data |" in result

    def test_col_with_width(self) -> None:
        html = '<table><colgroup><col width="50%"></colgroup><tr><td>Data</td></tr></table>'
        result = convert_to_markdown(html)
        assert '<col width="50%" />' not in result
        assert "| Data |" in result

    def test_col_with_style(self) -> None:
        html = '<table><colgroup><col style="background-color: yellow;"></colgroup><tr><td>Data</td></tr></table>'
        result = convert_to_markdown(html)
        assert '<col style="background-color: yellow;" />' not in result
        assert "| Data |" in result

    def test_col_with_span(self) -> None:
        html = '<table><colgroup><col span="2"></colgroup><tr><td>A</td><td>B</td></tr></table>'
        result = convert_to_markdown(html)
        assert '<col span="2" />' not in result
        assert "| A | B |" in result

    def test_col_with_multiple_attributes(self) -> None:
        html = '<table><colgroup><col span="2" width="30%" style="color: red;"></colgroup><tr><td>A</td><td>B</td></tr></table>'
        result = convert_to_markdown(html)
        assert '<col width="30%" style="color: red;" span="2" />' not in result
        assert "| A | B |" in result

    def test_col_inline_mode(self) -> None:
        html = '<col width="50%">'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ""


class TestTableSections:
    def test_thead_basic(self) -> None:
        html = "<table><thead><tr><th>Header</th></tr></thead><tbody><tr><td>Data</td></tr></tbody></table>"
        result = convert_to_markdown(html)
        assert "| Header |" in result
        assert "| Data |" in result

    def test_tbody_basic(self) -> None:
        html = "<table><tbody><tr><td>Data</td></tr></tbody></table>"
        result = convert_to_markdown(html)
        assert "| Data |" in result

    def test_tfoot_basic(self) -> None:
        html = "<table><tfoot><tr><td>Footer</td></tr></tfoot><tbody><tr><td>Data</td></tr></tbody></table>"
        result = convert_to_markdown(html)
        assert "| Footer |" in result
        assert "| Data |" in result

    def test_all_table_sections(self) -> None:
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

    def test_table_sections_inline_mode(self) -> None:
        html = "<thead><tr><th>Header</th></tr></thead>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert "Header" in result


class TestComplexTable:
    def test_complete_table_structure(self) -> None:
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

    def test_table_with_colgroup_and_mixed_cols(self) -> None:
        html = """<table>
            <colgroup>
                <col>
                <col width="50%">
                <col style="background: yellow;" span="2">
            </colgroup>
            <tr>
                <td>A</td>
                <td>B</td>
                <td>C</td>
                <td>D</td>
            </tr>
        </table>"""
        result = convert_to_markdown(html)
        assert "<col />" not in result
        assert '<col width="50%" />' not in result
        assert '<col style="background: yellow;" span="2" />' not in result
        assert "| A | B | C | D |" in result

    def test_nested_colgroups(self) -> None:
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
        assert '<col style="background: red;" />' not in result
        assert '<col style="background: blue;" />' not in result
        assert '<col style="background: green;" />' not in result
        assert "| Red | Blue | Green |" in result

    def test_table_with_caption_and_formatting(self) -> None:
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

    def test_empty_table_elements(self) -> None:
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


class TestTableCompatibility:
    def test_simple_table_still_works(self) -> None:
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

    def test_table_with_colspan_still_works(self) -> None:
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

    def test_mixed_table_elements(self) -> None:
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
