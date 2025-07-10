from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable
from functools import partial
from inspect import getfullargspec
from textwrap import fill
from typing import Any, Callable, Literal, TypeVar, cast

from bs4.element import Tag

from html_to_markdown.constants import (
    ATX_CLOSED,
    BACKSLASH,
    UNDERLINED,
    line_beginning_re,
)
from html_to_markdown.utils import chomp, indent, underline

SupportedElements = Literal[
    "a",
    "article",
    "aside",
    "audio",
    "b",
    "blockquote",
    "br",
    "cite",
    "code",
    "dd",
    "del",
    "details",
    "dl",
    "dt",
    "em",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "i",
    "iframe",
    "img",
    "input",
    "list",
    "main",
    "mark",
    "nav",
    "ul",
    "ol",
    "li",
    "p",
    "pre",
    "q",
    "script",
    "section",
    "style",
    "s",
    "strong",
    "samp",
    "sub",
    "summary",
    "sup",
    "table",
    "caption",
    "figcaption",
    "td",
    "th",
    "tr",
    "kbd",
    "video",
]

Converter = Callable[[str, Tag], str]
ConvertersMap = dict[SupportedElements, Converter]

T = TypeVar("T")


def _create_inline_converter(markup_prefix: str) -> Callable[[Tag, str], str]:
    """Create an inline converter for a markup pattern or tag.

    Args:
        markup_prefix: The markup prefix to insert.

    Returns:
        A function that can be used to convert HTML to Markdown.
    """

    def implementation(*, tag: Tag, text: str) -> str:
        if tag.find_parent(["pre", "code", "kbd", "samp"]):
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


def _convert_blockquote(*, text: str, tag: Tag, convert_as_inline: bool) -> str:
    if convert_as_inline:
        return text

    if not text:
        return ""

    # Handle cite attribute
    cite_url = tag.get("cite")
    quote_text = f"\n{line_beginning_re.sub('> ', text.strip())}\n\n"

    if cite_url:
        quote_text += f"— <{cite_url}>\n\n"

    return quote_text


def _convert_br(*, convert_as_inline: bool, newline_style: str, tag: Tag) -> str:
    # Convert br to line break, but handle headings specially
    if tag.find_parent(["h1", "h2", "h3", "h4", "h5", "h6"]):
        return " "  # Convert to space in headings

    # Always convert br to line break in other contexts
    _ = convert_as_inline  # Unused but kept for API consistency
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
    # Always preserve images in table cells (td, th) by default
    default_preserve_in = ["td", "th"]
    preserve_in = set(keep_inline_images_in or []) | set(default_preserve_in)
    if convert_as_inline and parent_name not in preserve_in:
        return alt
    if width or height:
        return f"<img src='{src}' alt='{alt}' title='{title}' width='{width}' height='{height}' />"
    return f"![{alt}]({src}{title_part})"


def _convert_list(*, tag: Tag, text: str) -> str:
    nested = False

    before_paragraph = False
    if tag.next_sibling and getattr(tag.next_sibling, "name", None) not in {"ul", "ol"}:
        before_paragraph = True

    while tag:
        if tag.name == "li":
            nested = True
            break

        if not tag.parent:
            break

        tag = tag.parent

    if nested:
        return "\n" + indent(text=text, level=1).rstrip()

    return text + ("\n" if before_paragraph else "")


def _convert_li(*, tag: Tag, text: str, bullets: str) -> str:
    # Check for task list (checkbox input)
    checkbox = tag.find("input", {"type": "checkbox"})
    if checkbox and isinstance(checkbox, Tag):
        checked = checkbox.get("checked") is not None
        checkbox_symbol = "[x]" if checked else "[ ]"
        # Remove the checkbox from the text content
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
    return "{} {}\n".format(bullet, (text or "").strip())


def _convert_p(*, wrap: bool, text: str, convert_as_inline: bool, wrap_width: int) -> str:
    if convert_as_inline:
        return text

    if wrap:
        text = fill(
            text,
            width=wrap_width,
            break_long_words=False,
            break_on_hyphens=False,
        )

    return f"{text}\n\n" if text else ""


