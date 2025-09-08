from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable
import base64
from collections.abc import Callable
from functools import partial
from inspect import getfullargspec
from textwrap import fill
from typing import Any, Literal, TypeVar, cast

from bs4.element import Tag

from html_to_markdown.constants import (
    ATX_CLOSED,
    BACKSLASH,
    UNDERLINED,
    line_beginning_re,
)
from html_to_markdown.utils import chomp, indent, underline


def _format_block_element(text: str) -> str:
    return f"{text.strip()}\n\n" if text.strip() else ""


def _format_inline_or_block(text: str, convert_as_inline: bool) -> str:
    return text.strip() if convert_as_inline else _format_block_element(text)


def _format_wrapped_block(text: str, start_marker: str, end_marker: str = "") -> str:
    if not end_marker:
        end_marker = start_marker
    return f"{start_marker}{text.strip()}{end_marker}\n\n" if text.strip() else ""


SupportedElements = Literal[
    "a",
    "abbr",
    "article",
    "aside",
    "audio",
    "b",
    "bdi",
    "bdo",
    "blockquote",
    "br",
    "button",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "div",
    "dl",
    "dt",
    "em",
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
    "hgroup",
    "hr",
    "i",
    "iframe",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "list",
    "main",
    "mark",
    "math",
    "menu",
    "meter",
    "nav",
    "ol",
    "li",
    "optgroup",
    "option",
    "output",
    "p",
    "picture",
    "pre",
    "progress",
    "q",
    "rb",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "s",
    "samp",
    "script",
    "section",
    "select",
    "small",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "svg",
    "table",
    "tbody",
    "td",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "tr",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
]

Converter = Callable[[str, Tag], str]
ConvertersMap = dict[SupportedElements, Converter]

T = TypeVar("T")


def _create_inline_converter(markup_prefix: str) -> Callable[[Tag, str], str]:
    def implementation(*, tag: Tag, text: str) -> str:
        from html_to_markdown.processing import _has_ancestor  # noqa: PLC0415

        if _has_ancestor(tag, ["pre", "code", "kbd", "samp"]):
            return text

        if not text.strip():
            return ""

        markup_suffix = markup_prefix
        if markup_prefix.startswith("<") and markup_prefix.endswith(">"):
            markup_suffix = "</" + markup_prefix[1:]

        prefix, suffix, text = chomp(text)
        return f"{prefix}{markup_prefix}{text}{markup_suffix}{suffix}"

    return cast("Callable[[Tag, str], str]", implementation)


def _get_colspan(tag: Tag) -> int:
    colspan = 1

    if "colspan" in tag.attrs and isinstance(tag["colspan"], str) and tag["colspan"].isdigit():
        colspan = int(tag["colspan"])

    return colspan


def _convert_a(*, tag: Tag, text: str, autolinks: bool, default_title: bool) -> str:
    prefix, suffix, text = chomp(text)
    if not text:
        return ""

    href = tag.get("href")
    title = tag.get("title")

    if autolinks and text.replace(r"\_", "_") == href and not title and not default_title:
        return f"<{href}>"

    if default_title and not title:
        title = href

    title_part = ' "{}"'.format(title.replace('"', r"\"")) if isinstance(title, str) else ""
    return f"{prefix}[{text}]({href}{title_part}){suffix}" if href else text


def _convert_blockquote(*, text: str, tag: Tag, convert_as_inline: bool, list_indent_str: str) -> str:
    if convert_as_inline:
        return text

    if not text:
        return ""

    from html_to_markdown.processing import _has_ancestor  # noqa: PLC0415

    cite_url = tag.get("cite")

    if _has_ancestor(tag, "li"):
        lines = text.strip().split("\n")
        indented_lines = [f"{list_indent_str}> {line}" if line.strip() else "" for line in lines]
        quote_text = "\n".join(indented_lines) + "\n\n"
    else:
        quote_text = f"\n{line_beginning_re.sub('> ', text.strip())}\n\n"

    if cite_url:
        if _has_ancestor(tag, "li"):
            quote_text += f"{list_indent_str}— <{cite_url}>\n\n"
        else:
            quote_text += f"— <{cite_url}>\n\n"

    return quote_text


