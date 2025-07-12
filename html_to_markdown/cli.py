import sys
from argparse import ArgumentParser, FileType

from html_to_markdown.constants import (
    ASTERISK,
    ATX,
    ATX_CLOSED,
    BACKSLASH,
    DOUBLE_EQUAL,
    SPACES,
    UNDERLINED,
    UNDERSCORE,
)
from html_to_markdown.processing import convert_to_markdown


def main(argv: list[str]) -> str:
    """Command-line entry point."""
    parser = ArgumentParser(
        prog="html_to_markdown",
        description="Converts HTML to Markdown.",
    )

    parser.add_argument(
        "html",
        nargs="?",
        type=FileType("r"),
        default=sys.stdin,
        help="The HTML file to convert. Defaults to STDIN if not provided.",
    )

    parser.add_argument(
        "-s",
        "--strip",
        nargs="*",
        help="A list of tags to strip from the conversion. Incompatible with the --convert option.",
    )

    parser.add_argument(
        "-c",
        "--convert",
        nargs="*",
        help="A list of HTML tags to explicitly convert. Incompatible with the --strip option.",
    )

    parser.add_argument(
        "-a",
        "--autolinks",
        action="store_true",
        help="Automatically convert anchor links where the content matches the href.",
    )

    parser.add_argument(
        "--default-title",
        action="store_true",
        help="Set the link title to its href when no title is provided.",
    )

    parser.add_argument(
        "--heading-style",
        default=UNDERLINED,
        choices=(ATX, ATX_CLOSED, UNDERLINED),
        help="Defines the heading conversion style: 'atx', 'atx_closed', or 'underlined'. Defaults to 'underlined'.",
    )

    parser.add_argument(
        "-b",
        "--bullets",
        default="*+-",
        help="A string of bullet styles to use for list items. The style alternates based on nesting level. Defaults to '*+-'.",
    )

    parser.add_argument(
        "--strong-em-symbol",
        default=ASTERISK,
        choices=(ASTERISK, UNDERSCORE),
        help="Choose between '*' or '_' for strong and emphasized text. Defaults to '*'.",
    )

    parser.add_argument(
        "--sub-symbol",
        default="",
        help="Define the characters used to surround <sub> text. Defaults to empty.",
    )

    parser.add_argument(
        "--sup-symbol",
        default="",
        help="Define the characters used to surround <sup> text. Defaults to empty.",
    )

    parser.add_argument(
        "--newline-style",
        default=SPACES,
        choices=(SPACES, BACKSLASH),
        help="Specify the <br> conversion style: two spaces (default) or a backslash at the end of the line.",
    )

    parser.add_argument(
        "--code-language",
        default="",
        help="Specify the default language for code blocks inside <pre> tags. Defaults to empty.",
    )

    parser.add_argument(
        "--no-escape-asterisks",
        dest="escape_asterisks",
        action="store_false",
        help="Disable escaping of '*' characters in text to '\\*'.",
    )

    parser.add_argument(
        "--no-escape-underscores",
        dest="escape_underscores",
        action="store_false",
        help="Disable escaping of '_' characters in text to '\\_'.",
    )

    parser.add_argument(
        "--no-escape-misc",
        dest="escape_misc",
        action="store_false",
        help="Disable escaping of miscellaneous characters to prevent conflicts in Markdown.",
    )

    parser.add_argument(
        "-i",
        "--keep-inline-images-in",
        nargs="*",
        help="Specify parent tags where inline images should be preserved as images, rather than converted to alt-text. Defaults to None.",
    )

    parser.add_argument(
        "-w",
        "--wrap",
        action="store_true",
        help="Enable word wrapping for paragraphs at --wrap-width characters.",
    )

    parser.add_argument(
        "--wrap-width",
        type=int,
        default=80,
        help="The number of characters at which text paragraphs should wrap. Defaults to 80.",
    )

    parser.add_argument(
        "--strip-newlines",
        action="store_true",
        help="Remove newlines from HTML input before processing. This helps flatten janky output from HTML with unnecessary line breaks.",
    )

    parser.add_argument(
        "--convert-as-inline",
        action="store_true",
        help="Treat the content as inline elements (no block elements like paragraphs).",
    )

    parser.add_argument(
        "--no-extract-metadata",
        dest="extract_metadata",
        action="store_false",
        help="Disable extraction of document metadata (title, meta tags) as a comment header.",
    )

    parser.add_argument(
        "--highlight-style",
        default=DOUBLE_EQUAL,
        choices=("double-equal", "html", "bold"),
        help="Style to use for highlighted text (mark elements). Defaults to 'double-equal'.",
    )

    parser.add_argument(
        "--stream-processing",
        action="store_true",
        help="Use streaming processing for large documents to reduce memory usage.",
    )

    parser.add_argument(
        "--chunk-size",
        type=int,
        default=1024,
        help="Size of chunks when using streaming processing. Defaults to 1024 characters.",
    )

    parser.add_argument(
        "--show-progress",
        action="store_true",
        help="Show progress information when processing large documents.",
    )

    args = parser.parse_args(argv)

    base_args = {
        "strip": args.strip,
        "convert": args.convert,
        "autolinks": args.autolinks,
        "default_title": args.default_title,
        "heading_style": args.heading_style,
        "bullets": args.bullets,
        "strong_em_symbol": args.strong_em_symbol,
        "sub_symbol": args.sub_symbol,
        "sup_symbol": args.sup_symbol,
        "newline_style": args.newline_style,
        "code_language": args.code_language,
        "escape_asterisks": args.escape_asterisks,
        "escape_underscores": args.escape_underscores,
        "escape_misc": args.escape_misc,
        "keep_inline_images_in": args.keep_inline_images_in,
        "wrap": args.wrap,
        "wrap_width": args.wrap_width,
        "strip_newlines": args.strip_newlines,
        "convert_as_inline": args.convert_as_inline,
        "extract_metadata": args.extract_metadata,
        "highlight_style": args.highlight_style,
    }

    if args.stream_processing:
        base_args["stream_processing"] = True
        base_args["chunk_size"] = args.chunk_size

        if args.show_progress:

            def progress_callback(processed: int, total: int) -> None:
                if total > 0:
                    percent = (processed / total) * 100

                    sys.stderr.write(f"\rProgress: {percent:.1f}% ({processed}/{total} bytes)")
                    sys.stderr.flush()

            base_args["progress_callback"] = progress_callback

    return convert_to_markdown(args.html.read(), **base_args)
