from __future__ import annotations


class HtmlToMarkdownError(Exception):
    pass


class MissingDependencyError(HtmlToMarkdownError):
    def __init__(self, dependency: str, install_command: str | None = None) -> None:
        self.dependency = dependency
        self.install_command = install_command

        message = f"{dependency} is not installed."
        if install_command:
            message += f" Install with: {install_command}"

        super().__init__(message)


class InvalidParserError(HtmlToMarkdownError):
    def __init__(self, parser: str, available_parsers: list[str]) -> None:
        self.parser = parser
        self.available_parsers = available_parsers

        message = f"Invalid parser '{parser}'. Available parsers: {', '.join(available_parsers)}"
        super().__init__(message)


class EmptyHtmlError(HtmlToMarkdownError):
    def __init__(self) -> None:
        super().__init__("The input HTML is empty.")


class ConflictingOptionsError(HtmlToMarkdownError):
    def __init__(self, option1: str, option2: str) -> None:
        self.option1 = option1
        self.option2 = option2

        super().__init__(f"Only one of '{option1}' and '{option2}' can be specified.")


class InvalidEncodingError(HtmlToMarkdownError):
    def __init__(self, encoding: str) -> None:
        super().__init__(f"The specified encoding ({encoding}) is not valid.")
