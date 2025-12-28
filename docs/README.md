# GitHub Pages Demo

This directory contains the live demo of the HTML to Markdown converter, powered by WebAssembly.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown)

## 📚 Documentation

- [Language bindings overview](./bindings.md) — install & usage notes for Rust, Node.js, WebAssembly, Python, Ruby, PHP, Elixir, Go, Java, .NET, and the CLI.
- [PHP extension details](./php.md) — packaging, PIE distribution, and usage examples.

## 🌐 Live Demo

Visit the live demo at: **<https://kreuzberg-dev.github.io/html-to-markdown/>**

## 🚀 Running Locally

To test the demo locally:

```bash
# Option 1: Using task
task serve:demo

# Option 2: Using Python
cd docs
python3 -m http.server 8000

# Option 3: Using Node.js
npx http-server docs -p 8000
```

Then open <http://localhost:8000> in your browser.

## 🔧 Building the WASM Files

When you update the Rust code and need to rebuild:

```bash
# Option 1: Using go-task (recommended)
go-task build:demo

# Option 2: Using the script
./scripts/build-demo.sh

# Option 3: Manual
cd crates/html-to-markdown-wasm
wasm-pack build --target web --out-dir dist-web
cd ../..
cp crates/html-to-markdown-wasm/dist-web/html_to_markdown_wasm.js docs/
cp crates/html-to-markdown-wasm/dist-web/html_to_markdown_wasm_bg.wasm docs/
```

Then commit and push to deploy:

```bash
git add docs/
git commit -m "Update demo"
git push
```

## 📝 Notes

- The WASM binary is ~2.6MB (optimized with `wasm-opt`)
- First load may take a moment to download and initialize WASM
- All conversion happens client-side - no data is sent to any server
- Must be served over HTTP/HTTPS (not `file://`) due to WASM/CORS requirements
