"""Integration tests for CLI functionality with actual conversions."""

import subprocess
import sys
import tempfile


def run_cli(args: list[str], input_html: str) -> str:
    """Run the CLI with given arguments and input."""
    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", *args],
        check=False,
        input=input_html,
        capture_output=True,
        text=True,
    )
    return result.stdout


def test_cli_discord_list_indentation() -> None:
    """Test Discord-compatible 2-space list indentation."""
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li><li>Item 2</li></ul>"
    output = run_cli(["--list-indent-width", "2", "--no-extract-metadata"], html)
    assert "* Item 1\n  + Nested\n* Item 2" in output


def test_cli_tab_list_indentation() -> None:
    """Test tab-based list indentation."""
    html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>"
    output = run_cli(["--list-indent-type", "tabs", "--no-extract-metadata"], html)
    assert "* Item 1\n\t+ Nested" in output


def test_cli_whitespace_mode_strict() -> None:
    """Test strict whitespace preservation."""
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "strict", "--no-extract-metadata"], html)
    # In strict mode, spaces are still normalized within text content
    # but block-level spacing is preserved
    assert "Multiple spaces here" in output


def test_cli_whitespace_mode_normalized() -> None:
    """Test normalized whitespace mode."""
    html = "<p>  Multiple   spaces   here  </p>"
    output = run_cli(["--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "Multiple spaces here" in output


def test_cli_parser_selection() -> None:
    """Test parser selection (if lxml is available)."""
    html = "<div>Test</div>"
    # This should work with any parser
    output = run_cli(["--parser", "html.parser", "--no-extract-metadata"], html)
    assert "Test" in output


def test_cli_preprocess_html() -> None:
    """Test HTML preprocessing."""
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


def test_cli_preprocess_keep_forms() -> None:
    """Test keeping forms during preprocessing."""
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


def test_cli_preprocess_keep_navigation() -> None:
    """Test keeping navigation during preprocessing."""
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


def test_cli_preprocessing_preset_minimal() -> None:
    """Test minimal preprocessing preset."""
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


def test_cli_preprocessing_preset_aggressive() -> None:
    """Test aggressive preprocessing preset."""
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
    # Aggressive mode may remove semantic elements
    assert "Main content" in output


def test_cli_combined_list_and_whitespace() -> None:
    """Test combining list indentation and whitespace modes."""
    html = "<ul><li>Item  with   spaces<ul><li>Nested  item</li></ul></li></ul>"
    output = run_cli(["--list-indent-width", "2", "--whitespace-mode", "normalized", "--no-extract-metadata"], html)
    assert "* Item with spaces\n  + Nested item" in output


def test_cli_all_new_options_combined() -> None:
    """Test all new CLI options combined."""
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
    assert "Navigation" not in output  # Removed by preprocessing
    assert "* Item 1\n   + Nested" in output  # 3-space indentation


def test_cli_help_includes_new_options() -> None:
    """Test that help text includes all new options."""
    result = subprocess.run(
        [sys.executable, "-m", "html_to_markdown", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    help_text = result.stdout

    # Check for new options in help
    assert "--parser" in help_text
    assert "--list-indent-type" in help_text
    assert "--list-indent-width" in help_text
    assert "--whitespace-mode" in help_text
    assert "--preprocess-html" in help_text
    assert "--preprocessing-preset" in help_text
    assert "--no-remove-forms" in help_text
    assert "--no-remove-navigation" in help_text
    assert "--source_encoding" in help_text

    # Check for Discord mention
    assert "Discord" in help_text or "2" in help_text


def test_cli_source_encoding_with_stdin() -> None:
    """Test that source_encoding is ignored when using stdin."""
    html = "<html><body><h1>Test with stdin</h1></body></html>"
    output = run_cli(["--source_encoding", "utf-8", "--no-extract-metadata"], html)
    assert "Test with stdin" in output


def test_cli_source_encoding_with_file() -> None:
    """Test source_encoding with an actual file.

    Create a temporary HTML file with a specific encoding and read it using the CLI.
    """
    html_content = "<html><body><h1>Tëst with special çharacters: ñáéíóú</h1></body></html>"

    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", delete=False, suffix=".html") as f:
        f.write(html_content)
        temp_file = f.name

        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "html_to_markdown",
                temp_file,
                "--source_encoding",
                "utf-8",
                "--no-extract-metadata",
            ],
            check=False,
            capture_output=True,
            text=True,
        )

        output = result.stdout
        assert "Tëst with special çharacters" in output
        assert "ñáéíóú" in output


def test_cli_source_encoding_invalid_encoding() -> None:
    """Test that invalid encoding produces an error.

    Create a temporary HTML file with a specific encoding and read it using the CLI.
    Pass a different, invalid encoding to trigger the error (resultcode != 0).
    """
    html_content = "<html><body><h1>Test content</h1></body></html>"

    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", delete=False, suffix=".html") as f:
        f.write(html_content)
        temp_file = f.name

        result = subprocess.run(
            [sys.executable, "-m", "html_to_markdown", temp_file, "--source_encoding", "invalid-encoding"],
            check=False,
            capture_output=True,
            text=True,
        )

        assert result.returncode != 0
        assert "encoding" in result.stderr.lower() or "invalid" in result.stderr.lower()
