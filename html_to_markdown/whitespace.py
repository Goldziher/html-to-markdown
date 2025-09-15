"""Whitespace handling module for HTML to Markdown conversion."""

from __future__ import annotations

import re
import unicodedata
from typing import TYPE_CHECKING, Literal

from bs4.element import NavigableString

if TYPE_CHECKING:
    from bs4 import PageElement


WhitespaceMode = Literal["normalized", "strict"]


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

PRESERVE_WHITESPACE_ELEMENTS = {"pre", "script", "style"}

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
    def __init__(self, mode: WhitespaceMode = "normalized") -> None:
        self.mode = mode
        self._multiple_spaces = re.compile(r"[ \t]+")
        self._multiple_newlines = re.compile(r"\n{2,}")
        self._leading_trailing_space = re.compile(r"^[ \t]+|[ \t]+$", re.MULTILINE)
        self._unicode_spaces = re.compile(r"[\u00A0\u1680\u2000-\u200A\u202F\u205F\u3000]")

    def normalize_unicode_spaces(self, text: str) -> str:
        text = self._unicode_spaces.sub(" ", text)

        text = text.replace("\r\n", "\n")

        normalized = []
        for char in text:
            if unicodedata.category(char) in ("Zs", "Zl", "Zp"):
                normalized.append(" ")
            elif char == "\r":  # pragma: no cover
                normalized.append("\n")
            else:
                normalized.append(char)

        return "".join(normalized)

    def should_preserve_whitespace(self, element: PageElement) -> bool:
        if self.mode == "strict":
            return True

        current: PageElement | None = element
        while current:
            if hasattr(current, "name") and current.name in PRESERVE_WHITESPACE_ELEMENTS:
                return True
            current = getattr(current, "parent", None)

        return False

    def is_block_element(self, element: PageElement | None) -> bool:
        if not element or not hasattr(element, "name"):
            return False
        return element.name in BLOCK_ELEMENTS

    def is_inline_element(self, element: PageElement | None) -> bool:
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
        if not text:  # pragma: no cover
            return ""

        if in_pre or self.should_preserve_whitespace(element):
            return text

        text = self.normalize_unicode_spaces(text)
        return self._process_normalized(text, element)

    def _process_normalized(self, text: str, element: NavigableString) -> str:
        if not text.strip():
            return self._process_whitespace_only(text, element)

        return self._process_text_with_content(text, element)

    def _process_whitespace_only(self, text: str, element: NavigableString) -> str:
        prev_sibling = element.previous_sibling
        next_sibling = element.next_sibling

        if self.is_block_element(prev_sibling) and self.is_block_element(next_sibling):
            return ""

        if "\n" in text:
            return ""

        if self.is_inline_element(prev_sibling) or self.is_inline_element(next_sibling):
            return " "

        return ""

    def _process_text_with_content(self, text: str, element: NavigableString) -> str:
        original = str(element)

        has_lead_space = bool(original and original[0] in " \t\n")
        has_trail_space = bool(original and original[-1] in " \t\n")

        text = self._multiple_spaces.sub(" ", text.strip())

        parent = element.parent

        if parent and hasattr(parent, "name") and parent.name in {"ruby", "select", "datalist"}:
            return self._process_special_inline_containers(text, original)

        if parent and self.is_inline_element(parent):
            return self._process_inline_element_text(text, original, has_lead_space, has_trail_space)

        return self._process_standalone_text(text, original, element, has_lead_space, has_trail_space)

    def _process_special_inline_containers(self, text: str, original: str) -> str:
        if original and "\n" not in original and "\t" not in original:
            if original[0] == " ":
                text = " " + text
            if original[-1] == " ":
                text = text + " "
        return text

    def _process_inline_element_text(
        self, text: str, original: str, has_lead_space: bool, has_trail_space: bool
    ) -> str:
        if has_lead_space and original[0] == " ":
            text = " " + text
        if has_trail_space and original[-1] == " ":
            text = text + " "
        return text

    def _process_standalone_text(
        self, text: str, original: str, element: NavigableString, has_lead_space: bool, has_trail_space: bool
    ) -> str:
        prev_sibling = element.previous_sibling
        next_sibling = element.next_sibling

        multiple_newlines_before_block = (
            original
            and original.count("\n") >= 2
            and self.is_block_element(next_sibling)
            and text.strip()
            and (self.is_inline_element(prev_sibling) or prev_sibling is None)
        )

        has_leading = (
            has_lead_space
            and original[0] == " "
            and (
                self.is_inline_element(prev_sibling)
                or self.is_block_element(prev_sibling)
                or prev_sibling is None
                or isinstance(prev_sibling, NavigableString)
            )
        )
        has_trailing = (
            has_trail_space
            and original[-1] == " "
            and (
                self.is_inline_element(next_sibling)
                or self.is_block_element(next_sibling)
                or next_sibling is None
                or isinstance(next_sibling, NavigableString)
            )
        )

        if original and original[0] in "\n\t" and self.is_inline_element(prev_sibling):
            text = " " + text
        elif original and original[0] in "\n\t":
            has_leading = False

        if original and original[-1] in "\n\t" and self.is_inline_element(next_sibling):
            text = text + " "
        elif original and original[-1] in "\n\t":
            has_trailing = False

        if has_leading and not (original and original[0] in "\n\t"):
            text = " " + text
        if has_trailing and not (original and original[-1] in "\n\t"):
            text = text + " "

        if multiple_newlines_before_block:
            text = text + "\n\n"

        return text
