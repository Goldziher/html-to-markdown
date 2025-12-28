# Python Test App for html-to-markdown

Tests the published html-to-markdown package from PyPI.

## Setup

```bash
uv sync
```

## Run Tests

```bash
# Smoke tests (fast)
uv run pytest smoke_test.py -v

# Comprehensive tests
uv run pytest comprehensive_test.py -v

# All tests
uv run pytest -v
```
