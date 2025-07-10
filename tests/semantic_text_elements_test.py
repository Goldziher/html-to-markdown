"""Tests for semantic text elements functionality."""

from html_to_markdown import convert_to_markdown


class TestAbbreviations:
    """Test abbreviation (abbr) element conversion."""

    def test_abbr_basic(self) -> None:
        """Test basic abbreviation conversion."""
        html = "<abbr>HTML</abbr>"
        result = convert_to_markdown(html)
        assert result == "HTML"

    def test_abbr_with_title(self) -> None:
        """Test abbreviation with title attribute."""
        html = '<abbr title="HyperText Markup Language">HTML</abbr>'
        result = convert_to_markdown(html)
        assert result == "HTML (HyperText Markup Language)"

    def test_abbr_with_empty_title(self) -> None:
        """Test abbreviation with empty title."""
        html = '<abbr title="">HTML</abbr>'
        result = convert_to_markdown(html)
        assert result == "HTML"

    def test_abbr_inline_mode(self) -> None:
        """Test abbreviation in inline mode."""
        html = '<abbr title="HyperText Markup Language">HTML</abbr>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "HTML (HyperText Markup Language)"

    def test_abbr_nested_content(self) -> None:
        """Test abbreviation with nested content."""
        html = '<p>Learn <abbr title="HyperText Markup Language">HTML</abbr> today!</p>'
        result = convert_to_markdown(html)
        assert result == "Learn HTML (HyperText Markup Language) today!\n\n"


class TestTimeElements:
    """Test time element conversion."""

    def test_time_basic(self) -> None:
        """Test basic time conversion."""
        html = "<time>2023-12-25</time>"
        result = convert_to_markdown(html)
        assert result == "2023\\-12\\-25"

    def test_time_with_datetime(self) -> None:
        """Test time with datetime attribute."""
        html = '<time datetime="2023-12-25T10:30:00">Christmas Day</time>'
        result = convert_to_markdown(html)
        assert result == '<time datetime="2023-12-25T10:30:00">Christmas Day</time>'

    def test_time_with_empty_datetime(self) -> None:
        """Test time with empty datetime."""
        html = '<time datetime="">December 25</time>'
        result = convert_to_markdown(html)
        assert result == "December 25"

    def test_time_inline_mode(self) -> None:
        """Test time in inline mode."""
        html = '<time datetime="2023-12-25">Christmas</time>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == '<time datetime="2023-12-25">Christmas</time>'

    def test_time_in_paragraph(self) -> None:
        """Test time element within paragraph."""
        html = '<p>The event is on <time datetime="2023-12-25">Christmas Day</time>.</p>'
        result = convert_to_markdown(html)
        assert result == 'The event is on <time datetime="2023-12-25">Christmas Day</time>.\n\n'


class TestDataElements:
    """Test data element conversion."""

    def test_data_basic(self) -> None:
        """Test basic data conversion."""
        html = "<data>Product Name</data>"
        result = convert_to_markdown(html)
        assert result == "Product Name"

    def test_data_with_value(self) -> None:
        """Test data with value attribute."""
        html = '<data value="12345">Product Name</data>'
        result = convert_to_markdown(html)
        assert result == '<data value="12345">Product Name</data>'

    def test_data_with_empty_value(self) -> None:
        """Test data with empty value."""
        html = '<data value="">Product</data>'
        result = convert_to_markdown(html)
        assert result == "Product"

    def test_data_inline_mode(self) -> None:
        """Test data in inline mode."""
        html = '<data value="12345">Product</data>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == '<data value="12345">Product</data>'

    def test_data_in_list(self) -> None:
        """Test data element within list."""
        html = '<ul><li><data value="A001">Product A</data></li><li><data value="B002">Product B</data></li></ul>'
        result = convert_to_markdown(html)
        assert result == '* <data value="A001">Product A</data>\n* <data value="B002">Product B</data>\n'


