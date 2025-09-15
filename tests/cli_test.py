"""Comprehensive tests for CLI functionality.

Covers argument parsing, option handling, file operations, error handling,
and integration with actual conversions.
"""

from __future__ import annotations

import os
import subprocess
import sys
from io import StringIO
from typing import TYPE_CHECKING
from unittest.mock import Mock, mock_open, patch

import pytest

from html_to_markdown.cli import main
from html_to_markdown.exceptions import InvalidEncodingError

if TYPE_CHECKING:
    from collections.abc import Generator
    from pathlib import Path

DEFAULT_CLI_ARGS = {
    "autolinks": False,
    "br_in_tables": False,
    "bullets": "*+-",
    "code_language": "",
    "convert": None,
    "convert_as_inline": False,
    "default_title": False,
    "escape_asterisks": True,
    "escape_misc": True,
    "escape_underscores": True,
    "extract_metadata": True,
    "heading_style": "underlined",
    "highlight_style": "double-equal",
    "keep_inline_images_in": None,
    "list_indent_type": "spaces",
    "list_indent_width": 4,
    "newline_style": "spaces",
    "preprocess_html": False,
    "preprocessing_preset": "standard",
    "remove_forms": True,
    "remove_navigation": True,
    "strip": None,
    "strip_newlines": False,
    "strong_em_symbol": "*",
    "sub_symbol": "",
    "sup_symbol": "",
    "whitespace_mode": "normalized",
    "wrap": False,
    "wrap_width": 80,
}


def run_cli_command(args: list[str], input_text: str | None = None, timeout: int = 60) -> tuple[str, str, int]:
    cli_command = [sys.executable, "-m", "html_to_markdown", *args]

    # Set up environment with proper UTF-8 encoding on Windows
    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8:replace"
    if os.name == "nt":  # Windows
        env["PYTHONUTF8"] = "1"

    process = subprocess.Popen(
        cli_command,
        stdin=subprocess.PIPE if input_text else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=False,
        env=env,
    )

    try:
        stdin_bytes = input_text.encode("utf-8") if input_text is not None else None
        stdout_b, stderr_b = process.communicate(input=stdin_bytes, timeout=timeout)
        # Decode with replacement to avoid platform threading decode errors
        stdout = (stdout_b or b"").decode("utf-8", "replace")
        stderr = (stderr_b or b"").decode("utf-8", "replace")
        return stdout, stderr, process.returncode
    except subprocess.TimeoutExpired:
        process.kill()
        raise


def run_cli(args: list[str], input_html: str) -> str:
    # Set up environment with proper UTF-8 encoding on Windows
    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8:replace"
    if os.name == "nt":  # Windows
        env["PYTHONUTF8"] = "1"

    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", *args],
        check=False,
        input=input_html.encode("utf-8"),
        capture_output=True,
        text=False,
        env=env,
    )
    return (result.stdout or b"").decode("utf-8", "replace")


@pytest.fixture
def mock_convert_to_markdown() -> Generator[Mock, None, None]:
    with patch("html_to_markdown.cli.convert_to_markdown") as mock:
        mock.return_value = "Mocked Markdown Output"
        yield mock


@pytest.fixture
def mock_stdin() -> Generator[None, None, None]:
    with patch("sys.stdin", new=StringIO("<html><body><p>Test from stdin</p></body></html>")):
        yield


@pytest.fixture
def sample_html_file(tmp_path: Path) -> Path:
    file_path = tmp_path / "test.html"
    content = """
    <html>
        <body>
            <h1>Sample Document</h1>
            <p>This is a <b>test</b> paragraph with some <i>formatted</i> text.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
            <pre><code>print("Hello World")</code></pre>
        </body>
    </html>
    """
    file_path.write_text(content)
    return file_path


@pytest.fixture
def complex_html_file(tmp_path: Path) -> Path:
    file_path = tmp_path / "complex.html"
    content = """
    <html>
        <body>
            <h1>Complex Document</h1>
            <table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
            </table>
            <blockquote>
                <p>Nested <em>formatting</em> with <code>inline code</code></p>
            </blockquote>
            <pre><code class="language-python">
def hello():
    print("Hello World")
            </code></pre>
            <p>Link: <a href="http://example.com" title="http://example.com">Example</a></p>
            <img src="image.jpg" alt="Test Image">
        </body>
    </html>
    """
    file_path.write_text(content)
    return file_path


