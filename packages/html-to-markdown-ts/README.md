# html-to-markdown

High-performance HTML to Markdown converter for Node.js and browsers. Written in Rust with TypeScript bindings.

## Features

- 🚀 **Blazing Fast** - 16-19x faster than pure JavaScript implementations
- 🎯 **Smart Backend** - Automatically uses native bindings when available, falls back to WebAssembly
- 💪 **Full TypeScript** - Complete type definitions included
- 🌐 **Universal** - Works in Node.js and browsers
- 🦀 **Rust Core** - Memory-safe and highly optimized

## Installation

```bash
npm install html-to-markdown
# or
yarn add html-to-markdown
# or
pnpm add html-to-markdown
```

## Quick Start

```typescript
import { convert } from 'html-to-markdown';

// Simple conversion
const markdown = await convert('<h1>Hello World</h1>');
console.log(markdown); // # Hello World

// With options
const markdown = await convert('<h1>Hello</h1>', {
  headingStyle: 'atx',
  wrap: true,
  wrapWidth: 80,
});
```

### Synchronous Usage

```typescript
import { initializeSync, convertSync } from 'html-to-markdown';

// Initialize once
initializeSync();

// Then use synchronously
const markdown = convertSync('<h1>Hello World</h1>');
```

## API

### `convert(html: string, options?: ConversionOptions): Promise<string>`

Convert HTML to Markdown asynchronously.

### `convertSync(html: string, options?: ConversionOptions): string`

Convert HTML to Markdown synchronously. Requires calling `initializeSync()` first.

### `initializeSync(): void`

Initialize the converter synchronously. Required before using `convertSync()`.

### `getBackend(): Backend | null`

Get the current backend type (`'native'` or `'wasm'`).

### `createConverter(): Converter`

Create a new converter instance (useful for managing multiple converters).

## Options

```typescript
interface ConversionOptions {
  // Heading style (default: Atx)
  headingStyle?: 'Underlined' | 'Atx' | 'AtxClosed';

  // List indentation
  listIndentType?: 'Spaces' | 'Tabs';
  listIndentWidth?: number; // default: 2

  // Bullets for unordered lists (default: "-")
  bullets?: string;

  // Code blocks
  codeBlockStyle?: 'Indented' | 'Backticks' | 'Tildes';
  codeLanguage?: string;

  // Emphasis
  strongEmSymbol?: string; // '*' or '_'

  // Escaping
  escapeAsterisks?: boolean;
  escapeUnderscores?: boolean;
  escapeMisc?: boolean;
  escapeAscii?: boolean;

  // Whitespace
  whitespaceMode?: 'Normalized' | 'Strict';
  stripNewlines?: boolean;

  // Text wrapping
  wrap?: boolean;
  wrapWidth?: number; // default: 80

  // Other options
  autolinks?: boolean;
  defaultTitle?: boolean;
  brInTables?: boolean;
  hocrSpatialTables?: boolean;

  // Preprocessing
  preprocessing?: {
    enabled?: boolean;
    preset?: 'Minimal' | 'Standard' | 'Aggressive';
    removeNavigation?: boolean;
    removeForms?: boolean;
  };

  // Debug mode
  debug?: boolean;

  // Strip specific HTML tags
  stripTags?: string[];
}
```

## Performance

Performance comparison (operations per second):

- **Native (NAPI-RS)**: ~691,000 ops/sec ⚡
- **WebAssembly**: ~229,000 ops/sec
- **Pure JavaScript**: ~276,000 ops/sec

The native backend is ~2.5x faster than WASM and automatically used when available.

## Backend Selection

The package automatically selects the best available backend:

1. **Native bindings** (NAPI-RS) - Fastest, used when available on your platform
1. **WebAssembly** - Universal fallback, works everywhere

You can check which backend is being used:

```typescript
import { convert, getBackend } from 'html-to-markdown';

await convert('<h1>Hello</h1>');
console.log('Using:', getBackend()); // 'native' or 'wasm'
```

## Browser Usage

For browser-only usage, install the WASM package directly:

```bash
npm install @html-to-markdown/wasm
```

```typescript
import { convert } from '@html-to-markdown/wasm';

const markdown = convert('<h1>Hello</h1>');
```

## Advanced Usage

### Custom Converter Instance

```typescript
import { createConverter } from 'html-to-markdown';

const converter = createConverter();

// Convert multiple documents
const md1 = await converter.convert('<h1>Doc 1</h1>');
const md2 = await converter.convert('<h1>Doc 2</h1>');

// Check backend
console.log(converter.getBackend());
```

### Error Handling

```typescript
import { convert, ConversionError } from 'html-to-markdown';

try {
  const markdown = await convert('<h1>Hello</h1>');
} catch (error) {
  if (error instanceof ConversionError) {
    console.error('Conversion failed:', error.message);
  }
}
```

## TypeScript

Full TypeScript support is included:

```typescript
import type { ConversionOptions, HeadingStyle } from 'html-to-markdown';

const options: ConversionOptions = {
  headingStyle: HeadingStyle.Atx,
  wrap: true,
};
```

## License

MIT

## Links

- [GitHub](https://github.com/Goldziher/html-to-markdown)
- [Documentation](https://github.com/Goldziher/html-to-markdown#readme)
- [Python Package](https://pypi.org/project/html-to-markdown/)
- [Rust Crate](https://crates.io/crates/html-to-markdown-rs)
