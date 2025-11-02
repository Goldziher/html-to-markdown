"""Comprehensive tests for CLI functionality.

Covers argument parsing, option handling, file operations, error handling,
and integration with actual conversions.
"""

from __future__ import annotations

import os
import subprocess
import sys
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from pathlib import Path


def run_cli_command(
    args: list[str], input_text: str | None = None, input_bytes: bytes | None = None, timeout: int = 60
) -> tuple[str, str, int]:
    cli_command = [sys.executable, "-m", "html_to_markdown", *args]

    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8:replace"
    if os.name == "nt":
        env["PYTHONUTF8"] = "1"

    process = subprocess.Popen(
        cli_command,
        stdin=subprocess.PIPE if (input_text or input_bytes) else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=False,
        env=env,
    )

    try:
        if input_bytes is not None:
            stdin_bytes = input_bytes
        elif input_text is not None:
            stdin_bytes = input_text.encode("utf-8")
        else:
            stdin_bytes = None
        stdout_b, stderr_b = process.communicate(input=stdin_bytes, timeout=timeout)
        stdout = (stdout_b or b"").decode("utf-8", "replace")
        stderr = (stderr_b or b"").decode("utf-8", "replace")
        stdout = stdout.replace("\r\n", "\n")
        stderr = stderr.replace("\r\n", "\n")
        return stdout, stderr, process.returncode
    except subprocess.TimeoutExpired:
        process.kill()
        raise


def run_cli(args: list[str], input_html: str) -> str:
    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8:replace"
    if os.name == "nt":
        env["PYTHONUTF8"] = "1"

    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", *args],
        check=False,
        input=input_html.encode("utf-8"),
        capture_output=True,
        text=False,
        env=env,
    )
    out = (result.stdout or b"").decode("utf-8", "replace")
    return out.replace("\r\n", "\n")


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


def test_basic_file_conversion(sample_html_file: Path) -> None:
    stdout, stderr, returncode = run_cli_command([str(sample_html_file)])

    assert returncode == 0
    assert stderr == ""
    assert "Sample Document" in stdout
    assert "**test**" in stdout
    assert "*formatted*" in stdout
    assert "- Item 1" in stdout
    assert '    print("Hello World")' in stdout


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

    stdout, _, _ = run_cli_command([str(sample_html_file), "--heading-style", "atx-closed"])
    assert "# Sample Document #" in stdout


def test_formatting_options_integration(sample_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(sample_html_file), "--strong-em-symbol", "_", "--wrap", "--wrap-width", "40"])

    assert "__test__" in stdout
    assert all(len(line) <= 40 for line in stdout.split("\n"))


def test_code_block_options_integration(complex_html_file: Path) -> None:
    stdout, _, _ = run_cli_command(
        [str(complex_html_file), "--code-language", "python", "--code-block-style", "backticks"]
    )
    assert "```python" in stdout


def test_special_characters_integration() -> None:
    input_html = "<p>Text with * and _ and ** symbols</p>"

    stdout, _, _ = run_cli_command([], input_text=input_html)
    assert "*" in stdout
    assert "_" in stdout

    stdout, _, _ = run_cli_command(["--escape-asterisks", "--escape-underscores"], input_text=input_html)
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

    stdout, stderr, returncode = run_cli_command([str(large_file)], timeout=120)

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
    assert "No such file" in stderr or "cannot find the file" in stderr

    _stdout, stderr, returncode = run_cli_command(["--invalid-option"])
    assert returncode != 0
    assert "unexpected argument" in stderr or "unrecognized" in stderr


def test_discord_list_indentation() -> None:
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li><li>Item 2</li></ul>"
    output = run_cli(["--list-indent-width", "2", "--no-extract-metadata"], html)
    assert "- Item 1\n  - Nested\n- Item 2" in output


def test_tab_list_indentation() -> None:
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>"
    output = run_cli(["--list-indent-type", "tabs", "--no-extract-metadata"], html)
    assert "- Item 1\n\t- Nested" in output


def test_whitespace_mode_strict() -> None:
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "strict", "--no-extract-metadata"], html)
    assert "  Multiple   spaces   here  " in output


def test_whitespace_mode_normalized() -> None:
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "Multiple spaces here" in output


def test_parser_selection() -> None:
    html = "<div>Test</div>"
    output = run_cli(["--no-extract-metadata"], html)
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
    output = run_cli(["--preprocess", "--no-extract-metadata"], html)
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
    output = run_cli(["--preprocess", "--keep-forms", "--no-extract-metadata"], html)
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
    output = run_cli(["--preprocess", "--keep-navigation", "--no-extract-metadata"], html)
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
    output = run_cli(["--preprocess", "--preset", "minimal", "--no-extract-metadata"], html)
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
    output = run_cli(["--preprocess", "--preset", "aggressive", "--no-extract-metadata"], html)
    assert "Main content" in output


def test_combined_list_and_whitespace() -> None:
    html = "<ul><li>Item  with   spaces<ul><li>Nested  item</li></ul></li></ul>"
    output = run_cli(["--list-indent-width", "2", "--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "- Item with spaces\n  - Nested item" in output


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
            "--preprocess",
            "--preset",
            "standard",
            "--no-extract-metadata",
        ],
        html,
    )
    assert "Navigation" not in output
    assert "- Item 1\n   - Nested" in output


