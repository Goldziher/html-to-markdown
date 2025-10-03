"""Performance tests for HOCR processing."""

import contextlib
import os
import time

import psutil
import pytest
from bs4 import BeautifulSoup

from html_to_markdown import convert_to_markdown
from html_to_markdown.hocr_processor import HOCRProcessor


def create_large_hocr_document(num_words: int = 10000) -> str:
    """Create a large HOCR document for performance testing."""
    header = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
    "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
<head>
    <meta name='ocr-system' content='tesseract 5.5.1' />
    <meta name='ocr-capabilities' content='ocr_page ocr_carea ocr_par ocr_line ocrx_word'/>
</head>
<body>
    <div class='ocr_page' id='page_1'>
        <div class='ocr_carea' id='block_1_1'>
            <p class='ocr_par' id='par_1_1'>"""

    footer = """
            </p>
        </div>
    </div>
</body>
</html>"""

    words = []
    for i in range(num_words):
        words.append(f'<span class="ocrx_word" id="word_1_{i + 1}">word{i + 1}</span>')
        if i < num_words - 1:
            words.append(" ")

    return header + "".join(words) + footer


def test_hocr_detection_performance() -> None:
    """Test HOCR detection performance on large documents."""
    sizes = [1000, 5000, 10000]

    for size in sizes:
        large_doc = create_large_hocr_document(size)

        start_time = time.perf_counter()
        is_hocr = HOCRProcessor.is_hocr_document(large_doc)
        detection_time = time.perf_counter() - start_time

        assert is_hocr, f"Should detect HOCR in {size}-word document"
        assert detection_time < 0.1, f"Detection took {detection_time:.3f}s for {size} words (should be <0.1s)"


def test_hocr_conversion_performance() -> None:
    """Test HOCR conversion performance on large documents."""
    medium_doc = create_large_hocr_document(1000)

    start_time = time.perf_counter()
    result = convert_to_markdown(medium_doc)
    conversion_time = time.perf_counter() - start_time

    assert isinstance(result, str), "Should return string result"
    assert len(result) > 1000, "Should have substantial content"
    assert conversion_time < 1.0, f"Conversion took {conversion_time:.3f}s (should be <1.0s)"

    assert "word1" in result, "Should contain first word"
    assert "word1000" in result, "Should contain last word"
    assert "word1 word2" in result, "Should have proper spacing"


def test_hocr_spacing_algorithm_performance() -> None:
    """Test that spacing algorithm is O(1) not O(n²)."""

    many_words_doc = create_large_hocr_document(5000)
    soup = BeautifulSoup(many_words_doc, "xml")

    word_spans = soup.find_all("span", class_="ocrx_word")
    assert len(word_spans) >= 1000, "Should have many word spans"

    children = list(soup.find("p").children)

    start_time = time.perf_counter()
    for i in range(min(1000, len(children))):
        if i > 0:
            HOCRProcessor.should_add_space_before_word(children, i)
    spacing_time = time.perf_counter() - start_time

    assert spacing_time < 0.01, f"Spacing decisions took {spacing_time:.3f}s (should be <0.01s)"


def test_memory_usage_with_large_hocr() -> None:
    """Test memory usage doesn't grow excessively with large HOCR."""

    process = psutil.Process(os.getpid())
    initial_memory = process.memory_info().rss / 1024 / 1024

    for _ in range(3):
        large_doc = create_large_hocr_document(2000)
        result = convert_to_markdown(large_doc)
        assert len(result) > 1000, "Should produce output"

    final_memory = process.memory_info().rss / 1024 / 1024
    memory_increase = final_memory - initial_memory

    assert memory_increase < 50, f"Memory usage increased by {memory_increase:.1f}MB"


@pytest.mark.parametrize("word_count", [100, 500, 1000, 2000])
def test_scaling_performance(word_count: int) -> None:
    """Test that performance scales reasonably with document size."""
    doc = create_large_hocr_document(word_count)

    start_time = time.perf_counter()
    result = convert_to_markdown(doc)
    conversion_time = time.perf_counter() - start_time

    assert isinstance(result, str), "Should return string"

    max_time = word_count / 5000
    assert conversion_time < max_time, f"Took {conversion_time:.3f}s for {word_count} words"


def test_malformed_hocr_handling() -> None:
    """Test handling of malformed HOCR doesn't cause performance issues."""
    malformed_cases = [
        '<div class="ocr_page"><span class="ocrx_word">test',
        '<div class="ocr_page">' + "<div>" * 100 + "content" + "</div>" * 100 + "</div>",
        f'<span class="ocrx_word {"x" * 10000}">test</span>',
    ]

    for malformed in malformed_cases:
        start_time = time.perf_counter()
        with contextlib.suppress(Exception):
            convert_to_markdown(malformed)
        processing_time = time.perf_counter() - start_time

        assert processing_time < 1.0, f"Malformed HOCR took {processing_time:.3f}s to process"


def test_large_document_size_limit() -> None:
    """Test that extremely large documents are rejected safely."""
    huge_content = "x" * (10_000_001)
    with pytest.raises(ValueError, match="Document too large"):
        HOCRProcessor.is_hocr_document(huge_content)
