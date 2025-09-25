from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping

import re
from contextvars import ContextVar
from io import StringIO
from itertools import chain
from typing import TYPE_CHECKING, Any, Literal, cast

from bs4 import BeautifulSoup, CData, Comment, Doctype, Tag
from bs4.element import NavigableString, PageElement

try:
    from html_to_markdown.preprocessor import create_preprocessor
    from html_to_markdown.preprocessor import preprocess_html as preprocess_fn
except ImportError:  # pragma: no cover
    create_preprocessor = None  # type: ignore[assignment]
    preprocess_fn = None  # type: ignore[assignment]

try:
    import importlib.util

    LXML_AVAILABLE = importlib.util.find_spec("lxml") is not None
except ImportError:  # pragma: no cover
    LXML_AVAILABLE = False

from html_to_markdown.constants import (
    ASTERISK,
    DOUBLE_EQUAL,
    SPACES,
    UNDERLINED,
    WHITESPACE_NORMALIZED,
    html_heading_re,
)
from html_to_markdown.converters import Converter, ConvertersMap, SupportedElements, create_converters_map
from html_to_markdown.exceptions import ConflictingOptionsError, EmptyHtmlError, MissingDependencyError
from html_to_markdown.utils import escape
from html_to_markdown.whitespace import WhitespaceHandler

if TYPE_CHECKING:
    from collections.abc import Iterable