def _convert_br(*, convert_as_inline: bool, newline_style: str, tag: Tag) -> str:
    from html_to_markdown.processing import _has_ancestor  # noqa: PLC0415

    if _has_ancestor(tag, ["h1", "h2", "h3", "h4", "h5", "h6"]):
        return " "

    _ = convert_as_inline
    return "\\\n" if newline_style.lower() == BACKSLASH else "  \n"


def _convert_hn(
    *,
    n: int,
    heading_style: Literal["atx", "atx_closed", "underlined"],
    text: str,
    convert_as_inline: bool,
) -> str:
    if convert_as_inline:
        return text

    text = text.strip()
    if heading_style == UNDERLINED and n <= 2:
        return underline(text=text, pad_char="=" if n == 1 else "-")

    hashes = "#" * n
    if heading_style == ATX_CLOSED:
        return f"{hashes} {text} {hashes}\n\n"

    return f"{hashes} {text}\n\n"


def _convert_img(*, tag: Tag, convert_as_inline: bool, keep_inline_images_in: Iterable[str] | None) -> str:
    alt = tag.attrs.get("alt", "")
    alt = alt if isinstance(alt, str) else ""
    src = tag.attrs.get("src", "")
    src = src if isinstance(src, str) else ""
    title = tag.attrs.get("title", "")
    title = title if isinstance(title, str) else ""
    width = tag.attrs.get("width", "")
    width = width if isinstance(width, str) else ""
    height = tag.attrs.get("height", "")
    height = height if isinstance(height, str) else ""
    title_part = ' "{}"'.format(title.replace('"', r"\"")) if title else ""
    parent_name = tag.parent.name if tag.parent else ""

    default_preserve_in = {"td", "th"}
    preserve_in = set(keep_inline_images_in or []) | default_preserve_in
    if convert_as_inline and parent_name not in preserve_in:
        return alt
    if width or height:
        return f"<img src='{src}' alt='{alt}' title='{title}' width='{width}' height='{height}' />"
    return f"![{alt}]({src}{title_part})"


def _convert_list(*, tag: Tag, text: str, list_indent_str: str) -> str:
    from html_to_markdown.processing import _has_ancestor  # noqa: PLC0415

    before_paragraph = False
    if tag.next_sibling and getattr(tag.next_sibling, "name", None) not in {"ul", "ol"}:
        before_paragraph = True

    if _has_ancestor(tag, "li"):
        parent = tag.parent
        while parent and parent.name != "li":
            parent = parent.parent

        if parent:
            prev_p = None
            for child in parent.children:
                if hasattr(child, "name"):
                    if child == tag:
                        break
                    if child.name == "p":
                        prev_p = child

            if prev_p:
                lines = text.strip().split("\n")
                indented_lines = []
                for line in lines:
                    if line.strip():
                        indented_lines.append(f"{list_indent_str}{line}")
                    else:
                        indented_lines.append("")
                return "\n" + "\n".join(indented_lines) + "\n"
            return "\n" + indent(text=text, level=1, indent_str=list_indent_str).rstrip()

    if tag.parent and tag.parent.name in {"ul", "ol"}:
        lines = text.strip().split("\n")
        indented_lines = []
        for line in lines:
            if line.strip():
                indented_lines.append(f"{list_indent_str}{line}")
            else:
                indented_lines.append("")
        result = "\n".join(indented_lines)
        if not result.endswith("\n"):
            result += "\n"
        return result

    return text + ("\n" if before_paragraph else "")


