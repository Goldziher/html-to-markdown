from html_to_markdown import convert_to_markdown


class TestRubyElements:
    def test_ruby_basic(self) -> None:
        """Test basic ruby element conversion."""
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_rb(self) -> None:
        html = "<ruby><rb>漢字</rb><rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_fallback_rp(self) -> None:
        html = "<ruby>漢字<rp>(</rp><rt>kanji</rt><rp>)</rp></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_complex_structure(self) -> None:
        html = "<ruby><rb>東京</rb><rp>(</rp><rt>とうきょう</rt><rp>)</rp></ruby>"
        result = convert_to_markdown(html)
        assert result == "東京(とうきょう)"

    def test_ruby_multiple_readings(self) -> None:
        html = "<ruby><rb>漢</rb><rt>kan</rt><rb>字</rb><rt>ji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢(kan)字(ji)"

    def test_ruby_inline_mode(self) -> None:
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "漢字(kanji)"

    def test_ruby_block_mode(self) -> None:
        html = "<ruby>漢字<rt>kanji</rt></ruby>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "漢字(kanji)"

    def test_ruby_nested_in_paragraph(self) -> None:
        html = "<p>This is <ruby>漢字<rt>kanji</rt></ruby> text.</p>"
        result = convert_to_markdown(html)
        assert result == "This is 漢字(kanji) text.\n\n"

    def test_ruby_with_whitespace(self) -> None:
        html = "<ruby> 漢字 <rt> kanji </rt> </ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字 (kanji)"

    def test_ruby_empty_elements(self) -> None:
        html = "<ruby><rb></rb><rt></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "()"

    def test_ruby_only_base_text(self) -> None:
        html = "<ruby>漢字</ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字"

    def test_ruby_only_annotation(self) -> None:
        html = "<ruby><rt>kanji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_ruby_with_formatting(self) -> None:
        html = "<ruby><strong>漢字</strong><rt><em>kanji</em></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "**漢字**(*kanji*)"

    def test_ruby_multiple_in_sentence(self) -> None:
        html = "I love <ruby>寿司<rt>sushi</rt></ruby> and <ruby>刺身<rt>sashimi</rt></ruby>!"
        result = convert_to_markdown(html)
        assert result == "I love 寿司(sushi) and 刺身(sashimi)!"

    def test_ruby_with_mixed_content(self) -> None:
        html = "<ruby>東<rb>京</rb>都<rt>とう<strong>きょう</strong>と</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "東京都(とう**きょう**と)"


class TestRubySubElements:
    def test_rb_standalone(self) -> None:
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html)
        assert result == "漢字"

    def test_rb_inline_mode(self) -> None:
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "漢字"

    def test_rb_block_mode(self) -> None:
        html = "<rb>漢字</rb>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "漢字"

    def test_rt_standalone(self) -> None:
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_rt_with_surrounding_rp(self) -> None:
        html = "<rp>(</rp><rt>kanji</rt><rp>)</rp>"
        result = convert_to_markdown(html)
        assert result == "(kanji)"

    def test_rt_inline_mode(self) -> None:
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "(kanji)"

    def test_rt_block_mode(self) -> None:
        html = "<rt>kanji</rt>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "(kanji)"

    def test_rp_standalone(self) -> None:
        html = "<rp>(</rp>"
        result = convert_to_markdown(html)
        assert result == "("

    def test_rp_inline_mode(self) -> None:
        html = "<rp>)</rp>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == ")"

    def test_rp_block_mode(self) -> None:
        html = "<rp>(</rp>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "("

    def test_rtc_standalone(self) -> None:
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html)
        assert result == "annotation"

    def test_rtc_inline_mode(self) -> None:
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "annotation"

    def test_rtc_block_mode(self) -> None:
        html = "<rtc>annotation</rtc>"
        result = convert_to_markdown(html, convert_as_inline=False)
        assert result == "annotation"


class TestRubyEdgeCases:
    def test_nested_ruby_elements(self) -> None:
        html = "<ruby><ruby>漢<rt>kan</rt></ruby><rt>字</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢(kan)(字)"

    def test_ruby_with_line_breaks(self) -> None:
        html = "<ruby>\n漢字\n<rt>\nkanji\n</rt>\n</ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)"

    def test_ruby_with_special_characters(self) -> None:
        html = "<ruby>*test*<rt>_annotation_</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "\\*test\\*(\\_annotation\\_)"

    def test_ruby_with_links(self) -> None:
        html = '<ruby><a href="https://example.com">漢字</a><rt>kanji</rt></ruby>'
        result = convert_to_markdown(html)
        assert result == "[漢字](https://example.com)(kanji)"

    def test_ruby_in_table(self) -> None:
        html = "<table><tr><td><ruby>漢字<rt>kanji</rt></ruby></td></tr></table>"
        result = convert_to_markdown(html)
        assert "漢字(kanji)" in result

    def test_ruby_in_list(self) -> None:
        html = "<ul><li><ruby>漢字<rt>kanji</rt></ruby></li></ul>"
        result = convert_to_markdown(html)
        assert "* 漢字(kanji)" in result

    def test_multiple_rt_elements(self) -> None:
        html = "<ruby>漢字<rt>kan</rt><rt>ji</rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kan)(ji)"

    def test_ruby_with_rtc_and_rt(self) -> None:
        html = "<ruby>漢字<rt>kanji</rt><rtc>Chinese characters</rtc></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字(kanji)Chinese characters"

    def test_complex_ruby_structure(self) -> None:
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
        html = "<ruby>漢字<rt></rt></ruby>"
        result = convert_to_markdown(html)
        assert result == "漢字()"

    def test_ruby_with_only_spaces(self) -> None:
        html = "<ruby>   <rt>   </rt>   </ruby>"
        result = convert_to_markdown(html)
        assert result == "()"
