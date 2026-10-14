#!/usr/bin/env bash
set -euo pipefail

REPO="xberg-io/html-to-markdown"
NPM_PKG="@xberg-io/html-to-markdown-cli"
PYPI_PKG="html-to-markdown-cli"

log() { printf 'html-to-markdown-launch: %s\n' "$*" >&2; }
die() {
  log "error: $*"
  exit 1
}

LAUNCHER="${HTML_TO_MARKDOWN_LAUNCHER:-auto}"
case "$LAUNCHER" in
auto | npx | uvx | brew | download) ;;
*) die "invalid HTML_TO_MARKDOWN_LAUNCHER='$LAUNCHER' (expected auto|npx|uvx|brew|download)" ;;
esac

want() { [ "$LAUNCHER" = "auto" ] || [ "$LAUNCHER" = "$1" ]; }

PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-}"
if [ -z "$PLUGIN_ROOT" ]; then
  PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

BINARY_NAME="html-to-markdown"
case "$(uname -s)" in
MINGW* | MSYS* | CYGWIN* | Windows_NT) BINARY_NAME="html-to-markdown.exe" ;;
esac
BIN_DIR="$PLUGIN_ROOT/bin"
BIN="$BIN_DIR/$BINARY_NAME"

have() { command -v "$1" >/dev/null 2>&1; }

runnable() { [ -x "$1" ] && "$1" --version >/dev/null 2>&1; }

if runnable "$BIN"; then
  log "using cached html-to-markdown at $BIN"
  exec "$BIN" "$@"
fi
if have "$BINARY_NAME"; then
  PATH_BIN="$(command -v "$BINARY_NAME")"
  if runnable "$PATH_BIN"; then
    log "using html-to-markdown on PATH ($PATH_BIN)"
    exec "$PATH_BIN" "$@"
  fi
fi

if want npx && have npx; then
  log "resolving npx $NPM_PKG@latest ..."
  scratch="$(mktemp -d)"
  # Resolve in an empty project, then execute the selected binary in the caller's cwd. ~keep
  if npm_bin="$(cd "$scratch" && npx -y --package "$NPM_PKG@latest" --call "command -v ${BINARY_NAME%.exe}" 2>/dev/null)" && runnable "$npm_bin"; then
    rm -rf "$scratch"
    log "launching npm package binary $npm_bin"
    exec "$npm_bin" "$@"
  fi
  rm -rf "$scratch"
  log "npx $NPM_PKG not available (no CLI bin yet); falling through"
fi

if want uvx && have uvx; then
  log "probing uvx --from $PYPI_PKG $BINARY_NAME ..."
  if uvx --from "$PYPI_PKG" "$BINARY_NAME" --version >/dev/null 2>&1; then
    log "launching via uvx --from $PYPI_PKG $BINARY_NAME"
    exec uvx --from "$PYPI_PKG" "$BINARY_NAME" "$@"
  fi
  log "uvx $PYPI_PKG not available (no CLI entry point yet); falling through"
fi

if want brew && have brew; then
  log "installing via 'brew install xberg-io/tap/html-to-markdown' ..."
  if brew install xberg-io/tap/html-to-markdown >&2; then
    if have "$BINARY_NAME"; then
      BREW_BIN="$(command -v "$BINARY_NAME")"
      runnable "$BREW_BIN" && exec "$BREW_BIN" "$@"
    fi
    log "brew install reported success but html-to-markdown is not on PATH; falling through"
  else
    log "brew install failed; falling through"
  fi
fi