def _convert_li(*, tag: Tag, text: str, bullets: str, list_indent_str: str) -> str:
    checkbox = tag.find("input", {"type": "checkbox"})
    if checkbox and isinstance(checkbox, Tag):
        checked = checkbox.get("checked") is not None
        checkbox_symbol = "[x]" if checked else "[ ]"

        checkbox_text = text
        if checkbox.string:
            checkbox_text = text.replace(str(checkbox.string), "").strip()
        return f"- {checkbox_symbol} {checkbox_text.strip()}\n"

    parent = tag.parent
    if parent is not None and parent.name == "ol":
        start = (
            int(cast("str", parent["start"]))
            if isinstance(parent.get("start"), str) and str(parent.get("start")).isnumeric()
            else 1
        )
        bullet = "%s." % (start + parent.index(tag))
    else:
        depth = -1
        while tag:
            if tag.name == "ul":
                depth += 1
            if not tag.parent:
                break

            tag = tag.parent

        bullet = bullets[depth % len(bullets)]

    has_block_children = any(
        child.name in {"p", "blockquote", "pre", "ul", "ol", "div", "h1", "h2", "h3", "h4", "h5", "h6"}
        for child in tag.children
        if hasattr(child, "name")
    )

    if has_block_children:
        paragraphs = text.strip().split("\n\n")

        if paragraphs:
            result_parts = [f"{bullet} {paragraphs[0].strip()}\n"]

            for para in paragraphs[1:]:
                if para.strip():
                    result_parts.append("\n")
                    result_parts.extend(
                        f"{list_indent_str}{line}\n" for line in para.strip().split("\n") if line.strip()
                    )

            return "".join(result_parts)

    return "{} {}\n".format(bullet, (text or "").strip())


def _convert_p(
    *, wrap: bool, text: str, convert_as_inline: bool, wrap_width: int, tag: Tag, list_indent_str: str
) -> str:
    if convert_as_inline:
        return text

    if wrap:
        text = fill(
            text,
            width=wrap_width,
            break_long_words=False,
            break_on_hyphens=False,
        )

    from html_to_markdown.processing import _has_ancestor  # noqa: PLC0415

    if _has_ancestor(tag, "li"):
        parent = tag.parent
        while parent and parent.name != "li":
            parent = parent.parent

        if parent:
            p_children = [child for child in parent.children if hasattr(child, "name") and child.name == "p"]

            if p_children and tag != p_children[0]:
                indented_lines = []
                for line in text.split("\n"):
                    if line.strip():
                        indented_lines.append(f"{list_indent_str}{line}")
                    else:
                        indented_lines.append("")
                text = "\n".join(indented_lines)

    return f"{text}\n\n" if text else ""


def _convert_mark(*, text: str, convert_as_inline: bool, highlight_style: str) -> str:
    if convert_as_inline:
        return text

    match highlight_style:
        case "double-equal":
            return f"=={text}=="
        case "bold":
            return f"**{text}**"
        case "html":
            return f"<mark>{text}</mark>"
        case _:
            return text


def _convert_pre(
    *,
    tag: Tag,
    text: str,
    code_language: str,
    code_language_callback: Callable[[Tag], str] | None,
) -> str:
    if not text:
        return ""

    if code_language_callback:
        code_language = code_language_callback(tag) or code_language

    return f"\n```{code_language}\n{text}\n```\n"


def _convert_td(*, tag: Tag, text: str) -> str:
    colspan = _get_colspan(tag)
    return " " + text.strip().replace("\n", " ") + " |" * colspan


def _convert_th(*, tag: Tag, text: str) -> str:
    colspan = _get_colspan(tag)
    return " " + text.strip().replace("\n", " ") + " |" * colspan