class TestInsertedText:
    """Test ins (inserted text) element conversion."""

    def test_ins_basic(self) -> None:
        """Test basic inserted text conversion."""
        html = "<ins>This text was added</ins>"
        result = convert_to_markdown(html)
        assert result == "==This text was added=="

    def test_ins_with_cite(self) -> None:
        """Test inserted text with cite attribute."""
        html = '<ins cite="https://example.com">Added text</ins>'
        result = convert_to_markdown(html)
        assert result == "==Added text=="

    def test_ins_with_datetime(self) -> None:
        """Test inserted text with datetime attribute."""
        html = '<ins datetime="2023-12-25">Added on Christmas</ins>'
        result = convert_to_markdown(html)
        assert result == "==Added on Christmas=="

    def test_ins_inline_mode(self) -> None:
        """Test inserted text in inline mode."""
        html = "<ins>Added text</ins>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "==Added text=="

    def test_ins_in_paragraph(self) -> None:
        """Test inserted text within paragraph."""
        html = "<p>Original text <ins>with addition</ins> and more.</p>"
        result = convert_to_markdown(html)
        assert result == "Original text ==with addition== and more.\n\n"


class TestVariableText:
    """Test var (variable) element conversion."""

    def test_var_basic(self) -> None:
        """Test basic variable conversion."""
        html = "<var>x</var>"
        result = convert_to_markdown(html)
        assert result == "*x*"

    def test_var_in_code(self) -> None:
        """Test variable in code context."""
        html = "<p>Set <var>username</var> to your login name.</p>"
        result = convert_to_markdown(html)
        assert result == "Set *username* to your login name.\n\n"

    def test_var_mathematical(self) -> None:
        """Test variable in mathematical context."""
        html = "<p>If <var>x</var> = 5, then <var>y</var> = <var>x</var> + 3.</p>"
        result = convert_to_markdown(html)
        assert result == "If *x* \\= 5, then *y* \\= *x* \\+ 3\\.\n\n"

    def test_var_inline_mode(self) -> None:
        """Test variable in inline mode."""
        html = "<var>variable</var>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "*variable*"


class TestDefinitionText:
    """Test dfn (definition) element conversion."""

    def test_dfn_basic(self) -> None:
        """Test basic definition conversion."""
        html = "<dfn>API</dfn>"
        result = convert_to_markdown(html)
        assert result == "*API*"

    def test_dfn_with_title(self) -> None:
        """Test definition with title attribute."""
        html = '<dfn title="Application Programming Interface">API</dfn>'
        result = convert_to_markdown(html)
        assert result == "*API*"

    def test_dfn_in_definition_list(self) -> None:
        """Test definition within definition list."""
        html = "<dl><dt><dfn>API</dfn></dt><dd>Application Programming Interface</dd></dl>"
        result = convert_to_markdown(html)
        assert result == "*API*\n:   Application Programming Interface\n\n"

    def test_dfn_inline_mode(self) -> None:
        """Test definition in inline mode."""
        html = "<dfn>term</dfn>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "*term*"


class TestBidirectionalText:
    """Test bdi and bdo (bidirectional text) element conversion."""

    def test_bdi_basic(self) -> None:
        """Test basic bidirectional isolation."""
        html = "<bdi>عربي</bdi>"
        result = convert_to_markdown(html)
        assert result == "عربي"

    def test_bdo_basic(self) -> None:
        """Test basic bidirectional override."""
        html = '<bdo dir="rtl">English text</bdo>'
        result = convert_to_markdown(html)
        assert result == "English text"

    def test_bdi_mixed_text(self) -> None:
        """Test bidirectional isolation with mixed text."""
        html = "<p>User <bdi>إيان</bdi> scored 90 points.</p>"
        result = convert_to_markdown(html)
        assert result == "User إيان scored 90 points.\n\n"

    def test_bdo_with_direction(self) -> None:
        """Test bidirectional override with direction."""
        html = '<p>The title is <bdo dir="rtl">مرحبا</bdo> in Arabic.</p>'
        result = convert_to_markdown(html)
        assert result == "The title is مرحبا in Arabic.\n\n"

    def test_bdi_inline_mode(self) -> None:
        """Test bidirectional isolation in inline mode."""
        html = "<bdi>نص عربي</bdi>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "نص عربي"


