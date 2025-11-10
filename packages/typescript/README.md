# html-to-markdown (TypeScript)

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

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
