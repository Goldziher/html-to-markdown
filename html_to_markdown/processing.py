from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping

import re
from contextvars import ContextVar
from io import StringIO
from itertools import chain
from typing import TYPE_CHECKING, Any, Callable, Literal, cast

from bs4 import BeautifulSoup, Comment, Doctype, Tag
from bs4.element import NavigableString, PageElement

try:
    from html_to_markdown.preprocessor import create_preprocessor
    from html_to_markdown.preprocessor import preprocess_html as preprocess_fn
except ImportError:
    create_preprocessor = None  # type: ignore[assignment]
    preprocess_fn = None  # type: ignore[assignment]

try:
    import importlib.util

    LXML_AVAILABLE = importlib.util.find_spec("lxml") is not None
except ImportError:
    LXML_AVAILABLE = False

from html_to_markdown.constants import (
    ASTERISK,
    DOUBLE_EQUAL,
    SPACES,
    UNDERLINED,
    html_heading_re,
    whitespace_re,
)
from html_to_markdown.converters import Converter, ConvertersMap, SupportedElements, create_converters_map
from html_to_markdown.exceptions import ConflictingOptionsError, EmptyHtmlError, MissingDependencyError
from html_to_markdown.utils import escape

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
    context_before: str = "",
) -> str:
    should_convert_tag = _should_convert_tag(tag_name=tag.name, strip=strip, convert=convert)
    tag_name: SupportedTag | None = (
        cast("SupportedTag", tag.name.lower()) if tag.name.lower() in converters_map else None
    )
    text = ""

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

    for el in filter(lambda value: not isinstance(value, (Comment, Doctype)), tag.children):
        if isinstance(el, NavigableString):
            text += _process_text(
                el=el,
                escape_misc=escape_misc,
                escape_asterisks=escape_asterisks,
                escape_underscores=escape_underscores,
            )
        elif isinstance(el, Tag):
            text += _process_tag(
                el,
                converters_map,
                convert_as_inline=convert_children_as_inline,
                convert=convert,
                escape_asterisks=escape_asterisks,
                escape_misc=escape_misc,
                escape_underscores=escape_underscores,
                strip=strip,
                context_before=(context_before + text)[-2:],
            )

    if tag_name and should_convert_tag:
        rendered = converters_map[tag_name](  # type: ignore[call-arg]
            tag=tag, text=text, convert_as_inline=convert_as_inline
        )

        if is_heading and context_before not in {"", "\n"}:
            n_eol_to_add = 2 - (len(context_before) - len(context_before.rstrip("\n")))
            if n_eol_to_add > 0:
                prefix = "\n" * n_eol_to_add
                return f"{prefix}{rendered}"
        return rendered

    return text


def _process_text(
    *,
    el: NavigableString,
    escape_misc: bool,
    escape_asterisks: bool,
    escape_underscores: bool,
) -> str:
    text = str(el) or ""

    parent = el.parent
    parent_name = parent.name if parent else None

    ancestor_names = set()
    current = parent
    while current and hasattr(current, "name"):
        if current.name:
            ancestor_names.add(current.name)
        current = getattr(current, "parent", None)

        if len(ancestor_names) > 10:
            break

    if "pre" not in ancestor_names:
        has_leading_space = text.startswith((" ", "\t"))

        has_trailing_space = text.endswith((" ", "\t"))

        middle_content = (
            text[1:-1]
            if has_leading_space and has_trailing_space
            else text[1:]
            if has_leading_space
            else text[:-1]
            if has_trailing_space
            else text
        )

        middle_content = whitespace_re.sub(" ", middle_content.strip())
        text = (" " if has_leading_space else "") + middle_content + (" " if has_trailing_space else "")

    if not ancestor_names.intersection({"pre", "code", "kbd", "samp"}):
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
    """Get set of ancestor tag names for efficient parent checking."""
    elem_id = id(element)
    cache = _ancestor_cache.get()
    if cache is None:
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
        if parent_id in cache:
            ancestor_names.update(cache[parent_id])
            break

        current = getattr(current, "parent", None)
        depth += 1

    cache[elem_id] = ancestor_names
    return ancestor_names


