"""Tests for SVG and Math elements."""

import base64

from html_to_markdown import convert_to_markdown


class TestSVGElement:
    """Test SVG element conversion."""

    def test_svg_basic(self) -> None:
        """Test basic SVG conversion to data URI."""
        svg = '<svg width="100" height="100"><circle cx="50" cy="50" r="40" /></svg>'
        result = convert_to_markdown(svg, extract_metadata=False)

        # Should convert to image with data URI
        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
        assert result.endswith(")")

        # Verify it's valid base64
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert 'width="100"' in decoded
        assert 'height="100"' in decoded
        assert 'cx="50"' in decoded
        assert 'cy="50"' in decoded
        assert 'r="40"' in decoded

    def test_svg_with_title(self) -> None:
        """Test SVG with title element for alt text."""
        svg = """<svg>
            <title>My Chart</title>
            <rect width="100" height="100" />
        </svg>"""
        result = convert_to_markdown(svg, extract_metadata=False)

        # Should use title as alt text
        assert result.startswith("![My Chart](data:image/svg+xml;base64,")

    def test_svg_complex(self) -> None:
        """Test complex SVG with multiple elements."""
        svg = """<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
            <title>Complex SVG</title>
            <rect x="10" y="10" width="180" height="180" fill="blue" />
            <circle cx="100" cy="100" r="50" fill="red" />
            <text x="100" y="100" text-anchor="middle">Hello</text>
        </svg>"""
        result = convert_to_markdown(svg, extract_metadata=False)

        assert result.startswith("![Complex SVG](data:image/svg+xml;base64,")

        # Decode and verify content
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert 'width="200"' in decoded
        assert 'fill="blue"' in decoded
        assert ">Hello</text>" in decoded

    def test_svg_inline_mode(self) -> None:
        """Test SVG in inline mode."""
        svg = '<svg><title>Icon</title><path d="M10 10" /></svg>'
        result = convert_to_markdown(svg, convert_as_inline=True, extract_metadata=False)

        # In inline mode, should return text content (title text)
        assert result == "Icon"

    def test_svg_with_text_content(self) -> None:
        """Test SVG with text elements."""
        svg = "<svg><text>Label Text</text></svg>"
        result = convert_to_markdown(svg, extract_metadata=False)

        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")

        # Verify text is preserved in the encoded SVG
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert "Label Text" in decoded

    def test_svg_empty(self) -> None:
        """Test empty SVG element."""
        svg = "<svg></svg>"
        result = convert_to_markdown(svg, extract_metadata=False)

        # Should still create data URI even for empty SVG
        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")

    def test_svg_with_namespaces(self) -> None:
        """Test SVG with namespace declarations."""
        svg = '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="#icon" /></svg>'
        result = convert_to_markdown(svg, extract_metadata=False)

        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")

        # Verify namespaces are preserved
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert 'xmlns="http://www.w3.org/2000/svg"' in decoded
        assert 'xlink:href="#icon"' in decoded


class TestMathElement:
    """Test Math (MathML) element conversion."""

    def test_math_basic(self) -> None:
        """Test basic math element conversion."""
        math = "<math><mn>42</mn></math>"
        result = convert_to_markdown(math, extract_metadata=False)

        # Should preserve MathML as comment with text representation
        assert "<!-- MathML:" in result
        assert "<math><mn>42</mn></math>" in result
        assert "42" in result

    def test_math_inline(self) -> None:
        """Test inline math element."""
        math = "<math><mi>x</mi><mo>+</mo><mn>1</mn></math>"
        result = convert_to_markdown(math, extract_metadata=False)

        assert "<!-- MathML:" in result
        assert "x\\+1" in result

    def test_math_display_block(self) -> None:
        """Test display math (block mode)."""
        math = '<math display="block"><mfrac><mn>1</mn><mn>2</mn></mfrac></math>'
        result = convert_to_markdown(math, extract_metadata=False)

        # Display math should be on its own line
        assert result.startswith("\n\n<!-- MathML:")
        assert result.endswith("12\n\n")

    def test_math_complex(self) -> None:
        """Test complex MathML expression."""
        math = """<math>
            <msup>
                <mi>x</mi>
                <mn>2</mn>
            </msup>
            <mo>+</mo>
            <msup>
                <mi>y</mi>
                <mn>2</mn>
            </msup>
            <mo>=</mo>
            <msup>
                <mi>r</mi>
                <mn>2</mn>
            </msup>
        </math>"""
        result = convert_to_markdown(math, extract_metadata=False)

        assert "<!-- MathML:" in result
        assert "x" in result
        assert "2" in result
        assert "\\+" in result
        assert "y" in result
        assert "\\=" in result
        assert "r" in result

    def test_math_with_mtext(self) -> None:
        """Test math with text elements."""
        math = "<math><mtext>The answer is </mtext><mn>42</mn></math>"
        result = convert_to_markdown(math, extract_metadata=False)

        assert "The answer is 42" in result

    def test_math_empty(self) -> None:
        """Test empty math element."""
        math = "<math></math>"
        result = convert_to_markdown(math, extract_metadata=False)

        assert result == ""

    def test_math_inline_mode(self) -> None:
        """Test math in inline conversion mode."""
        math = '<math display="block"><mi>E</mi><mo>=</mo><mi>mc</mi><msup><mi></mi><mn>2</mn></msup></math>'
        result = convert_to_markdown(math, convert_as_inline=True)

        # Should still include MathML comment but inline
        assert "<!-- MathML:" in result
        assert "E\\=mc2" in result
        assert not result.startswith("\n\n")

    def test_math_with_special_chars(self) -> None:
        """Test math with special characters."""
        math = "<math><mo>&lt;</mo><mo>&gt;</mo><mo>&amp;</mo></math>"
        result = convert_to_markdown(math, extract_metadata=False)

        assert "\\<\\>\\&" in result


