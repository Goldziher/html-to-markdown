#!/usr/bin/env bash
set -euo pipefail

if command -v rustup >/dev/null 2>&1; then
	rustup_rustc="$(rustup which rustc 2>/dev/null || true)"
	if [[ -n "$rustup_rustc" && -x "$rustup_rustc" ]]; then
		exec "$rustup_rustc" "$@"
	fi
fi

if [[ -x "$HOME/.cargo/bin/rustc" ]]; then
	exec "$HOME/.cargo/bin/rustc" "$@"
fi

if command -v rustc >/dev/null 2>&1; then
	exec "$(command -v rustc)" "$@"
fi

printf 'Error: rustc not found. Please install Rust via rustup.\n' >&2
exit 1
