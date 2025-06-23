# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a modern Python library and CLI tool for converting HTML to Markdown. It's a complete rewrite of the original `markdownify` library with emphasis on type safety, modern Python practices, and extensibility.

## Development Commands

### Initial Setup

```bash
# Install dependencies (uses uv package manager)
uv sync --all-extras --dev

# Install pre-commit hooks (required)
uv run pre-commit install
```

### Testing

```bash
# Run all tests (100% coverage required)
uv run pytest

# Run specific test file
uv run pytest tests/module_test.py

# Run tests with coverage report
uv run pytest --cov=html_to_markdown --cov-report=term-missing

# Run tests matching pattern
uv run pytest -k "test_pattern"
```

### Code Quality

```bash
# Run all pre-commit checks
uv run pre-commit run --all-files

# Lint and fix Python code
uv run ruff check --fix .

# Format Python code
uv run ruff format .

# Type checking (strict mode)
uv run mypy

# Check spelling
uv run codespell
```

### CLI Testing

```bash
# Run CLI during development
uv run python -m html_to_markdown <args>
```

### Build

```bash
# Build package for distribution
uv build
```

## Architecture Overview

### Core Components

- **Public API**: `convert_to_markdown()` and `markdownify()` functions in `__init__.py`
- **Conversion Engine**: `processing.py` handles the recursive HTML tree traversal
- **Converter System**: `converters.py` contains 64+ HTML tag converters using strategy pattern
- **CLI Interface**: `cli.py` exposes all functionality as command-line tool
- **Utilities**: `utils.py` provides escaping, formatting, and text manipulation helpers

### Key Design Patterns

- **Strategy Pattern**: Converter system maps HTML tags to conversion functions
- **Factory Pattern**: `create_converters_map()` builds converter mappings dynamically
- **Functional API**: Main conversion function with 21+ configuration parameters
- **Type Safety**: Comprehensive type hints with strict MyPy compliance

### Converter Architecture

Converters follow the signature: `Callable[[str, Tag], str]`

- **Inline Converters**: Generated for simple markup (bold, italic, etc.)
- **Custom Converters**: Hand-written for complex elements (tables, lists, headings)
- **Parameterized Converters**: Use `functools.partial` for configuration-dependent behavior

### Extension Points

- **Custom Converters**: Override default behavior via `custom_converters` parameter
- **Language Detection**: `code_language_callback` for programming language detection
- **Configuration**: 21 parameters control escaping, formatting, and content handling

## Code Standards

- **Python 3.9+** required
- **Line Length**: 120 characters
- **Docstrings**: Google style convention
- **Type Checking**: Strict MyPy mode enabled
- **Test Coverage**: 100% required (`fail_under = 100`)
- **Commit Messages**: Conventional commits enforced by commitlint

## Important Files

- `pyproject.toml`: Project configuration, dependencies, tool settings
- `.pre-commit-config.yaml`: 17 different code quality hooks
- `html_to_markdown/__init__.py`: Public API exports
- `html_to_markdown/processing.py`: Core conversion logic
- `html_to_markdown/converters.py`: HTML tag conversion strategies
- `tests/`: Test suite with 100% coverage requirement

## Development Workflow

1. Make code changes
1. Run `uv run ruff check --fix .` and `uv run ruff format .`
1. Run `uv run mypy` for type checking
1. Run `uv run pytest` to ensure tests pass
1. Commit (pre-commit hooks will run automatically)

The project uses GitHub Actions for CI/CD with comprehensive testing across Python 3.9-3.13.
