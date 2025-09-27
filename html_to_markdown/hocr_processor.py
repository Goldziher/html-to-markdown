"""HOCR (HTML-based OCR) document processing utilities.

This module handles the conversion of HOCR documents to clean markdown text,
including proper spacing, layout preservation, and metadata suppression.
"""

from __future__ import annotations

import re
from typing import TYPE_CHECKING, ClassVar

from bs4 import Tag
from bs4.element import NavigableString, PageElement

if TYPE_CHECKING:
    from bs4 import BeautifulSoup


class HOCRProcessor:
    """Handles HOCR-specific document processing."""

    _HOCR_PATTERNS: ClassVar[list[re.Pattern[str]]] = [
        re.compile(r'class\s*=\s*["\'].*?ocr_page.*?["\']', re.IGNORECASE),
        re.compile(r'class\s*=\s*["\'].*?ocrx_word.*?["\']', re.IGNORECASE),
        re.compile(r'name\s*=\s*["\']ocr-system["\']', re.IGNORECASE),
        re.compile(r'class\s*=\s*["\'].*?ocr_carea.*?["\']', re.IGNORECASE),
        re.compile(r'class\s*=\s*["\'].*?ocr_par.*?["\']', re.IGNORECASE),
        re.compile(r'class\s*=\s*["\'].*?ocr_line.*?["\']', re.IGNORECASE),
    ]

    @classmethod
    def is_hocr_document(cls, content: str) -> bool:
        """Check if content is an HOCR document.

        Args:
            content: Raw HTML/XML content to check

        Returns:
            True if content appears to be HOCR format

        Raises:
            ValueError: If content is too large (>10MB)
        """
        if len(content) > 10_000_000:
            raise ValueError("Document too large for HOCR processing")

        content_sample = content[:50000]

        return any(pattern.search(content_sample) for pattern in cls._HOCR_PATTERNS)

    @classmethod
    def is_hocr_word_element(cls, tag: Tag | None) -> bool:
        """Check if a tag is an HOCR word element.

        Args:
            tag: BeautifulSoup tag to check

        Returns:
            True if tag is a span with ocrx_word class
        """
        if not tag or tag.name != "span":
            return False

        class_attr = tag.get("class")
        if isinstance(class_attr, list):
            return "ocrx_word" in class_attr
        return class_attr == "ocrx_word"

    @classmethod
    def should_add_space_before_word(cls, children: list[PageElement], current_index: int) -> bool:
        """Determine if space should be added before an HOCR word.

        Args:
            children: List of child elements
            current_index: Index of current element

        Returns:
            True if a space should be added before this word
        """
        if not (0 < current_index < len(children)):
            return False

        prev_element = children[current_index - 1]

        if isinstance(prev_element, NavigableString):
            text_content = str(prev_element)
            return not (text_content.strip() or " " in text_content)

        return isinstance(prev_element, Tag) and cls.is_hocr_word_element(prev_element)

    @classmethod
    def is_hocr_element_in_soup(cls, soup: BeautifulSoup) -> bool:
        """Check if parsed soup contains HOCR elements.

        Args:
            soup: Parsed BeautifulSoup document

        Returns:
            True if soup contains HOCR elements
        """
        return bool(
            soup.find("meta", attrs={"name": "ocr-system"})
            or soup.find("meta", attrs={"name": "ocr-capabilities"})
            or soup.find(class_="ocr_page")
            or soup.find(class_="ocrx_word")
            or soup.find(class_="ocr_carea")
            or soup.find(class_="ocr_par")
            or soup.find(class_="ocr_line")
        )

    @classmethod
    def get_optimal_parser(cls, content: str, lxml_available: bool) -> str:
        """Get optimal parser for HOCR content.

        Args:
            content: Document content
            lxml_available: Whether lxml is available

        Returns:
            Parser name to use ('xml', 'lxml', or 'html.parser')
        """
        try:
            if cls.is_hocr_document(content) and lxml_available:
                return "xml"
        except ValueError:
            pass

        return "lxml" if lxml_available else "html.parser"
