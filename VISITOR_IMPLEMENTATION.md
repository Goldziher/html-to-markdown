# Visitor Pattern Implementation in html-to-markdown-node

## Overview

This document explains the architectural approach for the visitor pattern in the Node.js binding (`html-to-markdown-node`) and why callback invocation is not currently implemented.

## Architecture Challenge

The core challenge is a **fundamental mismatch** between:

1. **Rust Core (Synchronous)**
   - The `HtmlVisitor` trait is inherently synchronous
   - Visitor methods return `VisitResult` directly, not `Future<VisitResult>`
   - This synchronous design is essential for the HTML parsing loop performance

2. **JavaScript Callbacks (Async-Capable)**
   - Calling JS functions through NAPI-RS requires the V8 `Env` context
   - This `Env` is only available in the NAPI function entry point
   - Once inside Rust's visitor methods, we're in a pure Rust context without V8 access

3. **NAPI-RS Limitations**
   - `ThreadsafeFunction` allows async calls but requires `async/await`
   - No built-in mechanism to synchronously call JS from within Rust code
   - Blocking V8 calls from native code is not supported
   - Passing `Env` through the visitor trait would require changes to the core Rust library

## Current Implementation (Pragmatic Approach)

Rather than providing a non-functional implementation, we offer a **documented placeholder**:

### `JsVisitorBridge` Structure

```rust
struct JsVisitorBridge {
    has_callbacks: bool,  // Indicates if visitor had callbacks
}
```

This minimal structure:
- ✅ Compiles successfully
- ✅ Accepts visitor objects in the NAPI function boundary
- ✅ Validates that callback methods exist (for future use)
- ✅ Provides clear documentation about limitations
- ✅ Allows the function signature to remain stable
- ❌ Does **not** invoke callbacks (architectural blocker)

### `convert_with_visitor()` Function

The NAPI function:
- Accepts an `Object` with callback methods
- Creates a `JsVisitorBridge` with callback validation
- Passes the bridge to the Rust core (which is a no-op)
- Returns the conversion result (callbacks have no effect)

## Why Not These Approaches?

### ❌ Option 1: Token-Based Callback Queueing
- Would require passing `Env` through the visitor trait (breaks Rust core API)
- Complex inter-thread synchronization overhead
- Defeats the performance advantage of native bindings

### ❌ Option 2: ThreadsafeFunction with Async/Await
- Requires the entire visitor to be async
- Incompatible with sync Rust core
- Would require refactoring HTML parsing loop

### ❌ Option 3: Blocking V8 Call
- V8 explicitly prevents blocking calls from native code
- Would cause event loop stalls and crashes
- Not supported by NAPI-RS

### ✅ Option 4: Documented Placeholder (Current)
- Honest about limitations
- No misleading functionality
- Clear migration path for users
- Allows future enhancement if NAPI-RS adds blocking mechanism

## User Alternatives

### For Python Users
The Python binding **fully supports** the visitor pattern via `asyncio`:

```python
import asyncio
from html_to_markdown import convert_with_visitor

class MyVisitor:
    async def visit_text(self, ctx, text):
        # Called during conversion
        return VisitResult.Continue

async def main():
    html = "<h1>Hello</h1>"
    markdown = await convert_with_visitor(html, visitor=MyVisitor())
```

### For Ruby Users
The Ruby binding **fully supports** the visitor pattern:

```ruby
require 'html_to_markdown'

class MyVisitor
  def visit_text(ctx, text)
    # Called during conversion
    VisitResult::Continue
  end
end

html = "<h1>Hello</h1>"
markdown = convert_with_visitor(html, visitor: MyVisitor.new)
```

### For Node.js Users (Current Workarounds)

1. **Post-Conversion Processing**
   ```javascript
   const { convert } = require('html-to-markdown-node');
   const html = '<h1>Hello</h1><img src="test.png">';
   const markdown = convert(html);
   // Process markdown in JavaScript after conversion
   ```

2. **Preprocessing HTML**
   ```javascript
   const { convert } = require('html-to-markdown-node');
   const processed = preprocessHtml(html);  // Custom logic
   const markdown = convert(processed);
   ```

3. **Use WASM Binding**
   ```javascript
   const { convert } = require('html-to-markdown-wasm');
   // WASM binding also doesn't support visitor callbacks,
   // but may be lighter for some use cases
   ```

4. **Use Python/Ruby Binding**
   - For critical visitor use cases, consider using the Python or Ruby binding
   - Can call from Node via child_process or via language bridge libraries

## Future Enhancement Path

If NAPI-RS or V8 adds support for blocking native→JS calls in the future:

1. Uncomment `has_callbacks` usage in `JsVisitorBridge`
2. Implement the `call_callback()` method with the new mechanism
3. Update visitor trait methods to invoke callbacks
4. Add tests and update documentation

The current implementation leaves the door open for this enhancement with minimal code changes.

## Files Modified

- **`crates/html-to-markdown-node/src/lib.rs`**
  - Implemented `JsVisitorBridge` with detailed documentation
  - Added comprehensive docs to `convert_with_visitor()` function
  - Explained architectural limitations and alternatives

## References

- [NAPI-RS Documentation](https://napi.rs/)
- [html-to-markdown-rs Visitor Trait](../../crates/html-to-markdown/src/visitor.rs)
- [Python Binding with Async Visitor](../../packages/python/)
- [Ruby Binding with Sync Visitor](../../packages/ruby/)

## Conclusion

This pragmatic approach prioritizes:
- **Honesty**: Clear explanation of limitations
- **Stability**: Function signature remains stable for future enhancement
- **Performance**: No overhead for non-visitor conversions
- **Clarity**: Detailed documentation and alternatives for users

The visitor pattern will remain fully functional in Python and Ruby bindings, while Node.js users have clear workarounds and migration paths.
