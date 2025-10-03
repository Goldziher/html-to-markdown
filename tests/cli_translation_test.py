"""Tests for CLI v1 -> v2 argument translation."""

from __future__ import annotations

import pytest

from html_to_markdown.cli_proxy import translate_v1_args_to_v2


class TestCLITranslationBasic:
    """Test basic CLI argument translation."""

    def test_passthrough_unchanged_args(self) -> None:
        """Test that common args pass through unchanged."""
        args = ["input.html", "-o", "output.md", "--heading-style", "atx"]
        result = translate_v1_args_to_v2(args)
        assert result == args

    def test_empty_args(self) -> None:
        """Test empty argument list."""
        result = translate_v1_args_to_v2([])
        assert result == []

    def test_stdin_stdout(self) -> None:
        """Test stdin/stdout arguments."""
        args = ["-"]
        result = translate_v1_args_to_v2(args)
        assert result == ["-"]


class TestCLITranslationFlagNames:
    """Test translation of renamed flags."""

    def test_preprocess_html_to_preprocess(self) -> None:
        """Test --preprocess-html -> --preprocess."""
        args = ["--preprocess-html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--preprocess"]

    def test_preprocess_html_with_other_args(self) -> None:
        """Test --preprocess-html with other arguments."""
        args = ["input.html", "--preprocess-html", "--preset", "aggressive"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html", "--preprocess", "--preset", "aggressive"]


class TestCLITranslationBooleanFlags:
    """Test translation of boolean flags."""

    def test_escape_asterisks_default_removed(self) -> None:
        """Test --escape-asterisks (default) is removed."""
        args = ["--escape-asterisks"]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_no_escape_asterisks_preserved(self) -> None:
        """Test --no-escape-asterisks is preserved."""
        args = ["--no-escape-asterisks"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--no-escape-asterisks"]

    def test_escape_underscores_default_removed(self) -> None:
        """Test --escape-underscores (default) is removed."""
        args = ["--escape-underscores"]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_no_escape_underscores_preserved(self) -> None:
        """Test --no-escape-underscores is preserved."""
        args = ["--no-escape-underscores"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--no-escape-underscores"]

    def test_escape_misc_default_removed(self) -> None:
        """Test --escape-misc (default) is removed."""
        args = ["--escape-misc"]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_no_escape_misc_preserved(self) -> None:
        """Test --no-escape-misc is preserved."""
        args = ["--no-escape-misc"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--no-escape-misc"]

    def test_autolinks_preserved(self) -> None:
        """Test --autolinks is preserved."""
        args = ["--autolinks"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--autolinks"]

    def test_no_autolinks_removed(self) -> None:
        """Test --no-autolinks (default) is removed."""
        args = ["--no-autolinks"]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_extract_metadata_default_removed(self) -> None:
        """Test --extract-metadata (default) is removed."""
        args = ["--extract-metadata"]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_no_extract_metadata_preserved(self) -> None:
        """Test --no-extract-metadata is preserved."""
        args = ["--no-extract-metadata"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--no-extract-metadata"]

    def test_wrap_preserved(self) -> None:
        """Test --wrap is preserved."""
        args = ["--wrap"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--wrap"]

    def test_no_wrap_removed(self) -> None:
        """Test --no-wrap (default) is removed."""
        args = ["--no-wrap"]
        result = translate_v1_args_to_v2(args)
        assert result == []


class TestCLITranslationUnsupportedFlags:
    """Test that unsupported v1 flags raise errors."""

    def test_strip_flag_raises(self) -> None:
        """Test that --strip raises ValueError."""
        args = ["--strip", "nav,footer"]
        with pytest.raises(ValueError, match="--strip option is not supported"):
            translate_v1_args_to_v2(args)

    def test_convert_flag_raises(self) -> None:
        """Test that --convert raises ValueError."""
        args = ["--convert", "a,img"]
        with pytest.raises(ValueError, match="--convert option is not supported"):
            translate_v1_args_to_v2(args)


class TestCLITranslationComplex:
    """Test complex argument combinations."""

    def test_multiple_flag_translations(self) -> None:
        """Test multiple flag translations together."""
        args = [
            "input.html",
            "--preprocess-html",
            "--no-escape-asterisks",
            "--escape-underscores",  # Should be removed
            "--autolinks",
            "-o",
            "output.md",
        ]
        result = translate_v1_args_to_v2(args)
        expected = [
            "input.html",
            "--preprocess",
            "--no-escape-asterisks",
            "--autolinks",
            "-o",
            "output.md",
        ]
        assert result == expected

    def test_all_boolean_flags_default(self) -> None:
        """Test all boolean flags with default values (should be removed)."""
        args = [
            "--escape-asterisks",
            "--escape-underscores",
            "--escape-misc",
            "--extract-metadata",
            "--no-autolinks",
            "--no-wrap",
        ]
        result = translate_v1_args_to_v2(args)
        assert result == []

    def test_all_boolean_flags_non_default(self) -> None:
        """Test all boolean flags with non-default values (should be preserved)."""
        args = [
            "--no-escape-asterisks",
            "--no-escape-underscores",
            "--no-escape-misc",
            "--no-extract-metadata",
            "--autolinks",
            "--wrap",
        ]
        result = translate_v1_args_to_v2(args)
        assert result == args

    def test_mixed_renamed_and_boolean_flags(self) -> None:
        """Test mix of renamed and boolean flags."""
        args = [
            "input.html",
            "--preprocess-html",
            "--preset",
            "aggressive",
            "--no-escape-asterisks",
            "--heading-style",
            "atx",
            "--autolinks",
        ]
        result = translate_v1_args_to_v2(args)
        expected = [
            "input.html",
            "--preprocess",
            "--preset",
            "aggressive",
            "--no-escape-asterisks",
            "--heading-style",
            "atx",
            "--autolinks",
        ]
        assert result == expected


class TestCLITranslationEdgeCases:
    """Test edge cases."""

    def test_flags_with_values(self) -> None:
        """Test that flags with values are preserved."""
        args = [
            "--heading-style",
            "atx",
            "--bullets",
            "*",
            "--list-indent-width",
            "2",
            "--code-language",
            "python",
        ]
        result = translate_v1_args_to_v2(args)
        assert result == args

    def test_output_flag_variations(self) -> None:
        """Test output flag variations."""
        # Short form
        args1 = ["-o", "output.md"]
        assert translate_v1_args_to_v2(args1) == args1

        # Long form
        args2 = ["--output", "output.md"]
        assert translate_v1_args_to_v2(args2) == args2

    def test_complex_realistic_command(self) -> None:
        """Test a realistic complex command."""
        args = [
            "page.html",
            "-o",
            "page.md",
            "--heading-style",
            "atx",
            "--bullets",
            "-",
            "--list-indent-width",
            "2",
            "--preprocess-html",
            "--preset",
            "aggressive",
            "--no-escape-asterisks",
            "--autolinks",
            "--code-language",
            "python",
        ]
        result = translate_v1_args_to_v2(args)
        expected = [
            "page.html",
            "-o",
            "page.md",
            "--heading-style",
            "atx",
            "--bullets",
            "-",
            "--list-indent-width",
            "2",
            "--preprocess",
            "--preset",
            "aggressive",
            "--no-escape-asterisks",
            "--autolinks",
            "--code-language",
            "python",
        ]
        assert result == expected
