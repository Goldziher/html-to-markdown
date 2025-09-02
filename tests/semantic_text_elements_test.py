from html_to_markdown import convert_to_markdown


class TestAbbreviations:
    """Test abbreviation (abbr) element conversion."""

    def test_abbr_basic(self) -> None:
        """Test basic abbreviation conversion."""
        html = "<abbr>HTML</abbr>"
        result = convert_to_markdown(html)
        assert result == "HTML"

    def test_abbr_with_title(self) -> None:
        html = '<abbr title="HyperText Markup Language">HTML</abbr>'
        result = convert_to_markdown(html)
        assert result == "HTML (HyperText Markup Language)"

    def test_abbr_with_empty_title(self) -> None:
        html = '<abbr title="">HTML</abbr>'
        result = convert_to_markdown(html)
        assert result == "HTML"

    def test_abbr_inline_mode(self) -> None:
        html = '<abbr title="HyperText Markup Language">HTML</abbr>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "HTML (HyperText Markup Language)"

    def test_abbr_nested_content(self) -> None:
        html = '<p>Learn <abbr title="HyperText Markup Language">HTML</abbr> today!</p>'
        result = convert_to_markdown(html)
        assert result == "Learn HTML (HyperText Markup Language) today!\n\n"


class TestTimeElements:
    def test_time_basic(self) -> None:
        html = "<time>2023-12-25</time>"
        result = convert_to_markdown(html)
        assert result == "2023\\-12\\-25"

    def test_time_with_datetime(self) -> None:
        html = '<time datetime="2023-12-25T10:30:00">Christmas Day</time>'
        result = convert_to_markdown(html)
        assert result == "Christmas Day"

    def test_time_with_empty_datetime(self) -> None:
        html = '<time datetime="">December 25</time>'
        result = convert_to_markdown(html)
        assert result == "December 25"

    def test_time_inline_mode(self) -> None:
        html = '<time datetime="2023-12-25">Christmas</time>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Christmas"

    def test_time_in_paragraph(self) -> None:
        html = '<p>The event is on <time datetime="2023-12-25">Christmas Day</time>.</p>'
        result = convert_to_markdown(html)
        assert result == "The event is on Christmas Day.\n\n"


class TestDataElements:
    def test_data_basic(self) -> None:
        html = "<data>Product Name</data>"
        result = convert_to_markdown(html)
        assert result == "Product Name"

    def test_data_with_value(self) -> None:
        html = '<data value="12345">Product Name</data>'
        result = convert_to_markdown(html)
        assert result == "Product Name"

    def test_data_with_empty_value(self) -> None:
        html = '<data value="">Product</data>'
        result = convert_to_markdown(html)
        assert result == "Product"

    def test_data_inline_mode(self) -> None:
        html = '<data value="12345">Product</data>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Product"

    def test_data_in_list(self) -> None:
        html = '<ul><li><data value="A001">Product A</data></li><li><data value="B002">Product B</data></li></ul>'
        result = convert_to_markdown(html)
        assert result == "* Product A\n* Product B\n"


class TestInsertedText:
    def test_ins_basic(self) -> None:
        html = "<ins>This text was added</ins>"
        result = convert_to_markdown(html)
        assert result == "==This text was added=="

    def test_ins_with_cite(self) -> None:
        html = '<ins cite="https://example.com">Added text</ins>'
        result = convert_to_markdown(html)
        assert result == "==Added text=="

    def test_ins_with_datetime(self) -> None:
        html = '<ins datetime="2023-12-25">Added on Christmas</ins>'
        result = convert_to_markdown(html)
        assert result == "==Added on Christmas=="

    def test_ins_inline_mode(self) -> None:
        html = "<ins>Added text</ins>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "==Added text=="

    def test_ins_in_paragraph(self) -> None:
        html = "<p>Original text <ins>with addition</ins> and more.</p>"
        result = convert_to_markdown(html)
        assert result == "Original text ==with addition== and more.\n\n"


class TestVariableText:
    def test_var_basic(self) -> None:
        html = "<var>x</var>"
        result = convert_to_markdown(html)
        assert result == "*x*"

    def test_var_in_code(self) -> None:
        html = "<p>Set <var>username</var> to your login name.</p>"
        result = convert_to_markdown(html)
        assert result == "Set *username* to your login name.\n\n"

    def test_var_mathematical(self) -> None:
        html = "<p>If <var>x</var> = 5, then <var>y</var> = <var>x</var> + 3.</p>"
        result = convert_to_markdown(html)
        assert result == "If *x* \\= 5, then *y* \\= *x* \\+ 3\\.\n\n"

    def test_var_inline_mode(self) -> None:
        html = "<var>variable</var>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "*variable*"