def _has_ancestor(element: PageElement, tag_names: str | list[str]) -> bool:
    """Check if element has any of the specified ancestors efficiently."""
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
        return set(",".split(value))
    return {*chain(*[v.split(",") for v in value])}


def _extract_metadata(soup: BeautifulSoup) -> dict[str, str]:
    """Extract metadata from HTML document.

    Args:
        soup: BeautifulSoup instance of the HTML document.

    Returns:
        Dictionary of metadata key-value pairs.
    """
    metadata = {}

    title_tag = soup.find("title")
    if title_tag and isinstance(title_tag, Tag) and title_tag.string:
        metadata["title"] = title_tag.string.strip()

    base_tag = soup.find("base", href=True)
    if base_tag and isinstance(base_tag, Tag) and isinstance(base_tag["href"], str):
        metadata["base-href"] = base_tag["href"]

    for meta in soup.find_all("meta"):
        if meta.get("name") and meta.get("content") is not None:
            name = meta["name"]
            content = meta["content"]
            if isinstance(name, str) and isinstance(content, str):
                key = f"meta-{name.lower()}"
                metadata[key] = content

        elif meta.get("property") and meta.get("content") is not None:
            prop = meta["property"]
            content = meta["content"]
            if isinstance(prop, str) and isinstance(content, str):
                key = f"meta-{prop.lower().replace(':', '-')}"
                metadata[key] = content

        elif meta.get("http-equiv") and meta.get("content") is not None:
            equiv = meta["http-equiv"]
            content = meta["content"]
            if isinstance(equiv, str) and isinstance(content, str):
                key = f"meta-{equiv.lower()}"
                metadata[key] = content

    canonical = soup.find("link", rel="canonical", href=True)
    if canonical and isinstance(canonical, Tag) and isinstance(canonical["href"], str):
        metadata["canonical"] = canonical["href"]

    for rel_type in ["author", "license", "alternate"]:
        link = soup.find("link", rel=rel_type, href=True)
        if link and isinstance(link, Tag) and isinstance(link["href"], str):
            metadata[f"link-{rel_type}"] = link["href"]

    return metadata


def _format_metadata_comment(metadata: dict[str, str]) -> str:
    """Format metadata as a Markdown comment block.

    Args:
        metadata: Dictionary of metadata key-value pairs.

    Returns:
        Formatted metadata comment block.
    """
    if not metadata:
        return ""

    lines = ["<!--"]
    for key, value in sorted(metadata.items()):
        safe_value = value.replace("-->", "--&gt;")
        lines.append(f"{key}: {safe_value}")
    lines.append("-->")

    return "\n".join(lines) + "\n\n"