def test_file_input_mocked(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test</h1></body></html>"
    with patch("builtins.open", mock_open(read_data=test_html)):
        result = main(["input.html"])

    assert result == "Mocked Markdown Output"
    mock_convert_to_markdown.assert_called_once_with(test_html, **DEFAULT_CLI_ARGS)


def test_stdin_input_mocked(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    result = main([])

    assert result == "Mocked Markdown Output"
    mock_convert_to_markdown.assert_called_once_with(
        "<html><body><p>Test from stdin</p></body></html>", **DEFAULT_CLI_ARGS
    )


def test_strip_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strip", "div", "span"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strip"] == ["div", "span"]


def test_convert_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--convert", "p", "h1"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["convert"] == ["p", "h1"]


def test_autolinks_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--autolinks"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["autolinks"] is True


def test_default_title_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--default-title"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["default_title"] is True


def test_heading_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--heading-style", "atx"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["heading_style"] == "atx"


def test_bullets_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--bullets", "+-*"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["bullets"] == "+-*"


def test_strong_em_symbol_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strong-em-symbol", "_"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strong_em_symbol"] == "_"


def test_sub_and_sup_symbol_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--sub-symbol", "~", "--sup-symbol", "^"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["sub_symbol"] == "~"
    assert mock_convert_to_markdown.call_args[1]["sup_symbol"] == "^"


def test_newline_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--newline-style", "backslash"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["newline_style"] == "backslash"


def test_code_language_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--code-language", "python"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["code_language"] == "python"


def test_no_escape_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-escape-asterisks", "--no-escape-underscores"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["escape_asterisks"] is False
    assert mock_convert_to_markdown.call_args[1]["escape_underscores"] is False


def test_keep_inline_images_in_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--keep-inline-images-in", "p", "div"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["keep_inline_images_in"] == ["p", "div"]


def test_wrap_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--wrap", "--wrap-width", "100"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["wrap"] is True
    assert mock_convert_to_markdown.call_args[1]["wrap_width"] == 100


def test_strip_newlines_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--strip-newlines"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["strip_newlines"] is True


def test_br_in_tables_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--br-in-tables"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["br_in_tables"] is True


def test_stream_processing_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--stream-processing", "--chunk-size", "2048"])
    mock_convert_to_markdown.assert_called_once()
    args = mock_convert_to_markdown.call_args[1]
    assert args["stream_processing"] is True
    assert args["chunk_size"] == 2048


def test_stream_processing_with_progress(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
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


def test_convert_as_inline_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--convert-as-inline"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["convert_as_inline"] is True


def test_no_extract_metadata_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--no-extract-metadata"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["extract_metadata"] is False


def test_highlight_style_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--highlight-style", "html"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["highlight_style"] == "html"


def test_parser_option(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--parser", "lxml"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["parser"] == "lxml"


def test_list_indent_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--list-indent-type", "tabs", "--list-indent-width", "2"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["list_indent_type"] == "tabs"
    assert mock_convert_to_markdown.call_args[1]["list_indent_width"] == 2


def test_whitespace_mode_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--whitespace-mode", "strict"])
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1]["whitespace_mode"] == "strict"


def test_preprocessing_options(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
    main(["--preprocess-html", "--preprocessing-preset", "aggressive", "--no-remove-forms", "--no-remove-navigation"])
    mock_convert_to_markdown.assert_called_once()
    args = mock_convert_to_markdown.call_args[1]
    assert args["preprocess_html"] is True
    assert args["preprocessing_preset"] == "aggressive"
    assert args["remove_forms"] is False
    assert args["remove_navigation"] is False


def test_all_options_combined(mock_convert_to_markdown: Mock, mock_stdin: Mock) -> None:
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


def test_basic_file_conversion(sample_html_file: Path) -> None:
    stdout, stderr, returncode = run_cli_command([str(sample_html_file)])

    assert returncode == 0
    assert stderr == ""
    assert "Sample Document" in stdout
    assert "**test**" in stdout
    assert "*formatted*" in stdout
    assert "* Item 1" in stdout
    assert "```\nprint" in stdout


def test_complex_file_conversion(complex_html_file: Path) -> None:
    stdout, stderr, returncode = run_cli_command([str(complex_html_file)])

    assert returncode == 0
    assert stderr == ""
    assert "Complex Document" in stdout
    assert "| Header 1 | Header 2 |" in stdout
    assert "> Nested" in stdout
    assert "`inline code`" in stdout
    assert '[Example](http://example.com "http://example.com")' in stdout
    assert "![Test Image](image.jpg)" in stdout


def test_stdin_input_integration() -> None:
    input_html = "<h1>Test</h1><p>Content</p>"
    stdout, stderr, returncode = run_cli_command([], input_text=input_html)

    assert returncode == 0
    assert stderr == ""
    assert "Test" in stdout
    assert "Content" in stdout


def test_heading_styles_integration(sample_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(sample_html_file), "--heading-style", "atx"])
    assert "# Sample Document" in stdout

    stdout, _, _ = run_cli_command([str(sample_html_file), "--heading-style", "atx_closed"])
    assert "# Sample Document #" in stdout


def test_formatting_options_integration(sample_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(sample_html_file), "--strong-em-symbol", "_", "--wrap", "--wrap-width", "40"])

    assert "__test__" in stdout
    assert all(len(line) <= 40 for line in stdout.split("\n"))