class TestDefinitionText:
    def test_dfn_basic(self) -> None:
        html = "<dfn>API</dfn>"
        result = convert_to_markdown(html)
        assert result == "*API*"

    def test_dfn_with_title(self) -> None:
        html = '<dfn title="Application Programming Interface">API</dfn>'
        result = convert_to_markdown(html)
        assert result == "*API*"

    def test_dfn_in_definition_list(self) -> None:
        html = "<dl><dt><dfn>API</dfn></dt><dd>Application Programming Interface</dd></dl>"
        result = convert_to_markdown(html)
        assert result == "*API*\n:   Application Programming Interface\n\n"

    def test_dfn_inline_mode(self) -> None:
        html = "<dfn>term</dfn>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "*term*"


class TestBidirectionalText:
    def test_bdi_basic(self) -> None:
        html = "<bdi>عربي</bdi>"
        result = convert_to_markdown(html)
        assert result == "عربي"

    def test_bdo_basic(self) -> None:
        html = '<bdo dir="rtl">English text</bdo>'
        result = convert_to_markdown(html)
        assert result == "English text"

    def test_bdi_mixed_text(self) -> None:
        html = "<p>User <bdi>إيان</bdi> scored 90 points.</p>"
        result = convert_to_markdown(html)
        assert result == "User إيان scored 90 points.\n\n"

    def test_bdo_with_direction(self) -> None:
        html = '<p>The title is <bdo dir="rtl">مرحبا</bdo> in Arabic.</p>'
        result = convert_to_markdown(html)
        assert result == "The title is مرحبا in Arabic.\n\n"

    def test_bdi_inline_mode(self) -> None:
        html = "<bdi>نص عربي</bdi>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "نص عربي"


class TestSmallText:
    def test_small_basic(self) -> None:
        html = "<small>Fine print</small>"
        result = convert_to_markdown(html)
        assert result == "Fine print"

    def test_small_copyright(self) -> None:
        html = "<p>© 2023 Company Name. <small>All rights reserved.</small></p>"
        result = convert_to_markdown(html)
        assert result == "© 2023 Company Name. All rights reserved.\n\n"

    def test_small_inline_mode(self) -> None:
        html = "<small>Legal disclaimer</small>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Legal disclaimer"


class TestUnderlinedText:
    def test_u_basic(self) -> None:
        html = "<u>Underlined text</u>"
        result = convert_to_markdown(html)
        assert result == "Underlined text"

    def test_u_misspelling(self) -> None:
        html = "<p>This word is <u>mispelled</u>.</p>"
        result = convert_to_markdown(html)
        assert result == "This word is mispelled.\n\n"

    def test_u_inline_mode(self) -> None:
        html = "<u>underlined</u>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "underlined"


class TestWordBreakOpportunity:
    def test_wbr_basic(self) -> None:
        html = "super<wbr>cali<wbr>fragilistic"
        result = convert_to_markdown(html)
        assert result == "supercalifragilistic"

    def test_wbr_long_url(self) -> None:
        html = "<p>Visit https://www.<wbr>example.<wbr>com/very/<wbr>long/<wbr>path</p>"
        result = convert_to_markdown(html)
        assert result == "Visit https://www.example.com/very/long/path\n\n"

    def test_wbr_inline_mode(self) -> None:
        html = "long<wbr>word"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "longword"


class TestSemanticTextMixedContent:
    def test_mixed_semantic_elements(self) -> None:
        html = """<article>
            <h2>Programming Concepts</h2>
            <p>An <dfn>API</dfn> (<abbr title="Application Programming Interface">API</abbr>)
            allows different software components to communicate.</p>
            <p>When you set the variable <var>timeout</var> to <data value="5000">5 seconds</data>,
            <ins>added in version 2.0</ins>, the system will wait.</p>
            <p><small>Last updated: <time datetime="2023-12-25">December 25, 2023</time></small></p>
        </article>"""
        result = convert_to_markdown(html, heading_style="atx")
        expected = """## Programming Concepts

An *API* (API (Application Programming Interface))
 allows different software components to communicate.

When you set the variable *timeout* to 5 seconds, ==added in version 2\\.0==, the system will wait.

Last updated: December 25, 2023

"""
        assert result == expected

    def test_nested_semantic_elements(self) -> None:
        html = '<p>The <dfn><abbr title="Application Programming Interface">API</abbr></dfn> documentation has been <ins>updated with <var>new_parameter</var></ins>.</p>'
        result = convert_to_markdown(html)
        assert (
            result
            == "The *API (Application Programming Interface)* documentation has been ==updated with *new\\_parameter*==.\n\n"
        )

    def test_semantic_elements_inline_mode(self) -> None:
        html = '<abbr title="HyperText Markup Language">HTML</abbr> and <var>css</var> with <ins>updates</ins>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "HTML (HyperText Markup Language) and *css* with ==updates=="

    def test_empty_semantic_elements(self) -> None:
        html = "<p>Empty elements: <abbr></abbr> <var></var> <ins></ins> <dfn></dfn></p>"
        result = convert_to_markdown(html)
        assert result == "Empty elements: \n\n"

    def test_whitespace_handling(self) -> None:
        html = "<p>Spaces around <var>  variable  </var> and <abbr title='  title  '>  abbr  </abbr></p>"
        result = convert_to_markdown(html)
        assert result == "Spaces around  *variable*  and abbr (title)\n\n"
