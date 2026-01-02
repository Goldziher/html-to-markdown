______________________________________________________________________

## priority: critical

# Core Principles

**Do only what's asked. Never create files unnecessarily. Prefer editing. No proactive docs/READMEs.**

**Python**: Builtin imports at top, dataclasses frozen/hashable/slots, function-based tests only.

**Rust**: Never .unwrap() in production, SAFETY comments for unsafe, handle lock poisoning.

**Architecture**: ALL extraction logic lives in Rust core. Bindings provide language-idiomatic APIs only.
