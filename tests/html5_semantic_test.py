from html_to_markdown import convert_to_markdown


class TestSemanticElements:
    """Test HTML5 semantic elements conversion."""

    def test_article_element(self) -> None:
        """Test article element conversion."""
        html = "<article>This is an article</article>"
        result = convert_to_markdown(html)
        assert result == "This is an article\n\n"

    def test_section_element(self) -> None:
        html = "<section>This is a section</section>"
        result = convert_to_markdown(html)
        assert result == "This is a section\n\n"

    def test_nav_element(self) -> None:
        html = "<nav>This is navigation</nav>"
        result = convert_to_markdown(html)
        assert result == "This is navigation\n\n"

    def test_aside_element(self) -> None:
        html = "<aside>This is an aside</aside>"
        result = convert_to_markdown(html)
        assert result == "This is an aside\n\n"

    def test_header_element(self) -> None:
        html = "<header>This is a header</header>"
        result = convert_to_markdown(html)
        assert result == "This is a header\n\n"

    def test_footer_element(self) -> None:
        html = "<footer>This is a footer</footer>"
        result = convert_to_markdown(html)
        assert result == "This is a footer\n\n"

    def test_main_element(self) -> None:
        html = "<main>This is main content</main>"
        result = convert_to_markdown(html)
        assert result == "This is main content\n\n"

    def test_nested_semantic_elements(self) -> None:
        html = "<article><header>Article Header</header><section><h2>Section Title</h2><p>Section content</p></section><footer>Article Footer</footer></article>"
        result = convert_to_markdown(html, heading_style="atx")
        expected = "Article Header\n\n## Section Title\n\nSection content\n\nArticle Footer\n\n"
        assert result == expected

    def test_semantic_elements_with_other_content(self) -> None:
        html = '<nav><ul><li><a href="#home">Home</a></li><li><a href="#about">About</a></li></ul></nav><main><article><h1>Article Title</h1><p>Article content</p></article></main>'
        result = convert_to_markdown(html, heading_style="atx")
        expected = "* [Home](#home)\n* [About](#about)\n\n# Article Title\n\nArticle content\n\n"
        assert result == expected

    def test_empty_semantic_elements(self) -> None:
        html = "<article></article>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_semantic_elements_inline_mode(self) -> None:
        html = "<article>This is inline content</article>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "This is inline content"

    def test_semantic_elements_with_whitespace(self) -> None:
        html = "<section>  \n  Content with whitespace  \n  </section>"
        result = convert_to_markdown(html)
        assert result == " Content with whitespace \n\n"


class TestCollapsibleContent:
    def test_details_element(self) -> None:
        html = "<details>This is details content</details>"
        result = convert_to_markdown(html)
        assert result == "This is details content\n\n"

    def test_summary_element(self) -> None:
        html = "<summary>Summary text</summary>"
        result = convert_to_markdown(html)
        assert result == "**Summary text**\n\n"

    def test_details_with_summary(self) -> None:
        html = "<details><summary>Click to expand</summary><p>Hidden content here</p></details>"
        result = convert_to_markdown(html)
        expected = "**Click to expand**\n\nHidden content here\n\n"
        assert result == expected

    def test_nested_details(self) -> None:
        html = "<details><summary>Level 1</summary><details><summary>Level 2</summary><p>Nested content</p></details></details>"
        result = convert_to_markdown(html)
        expected = "**Level 1**\n\n**Level 2**\n\nNested content\n\n"
        assert result == expected

    def test_details_with_complex_content(self) -> None:
        html = '<details><summary>Code Example</summary><pre><code>def hello():\n    print("Hello, World!")</code></pre><p>This is a Python function.</p></details>'
        result = convert_to_markdown(html)
        expected = (
            '**Code Example**\n\n```\ndef hello():\n    print("Hello, World!")\n```\nThis is a Python function.\n\n'
        )
        assert result == expected

    def test_empty_details(self) -> None:
        html = "<details></details>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_empty_summary(self) -> None:
        html = "<summary></summary>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_details_inline_mode(self) -> None:
        html = "<details>Inline details</details>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline details"

    def test_summary_inline_mode(self) -> None:
        html = "<summary>Inline summary</summary>"
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline summary"

    def test_details_with_attributes(self) -> None:
        html = "<details open><summary>Always open</summary><p>Content</p></details>"
        result = convert_to_markdown(html)
        expected = "**Always open**\n\nContent\n\n"
        assert result == expected