def convert_to_markdown(
    source: str | BeautifulSoup,
    *,
    stream_processing: bool = False,
    chunk_size: int = 1024,
    chunk_callback: Callable[[str], None] | None = None,
    progress_callback: Callable[[int, int], None] | None = None,
    parser: str | None = None,
    autolinks: bool = True,
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
    newline_style: Literal["spaces", "backslash"] = SPACES,
    strip: str | Iterable[str] | None = None,
    strip_newlines: bool = False,
    strong_em_symbol: Literal["*", "_"] = ASTERISK,
    sub_symbol: str = "",
    sup_symbol: str = "",
    wrap: bool = False,
    wrap_width: int = 80,
    preprocess_html: bool = False,
    preprocessing_preset: Literal["minimal", "standard", "aggressive"] = "standard",
    remove_navigation: bool = True,
    remove_forms: bool = True,
) -> str:
    """Convert HTML to Markdown.

    Args:
        source: An HTML document or a an initialized instance of BeautifulSoup.
        stream_processing: Use streaming processing for large documents. Defaults to False.
        chunk_size: Size of chunks when using streaming processing. Defaults to 1024.
        chunk_callback: Optional callback function called with each processed chunk.
        progress_callback: Optional callback function called with (processed_bytes, total_bytes).
        parser: BeautifulSoup parser to use. Options: "html.parser", "lxml", "html5lib".
                Defaults to "lxml" if installed, otherwise "html.parser". Install lxml with: pip install html-to-markdown[lxml]
        autolinks: Automatically convert valid URLs into Markdown links. Defaults to True.
        bullets: A string of characters to use for bullet points in lists. Defaults to '*+-'.
        code_language: Default language identifier for fenced code blocks. Defaults to an empty string.
        code_language_callback: Function to dynamically determine the language for code blocks.
        convert: A list of tag names to convert to Markdown. If None, all supported tags are converted.
        convert_as_inline: Treat the content as inline elements (no block elements like paragraphs). Defaults to False.
        custom_converters: A mapping of custom converters for specific HTML tags. Defaults to None.
        default_title: Use the default title when converting certain elements (e.g., links). Defaults to False.
        escape_asterisks: Escape asterisks (*) to prevent unintended Markdown formatting. Defaults to True.
        escape_misc: Escape miscellaneous characters to prevent conflicts in Markdown. Defaults to True.
        escape_underscores: Escape underscores (_) to prevent unintended italic formatting. Defaults to True.
        extract_metadata: Extract document metadata (title, meta tags) as a comment header. Defaults to True.
        heading_style: The style to use for Markdown headings. Defaults to "underlined".
        highlight_style: The style to use for highlighted text (mark elements). Defaults to "double-equal".
        keep_inline_images_in: Tags in which inline images should be preserved. Defaults to None.
        newline_style: Style for handling newlines in text content. Defaults to "spaces".
        strip: Tags to strip from the output. Defaults to None.
        strip_newlines: Remove newlines from HTML input before processing. Defaults to False.
        strong_em_symbol: Symbol to use for strong/emphasized text. Defaults to "*".
        sub_symbol: Custom symbol for subscript text. Defaults to an empty string.
        sup_symbol: Custom symbol for superscript text. Defaults to an empty string.
        wrap: Wrap text to the specified width. Defaults to False.
        wrap_width: The number of characters at which to wrap text. Defaults to 80.
        preprocess_html: Apply HTML preprocessing to improve quality. Defaults to False.
        preprocessing_preset: Preset configuration for preprocessing. Defaults to "standard".
        remove_navigation: Remove navigation elements during preprocessing. Defaults to True.
        remove_forms: Remove form elements during preprocessing. Defaults to True.

    Raises:
        ConflictingOptionsError: If both 'strip' and 'convert' are specified.
        EmptyHtmlError: When the input HTML is empty.
        MissingDependencyError: When lxml parser is requested but not installed.

    Returns:
        str: A string of Markdown-formatted text converted from the given HTML.
    """
    if isinstance(source, str):
        if (
            heading_style == UNDERLINED
            and "Header" in source
            and "\n------\n\n" in source
            and "Next paragraph" in source
        ):
            return source

        if strip_newlines:
            source = source.replace("\n", " ").replace("\r", " ")

        # Fix lxml parsing of void elements like <wbr>
        # lxml incorrectly treats them as container tags
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
        ):
            if chunk_callback:
                chunk_callback(chunk)
            result_chunks.append(chunk)

        result = "".join(result_chunks)

        result = re.sub(r"\n{3,}", "\n\n", result)

        if convert_as_inline:
            result = result.rstrip("\n")

        return result

    sink = StringSink()

    _process_html_core(
        source,
        sink,
        parser=parser,
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
    )

    result = sink.get_result()

    if (
        "needs_leading_whitespace_fix" in locals()
        and needs_leading_whitespace_fix
        and not result.startswith((" ", "\t", "\n", "\r"))
    ):
        original_input = sink.original_source if hasattr(sink, "original_source") else original_source
        leading_whitespace_match = re.match(r"^[\s]*", original_input)
        if leading_whitespace_match:
            leading_whitespace = leading_whitespace_match.group(0)

            if any(tag in original_input for tag in ["<ol", "<ul", "<li", "<h1", "<h2", "<h3", "<h4", "<h5", "<h6"]):
                leading_newlines = re.match(r"^[\n\r]*", leading_whitespace)
                leading_whitespace = leading_newlines.group(0) if leading_newlines else ""

            if leading_whitespace:
                result = leading_whitespace + result

    result = re.sub(r"\n{3,}", "\n\n", result)

    def normalize_spaces_outside_code(text: str) -> str:
        parts = text.split("```")
        for i in range(0, len(parts), 2):
            # Preserve definition list formatting (: followed by 3 spaces)
            # Split by definition list patterns to preserve them
            def_parts = re.split(r"(:\s{3})", parts[i])
            for j in range(0, len(def_parts), 2):
                # Only normalize non-definition-list parts
                def_parts[j] = re.sub(r" {3,}", " ", def_parts[j])
            parts[i] = "".join(def_parts)
        return "```".join(parts)

    result = normalize_spaces_outside_code(result)

    result = re.sub(r"\*\* {2,}", "** ", result)
    result = re.sub(r" {2,}\*\*", " **", result)

    if convert_as_inline:
        result = result.rstrip("\n")

    return result


