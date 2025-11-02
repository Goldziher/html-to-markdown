# html-to-markdown (TypeScript)

High-performance HTML to Markdown converter for Node.js and Bun. This package wraps the
native `html-to-markdown-node` bindings and adds a TypeScript-friendly API plus a
cross-platform CLI.

```bash
npm install html-to-markdown
```

## Usage

```ts
import { convert } from 'html-to-markdown';

const markdown = convert('<h1>Hello</h1>');
```

The package re-exports all conversion options exposed by the native bindings. See the
[core documentation](https://github.com/Goldziher/html-to-markdown) for complete
option descriptions.

### File helpers

```ts
import { convertFile } from 'html-to-markdown';

const markdown = await convertFile('input.html');
```

### CLI

```bash
npx html-to-markdown --input input.html --output output.md
```

Use `npx html-to-markdown --help` for full usage information.
