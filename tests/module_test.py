from __future__ import annotations

import os
import subprocess
import sys
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from pathlib import Path


def run_cli_command(args: list[str], input_text: str | None = None, timeout: int = 60) -> tuple[str, str, int]:
    cli_command = [sys.executable, "-m", "html_to_markdown", *args]

    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8:replace"
    if os.name == "nt":
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
        stdout = (stdout_b or b"").decode("utf-8", "replace").replace("\r\n", "\n")
        stderr = (stderr_b or b"").decode("utf-8", "replace").replace("\r\n", "\n")
        return stdout, stderr, process.returncode
    except subprocess.TimeoutExpired:
        process.kill()
        raise


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


def test_error_handling() -> None:
    _stdout, stderr, returncode = run_cli_command(["nonexistent.html"])
    assert returncode != 0
    assert "No such file" in stderr

    _stdout, stderr, returncode = run_cli_command(["--invalid-option"])
    assert returncode != 0
    assert "unexpected argument" in stderr or "unrecognized" in stderr


def test_stdin_input() -> None:
    input_html = "<h1>Test</h1><p>Content</p>"
    stdout, stderr, returncode = run_cli_command([], input_text=input_html)

    assert returncode == 0
    assert stderr == ""
    assert "Test" in stdout
    assert "Content" in stdout


def test_heading_styles(sample_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(sample_html_file), "--heading-style", "atx"])
    assert "# Sample Document" in stdout

    stdout, _, _ = run_cli_command([str(sample_html_file), "--heading-style", "atx-closed"])
    assert "# Sample Document #" in stdout


def test_formatting_options(sample_html_file: Path) -> None:
    stdout, _, _ = run_cli_command([str(sample_html_file), "--strong-em-symbol", "_", "--wrap", "--wrap-width", "40"])

    assert "__test__" in stdout

    assert all(len(line) <= 40 for line in stdout.split("\n"))


def test_code_block_options(complex_html_file: Path) -> None:
    stdout, _, _ = run_cli_command(
        [str(complex_html_file), "--code-block-style", "backticks", "--code-language", "python"]
    )

    assert "```python" in stdout


def test_special_characters() -> None:
    input_html = "<p>Text with * and _ and ** symbols</p>"

    # v2 default: minimal escaping (no escaping)
    stdout, _, _ = run_cli_command([], input_text=input_html)
    assert "\\*" not in stdout
    assert "\\_" not in stdout

    # With explicit escaping flags
    stdout, _, _ = run_cli_command(["--escape-asterisks", "--escape-underscores"], input_text=input_html)
    assert "\\*" in stdout
    assert "\\_" in stdout


def test_large_file_handling(tmp_path: Path) -> None:
    large_file = tmp_path / "large.html"

    with large_file.open("w") as f:
        f.write("<p>")
        for i in range(50000):
            f.write(f"Line {i} with some <b>bold</b> text.\n")
        f.write("</p>")

    stdout, stderr, returncode = run_cli_command(
        [str(large_file)],
        timeout=120,
    )

    assert returncode == 0
    assert stderr == ""
    assert "Line 0" in stdout
    assert "Line 49999" in stdout


def test_unicode_handling() -> None:
    input_html = "<p>Unicode: 你好 • é è à ñ</p>"
    stdout, _stderr, returncode = run_cli_command([], input_text=input_html)

    assert returncode == 0
    assert "你好" in stdout
    assert "é è à ñ" in stdout


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


@pytest.mark.parametrize("newline_style", ["spaces", "backslash"])
def test_newline_styles(newline_style: str) -> None:
    input_html = "<p>Line 1<br>Line 2</p>"
    stdout, _, _ = run_cli_command(["--newline-style", newline_style], input_text=input_html)

    expected_break = "\\\n" if newline_style == "backslash" else "  \n"
    assert expected_break in stdout
