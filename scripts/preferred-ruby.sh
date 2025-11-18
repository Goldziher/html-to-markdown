#!/usr/bin/env bash
set -euo pipefail
# Allow manual override
if [[ -n "${HTML_TO_MARKDOWN_RUBY:-}" && -x "${HTML_TO_MARKDOWN_RUBY}" ]]; then
  exec "${HTML_TO_MARKDOWN_RUBY}" "$@"
fi
# Prefer Homebrew ruby if available (macOS)
if command -v brew >/dev/null 2>&1; then
  brew_ruby_dir=$(brew --prefix ruby 2>/dev/null || true)
  if [[ -n "$brew_ruby_dir" && -x "$brew_ruby_dir/bin/ruby" ]]; then
    exec "$brew_ruby_dir/bin/ruby" "$@"
  fi
fi
# Prefer rbenv if available
if command -v rbenv >/dev/null 2>&1; then
  rb_path="$(rbenv which ruby 2>/dev/null || true)"
  if [[ -n "$rb_path" && -x "$rb_path" ]]; then
    exec "$rb_path" "$@"
  fi
fi
# Fallback to whatever ruby is on PATH
if command -v ruby >/dev/null 2>&1; then
  exec "$(command -v ruby)" "$@"
fi
printf 'Error: Ruby interpreter not found. Please install Ruby 3.x\n' >&2
exit 1
