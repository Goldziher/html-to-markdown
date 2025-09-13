from collections.abc import Generator
from io import StringIO
from unittest.mock import Mock, mock_open, patch

import pytest

from html_to_markdown.cli import main
from html_to_markdown.exceptions import InvalidEncodingSpecifiedError


@pytest.fixture
def mock_convert_to_markdown() -> Generator[Mock, None, None]:
    with patch("html_to_markdown.cli.convert_to_markdown") as mock:
        mock.return_value = "Mocked Markdown Output"
        yield mock


@pytest.fixture
def mock_stdin() -> Generator[None, None, None]:
    with patch("sys.stdin", new=StringIO("<html><body><p>Test from stdin</p></body></html>")):
        yield


def test_main_with_file_input(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test</h1></body></html>"
    with patch("builtins.open", mock_open(read_data=test_html)):
        result = main(["input.html"])

    assert result == "Mocked Markdown Output"
    mock_convert_to_markdown.assert_called_once_with(
        test_html,
        autolinks=False,
        bullets="*+-",
        code_language="",
        convert=None,
        convert_as_inline=False,
        default_title=False,
        escape_asterisks=True,
        escape_misc=True,
        escape_underscores=True,
        extract_metadata=True,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
        newline_style="spaces",
        preprocess_html=False,
        preprocessing_preset="standard",
        remove_forms=True,
        remove_navigation=True,
        strip=None,
        strip_newlines=False,
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        whitespace_mode="normalized",
        wrap=False,
        wrap_width=80,
    )


def test_main_with_stdin_input(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    result = main([])

    assert result == "Mocked Markdown Output"
    mock_convert_to_markdown.assert_called_once_with(
        "<html><body><p>Test from stdin</p></body></html>",
        autolinks=False,
        bullets="*+-",
        code_language="",
        convert=None,
        convert_as_inline=False,
        default_title=False,
        escape_asterisks=True,
        escape_misc=True,
        escape_underscores=True,
        extract_metadata=True,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
        newline_style="spaces",
        preprocess_html=False,
        preprocessing_preset="standard",
        remove_forms=True,
        remove_navigation=True,
        strip=None,
        strip_newlines=False,
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        whitespace_mode="normalized",
        wrap=False,
        wrap_width=80,
    )


def test_main_with_strip_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strip", "div", "span"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strip"] == ["div", "span"]


def test_main_with_convert_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--convert", "p", "h1"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["convert"] == ["p", "h1"]


def test_main_with_autolinks_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--autolinks"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["autolinks"] is True


def test_main_with_default_title_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--default-title"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["default_title"] is True


def test_main_with_heading_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--heading-style", "atx"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["heading_style"] == "atx"


def test_main_with_bullets_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--bullets", "+-*"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["bullets"] == "+-*"


def test_main_with_strong_em_symbol_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strong-em-symbol", "_"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strong_em_symbol"] == "_"


def test_main_with_sub_and_sup_symbol_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--sub-symbol", "~", "--sup-symbol", "^"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["sub_symbol"] == "~"
    assert mock_convert_to_markdown.call_args[1]["sup_symbol"] == "^"


def test_main_with_newline_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--newline-style", "backslash"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["newline_style"] == "backslash"


def test_main_with_code_language_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--code-language", "python"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["code_language"] == "python"


def test_main_with_no_escape_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-escape-asterisks", "--no-escape-underscores"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["escape_asterisks"] is False
    assert mock_convert_to_markdown.call_args[1]["escape_underscores"] is False


def test_main_with_keep_inline_images_in_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--keep-inline-images-in", "p", "div"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["keep_inline_images_in"] == ["p", "div"]


def test_main_with_wrap_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--wrap", "--wrap-width", "100"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["wrap"] is True
    assert mock_convert_to_markdown.call_args[1]["wrap_width"] == 100


def test_main_with_strip_newlines_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strip-newlines"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strip_newlines"] is True


def test_main_with_stream_processing_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--stream-processing", "--chunk-size", "2048"])
    mock_convert_to_markdown.assert_called_once()
    args = mock_convert_to_markdown.call_args[1]
    assert args["stream_processing"] is True
    assert args["chunk_size"] == 2048