def test_code_block_options_integration(complex_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(complex_html_file), "--code-language", "python"])
    assert "```python" in stdout


def test_special_characters_integration() -> None:
    input_html = "<p>Text with * and _ and ** symbols</p>"

    stdout, _, _ = run_cli_command([], input_text=input_html)
    assert "\\*" in stdout
    assert "\\_" in stdout

    stdout, _, _ = run_cli_command(["--no-escape-asterisks", "--no-escape-underscores"], input_text=input_html)
    assert "\\*" not in stdout
    assert "\\_" not in stdout


def test_unicode_handling() -> None:
    input_html = "<p>Unicode: 你好 • é è à ñ</p>"
    stdout, _stderr, returncode = run_cli_command([], input_text=input_html)

    assert returncode == 0
    assert "你好" in stdout
    assert "é è à ñ" in stdout


def test_large_file_handling(tmp_path: Path) -> None:
    large_file = tmp_path / "large.html"

    with large_file.open("w") as f:
        f.write("<p>")
        for i in range(50000):
            f.write(f"Line {i} with some <b>bold</b> text.\n")
        f.write("</p>")

    stdout, stderr, returncode = run_cli_command([str(large_file)], timeout=30)

    assert returncode == 0
    assert stderr == ""
    assert "Line 0" in stdout
    assert "Line 49999" in stdout


def test_multiple_files(sample_html_file: Path, complex_html_file: Path, tmp_path: Path) -> None:
    for file in [sample_html_file, complex_html_file]:
        stdout, stderr, returncode = run_cli_command([str(file)])
        assert returncode == 0
        assert stderr == ""

        output_file = tmp_path / f"{file.stem}.md"
        output_file.write_text(stdout)

        assert output_file.exists()
        assert output_file.stat().st_size > 0


def test_pipe_chain() -> None:
    html_input = "<h1>Test</h1>"
    process = subprocess.Popen(
        [sys.executable, "-m", "html_to_markdown"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    output, _ = process.communicate(input=html_input)
    assert "Test" in output


def test_error_handling() -> None:
    _stdout, stderr, returncode = run_cli_command(["nonexistent.html"])
    assert returncode != 0
    assert "No such file" in stderr

    _stdout, stderr, returncode = run_cli_command(["--invalid-option"])
    assert returncode != 0
    assert "unrecognized arguments" in stderr

    _stdout, stderr, returncode = run_cli_command(["--strip", "p", "--convert", "p"], input_text="<p>Test</p>")
    assert returncode != 0
    assert "Only one of 'strip' and 'convert' can be specified" in stderr

    _stdout, stderr, returncode = run_cli_command(["--strip", "p"], input_text="")
    assert returncode != 0
    assert "The input HTML is empty" in stderr


def test_discord_list_indentation() -> None:
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li><li>Item 2</li></ul>"
    output = run_cli(["--list-indent-width", "2", "--no-extract-metadata"], html)
    assert "* Item 1\n\n  + Nested\n* Item 2" in output


def test_tab_list_indentation() -> None:
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>"
    output = run_cli(["--list-indent-type", "tabs", "--no-extract-metadata"], html)
    assert "* Item 1\n\n\t+ Nested" in output


def test_whitespace_mode_strict() -> None:
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "strict", "--no-extract-metadata"], html)
    assert "Multiple spaces here" in output


