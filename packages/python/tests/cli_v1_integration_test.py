from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path


class TestCLIV1Integration:
    def run_cli(self, args: list[str], input_html: str = "") -> tuple[str, str, int]:
        result = subprocess.run(
            ["python", "-m", "html_to_markdown", *args],
            input=input_html,
            capture_output=True,
            text=True,
            check=False,
        )
        return result.stdout, result.stderr, result.returncode

    def test_basic_conversion_stdin(self) -> None:
        html = "<p>Hello <strong>world</strong>!</p>"
        stdout, stderr, returncode = self.run_cli(["-"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "Hello **world**!" in stdout

    def test_file_conversion_with_v1_flags(self) -> None:
        html = "<h1>Title</h1><p>Content with <em>emphasis</em>.</p>"

        with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f:
            f.write(html)
            input_file = Path(f.name)

        try:
            stdout, stderr, returncode = self.run_cli([str(input_file), "--heading-style", "atx", "--escape-asterisks"])
            assert returncode == 0, f"CLI failed: {stderr}"
            assert "# Title" in stdout
            assert "*emphasis*" in stdout
        finally:
            input_file.unlink()

    def test_preprocess_html_flag_translation(self) -> None:
        html = "<nav>Navigation</nav><p>Content</p>"
        stdout, stderr, returncode = self.run_cli(["-", "--preprocess-html"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "Content" in stdout

    def test_boolean_flags_translation(self) -> None:
        html = "Text with *asterisks* and _underscores_"

        stdout, stderr, returncode = self.run_cli(["-", "--no-escape-asterisks"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "*asterisks*" in stdout

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--escape-asterisks"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert r"\*asterisks\*" in stdout2

    def test_output_to_file(self) -> None:
        html = "<h2>Heading</h2><ul><li>Item</li></ul>"

        with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f_in:
            f_in.write(html)
            input_file = Path(f_in.name)

        with tempfile.NamedTemporaryFile(mode="r", suffix=".md", delete=False) as f_out:
            output_file = Path(f_out.name)

        try:
            _stdout, stderr, returncode = self.run_cli(
                [str(input_file), "-o", str(output_file), "--heading-style", "atx", "--bullets", "-"]
            )
            assert returncode == 0, f"CLI failed: {stderr}"

            content = output_file.read_text()
            assert "## Heading" in content
            assert "- Item" in content
        finally:
            input_file.unlink()
            output_file.unlink()

    def test_unsupported_v1_flag_strip_raises_error(self) -> None:
        html = "<a href='#'>link</a>"
        _stdout, stderr, returncode = self.run_cli(["-", "--strip", "a"], html)
        assert returncode != 0
        assert "--strip option has been removed in v2" in stderr

    def test_unsupported_v1_flag_convert_raises_error(self) -> None:
        html = "<a href='#'>link</a>"
        _stdout, stderr, returncode = self.run_cli(["-", "--convert", "a"], html)
        assert returncode != 0
        assert "--convert option has been removed in v2" in stderr

    def test_complex_v1_command(self) -> None:
        html = """
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Main Title</h1>
            <p>Text with <strong>bold</strong> and <em>italic</em>.</p>
            <ul>
                <li>First item</li>
                <li>Second item</li>
            </ul>
            <pre><code>def hello():
    print("world")</code></pre>
        </body>
        </html>
        """

        with tempfile.NamedTemporaryFile(mode="w", suffix=".html", delete=False) as f_in:
            f_in.write(html)
            input_file = Path(f_in.name)

        try:
            stdout, stderr, returncode = self.run_cli(
                [
                    str(input_file),
                    "--heading-style",
                    "atx",
                    "--bullets",
                    "-",
                    "--list-indent-width",
                    "2",
                    "--code-language",
                    "python",
                    "--strong-em-symbol",
                    "_",
                    "--extract-metadata",
                    "--escape-asterisks",
                ]
            )
            assert returncode == 0, f"CLI failed: {stderr}"
            assert "# Main Title" in stdout
            assert "__bold__" in stdout
            assert "_italic_" in stdout
            assert "- First item" in stdout
            assert "    def hello():" in stdout
            assert "title: Test Page" in stdout
        finally:
            input_file.unlink()

    def test_autolinks_flag(self) -> None:
        html = '<a href="https://example.com">https://example.com</a>'

        stdout, stderr, returncode = self.run_cli(["-", "--autolinks"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "example.com" in stdout

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--no-autolinks"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert "example.com" in stdout2

    def test_metadata_extraction_flags(self) -> None:
        html = "<html><head><title>My Page</title></head><body><p>Content</p></body></html>"

        stdout, stderr, returncode = self.run_cli(["-", "--extract-metadata"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "title: My Page" in stdout

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--no-extract-metadata"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert "title:" not in stdout2

    def test_wrap_flags(self) -> None:
        html = "<p>This is a long line of text that should be wrapped when wrap is enabled with a specific width.</p>"

        stdout, stderr, returncode = self.run_cli(["-", "--wrap", "--wrap-width", "20"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        [line for line in stdout.split("\n") if line.strip()]

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--no-wrap"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert "long line of text" in stdout2

    def test_highlight_style(self) -> None:
        html = "<mark>highlighted text</mark>"

        stdout, stderr, returncode = self.run_cli(["-", "--highlight-style", "double-equal"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "==highlighted text==" in stdout

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--highlight-style", "bold"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert "**highlighted text**" in stdout2

    def test_newline_style(self) -> None:
        html = "Line 1<br />Line 2"

        stdout, stderr, returncode = self.run_cli(["-", "--newline-style", "spaces"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "Line 1  \nLine 2" in stdout

        stdout2, stderr2, returncode2 = self.run_cli(["-", "--newline-style", "backslash"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"
        assert "Line 1\\\nLine 2" in stdout2

    def test_sub_sup_symbols(self) -> None:
        html = "H<sub>2</sub>O and E=mc<sup>2</sup>"

        stdout, stderr, returncode = self.run_cli(["-", "--sub-symbol", "~", "--sup-symbol", "^"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert "H~2~O" in stdout
        assert "mc^2^" in stdout

    def test_convert_as_inline(self) -> None:
        html = "<p>Just text</p>"

        stdout, stderr, returncode = self.run_cli(["-", "--convert-as-inline"], html)
        assert returncode == 0, f"CLI failed: {stderr}"
        assert stdout.strip() == "Just text"

    def test_br_in_tables(self) -> None:
        html = """
        <table>
            <tr><td>Line 1<br />Line 2</td></tr>
        </table>
        """

        _stdout, stderr, returncode = self.run_cli(["-"], html)
        assert returncode == 0, f"CLI failed: {stderr}"

        _stdout2, stderr2, returncode2 = self.run_cli(["-", "--br-in-tables"], html)
        assert returncode2 == 0, f"CLI failed: {stderr2}"


class TestCLIV1ErrorHandling:
    def run_cli(self, args: list[str], input_html: str = "") -> tuple[str, str, int]:
        result = subprocess.run(
            ["python", "-m", "html_to_markdown", *args],
            input=input_html,
            capture_output=True,
            text=True,
            check=False,
        )
        return result.stdout, result.stderr, result.returncode

    def test_invalid_heading_style(self) -> None:
        html = "<h1>Title</h1>"
        _stdout, _stderr, returncode = self.run_cli(["-", "--heading-style", "invalid"], html)
        assert returncode != 0

    def test_file_not_found(self) -> None:
        _stdout, _stderr, returncode = self.run_cli(["nonexistent.html"])
        assert returncode != 0

    def test_empty_input_from_stdin(self) -> None:
        stdout, _stderr, returncode = self.run_cli(["-"], "")
        assert returncode == 0
        assert stdout == ""
