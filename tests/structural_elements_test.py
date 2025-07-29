"""Tests for structural HTML elements (figure, hgroup, picture)."""

from html_to_markdown import convert_to_markdown


class TestFigureElement:
    """Test figure element conversion."""

    def test_figure_basic(self) -> None:
        """Test basic figure conversion."""
        html = '<figure><img src="image.jpg" alt="Test image"></figure>'
        result = convert_to_markdown(html)
        # Figure is a semantic container, only content is converted
        assert result == "![Test image](image.jpg)\n\n"

    def test_figure_with_caption(self) -> None:
        """Test figure with figcaption."""
        html = '<figure><img src="test.jpg"><figcaption>Image caption</figcaption></figure>'
        result = convert_to_markdown(html)
        # Figure and figcaption convert to semantic Markdown
        expected = "![](test.jpg)\n\n*Image caption*\n\n"
        assert result == expected

    def test_figure_with_id(self) -> None:
        """Test figure with id attribute."""
        html = '<figure id="fig1"><img src="chart.png"></figure>'
        result = convert_to_markdown(html)
        # ID attribute is not preserved in Markdown
        assert result == "![](chart.png)\n\n"

    def test_figure_with_class(self) -> None:
        """Test figure with class attribute."""
        html = '<figure class="photo"><img src="photo.jpg"></figure>'
        result = convert_to_markdown(html)
        # Class attribute is not preserved in Markdown
        assert result == "![](photo.jpg)\n\n"

    def test_figure_with_multiple_attributes(self) -> None:
        """Test figure with multiple attributes."""
        html = '<figure id="fig2" class="diagram"><img src="diagram.svg"></figure>'
        result = convert_to_markdown(html)
        # Attributes are not preserved in Markdown
        assert result == "![](diagram.svg)\n\n"

    def test_figure_empty(self) -> None:
        """Test empty figure."""
        html = "<figure></figure>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_figure_inline_mode(self) -> None:
        """Test figure in inline mode."""
        html = '<figure><img src="inline.jpg" alt="Inline image"></figure>'
        result = convert_to_markdown(html, convert_as_inline=True)
        assert result == "Inline image"

    def test_figure_with_complex_content(self) -> None:
        """Test figure with multiple elements."""
        html = """<figure>
            <img src="main.jpg" alt="Main image">
            <figcaption>
                <strong>Figure 1:</strong> This is a complex caption with <em>emphasis</em>.
            </figcaption>
        </figure>"""
        result = convert_to_markdown(html)
        expected = """![Main image](main.jpg)\n\n***Figure 1:** This is a complex caption with *emphasis*.*\n\n"""
        assert result == expected

    def test_figure_with_multiple_images(self) -> None:
        """Test figure containing multiple images."""
        html = """<figure>
            <img src="before.jpg" alt="Before">
            <img src="after.jpg" alt="After">
            <figcaption>Before and after comparison</figcaption>
        </figure>"""
        result = convert_to_markdown(html)
        expected = """![Before](before.jpg)![After](after.jpg)\n\n*Before and after comparison*\n\n"""
        assert result == expected

    def test_figure_with_nested_elements(self) -> None:
        """Test figure with various nested elements."""
        html = """<figure id="stats">
            <table>
                <tr><th>Year</th><th>Sales</th></tr>
                <tr><td>2023</td><td>100</td></tr>
            </table>
            <figcaption>Annual sales data</figcaption>
        </figure>"""
        result = convert_to_markdown(html)
        expected = """| Year | Sales |\n| --- | --- |\n| 2023 | 100 |\n\n*Annual sales data*\n\n"""
        assert result == expected