class TestSVGMathIntegration:
    """Test SVG and Math elements in various contexts."""

    def test_svg_in_paragraph(self) -> None:
        """Test SVG within paragraph."""
        html = '<p>Here is an icon: <svg width="16" height="16"><circle r="8" /></svg> inline.</p>'
        result = convert_to_markdown(html, extract_metadata=False)

        # SVG should be converted even within paragraph
        assert "Here is an icon: ![SVG Image](data:image/svg+xml;base64," in result

    def test_math_in_paragraph(self) -> None:
        """Test math within paragraph."""
        html = (
            "<p>The formula <math><mi>E</mi><mo>=</mo><mi>mc</mi><msup><mi></mi><mn>2</mn></msup></math> is famous.</p>"
        )
        result = convert_to_markdown(html, extract_metadata=False)

        assert "The formula <!-- MathML:" in result
        assert "E\\=mc2" in result
        assert "is famous." in result

    def test_svg_in_figure(self) -> None:
        """Test SVG within figure element."""
        html = """<figure>
            <svg><title>Chart</title><rect width="100" height="50" /></svg>
            <figcaption>Sales chart</figcaption>
        </figure>"""
        result = convert_to_markdown(html, extract_metadata=False)

        assert "<figure>" in result
        assert "![Chart](data:image/svg+xml;base64," in result
        assert "Sales chart" in result

    def test_multiple_svg_elements(self) -> None:
        """Test multiple SVG elements."""
        html = """
        <svg><title>Icon 1</title><circle r="5" /></svg>
        <svg><title>Icon 2</title><rect width="10" height="10" /></svg>
        """
        result = convert_to_markdown(html, extract_metadata=False)

        assert result.count("![Icon 1](data:image/svg+xml;base64,") == 1
        assert result.count("![Icon 2](data:image/svg+xml;base64,") == 1

    def test_nested_math_elements(self) -> None:
        """Test nested math elements."""
        html = """<div>
            <h2>Equations</h2>
            <math display="block">
                <mi>a</mi><mo>+</mo><mi>b</mi>
            </math>
            <p>And also:</p>
            <math display="block">
                <mi>c</mi><mo>-</mo><mi>d</mi>
            </math>
        </div>"""
        result = convert_to_markdown(html, extract_metadata=False)

        assert "Equations" in result
        assert "a\\+b" in result
        assert "And also:" in result
        assert "c\\-d" in result

    def test_svg_with_fallback_img(self) -> None:
        """Test SVG with img fallback pattern."""
        html = """<picture>
            <source type="image/svg+xml" srcset="chart.svg">
            <img src="chart.png" alt="Chart">
        </picture>"""
        result = convert_to_markdown(html, extract_metadata=False)

        # Picture element should handle this, not SVG converter
        assert "<!-- picture sources:" in result
        assert "![Chart](chart.png)" in result


class TestSVGMathEdgeCases:
    """Test edge cases for SVG and Math elements."""

    def test_svg_with_script(self) -> None:
        """Test SVG with embedded script (should be preserved)."""
        svg = '<svg><script>alert("test")</script><circle r="10" /></svg>'
        result = convert_to_markdown(svg, extract_metadata=False)

        # Script should be preserved in the data URI
        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert "<script>" in decoded

    def test_math_with_annotation(self) -> None:
        """Test math with annotation elements."""
        math = """<math>
            <semantics>
                <mrow><mi>x</mi><mo>+</mo><mn>1</mn></mrow>
                <annotation encoding="TeX">x + 1</annotation>
            </semantics>
        </math>"""
        result = convert_to_markdown(math, extract_metadata=False)

        assert "x\\+1" in result
        assert "x \\+ 1" in result  # Annotation text also appears

    def test_svg_with_style(self) -> None:
        """Test SVG with style element."""
        svg = '<svg><style>.red { fill: red; }</style><circle class="red" r="10" /></svg>'
        result = convert_to_markdown(svg, extract_metadata=False)

        assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
        data_uri = result[result.find("base64,") + 7 : -1]
        decoded = base64.b64decode(data_uri).decode("utf-8")
        assert "<style>" in decoded
        assert "fill: red" in decoded

    def test_math_whitespace_handling(self) -> None:
        """Test math with various whitespace."""
        math = """<math>
            <mi> x </mi>
            <mo> + </mo>
            <mi> y </mi>
        </math>"""
        result = convert_to_markdown(math, extract_metadata=False)

        # Should handle whitespace appropriately
        assert "x" in result
        assert "\\+" in result
        assert "y" in result

    def test_svg_special_characters_in_title(self) -> None:
        """Test SVG with special characters in title."""
        svg = "<svg><title>Chart & Graph</title><rect /></svg>"
        result = convert_to_markdown(svg, extract_metadata=False)

        assert "![Chart & Graph](data:image/svg+xml;base64," in result

    def test_empty_math_with_display(self) -> None:
        """Test empty math element with display attribute."""
        math = '<math display="block"></math>'
        result = convert_to_markdown(math, extract_metadata=False)

        assert result == ""