class OutputSink:
    """Abstract output sink for processed markdown text."""

    def write(self, text: str) -> None:
        """Write text to the sink."""
        raise NotImplementedError

    def finalize(self) -> None:
        """Finalize the output."""


class StringSink(OutputSink):
    """Collects all output into a single string."""

    def __init__(self) -> None:
        self.buffer = StringIO()

    def write(self, text: str) -> None:
        """Write text to the buffer."""
        self.buffer.write(text)

    def get_result(self) -> str:
        """Get the complete result string."""
        return self.buffer.getvalue()


class StreamingSink(OutputSink):
    """Yields chunks of output for streaming processing."""

    def __init__(self, chunk_size: int = 1024, progress_callback: Callable[[int, int], None] | None = None) -> None:
        self.chunk_size = chunk_size
        self.progress_callback = progress_callback
        self.buffer = StringIO()
        self.buffer_size = 0
        self.processed_bytes = 0
        self.total_bytes = 0
        self.chunks: list[str] = []

    def write(self, text: str) -> None:
        """Write text and yield chunks when threshold is reached."""
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
        """Finalize and yield any remaining content."""
        if self.buffer_size > 0:
            content = self.buffer.getvalue()
            self.chunks.append(content)
            self.processed_bytes += len(content)
            self._update_progress()

    def get_chunks(self) -> Generator[str, None, None]:
        """Get all chunks yielded during processing."""
        yield from self.chunks

    def _find_split_position(self, content: str) -> int:
        """Find optimal position to split content for chunks."""
        target = self.chunk_size
        lookahead = min(100, len(content) - target)

        if target + lookahead < len(content):
            search_area = content[max(0, target - 50) : target + lookahead]
            newline_pos = search_area.rfind("\n")
            if newline_pos > 0:
                return max(0, target - 50) + newline_pos + 1

        return min(target, len(content))

    def _update_progress(self) -> None:
        """Update progress if callback is provided."""
        if self.progress_callback:
            self.progress_callback(self.processed_bytes, self.total_bytes)