def test_help_includes_new_options() -> None:
    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    help_text = result.stdout

    assert "--list-indent-type" in help_text
    assert "--list-indent-width" in help_text
    assert "--whitespace-mode" in help_text
    assert "--preprocess" in help_text
    assert "--preset" in help_text
    assert "--keep-forms" in help_text
    assert "--keep-navigation" in help_text

    assert "Discord" in help_text or "2" in help_text


@pytest.mark.parametrize("newline_style", ["spaces", "backslash"])
def test_newline_styles(newline_style: str) -> None:
    input_html = "<p>Line 1<br>Line 2</p>"
    stdout, _, _ = run_cli_command(["--newline-style", newline_style], input_text=input_html)

    expected_break = "\\\n" if newline_style == "backslash" else "  \n"
    assert expected_break in stdout


def test_source_encoding_parameter_with_piped_bytes() -> None:
    html_utf8 = "<p>UTF-8: café 日本語</p>".encode()
    stdout, stderr, returncode = run_cli_command([], input_bytes=html_utf8)
    assert returncode == 0
    assert "café 日本語" in stdout
    assert stderr == ""

    html_latin1 = "<p>Latin-1: café naïve résumé</p>".encode("latin-1")
    stdout, stderr, returncode = run_cli_command(["--encoding", "latin-1"], input_bytes=html_latin1)
    assert returncode == 0
    assert "café naïve résumé" in stdout
    assert stderr == ""

    html_windows = '<p>Windows: "smart quotes"</p>'.encode("windows-1252")
    stdout, stderr, returncode = run_cli_command(["--encoding", "windows-1252"], input_bytes=html_windows)
    assert returncode == 0
    assert "Windows:" in stdout
    assert "smart quotes" in stdout
    assert stderr == ""

    html_iso = "<p>ISO: español português</p>".encode("iso-8859-1")
    stdout, stderr, returncode = run_cli_command(["--encoding", "iso-8859-1"], input_bytes=html_iso)
    assert returncode == 0
    assert "español português" in stdout
    assert stderr == ""


def test_encoding_with_file_input(tmp_path: Path) -> None:
    html_content = "<p>File test: café résumé</p>"
    file_path = tmp_path / "test_latin1.html"
    file_path.write_bytes(html_content.encode("latin-1"))

    stdout, stderr, returncode = run_cli_command([str(file_path), "--encoding", "latin-1"])
    assert returncode == 0
    assert "File test: café résumé" in stdout
    assert stderr == ""

    stdout, stderr, returncode = run_cli_command([str(file_path), "--encoding", "utf-8"])
    assert returncode == 0
    assert "File test:" in stdout


def test_source_encoding_parameter(tmp_path: Path) -> None:
    html_content = "<p>Source encoding: café</p>"
    file_path = tmp_path / "test_source.html"
    file_path.write_text(html_content, encoding="latin-1")

    stdout, stderr, returncode = run_cli_command([str(file_path), "--encoding", "latin-1"])
    assert returncode == 0
    assert "Source encoding: café" in stdout
    assert stderr == ""


def test_encoding_cross_platform_compatibility() -> None:
    test_cases = [
        ("<p>Emoji: 😀 🎉 ✨</p>", "Emoji: 😀 🎉 ✨"),
        ("<p>CJK: 日本語 中文 한국어</p>", "CJK: 日本語 中文 한국어"),
        ("<p>Diacritics: àáäâ èéëê ìíïî</p>", "Diacritics: àáäâ èéëê ìíïî"),
        ("<p>Symbols: © ® ™ € £ ¥</p>", "Symbols: © ® ™ € £ ¥"),
        ("<p>Math: ∑ ∫ √ ∞ ≈ ≠</p>", "Math: ∑ ∫ √ ∞ ≈ ≠"),
    ]

    for html, expected_text in test_cases:
        stdout, stderr, returncode = run_cli_command([], input_text=html)
        assert returncode == 0
        assert expected_text in stdout
        assert stderr == ""


def test_encoding_with_invalid_bytes() -> None:
    invalid_bytes = b"<p>Invalid: \xff\xfe</p>"

    stdout, stderr, returncode = run_cli_command([], input_bytes=invalid_bytes)
    assert returncode == 0
    assert "Invalid:" in stdout
    assert stderr == ""


def test_encoding_with_mixed_content(tmp_path: Path) -> None:
    complex_html = """<!DOCTYPE html>
<html>
<head>
    <meta charset="ISO-8859-1">
    <title>Mixed Content Test</title>
</head>
<body>
    <h1>Título en Español</h1>
    <p>Text with special chars: café, naïve, résumé</p>
    <blockquote>Quote: "Hello World"</blockquote>
    <ul>
        <li>Item with - dash</li>
        <li>Item with … ellipsis</li>
    </ul>
</body>
</html>"""

    stdout, stderr, returncode = run_cli_command([], input_text=complex_html)
    assert returncode == 0
    assert "Título en Español" in stdout
    assert stderr == ""

    latin1_bytes = complex_html.encode("latin-1", errors="ignore")
    stdout, stderr, returncode = run_cli_command(["--encoding", "latin-1"], input_bytes=latin1_bytes)
    assert returncode == 0
    assert "Título en Español" in stdout
