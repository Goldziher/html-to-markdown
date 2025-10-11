from __future__ import annotations

import pytest

from html_to_markdown.cli_proxy import translate_v1_args_to_v2
from html_to_markdown.exceptions import RemovedV1FlagError


class TestCLITranslationBasic:
    def test_passthrough_unchanged_args(self) -> None:
        args = ["input.html", "-o", "output.md", "--heading-style", "atx"]
        result = translate_v1_args_to_v2(args)
        assert result == args

    def test_empty_args(self) -> None:
        result = translate_v1_args_to_v2([])
        assert result == []

    def test_stdin_stdout(self) -> None:
        args = ["-"]
        result = translate_v1_args_to_v2(args)
        assert result == ["-"]


class TestCLITranslationFlagNames:
    def test_preprocess_html_to_preprocess(self) -> None:
        args = ["--preprocess-html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--preprocess"]

    def test_preprocess_html_with_other_args(self) -> None:
        args = ["input.html", "--preprocess-html", "--preset", "aggressive"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html", "--preprocess", "--preset", "aggressive"]


class TestCLITranslationBooleanFlags:
    def test_escape_asterisks_preserved(self) -> None:
        args = ["--escape-asterisks"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--escape-asterisks"]

    def test_no_escape_asterisks_silently_accepted(self) -> None:
        args = ["--no-escape-asterisks", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_escape_underscores_preserved(self) -> None:
        args = ["--escape-underscores"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--escape-underscores"]

    def test_no_escape_underscores_silently_accepted(self) -> None:
        args = ["--no-escape-underscores", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_escape_misc_preserved(self) -> None:
        args = ["--escape-misc"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--escape-misc"]

    def test_no_escape_misc_silently_accepted(self) -> None:
        args = ["--no-escape-misc", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_autolinks_preserved(self) -> None:
        args = ["--autolinks"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--autolinks"]

    def test_no_autolinks_silently_accepted(self) -> None:
        args = ["--no-autolinks", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_extract_metadata_preserved(self) -> None:
        args = ["--extract-metadata"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--extract-metadata"]

    def test_no_extract_metadata_silently_accepted(self) -> None:
        args = ["--no-extract-metadata", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_wrap_preserved(self) -> None:
        args = ["--wrap"]
        result = translate_v1_args_to_v2(args)
        assert result == ["--wrap"]

    def test_no_wrap_silently_accepted(self) -> None:
        args = ["--no-wrap", "input.html"]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]


class TestCLITranslationUnsupportedFlags:
    def test_strip_flag_raises(self) -> None:
        args = ["--strip", "nav,footer"]
        with pytest.raises(RemovedV1FlagError) as exc_info:
            translate_v1_args_to_v2(args)
        assert exc_info.value.flag == "--strip"
        assert "removed" in exc_info.value.reason.lower()

    def test_convert_flag_raises(self) -> None:
        args = ["--convert", "a,img"]
        with pytest.raises(RemovedV1FlagError) as exc_info:
            translate_v1_args_to_v2(args)
        assert exc_info.value.flag == "--convert"
        assert "removed" in exc_info.value.reason.lower()


class TestCLITranslationComplex:
    def test_multiple_flag_translations(self) -> None:
        args = [
            "input.html",
            "--preprocess-html",
            "--escape-underscores",
            "--autolinks",
            "-o",
            "output.md",
        ]
        result = translate_v1_args_to_v2(args)
        expected = [
            "input.html",
            "--preprocess",
            "--escape-underscores",
            "--autolinks",
            "-o",
            "output.md",
        ]
        assert result == expected

    def test_all_boolean_flags_default(self) -> None:
        args = [
            "--no-escape-asterisks",
            "input.html",
        ]
        result = translate_v1_args_to_v2(args)
        assert result == ["input.html"]

    def test_all_boolean_flags_non_default(self) -> None:
        args = [
            "--escape-asterisks",
            "--escape-underscores",
            "--escape-misc",
            "--extract-metadata",
            "--autolinks",
            "--wrap",
        ]
        result = translate_v1_args_to_v2(args)
        assert result == args

    def test_mixed_renamed_and_boolean_flags(self) -> None:
        args = [
            "input.html",
            "--preprocess-html",
            "--preset",
            "aggressive",
            "--escape-asterisks",
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
            "--escape-asterisks",
            "--heading-style",
            "atx",
            "--autolinks",
        ]
        assert result == expected


class TestCLITranslationEdgeCases:
    def test_flags_with_values(self) -> None:
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
        args1 = ["-o", "output.md"]
        assert translate_v1_args_to_v2(args1) == args1

        args2 = ["--output", "output.md"]
        assert translate_v1_args_to_v2(args2) == args2

    def test_complex_realistic_command(self) -> None:
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
            "--escape-asterisks",
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
            "--escape-asterisks",
            "--autolinks",
            "--code-language",
            "python",
        ]
        assert result == expected
