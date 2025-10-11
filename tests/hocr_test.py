"""Tests for HOCR (HTML-based OCR) format conversion.

HOCR is a standard format used by OCR software like Tesseract to output
structured text with positioning and confidence information.
"""

from pathlib import Path

import pytest

from html_to_markdown import convert


def get_hocr_file(filename: str) -> Path:
    return Path(__file__).parent / "test_data" / "hocr" / filename


def get_content_without_frontmatter(markdown: str) -> str:
    """Extract markdown content, excluding YAML frontmatter."""
    if markdown.startswith("---\n"):
        parts = markdown.split("---\n", 2)
        return parts[2] if len(parts) > 2 else markdown
    return markdown


def test_german_pdf_hocr_conversion() -> None:
    hocr_content = get_hocr_file("german_pdf_german.hocr").read_text(encoding="utf-8")

    result = convert(hocr_content)

    assert "<!--" not in result, "Result should not contain HTML comments"
    assert "meta-content-type" not in result, "Result should not contain meta tags"
    assert "meta-ocr-capabilities" not in result, "Result should not contain OCR meta tags"

    assert "DR Heimat Bayern" in result, "Should contain German text from document header"
    assert "Bayerischer Landesverein" in result, "Should contain organization name"
    assert "München" in result, "Should contain Munich city name"
    assert "Archivgesetz" in result, "Should contain law reference"

    lines = [line.strip() for line in result.split("\n") if line.strip()]
    assert len(lines) > 10, "Should have multiple lines of content"

    meaningful_lines = [line for line in lines if not line.startswith("#") and len(line) > 5]
    assert len(meaningful_lines) > 0, "Should have meaningful content lines"

    first_line = meaningful_lines[0] if meaningful_lines else ""
    assert not first_line.startswith("meta-"), "First line should not be meta information"


def test_english_pdf_hocr_conversion() -> None:
    hocr_content = get_hocr_file("english_pdf_default.hocr").read_text(encoding="utf-8")

    result = convert(hocr_content)

    assert "<!--" not in result, "Result should not contain HTML comments"
    assert "meta-ocr-system" not in result, "Result should not contain OCR system info"

    assert len(result.strip()) > 50, "Should have substantial content"


def test_invoice_hocr_conversion() -> None:
    hocr_content = get_hocr_file("invoice_image_default.hocr").read_text(encoding="utf-8")

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert "<!--" not in result, "Result should not contain HTML comments"
    assert "ocr_page" not in content, "Content should not contain HOCR class names"
    assert "bbox" not in content, "Content should not contain bounding box info"

    assert len(result.strip()) > 10, "Should have some content"


def test_hocr_with_confidence_and_coordinates() -> None:
    hocr_content = get_hocr_file("german_pdf_german.hocr").read_text(encoding="utf-8")

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert "x_wconf" not in content, "Content should not contain confidence scores"
    assert "bbox" not in content, "Content should not contain bounding boxes"
    assert "baseline" not in content, "Content should not contain baseline info"
    assert "x_size" not in content, "Content should not contain size info"
    assert "ppageno" not in content, "Content should not contain page number info"


def test_hocr_preserves_text_structure() -> None:
    hocr_content = get_hocr_file("german_pdf_german.hocr").read_text(encoding="utf-8")

    result = convert(hocr_content)

    lines = [line.strip() for line in result.split("\n") if line.strip()]
    assert len(lines) > 5, "Should preserve multiple text blocks"

    blank_line_ratio = result.count("\n\n\n") / max(1, result.count("\n"))
    assert blank_line_ratio < 0.3, "Should not have too many consecutive blank lines"


def test_empty_hocr_handling() -> None:
    minimal_hocr = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
 <head>
  <meta name='ocr-system' content='tesseract 5.5.1' />
 </head>
 <body>
  <div class='ocr_page' id='page_1'>
  </div>
 </body>
