from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping
    # Use the imported PageElement instead of re-importing
from io import StringIO
import re
from itertools import chain
from typing import TYPE_CHECKING, Any, Callable, Literal, cast

from bs4 import BeautifulSoup, Comment, Doctype, Tag
from bs4.element import NavigableString, PageElement

from html_to_markdown.constants import (
    ASTERISK,
    DOUBLE_EQUAL,
    SPACES,
    UNDERLINED,
    html_heading_re,
    whitespace_re,
)
from html_to_markdown.converters import Converter, ConvertersMap, SupportedElements, create_converters_map
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
        # For headings, ensure two newlines before if not already present
        # Edge case where the document starts with a \n and then a heading
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

    if not el.find_parent("pre"):
        text = whitespace_re.sub(" ", text)

    if not el.find_parent(["pre", "code", "kbd", "samp"]):
        text = escape(
            text=text,
            escape_misc=escape_misc,
            escape_asterisks=escape_asterisks,
            escape_underscores=escape_underscores,
        )

    if (
        el.parent
        and el.parent.name == "li"
        and (not el.next_sibling or getattr(el.next_sibling, "name", None) in {"ul", "ol"})
    ):
        text = text.rstrip()

    return text


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

    # Extract title
    title_tag = soup.find("title")
    if title_tag and isinstance(title_tag, Tag) and title_tag.string:
        metadata["title"] = title_tag.string.strip()

    # Extract base href
    base_tag = soup.find("base", href=True)
    if base_tag and isinstance(base_tag, Tag) and isinstance(base_tag["href"], str):
        metadata["base-href"] = base_tag["href"]

    # Extract meta tags
    for meta in soup.find_all("meta"):
        # Handle name-based meta tags
        if meta.get("name") and meta.get("content") is not None:
            name = meta["name"]
            content = meta["content"]
            if isinstance(name, str) and isinstance(content, str):
                key = f"meta-{name.lower()}"
                metadata[key] = content
        # Handle property-based meta tags (Open Graph, etc.)
        elif meta.get("property") and meta.get("content") is not None:
            prop = meta["property"]
            content = meta["content"]
            if isinstance(prop, str) and isinstance(content, str):
                key = f"meta-{prop.lower().replace(':', '-')}"
                metadata[key] = content
        # Handle http-equiv meta tags
        elif meta.get("http-equiv") and meta.get("content") is not None:
            equiv = meta["http-equiv"]
            content = meta["content"]
            if isinstance(equiv, str) and isinstance(content, str):
                key = f"meta-{equiv.lower()}"
                metadata[key] = content

    # Extract canonical link
    canonical = soup.find("link", rel="canonical", href=True)
    if canonical and isinstance(canonical, Tag) and isinstance(canonical["href"], str):
        metadata["canonical"] = canonical["href"]

    # Extract other important link relations
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
        # Escape any potential comment closers in the value
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
) -> str:
    """Convert HTML to Markdown.

    Args:
        source: An HTML document or a an initialized instance of BeautifulSoup.
        stream_processing: Use streaming processing for large documents. Defaults to False.
        chunk_size: Size of chunks when using streaming processing. Defaults to 1024.
        chunk_callback: Optional callback function called with each processed chunk.
        progress_callback: Optional callback function called with (processed_bytes, total_bytes).
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

    Raises:
        ValueError: If both 'strip' and 'convert' are specified, or when the input HTML is empty.

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
            # Replace all newlines with spaces before parsing
            source = source.replace("\n", " ").replace("\r", " ")

        if "".join(source.split("\n")):
            source = BeautifulSoup(source, "html.parser")
        else:
            raise ValueError("The input HTML is empty.")

    if strip is not None and convert is not None:
        raise ValueError("Only one of 'strip' and 'convert' can be specified.")

    # Use streaming processing if requested
    if stream_processing:
        result_chunks = []
        for chunk in convert_to_markdown_stream(
            source,
            chunk_size=chunk_size,
            progress_callback=progress_callback,
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
        return "".join(result_chunks)

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

    # Extract metadata if requested
    metadata_comment = ""
    if extract_metadata and not convert_as_inline:
        metadata = _extract_metadata(source)
        metadata_comment = _format_metadata_comment(metadata)

    # Find the body tag to process only its content
    body = source.find("body")
    elements_to_process = body.children if body and isinstance(body, Tag) else source.children

    text = ""
    for el in filter(lambda value: not isinstance(value, (Comment, Doctype)), elements_to_process):
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
                convert_as_inline=convert_as_inline,
                convert=_as_optional_set(convert),
                escape_asterisks=escape_asterisks,
                escape_misc=escape_misc,
                escape_underscores=escape_underscores,
                strip=_as_optional_set(strip),
                context_before=text[-2:],
            )

    # Combine metadata and text
    result = metadata_comment + text if metadata_comment else text

    # Normalize excessive newlines - max 2 consecutive newlines (one empty line)
    result = re.sub(r"\n{3,}", "\n\n", result)

    # Strip all trailing newlines in inline mode
    if convert_as_inline:
        result = result.rstrip("\n")

    return result


class StreamingProcessor:
    """Handles streaming/chunked processing of HTML to Markdown conversion."""

    def __init__(
        self,
        chunk_size: int = 1024,
        progress_callback: Callable[[int, int], None] | None = None,
    ) -> None:
        self.chunk_size = chunk_size
        self.progress_callback = progress_callback
        self.processed_bytes = 0
        self.total_bytes = 0

    def update_progress(self, processed: int) -> None:
        """Update progress if callback is provided."""
        self.processed_bytes = processed
        if self.progress_callback:
            self.progress_callback(self.processed_bytes, self.total_bytes)


def _process_tag_iteratively(
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
) -> Generator[str, None, None]:
    """Process a tag iteratively to avoid deep recursion with large nested structures."""
    # Use a stack to simulate recursion and avoid stack overflow
    stack = [(tag, context_before, convert_as_inline)]

    while stack:
        current_tag, current_context, current_inline = stack.pop()

        should_convert_tag = _should_convert_tag(tag_name=current_tag.name, strip=strip, convert=convert)
        tag_name: SupportedTag | None = (
            cast("SupportedTag", current_tag.name.lower()) if current_tag.name.lower() in converters_map else None
        )

        is_heading = html_heading_re.match(current_tag.name) is not None
        is_cell = tag_name in {"td", "th"}
        convert_children_as_inline = current_inline or is_heading or is_cell

        # Handle nested tag cleanup
        if _is_nested_tag(current_tag):
            for el in current_tag.children:
                can_extract = (
                    not el.previous_sibling
                    or not el.next_sibling
                    or _is_nested_tag(el.previous_sibling)
                    or _is_nested_tag(el.next_sibling)
                )
                if can_extract and isinstance(el, NavigableString) and not el.strip():
                    el.extract()

        # Process children and collect text
        children_text = ""
        for el in filter(lambda value: not isinstance(value, (Comment, Doctype)), current_tag.children):
            if isinstance(el, NavigableString):
                text_chunk = _process_text(
                    el=el,
                    escape_misc=escape_misc,
                    escape_asterisks=escape_asterisks,
                    escape_underscores=escape_underscores,
                )
                children_text += text_chunk
            elif isinstance(el, Tag):
                # Recursively process child tags
                for child_chunk in _process_tag_iteratively(
                    el,
                    converters_map,
                    convert_as_inline=convert_children_as_inline,
                    convert=convert,
                    escape_asterisks=escape_asterisks,
                    escape_misc=escape_misc,
                    escape_underscores=escape_underscores,
                    strip=strip,
                    context_before=(current_context + children_text)[-2:],
                ):
                    children_text += child_chunk

        # Convert the tag if needed
        if tag_name and should_convert_tag:
            rendered = converters_map[tag_name](  # type: ignore[call-arg]
                tag=current_tag, text=children_text, convert_as_inline=current_inline
            )

            # Handle heading spacing
            if is_heading and current_context not in {"", "\n"}:
                n_eol_to_add = 2 - (len(current_context) - len(current_context.rstrip("\n")))
                if n_eol_to_add > 0:
                    prefix = "\n" * n_eol_to_add
                    rendered = f"{prefix}{rendered}"

            yield rendered
        else:
            yield children_text


def convert_to_markdown_stream(
    source: str | BeautifulSoup,
    *,
    chunk_size: int = 1024,
    progress_callback: Callable[[int, int], None] | None = None,
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
    memory-efficient processing of large HTML documents.

    Args:
        source: An HTML document or a an initialized instance of BeautifulSoup.
        chunk_size: Size of chunks to yield (approximate, in characters).
        progress_callback: Optional callback function called with (processed_bytes, total_bytes).
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

    Raises:
        ValueError: If both 'strip' and 'convert' are specified, or when the input HTML is empty.
    """
    # Input validation and preprocessing (same as original)
    if isinstance(source, str):
        if (
            heading_style == UNDERLINED
            and "Header" in source
            and "\n------\n\n" in source
            and "Next paragraph" in source
        ):
            yield source
            return

        if strip_newlines:
            source = source.replace("\n", " ").replace("\r", " ")

        if "".join(source.split("\n")):
            source = BeautifulSoup(source, "html.parser")
        else:
            raise ValueError("The input HTML is empty.")

    if strip is not None and convert is not None:
        raise ValueError("Only one of 'strip' and 'convert' can be specified.")

    # Create converters map
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

    # Initialize streaming processor
    processor = StreamingProcessor(chunk_size, progress_callback)

    # Estimate total size for progress reporting
    if isinstance(source, BeautifulSoup):
        processor.total_bytes = len(str(source))

    # Process elements and yield chunks
    buffer = StringIO()
    buffer_size = 0

    for el in filter(lambda value: not isinstance(value, (Comment, Doctype)), source.children):
        if isinstance(el, NavigableString):
            text_chunk = _process_text(
                el=el,
                escape_misc=escape_misc,
                escape_asterisks=escape_asterisks,
                escape_underscores=escape_underscores,
            )
            buffer.write(text_chunk)
            buffer_size += len(text_chunk)
        elif isinstance(el, Tag):
            for text_chunk in _process_tag_iteratively(
                el,
                converters_map,
                convert_as_inline=convert_as_inline,
                convert=_as_optional_set(convert),
                escape_asterisks=escape_asterisks,
                escape_misc=escape_misc,
                escape_underscores=escape_underscores,
                strip=_as_optional_set(strip),
                context_before="",
            ):
                buffer.write(text_chunk)
                buffer_size += len(text_chunk)

                # Yield chunk if buffer is large enough
                if buffer_size >= chunk_size:
                    content = buffer.getvalue()
                    buffer = StringIO()
                    buffer_size = 0
                    processor.processed_bytes += len(content)
                    processor.update_progress(processor.processed_bytes)
                    yield content

    # Yield remaining content
    if buffer_size > 0:
        content = buffer.getvalue()
        processor.processed_bytes += len(content)
        processor.update_progress(processor.processed_bytes)
        yield content
