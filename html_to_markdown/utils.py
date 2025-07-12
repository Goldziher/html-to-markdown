from __future__ import annotations

import re

from html_to_markdown.constants import line_beginning_re


def chomp(text: str) -> tuple[str, str, str]:
    """Simplified whitespace handling for inline elements.

    For semantic markdown output, preserves leading/trailing spaces as single spaces
    and normalizes internal whitespace.

    Args:
        text: The text to chomp.

    Returns:
        A tuple containing the prefix, suffix, and the normalized text.
    """
    if not text:
        return "", "", ""

    prefix = " " if text.startswith((" ", "\t")) else ""
    suffix = " " if text.endswith((" ", "\t")) else ""

    text = text.strip()

    return prefix, suffix, text


def escape(*, text: str, escape_misc: bool, escape_asterisks: bool, escape_underscores: bool) -> str:
    """Escape special characters in text.

    Args:
        text: The text to escape.
        escape_misc: Whether to escape miscellaneous characters.
        escape_asterisks: Whether to escape asterisks.
        escape_underscores: Whether to escape underscores.

    Returns:
        The escaped text.
    """
    if not text:
        return ""
    if escape_misc:
        text = re.sub(r"([\\&<`[>~#=+|-])", r"\\\1", text)
        text = re.sub(r"([0-9])([.)])", r"\1\\\2", text)
    if escape_asterisks:
        text = text.replace("*", r"\*")
    if escape_underscores:
        text = text.replace("_", r"\_")
    return text


def indent(*, text: str, level: int) -> str:
    """Indent text by a given level.

    Args:
        text: The text to indent.
        level: The level of indentation.

    Returns:
        The indented text.
    """
    return line_beginning_re.sub("\t" * level, text) if text else ""


def underline(*, text: str, pad_char: str) -> str:
    """Underline text with a given character.

    Args:
        text: The text to underline.
        pad_char: The character to use for underlining.

    Returns:
        The underlined text.
    """
    text = (text or "").rstrip()
    return f"{text}\n{pad_char * len(text)}\n\n" if text else ""
