______________________________________________________________________

## priority: critical

# Error Handling Strategy

**CRITICAL: OSError/RuntimeError must ALWAYS bubble up** (Python + Rust). SystemExit, KeyboardInterrupt, MemoryError too.

**Python**: Exception-based, inherit from KreuzbergError. OSError patterns: 1) Library misuse→bubble up, 2) Subprocess→analyze stderr for parsing keywords, 3) Cache→ignore, 4) Dependencies→MissingDependencyError or bubble up. Always add ~keep comments.

**Rust**: KreuzbergError::Io always bubbles up unchanged. Result\<T, KreuzbergError>, never .unwrap() in production, use `?`.

**Exception hierarchy**: ValidationError, ParsingError, OCRError, MissingDependencyError.