def test_main_with_stream_processing_and_progress(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    with patch("sys.stderr") as mock_stderr:
        main(["--stream-processing", "--show-progress"])
        mock_convert_to_markdown.assert_called_once()
        args = mock_convert_to_markdown.call_args[1]
        assert args["stream_processing"] is True
        assert "progress_callback" in args

        callback = args["progress_callback"]
        callback(50, 100)
        mock_stderr.write.assert_called_with("\rProgress: 50.0% (50/100 bytes)")
        mock_stderr.flush.assert_called_once()


def test_main_with_convert_as_inline_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--convert-as-inline"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["convert_as_inline"] is True


def test_main_with_no_extract_metadata_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-extract-metadata"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["extract_metadata"] is False


def test_main_with_highlight_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--highlight-style", "html"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["highlight_style"] == "html"


def test_main_with_parser_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--parser", "lxml"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["parser"] == "lxml"


def test_main_with_list_indent_type_spaces(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--list-indent-type", "spaces"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["list_indent_type"] == "spaces"


def test_main_with_list_indent_type_tabs(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--list-indent-type", "tabs"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["list_indent_type"] == "tabs"


def test_main_with_list_indent_width_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--list-indent-width", "2"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["list_indent_width"] == 2


def test_main_with_list_indent_discord_compatible(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--list-indent-width", "2", "--list-indent-type", "spaces"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["list_indent_width"] == 2
    assert mock_convert_to_markdown.call_args[1]["list_indent_type"] == "spaces"


def test_main_with_whitespace_mode_normalized(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--whitespace-mode", "normalized"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["whitespace_mode"] == "normalized"


def test_main_with_whitespace_mode_strict(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--whitespace-mode", "strict"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["whitespace_mode"] == "strict"


def test_main_with_preprocess_html_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--preprocess-html"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["preprocess_html"] is True


def test_main_with_preprocessing_preset_minimal(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--preprocessing-preset", "minimal"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["preprocessing_preset"] == "minimal"


def test_main_with_preprocessing_preset_aggressive(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--preprocessing-preset", "aggressive"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["preprocessing_preset"] == "aggressive"


def test_main_with_no_remove_forms_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-remove-forms"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["remove_forms"] is False


def test_main_with_no_remove_navigation_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-remove-navigation"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["remove_navigation"] is False


def test_main_with_preprocessing_combined_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--preprocess-html", "--preprocessing-preset", "aggressive", "--no-remove-forms", "--no-remove-navigation"])
    mock_convert_to_markdown.assert_called_once()
    args = mock_convert_to_markdown.call_args[1]
    assert args["preprocess_html"] is True
    assert args["preprocessing_preset"] == "aggressive"
    assert args["remove_forms"] is False
    assert args["remove_navigation"] is False


def test_main_with_all_new_options_combined(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(
        [
            "--parser",
            "lxml",
            "--list-indent-type",
            "tabs",
            "--list-indent-width",
            "3",
            "--whitespace-mode",
            "strict",
            "--preprocess-html",
            "--preprocessing-preset",
            "minimal",
            "--no-remove-forms",
            "--no-remove-navigation",
        ]
    )
    mock_convert_to_markdown.assert_called_once()
    args = mock_convert_to_markdown.call_args[1]
    assert args["parser"] == "lxml"
    assert args["list_indent_type"] == "tabs"
    assert args["list_indent_width"] == 3
    assert args["whitespace_mode"] == "strict"
    assert args["preprocess_html"] is True
    assert args["preprocessing_preset"] == "minimal"
    assert args["remove_forms"] is False
    assert args["remove_navigation"] is False


def test_main_with_source_encoding_option(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test ñ</h1></body></html>"
    mock_file = mock_open(read_data=test_html)

    # Mock the Path.open context manager to return the file content with specific encoding
    with patch("builtins.open", mock_file), patch("pathlib.Path.open") as mock_path_open:
        mock_path_open.return_value.__enter__ = lambda self: mock_file.return_value
        mock_path_open.return_value.__exit__ = lambda self, *args: None
        mock_path_open.return_value.read.return_value = test_html

        result = main(["input.html", "--source_encoding", "utf-8"])

    assert result == "Mocked Markdown Output"
    mock_path_open.assert_called_once_with(encoding="utf-8")
    mock_convert_to_markdown.assert_called_once_with(
        test_html,
        autolinks=False,
        bullets="*+-",
        code_language="",
        convert=None,
        convert_as_inline=False,
        default_title=False,
        escape_asterisks=True,
        escape_misc=True,
        escape_underscores=True,
        extract_metadata=True,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
        newline_style="spaces",
        preprocess_html=False,
        preprocessing_preset="standard",
        remove_forms=True,
        remove_navigation=True,
        strip=None,
        strip_newlines=False,
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        whitespace_mode="normalized",
        wrap=False,
        wrap_width=80,
    )


def test_main_with_invalid_source_encoding_raises_error(mock_convert_to_markdown: Mock) -> None:
    """Test that an invalid source_encoding raises InvalidEncodingSpecifiedError."""
    test_html = "<html><body><h1>Test</h1></body></html>"
    mock_file = mock_open(read_data=test_html)

    with patch("builtins.open", mock_file), patch("pathlib.Path.open") as mock_path_open:
        # Create a mock file object that raises LookupError on read()
        mock_file_obj = Mock()
        mock_file_obj.read.side_effect = LookupError("unknown encoding: invalid-encoding")

        # Set up the context manager to return our mock file object
        mock_path_open.return_value.__enter__.return_value = mock_file_obj
        mock_path_open.return_value.__exit__.return_value = None

        with pytest.raises(InvalidEncodingSpecifiedError) as exc_info:
            main(["input.html", "--source_encoding", "invalid-encoding"])

        assert str(exc_info.value) == "The specified encoding (invalid-encoding) is not valid."
        mock_path_open.assert_called_once_with(encoding="invalid-encoding")


def test_main_with_source_encoding_ignored_for_stdin(mock_convert_to_markdown: Mock) -> None:
    """Test that source_encoding argument is ignored when input comes from stdin."""
    # Create a custom stdin mock with the name attribute
    mock_stdin_io = StringIO("<html><body><p>Test from stdin</p></body></html>")
    mock_stdin_io.name = "<stdin>"

    with patch("sys.stdin", new=mock_stdin_io):
        # When using stdin, source_encoding should be ignored
        result = main(["--source_encoding", "utf-8"])

    assert result == "Mocked Markdown Output"

    # Verify that the stdin content was used directly without encoding handling
    mock_convert_to_markdown.assert_called_once_with(
        "<html><body><p>Test from stdin</p></body></html>",
        autolinks=False,
        bullets="*+-",
        code_language="",
        convert=None,
        convert_as_inline=False,
        default_title=False,
        escape_asterisks=True,
        escape_misc=True,
        escape_underscores=True,
        extract_metadata=True,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
        newline_style="spaces",
        preprocess_html=False,
        preprocessing_preset="standard",
        remove_forms=True,
        remove_navigation=True,
        strip=None,
        strip_newlines=False,
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        whitespace_mode="normalized",
        wrap=False,
        wrap_width=80,
    )


def test_main_with_source_encoding_default_none(mock_convert_to_markdown: Mock) -> None:
    """Test that when no source_encoding is specified, default file reading is used."""
    test_html = "<html><body><h1>Test default encoding</h1></body></html>"

    with patch("builtins.open", mock_open(read_data=test_html)):
        result = main(["input.html"])

    assert result == "Mocked Markdown Output"
    # Verify that when no source_encoding is specified, the normal file reading is used
    mock_convert_to_markdown.assert_called_once_with(
        test_html,
        autolinks=False,
        bullets="*+-",
        code_language="",
        convert=None,
        convert_as_inline=False,
        default_title=False,
        escape_asterisks=True,
        escape_misc=True,
        escape_underscores=True,
        extract_metadata=True,
        heading_style="underlined",
        highlight_style="double-equal",
        keep_inline_images_in=None,
        list_indent_type="spaces",
        list_indent_width=4,
        newline_style="spaces",
        preprocess_html=False,
        preprocessing_preset="standard",
        remove_forms=True,
        remove_navigation=True,
        strip=None,
        strip_newlines=False,
        strong_em_symbol="*",
        sub_symbol="",
        sup_symbol="",
        whitespace_mode="normalized",
        wrap=False,
        wrap_width=80,
    )
