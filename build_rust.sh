#!/bin/bash
# Quick build script for Rust backend development

set -e

cd "$(dirname "$0")/crates/html-to-markdown-py"

echo "🔨 Building Rust backend..."
maturin build --release --quiet

echo "📦 Extracting .so file..."
cd ../../
unzip -o target/wheels/html_to_markdown-2.0.0-cp310-abi3-macosx_11_0_arm64.whl \
    html_to_markdown/_html_to_markdown.abi3.so -d /tmp >/dev/null

echo "📋 Copying to project..."
cp /tmp/html_to_markdown/_html_to_markdown.abi3.so html_to_markdown/

echo "✅ Build complete! Run tests with:"
echo "   .venv/bin/python -m pytest tests/elements_test.py tests/lists_test.py tests/tables_test.py --tb=no -q"
