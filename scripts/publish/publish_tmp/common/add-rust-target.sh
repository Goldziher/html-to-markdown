#!/usr/bin/env bash
set -euo pipefail

target="${RUST_TARGET:?RUST_TARGET is required}"
rustup target add "${target}"
