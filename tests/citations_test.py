from html_to_markdown import convert_to_markdown


class TestCitations:
    def test_cite_element(self) -> None:
        """Test cite element conversion to italic."""
        html = "<cite>Author Name</cite>"
        result = convert_to_markdown(html)
        assert result == "*Author Name*"

    def test_cite_with_whitespace(self) -> None:
        html = "<cite>  Author Name  </cite>"
        result = convert_to_markdown(html)
        assert result == "*Author Name*"

    def test_cite_inline_mode(self) -> None:
        html = "<cite>Author Name</cite>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Author Name"

    def test_empty_cite(self) -> None:
        html = "<cite></cite>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_cite_with_nested_elements(self) -> None:
        html = "<cite>Author <strong>Name</strong></cite>"
        result = convert_to_markdown(html)
        assert result == "*Author **Name***"

    def test_cite_with_link(self) -> None:
        html = '<cite><a href="https://example.com">Author Name</a></cite>'
        result = convert_to_markdown(html)
        assert result == "*[Author Name](https://example.com)*"


class TestQuotations:
    def test_q_element(self) -> None:
        html = "<q>Short quotation</q>"
        result = convert_to_markdown(html)
        assert result == '"Short quotation"'

    def test_q_with_whitespace(self) -> None:
        html = "<q>  Short quotation  </q>"
        result = convert_to_markdown(html)
        assert result == '"Short quotation"'

    def test_q_inline_mode(self) -> None:
        html = "<q>Short quotation</q>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Short quotation"

    def test_empty_q(self) -> None:
        html = "<q></q>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_q_with_existing_quotes(self) -> None:
        html = '<q>He said "Hello" to me</q>'
        result = convert_to_markdown(html)
        assert result == '"He said \\"Hello\\" to me"'

    def test_q_with_nested_elements(self) -> None:
        html = "<q>A <em>short</em> quotation</q>"
        result = convert_to_markdown(html)
        assert result == '"A *short* quotation"'

    def test_q_with_code(self) -> None:
        html = "<q>The function <code>print()</code> outputs text</q>"
        result = convert_to_markdown(html)
        assert result == '"The function `print()` outputs text"'

    def test_nested_q_elements(self) -> None:
        html = "<q>Outer quote <q>inner quote</q> continues</q>"
        result = convert_to_markdown(html)
        assert result == '"Outer quote \\"inner quote\\" continues"'


class TestBlockquoteWithCite:
    def test_simple_blockquote(self) -> None:
        html = "<blockquote>Simple quote</blockquote>"
        result = convert_to_markdown(html)
        assert result == "\n> Simple quote\n\n"

    def test_blockquote_with_cite(self) -> None:
        html = '<blockquote cite="https://example.com">Quote with source</blockquote>'
        result = convert_to_markdown(html)
        expected = "\n> Quote with source\n\n— <https://example.com>\n\n"
        assert result == expected

    def test_blockquote_with_cite_and_content(self) -> None:
        html = '<blockquote cite="https://shakespeare.com"><p>To be or not to be, that is the question.</p><p>Whether \'tis nobler in the mind to suffer...</p></blockquote>'
        result = convert_to_markdown(html)
        expected = "\n> To be or not to be, that is the question.\n> \n> Whether 'tis nobler in the mind to suffer...\n\n— <https://shakespeare.com>\n\n"
        assert result == expected

    def test_nested_blockquotes(self) -> None:
        html = '<blockquote cite="https://outer.com">Outer quote<blockquote cite="https://inner.com">Inner quote</blockquote>Back to outer</blockquote>'
        result = convert_to_markdown(html)
        expected = "\n> Outer quote\n> > Inner quote\n> \n> — <https://inner.com>\n> \n> Back to outer\n\n— <https://outer.com>\n\n"
        assert result == expected

    def test_blockquote_inline_mode(self) -> None:
        html = '<blockquote cite="https://example.com">Inline quote</blockquote>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline quote"

    def test_empty_blockquote_with_cite(self) -> None:
        html = '<blockquote cite="https://example.com"></blockquote>'
        result = convert_to_markdown(html)
        assert result == ""


class TestMixedCitationsAndQuotations:
    def test_cite_in_blockquote(self) -> None:
        html = "<blockquote>Quote by <cite>Author Name</cite></blockquote>"
        result = convert_to_markdown(html)
        assert result == "\n> Quote by *Author Name*\n\n"

    def test_q_in_blockquote(self) -> None:
        html = "<blockquote>He said <q>Hello world</q> to everyone.</blockquote>"
        result = convert_to_markdown(html)
        assert result == '\n> He said "Hello world" to everyone.\n\n'

    def test_blockquote_in_cite(self) -> None:
        html = "<cite>Author: <blockquote>Their famous quote</blockquote></cite>"
        result = convert_to_markdown(html)
        assert result == "*Author: \n> Their famous quote*"

    def test_complex_citation_structure(self) -> None:
        html = '<article><p>According to <cite><a href="https://example.com">John Doe</a></cite>, the statement <q>Innovation drives progress</q> is fundamental.</p><blockquote cite="https://johndoe.com/quotes"><p>Innovation is not just about technology, it\'s about <em>thinking differently</em>.</p><cite>John Doe, 2023</cite></blockquote></article>'
        result = convert_to_markdown(html)
        expected = 'According to *[John Doe](https://example.com)*, the statement "Innovation drives progress" is fundamental.\n\n> Innovation is not just about technology, it\'s about *thinking differently*.\n> \n> *John Doe, 2023*\n\n— <https://johndoe.com/quotes>\n\n'
        assert result == expected

    def test_quote_escaping_edge_cases(self) -> None:
        html = '<div><q>Quote with "nested quotes" and \'single quotes\'</q><q>Quote with backslash: \\</q><q>Quote with both \\" and regular quotes</q></div>'
        result = convert_to_markdown(html)
        expected = '"Quote with \\"nested quotes\\" and \'single quotes\'""Quote with backslash: \\\\""Quote with both \\\\\\" and regular quotes"'
        assert result == expected

    def test_attributes_preservation(self) -> None:
        html = '<blockquote cite="https://example.com" class="important" id="quote1" data-author="John">Important quote</blockquote>'
        result = convert_to_markdown(html)
        expected = "\n> Important quote\n\n— <https://example.com>\n\n"
        assert result == expected
