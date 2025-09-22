import sys
from argparse import ArgumentParser
from pathlib import Path

from html_to_markdown.constants import (
    ASTERISK,
    ATX,
    ATX_CLOSED,
    BACKSLASH,
    DOUBLE_EQUAL,
    SPACES,
    UNDERLINED,
    UNDERSCORE,
    WHITESPACE_NORMALIZED,
    WHITESPACE_STRICT,
)
from html_to_markdown.exceptions import InvalidEncodingError
from html_to_markdown.processing import convert_to_markdown


def main(argv: list[str]) -> str:
    parser = ArgumentParser(
        prog="html_to_markdown",
        description="Convert HTML to Markdown with comprehensive customization options.",
    )

    parser.add_argument(
        "html",
        nargs="?",
        default="-",
        help="The HTML file to convert. Defaults to STDIN if not provided.",
    )

    parser.add_argument(
        "-s",
        "--strip",
        nargs="*",
        help="HTML tags to remove from output. Incompatible with --convert.",
    )

    parser.add_argument(
        "-c",
        "--convert",
        nargs="*",
        help="HTML tags to convert (only these will be processed). Incompatible with --strip.",
    )

    parser.add_argument(
        "-a",
        "--autolinks",
        action="store_true",
        help="Convert URLs to automatic links when text matches href.",
    )

    parser.add_argument(
        "--default-title",
        action="store_true",
        help="Use href as link title when no title is provided.",
    )

    parser.add_argument(
        "--heading-style",
        default=UNDERLINED,
        choices=(ATX, ATX_CLOSED, UNDERLINED),
        help="Header style: 'atx' (#), 'atx_closed' (# #), or 'underlined' (===). Default: underlined.",
    )

    parser.add_argument(
        "-b",
        "--bullets",
        default="*+-",
        help="Characters for bullet points, alternates by nesting level. Default: '*+-'.",
    )

    parser.add_argument(
        "--strong-em-symbol",
        default=ASTERISK,
        choices=(ASTERISK, UNDERSCORE),
        help="Symbol for bold/italic text: '*' or '_'. Default: '*'.",
    )

    parser.add_argument(
        "--sub-symbol",
        default="",
        help="Characters to surround subscript text. Default: none.",
    )

    parser.add_argument(
        "--sup-symbol",
        default="",
        help="Characters to surround superscript text. Default: none.",
    )

    parser.add_argument(
        "--newline-style",
        default=SPACES,
        choices=(SPACES, BACKSLASH),
        help="Line break style: 'spaces' (two spaces) or 'backslash' (\\). Default: spaces.",
    )

    parser.add_argument(
        "--code-language",
        default="",
        help="Default language for code blocks. Default: none.",
    )

    parser.add_argument(
        "--no-escape-asterisks",
        dest="escape_asterisks",
        action="store_false",
        help="Don't escape asterisk (*) characters.",
    )

    parser.add_argument(
        "--no-escape-underscores",
        dest="escape_underscores",
        action="store_false",
        help="Don't escape underscore (_) characters.",
    )

    parser.add_argument(
        "--no-escape-misc",
        dest="escape_misc",
        action="store_false",
        help="Don't escape other special Markdown characters.",
    )

    parser.add_argument(
        "-i",
        "--keep-inline-images-in",
        nargs="*",
        help="Parent tags where images remain inline (not converted to alt-text).",
    )

    parser.add_argument(
        "--br-in-tables",
        action="store_true",
        help="Use <br> tags for line breaks in table cells instead of spaces.",
    )

    parser.add_argument("-w", "--wrap", action="store_true", help="Enable text wrapping at --wrap-width characters.")

    parser.add_argument(
        "--wrap-width",
        type=int,
        default=80,
        help="Column width for text wrapping. Default: 80.",
    )

    parser.add_argument(
        "--strip-newlines",
        action="store_true",
        help="Remove newlines from HTML input (helps with messy HTML formatting).",
    )

    parser.add_argument(
        "--convert-as-inline",
        action="store_true",
        help="Treat all content as inline elements (no paragraph breaks).",
    )

    parser.add_argument(
        "--no-extract-metadata",
        dest="extract_metadata",
        action="store_false",
        help="Don't extract metadata (title, meta tags) as comment header.",
    )

    parser.add_argument(
        "--highlight-style",
        default=DOUBLE_EQUAL,
        choices=("double-equal", "html", "bold"),
        help="Highlighting style: 'double-equal' (==), 'html' (<mark>), or 'bold' (**). Default: double-equal.",
    )

    parser.add_argument(
        "--stream-processing",
        action="store_true",
        help="Process large documents in chunks to reduce memory usage.",
    )

    parser.add_argument(
        "--chunk-size",
        type=int,
        default=1024,
        help="Chunk size for streaming processing. Default: 1024 characters.",
    )

    parser.add_argument("--show-progress", action="store_true", help="Show progress bar for large documents.")

    parser.add_argument(
        "--parser",
        choices=("html.parser", "lxml", "html5lib"),
        help="HTML parser: 'lxml', 'html.parser', or 'html5lib'. Default: auto-detect.",
    )

    parser.add_argument(
        "--list-indent-type",
        default="spaces",
        choices=("spaces", "tabs"),
        help="List indentation: 'spaces' or 'tabs'. Default: spaces.",
    )

    parser.add_argument(
        "--list-indent-width",
        type=int,
        default=4,
        help="Spaces per list indent level (use 2 for Discord/Slack). Default: 4.",
    )

    parser.add_argument(
        "--whitespace-mode",
        default=WHITESPACE_NORMALIZED,
        choices=(WHITESPACE_NORMALIZED, WHITESPACE_STRICT),
        help="Whitespace handling: 'normalized' (clean) or 'strict' (preserve). Default: normalized.",
    )

    parser.add_argument(
        "--preprocess-html",
        action="store_true",
        help="Clean messy HTML (removes navigation, ads, forms, etc).",
    )

    parser.add_argument(
        "--preprocessing-preset",
        default="standard",
        choices=("minimal", "standard", "aggressive"),
        help="Cleaning level: 'minimal', 'standard', or 'aggressive'. Default: standard.",
    )

    parser.add_argument(
        "--no-remove-forms",
        dest="remove_forms",
        action="store_false",
        help="Keep form elements when preprocessing (normally removed).",
    )

    parser.add_argument(
        "--no-remove-navigation",
        dest="remove_navigation",
        action="store_false",
        help="Keep navigation elements when preprocessing (normally removed).",
    )

    parser.add_argument(
        "--source-encoding",
        type=str,
        default=None,
        help="Encoding for reading input files and decoding bytes (e.g. 'utf-8', 'latin-1'). Default: utf-8.",
    )

    args = parser.parse_args(argv)

    base_args = {
        "autolinks": args.autolinks,
        "br_in_tables": args.br_in_tables,
        "bullets": args.bullets,
        "code_language": args.code_language,
        "convert": args.convert,
        "convert_as_inline": args.convert_as_inline,
        "default_title": args.default_title,
        "source_encoding": args.source_encoding,
        "escape_asterisks": args.escape_asterisks,
        "escape_misc": args.escape_misc,
        "escape_underscores": args.escape_underscores,
        "extract_metadata": args.extract_metadata,
        "heading_style": args.heading_style,
        "highlight_style": args.highlight_style,
        "keep_inline_images_in": args.keep_inline_images_in,
        "list_indent_type": args.list_indent_type,
        "list_indent_width": args.list_indent_width,
        "newline_style": args.newline_style,
        "preprocess_html": args.preprocess_html,
        "preprocessing_preset": args.preprocessing_preset,
        "remove_forms": args.remove_forms,
        "remove_navigation": args.remove_navigation,
        "strip": args.strip,
        "strip_newlines": args.strip_newlines,
        "strong_em_symbol": args.strong_em_symbol,
        "sub_symbol": args.sub_symbol,
        "sup_symbol": args.sup_symbol,
        "whitespace_mode": args.whitespace_mode,
        "wrap": args.wrap,
        "wrap_width": args.wrap_width,
    }

    if args.parser:
        base_args["parser"] = args.parser

    if args.stream_processing:
        base_args["stream_processing"] = True
        base_args["chunk_size"] = args.chunk_size

        if args.show_progress:

            def progress_callback(processed: int, total: int) -> None:
                if total > 0:  # pragma: no cover
                    percent = (processed / total) * 100

                    sys.stderr.write(f"\rProgress: {percent:.1f}% ({processed}/{total} bytes)")
                    sys.stderr.flush()

            base_args["progress_callback"] = progress_callback

    if args.html == "-":
        html_content = sys.stdin.buffer.read()
    else:
        try:
            file_path = Path(args.html)
            if args.source_encoding:
                with file_path.open(encoding=args.source_encoding, errors="replace") as f:
                    html_content = f.read()
            else:
                with file_path.open("rb") as f:
                    html_content = f.read()
        except (OSError, LookupError) as e:
            if isinstance(e, LookupError):
                raise InvalidEncodingError(args.source_encoding) from e
            raise

    return convert_to_markdown(html_content, **base_args)