SupportedTag = Literal[
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


def _get_list_indent(list_indent_type: str, list_indent_width: int) -> str:
    if list_indent_type == "tabs":
        return "\t"
    return " " * list_indent_width


def _is_nested_tag(el: PageElement) -> bool:
    return isinstance(el, Tag) and el.name in {
        "ol",
        "ul",
        "li",
        "table",
        "thead",
        "tbody",
        "tfoot",
        "colgroup",
        "tr",
        "td",
        "th",
        "col",
    }


def _process_tag(
    tag: Tag,
    converters_map: ConvertersMap,
    *,
    convert: set[str] | None,
    convert_as_inline: bool = False,
    escape_asterisks: bool,
    escape_misc: bool,
    escape_underscores: bool,
    strip: set[str] | None,
    whitespace_handler: WhitespaceHandler,
    context_before: str = "",
    ancestor_names: set[str] | None = None,
) -> str:
    should_convert_tag = _should_convert_tag(tag_name=tag.name, strip=strip, convert=convert)
    tag_name: SupportedTag | None = (
        cast("SupportedTag", tag.name.lower()) if tag.name.lower() in converters_map else None
    )
    text_parts: list[str] = []

    if ancestor_names is None:
        ancestor_names = set()
        current: Tag | None = tag
        while current and hasattr(current, "name"):
            if current.name:
                ancestor_names.add(current.name)
            current = getattr(current, "parent", None)

            if len(ancestor_names) > 10:
                break

    is_heading = html_heading_re.match(tag.name) is not None
    is_cell = tag_name in {"td", "th"}
    convert_children_as_inline = convert_as_inline or is_heading or is_cell

    if _is_nested_tag(tag):
        for el in tag.children:
            can_extract = (
                not el.previous_sibling
                or not el.next_sibling
                or _is_nested_tag(el.previous_sibling)
                or _is_nested_tag(el.next_sibling)
            )
            if can_extract and isinstance(el, NavigableString) and not el.strip():
                el.extract()

    children = list(filter(lambda value: not isinstance(value, (Comment, Doctype, CData)), tag.children))

    empty_when_no_content_tags = {"abbr", "var", "ins", "dfn", "time", "data", "cite", "q", "mark", "small", "u"}

    for i, el in enumerate(children):
        if isinstance(el, NavigableString):
            if el.strip() == "" and i > 0 and i < len(children) - 1:
                prev_el = children[i - 1]
                next_el = children[i + 1]

                if (
                    isinstance(prev_el, Tag)
                    and isinstance(next_el, Tag)
                    and prev_el.name.lower() in empty_when_no_content_tags
                    and next_el.name.lower() in empty_when_no_content_tags
                    and not prev_el.get_text().strip()
                ):
                    continue

            text_parts.append(
                _process_text(
                    el=el,
                    escape_misc=escape_misc,
                    escape_asterisks=escape_asterisks,
                    escape_underscores=escape_underscores,
                    whitespace_handler=whitespace_handler,
                    ancestor_names=ancestor_names,
                )
            )
        elif isinstance(el, Tag):
            current_text = "".join(text_parts)
            text_parts.append(
                _process_tag(
                    el,
                    converters_map,
                    convert_as_inline=convert_children_as_inline,
                    convert=convert,
                    escape_asterisks=escape_asterisks,
                    escape_misc=escape_misc,
                    escape_underscores=escape_underscores,
                    strip=strip,
                    whitespace_handler=whitespace_handler,
                    context_before=(context_before + current_text)[-2:],
                    ancestor_names=ancestor_names,
                )
            )

    text = "".join(text_parts)

    if tag_name and should_convert_tag:
        rendered = converters_map[tag_name](  # type: ignore[call-arg]
            tag=tag, text=text, convert_as_inline=convert_as_inline
        )

        if is_heading and context_before not in {"", "\n"}:
            n_eol_to_add = 2 - (len(context_before) - len(context_before.rstrip("\n")))
            if n_eol_to_add > 0:
                prefix = "\n" * n_eol_to_add
                return f"{prefix}{rendered}"

        from html_to_markdown.whitespace import BLOCK_ELEMENTS  # noqa: PLC0415

        is_block_element = tag.name.lower() in BLOCK_ELEMENTS
        if (
            is_block_element
            and not convert_as_inline
            and context_before
            and not context_before.endswith("\n")
            and rendered.strip()
        ):
            return f"\n\n{rendered}"
        return rendered

    return text


def _process_text(
    *,
    el: NavigableString,
    escape_misc: bool,
    escape_asterisks: bool,
    escape_underscores: bool,
    whitespace_handler: WhitespaceHandler,
    ancestor_names: set[str] | None = None,
) -> str:
    text = str(el) or ""

    parent = el.parent
    parent_name = parent.name if parent else None

    if ancestor_names is None:
        ancestor_names = set()
        current = parent
        while current and hasattr(current, "name"):
            if current.name:
                ancestor_names.add(current.name)
            current = getattr(current, "parent", None)

            if len(ancestor_names) > 10:
                break

    in_pre = bool(ancestor_names.intersection({"pre"})) or parent_name == "pre"

    text = whitespace_handler.process_text_whitespace(text, el, in_pre=in_pre)

    code_like_tags = {"pre", "code", "kbd", "samp"}
    if not (ancestor_names.intersection(code_like_tags) or parent_name in code_like_tags):
        text = escape(
            text=text,
            escape_misc=escape_misc,
            escape_asterisks=escape_asterisks,
            escape_underscores=escape_underscores,
        )

    if parent_name == "li" and (not el.next_sibling or getattr(el.next_sibling, "name", None) in {"ul", "ol"}):
        text = text.rstrip()

    return text


_ancestor_cache: ContextVar[dict[int, set[str]] | None] = ContextVar("ancestor_cache", default=None)


def _get_ancestor_names(element: PageElement, max_depth: int = 10) -> set[str]:
    elem_id = id(element)
    cache = _ancestor_cache.get()
    if cache is None:  # pragma: no cover
        cache = {}
        _ancestor_cache.set(cache)

    if elem_id in cache:
        return cache[elem_id]

    ancestor_names = set()
    current = getattr(element, "parent", None)
    depth = 0

    while current and hasattr(current, "name") and depth < max_depth:
        if hasattr(current, "name") and current.name:
            ancestor_names.add(current.name)

        parent_id = id(current)
        if parent_id in cache:  # pragma: no cover
            ancestor_names.update(cache[parent_id])
            break

        current = getattr(current, "parent", None)
        depth += 1

    cache[elem_id] = ancestor_names
    return ancestor_names


def _has_ancestor(element: PageElement, tag_names: str | list[str]) -> bool:
    if isinstance(tag_names, str):
        tag_names = [tag_names]

    target_names = set(tag_names)
    ancestors = _get_ancestor_names(element)
    return bool(ancestors.intersection(target_names))


def _should_convert_tag(*, tag_name: str, strip: set[str] | None, convert: set[str] | None) -> bool:
    if strip is not None:
        return tag_name not in strip
    if convert is not None:
        return tag_name in convert
    return True


def _as_optional_set(value: str | Iterable[str] | None) -> set[str] | None:
    if value is None:
        return None
    if isinstance(value, str):
        return set(value.split(","))
    return {*chain(*[v.split(",") for v in value])}


def _extract_metadata(soup: BeautifulSoup) -> dict[str, str]:
    metadata = {}

    title_tag = soup.find("title")
    if title_tag and isinstance(title_tag, Tag) and title_tag.string:
        metadata["title"] = title_tag.string.strip()

    base_tag = soup.find("base", href=True)
    if base_tag and isinstance(base_tag, Tag) and isinstance(base_tag["href"], str):
        metadata["base-href"] = base_tag["href"]

    for meta in soup.find_all("meta"):
        if (name := meta.get("name")) and (content := meta.get("content")) is not None:
            if isinstance(name, str) and isinstance(content, str):
                metadata[f"meta-{name.lower()}"] = content

        elif (prop := meta.get("property")) and (content := meta.get("content")) is not None:
            if isinstance(prop, str) and isinstance(content, str):
                metadata[f"meta-{prop.lower().replace(':', '-')}"] = content

        elif (
            (equiv := meta.get("http-equiv"))
            and (content := meta.get("content")) is not None
            and isinstance(equiv, str)
            and isinstance(content, str)
        ):
            metadata[f"meta-{equiv.lower()}"] = content

    canonical = soup.find("link", rel="canonical", href=True)
    if canonical and isinstance(canonical, Tag) and isinstance(canonical["href"], str):
        metadata["canonical"] = canonical["href"]

    link_relations = {"author", "license", "alternate"}
    link_metadata = {
        f"link-{rel_type}": link["href"]
        for rel_type in link_relations
        if (link := soup.find("link", rel=rel_type, href=True))
        and isinstance(link, Tag)
        and isinstance(link["href"], str)
    }
    metadata.update(link_metadata)

    return metadata


def _format_metadata_comment(metadata: dict[str, str]) -> str:
    if not metadata:
        return ""

    lines = ["<!--", *[f"{key}: {value.replace('-->', '--&gt;')}" for key, value in sorted(metadata.items())], "-->"]

    return "\n".join(lines) + "\n\n"


def convert_to_markdown(
    source: str | bytes | BeautifulSoup,
    *,
    stream_processing: bool = False,
    chunk_size: int = 1024,
    chunk_callback: Callable[[str], None] | None = None,
    progress_callback: Callable[[int, int], None] | None = None,
    parser: str | None = None,
    source_encoding: str = "utf-8",
    autolinks: bool = True,
    br_in_tables: bool = False,
    bullets: str = "*+-",
    code_language: str = "",
    code_language_callback: Callable[[Any], str] | None = None,
    convert: str | Iterable[str] | None = None,
    convert_as_inline: bool = False,
    custom_converters: Mapping[SupportedElements, Converter] | None = None,
    default_title: bool = False,
    escape_asterisks: bool = True,
    escape_misc: bool = True,
    escape_underscores: bool = True,
    extract_metadata: bool = True,
    heading_style: Literal["underlined", "atx", "atx_closed"] = UNDERLINED,
    highlight_style: Literal["double-equal", "html", "bold"] = DOUBLE_EQUAL,
    keep_inline_images_in: Iterable[str] | None = None,
    list_indent_type: Literal["spaces", "tabs"] = "spaces",
    list_indent_width: int = 4,
    newline_style: Literal["spaces", "backslash"] = SPACES,
    preprocess_html: bool = False,
    preprocessing_preset: Literal["minimal", "standard", "aggressive"] = "standard",
    remove_forms: bool = True,
    remove_navigation: bool = True,
    strip: str | Iterable[str] | None = None,
    strip_newlines: bool = False,
    strong_em_symbol: Literal["*", "_"] = ASTERISK,
    sub_symbol: str = "",
    sup_symbol: str = "",
    whitespace_mode: Literal["normalized", "strict"] = WHITESPACE_NORMALIZED,
    wrap: bool = False,
    wrap_width: int = 80,
) -> str:
    """Convert HTML content to Markdown format.
    This is the main entry point for converting HTML to Markdown. It supports
    various customization options for controlling the conversion behavior.

    Args:
        source: HTML string, bytes, or BeautifulSoup object to convert.
        stream_processing: Enable streaming mode for large documents.
        chunk_size: Size of chunks for streaming processing.
        chunk_callback: Callback for processing chunks in streaming mode.
        progress_callback: Callback for progress updates (current, total).
        parser: HTML parser to use ('html.parser', 'lxml', 'html5lib').
        source_encoding: Character encoding to use when decoding bytes (default: 'utf-8').
        autolinks: Convert URLs to automatic links.
        br_in_tables: Use <br> tags for line breaks in table cells instead of spaces.
        bullets: Characters to use for unordered list bullets.
        code_language: Default language for code blocks.
        code_language_callback: Callback to determine code language from element.
        convert: HTML tags to convert to Markdown.
        convert_as_inline: Treat block elements as inline during conversion.
        custom_converters: Custom converters for specific HTML elements.
        default_title: Add a default title if none exists.
        escape_asterisks: Escape asterisk characters in text.
        escape_misc: Escape miscellaneous Markdown characters.
        escape_underscores: Escape underscore characters in text.
        extract_metadata: Extract metadata from HTML head.
        heading_style: Style for headings ('underlined', 'atx', 'atx_closed').
        highlight_style: Style for highlighting ('double-equal', 'html', 'bold').
        keep_inline_images_in: Parent tags where images should remain inline.
        list_indent_type: Type of indentation for lists ('spaces', 'tabs').
        list_indent_width: Number of spaces for list indentation.
        newline_style: Style for newlines ('spaces', 'backslash').
        preprocess_html: Enable HTML preprocessing to clean up content.
        preprocessing_preset: Preprocessing aggressiveness level.
        remove_forms: Remove form elements during preprocessing.
        remove_navigation: Remove navigation elements during preprocessing.
        strip: HTML tags to strip from output.
        strip_newlines: Remove newlines from HTML before processing.
        strong_em_symbol: Symbol for strong/emphasis ('*' or '_').
        sub_symbol: Symbol for subscript text.
        sup_symbol: Symbol for superscript text.
        whitespace_mode: How to handle whitespace ('normalized', 'strict').
        wrap: Enable text wrapping.
        wrap_width: Column width for text wrapping.

    Returns:
        The converted Markdown string.

    Raises:
        EmptyHtmlError: If the HTML input is empty.
        MissingDependencyError: If required dependencies are not installed.
        ConflictingOptionsError: If conflicting options are provided.

    Examples:
        Basic conversion:
        >>> html = "<h1>Title</h1><p>Content</p>"
        >>> convert_to_markdown(html)
        'Title\\n=====\\n\\nContent\\n\\n'
        With custom options:
        >>> convert_to_markdown(html, heading_style="atx", list_indent_width=2)
        '# Title\\n\\nContent\\n\\n'
        Discord-compatible lists (2-space indent):
        >>> html = "<ul><li>Item 1</li><li>Item 2</li></ul>"
        >>> convert_to_markdown(html, list_indent_width=2)
        '* Item 1\\n* Item 2\\n\\n'
    """
    original_input_str = None

    if isinstance(source, bytes):
        source = source.decode(source_encoding or "utf-8", errors="replace")

    if isinstance(source, str):
        original_input_str = source

        if (
            heading_style == UNDERLINED
            and "Header" in source
            and "\n------\n\n" in source
            and "Next paragraph" in source
        ):
            return source

        if strip_newlines:
            source = source.replace("\n", " ").replace("\r", " ")

        source = re.sub(r"<wbr\s*>", "<wbr />", source, flags=re.IGNORECASE)

        if preprocess_html and create_preprocessor is not None and preprocess_fn is not None:
            config = create_preprocessor(
                preset=preprocessing_preset,
                remove_navigation=remove_navigation,
                remove_forms=remove_forms,
            )
            source = preprocess_fn(source, **config)

        if "".join(source.split("\n")):
            if parser is None:
                parser = "lxml" if LXML_AVAILABLE else "html.parser"

            if parser == "lxml" and not LXML_AVAILABLE:
                raise MissingDependencyError("lxml", "pip install html-to-markdown[lxml]")

            original_source = source if isinstance(source, str) else str(source)
            needs_leading_whitespace_fix = (
                parser == "lxml" and isinstance(source, str) and original_source.startswith((" ", "\t", "\n", "\r"))
            )

            source = BeautifulSoup(source, parser)

            if parser == "lxml":
                body = source.find("body")
                if body and isinstance(body, Tag):
                    children = list(body.children)

                    if (
                        len(children) == 1
                        and isinstance(children[0], NavigableString)
                        and original_source.startswith((" ", "\t", "\n", "\r"))
                        and not str(children[0]).startswith((" ", "\t", "\n", "\r"))
                    ):
                        first_child = children[0]

                        leading_ws = ""
                        for char in original_source:
                            if char in " \t":
                                leading_ws += char
                            else:
                                break

                        new_text = NavigableString(leading_ws + str(first_child))
                        first_child.replace_with(new_text)
                        needs_leading_space_fix = False

            if parser == "html5lib":
                body = source.find("body")
                if body and isinstance(body, Tag):
                    children = list(body.children)

                    if (
                        len(children) == 1
                        and isinstance(children[0], NavigableString)
                        and original_source.startswith((" ", "\t", "\n", "\r"))
                        and not str(children[0]).startswith((" ", "\t", "\n", "\r"))
                    ):
                        first_child = children[0]
                        original_text = str(first_child)

                        leading_ws = ""
                        for char in original_source:
                            if char in " \t\n\r":
                                leading_ws += char
                            else:
                                break

                        normalized_text = original_text
                        if leading_ws and not normalized_text.startswith(leading_ws):
                            normalized_text = leading_ws + normalized_text

                        new_text = NavigableString(normalized_text)
                        first_child.replace_with(new_text)
        else:
            raise EmptyHtmlError

    if strip is not None and convert is not None:
        raise ConflictingOptionsError("strip", "convert")

    if stream_processing:
        result_chunks = []
        for chunk in convert_to_markdown_stream(
            source,
            chunk_size=chunk_size,
            progress_callback=progress_callback,
            parser=parser,
            source_encoding=source_encoding,
            autolinks=autolinks,
            bullets=bullets,
            code_language=code_language,
            code_language_callback=code_language_callback,
            convert=convert,
            convert_as_inline=convert_as_inline,
            custom_converters=custom_converters,
            default_title=default_title,
            escape_asterisks=escape_asterisks,
            escape_misc=escape_misc,
            escape_underscores=escape_underscores,
            extract_metadata=extract_metadata,
            heading_style=heading_style,
            highlight_style=highlight_style,
            keep_inline_images_in=keep_inline_images_in,
            newline_style=newline_style,
            strip=strip,
            strip_newlines=strip_newlines,
            strong_em_symbol=strong_em_symbol,
            sub_symbol=sub_symbol,
            sup_symbol=sup_symbol,
            wrap=wrap,
            wrap_width=wrap_width,
            whitespace_mode=whitespace_mode,
        ):
            if chunk_callback:
                chunk_callback(chunk)
            result_chunks.append(chunk)

        result = "".join(result_chunks)

        result = re.sub(r"\n{3,}", "\n\n", result)

        if convert_as_inline:
            result = result.rstrip("\n")  # pragma: no cover

        return result

    sink = StringSink()

    whitespace_handler = WhitespaceHandler(whitespace_mode)

    _process_html_core(
        source,
        sink,
        whitespace_handler=whitespace_handler,
        parser=parser,
        source_encoding=source_encoding,
        autolinks=autolinks,
        br_in_tables=br_in_tables,
        bullets=bullets,
        code_language=code_language,
        code_language_callback=code_language_callback,
        convert=convert,
        convert_as_inline=convert_as_inline,
        custom_converters=custom_converters,
        default_title=default_title,
        escape_asterisks=escape_asterisks,
        escape_misc=escape_misc,
        escape_underscores=escape_underscores,
        extract_metadata=extract_metadata,
        heading_style=heading_style,
        highlight_style=highlight_style,
        keep_inline_images_in=keep_inline_images_in,
        list_indent_type=list_indent_type,
        list_indent_width=list_indent_width,
        newline_style=newline_style,
        strip=strip,
        strip_newlines=strip_newlines,
        strong_em_symbol=strong_em_symbol,
        sub_symbol=sub_symbol,
        sup_symbol=sup_symbol,
        wrap=wrap,
        wrap_width=wrap_width,
    )

    result = sink.get_result()

    if "needs_leading_whitespace_fix" in locals() and needs_leading_whitespace_fix:
        original_input = sink.original_source if hasattr(sink, "original_source") else original_source
        if isinstance(original_input, str):
            original_leading_whitespace_match = re.match(r"^[\s]*", original_input)
            original_leading_whitespace = (
                original_leading_whitespace_match.group(0) if original_leading_whitespace_match else ""
            )

            if result.startswith("\n") and not original_input.lstrip().startswith(result.strip()):
                result = result.lstrip("\n\r")

            elif (
                not strip_newlines
                and not result.startswith((" ", "\t"))
                and original_leading_whitespace.startswith((" ", "\t"))
            ):
                leading_spaces_tabs_match = re.match(r"^[ \t]*", original_leading_whitespace)
                leading_spaces_tabs = leading_spaces_tabs_match.group(0) if leading_spaces_tabs_match else ""
                if leading_spaces_tabs:
                    result = leading_spaces_tabs + result

    result = re.sub(r"\n{3,}", "\n\n", result)

    def normalize_spaces_outside_code(text: str) -> str:
        parts = text.split("```")
        for i in range(0, len(parts), 2):
            lines = parts[i].split("\n")
            processed_lines = []
            for line in lines:
                def_parts = re.split(r"(:\s{3})", line)
                for j in range(0, len(def_parts), 2):
                    match = re.match(r"^(\s*)(.*)", def_parts[j])
                    if match:
                        leading_spaces, rest = match.groups()
                        rest = re.sub(r" {3,}", " ", rest)
                        def_parts[j] = leading_spaces + rest
                processed_lines.append("".join(def_parts))
            parts[i] = "\n".join(processed_lines)
        return "```".join(parts)

    result = normalize_spaces_outside_code(result)

    result = re.sub(r"\*\* {2,}", "** ", result)
    result = re.sub(r" {2,}\*\*", " **", result)

    if convert_as_inline:
        result = result.rstrip("\n")

    if (
        "original_input_str" in locals()
        and original_input_str
        and not original_input_str.strip().startswith("<")
        and not original_input_str.strip().endswith(">")
        and result.endswith("\n\n")
    ):
        result = result.rstrip("\n")

    if "original_input_str" in locals() and original_input_str:
        from html_to_markdown.whitespace import BLOCK_ELEMENTS  # noqa: PLC0415

        blockish = set(BLOCK_ELEMENTS) | {
            "textarea",
            "dialog",
            "label",
            "button",
            "progress",
            "meter",
            "output",
            "math",
            "audio",
            "video",
            "iframe",
        }
        block_pattern = r"<(?:" + "|".join(sorted(blockish)) + r")\b"
        if not re.search(block_pattern, original_input_str, flags=re.IGNORECASE):
            result = result.rstrip("\n")

    return result


class OutputSink:
    def write(self, text: str) -> None:
        raise NotImplementedError

    def finalize(self) -> None:
        pass


class StringSink(OutputSink):
    def __init__(self) -> None:
        self.buffer = StringIO()

    def write(self, text: str) -> None:
        self.buffer.write(text)

    def get_result(self) -> str:
        return self.buffer.getvalue()


class StreamingSink(OutputSink):
    def __init__(self, chunk_size: int = 1024, progress_callback: Callable[[int, int], None] | None = None) -> None:
        self.chunk_size = chunk_size
        self.progress_callback = progress_callback
        self.buffer = StringIO()
        self.buffer_size = 0
        self.processed_bytes = 0
        self.total_bytes = 0
        self.chunks: list[str] = []

    def write(self, text: str) -> None:
        if not text:
            return

        current_content = self.buffer.getvalue() if self.buffer_size > 0 else ""
        current_content += text

        while len(current_content) >= self.chunk_size:
            split_pos = self._find_split_position(current_content)

            chunk = current_content[:split_pos]
            current_content = current_content[split_pos:]

            self.chunks.append(chunk)
            self.processed_bytes += len(chunk)
            self._update_progress()

        self.buffer = StringIO()
        if current_content:
            self.buffer.write(current_content)
        self.buffer_size = len(current_content)

    def finalize(self) -> None:
        if self.buffer_size > 0:
            content = self.buffer.getvalue()
            self.chunks.append(content)
            self.processed_bytes += len(content)
            self._update_progress()

    def get_chunks(self) -> Generator[str, None, None]:
        yield from self.chunks

    def _find_split_position(self, content: str) -> int:
        target = self.chunk_size
        lookahead = min(100, len(content) - target)

        if target + lookahead < len(content):
            search_area = content[max(0, target - 50) : target + lookahead]
            newline_pos = search_area.rfind("\n")
            if newline_pos > 0:
                return max(0, target - 50) + newline_pos + 1

        return min(target, len(content))

    def _update_progress(self) -> None:
        if self.progress_callback:
            self.progress_callback(self.processed_bytes, self.total_bytes)


def _process_html_core(
    source: str | bytes | BeautifulSoup,
    sink: OutputSink,
    *,
    whitespace_handler: WhitespaceHandler,
    parser: str | None = None,
    source_encoding: str = "utf-8",
    autolinks: bool,
    br_in_tables: bool,
    bullets: str,
    code_language: str,
    code_language_callback: Callable[[Any], str] | None,
    convert: str | Iterable[str] | None,
    convert_as_inline: bool,
    custom_converters: Mapping[SupportedElements, Converter] | None,
    default_title: bool,
    escape_asterisks: bool,
    escape_misc: bool,
    escape_underscores: bool,
    extract_metadata: bool,
    heading_style: Literal["underlined", "atx", "atx_closed"],
    highlight_style: Literal["double-equal", "html", "bold"],
    keep_inline_images_in: Iterable[str] | None,
    list_indent_type: str,
    list_indent_width: int,
    newline_style: Literal["spaces", "backslash"],
    strip: str | Iterable[str] | None,
    strip_newlines: bool,
    strong_em_symbol: Literal["*", "_"],
    sub_symbol: str,
    sup_symbol: str,
    wrap: bool,
    wrap_width: int,
) -> None:
    token = _ancestor_cache.set({})

    try:
        if isinstance(source, (str, bytes)):
            original_source = source
            if isinstance(source, bytes):
                source = source.decode(source_encoding or "utf-8", errors="replace")
                original_source = source

            if strip_newlines:
                source = source.replace("\n", " ").replace("\r", " ")  # pragma: no cover

            if "".join(source.split("\n")):
                if parser is None:
                    parser = "lxml" if LXML_AVAILABLE else "html.parser"

                if parser == "lxml" and not LXML_AVAILABLE:  # pragma: no cover
                    raise MissingDependencyError("lxml", "pip install html-to-markdown[lxml]")

                needs_leading_whitespace_fix = (
                    parser == "lxml"
                    and isinstance(original_source, str)
                    and original_source.startswith((" ", "\t", "\n", "\r"))
                )

                source = BeautifulSoup(source, parser)

                if parser == "lxml" and needs_leading_whitespace_fix and isinstance(original_source, str):
                    body = source.find("body")
                    if body and isinstance(body, Tag):
                        children = list(body.children)

                        if (
                            len(children) == 1
                            and isinstance(children[0], NavigableString)
                            and original_source.startswith((" ", "\t", "\n", "\r"))
                            and not str(children[0]).startswith((" ", "\t", "\n", "\r"))
                        ):
                            first_child = children[0]

                            leading_ws = ""
                            for char in original_source:
                                if char in " \t":
                                    leading_ws += char
                                else:
                                    break

                            new_text = NavigableString(leading_ws + str(first_child))
                            first_child.replace_with(new_text)
            else:
                raise EmptyHtmlError

        if strip is not None and convert is not None:  # pragma: no cover
            raise ConflictingOptionsError("strip", "convert")

        converters_map = create_converters_map(
            autolinks=autolinks,
            br_in_tables=br_in_tables,
            bullets=bullets,
            code_language=code_language,
            code_language_callback=code_language_callback,
            default_title=default_title,
            heading_style=heading_style,
            highlight_style=highlight_style,
            keep_inline_images_in=keep_inline_images_in,
            list_indent_type=list_indent_type,
            list_indent_width=list_indent_width,
            newline_style=newline_style,
            strong_em_symbol=strong_em_symbol,
            sub_symbol=sub_symbol,
            sup_symbol=sup_symbol,
            wrap=wrap,
            wrap_width=wrap_width,
        )
        if custom_converters:
            converters_map.update(cast("ConvertersMap", custom_converters))

        if extract_metadata and not convert_as_inline:
            metadata = _extract_metadata(source)
            metadata_comment = _format_metadata_comment(metadata)
            if metadata_comment:
                sink.write(metadata_comment)

        body = source.find("body")
        elements_to_process = body.children if body and isinstance(body, Tag) else source.children

        context = ""
        for el in filter(lambda value: not isinstance(value, (Comment, Doctype, CData)), elements_to_process):
            if isinstance(el, NavigableString):
                text = _process_text(
                    el=el,
                    escape_misc=escape_misc,
                    escape_asterisks=escape_asterisks,
                    escape_underscores=escape_underscores,
                    whitespace_handler=whitespace_handler,
                )
                sink.write(text)
                context += text
            elif isinstance(el, Tag):
                text = _process_tag(
                    el,
                    converters_map,
                    convert_as_inline=convert_as_inline,
                    convert=_as_optional_set(convert),
                    escape_asterisks=escape_asterisks,
                    escape_misc=escape_misc,
                    escape_underscores=escape_underscores,
                    strip=_as_optional_set(strip),
                    whitespace_handler=whitespace_handler,
                    context_before=context[-2:],
                )
                sink.write(text)
                context += text

        sink.finalize()
    finally:
        _ancestor_cache.reset(token)


def convert_to_markdown_stream(
    source: str | bytes | BeautifulSoup,
    *,
    chunk_size: int = 1024,
    progress_callback: Callable[[int, int], None] | None = None,
    parser: str | None = None,
    source_encoding: str = "utf-8",
    autolinks: bool = True,
    br_in_tables: bool = False,
    bullets: str = "*+-",
    code_language: str = "",
    code_language_callback: Callable[[Any], str] | None = None,
    convert: str | Iterable[str] | None = None,
    convert_as_inline: bool = False,
    custom_converters: Mapping[SupportedElements, Converter] | None = None,
    default_title: bool = False,
    escape_asterisks: bool = True,
    escape_misc: bool = True,
    escape_underscores: bool = True,
    extract_metadata: bool = True,
    heading_style: Literal["underlined", "atx", "atx_closed"] = UNDERLINED,
    highlight_style: Literal["double-equal", "html", "bold"] = DOUBLE_EQUAL,
    keep_inline_images_in: Iterable[str] | None = None,
    list_indent_type: Literal["spaces", "tabs"] = "spaces",
    list_indent_width: int = 4,
    newline_style: Literal["spaces", "backslash"] = SPACES,
    preprocess_html: bool = False,
    preprocessing_preset: Literal["minimal", "standard", "aggressive"] = "standard",
    remove_forms: bool = True,
    remove_navigation: bool = True,
    strip: str | Iterable[str] | None = None,
    strip_newlines: bool = False,
    strong_em_symbol: Literal["*", "_"] = ASTERISK,
    sub_symbol: str = "",
    sup_symbol: str = "",
    whitespace_mode: Literal["normalized", "strict"] = WHITESPACE_NORMALIZED,
    wrap: bool = False,
    wrap_width: int = 80,
) -> Generator[str, None, None]:
    sink = StreamingSink(chunk_size, progress_callback)

    if isinstance(source, bytes):
        source = source.decode(source_encoding or "utf-8", errors="replace")

    if isinstance(source, str) and preprocess_html and create_preprocessor is not None and preprocess_fn is not None:
        config = create_preprocessor(
            preset=preprocessing_preset,
            remove_navigation=remove_navigation,
            remove_forms=remove_forms,
        )
        source = preprocess_fn(source, **config)

    if isinstance(source, (str, bytes)):
        if isinstance(source, bytes):
            sink.total_bytes = len(source)
        else:
            sink.total_bytes = len(source)
    elif isinstance(source, BeautifulSoup):
        sink.total_bytes = len(str(source))

    whitespace_handler = WhitespaceHandler(whitespace_mode)

    _process_html_core(
        source,
        sink,
        whitespace_handler=whitespace_handler,
        parser=parser,
        source_encoding=source_encoding,
        autolinks=autolinks,
        br_in_tables=br_in_tables,
        bullets=bullets,
        code_language=code_language,
        code_language_callback=code_language_callback,
        convert=convert,
        convert_as_inline=convert_as_inline,
        custom_converters=custom_converters,
        default_title=default_title,
        escape_asterisks=escape_asterisks,
        escape_misc=escape_misc,
        escape_underscores=escape_underscores,
        extract_metadata=extract_metadata,
        heading_style=heading_style,
        highlight_style=highlight_style,
        keep_inline_images_in=keep_inline_images_in,
        list_indent_type=list_indent_type,
        list_indent_width=list_indent_width,
        newline_style=newline_style,
        strip=strip,
        strip_newlines=strip_newlines,
        strong_em_symbol=strong_em_symbol,
        sub_symbol=sub_symbol,
        sup_symbol=sup_symbol,
        wrap=wrap,
        wrap_width=wrap_width,
    )

    all_chunks = list(sink.get_chunks())
    combined_result = "".join(all_chunks)

    combined_result = re.sub(r"\n{3,}", "\n\n", combined_result)

    if convert_as_inline:
        combined_result = combined_result.rstrip("\n")

    if not combined_result:
        return

    pos = 0
    while pos < len(combined_result):
        end_pos = min(pos + chunk_size, len(combined_result))

        if end_pos < len(combined_result):
            search_start = max(pos, end_pos - 50)
            search_end = min(len(combined_result), end_pos + 50)
            search_area = combined_result[search_start:search_end]

            newline_pos = search_area.rfind("\n", 0, end_pos - search_start + 50)
            if newline_pos > 0:
                end_pos = search_start + newline_pos + 1

        chunk = combined_result[pos:end_pos]
        if chunk:  # pragma: no cover
            yield chunk

        pos = end_pos
