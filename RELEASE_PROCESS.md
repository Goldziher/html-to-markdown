# Release Process

This document describes how to release new versions of html-to-markdown.

## Overview

html-to-markdown has three main distribution channels:

1. **Cargo (crates.io)**: Rust library and CLI
1. **Homebrew**: macOS/Linux CLI binary
1. **PyPI**: Python package with Rust bindings

## Release Workflows

### 1. GitHub Release with Binaries (`release.yml`)

**Trigger**: Push a version tag (e.g., `v2.0.1`)

**What it does**:

- Creates a GitHub release
- Builds CLI binaries for:
    - Linux (x86_64, aarch64)
    - macOS (x86_64 Intel, aarch64 Apple Silicon)
    - Windows (x86_64)
- Uploads binaries to the release

**Usage**:

```bash
git tag v2.0.1
git push origin v2.0.1
```

### 2. Cargo Publication (`publish-cargo.yml`)

**Trigger**:

- Automatically when a release is published
- Manually via workflow dispatch

**What it does**:

1. Publishes `html-to-markdown` crate (core library)
1. Waits 30 seconds for crates.io to index
1. Publishes `html-to-markdown-cli` crate (CLI)

**Manual trigger**:

```bash
gh workflow run publish-cargo.yml -f tag=v2.0.1
```

**Required Secret**: `CARGO_TOKEN`

- Get from <https://crates.io/settings/tokens>
- Add to repository secrets as `CARGO_TOKEN`

### 3. Homebrew Release (`release-homebrew.yml`)

**Trigger**: Push a version tag (e.g., `v2.0.1`)

**What it does**:

1. Builds binaries for all platforms
1. Creates GitHub release with binaries
1. Updates the Homebrew formula in `Goldziher/homebrew-tap`

**Required Secret**: `HOMEBREW_TOKEN`

- Personal access token with `repo` scope
- Add to repository secrets as `HOMEBREW_TOKEN`

**Homebrew Tap**: <https://github.com/Goldziher/homebrew-tap>

- Formula is at `homebrew-tap/Formula/html-to-markdown.rb` (git submodule)
- Automatically updated by the workflow

## Pre-release Checklist

1. **Update version** in `Cargo.toml`:

    ```toml
    [workspace.package]
    version = "2.0.1"
    ```

1. **Update CHANGELOG.md** with changes

1. **Run tests**:

    ```bash
    task test
    ```

1. **Build locally** to ensure it works:

    ```bash
    task build:cli
    ./target/release/html-to-markdown --version
    ```

1. **Commit changes**:

    ```bash
    git add Cargo.toml CHANGELOG.md
    git commit -m "chore: bump version to 2.0.1"
    git push
    ```

## Release Steps

1. **Create and push tag**:

    ```bash
    git tag -a v2.0.1 -m "Release v2.0.1"
    git push origin v2.0.1
    ```

1. **Monitor workflows**:

    - Check GitHub Actions: <https://github.com/Goldziher/html-to-markdown/actions>
    - `release.yml` or `release-homebrew.yml` creates the release
    - `publish-cargo.yml` publishes to crates.io

1. **Verify publication**:

    - Cargo: <https://crates.io/crates/html-to-markdown>
    - CLI: <https://crates.io/crates/html-to-markdown-cli>
    - Homebrew: <https://github.com/Goldziher/homebrew-tap>
    - GitHub: <https://github.com/Goldziher/html-to-markdown/releases>

## Installation Commands

After release, users can install via:

### Cargo

```bash
cargo install html-to-markdown-cli
```

### Homebrew

```bash
brew tap goldziher/tap
brew install html-to-markdown
```

### PyPI (Python bindings)

```bash
pip install html-to-markdown
```

### Direct Download

Download from: <https://github.com/Goldziher/html-to-markdown/releases>

## Troubleshooting

### Cargo publish fails with "crate already exists"

- You cannot republish the same version
- Bump the version and create a new tag

### Homebrew formula not updated

- Check `HOMEBREW_TOKEN` secret is set correctly
- Verify the token has `repo` scope
- Check workflow logs for errors

### Binary build fails

- Check that cross-compilation tools are installed
- Review the build logs in GitHub Actions
- Test locally with: `cargo build --release --target <target-triple>`

## Secrets Required

Add these secrets to your GitHub repository (Settings → Secrets → Actions):

1. **CARGO_TOKEN**: Token from <https://crates.io/settings/tokens>
1. **HOMEBREW_TOKEN**: Personal access token with `repo` scope

## First-Time Setup

### Create Homebrew Tap

If you don't have a homebrew tap yet:

```bash
# Create a new repository: homebrew-tap
gh repo create Goldziher/homebrew-tap --public

# Clone it
git clone https://github.com/Goldziher/homebrew-tap
cd homebrew-tap

# Create Formula directory
mkdir -p Formula

# The workflow will create Formula/html-to-markdown.rb automatically
```

### Setup as Git Submodule (optional)

If you want to track the formula in this repo:

```bash
git submodule add https://github.com/Goldziher/homebrew-tap.git homebrew-tap
```
