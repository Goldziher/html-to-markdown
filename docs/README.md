# GitHub Pages Demo

This directory contains the live demo of the HTML to Markdown converter, powered by WebAssembly.

## 🌐 Live Demo

Visit the live demo at: **https://goldziher.github.io/html-to-markdown/**

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

Then open http://localhost:8000 in your browser.

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
