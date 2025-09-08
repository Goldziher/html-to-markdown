"""Whitespace handling module for HTML to Markdown conversion."""

from __future__ import annotations

import re
import unicodedata
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from bs4 import NavigableString, PageElement, Tag


WhitespaceMode = Literal["normalized", "strict"]


# Block-level elements that should have spacing around them
BLOCK_ELEMENTS = {
    "address",
    "article",
    "aside",
    "blockquote",
    "canvas",
    "datalist",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "legend",
    "li",
    "main",
    "nav",
    "noscript",
    "ol",
    "option",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tfoot",
    "ul",
}

# Elements that preserve whitespace exactly (only pre in normalized mode)
PRESERVE_WHITESPACE_ELEMENTS = {"pre", "script", "style"}

# Inline elements that should not add extra spacing
INLINE_ELEMENTS = {
    "a",
    "abbr",
    "acronym",
    "audio",
    "b",
    "bdi",
    "bdo",
    "big",
    "br",
    "button",
    "cite",
    "code",
    "data",
    "dfn",
    "dialog",
    "em",
    "i",
    "iframe",
    "img",
    "input",
    "kbd",
    "label",
    "map",
    "math",
    "menu",
    "meter",
    "object",
    "output",
    "progress",
    "q",
    "rb",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "samp",
    "script",
    "select",
    "small",
    "span",
    "strong",
    "style",
    "sub",
    "sup",
    "svg",
    "textarea",
    "time",
    "tt",
    "u",
    "var",
    "video",
    "del",
    "ins",
    "mark",
    "s",
    "strike",
    "wbr",
}


