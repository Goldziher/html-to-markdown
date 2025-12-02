#!/usr/bin/env bash
set -euo pipefail

pushd packages/python >/dev/null
maturin sdist --out dist/
popd >/dev/null
