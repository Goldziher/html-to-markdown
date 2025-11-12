# Examples & Smoke Tests

These sub-projects make it easy to reproduce installation issues for every
binding. Each folder contains a README with detailed instructions for testing
both the latest published artifact and a local build from this repository.

| Path | Runtime | Notes |
| --- | --- | --- |
| `examples/node-smoke` | Node.js / Bun | Installs `html-to-markdown-node` from npm or a local `pnpm --filter html-to-markdown-node run build`. |
| `examples/wasm-smoke` | Node.js (WASM) | Loads `html-to-markdown-wasm` via the `dist-node` entry and can swap in the locally-built bundle. |
| `examples/python-smoke` | Python | Uses a virtual environment plus either PyPI (`pip install -r requirements.txt`) or `pip install ../../packages/python`. |
| `examples/ruby-smoke` | Ruby | Bundler project that can point at RubyGems or the `packages/ruby` sources via `bundle config local.html-to-markdown`. |
| `examples/php-smoke` | PHP | Composer project that exercises `html_to_markdown` extension after installing via PIE or a local build. |
| `examples/rust-smoke` | Rust | Simple binary wired to crates.io by default with an opt-in local patch template. |

Legacy demo files such as `browser.html` and `node-native.js` remain available
for quick manual testing, but the new smoke tests should be used for release
validation.