def test_whitespace_mode_normalized() -> None:
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "Multiple spaces here" in output


def test_parser_selection() -> None:
    html = "<div>Test</div>"
    output = run_cli(["--parser", "html.parser", "--no-extract-metadata"], html)
    assert "Test" in output


def test_html_preprocessing() -> None:
    html = """
    <html>
    <body>
        <nav>Navigation menu</nav>
        <main>Main content</main>
        <form>Form content</form>
    </body>
    </html>
    """
    output = run_cli(["--preprocess-html", "--no-extract-metadata"], html)
    assert "Navigation menu" not in output
    assert "Main content" in output
    assert "Form content" not in output


def test_preprocess_keep_forms() -> None:
    html = """
    <html>
    <body>
        <form>Form content</form>
        <main>Main content</main>
    </body>
    </html>
    """
    output = run_cli(["--preprocess-html", "--no-remove-forms", "--no-extract-metadata"], html)
    assert "Form content" in output
    assert "Main content" in output


def test_preprocess_keep_navigation() -> None:
    html = """
    <html>
    <body>
        <nav>Navigation menu</nav>
        <main>Main content</main>
    </body>
    </html>
    """
    output = run_cli(["--preprocess-html", "--no-remove-navigation", "--no-extract-metadata"], html)
    assert "Navigation menu" in output
    assert "Main content" in output


def test_preprocessing_preset_minimal() -> None:
    html = """
    <html>
    <body>
        <script>alert('test');</script>
        <style>body { color: red; }</style>
        <main>Main content</main>
    </body>
    </html>
    """
    output = run_cli(["--preprocess-html", "--preprocessing-preset", "minimal", "--no-extract-metadata"], html)
    assert "alert" not in output
    assert "color: red" not in output
    assert "Main content" in output


def test_preprocessing_preset_aggressive() -> None:
    html = """
    <html>
    <body>
        <aside>Sidebar</aside>
        <footer>Footer</footer>
        <main>Main content</main>
    </body>
    </html>
    """
    output = run_cli(["--preprocess-html", "--preprocessing-preset", "aggressive", "--no-extract-metadata"], html)
    assert "Main content" in output