class TestHgroupElement:
    """Test hgroup element conversion."""

    def test_hgroup_basic(self) -> None:
        """Test basic hgroup conversion."""
        html = "<hgroup><h1>Main Title</h1><h2>Subtitle</h2></hgroup>"
        result = convert_to_markdown(html)
        # Hgroup is a semantic container, headings are converted normally
        expected = "Main Title\n==========\n\nSubtitle\n--------\n\n"
        assert result == expected

    def test_hgroup_multiple_headings(self) -> None:
        """Test hgroup with multiple heading levels."""
        html = "<hgroup><h1>Title</h1><h2>Subtitle</h2><h3>Section</h3></hgroup>"
        result = convert_to_markdown(html)
        # All headings are converted to their Markdown equivalents
        expected = "Title\n=====\n\nSubtitle\n--------\n\n### Section\n\n"
        assert result == expected

    def test_hgroup_empty(self) -> None:
        """Test empty hgroup."""
        html = "<hgroup></hgroup>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_hgroup_inline_mode(self) -> None:
        """Test hgroup in inline mode."""
        html = "<hgroup><h1>Inline Title</h1></hgroup>"
        result = convert_to_markdown(html, convert_as_inline=True)
        # In inline mode, just the text content
        assert result == "Inline Title"

    def test_hgroup_with_atx_headings(self) -> None:
        """Test hgroup with ATX-style headings."""
        html = "<hgroup><h1>Main</h1><h2>Sub</h2></hgroup>"
        result = convert_to_markdown(html, heading_style="atx")
        expected = "# Main\n\n## Sub\n\n"
        assert result == expected

    def test_hgroup_excessive_spacing(self) -> None:
        """Test that hgroup removes excessive spacing between headings."""
        html = "<hgroup><h1>Title</h1><p></p><p></p><h2>Subtitle</h2></hgroup>"
        result = convert_to_markdown(html)
        # Empty paragraphs are removed, headings have normal spacing
        expected = "Title\n=====\n\nSubtitle\n--------\n\n"
        assert result == expected

    def test_hgroup_with_formatted_headings(self) -> None:
        """Test hgroup with formatted heading content."""
        html = "<hgroup><h1>The <em>Amazing</em> Title</h1><h2>A <strong>Bold</strong> Subtitle</h2></hgroup>"
        result = convert_to_markdown(html)
        expected = "The *Amazing* Title\n===================\n\nA **Bold** Subtitle\n-------------------\n\n"
        assert result == expected


class TestPictureElement:
    """Test picture element conversion."""

    def test_picture_basic(self) -> None:
        """Test basic picture with just img."""
        html = '<picture><img src="image.jpg" alt="Test"></picture>'
        result = convert_to_markdown(html)
        assert result == "![Test](image.jpg)"

    def test_picture_with_source(self) -> None:
        """Test picture with source element."""
        html = """<picture>
            <source srcset="large.jpg" media="(min-width: 800px)">
            <img src="small.jpg" alt="Responsive image">
        </picture>"""
        result = convert_to_markdown(html)
        # Picture is a container, only the img is converted
        assert result == "![Responsive image](small.jpg)"

    def test_picture_multiple_sources(self) -> None:
        """Test picture with multiple source elements."""
        html = """<picture>
            <source srcset="image.webp" type="image/webp">
            <source srcset="image.jpg" type="image/jpeg">
            <img src="fallback.jpg" alt="Multi-format">
        </picture>"""
        result = convert_to_markdown(html)
        # Only the img element is converted to Markdown
        assert result == "![Multi-format](fallback.jpg)"

    def test_picture_complex_srcset(self) -> None:
        """Test picture with complex srcset values."""
        html = """<picture>
            <source srcset="small.jpg 480w, medium.jpg 800w, large.jpg 1200w"
                    media="(min-width: 600px)">
            <img src="default.jpg">
        </picture>"""
        result = convert_to_markdown(html)
        # Source elements are ignored, only img is converted
        assert result == "![](default.jpg)"

    def test_picture_no_img(self) -> None:
        """Test picture without img element."""
        html = '<picture><source srcset="test.jpg"></picture>'
        result = convert_to_markdown(html)
        assert result == ""

    def test_picture_empty(self) -> None:
        """Test empty picture."""
        html = "<picture></picture>"
        result = convert_to_markdown(html)
        assert result == ""

    def test_picture_inline_mode(self) -> None:
        """Test picture in inline mode."""
        html = """<picture>
            <source srcset="large.jpg" media="(min-width: 800px)">
            <img src="small.jpg" alt="Test">
        </picture>"""
        result = convert_to_markdown(html, convert_as_inline=True)
        # In inline mode, just the alt text
        assert result == "Test"

    def test_picture_with_sizes(self) -> None:
        """Test picture with sizes attribute."""
        html = """<picture>
            <source srcset="img-480.jpg 480w, img-800.jpg 800w"
                    sizes="(max-width: 600px) 480px, 800px">
            <img src="default.jpg">
        </picture>"""
        result = convert_to_markdown(html)
        # Only img element matters
        assert result == "![](default.jpg)"