def _convert_tr(*, tag: Tag, text: str) -> str:
    cells = tag.find_all(["td", "th"])
    parent_name = tag.parent.name if tag.parent and hasattr(tag.parent, "name") else ""
    tag_grand_parent = tag.parent.parent if tag.parent else None

    if tag.previous_sibling and hasattr(tag.previous_sibling, "name") and tag.previous_sibling.name == "tr":
        prev_cells = cast("Tag", tag.previous_sibling).find_all(["td", "th"])
        rowspan_positions = []
        col_pos = 0

        for prev_cell in prev_cells:
            rowspan = 1
            if (
                "rowspan" in prev_cell.attrs
                and isinstance(prev_cell["rowspan"], str)
                and prev_cell["rowspan"].isdigit()
            ):
                rowspan = int(prev_cell["rowspan"])

            if rowspan > 1:
                rowspan_positions.append(col_pos)

            colspan = 1
            if (
                "colspan" in prev_cell.attrs
                and isinstance(prev_cell["colspan"], str)
                and prev_cell["colspan"].isdigit()
            ):
                colspan = int(prev_cell["colspan"])
            col_pos += colspan

        if rowspan_positions:
            converted_cells: list[str] = []
            if text.strip():
                parts = text.split("|")
                converted_cells.extend(part.rstrip() + " |" for part in parts[:-1] if part)

            new_cells: list[str] = []
            cell_index = 0

            for pos in range(col_pos):
                if pos in rowspan_positions:
                    new_cells.append(" |")
                elif cell_index < len(converted_cells):
                    new_cells.append(converted_cells[cell_index])
                    cell_index += 1

            text = "".join(new_cells)

    is_headrow = (
        all(hasattr(cell, "name") and cell.name == "th" for cell in cells)
        or (not tag.previous_sibling and parent_name != "tbody")
        or (
            not tag.previous_sibling
            and parent_name == "tbody"
            and (not tag_grand_parent or len(tag_grand_parent.find_all(["thead"])) < 1)
        )
    )
    overline = ""
    underline = ""
    if is_headrow and not tag.previous_sibling:
        full_colspan = 0
        for cell in cells:
            if hasattr(cell, "attrs") and "colspan" in cell.attrs:
                colspan_value = cell.attrs["colspan"]
                if isinstance(colspan_value, str) and colspan_value.isdigit():
                    full_colspan += int(colspan_value)
                else:
                    full_colspan += 1
            else:
                full_colspan += 1
        underline += "| " + " | ".join(["---"] * full_colspan) + " |" + "\n"
    elif not tag.previous_sibling and (
        parent_name == "table" or (parent_name == "tbody" and not cast("Tag", tag.parent).previous_sibling)
    ):
        overline += "| " + " | ".join([""] * len(cells)) + " |" + "\n"
        overline += "| " + " | ".join(["---"] * len(cells)) + " |" + "\n"
    return overline + "|" + text + "\n" + underline


