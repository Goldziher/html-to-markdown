"""Tests for Ruby text elements (ruby, rb, rt, rp, rtc)."""

from html_to_markdown import convert_to_markdown


class TestRubyElements:
    """Test Ruby text annotation elements."""

    def test_ruby_basic(self) -> None:
        """Test basic ruby element conversion."""
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_rb(self) -> None:
        """Test ruby with rb element."""
        html = "<ruby><rb>漢字</rb><rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_fallback_rp(self) -> None:
        """Test ruby with fallback parentheses."""
        html = "<ruby>漢字<rp>(</rp><rt>kanji</rt><rp>)</rp></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_complex_structure(self) -> None:
        """Test complex ruby structure with multiple elements."""
        html = "<ruby><rb>東京</rb><rp>(</rp><rt>とうきょう</rt><rp>)</rp></ruby>"
        result = convert_to_markdown(html)
        assert result == "東京(とうきょう)"

    def test_ruby_multiple_readings(self) -> None:
        """Test ruby with multiple readings."""
        html = "<ruby><rb>漢</rb><rt>kan</rt><rb>字</rb><rt>ji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢(kan)字(ji)"

    def test_ruby_inline_mode(self) -> None:
        """Test ruby in inline mode."""
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "漢字(kanji)"

    def test_ruby_block_mode(self) -> None:
        """Test ruby in block mode."""
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "漢字(kanji)"

    def test_ruby_nested_in_paragraph(self) -> None:
        """Test ruby nested in paragraph."""
        html = "<p>This is <ruby>漢字<rt>kanji</rt></ruby> text.</p>"
        result = convert_to_markdown(html)
        assert result == "This is 漢字(kanji) text.\n\n"

    def test_ruby_with_whitespace(self) -> None:
        """Test ruby with whitespace handling."""
        html = "<ruby> 漢字 <rt> kanji </rt> </ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字 (kanji)"

    def test_ruby_empty_elements(self) -> None:
        """Test ruby with empty elements."""
        html = "<ruby><rb></rb><rt></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "()"

    def test_ruby_only_base_text(self) -> None:
        """Test ruby with only base text."""
        html = "<ruby>漢字</ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字"

    def test_ruby_only_annotation(self) -> None:
        """Test ruby with only annotation."""
        html = "<ruby><rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_ruby_with_formatting(self) -> None:
        """Test ruby with inline formatting."""
        html = "<ruby><strong>漢字</strong><rt><em>kanji</em></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "**漢字**(*kanji*)"

    def test_ruby_multiple_in_sentence(self) -> None:
        """Test multiple ruby elements in one sentence."""
        html = "I love <ruby>寿司<rt>sushi</rt></ruby> and <ruby>刺身<rt>sashimi</rt></ruby>!"
        result = convert_to_markdown(html)
        assert result == "I love 寿司(sushi) and 刺身(sashimi)!"

    def test_ruby_with_mixed_content(self) -> None:
        """Test ruby with mixed text and HTML content."""
        html = "<ruby>東<rb>京</rb>都<rt>とう<strong>きょう</strong>と</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "東京都(とう**きょう**と)"


class TestRubySubElements:
    """Test individual Ruby sub-elements."""

    def test_rb_standalone(self) -> None:
        """Test rb element standalone."""
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html)
        assert result == "漢字"

    def test_rb_inline_mode(self) -> None:
        """Test rb in inline mode."""
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "漢字"

    def test_rb_block_mode(self) -> None:
        """Test rb in block mode."""
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "漢字"

    def test_rt_standalone(self) -> None:
        """Test rt element standalone."""
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_rt_with_surrounding_rp(self) -> None:
        """Test rt with surrounding rp elements."""
        html = "<rp>(</rp><rt>kanji</rt><rp>)</rp>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_rt_inline_mode(self) -> None:
        """Test rt in inline mode."""
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "(kanji)"

    def test_rt_block_mode(self) -> None:
        """Test rt in block mode."""
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "(kanji)"

    def test_rp_standalone(self) -> None:
        """Test rp element standalone."""
        html = "<rp>(</rp>"
        result = convert_to_markdown(html)
        assert result == "("

    def test_rp_inline_mode(self) -> None:
        """Test rp in inline mode."""
        html = "<rp>)</rp>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ")"

    def test_rp_block_mode(self) -> None:
        """Test rp in block mode."""
        html = "<rp>(</rp>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "("

    def test_rtc_standalone(self) -> None:
        """Test rtc element standalone."""
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html)
        assert result == "annotation"

    def test_rtc_inline_mode(self) -> None:
        """Test rtc in inline mode."""
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "annotation"

    def test_rtc_block_mode(self) -> None:
        """Test rtc in block mode."""
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "annotation"


class TestRubyEdgeCases:
    """Test edge cases and error conditions."""

    def test_nested_ruby_elements(self) -> None:
        """Test nested ruby elements."""
        html = "<ruby><ruby>漢<rt>kan</rt></ruby><rt>字</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢(kan)(字)"

    def test_ruby_with_line_breaks(self) -> None:
        """Test ruby with line breaks."""
        html = "<ruby>\n漢字\n<rt>\nkanji\n</rt>\n</ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_special_characters(self) -> None:
        """Test ruby with special Markdown characters."""
        html = "<ruby>*test*<rt>_annotation_</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "\\*test\\*(\\_annotation\\_)"

    def test_ruby_with_links(self) -> None:
        """Test ruby containing links."""
        html = '<ruby><a href="https://example.com">漢字</a><rt>kanji</rt></ruby>'
        result = convert_to_markdown(html)
        assert result == "[漢字](https://example.com)(kanji)"

    def test_ruby_in_table(self) -> None:
        """Test ruby in table cells."""
        html = "<table><tr><td><ruby>漢字<rt>kanji</rt></ruby></td></tr></table>"
        result = convert_to_markdown(html)
        assert "漢字(kanji)" in result

    def test_ruby_in_list(self) -> None:
        """Test ruby in list items."""
        html = "<ul><li><ruby>漢字<rt>kanji</rt></ruby></li></ul>"
        result = convert_to_markdown(html)
        assert "* 漢字(kanji)" in result

    def test_multiple_rt_elements(self) -> None:
        """Test ruby with multiple rt elements."""
        html = "<ruby>漢字<rt>kan</rt><rt>ji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kan)(ji)"

    def test_ruby_with_rtc_and_rt(self) -> None:
        """Test ruby with both rtc and rt elements."""
        html = "<ruby>漢字<rt>kanji</rt><rtc>Chinese characters</rtc></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)Chinese characters"

    def test_complex_ruby_structure(self) -> None:
        """Test complex ruby structure with all elements."""
        html = """<ruby>
            <rb>漢</rb>
            <rb>字</rb>
            <rp>(</rp>
            <rt>kan</rt>
            <rt>ji</rt>
            <rp>)</rp>
            <rtc>Chinese characters</rtc>
        </ruby>"""
        result = convert_to_markdown(html)
        assert result == "漢字((kan)(ji))Chinese characters"

    def test_ruby_with_empty_rt(self) -> None:
        """Test ruby with empty rt element."""
        html = "<ruby>漢字<rt></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字()"

    def test_ruby_with_only_spaces(self) -> None:
        """Test ruby with only spaces in elements."""
        html = "<ruby>   <rt>   </rt>   </ruby>"
        result = convert_to_markdown(html)
        assert result == "()"
