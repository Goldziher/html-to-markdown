______________________________________________________________________

## priority: critical

# Python Modern & Performance Standards

**Python 3.10+ · Functional-first · msgspec · Fully async · Strongest typing**

- Target Python 3.10+; match/case, union types (X | Y), structural pattern matching
- msgspec ONLY (NEVER pydantic); msgspec.Struct with slots=True, kw_only=True, frozen=True
- Full type hints: ParamSpec for decorators, TypeVar/Generic[T], Protocol for structural typing
- Enable mypy --strict --warn-unreachable --disallow-any-expr; never use Any
- Functional patterns: pure functions, composition, map/filter/reduce, immutability
- Walrus operator := in comprehensions; match/case for conditionals
- contextlib.suppress for intentional exception suppression
- O(1) optimization: dict/set lookups over if/elif chains
- Fully async: anyio.Path (not pathlib), httpx AsyncClient, asyncpg, asyncio.gather
- Function-based tests ONLY (\*\_test.py); pytest fixtures, 95% coverage, real PostgreSQL
- Never: class tests, pydantic, sync I/O in async, Any type, Optional[T] (use T | None)
