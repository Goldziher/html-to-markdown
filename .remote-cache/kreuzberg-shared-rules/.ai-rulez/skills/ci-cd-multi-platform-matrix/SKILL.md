______________________________________________________________________

## priority: critical

# CI/CD Multi-Platform Matrix

## GitHub Actions Matrix Strategy

Use matrix strategy to test across multiple OS, architectures, and language versions with minimal duplication:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  test-matrix:
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          # Linux: x86_64 and ARM64
          - { os: ubuntu-latest, arch: x86_64, target: x86_64-unknown-linux-gnu }
          - { os: ubuntu-latest, arch: arm64, target: aarch64-unknown-linux-gnu }

          # macOS: Intel and Apple Silicon
          - { os: macos-12, arch: x86_64, target: x86_64-apple-darwin }
          - { os: macos-14, arch: arm64, target: aarch64-apple-darwin }

          # Windows: x86_64 only
          - { os: windows-latest, arch: x86_64, target: x86_64-pc-windows-msvc }

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Setup cross compilation
        if: matrix.arch == 'arm64' && matrix.os == 'ubuntu-latest'
        run: cargo install cross

      - name: Build
        run: |
          if [ "${{ matrix.arch }}" = "arm64" ] && [ "${{ matrix.os }}" = "ubuntu-latest" ]; then
            cross build --target ${{ matrix.target }} --release
          else
            cargo build --target ${{ matrix.target }} --release
          fi
        shell: bash

      - name: Test
        run: |
          if [ "${{ matrix.arch }}" = "arm64" ] && [ "${{ matrix.os }}" = "ubuntu-latest" ]; then
            cross test --target ${{ matrix.target }}
          else
            cargo test --target ${{ matrix.target }}
          fi
        shell: bash

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: binaries-${{ matrix.os }}-${{ matrix.arch }}
          path: target/${{ matrix.target }}/release/myapp*
```

## Language Version Matrices

Test against multiple language versions simultaneously:

```yaml
name: Polyglot CI

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust-version: ['1.70', 'stable', 'nightly']
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust-version }}
      - run: cargo test --all-features

  test-python:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      - run: |
          pip install -e . -v
          pytest tests/ -v

  test-node:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: ['18.x', '20.x', '22.x']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
      - run: |
          npm ci
          npm run test

  test-ruby:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        ruby-version: ['3.0', '3.1', '3.2', '3.3']
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: ${{ matrix.ruby-version }}
          bundler-cache: true
      - run: bundle exec rspec

  test-java:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        java-version: ['11', '17', '21']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v3
        with:
          java-version: ${{ matrix.java-version }}
          distribution: 'temurin'
      - run: ./gradlew test

  test-go:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        go-version: ['1.20', '1.21', '1.22']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v4
        with:
          go-version: ${{ matrix.go-version }}
      - run: go test ./...
```

## Artifact Caching Strategy

Reuse built artifacts and dependencies across runs to speed up CI:

```yaml
name: Build with Cache

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Rust dependency caching
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: 'crates'
          cache-targets: true

      # Python pip caching
      - uses: actions/cache@v3
        with:
          path: ~/.cache/pip
          key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}
          restore-keys: |
            ${{ runner.os }}-pip-

      # Node.js caching
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: '**/package-lock.json'

      # Docker layer caching
      - uses: docker/setup-buildx-action@v2

      - name: Build with Docker cache
        uses: docker/build-push-action@v5
        with:
          context: .
          push: false
          cache-from: type=gha
          cache-to: type=gha,mode=max

      # Custom cargo build cache
      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-build-

      - run: cargo build --release
```

## Test Result Aggregation

Combine test results from multiple jobs into single report:

```yaml
name: Aggregate Test Results

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --no-fail-fast -- --test-threads=1 --nocapture
        continue-on-error: true
      - uses: actions/upload-artifact@v3
        with:
          name: test-results-rust
          path: target/test-results.xml

  test-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - run: pip install pytest pytest-xml pytest-cov
      - run: pytest --junitxml=test-results.xml --cov=. --cov-report=term
        continue-on-error: true
      - uses: actions/upload-artifact@v3
        with:
          name: test-results-python
          path: test-results.xml

  aggregate:
    runs-on: ubuntu-latest
    needs: [test-rust, test-python]
    if: always()
    steps:
      - uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Publish test results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: artifacts/**/test-results.xml
          check_name: 'Test Results'
          comment_mode: 'always'
```

## Split CI Workflows by Domain

Organize workflows by language/domain to parallelize and reduce noise:

**ci-rust.yaml** (Rust-specific testing):

```yaml
name: CI - Rust

