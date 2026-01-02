______________________________________________________________________

## priority: critical

# Polyglot Error Handling Standardization

**FFI error conversion · Language-specific exceptions · Error context preservation · Safe boundaries**

## Error Conversion at FFI Boundaries

Error handling is language-specific; FFI boundaries MUST convert between error models:

### Rust → Host Language

**Rust errors MUST be converted to language-appropriate types at FFI boundary:**

- **Rust `Result<T, E>`** (sum type) → Host exception/error/nil
- Use dedicated conversion functions; never expose Rust types directly
- Context must be preserved across boundary (error messages, codes)
- All error paths must be handled before returning to host language

### Conversion Example (Rust → Python)

```rust
// Rust core (kreuzberg_core::lib.rs)
pub fn parse_document(data: &[u8]) -> Result<Document, ParseError> {
    // ... parsing logic
}

// FFI boundary (kreuzberg_pyo3::lib.rs)
use pyo3::prelude::*;

#[pyfunction]
fn parse_document(py: Python, data: &[u8]) -> PyResult<PyDocument> {
    kreuzberg_core::parse_document(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::ValueError, _>(
            format!("Parse error: {} at line {}", e.message, e.line)
        ))
        .map(|doc| PyDocument { inner: doc })
}
```

## Language-Specific Error Patterns

### Python Exceptions

- **Exception hierarchy**: Inherit from appropriate base (`ValueError`, `OSError`, `RuntimeError`)
- **Custom exceptions**: Define in `__init__.py` inheriting from `kreuzberg.KreuzbergError`
- **Context preservation**: Include error code, message, line number, suggestion in exception
- **Never silent failures**: Re-raise or log; logging suppression requires ~keep comment

```python
# Python error handling
class KreuzbergError(Exception):
    """Base exception for Kreuzberg errors"""
    def __init__(self, message: str, code: int | None = None, context: dict | None = None):
        self.message = message
        self.code = code
        self.context = context or {}
        super().__init__(message)

class ParseError(KreuzbergError):
    """Parse error with line/column info"""
    def __init__(self, message: str, line: int, column: int):
        self.line = line
        self.column = column
        super().__init__(f"{message} at line {line}:{column}", code=1001)

# Usage
try:
    result = kreuzberg.parse_document(data)
except ParseError as e:
    print(f"Error at {e.line}:{e.column}: {e.message}")
    # Log context for debugging
    logger.error("Parse failed", extra=e.context)
```

### JavaScript/TypeScript Promises & Errors

- **Promise rejection**: Use typed error classes inheriting from Error
- **Async/await**: Always handle promise rejections
- **Error details**: Include code, message, context in error object
- **Type safety**: Use discriminated unions or error types for catching specific errors

```typescript
// TypeScript error handling
class KreuzbergError extends Error {
    constructor(
        message: string,
        public code: number,
        public context?: Record<string, unknown>
    ) {
        super(message);
        this.name = "KreuzbergError";
    }
}

class ParseError extends KreuzbergError {
    constructor(message: string, public line: number, public column: number) {
        super(message, 1001, { line, column });
        this.name = "ParseError";
    }
}

// Promise rejection
async function parseDocument(data: Uint8Array): Promise<Document> {
    return new Promise((resolve, reject) => {
        kreuzberg.parseDocument(data, (err: Error | null, doc?: Document) => {
            if (err) {
                if (err instanceof ParseError) {
                    reject(new ParseError(err.message, err.line, err.column));
                } else {
                    reject(new KreuzbergError(err.message, 500));
                }
            } else {
                resolve(doc!);
            }
        });
    });
}

// Async/await with error handling
try {
    const doc = await parseDocument(data);
} catch (e) {
    if (e instanceof ParseError) {
        console.error(`Parse error at ${e.line}:${e.column}: ${e.message}`);
    } else if (e instanceof KreuzbergError) {
        console.error(`Kreuzberg error [${e.code}]: ${e.message}`);
    } else {
        console.error("Unknown error:", e);
    }
}
```

### Ruby Exceptions

- **Exception hierarchy**: Inherit from StandardError or appropriate ancestor
- **Raise with context**: Include error code and details
- **Rescue clauses**: Use type-specific rescue for clarity
- **Ensure blocks**: Cleanup always happens

```ruby
# Ruby error handling
module Kreuzberg
  class Error < StandardError
    attr_reader :code, :context

    def initialize(message, code: nil, context: {})
      @code = code
      @context = context
      super(message)
    end
  end

  class ParseError < Error
    attr_reader :line, :column

    def initialize(message, line:, column:)
      @line = line
      @column = column
      super("#{message} at #{line}:#{column}", code: 1001, context: { line: line, column: column })
    end
  end
end

# Usage
begin
  doc = Kreuzberg.parse_document(data)
rescue Kreuzberg::ParseError => e
  puts "Parse error at #{e.line}:#{e.column}: #{e.message}"
  # Handle parse error specifically
rescue Kreuzberg::Error => e
  puts "Kreuzberg error [#{e.code}]: #{e.message}"
  # Handle other Kreuzberg errors
rescue StandardError => e
  puts "Unexpected error: #{e.message}"
  raise  # Re-raise if unexpected
ensure
  # Cleanup always happens
end
```

### Go Errors

