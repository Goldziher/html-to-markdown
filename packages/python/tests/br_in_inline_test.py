from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def test_br_inside_bold_tags(convert: Callable[..., str]) -> None:
    html = "<b>Hello!<br/></b><b>Hola!</b>"
    result = convert(html)

    assert "**Hello!**  \n**Hola!**" in result
    assert "**Hello!****Hola!**" not in result


def test_br_inside_strong_tags(convert: Callable[..., str]) -> None:
    html = "<strong>First<br/></strong><strong>Second</strong>"
    result = convert(html)

    assert "**First**  \n**Second**" in result
    assert "**First****Second**" not in result


def test_multiple_bolds_with_br(convert: Callable[..., str]) -> None:
    html = "<b>Line 1<br/></b><b>Line 2<br/></b><b>Line 3</b>"
    result = convert(html)

    assert result.count("**") >= 6
    assert "**Line 1**" in result
    assert "**Line 2**" in result
    assert "**Line 3**" in result
    assert "**Line 1**  \n**Line 2**" in result


def test_br_inside_em_tags(convert: Callable[..., str]) -> None:
    html = "<em>First<br/></em><em>Second</em>"
    result = convert(html)

    assert "*First*  \n*Second*" in result
    assert "*First**Second*" not in result


def test_br_inside_italic_tags(convert: Callable[..., str]) -> None:
    html = "<i>Alpha<br/></i><i>Beta</i>"
    result = convert(html)

    assert "*Alpha*  \n*Beta*" in result
    assert "*Alpha**Beta*" not in result


def test_br_with_backslash_style(convert: Callable[..., str]) -> None:
    html = "<b>Hello<br/></b><b>World</b>"
    result = convert(html, newline_style="backslash")

    assert "**Hello**\\\n**World**" in result
    assert "**Hello****World**" not in result


def test_br_inside_nested_formatting(convert: Callable[..., str]) -> None:
    html = "<b><i>Bold italic<br/></i></b><b>Just bold</b>"
    result = convert(html)

    assert "***Bold italic***  \n**Just bold**" in result


def test_br_at_end_of_paragraph_with_bold(convert: Callable[..., str]) -> None:
    html = "<p><b>Line 1<br/></b><b>Line 2</b></p>"
    result = convert(html)

    assert "**Line 1**  \n**Line 2**" in result