try_download() {
  local arch triple ext base_url asset asset_url tmp ex src_dir
  local sums_asset sums_url sums_fetched expected_sum actual_sum
  arch="$(uname -m)"
  case "$(uname -s)" in
  Darwin)
    case "$arch" in
    arm64 | aarch64) triple="aarch64-apple-darwin" ;;
    *)
      log "no prebuilt macOS archive for $arch (only Apple Silicon is published); falling through"
      return 1
      ;;
    esac
    ;;
  Linux)
    case "$arch" in
    aarch64 | arm64) triple="aarch64-unknown-linux-gnu" ;;
    x86_64) triple="x86_64-unknown-linux-gnu" ;;
    *)
      log "no prebuilt Linux archive for $arch; falling through"
      return 1
      ;;
    esac
    ;;
  MINGW* | MSYS* | CYGWIN* | Windows_NT) triple="x86_64-pc-windows-msvc" ;;
  *)
    log "no prebuilt archive for $(uname -s)/$arch; falling through"
    return 1
    ;;
  esac
  case "$triple" in
  *windows*) ext="zip" ;;
  *) ext="tar.gz" ;;
  esac

  base_url="https://github.com/${REPO}/releases/latest/download"
  asset="cli-${triple}.${ext}"
  asset_url="${base_url}/${asset}"
  sums_asset="cli-SHA256SUMS"
  sums_url="${base_url}/${sums_asset}"

  if have curl; then
    fetch() { curl -fsSL --retry 3 -o "$2" "$1"; }
  elif have wget; then
    fetch() { wget -q -O "$2" "$1"; }
  else
    log "no curl or wget available for download; falling through"
    return 1
  fi

  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064  # expand $tmp now so the trap removes this exact dir
  trap "rm -rf '$tmp'" RETURN

  log "downloading $asset from latest release ..."
  if ! fetch "$asset_url" "$tmp/$asset"; then
    log "download failed or asset not found: $asset_url; falling through"
    return 1
  fi

  # Right after a release the checksums asset can briefly 404 while GitHub
  # finishes propagating release assets; retry a few times before giving up.
  sums_fetched=0
  for _attempt in 1 2 3 4 5; do
    if fetch "$sums_url" "$tmp/$sums_asset" 2>/dev/null; then
      sums_fetched=1
      break
    fi
    sleep 2
  done
  if [ "$sums_fetched" -ne 1 ]; then
    log "could not download $sums_asset after retries; refusing to execute an unverified binary; falling through"
    return 1
  fi

  expected_sum="$(awk -v asset="$asset" '$2 == asset { print $1 }' "$tmp/$sums_asset")"
  if [ -z "$expected_sum" ]; then
    log "no checksum entry for $asset in $sums_asset; refusing to execute an unverified binary; falling through"
    return 1
  fi

  if have sha256sum; then
    actual_sum="$(sha256sum "$tmp/$asset" | awk '{print $1}')"
  elif have shasum; then
    actual_sum="$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')"
  else
    log "no sha256sum or shasum available to verify $asset; refusing to execute an unverified binary; falling through"
    return 1
  fi

  if [ "$actual_sum" != "$expected_sum" ]; then
    die "checksum mismatch for $asset (expected $expected_sum, got $actual_sum) — refusing to execute"
  fi
  log "checksum verified for $asset"

  ex="$tmp/extracted"
  mkdir -p "$ex"
  case "$ext" in
  tar.gz) tar -xzf "$tmp/$asset" -C "$ex" || {
    log "extraction failed; falling through"
    return 1
  } ;;
  zip)
    if have unzip; then
      unzip -qo "$tmp/$asset" -d "$ex" || {
        log "extraction failed; falling through"
        return 1
      }
    else
      log "need unzip to extract $asset; falling through"
      return 1
    fi
    ;;
  esac

  src_dir="$ex/cli-${triple}"
  [ -d "$src_dir" ] || src_dir="$ex"
  if [ ! -f "$src_dir/$BINARY_NAME" ]; then
    log "binary $BINARY_NAME not found inside $asset; falling through"
    return 1
  fi

  rm -rf "$BIN_DIR"
  mkdir -p "$BIN_DIR"
  mv "$src_dir"/* "$BIN_DIR"/
  chmod +x "$BIN"
  log "installed html-to-markdown to $BIN"
  return 0
}

if want download; then
  if try_download; then
    exec "$BIN" "$@"
  fi
fi

die "could not locate or install html-to-markdown. Install it manually with one of:
  brew install xberg-io/tap/html-to-markdown
  cargo install html-to-markdown-cli
  or download a prebuilt archive from https://github.com/${REPO}/releases/latest
then ensure 'html-to-markdown' is on PATH and retry."