- **Error wrapping**: Use `fmt.Errorf("%w", err)` to preserve error chain
- **Error types**: Define error types as values (e.g., `var ErrParse = errors.New("parse error")`)
- **Sentinel errors**: Use for comparison; never use string comparison
- **Error interface**: Implement when adding context

```go
// Go error handling
package kreuzberg

import (
    "errors"
    "fmt"
)

var (
    ErrParse = errors.New("parse error")
    ErrInvalidInput = errors.New("invalid input")
)

type ParseError struct {
    Err    error
    Line   int
    Column int
}

func (e *ParseError) Error() string {
    return fmt.Sprintf("parse error at %d:%d: %v", e.Line, e.Column, e.Err)
}

func (e *ParseError) Unwrap() error {
    return e.Err
}

// Parse function
func ParseDocument(data []byte) (*Document, error) {
    // ... parsing logic
    if err := validateData(data); err != nil {
        return nil, fmt.Errorf("%w", err)  // Preserve chain
    }

    doc, err := parseBytes(data)
    if err != nil {
        return nil, &ParseError{
            Err: err,
            Line: getLine(),
            Column: getColumn(),
        }
    }
    return doc, nil
}

// Caller
doc, err := kreuzberg.ParseDocument(data)
if err != nil {
    if errors.Is(err, kreuzberg.ErrParse) {
        fmt.Printf("Got parse error\n")
    }

    var parseErr *kreuzberg.ParseError
    if errors.As(err, &parseErr) {
        fmt.Printf("Parse error at %d:%d\n", parseErr.Line, parseErr.Column)
    }

    return err
}
```

### Java Errors

- **Exception hierarchy**: Throw checked exceptions for recoverable errors; use unchecked for programming errors
- **Error context**: Use exception fields for details
- **Cause chain**: Use `initCause()` to preserve error chain
- **Try-with-resources**: Automatic cleanup

```java
// Java error handling
public class KreuzbergException extends Exception {
    private final int code;
    private final Map<String, Object> context;

    public KreuzbergException(String message, int code) {
        super(message);
        this.code = code;
        this.context = new HashMap<>();
    }

    public KreuzbergException(String message, Throwable cause, int code) {
        super(message, cause);
        this.code = code;
        this.context = new HashMap<>();
    }

    public int getCode() { return code; }
    public Map<String, Object> getContext() { return context; }
}

public class ParseException extends KreuzbergException {
    private final int line;
    private final int column;

    public ParseException(String message, int line, int column) {
        super(String.format("%s at %d:%d", message, line, column), 1001);
        this.line = line;
        this.column = column;
        getContext().put("line", line);
        getContext().put("column", column);
    }

    public int getLine() { return line; }
    public int getColumn() { return column; }
}

// Usage
try {
    Document doc = Kreuzberg.parseDocument(data);
} catch (ParseException e) {
    System.err.printf("Parse error at %d:%d: %s%n", e.getLine(), e.getColumn(), e.getMessage());
    // Handle parse error
} catch (KreuzbergException e) {
    System.err.printf("Kreuzberg error [%d]: %s%n", e.getCode(), e.getMessage());
    // Handle other errors
} catch (Exception e) {
    System.err.println("Unexpected error: " + e.getMessage());
    throw e;
}
```

## Error Context Preservation

- **Message**: Human-readable description of what failed and why
- **Error code**: Numeric identifier for programmatic handling (1000+ for business logic)
- **Source location**: File, line, column (for parsers) or function name
- **Context data**: Relevant variables, input preview, suggestions
- **Cause chain**: Link errors together (Rust: anyhow, Python: `from`, Go: `Errorf "%w"`, Java: `initCause`)

```rust
// Rust error with full context
use anyhow::{Context, Result, anyhow};

pub fn parse_document(data: &[u8]) -> Result<Document> {
    let content = String::from_utf8(data)
        .context("Input must be valid UTF-8")?;

    let doc = parse_impl(&content)
        .with_context(|| format!("Failed to parse {} bytes", content.len()))?;

    Ok(doc)
}

// Error includes: message, cause, context
// Result propagates with full chain:
// Error: Failed to parse 1024 bytes
//   Caused by:
//       0: Expected 'tag' at line 5, column 10
//       1: Input must be valid UTF-8
```

## Anti-Patterns

- **Silent failures**: Ignoring errors without logging or re-raising (requires ~keep comment for intentional suppression)
- **Losing context**: Stripping error message when converting between languages
- **String-based error codes**: Use numeric codes for programmatic handling
- **Over-wrapping errors**: Each layer adds wrapping; limit to 2-3 levels maximum
- **Catching all errors with generic handler**: Different error types need different handling
- **Async errors without context**: Promise rejections lose stack trace; add logging
- **No error recovery options**: Errors should suggest next steps or alternatives
- **Exposing internal types at boundary**: Return only language-native types from FFI

## FFI Boundary Checklist

- [ ] All Rust errors converted to host exceptions before return
- [ ] Error messages include context (line/column, input snippet, suggestion)
- [ ] Error codes are numeric (1000+) for programmatic handling
- [ ] Cause chain preserved (anyhow, Python from, Go %w, Java initCause)
- [ ] Tests verify all error paths throw correct exception types
- [ ] Performance: error construction doesn't allocate excessively
- [ ] Documentation explains each error code and recovery options