on:
  push:
    paths:
      - 'crates/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - '.github/workflows/ci-rust.yaml'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all --all-targets --all-features
      - run: cargo test --all --all-features
      - run: cargo tarpaulin --out Xml --timeout 300 --fail-under 95
      - uses: codecov/codecov-action@v3

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: '1.70'  # MSRV
      - run: cargo test --all
```

**ci-python.yaml** (Python-specific testing):

```yaml
name: CI - Python

on:
  push:
    paths:
      - 'python/**'
      - 'bindings/python/**'
      - 'pyproject.toml'
      - '.github/workflows/ci-python.yaml'

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
          cache: 'pip'
      - run: pip install -e .[dev]
      - run: ruff check .
      - run: ruff format --check .
      - run: mypy bindings/python --strict
      - run: pytest tests/ -v --cov=bindings/python --cov-report=xml
      - uses: codecov/codecov-action@v3
        if: matrix.python-version == '3.11'
```

**ci-node.yaml** (Node.js/TypeScript-specific):

```yaml
name: CI - Node.js

on:
  push:
    paths:
      - 'bindings/node/**'
      - 'package.json'
      - 'pnpm-lock.yaml'
      - '.github/workflows/ci-node.yaml'

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: ['18.x', '20.x', '22.x']
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
          cache: 'pnpm'
      - run: pnpm install
      - run: pnpm lint
      - run: pnpm format:check
      - run: pnpm test:unit
      - run: pnpm build
      - uses: codecov/codecov-action@v3
        if: matrix.node-version == '20.x'
```

**ci-java.yaml** (Java-specific testing):

```yaml
name: CI - Java

on:
  push:
    paths:
      - 'bindings/java/**'
      - 'build.gradle'
      - '.github/workflows/ci-java.yaml'

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        java-version: ['11', '17', '21']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v3
        with:
          java-version: ${{ matrix.java-version }}
          distribution: 'temurin'
          cache: 'gradle'
      - run: ./gradlew check
      - run: ./gradlew jacocoTestReport
      - uses: codecov/codecov-action@v3
```

## Matrix Expansion with Environment

Avoid duplicating test definitions by expanding with environment matrices:

```yaml
name: Build & Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.config.os }}
    strategy:
      matrix:
        config:
          - { os: ubuntu-latest, rust: stable, coverage: true }
          - { os: ubuntu-latest, rust: nightly }
          - { os: macos-latest, rust: stable }
          - { os: windows-latest, rust: stable }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.config.rust }}

      - name: Run tests
        run: cargo test --all

      - name: Generate coverage
        if: matrix.config.coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        if: matrix.config.coverage
        uses: codecov/codecov-action@v3
```

## Conditional Step Execution

Control which steps run based on matrix context:

```yaml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            install-cmd: apt-get install -y
          - os: macos-latest
            install-cmd: brew install
          - os: windows-latest
            install-cmd: choco install

    steps:
      - name: Install dependencies (Linux)
        if: runner.os == 'Linux'
        run: sudo ${{ matrix.install-cmd }} libc6-dev

      - name: Install dependencies (macOS)
        if: runner.os == 'macOS'
        run: ${{ matrix.install-cmd }} openssl

      - name: Install dependencies (Windows)
        if: runner.os == 'Windows'
        run: ${{ matrix.install-cmd }} openssl
```

## Anti-Patterns

- **No fail-fast control**: Use `fail-fast: false` to run all matrix variants even if one fails

- **Duplicating CI logic**: Extract common steps to reusable workflows

- **Unbounded matrix expansion**: Avoid cartesian product of too many dimensions

  ```yaml
  # BAD: 3×4×3×2 = 72 jobs
  strategy:
    matrix:
      os: [ubuntu, macos, windows]
      rust: [1.70, stable, beta, nightly]
      python: [3.8, 3.9, 3.10, 3.11]
      node: [18, 20]

  # GOOD: Use include/exclude to select specific combinations
  strategy:
    matrix:
      include:
        - { os: ubuntu, rust: stable, python: '3.11', node: '20' }
        - { os: macos, rust: stable, python: '3.11', node: '20' }
        - { os: windows, rust: stable, python: '3.11', node: '20' }
  ```

- **No artifact caching**: Always cache dependencies and build artifacts

- **Mixed domains in single workflow**: Separate Rust, Python, Node.js, Java into independent workflows

- **Test failures not surfaced**: Always publish results to PR comments with action

- **No cross-platform testing**: Test on Linux, macOS, Windows (especially for FFI bindings)

- **Hardcoded versions**: Use matrix variables for all version-dependent steps

- **No architecture testing**: Test both x86_64 and ARM64 (cross-compile with `cross` crate)