class TestStructuralElementsIntegration:
    """Test structural elements in various contexts."""

    def test_figure_in_article(self) -> None:
        """Test figure within article structure."""
        html = """<article>
            <h1>Article Title</h1>
            <figure id="main-image">
                <img src="hero.jpg" alt="Hero image">
                <figcaption>The main article image</figcaption>
            </figure>
            <p>Article content...</p>
        </article>"""
        result = convert_to_markdown(html)
        expected = """Article Title\n=============\n\n![Hero image](hero.jpg)\n\n*The main article image*\n\nArticle content...\n\n"""
        assert result == expected

    def test_hgroup_in_header(self) -> None:
        """Test hgroup within header element."""
        html = """<header>
            <hgroup>
                <h1>Site Title</h1>
                <h2>Site Tagline</h2>
            </hgroup>
            <nav>Navigation here</nav>
        </header>"""
        result = convert_to_markdown(html)
        expected = """Site Title\n==========\n\nSite Tagline\n------------\n\nNavigation here\n\n"""
        assert result == expected

    def test_picture_in_figure(self) -> None:
        """Test picture element within figure."""
        html = """<figure>
            <picture>
                <source srcset="large.webp" type="image/webp">
                <img src="fallback.jpg" alt="Test image">
            </picture>
            <figcaption>A responsive image in a figure</figcaption>
        </figure>"""
        result = convert_to_markdown(html)
        expected = """![Test image](fallback.jpg)\n\n*A responsive image in a figure*\n\n"""
        assert result == expected

    def test_multiple_figures(self) -> None:
        """Test multiple figure elements."""
        html = """
        <figure id="fig1">
            <img src="image1.jpg">
            <figcaption>First figure</figcaption>
        </figure>
        <figure id="fig2">
            <img src="image2.jpg">
            <figcaption>Second figure</figcaption>
        </figure>
        """
        result = convert_to_markdown(html)
        expected = """\n        ![](image1.jpg)\n\n*First figure*\n\n![](image2.jpg)\n\n*Second figure*\n\n"""
        assert result == expected

    def test_nested_structural_elements(self) -> None:
        """Test complex nesting of structural elements."""
        html = """<section>
            <hgroup>
                <h1>Section Title</h1>
                <h2>Section Subtitle</h2>
            </hgroup>
            <figure>
                <picture>
                    <source srcset="chart.svg" type="image/svg+xml">
                    <img src="chart.png" alt="Data chart">
                </picture>
                <figcaption>Quarterly results</figcaption>
            </figure>
        </section>"""
        result = convert_to_markdown(html)
        expected = """Section Title\n=============\n\nSection Subtitle\n----------------\n\n![Data chart](chart.png)\n\n*Quarterly results*\n\n"""
        assert result == expected


class TestStructuralElementsEdgeCases:
    """Test edge cases for structural elements."""

    def test_figure_with_special_characters(self) -> None:
        """Test figure with special Markdown characters."""
        html = (
            '<figure><img src="test.jpg"><figcaption>Caption with *asterisks* and _underscores_</figcaption></figure>'
        )
        result = convert_to_markdown(html)
        expected = "![](test.jpg)\n\n*Caption with \\*asterisks\\* and \\_underscores\\_*\n\n"
        assert result == expected

    def test_hgroup_single_heading(self) -> None:
        """Test hgroup with only one heading."""
        html = "<hgroup><h1>Only Title</h1></hgroup>"
        result = convert_to_markdown(html)
        expected = "Only Title\n==========\n\n"
        assert result == expected

    def test_picture_malformed_source(self) -> None:
        """Test picture with malformed source elements."""
        html = """<picture>
            <source>
            <source srcset="">
            <img src="valid.jpg">
        </picture>"""
        result = convert_to_markdown(html)
        # Malformed sources are ignored
        assert result == "![](valid.jpg)"

    def test_figure_whitespace_handling(self) -> None:
        """Test figure with various whitespace."""
        html = """<figure>

            <img src="test.jpg">

            <figcaption>
                Caption text
            </figcaption>

        </figure>"""
        result = convert_to_markdown(html)
        expected = "![](test.jpg)\n\n*Caption text*\n\n"
        assert result == expected

    def test_empty_elements_with_attributes(self) -> None:
        """Test empty structural elements with attributes."""
        html1 = '<figure id="empty-fig"></figure>'
        assert convert_to_markdown(html1) == ""

        html2 = '<hgroup class="empty"></hgroup>'
        assert convert_to_markdown(html2) == ""

        html3 = '<picture id="empty-pic"></picture>'
        assert convert_to_markdown(html3) == ""

    def test_figure_with_pre_content(self) -> None:
        """Test figure containing preformatted content."""
        html = """<figure>
            <pre><code>function example() {
  return 42;
}</code></pre>
            <figcaption>Code example</figcaption>
        </figure>"""
        result = convert_to_markdown(html)
        expected = """```\nfunction example() {\n  return 42;\n}\n```\n\n*Code example*\n\n"""
        assert result == expected