def test_combined_list_and_whitespace() -> None:
    html = "<ul><li>Item  with   spaces<ul><li>Nested  item</li></ul></li></ul>"
    output = run_cli(["--list-indent-width", "2", "--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "* Item with spaces\n\n  + Nested item" in output


def test_all_new_options_combined() -> None:
    html = """
    <html>
    <body>
        <nav>Navigation</nav>
        <ul>
            <li>Item 1
                <ul><li>Nested</li></ul>
            </li>
        </ul>
    </body>
    </html>
    """
    output = run_cli(
        [
            "--list-indent-width",
            "3",
            "--list-indent-type",
            "spaces",
            "--whitespace-mode",
            "normalized",
            "--preprocess-html",
            "--preprocessing-preset",
            "standard",
            "--no-extract-metadata",
        ],
        html,
    )
    assert "Navigation" not in output
    assert "* Item 1\n\n   + Nested" in output


def test_help_includes_new_options() -> None:
    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    help_text = result.stdout

    assert "--parser" in help_text
    assert "--list-indent-type" in help_text
    assert "--list-indent-width" in help_text
    assert "--whitespace-mode" in help_text
    assert "--preprocess-html" in help_text
    assert "--preprocessing-preset" in help_text
    assert "--no-remove-forms" in help_text
    assert "--no-remove-navigation" in help_text

    assert "Discord" in help_text or "2" in help_text


@pytest.mark.parametrize("newline_style", ["spaces", "backslash"])
def test_newline_styles(newline_style: str) -> None:
    input_html = "<p>Line 1<br>Line 2</p>"
    stdout, _, _ = run_cli_command(["--newline-style", newline_style], input_text=input_html)

    expected_break = "\\\n" if newline_style == "backslash" else "  \n"
    assert expected_break in stdout


@pytest.mark.parametrize(
    "option,expected_param,expected_value",
    [
        (["--autolinks"], "autolinks", True),
        (["--default-title"], "default_title", True),
        (["--heading-style", "atx"], "heading_style", "atx"),
        (["--bullets", "+-*"], "bullets", "+-*"),
        (["--strong-em-symbol", "_"], "strong_em_symbol", "_"),
        (["--newline-style", "backslash"], "newline_style", "backslash"),
        (["--code-language", "python"], "code_language", "python"),
        (["--no-escape-asterisks"], "escape_asterisks", False),
        (["--strip-newlines"], "strip_newlines", True),
        (["--convert-as-inline"], "convert_as_inline", True),
        (["--no-extract-metadata"], "extract_metadata", False),
        (["--parser", "lxml"], "parser", "lxml"),
        (["--list-indent-type", "tabs"], "list_indent_type", "tabs"),
        (["--list-indent-width", "2"], "list_indent_width", 2),
        (["--whitespace-mode", "strict"], "whitespace_mode", "strict"),
        (["--preprocess-html"], "preprocess_html", True),
        (["--preprocessing-preset", "aggressive"], "preprocessing_preset", "aggressive"),
        (["--no-remove-forms"], "remove_forms", False),
        (["--no-remove-navigation"], "remove_navigation", False),
    ],
)
def test_individual_cli_options(
    option: list[str],
    expected_param: str,
    expected_value: str | bool | int,
    mock_convert_to_markdown: Mock,
    mock_stdin: Mock,
) -> None:
    main(option)
    mock_convert_to_markdown.assert_called_once()
    assert mock_convert_to_markdown.call_args[1][expected_param] == expected_value


def test_main_with_source_encoding_option(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test ñ</h1></body></html>"
    mock_file = mock_open(read_data=test_html)

    with patch("builtins.open", mock_file), patch("pathlib.Path.open") as mock_path_open:
        mock_path_open.return_value.__enter__ = lambda self: mock_file.return_value
        mock_path_open.return_value.__exit__ = lambda self, *args: None
        mock_path_open.return_value.read.return_value = test_html

        result = main(["input.html", "--source-encoding", "utf-8"])

    assert result == "Mocked Markdown Output"
    mock_path_open.assert_called_once_with(encoding="utf-8")
    mock_convert_to_markdown.assert_called_once_with(test_html, **DEFAULT_CLI_ARGS)


def test_main_with_invalid_source_encoding_raises_error(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test</h1></body></html>"
    mock_file = mock_open(read_data=test_html)

    with patch("builtins.open", mock_file), patch("pathlib.Path.open") as mock_path_open:
        mock_file_obj = Mock()
        mock_file_obj.read.side_effect = LookupError("unknown encoding: invalid-encoding")

        mock_path_open.return_value.__enter__.return_value = mock_file_obj
        mock_path_open.return_value.__exit__.return_value = None

        with pytest.raises(InvalidEncodingError) as exc_info:
            main(["input.html", "--source-encoding", "invalid-encoding"])

        assert str(exc_info.value) == "The specified encoding (invalid-encoding) is not valid."
        mock_path_open.assert_called_once_with(encoding="invalid-encoding")


def test_main_with_source_encoding_ignored_for_stdin(mock_convert_to_markdown: Mock) -> None:
    mock_stdin_io = StringIO("<html><body><p>Test from stdin</p></body></html>")
    mock_stdin_io.name = "<stdin>"

    with patch("sys.stdin", new=mock_stdin_io):
        result = main(["--source-encoding", "utf-8"])

    assert result == "Mocked Markdown Output"

    mock_convert_to_markdown.assert_called_once_with(
        "<html><body><p>Test from stdin</p></body></html>", **DEFAULT_CLI_ARGS
    )


def test_main_with_source_encoding_default_none(mock_convert_to_markdown: Mock) -> None:
    test_html = "<html><body><h1>Test default encoding</h1></body></html>"

    with patch("builtins.open", mock_open(read_data=test_html)):
        result = main(["input.html"])

    assert result == "Mocked Markdown Output"
    mock_convert_to_markdown.assert_called_once_with(test_html, **DEFAULT_CLI_ARGS)
