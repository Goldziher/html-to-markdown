from __future__ import annotations

import re

from html_to_markdown.constants import line_beginning_re


def chomp(text: str) -> tuple[str, str, str]:
    if not text:
        return "", "", ""

    prefix = " " if text.startswith((" ", "\t")) else ""
    suffix = " " if text.endswith((" ", "\t")) else ""

    return prefix, suffix, text.strip()


def escape(*, text: str, escape_misc: bool, escape_asterisks: bool, escape_underscores: bool) -> str:
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


def indent(*, text: str, level: int, indent_str: str = "\t") -> str:
    return line_beginning_re.sub(indent_str * level, text) if text else ""


def underline(*, text: str, pad_char: str) -> str:
    text = (text or "").rstrip()
    return f"{text}\n{pad_char * len(text)}\n\n" if text else ""