class TestSmallText:
    """Test small element conversion."""

    def test_small_basic(self) -> None:
        """Test basic small text conversion."""
        html = "<small>Fine print</small>"
        result = convert_to_markdown(html)
        assert result == "Fine print"

    def test_small_copyright(self) -> None:
        """Test small text for copyright."""
        html = "<p>© 2023 Company Name. <small>All rights reserved.</small></p>"
        result = convert_to_markdown(html)
        assert result == "© 2023 Company Name. All rights reserved.\n\n"

    def test_small_inline_mode(self) -> None:
        """Test small text in inline mode."""
        html = "<small>Legal disclaimer</small>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Legal disclaimer"


class TestUnderlinedText:
    """Test u (underlined) element conversion."""

    def test_u_basic(self) -> None:
        """Test basic underlined text conversion."""
        html = "<u>Underlined text</u>"
        result = convert_to_markdown(html)
        assert result == "Underlined text"

    def test_u_misspelling(self) -> None:
        """Test underlined text for misspelling indication."""
        html = "<p>This word is <u>mispelled</u>.</p>"
        result = convert_to_markdown(html)
        assert result == "This word is mispelled.\n\n"

    def test_u_inline_mode(self) -> None:
        """Test underlined text in inline mode."""
        html = "<u>underlined</u>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "underlined"


class TestWordBreakOpportunity:
    """Test wbr (word break opportunity) element conversion."""

    def test_wbr_basic(self) -> None:
        """Test basic word break opportunity conversion."""
        html = "super<wbr>cali<wbr>fragilistic"
        result = convert_to_markdown(html)
        assert result == "supercalifragilistic"

    def test_wbr_long_url(self) -> None:
        """Test word break opportunity in long URL."""
        html = "<p>Visit https://www.<wbr>example.<wbr>com/very/<wbr>long/<wbr>path</p>"
        result = convert_to_markdown(html)
        assert result == "Visit https://www.example.com/very/long/path\n\n"

    def test_wbr_inline_mode(self) -> None:
        """Test word break opportunity in inline mode."""
        html = "long<wbr>word"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "longword"


class TestSemanticTextMixedContent:
    """Test semantic text elements with mixed content."""

    def test_mixed_semantic_elements(self) -> None:
        """Test document with multiple semantic text elements."""
        html = """<article>
            <h2>Programming Concepts</h2>
            <p>An <dfn>API</dfn> (<abbr title="Application Programming Interface">API</abbr>)
            allows different software components to communicate.</p>
            <p>When you set the variable <var>timeout</var> to <data value="5000">5 seconds</data>,
            <ins>added in version 2.0</ins>, the system will wait.</p>
            <p><small>Last updated: <time datetime="2023-12-25">December 25, 2023</time></small></p>
        </article>"""
        result = convert_to_markdown(html, heading_style="atx")
        expected = """
## Programming Concepts

An *API* (API (Application Programming Interface))
 allows different software components to communicate.

When you set the variable *timeout* to <data value="5000">5 seconds</data>,
 ==added in version 2\\.0==, the system will wait.

Last updated: <time datetime="2023-12-25">December 25, 2023</time>

"""
        assert result == expected

    def test_nested_semantic_elements(self) -> None:
        """Test nested semantic text elements."""
        html = '<p>The <dfn><abbr title="Application Programming Interface">API</abbr></dfn> documentation has been <ins>updated with <var>new_parameter</var></ins>.</p>'
        result = convert_to_markdown(html)
        assert (
            result
            == "The *API (Application Programming Interface)* documentation has been ==updated with *new\\_parameter*==.\n\n"
        )

    def test_semantic_elements_inline_mode(self) -> None:
        """Test semantic text elements in inline mode."""
        html = '<abbr title="HyperText Markup Language">HTML</abbr> and <var>css</var> with <ins>updates</ins>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "HTML (HyperText Markup Language) and *css* with ==updates=="

    def test_empty_semantic_elements(self) -> None:
        """Test empty semantic text elements."""
        html = "<p>Empty elements: <abbr></abbr> <var></var> <ins></ins> <dfn></dfn></p>"
        result = convert_to_markdown(html)
        assert result == "Empty elements:    \n\n"

    def test_whitespace_handling(self) -> None:
        """Test whitespace handling in semantic elements."""
        html = "<p>Spaces around <var>  variable  </var> and <abbr title='  title  '>  abbr  </abbr></p>"
        result = convert_to_markdown(html)
        assert result == "Spaces around  *variable*  and abbr (title)\n\n"
