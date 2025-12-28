# Test Fixtures

Shared test fixtures for testing published html-to-markdown packages across all language bindings.

## Format

Each fixture file contains a JSON array of test cases:

```json
[
  {
    "name": "Test case name",
    "html": "<p>Input HTML</p>",
    "expectedMarkdown": "Expected markdown output",
    "options": {}
  }
]
```

## Files

- **basic-html.json** - 100 basic HTML elements (headings, paragraphs, lists, etc.)
- **complex-html.json** - 50 complex structures (nested lists, tables, etc.)
- **edge-cases.json** - 30 edge cases (special chars, Unicode, entities)
- **metadata-extraction.json** - 20 metadata extraction tests
- **real-world.json** - 10 real-world HTML samples
