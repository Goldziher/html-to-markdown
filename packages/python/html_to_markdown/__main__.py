import sys

from html_to_markdown.cli_proxy import main


def cli() -> None:
    try:
        result = main(sys.argv[1:])
        print(result, end="")  # noqa: T201
    except (ValueError, FileNotFoundError) as e:
        print(str(e), file=sys.stderr)  # noqa: T201
        sys.exit(1)


if __name__ == "__main__":
    cli()
