#!/usr/bin/env bash
set -e

echo "🔨 Building WASM package..."
cd crates/html-to-markdown-wasm
wasm-pack build --target web --out-dir dist-web

echo "📦 Copying files to docs/..."
cd ../..
cp crates/html-to-markdown-wasm/dist-web/html_to_markdown_wasm.js docs/
cp crates/html-to-markdown-wasm/dist-web/html_to_markdown_wasm_bg.wasm docs/

echo "✅ Demo updated successfully!"
echo ""
echo "To test locally, run:"
echo "  cd docs && python3 -m http.server 8000"
echo ""
echo "Then open http://localhost:8000 in your browser"
