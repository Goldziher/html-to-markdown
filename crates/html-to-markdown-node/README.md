# @html-to-markdown/node

Native Node.js bindings for html-to-markdown using NAPI-RS.

This package provides high-performance HTML to Markdown conversion using native Rust code compiled to platform-specific binaries.

## Performance

~691,000 operations per second - approximately **2.5x faster** than the WebAssembly version.

## Installation

```bash
npm install @html-to-markdown/node
```

## Usage

```javascript
const { convert } = require('@html-to-markdown/node');

const html = '<h1>Hello World</h1>';
const markdown = convert(html);
console.log(markdown); // # Hello World

// With options
const markdown = convert(html, {
  headingStyle: 'Atx',
  wrap: true,
  wrapWidth: 80
});
```

## TypeScript

Full TypeScript definitions included:

```typescript
import { convert, type ConversionOptions } from '@html-to-markdown/node';

const options: ConversionOptions = {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
};

const markdown = convert('<h1>Hello</h1>', options);
```

## Supported Platforms

Pre-built binaries are provided for:

- macOS (x64, ARM64)
- Linux (x64 gnu/musl, ARM64 gnu/musl, ARMv7)
- Windows (x64, ARM64)

## Recommendation

For most users, we recommend installing the `html-to-markdown` package instead, which automatically selects between native and WebAssembly backends.

## License

MIT
