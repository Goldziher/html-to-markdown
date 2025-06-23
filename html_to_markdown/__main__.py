import sys

from html_to_markdown.cli import main


def cli() -> None:
    """Main CLI entrypoint."""
    try:
        result = main(sys.argv[1:])
        print(result)  # noqa: T201
    except ValueError as e:
        print(str(e), file=sys.stderr)  # noqa: T201
        sys.exit(1)


if __name__ == "__main__":
    cli()
