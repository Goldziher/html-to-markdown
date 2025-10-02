# Rust Implementation Plan - Executive Summary

## Status Overview

**Current Completion**: ~25% (14/80+ tags, basic features only)
**Estimated Time to Production**: 3-4 weeks (20-25 days)
**Critical Blockers**: 4 (text escaping, whitespace, ordered lists, table bugs)

## What Works Now ✅

- Basic HTML tags: h1-h6, p, strong, b, em, i, a, img
- Lists: ul, li (bullets only, ol broken)
- Tables: Basic tables with colspan (rowspan broken)
- Code: inline code, pre blocks
- Misc: blockquote, br, hr
- Infrastructure: html5ever parser, PyO3 bindings, ammonia sanitization

## What's Broken ❌

### Critical (Blockers)

1. **No text escaping** → `Use *wildcards*` renders as italics instead of `Use \*wildcards\*`
1. **No whitespace normalization** → Spacing between inline elements is wrong
1. **Ordered lists broken** → All output as bullets instead of `1. 2. 3.`
1. **Table rowspan broken** → First-row detection always fails

### Missing Features

- 60+ HTML tags (mark, del, ins, cite, q, dl/dt/dd, forms, media, etc.)
- Custom converters (Python callbacks)
- Text wrapping
- Metadata extraction
- Autolinks
- convert/strip options
- convert_as_inline support

## Implementation Strategy

### 11-Phase Plan (See TODO.md)

**Phase 1: Critical Blockers (3 days)**

- Text escaping system
- Whitespace normalization (chomp, modes)
- Fix ordered lists
- Fix table bugs

**Phase 2: Core HTML Tags (4 days)**

- Inline formatting (mark, del, ins, sub, sup, etc.)
- Semantic blocks (article, section, nav, etc.)
- Citations (cite, q)
- Definition lists (dl, dt, dd)

**Phase 3-4: Interactive & Forms (4 days)**

- Interactive elements (details, summary, dialog)
- Media tags (audio, video, iframe, svg)
- Form elements (input, select, button, etc.)

**Phase 5: Advanced Features (4 days)**

- Ruby annotations
- Task lists (GitHub checkboxes)
- Custom converters (Python callbacks)
- Text wrapping
- Metadata extraction
- Autolinks

**Phase 6-8: Integration (3 days)**

- Enhanced preprocessing options
- hOCR support
- PyO3 bindings completion
- Replace Python implementation

**Phase 9: Testing (5 days)**

- Run all 700+ tests
- Fix failures iteratively
- Track progress per test file

**Phase 10-11: Release (2 days)**

- Documentation
- Performance benchmarks
- CI/CD setup
- Release preparation

## Development Workflow

### Per Session

1. Check `TODO.md` for current phase/task
1. Reference Python implementation in `html_to_markdown/converters.py`
1. Implement in `crates/html-to-markdown/src/converter.rs`
1. Update PyO3 bindings if needed
1. Run tests: `pytest tests/elements_test.py -v`
1. Update `TODO.md` with ✅
1. Commit progress

### Quick Commands

```bash
# Build Rust
cargo build --release

# Install with Rust backend
pip install -e .

# Run specific test
pytest tests/elements_test.py::test_cite_element -v

# Run all non-benchmark tests
pytest tests/ -k "not benchmark" -v

# Use Python fallback for comparison
env HTML_TO_MD_USE_PYTHON=1 pytest tests/...
```

## Next Steps

**Immediate (Start of Next Session)**:

1. Open `TODO.md`
1. Start Phase 1.1: Text Escaping Integration
1. Reference: `html_to_markdown/utils.py:escape()`
1. Implement: Apply escaping in `converter.rs`
1. Test: Create simple test for `*` escaping
1. Mark complete in `TODO.md`

---

**Remember**: The foundation is solid. This is about adding features systematically. Use Python code as reference, tests will tell you when you're done.
