"""Test exceptions module."""

from html_to_markdown.exceptions import (
    ConflictingOptionsError,
    EmptyHtmlError,
    HtmlToMarkdownError,
    InvalidParserError,
    MissingDependencyError,
)


def test_html_to_markdown_error():
    """Test base exception class."""
    error = HtmlToMarkdownError("test message")
    assert str(error) == "test message"
    assert isinstance(error, Exception)


def test_missing_dependency_error_with_install_command():
    """Test MissingDependencyError with install command."""
    error = MissingDependencyError("lxml", "pip install lxml")

    assert error.dependency == "lxml"
    assert error.install_command == "pip install lxml"
    assert str(error) == "lxml is not installed. Install with: pip install lxml"


def test_missing_dependency_error_without_install_command():
    """Test MissingDependencyError without install command."""
    error = MissingDependencyError("unknown-lib", None)

    assert error.dependency == "unknown-lib"
    assert error.install_command is None
    assert str(error) == "unknown-lib is not installed."


def test_missing_dependency_error_without_install_param():
    """Test MissingDependencyError without install parameter."""
    error = MissingDependencyError("another-lib")

    assert error.dependency == "another-lib"
    assert error.install_command is None
    assert str(error) == "another-lib is not installed."


def test_invalid_parser_error():
    """Test InvalidParserError."""
    available = ["html.parser", "lxml", "html5lib"]
    error = InvalidParserError("invalid", available)

    assert error.parser == "invalid"
    assert error.available_parsers == available
    assert str(error) == "Invalid parser 'invalid'. Available parsers: html.parser, lxml, html5lib"


def test_empty_html_error():
    """Test EmptyHtmlError."""
    error = EmptyHtmlError()
    assert str(error) == "The input HTML is empty."


def test_conflicting_options_error():
    """Test ConflictingOptionsError."""
    error = ConflictingOptionsError("strip", "convert")

    assert error.option1 == "strip"
    assert error.option2 == "convert"
    assert str(error) == "Only one of 'strip' and 'convert' can be specified."


def test_exceptions_inheritance():
    """Test that all custom exceptions inherit from base exception."""
    exceptions = [
        MissingDependencyError("test"),
        InvalidParserError("test", []),
        EmptyHtmlError(),
        ConflictingOptionsError("a", "b"),
    ]

    for exc in exceptions:
        assert isinstance(exc, HtmlToMarkdownError)
        assert isinstance(exc, Exception)