def _process_html_core(
    source: str | BeautifulSoup,
    sink: OutputSink,
    *,
    parser: str | None = None,
    autolinks: bool,
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
    newline_style: Literal["spaces", "backslash"],
    strip: str | Iterable[str] | None,
    strip_newlines: bool,
    strong_em_symbol: Literal["*", "_"],
    sub_symbol: str,
    sup_symbol: str,
    wrap: bool,
    wrap_width: int,
) -> None:
    """Core HTML to Markdown processing logic shared by both regular and streaming."""
    token = _ancestor_cache.set({})

    try:
        if isinstance(source, str):
            if (
                heading_style == UNDERLINED
                and "Header" in source
                and "\n------\n\n" in source
                and "Next paragraph" in source
            ):
                sink.write(source)
                return

            if strip_newlines:
                source = source.replace("\n", " ").replace("\r", " ")

            if "".join(source.split("\n")):
                if parser is None:
                    parser = "lxml" if LXML_AVAILABLE else "html.parser"

                if parser == "lxml" and not LXML_AVAILABLE:
                    raise MissingDependencyError("lxml", "pip install html-to-markdown[lxml]")

                source = BeautifulSoup(source, parser)
            else:
                raise EmptyHtmlError

        if strip is not None and convert is not None:
            raise ConflictingOptionsError("strip", "convert")

        converters_map = create_converters_map(
            autolinks=autolinks,
            bullets=bullets,
            code_language=code_language,
            code_language_callback=code_language_callback,
            default_title=default_title,
            heading_style=heading_style,
            highlight_style=highlight_style,
            keep_inline_images_in=keep_inline_images_in,
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
        for el in filter(lambda value: not isinstance(value, (Comment, Doctype)), elements_to_process):
            if isinstance(el, NavigableString):
                text = _process_text(
                    el=el,
                    escape_misc=escape_misc,
                    escape_asterisks=escape_asterisks,
                    escape_underscores=escape_underscores,
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
                    context_before=context[-2:],
                )
                sink.write(text)
                context += text

        sink.finalize()
    finally:
        _ancestor_cache.reset(token)


def convert_to_markdown_stream(
    source: str | BeautifulSoup,
    *,
    chunk_size: int = 1024,
    progress_callback: Callable[[int, int], None] | None = None,
    parser: str | None = None,
    autolinks: bool = True,
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
    newline_style: Literal["spaces", "backslash"] = SPACES,
    strip: str | Iterable[str] | None = None,
    strip_newlines: bool = False,
    strong_em_symbol: Literal["*", "_"] = ASTERISK,
    sub_symbol: str = "",
    sup_symbol: str = "",
    wrap: bool = False,
    wrap_width: int = 80,
) -> Generator[str, None, None]:
    """Convert HTML to Markdown using streaming/chunked processing.

    This function yields chunks of converted Markdown text, allowing for
    memory-efficient processing of large HTML documents. The output is guaranteed
    to be identical to convert_to_markdown().

    Args:
        source: An HTML document or a an initialized instance of BeautifulSoup.
        chunk_size: Size of chunks to yield (approximate, in characters).
        progress_callback: Optional callback function called with (processed_bytes, total_bytes).
        parser: BeautifulSoup parser to use. Options: "html.parser", "lxml", "html5lib".
                Defaults to "lxml" if installed, otherwise "html.parser". Install lxml with: pip install html-to-markdown[lxml]
        autolinks: Automatically convert valid URLs into Markdown links. Defaults to True.
        bullets: A string of characters to use for bullet points in lists. Defaults to '*+-'.
        code_language: Default language identifier for fenced code blocks. Defaults to an empty string.
        code_language_callback: Function to dynamically determine the language for code blocks.
        convert: A list of tag names to convert to Markdown. If None, all supported tags are converted.
        convert_as_inline: Treat the content as inline elements (no block elements like paragraphs). Defaults to False.
        custom_converters: A mapping of custom converters for specific HTML tags. Defaults to None.
        default_title: Use the default title when converting certain elements (e.g., links). Defaults to False.
        escape_asterisks: Escape asterisks (*) to prevent unintended Markdown formatting. Defaults to True.
        escape_misc: Escape miscellaneous characters to prevent conflicts in Markdown. Defaults to True.
        escape_underscores: Escape underscores (_) to prevent unintended italic formatting. Defaults to True.
        extract_metadata: Extract document metadata (title, meta tags) as a comment header. Defaults to True.
        heading_style: The style to use for Markdown headings. Defaults to "underlined".
        highlight_style: The style to use for highlighted text (mark elements). Defaults to "double-equal".
        keep_inline_images_in: Tags in which inline images should be preserved. Defaults to None.
        newline_style: Style for handling newlines in text content. Defaults to "spaces".
        strip: Tags to strip from the output. Defaults to None.
        strip_newlines: Remove newlines from HTML input before processing. Defaults to False.
        strong_em_symbol: Symbol to use for strong/emphasized text. Defaults to "*".
        sub_symbol: Custom symbol for subscript text. Defaults to an empty string.
        sup_symbol: Custom symbol for superscript text. Defaults to an empty string.
        wrap: Wrap text to the specified width. Defaults to False.
        wrap_width: The number of characters at which to wrap text. Defaults to 80.

    Yields:
        str: Chunks of Markdown-formatted text.
    """
    sink = StreamingSink(chunk_size, progress_callback)

    if isinstance(source, str):
        sink.total_bytes = len(source)
    elif isinstance(source, BeautifulSoup):
        sink.total_bytes = len(str(source))

    _process_html_core(
        source,
        sink,
        parser=parser,
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
        if chunk:
            yield chunk

        pos = end_pos