def _convert_caption(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_wrapped_block(text, "*")


def _convert_thead(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return text


def _convert_tbody(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return text


def _convert_tfoot(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return text


def _convert_colgroup(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag, text, convert_as_inline
    return ""


def _convert_col(*, tag: Tag, convert_as_inline: bool) -> str:
    _ = tag, convert_as_inline
    return ""


def _convert_semantic_block(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return f"{text}\n\n" if text.strip() else ""


def _convert_div(*, text: str, convert_as_inline: bool) -> str:  # noqa: ARG001
    return text


def _convert_details(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return _format_block_element(text)


def _convert_summary(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return _format_wrapped_block(text, "**")


def _convert_dl(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    return f"{text}\n" if text.strip() else ""


def _convert_dt(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f"{text.strip()}\n"


def _convert_dd(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f":   {text.strip()}\n\n"


def _convert_cite(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f"*{text.strip()}*"


def _convert_q(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    escaped_text = text.strip().replace('"', '\\"')
    return f'"{escaped_text}"'


def _convert_media_element(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    src = tag.get("src", "")

    if not src and (source_tag := tag.find("source")) and isinstance(source_tag, Tag):
        src = source_tag.get("src", "")

    if src and isinstance(src, str) and src.strip():
        link = f"[{src}]({src})"
        if convert_as_inline:
            return link
        result = f"{link}\n\n"
        if text.strip():
            result += f"{text.strip()}\n\n"
        return result

    if text.strip():
        return _format_inline_or_block(text, convert_as_inline)

    return ""


def _convert_iframe(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = text
    src = tag.get("src", "")

    if src and isinstance(src, str) and src.strip():
        link = f"[{src}]({src})"
        if convert_as_inline:
            return link
        return f"{link}\n\n"

    return ""


def _convert_abbr(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = convert_as_inline
    if not text.strip():
        return ""

    title = tag.get("title")
    if title and isinstance(title, str) and title.strip():
        return f"{text.strip()} ({title.strip()})"

    return text.strip()


def _convert_time(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    _ = convert_as_inline
    if not text.strip():
        return ""

    return text.strip()


def _convert_data(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    _ = convert_as_inline
    if not text.strip():
        return ""

    return text.strip()


def _convert_wbr(*, convert_as_inline: bool) -> str:
    _ = convert_as_inline
    return ""


def _convert_form(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return text


def _convert_fieldset(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return text


def _convert_legend(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_wrapped_block(text, "**")


def _convert_label(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if not text.strip():
        return ""

    return _format_inline_or_block(text, convert_as_inline)


def _convert_input_enhanced(*, tag: Tag, convert_as_inline: bool) -> str:
    _ = tag, convert_as_inline
    return ""


def _convert_textarea(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if not text.strip():
        return ""

    return _format_inline_or_block(text, convert_as_inline)


def _convert_select(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if not text.strip():
        return ""

    if convert_as_inline:
        options = [opt.strip() for opt in text.strip().split("\n") if opt.strip()]
        return ", ".join(options)

    return _format_block_element(text)


def _convert_option(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    if not text.strip():
        return ""

    selected = tag.get("selected") is not None
    content = text.strip()

    if convert_as_inline:
        return content

    if selected:
        return f"* {content}\n"
    return f"{content}\n"


def _convert_optgroup(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    label = tag.get("label", "")
    content = text.strip()

    if label and isinstance(label, str) and label.strip():
        return f"**{label.strip()}**\n{content}\n"

    return f"{content}\n"


def _convert_button(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if not text.strip():
        return ""

    return _format_inline_or_block(text, convert_as_inline)


def _convert_progress(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_meter(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_output(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_datalist(*, tag: Tag, text: str, convert_as_inline: bool) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_ruby(*, text: str, convert_as_inline: bool) -> str:  # noqa: ARG001
    if not text.strip():
        return ""

    return text.strip()


def _convert_rb(*, text: str, convert_as_inline: bool) -> str:  # noqa: ARG001
    if not text.strip():
        return ""

    return text.strip()


def _convert_rt(*, text: str, convert_as_inline: bool, tag: Tag) -> str:  # noqa: ARG001
    content = text.strip()

    prev_sibling = tag.previous_sibling
    next_sibling = tag.next_sibling

    has_rp_before = prev_sibling and getattr(prev_sibling, "name", None) == "rp"
    has_rp_after = next_sibling and getattr(next_sibling, "name", None) == "rp"

    if has_rp_before and has_rp_after:
        return content

    return f"({content})"


def _convert_rp(*, text: str, convert_as_inline: bool) -> str:  # noqa: ARG001
    if not text.strip():
        return ""

    return text.strip()


def _convert_rtc(*, text: str, convert_as_inline: bool) -> str:  # noqa: ARG001
    if not text.strip():
        return ""

    return text.strip()


def _convert_dialog(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_menu(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    _ = tag
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return _format_block_element(text)


def _convert_figure(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    _ = tag
    if not text.strip():
        return ""

    if convert_as_inline:
        return text

    content = text.strip()
    if content and not content.endswith("\n\n"):
        if content.endswith("\n"):
            content += "\n"
        else:
            content += "\n\n"
    return content


def _convert_hgroup(*, text: str, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return text


def _convert_picture(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    _ = tag, convert_as_inline
    if not text.strip():
        return ""

    return text.strip()


def _convert_svg(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    if convert_as_inline:
        return text.strip()

    title = tag.find("title")
    title_text = title.get_text().strip() if title else ""

    svg_markup = str(tag)

    svg_bytes = svg_markup.encode("utf-8")
    svg_base64 = base64.b64encode(svg_bytes).decode("utf-8")
    data_uri = f"data:image/svg+xml;base64,{svg_base64}"

    alt_text = title_text or "SVG Image"

    return f"![{alt_text}]({data_uri})"


def _convert_math(*, text: str, convert_as_inline: bool, tag: Tag) -> str:
    if not text.strip():
        return ""

    display = tag.get("display") == "block"

    math_comment = f"<!-- MathML: {tag!s} -->"

    if convert_as_inline or not display:
        return f"{math_comment}{text.strip()}"

    return f"\n\n{math_comment}\n{text.strip()}\n\n"


def create_converters_map(
    autolinks: bool,
    bullets: str,
    code_language: str,
    code_language_callback: Callable[[Tag], str] | None,
    default_title: bool,
    heading_style: Literal["atx", "atx_closed", "underlined"],
    highlight_style: Literal["double-equal", "html", "bold"],
    keep_inline_images_in: Iterable[str] | None,
    list_indent_type: str,
    list_indent_width: int,
    newline_style: str,
    strong_em_symbol: str,
    sub_symbol: str,
    sup_symbol: str,
    wrap: bool,
    wrap_width: int,
) -> ConvertersMap:
    list_indent_str = "\t" if list_indent_type == "tabs" else " " * list_indent_width

    def _wrapper(func: Callable[..., T]) -> Callable[[str, Tag], T]:
        spec = getfullargspec(func)

        def _inner(*, text: str, tag: Tag, convert_as_inline: bool) -> T:
            if spec.kwonlyargs:
                kwargs: dict[str, Any] = {}
                if "tag" in spec.kwonlyargs:
                    kwargs["tag"] = tag
                if "text" in spec.kwonlyargs:
                    kwargs["text"] = text
                if "convert_as_inline" in spec.kwonlyargs:
                    kwargs["convert_as_inline"] = convert_as_inline
                if "list_indent_str" in spec.kwonlyargs:
                    kwargs["list_indent_str"] = list_indent_str
                return func(**kwargs)
            return func(text)

        return cast("Callable[[str, Tag], T]", _inner)

    return {
        "a": _wrapper(partial(_convert_a, autolinks=autolinks, default_title=default_title)),
        "abbr": _wrapper(_convert_abbr),
        "article": _wrapper(_convert_semantic_block),
        "aside": _wrapper(_convert_semantic_block),
        "audio": _wrapper(_convert_media_element),
        "b": _wrapper(partial(_create_inline_converter(2 * strong_em_symbol))),
        "bdi": _wrapper(_create_inline_converter("")),
        "bdo": _wrapper(_create_inline_converter("")),
        "blockquote": _wrapper(partial(_convert_blockquote, list_indent_str=list_indent_str)),
        "br": _wrapper(partial(_convert_br, newline_style=newline_style)),
        "button": _wrapper(_convert_button),
        "caption": _wrapper(_convert_caption),
        "cite": _wrapper(_convert_cite),
        "code": _wrapper(_create_inline_converter("`")),
        "col": _wrapper(_convert_col),
        "colgroup": _wrapper(_convert_colgroup),
        "data": _wrapper(_convert_data),
        "datalist": _wrapper(_convert_datalist),
        "dd": _wrapper(_convert_dd),
        "del": _wrapper(_create_inline_converter("~~")),
        "details": _wrapper(_convert_details),
        "dfn": _wrapper(_create_inline_converter("*")),
        "dialog": _wrapper(_convert_dialog),
        "div": _wrapper(_convert_div),
        "dl": _wrapper(_convert_dl),
        "dt": _wrapper(_convert_dt),
        "em": _wrapper(_create_inline_converter(strong_em_symbol)),
        "fieldset": _wrapper(_convert_fieldset),
        "figcaption": _wrapper(lambda text: f"\n\n*{text.strip()}*\n\n" if text.strip() else ""),
        "figure": _wrapper(_convert_figure),
        "footer": _wrapper(_convert_semantic_block),
        "form": _wrapper(_convert_form),
        "h1": _wrapper(partial(_convert_hn, n=1, heading_style=heading_style)),
        "h2": _wrapper(partial(_convert_hn, n=2, heading_style=heading_style)),
        "h3": _wrapper(partial(_convert_hn, n=3, heading_style=heading_style)),
        "h4": _wrapper(partial(_convert_hn, n=4, heading_style=heading_style)),
        "h5": _wrapper(partial(_convert_hn, n=5, heading_style=heading_style)),
        "h6": _wrapper(partial(_convert_hn, n=6, heading_style=heading_style)),
        "header": _wrapper(_convert_semantic_block),
        "hgroup": _wrapper(_convert_hgroup),
        "hr": _wrapper(lambda _: "\n\n---\n\n"),
        "i": _wrapper(partial(_create_inline_converter(strong_em_symbol))),
        "iframe": _wrapper(_convert_iframe),
        "img": _wrapper(partial(_convert_img, keep_inline_images_in=keep_inline_images_in)),
        "input": _wrapper(_convert_input_enhanced),
        "ins": _wrapper(_create_inline_converter("==")),
        "kbd": _wrapper(_create_inline_converter("`")),
        "label": _wrapper(_convert_label),
        "legend": _wrapper(_convert_legend),
        "li": _wrapper(partial(_convert_li, bullets=bullets, list_indent_str=list_indent_str)),
        "list": _wrapper(partial(_convert_list, list_indent_str=list_indent_str)),
        "main": _wrapper(_convert_semantic_block),
        "mark": _wrapper(partial(_convert_mark, highlight_style=highlight_style)),
        "math": _wrapper(_convert_math),
        "menu": _wrapper(_convert_menu),
        "meter": _wrapper(_convert_meter),
        "nav": _wrapper(_convert_semantic_block),
        "ol": _wrapper(partial(_convert_list, list_indent_str=list_indent_str)),
        "optgroup": _wrapper(_convert_optgroup),
        "option": _wrapper(_convert_option),
        "output": _wrapper(_convert_output),
        "p": _wrapper(partial(_convert_p, wrap=wrap, wrap_width=wrap_width, list_indent_str=list_indent_str)),
        "picture": _wrapper(_convert_picture),
        "pre": _wrapper(
            partial(
                _convert_pre,
                code_language=code_language,
                code_language_callback=code_language_callback,
            )
        ),
        "progress": _wrapper(_convert_progress),
        "q": _wrapper(_convert_q),
        "rb": _wrapper(_convert_rb),
        "rp": _wrapper(_convert_rp),
        "rt": _wrapper(_convert_rt),
        "rtc": _wrapper(_convert_rtc),
        "ruby": _wrapper(_convert_ruby),
        "s": _wrapper(_create_inline_converter("~~")),
        "samp": _wrapper(_create_inline_converter("`")),
        "script": _wrapper(lambda _: ""),
        "section": _wrapper(_convert_semantic_block),
        "select": _wrapper(_convert_select),
        "small": _wrapper(_create_inline_converter("")),
        "strong": _wrapper(_create_inline_converter(strong_em_symbol * 2)),
        "style": _wrapper(lambda _: ""),
        "sub": _wrapper(_create_inline_converter(sub_symbol)),
        "summary": _wrapper(_convert_summary),
        "sup": _wrapper(_create_inline_converter(sup_symbol)),
        "svg": _wrapper(_convert_svg),
        "table": _wrapper(lambda text: f"\n\n{text}\n"),
        "tbody": _wrapper(_convert_tbody),
        "td": _wrapper(_convert_td),
        "textarea": _wrapper(_convert_textarea),
        "tfoot": _wrapper(_convert_tfoot),
        "th": _wrapper(_convert_th),
        "thead": _wrapper(_convert_thead),
        "time": _wrapper(_convert_time),
        "tr": _wrapper(_convert_tr),
        "u": _wrapper(_create_inline_converter("")),
        "ul": _wrapper(partial(_convert_list, list_indent_str=list_indent_str)),
        "var": _wrapper(_create_inline_converter("*")),
        "video": _wrapper(_convert_media_element),
        "wbr": _wrapper(_convert_wbr),
    }
