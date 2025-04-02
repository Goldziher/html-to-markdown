from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from bs4.element import Tag

from html_to_markdown import convert_to_markdown


def test_custom_converters() -> None:
    """Test custom converters functionality."""

    def custom_b_converter(*, tag: Tag, text: str, **kwargs: Any) -> str:
        return text.upper()

    def custom_em_converter(*, tag: Tag, text: str, **kwargs: Any) -> str:
        return f"::{text}::"

    html = "<p>This is <b>bold</b> and <em>emphasized</em> text.</p>"

    markdown = convert_to_markdown(
        html,
        custom_converters={
            "b": lambda text, tag: custom_b_converter(tag=tag, text=text),
            "em": lambda text, tag: custom_em_converter(tag=tag, text=text),
        },
    )

    assert "BOLD" in markdown
    assert "**bold**" not in markdown
    assert "::emphasized::" in markdown
    assert "*emphasized*" not in markdown


def test_custom_converters_precedence() -> None:
    """Test that custom converters take precedence over default converters."""

    def custom_h1_converter(*, tag: Tag, text: str, **kwargs: Any) -> str:
        return f"CUSTOM_HEADING: {text}\n\n"

    html = "<h1>Hello World</h1>"

    markdown = convert_to_markdown(
        html,
        custom_converters={
            "h1": lambda text, tag: custom_h1_converter(tag=tag, text=text),
        },
    )

    assert "CUSTOM_HEADING: Hello World" in markdown
    assert "Hello World\n=====" not in markdown


def test_custom_converters_with_other_options() -> None:
    """Test that custom converters work alongside other conversion options."""

    def custom_code_converter(*, tag: Tag, text: str, **kwargs: Any) -> str:
        return f"`python:{text}`"

    html = '<p>Some <code>print("Hello")</code> code.</p>'

    markdown = convert_to_markdown(
        html,
        custom_converters={
            "code": lambda text, tag: custom_code_converter(tag=tag, text=text),
        },
        strong_em_symbol="_",
        wrap=True,
        wrap_width=20,
    )

    assert '`python:print("Hello")`' in markdown