def _convert_mark(*, text: str, convert_as_inline: bool, highlight_style: str) -> str:
    """Convert HTML mark element to Markdown highlighting.

    Args:
        text: The text content of the mark element.
        convert_as_inline: Whether to convert as inline content.
        highlight_style: The style to use for highlighting ("double-equal", "html", "bold").

    Returns:
        The converted markdown text.
    """
    if convert_as_inline:
        return text

    if highlight_style == "double-equal":
        return f"=={text}=="
    if highlight_style == "bold":
        return f"**{text}**"
    if highlight_style == "html":
        return f"<mark>{text}</mark>"
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


def _convert_semantic_block(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML5 semantic elements to block-level Markdown.

    Args:
        text: The text content of the semantic element.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text with proper block spacing.
    """
    if convert_as_inline:
        return text

    return f"{text}\n\n" if text.strip() else ""


def _convert_details(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML details element preserving HTML structure.

    Args:
        text: The text content of the details element.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text preserving HTML structure.
    """
    if convert_as_inline:
        return text

    return f"<details>\n{text.strip()}\n</details>\n\n" if text.strip() else ""


def _convert_summary(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML summary element preserving HTML structure.

    Args:
        text: The text content of the summary element.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text preserving HTML structure.
    """
    if convert_as_inline:
        return text

    return f"<summary>{text.strip()}</summary>\n\n" if text.strip() else ""


def _convert_dl(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML definition list element.

    Args:
        text: The text content of the definition list.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text with proper spacing.
    """
    if convert_as_inline:
        return text

    return f"{text}\n" if text.strip() else ""


def _convert_dt(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML definition term element.

    Args:
        text: The text content of the definition term.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text as a definition term.
    """
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f"{text.strip()}\n"


def _convert_dd(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML definition description element.

    Args:
        text: The text content of the definition description.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text as a definition description.
    """
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f":   {text.strip()}\n\n"


def _convert_cite(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML cite element to italic text.

    Args:
        text: The text content of the cite element.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text in italic format.
    """
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    return f"*{text.strip()}*"


def _convert_q(*, text: str, convert_as_inline: bool) -> str:
    """Convert HTML q element to quoted text.

    Args:
        text: The text content of the q element.
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text with quotes.
    """
    if convert_as_inline:
        return text

    if not text.strip():
        return ""

    # Escape any existing quotes in the text
    escaped_text = text.strip().replace('"', '\\"')
    return f'"{escaped_text}"'


def _convert_audio(*, tag: Tag, text: str, convert_as_inline: bool) -> str:  # noqa: C901
    """Convert HTML audio element preserving structure with fallback.

    Args:
        tag: The audio tag element.
        text: The text content of the audio element (fallback content).
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text preserving audio element.
    """
    _ = convert_as_inline  # Unused but kept for API consistency
    src = tag.get("src", "")

    # Check for source elements if no src attribute
    if not src:
        source_tag = tag.find("source")
        if source_tag and isinstance(source_tag, Tag):
            src = source_tag.get("src", "")

    # Get other attributes
    controls = "controls" if tag.get("controls") is not None else ""
    autoplay = "autoplay" if tag.get("autoplay") is not None else ""
    loop = "loop" if tag.get("loop") is not None else ""
    muted = "muted" if tag.get("muted") is not None else ""
    preload = tag.get("preload", "")

    # Build attributes string
    attrs = []
    if src and isinstance(src, str) and src.strip():
        attrs.append(f'src="{src}"')
    if controls:
        attrs.append(controls)
    if autoplay:
        attrs.append(autoplay)
    if loop:
        attrs.append(loop)
    if muted:
        attrs.append(muted)
    if preload and isinstance(preload, str) and preload.strip():
        attrs.append(f'preload="{preload}"')

    attrs_str = " ".join(attrs)

    # If there's fallback content, preserve it
    if text.strip():
        if attrs_str:
            return f"<audio {attrs_str}>\n{text.strip()}\n</audio>\n\n"
        return f"<audio>\n{text.strip()}\n</audio>\n\n"

    # Self-closing for no fallback content
    if attrs_str:
        return f"<audio {attrs_str} />\n\n"
    return "<audio />\n\n"


def _convert_video(*, tag: Tag, text: str, convert_as_inline: bool) -> str:  # noqa: C901, PLR0912
    """Convert HTML video element preserving structure with fallback.

    Args:
        tag: The video tag element.
        text: The text content of the video element (fallback content).
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text preserving video element.
    """
    _ = convert_as_inline  # Unused but kept for API consistency
    src = tag.get("src", "")

    # Check for source elements if no src attribute
    if not src:
        source_tag = tag.find("source")
        if source_tag and isinstance(source_tag, Tag):
            src = source_tag.get("src", "")

    # Get other attributes
    width = tag.get("width", "")
    height = tag.get("height", "")
    poster = tag.get("poster", "")
    controls = "controls" if tag.get("controls") is not None else ""
    autoplay = "autoplay" if tag.get("autoplay") is not None else ""
    loop = "loop" if tag.get("loop") is not None else ""
    muted = "muted" if tag.get("muted") is not None else ""
    preload = tag.get("preload", "")

    # Build attributes string
    attrs = []
    if src and isinstance(src, str) and src.strip():
        attrs.append(f'src="{src}"')
    if width and isinstance(width, str) and width.strip():
        attrs.append(f'width="{width}"')
    if height and isinstance(height, str) and height.strip():
        attrs.append(f'height="{height}"')
    if poster and isinstance(poster, str) and poster.strip():
        attrs.append(f'poster="{poster}"')
    if controls:
        attrs.append(controls)
    if autoplay:
        attrs.append(autoplay)
    if loop:
        attrs.append(loop)
    if muted:
        attrs.append(muted)
    if preload and isinstance(preload, str) and preload.strip():
        attrs.append(f'preload="{preload}"')

    attrs_str = " ".join(attrs)

    # If there's fallback content, preserve it
    if text.strip():
        if attrs_str:
            return f"<video {attrs_str}>\n{text.strip()}\n</video>\n\n"
        return f"<video>\n{text.strip()}\n</video>\n\n"

    # Self-closing for no fallback content
    if attrs_str:
        return f"<video {attrs_str} />\n\n"
    return "<video />\n\n"


def _convert_iframe(*, tag: Tag, text: str, convert_as_inline: bool) -> str:  # noqa: C901, PLR0912
    """Convert HTML iframe element preserving structure.

    Args:
        tag: The iframe tag element.
        text: The text content of the iframe element (usually empty).
        convert_as_inline: Whether to convert as inline content.

    Returns:
        The converted markdown text preserving iframe element.
    """
    _ = text  # Unused but kept for API consistency
    _ = convert_as_inline  # Unused but kept for API consistency
    src = tag.get("src", "")
    width = tag.get("width", "")
    height = tag.get("height", "")
    title = tag.get("title", "")
    allow = tag.get("allow", "")
    sandbox = tag.get("sandbox")  # Don't provide default
    loading = tag.get("loading", "")

    # Build attributes string
    attrs = []
    if src and isinstance(src, str) and src.strip():
        attrs.append(f'src="{src}"')
    if width and isinstance(width, str) and width.strip():
        attrs.append(f'width="{width}"')
    if height and isinstance(height, str) and height.strip():
        attrs.append(f'height="{height}"')
    if title and isinstance(title, str) and title.strip():
        attrs.append(f'title="{title}"')
    if allow and isinstance(allow, str) and allow.strip():
        attrs.append(f'allow="{allow}"')
    if sandbox is not None:
        if isinstance(sandbox, list):
            # BeautifulSoup returns AttributeValueList for space-separated values
            if sandbox:
                attrs.append(f'sandbox="{" ".join(sandbox)}"')
            else:
                # Empty list means boolean attribute
                attrs.append("sandbox")
        elif isinstance(sandbox, str) and sandbox:
            attrs.append(f'sandbox="{sandbox}"')
        else:
            attrs.append("sandbox")
    if loading and isinstance(loading, str) and loading.strip():
        attrs.append(f'loading="{loading}"')

    attrs_str = " ".join(attrs)

    # iframes are typically self-closing in usage
    if attrs_str:
        return f"<iframe {attrs_str}></iframe>\n\n"
    return "<iframe></iframe>\n\n"


def create_converters_map(
    autolinks: bool,
    bullets: str,
    code_language: str,
    code_language_callback: Callable[[Tag], str] | None,
    default_title: bool,
    heading_style: Literal["atx", "atx_closed", "underlined"],
    highlight_style: Literal["double-equal", "html", "bold"],
    keep_inline_images_in: Iterable[str] | None,
    newline_style: str,
    strong_em_symbol: str,
    sub_symbol: str,
    sup_symbol: str,
    wrap: bool,
    wrap_width: int,
) -> ConvertersMap:
    """Create a mapping of HTML elements to their corresponding conversion functions.

    Args:
        autolinks: Whether to convert URLs into links.
        bullets: The bullet characters to use for unordered lists.
        code_language: The default code language to use.
        code_language_callback: A callback to get the code language.
        default_title: Whether to use the URL as the title for links.
        heading_style: The style of headings.
        highlight_style: The style to use for highlighted text (mark elements).
        keep_inline_images_in: The tags to keep inline images in.
        newline_style: The style of newlines.
        strong_em_symbol: The symbol to use for strong and emphasis text.
        sub_symbol: The symbol to use for subscript text.
        sup_symbol: The symbol to use for superscript text.
        wrap: Whether to wrap text.
        wrap_width: The width to wrap text at.

    Returns:
        A mapping of HTML elements to their corresponding conversion functions
    """

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
                return func(**kwargs)
            return func(text)

        return cast("Callable[[str, Tag], T]", _inner)

    return {
        "a": _wrapper(partial(_convert_a, autolinks=autolinks, default_title=default_title)),
        "article": _wrapper(_convert_semantic_block),
        "aside": _wrapper(_convert_semantic_block),
        "audio": _wrapper(_convert_audio),
        "b": _wrapper(partial(_create_inline_converter(2 * strong_em_symbol))),
        "blockquote": _wrapper(partial(_convert_blockquote)),
        "br": _wrapper(partial(_convert_br, newline_style=newline_style)),
        "cite": _wrapper(_convert_cite),
        "code": _wrapper(_create_inline_converter("`")),
        "dd": _wrapper(_convert_dd),
        "del": _wrapper(_create_inline_converter("~~")),
        "details": _wrapper(_convert_details),
        "dl": _wrapper(_convert_dl),
        "dt": _wrapper(_convert_dt),
        "em": _wrapper(_create_inline_converter(strong_em_symbol)),
        "footer": _wrapper(_convert_semantic_block),
        "h1": _wrapper(partial(_convert_hn, n=1, heading_style=heading_style)),
        "h2": _wrapper(partial(_convert_hn, n=2, heading_style=heading_style)),
        "h3": _wrapper(partial(_convert_hn, n=3, heading_style=heading_style)),
        "h4": _wrapper(partial(_convert_hn, n=4, heading_style=heading_style)),
        "h5": _wrapper(partial(_convert_hn, n=5, heading_style=heading_style)),
        "h6": _wrapper(partial(_convert_hn, n=6, heading_style=heading_style)),
        "header": _wrapper(_convert_semantic_block),
        "hr": _wrapper(lambda _: "\n\n---\n\n"),
        "i": _wrapper(partial(_create_inline_converter(strong_em_symbol))),
        "iframe": _wrapper(_convert_iframe),
        "img": _wrapper(partial(_convert_img, keep_inline_images_in=keep_inline_images_in)),
        "input": _wrapper(lambda _: ""),
        "kbd": _wrapper(_create_inline_converter("`")),
        "list": _wrapper(_convert_list),
        "main": _wrapper(_convert_semantic_block),
        "mark": _wrapper(partial(_convert_mark, highlight_style=highlight_style)),
        "nav": _wrapper(_convert_semantic_block),
        "ul": _wrapper(_convert_list),
        "ol": _wrapper(_convert_list),
        "li": _wrapper(partial(_convert_li, bullets=bullets)),
        "p": _wrapper(partial(_convert_p, wrap=wrap, wrap_width=wrap_width)),
        "pre": _wrapper(
            partial(
                _convert_pre,
                code_language=code_language,
                code_language_callback=code_language_callback,
            )
        ),
        "q": _wrapper(_convert_q),
        "script": _wrapper(lambda _: ""),
        "section": _wrapper(_convert_semantic_block),
        "style": _wrapper(lambda _: ""),
        "s": _wrapper(_create_inline_converter("~~")),
        "strong": _wrapper(_create_inline_converter(strong_em_symbol * 2)),
        "samp": _wrapper(_create_inline_converter("`")),
        "sub": _wrapper(_create_inline_converter(sub_symbol)),
        "summary": _wrapper(_convert_summary),
        "sup": _wrapper(_create_inline_converter(sup_symbol)),
        "table": _wrapper(lambda text: f"\n\n{text}\n"),
        "caption": _wrapper(lambda text: f"{text}\n"),
        "figcaption": _wrapper(lambda text: f"\n\n{text}\n\n"),
        "td": _wrapper(_convert_td),
        "th": _wrapper(_convert_th),
        "tr": _wrapper(_convert_tr),
        "video": _wrapper(_convert_video),
    }
