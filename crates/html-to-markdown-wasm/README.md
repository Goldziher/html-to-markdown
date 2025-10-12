# @html-to-markdown/wasm

WebAssembly bindings for html-to-markdown using wasm-bindgen.

Universal HTML to Markdown converter that runs anywhere: Node.js, Deno, browsers, and edge runtimes.

## Performance

~229,000 operations per second - fast and portable.

## Installation

```bash
npm install @html-to-markdown/wasm
```

## Usage

### Bundler (Webpack, Vite, etc.)

```javascript
import { convert } from '@html-to-markdown/wasm';

const html = '<h1>Hello World</h1>';
const markdown = convert(html);
console.log(markdown); // # Hello World
```

### Node.js

```javascript
const { convert } = require('@html-to-markdown/wasm/dist-node');

const markdown = convert('<h1>Hello</h1>');
```

### Browser (ESM)

```html
<script type="module">
  import init, { convert } from '@html-to-markdown/wasm/dist-web';

  await init(); // Initialize WASM
  const markdown = convert('<h1>Hello World</h1>');
</script>
```

## TypeScript

Full TypeScript support included:

```typescript
import { convert, type WasmConversionOptions } from '@html-to-markdown/wasm';

const markdown = convert('<h1>Hello</h1>', {
  headingStyle: { atx: null },
  wrap: true,
});
```

## Builds

Three build targets are provided:

- `dist/` - Bundler target (default)
- `dist-node/` - Node.js target
- `dist-web/` - Web/ESM target

## When to Use

- Browser/client-side conversion
- Edge runtimes (Cloudflare Workers, Deno Deploy)
- Platforms without native binary support
- Universal packages that need to run everywhere

For Node.js-only applications with maximum performance, consider `@html-to-markdown/node` instead.

## Recommendation

For most users, we recommend the `html-to-markdown` package, which automatically uses native bindings when available and falls back to WASM.

## License

MIT
