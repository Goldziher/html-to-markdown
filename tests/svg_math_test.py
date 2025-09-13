import base64

from html_to_markdown import convert_to_markdown


def test_svg_basic() -> None:
    svg = '<svg width="100" height="100"><circle cx="50" cy="50" r="40" /></svg>'
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
    assert result.endswith(")")

    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert 'width="100"' in decoded
    assert 'height="100"' in decoded
    assert 'cx="50"' in decoded
    assert 'cy="50"' in decoded
    assert 'r="40"' in decoded


def test_svg_with_title() -> None:
    svg = """<svg>
        <title>My Chart</title>
        <rect width="100" height="100" />
    </svg>"""
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![My Chart](data:image/svg+xml;base64,")


def test_svg_complex() -> None:
    svg = """<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
        <title>Complex SVG</title>
        <rect x="10" y="10" width="180" height="180" fill="blue" />
        <circle cx="100" cy="100" r="50" fill="red" />
        <text x="100" y="100" text-anchor="middle">Hello</text>
    </svg>"""
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![Complex SVG](data:image/svg+xml;base64,")

    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert 'width="200"' in decoded
    assert 'fill="blue"' in decoded
    assert ">Hello</text>" in decoded


def test_svg_inline_mode() -> None:
    svg = '<svg><title>Icon</title><path d="M10 10" /></svg>'
    result = convert_to_markdown(svg, convert_as_inline=True, extract_metadata=False)

    assert result == "Icon"


def test_svg_with_text_content() -> None:
    svg = "<svg><text>Label Text</text></svg>"
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")

    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert "Label Text" in decoded


def test_svg_empty() -> None:
    svg = "<svg></svg>"
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")


def test_svg_with_namespaces() -> None:
    svg = '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="#icon" /></svg>'
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")

    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert 'xmlns="http://www.w3.org/2000/svg"' in decoded
    assert 'xlink:href="#icon"' in decoded


def test_math_basic() -> None:
    math = "<math><mn>42</mn></math>"
    result = convert_to_markdown(math, extract_metadata=False)

    assert "<!-- MathML:" in result
    assert "<math><mn>42</mn></math>" in result
    assert "42" in result


def test_math_inline() -> None:
    math = "<math><mi>x</mi><mo>+</mo><mn>1</mn></math>"
    result = convert_to_markdown(math, extract_metadata=False)

    assert "<!-- MathML:" in result
    assert "x\\+1" in result


def test_math_display_block() -> None:
    math = '<math display="block"><mfrac><mn>1</mn><mn>2</mn></mfrac></math>'
    result = convert_to_markdown(math, extract_metadata=False)

    assert result.startswith("\n\n<!-- MathML:")
    assert result.endswith("12\n\n")


def test_math_complex() -> None:
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


def test_math_with_mtext() -> None:
    math = "<math><mtext>The answer is </mtext><mn>42</mn></math>"
    result = convert_to_markdown(math, extract_metadata=False)

    assert "The answer is 42" in result


def test_math_empty() -> None:
    math = "<math></math>"
    result = convert_to_markdown(math, extract_metadata=False)

    assert result == ""


def test_math_inline_mode() -> None:
    math = '<math display="block"><mi>E</mi><mo>=</mo><mi>mc</mi><msup><mi></mi><mn>2</mn></msup></math>'
    result = convert_to_markdown(math, convert_as_inline=True)

    assert "<!-- MathML:" in result
    assert "E\\=mc2" in result
    assert not result.startswith("\n\n")


def test_math_with_special_chars() -> None:
    math = "<math><mo>&lt;</mo><mo>&gt;</mo><mo>&amp;</mo></math>"
    result = convert_to_markdown(math, extract_metadata=False)

    assert "\\<\\>\\&" in result


def test_svg_in_paragraph() -> None:
    html = '<p>Here is an icon: <svg width="16" height="16"><circle r="8" /></svg> inline.</p>'
    result = convert_to_markdown(html, extract_metadata=False)

    assert "Here is an icon: ![SVG Image](data:image/svg+xml;base64," in result


def test_math_in_paragraph() -> None:
    html = "<p>The formula <math><mi>E</mi><mo>=</mo><mi>mc</mi><msup><mi></mi><mn>2</mn></msup></math> is famous.</p>"
    result = convert_to_markdown(html, extract_metadata=False)

    assert "The formula <!-- MathML:" in result
    assert "E\\=mc2" in result
    assert "is famous." in result


def test_svg_in_figure() -> None:
    html = """<figure>
        <svg><title>Chart</title><rect width="100" height="50" /></svg>
        <figcaption>Sales chart</figcaption>
    </figure>"""
    result = convert_to_markdown(html, extract_metadata=False)

    assert "![Chart](data:image/svg+xml;base64," in result
    assert "*Sales chart*" in result


def test_multiple_svg_elements() -> None:
    html = """
    <svg><title>Icon 1</title><circle r="5" /></svg>
    <svg><title>Icon 2</title><rect width="10" height="10" /></svg>
    """
    result = convert_to_markdown(html, extract_metadata=False)

    assert result.count("![Icon 1](data:image/svg+xml;base64,") == 1
    assert result.count("![Icon 2](data:image/svg+xml;base64,") == 1


def test_nested_math_elements() -> None:
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


def test_svg_with_fallback_img() -> None:
    html = """<picture>
        <source type="image/svg+xml" srcset="chart.svg">
        <img src="chart.png" alt="Chart">
    </picture>"""
    result = convert_to_markdown(html, extract_metadata=False)

    assert result == "![Chart](chart.png)"


def test_svg_with_script() -> None:
    svg = '<svg><script>alert("test")</script><circle r="10" /></svg>'
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert "<script>" in decoded


def test_math_with_annotation() -> None:
    math = """<math>
        <semantics>
            <mrow><mi>x</mi><mo>+</mo><mn>1</mn></mrow>
            <annotation encoding="TeX">x + 1</annotation>
        </semantics>
    </math>"""
    result = convert_to_markdown(math, extract_metadata=False)

    assert "x\\+1" in result
    assert "x \\+ 1" in result


def test_svg_with_style() -> None:
    svg = '<svg><style>.red { fill: red; }</style><circle class="red" r="10" /></svg>'
    result = convert_to_markdown(svg, extract_metadata=False)

    assert result.startswith("![SVG Image](data:image/svg+xml;base64,")
    data_uri = result[result.find("base64,") + 7 : -1]
    decoded = base64.b64decode(data_uri).decode("utf-8")
    assert "<style>" in decoded
    assert "fill: red" in decoded


def test_math_whitespace_handling() -> None:
    math = """<math>
        <mi> x </mi>
        <mo> + </mo>
        <mi> y </mi>
    </math>"""
    result = convert_to_markdown(math, extract_metadata=False)

    assert "x" in result
    assert "\\+" in result
    assert "y" in result


def test_svg_special_characters_in_title() -> None:
    svg = "<svg><title>Chart & Graph</title><rect /></svg>"
    result = convert_to_markdown(svg, extract_metadata=False)

    assert "![Chart & Graph](data:image/svg+xml;base64," in result


def test_empty_math_with_display() -> None:
    math = '<math display="block"></math>'
    result = convert_to_markdown(math, extract_metadata=False)

    assert result == ""