class WhitespaceHandler:
    """Handles whitespace processing for HTML to Markdown conversion.

    Args:
        mode: The whitespace preservation mode to use ("normalized" or "strict").
    """

    def __init__(self, mode: WhitespaceMode = "normalized") -> None:
        self.mode = mode
        # Compile regex patterns for performance
        self._multiple_spaces = re.compile(r"[ \t]+")
        self._multiple_newlines = re.compile(r"\n{2,}")
        self._leading_trailing_space = re.compile(r"^[ \t]+|[ \t]+$", re.MULTILINE)
        # Unicode whitespace categories
        self._unicode_spaces = re.compile(r"[\u00A0\u1680\u2000-\u200A\u202F\u205F\u3000]")

    def normalize_unicode_spaces(self, text: str) -> str:
        """Normalize all Unicode whitespace characters to ASCII spaces.

        Args:
            text: The text to normalize.

        Returns:
            Text with normalized whitespace.
        """
        # Replace non-breaking spaces and other Unicode spaces with regular spaces
        text = self._unicode_spaces.sub(" ", text)

        # Also normalize other whitespace characters
        normalized = []
        for char in text:
            if unicodedata.category(char) in ("Zs", "Zl", "Zp"):
                normalized.append(" ")
            elif char in ("\r\n", "\r"):
                normalized.append("\n")
            else:
                normalized.append(char)

        return "".join(normalized)

    def should_preserve_whitespace(self, element: PageElement) -> bool:
        """Check if whitespace should be preserved for an element.

        Args:
            element: The element to check.

        Returns:
            True if whitespace should be preserved.
        """
        if self.mode == "strict":
            return True

        # Check if element or any ancestor is a whitespace-preserving element
        current: PageElement | None = element
        while current:
            if hasattr(current, "name") and current.name in PRESERVE_WHITESPACE_ELEMENTS:
                return True
            current = getattr(current, "parent", None)

        return False

    def is_block_element(self, element: PageElement | None) -> bool:
        """Check if an element is a block-level element.

        Args:
            element: The element to check.

        Returns:
            True if the element is block-level.
        """
        if not element or not hasattr(element, "name"):
            return False
        return element.name in BLOCK_ELEMENTS

    def is_inline_element(self, element: PageElement | None) -> bool:
        """Check if an element is an inline element.

        Args:
            element: The element to check.

        Returns:
            True if the element is inline.
        """
        if not element or not hasattr(element, "name"):
            return False
        return element.name in INLINE_ELEMENTS

    def process_text_whitespace(
        self,
        text: str,
        element: NavigableString,
        *,
        in_pre: bool = False,
    ) -> str:
        """Process whitespace in text content based on context.

        Args:
            text: The text to process.
            element: The NavigableString element containing the text.
            in_pre: Whether the text is inside a pre element.

        Returns:
            The processed text with appropriate whitespace handling.
        """
        if not text:
            return ""

        # Normalize Unicode spaces first
        text = self.normalize_unicode_spaces(text)

        # If in a whitespace-preserving element, return as-is
        if in_pre or self.should_preserve_whitespace(element):
            return text

        if self.mode == "strict":
            # Preserve everything except normalize Unicode
            return text
        # normalized mode
        return self._process_normalized(text, element)

    def _process_normalized(self, text: str, element: NavigableString) -> str:
        """Process text with normalized whitespace preservation.

        Args:
            text: The text to process.
            element: The NavigableString element.

        Returns:
            Processed text with normalized whitespace.
        """
        # If only whitespace
        if not text.strip():
            return self._process_whitespace_only(text, element)

        return self._process_text_with_content(text, element)

    def _process_whitespace_only(self, text: str, element: NavigableString) -> str:
        """Process text that contains only whitespace."""
        prev_sibling = element.previous_sibling
        next_sibling = element.next_sibling

        # Between block elements, remove
        if self.is_block_element(prev_sibling) and self.is_block_element(next_sibling):
            return ""

        # If it contains newlines, it's likely formatting whitespace (indentation)
        if "\n" in text:
            return ""

        # Preserve space when it's between or adjacent to inline elements
        if self.is_inline_element(prev_sibling) or self.is_inline_element(next_sibling):
            return " "

        # Otherwise remove
        return ""

    def _process_text_with_content(self, text: str, element: NavigableString) -> str:
        """Process text that contains actual content (not just whitespace)."""
        original = str(element)

        # Collapse multiple internal spaces to single space
        # But first preserve info about leading/trailing spaces
        has_lead_space = original and original[0] in " \t\n"
        has_trail_space = original and original[-1] in " \t\n"

        # Strip and collapse internal whitespace
        text = self._multiple_spaces.sub(" ", text.strip())

        parent = element.parent

        # Special handling for certain inline containers
        if parent and hasattr(parent, "name") and parent.name in {"ruby", "select", "datalist"}:
            return self._process_special_inline_containers(text, original)

        # For text inside inline elements, preserve meaningful spaces
        if parent and self.is_inline_element(parent):
            return self._process_inline_element_text(text, original, bool(has_lead_space), bool(has_trail_space))

        # For text between elements or standalone text
        return self._process_standalone_text(text, original, element, bool(has_lead_space), bool(has_trail_space))

    def _process_special_inline_containers(self, text: str, original: str) -> str:
        """Process text inside special inline containers like ruby, select, datalist."""
        # Inside these elements, only preserve actual spaces, not formatting whitespace
        # If the original had only spaces (no newlines/tabs), preserve them
        if original and "\n" not in original and "\t" not in original:
            # Preserve leading/trailing spaces if they exist
            if original[0] == " ":
                text = " " + text
            if original[-1] == " ":
                text = text + " "
        return text

    def _process_inline_element_text(
        self, text: str, original: str, has_lead_space: bool, has_trail_space: bool
    ) -> str:
        """Process text inside inline elements."""
        # Inside inline elements, preserve leading/trailing spaces if they're actual spaces
        if has_lead_space and original[0] == " ":
            text = " " + text
        if has_trail_space and original[-1] == " ":
            text = text + " "
        return text

    def _process_standalone_text(
        self, text: str, original: str, element: NavigableString, has_lead_space: bool, has_trail_space: bool
    ) -> str:
        """Process standalone text or text between elements."""
        prev_sibling = element.previous_sibling
        next_sibling = element.next_sibling

        # Preserve space when meaningful
        has_leading = (
            has_lead_space
            and original[0] == " "  # Only space, not newline/tab
            and (
                self.is_inline_element(prev_sibling) or self.is_block_element(prev_sibling) or prev_sibling is None
            )  # Include blocks too
        )
        has_trailing = (
            has_trail_space
            and original[-1] == " "  # Only space, not newline/tab
            and (
                self.is_inline_element(next_sibling) or self.is_block_element(next_sibling) or next_sibling is None
            )  # Include blocks too
        )

        # Convert newlines to spaces when they separate meaningful content
        # But only if there's an inline element before/after
        if original and original[0] in "\n\t" and self.is_inline_element(prev_sibling):
            # Newline after inline element should become space
            text = " " + text
        elif original and original[0] in "\n\t":
            has_leading = False

        if original and original[-1] in "\n\t" and self.is_inline_element(next_sibling):
            # Newline before inline element should become space
            text = text + " "
        elif original and original[-1] in "\n\t":
            has_trailing = False

        # Apply remaining leading/trailing spaces
        if has_leading and not (original and original[0] in "\n\t"):
            text = " " + text
        if has_trailing and not (original and original[-1] in "\n\t"):
            text = text + " "

        return text

    def get_block_spacing(self, tag: Tag, next_sibling: PageElement | None = None) -> str:
        """Get appropriate spacing after a block element.

        Args:
            tag: The block element tag.
            next_sibling: The next sibling element.

        Returns:
            The spacing to add after the block element.
        """
        if self.mode == "strict":
            # Preserve original spacing
            return ""  # Original spacing is preserved in text nodes

        # Normalized mode - smart spacing
        tag_name = tag.name.lower() if hasattr(tag, "name") else ""

        # These elements typically need double newline after them
        double_newline_elements = {"p", "div", "blockquote", "pre", "table", "ul", "ol", "dl"}

        # These elements typically need single newline
        single_newline_elements = {"li", "dt", "dd", "tr", "td", "th"}

        if tag_name in double_newline_elements:
            # Check if next sibling is also a block that needs spacing
            if self.is_block_element(next_sibling):
                return "\n\n"
            return "\n"
        if tag_name in single_newline_elements:
            return "\n"
        if tag_name.startswith("h") and len(tag_name) == 2:  # h1-h6
            return "\n\n"

        return ""
