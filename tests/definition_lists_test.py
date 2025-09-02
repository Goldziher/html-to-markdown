from html_to_markdown import convert_to_markdown


class TestDefinitionLists:
    """Test definition list (dl, dt, dd) conversion."""

    def test_simple_definition_list(self) -> None:
        """Test simple definition list conversion."""
        html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
        result = convert_to_markdown(html)
        expected = "Term\n:   Definition\n\n"
        assert result == expected

    def test_multiple_terms_and_definitions(self) -> None:
        html = "<dl><dt>First Term</dt><dd>First Definition</dd><dt>Second Term</dt><dd>Second Definition</dd></dl>"
        result = convert_to_markdown(html)
        expected = "First Term\n:   First Definition\n\nSecond Term\n:   Second Definition\n\n"
        assert result == expected

    def test_term_with_multiple_definitions(self) -> None:
        html = "<dl><dt>Term</dt><dd>First definition</dd><dd>Second definition</dd></dl>"
        result = convert_to_markdown(html)
        expected = "Term\n:   First definition\n\n:   Second definition\n\n"
        assert result == expected

    def test_multiple_terms_single_definition(self) -> None:
        html = "<dl><dt>Term 1</dt><dt>Term 2</dt><dd>Shared definition</dd></dl>"
        result = convert_to_markdown(html)
        expected = "Term 1\nTerm 2\n:   Shared definition\n\n"
        assert result == expected

    def test_definition_with_inline_formatting(self) -> None:
        html = "<dl><dt><strong>Bold Term</strong></dt><dd>Definition with <em>italic</em> text</dd></dl>"
        result = convert_to_markdown(html)
        expected = "**Bold Term**\n:   Definition with *italic* text\n\n"
        assert result == expected

    def test_definition_with_links(self) -> None:
        html = '<dl><dt><a href="https://example.com">Linked Term</a></dt><dd>Definition with <a href="https://test.com">link</a></dd></dl>'
        result = convert_to_markdown(html)
        expected = "[Linked Term](https://example.com)\n:   Definition with [link](https://test.com)\n\n"
        assert result == expected

    def test_definition_with_code(self) -> None:
        html = "<dl><dt><code>function</code></dt><dd>A block of code with <code>parameters</code></dd></dl>"
        result = convert_to_markdown(html)
        expected = "`function`\n:   A block of code with `parameters`\n\n"
        assert result == expected

    def test_nested_definition_lists(self) -> None:
        html = (
            "<dl><dt>Outer Term</dt><dd>Outer definition<dl><dt>Inner Term</dt><dd>Inner definition</dd></dl></dd></dl>"
        )
        result = convert_to_markdown(html)
        expected = "Outer Term\n:   Outer definitionInner Term\n:   Inner definition\n\n"
        assert result == expected

    def test_definition_with_paragraphs(self) -> None:
        html = "<dl><dt>Complex Term</dt><dd><p>First paragraph of definition.</p><p>Second paragraph of definition.</p></dd></dl>"
        result = convert_to_markdown(html)
        expected = "Complex Term\n:   First paragraph of definition.\n\nSecond paragraph of definition.\n\n"
        assert result == expected

    def test_definition_with_lists(self) -> None:
        html = "<dl><dt>List Term</dt><dd>Definition with list:<ul><li>Item 1</li><li>Item 2</li></ul></dd></dl>"
        result = convert_to_markdown(html)
        expected = "List Term\n:   Definition with list:* Item 1\n* Item 2\n\n"
        assert result == expected

    def test_empty_definition_list(self) -> None:
        html = "<dl></dl>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_empty_term(self) -> None:
        html = "<dl><dt></dt><dd>Definition without term</dd></dl>"
        result = convert_to_markdown(html)
        expected = ":   Definition without term\n\n"
        assert result == expected

    def test_empty_definition(self) -> None:
        html = "<dl><dt>Term without definition</dt><dd></dd></dl>"
        result = convert_to_markdown(html)
        expected = "Term without definition\n\n"
        assert result == expected

    def test_definition_list_inline_mode(self) -> None:
        html = "<dl><dt>Term</dt><dd>Definition</dd></dl>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "TermDefinition"

    def test_whitespace_handling(self) -> None:
        html = "<dl><dt>  Term with spaces  </dt><dd>  Definition with spaces  </dd></dl>"
        result = convert_to_markdown(html)
        expected = "Term with spaces\n:   Definition with spaces\n\n"
        assert result == expected

    def test_definition_with_blockquote(self) -> None:
        html = "<dl><dt>Quote Term</dt><dd><blockquote>This is a quoted definition.</blockquote></dd></dl>"
        result = convert_to_markdown(html)
        expected = "Quote Term\n:   > This is a quoted definition.\n\n"
        assert result == expected

    def test_complex_definition_list(self) -> None:
        html = "<dl><dt><strong>HTML</strong></dt><dd>HyperText Markup Language</dd><dt><em>CSS</em></dt><dt>Cascading Style Sheets</dt><dd>A style sheet language used for describing the presentation of a document written in HTML</dd><dd>Also used with XML documents</dd><dt><code>JavaScript</code></dt><dd>A programming language that conforms to the ECMAScript specification.<ul><li>Dynamic typing</li><li>First-class functions</li></ul></dd></dl>"
        result = convert_to_markdown(html)
        expected = "**HTML**\n:   HyperText Markup Language\n\n*CSS*\nCascading Style Sheets\n:   A style sheet language used for describing the presentation of a document written in HTML\n\n:   Also used with XML documents\n\n`JavaScript`\n:   A programming language that conforms to the ECMAScript specification.* Dynamic typing\n* First\\-class functions\n\n"
        assert result == expected

    def test_definition_list_attributes(self) -> None:
        html = '<dl class="definitions" id="main-list"><dt title="Term title">Term</dt><dd data-id="1">Definition</dd></dl>'
        result = convert_to_markdown(html)
        expected = "Term\n:   Definition\n\n"
        assert result == expected