</html>"""

    result = convert(minimal_hocr)

    assert isinstance(result, str), "Should return string even for empty HOCR"
    assert "meta" not in result, "Should not contain meta information"


@pytest.mark.parametrize(
    "hocr_file",
    [
        "german_pdf_german.hocr",
        "english_pdf_default.hocr",
        "invoice_image_default.hocr",
    ],
)
def test_all_hocr_files_convert_cleanly(hocr_file: str) -> None:
    hocr_content = get_hocr_file(hocr_file).read_text(encoding="utf-8")

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert isinstance(result, str), "Should return string"
    assert "<?xml" not in result, "Should not contain XML declaration"
    assert "<!DOCTYPE" not in result, "Should not contain DOCTYPE"
    assert "<html" not in result, "Should not contain HTML tags"
    assert "ocr_" not in content, "Content should not contain HOCR class names"


def test_multilingual_hocr_conversion() -> None:
    hocr_content = (Path(__file__).parent / "test_data" / "hocr" / "comprehensive" / "valid_file.hocr").read_text(
        encoding="utf-8"
    )

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert "<!--" not in result, "Should not contain HTML comments"
    assert "<?xml" not in result, "Should not contain XML declaration"
    assert "ocr_" not in content, "Content should not contain HOCR class names"
    assert "bbox" not in content, "Content should not contain bounding box info"

    assert "The (quick)" in result, "Should contain English text with proper spacing"
    assert "[brown]" in result or "\\[brown]" in result, "Should contain bracketed text"
    assert "{fox} jumps!" in result, "Should contain braced text"
    assert "Der ,.schnelle" in result, "Should contain German text"
    assert "Le renard brun" in result, "Should contain French text"
    assert "La volpe marrone" in result, "Should contain Italian text"
    assert "$43,456" in result, "Should preserve numbers"
    assert "aspammer@website.com" in result, "Should preserve email addresses"


def test_utf8_encoding_hocr() -> None:
    hocr_content = (Path(__file__).parent / "test_data" / "hocr" / "comprehensive" / "utf8_encoding.hocr").read_text(
        encoding="utf-8"
    )

    result = convert(hocr_content)

    assert "fööbär" in result, "Should preserve UTF-8 special characters"


def test_overlapping_bbox_hocr() -> None:
    hocr_content = (Path(__file__).parent / "test_data" / "hocr" / "comprehensive" / "bbox_overlapping.hocr").read_text(
        encoding="utf-8"
    )

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert isinstance(result, str), "Should return string"
    assert "bbox" not in content, "Content should not contain bounding box information"
    assert "<!--" not in result, "Should not contain HTML comments"


@pytest.mark.parametrize(
    "comprehensive_file",
    [
        "valid_file.hocr",
        "with_body_tag.hocr",
        "utf8_encoding.hocr",
        "word_confidence.hocr",
        "bbox_overlapping.hocr",
    ],
)
def test_comprehensive_hocr_files(comprehensive_file: str) -> None:
    hocr_path = Path(__file__).parent / "test_data" / "hocr" / "comprehensive" / comprehensive_file
    hocr_content = hocr_path.read_text(encoding="utf-8")

    result = convert(hocr_content)
    content = get_content_without_frontmatter(result)

    assert isinstance(result, str), "Should return string"
    assert "<?xml" not in result, "Should not contain XML declaration"
    assert "<!DOCTYPE" not in result, "Should not contain DOCTYPE"
    assert "<html" not in result, "Should not contain HTML tags"

    assert "bbox" not in content, "Content should not contain bounding box information"
    assert "x_wconf" not in content, "Content should not contain confidence scores"
    assert "baseline" not in content, "Content should not contain baseline information"
    assert "ppageno" not in content, "Content should not contain page number information"


def test_hocr_table_extraction() -> None:
    """Test automatic table reconstruction for explicit ocr_table elements."""
    hocr_content = """
    <html>
    <body>
        <div class="ocr_page">
            <div class="ocr_table">
                <span class="ocrx_word" title="bbox 100 50 140 70; x_wconf 95">Product</span>
                <span class="ocrx_word" title="bbox 200 50 240 70; x_wconf 95">Price</span>
                <span class="ocrx_word" title="bbox 300 50 340 70; x_wconf 95">Stock</span>
                <span class="ocrx_word" title="bbox 100 100 140 120; x_wconf 95">Apple</span>
                <span class="ocrx_word" title="bbox 200 100 240 120; x_wconf 95">$1.50</span>
                <span class="ocrx_word" title="bbox 300 100 340 120; x_wconf 95">Yes</span>
                <span class="ocrx_word" title="bbox 100 150 140 170; x_wconf 95">Orange</span>
                <span class="ocrx_word" title="bbox 200 150 240 170; x_wconf 95">$2.00</span>
                <span class="ocrx_word" title="bbox 300 150 340 170; x_wconf 95">No</span>
            </div>
        </div>
    </body>
    </html>
    """

    result = convert(hocr_content)

    assert "|" in result, "Should contain table markdown"
    assert "Product" in result, "Should contain header"
    assert "Price" in result, "Should contain header"
    assert "Stock" in result, "Should contain header"
    assert "Apple" in result, "Should contain data"
    assert "$1.50" in result, "Should contain data"
    assert "Orange" in result, "Should contain data"
    assert "$2.00" in result, "Should contain data"

    assert "| ---" in result, "Should contain header separator"


def test_hocr_without_table_element() -> None:
    """Test that words without ocr_table wrapper are processed as normal text."""
    hocr_content = """
    <html>
    <body>
        <div class="ocr_page">
            <span class="ocrx_word" title="bbox 100 50 140 70; x_wconf 95">Col1</span>
            <span class="ocrx_word" title="bbox 200 50 240 70; x_wconf 95">Col2</span>
            <span class="ocrx_word" title="bbox 100 100 140 120; x_wconf 95">Data1</span>
            <span class="ocrx_word" title="bbox 200 100 240 120; x_wconf 95">Data2</span>
        </div>
    </body>
    </html>
    """

    result = convert(hocr_content)

    # Without ocr_table wrapper, words are processed as plain text
    assert "Col1" in result
    assert "Col2" in result
    assert "Data1" in result
    assert "Data2" in result
    assert "|" not in result, "Should not create table without explicit ocr_table element"


def test_hocr_word_extraction() -> None:
    """Test that words are extracted regardless of confidence scores."""
    hocr_content = """
    <html>
    <body>
        <div class="ocr_page">
            <span class="ocrx_word" title="bbox 100 50 140 70; x_wconf 95">Good</span>
            <span class="ocrx_word" title="bbox 200 50 240 70; x_wconf 30">Bad</span>
            <span class="ocrx_word" title="bbox 100 100 140 120; x_wconf 92">Quality</span>
        </div>
    </body>
    </html>
    """

    result = convert(hocr_content)

    assert "Good" in result, "High confidence word should be included"
    assert "Bad" in result, "Low confidence word should also be included"
    assert "Quality" in result, "High confidence word should be included"
