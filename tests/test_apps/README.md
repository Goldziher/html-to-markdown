# test_apps/ - End-to-End Testing Infrastructure

This directory contains **end-to-end (e2e) tests** for validating published packages from public registries (PyPI, npm, RubyGems, Maven Central, NuGet, Packagist, Hex.pm, pkg.go.dev).

## Purpose

Unlike the main test suites that validate local development builds, `test_apps/` validates that:

1. **Packages publish correctly** to public registries
2. **Installation works** for end users (`pip install`, `npm install`, etc.)
3. **APIs function correctly** in real-world usage
4. **Breaking changes** are caught before users encounter them
5. **Version synchronization** works across all language packages

## Two-Tier Testing Strategy

### Tier 1: Smoke Tests (Fast)

**Purpose**: Quick validation that packages are installable and functional.

**Characteristics**:
- **Fast**: <30 seconds per language, <5 minutes total
- **Minimal**: Basic import/require + simple conversion test
- **Pre-Release Gate**: Runs before publishing to registries
- **CI Integration**: `task e2e:smoke:all` in publish workflow

**Example** (`python/smoke_test.py`):
```python
def test_package_imports():
    """Verify package can be imported."""
    import html_to_markdown
    assert html_to_markdown is not None

def test_basic_conversion():
    """Verify basic HTML→Markdown conversion works."""
    from html_to_markdown import convert_html_to_markdown
    html = "<p>Hello World</p>"
    result = convert_html_to_markdown(html)
    assert "Hello World" in result
```

### Tier 2: Comprehensive Tests (Thorough)

**Purpose**: Exhaustive validation using shared fixtures across all languages.

**Characteristics**:
- **Thorough**: 1-3 minutes per language
- **Fixture-Driven**: Shared JSON fixtures ensure language parity
- **Post-Release Validation**: Runs after publishing (can run locally too)
- **CI Integration**: `task e2e:test:all` (optional, can be manual)

**Example** (`python/comprehensive_test.py`):
```python
import json
from pathlib import Path
import pytest
from html_to_markdown import convert_html_to_markdown

def load_fixtures(filename):
    """Load shared JSON fixtures."""
    fixture_path = Path(__file__).parent.parent / "fixtures" / filename
    with open(fixture_path) as f:
        return json.load(f)

@pytest.mark.parametrize(
    "test_case",
    load_fixtures("basic-html.json"),
    ids=lambda tc: tc["name"]
)
def test_basic_html_conversion(test_case):
    """Test basic HTML conversions against fixtures."""
    result = convert_html_to_markdown(
        test_case["html"],
        **test_case.get("options", {})
    )
    assert result.strip() == test_case["expectedMarkdown"].strip()
```

## Directory Structure

```
tests/test_apps/
├── fixtures/                          # Shared test fixtures (JSON)
│   ├── README.md                     # Fixture format documentation
│   ├── basic-html.json               # Basic HTML elements (10 cases)
│   ├── complex-html.json             # Complex structures (future)
│   ├── edge-cases.json               # Edge cases (future)
│   ├── metadata-extraction.json      # Metadata features (future)
│   └── real-world.json               # Real-world samples (future)
│
├── python/                           # Python test app (PyPI)
│   ├── .python-version               # Python version (3.10)
│   ├── pyproject.toml                # Depends on html-to-markdown from PyPI
│   ├── uv.lock                       # Locked dependencies
│   ├── smoke_test.py                 # Fast smoke tests
│   ├── comprehensive_test.py         # Fixture-driven tests
│   └── README.md                     # Python-specific notes
│
├── node/                             # Node.js test app (npm)
│   ├── .nvmrc                        # Node version (18+)
│   ├── package.json                  # Depends on html-to-markdown from npm
│   ├── pnpm-lock.yaml                # Locked dependencies
│   ├── smoke.spec.ts                 # Fast smoke tests
│   ├── comprehensive.spec.ts         # Fixture-driven tests
│   └── README.md                     # Node-specific notes
│
├── ruby/                             # Ruby test app (RubyGems)
│   ├── .ruby-version                 # Ruby version (3.2+)
│   ├── Gemfile                       # Depends on html-to-markdown from RubyGems
│   ├── Gemfile.lock                  # Locked dependencies
│   ├── smoke_test.rb                 # Fast smoke tests
│   ├── comprehensive_test.rb         # Fixture-driven tests
│   └── README.md                     # Ruby-specific notes
│
├── go/                               # Go test app (pkg.go.dev)
│   ├── go.mod                        # Depends on html-to-markdown from pkg.go.dev
│   ├── go.sum                        # Locked dependencies
│   ├── main_test.go                  # Smoke + comprehensive tests
│   ├── fixtures_test.go              # Fixture loading
│   └── README.md                     # Go-specific notes
│
├── java/                             # Java test app (Maven Central)
│   ├── pom.xml                       # Depends on html-to-markdown from Maven
│   ├── src/test/java/
│   │   └── io/github/goldziher/htmltomarkdown/
│   │       ├── SmokeTest.java        # Fast smoke tests
│   │       └── ComprehensiveTest.java # Fixture-driven tests
│   └── README.md                     # Java-specific notes
│
├── csharp/                           # C# test app (NuGet)
│   ├── TestApp.csproj                # Depends on html-to-markdown from NuGet
│   ├── SmokeTest.cs                  # Fast smoke tests
│   ├── ComprehensiveTest.cs          # Fixture-driven tests
│   ├── Fixtures.cs                   # Fixture loading
│   └── README.md                     # C#-specific notes
│
├── php/                              # PHP test app (Packagist)
│   ├── composer.json                 # Depends on html-to-markdown from Packagist
│   ├── composer.lock                 # Locked dependencies
│   ├── smoke_test.php                # Fast smoke tests
│   ├── comprehensive_test.php        # Fixture-driven tests
│   └── README.md                     # PHP-specific notes
│
└── elixir/                           # Elixir test app (Hex.pm)
    ├── mix.exs                       # Depends on html_to_markdown from Hex.pm
    ├── mix.lock                      # Locked dependencies
    ├── test/smoke_test.exs           # Fast smoke tests
    ├── test/comprehensive_test.exs   # Fixture-driven tests
    └── README.md                     # Elixir-specific notes
```

## Shared Fixtures Format

All fixtures follow a consistent JSON schema to ensure language parity.

### Fixture Schema

**File**: `fixtures/basic-html.json` (example)

```json
[
  {
    "name": "Simple paragraph",
    "html": "<p>Hello World</p>",
    "expectedMarkdown": "Hello World",
    "options": {}
  },
  {
    "name": "Heading level 1",
    "html": "<h1>Title</h1>",
    "expectedMarkdown": "# Title",
    "options": {
      "headingStyle": "Atx"
    }
  },
  {
    "name": "Strong emphasis",
    "html": "<strong>Bold text</strong>",
    "expectedMarkdown": "**Bold text**",
    "options": {}
  }
]
```

**Schema Fields**:
- `name` (string): Human-readable test case name
- `html` (string): Input HTML
- `expectedMarkdown` (string): Expected Markdown output
- `options` (object): Conversion options (language-specific format)

### Fixture Categories

1. **basic-html.json**: Core HTML elements (p, h1-h6, strong, em, lists, links, images)
2. **complex-html.json**: Complex structures (nested lists, tables, blockquotes)
3. **edge-cases.json**: Edge cases (special chars, Unicode, HTML entities, malformed HTML)
4. **metadata-extraction.json**: Metadata features (title extraction, meta tags)
5. **real-world.json**: Real-world samples (Wikipedia, Medium, GitHub README)

## Running Tests

### Smoke Tests (Pre-Release)

Run before publishing packages to registries:

```bash
# All languages (fast: <5 min)
task e2e:smoke:all

# Individual languages
task e2e:smoke:python
task e2e:smoke:node
task e2e:smoke:ruby
task e2e:smoke:php
task e2e:smoke:go
task e2e:smoke:java
task e2e:smoke:csharp
task e2e:smoke:elixir
```

### Comprehensive Tests (Post-Release)

Run after publishing or during development:

```bash
# All languages (thorough: 8-24 min)
task e2e:test:all

# Individual languages
task e2e:test:python
task e2e:test:node
task e2e:test:ruby
task e2e:test:php
task e2e:test:go
task e2e:test:java
task e2e:test:csharp
task e2e:test:elixir
```

### Manual Testing

For debugging or local validation:

```bash
# Python
cd tests/test_apps/python
uv sync
uv run pytest smoke_test.py -v
uv run pytest comprehensive_test.py -v

# Node.js
cd tests/test_apps/node
pnpm install
pnpm test:smoke
pnpm test:comprehensive

# Ruby
cd tests/test_apps/ruby
bundle install
bundle exec rspec smoke_test.rb
bundle exec rspec comprehensive_test.rb

# Go
cd tests/test_apps/go
go test -v -run TestSmoke
go test -v -run TestComprehensive
```

## CI/CD Integration

### Pre-Release Gate (Publish Workflow)

**File**: `.github/workflows/publish.yaml`

```yaml
jobs:
  smoke-tests:
    name: Smoke Tests - ${{ matrix.language }}
    needs: prepare
    strategy:
      fail-fast: false
      matrix:
        language: [python, node, ruby, php, go, java, csharp, elixir]
    steps:
      - uses: actions/checkout@v6

      - name: Install Task
        uses: arduino/setup-task@v2

      - name: Setup Language
        # ... language-specific setup

      - name: Run Smoke Tests
        run: task e2e:smoke:${{ matrix.language }}

  publish:
    needs: [prepare, smoke-tests]  # Blocks on smoke tests
    if: success()
    # ... publish steps
```

**Behavior**:
- Runs BEFORE publishing packages
- Blocks release if ANY smoke test fails
- Fast feedback (<5 min total)

### Post-Release Validation (Optional)

**File**: `.github/workflows/post-release-validation.yaml` (future)

```yaml
on:
  release:
    types: [published]

jobs:
  comprehensive-tests:
    name: Comprehensive Tests - ${{ matrix.language }}
    strategy:
      fail-fast: false
      matrix:
        language: [python, node, ruby, php, go, java, csharp, elixir]
    steps:
      - name: Install Task
        uses: arduino/setup-task@v2

      - name: Run Comprehensive Tests
        run: task e2e:test:${{ matrix.language }}
```

**Behavior**:
- Runs AFTER packages are published
- Validates real-world installation
- Does NOT block release (informational)

## Version Management

### Automatic Version Sync

The `scripts/sync_versions.py` script updates test_apps manifests when versions change:

```python
# From Cargo.toml (source of truth)
version = "2.18.0"

# Propagates to:
# - tests/test_apps/python/pyproject.toml   → html-to-markdown>=2.18.0
# - tests/test_apps/node/package.json       → html-to-markdown@>=2.18.0
# - tests/test_apps/ruby/Gemfile            → gem 'html-to-markdown', '>= 2.18.0'
# - tests/test_apps/go/go.mod               → (version in module path)
# - tests/test_apps/java/pom.xml            → <version>2.18.0</version>
# - tests/test_apps/csharp/TestApp.csproj   → <Version>2.18.0</Version>
# - tests/test_apps/php/composer.json       → "kreuzberg-dev/html-to-markdown": ">=2.18.0"
# - tests/test_apps/elixir/mix.exs          → {:html_to_markdown, "~> 2.18.0"}
```

**Usage**:
```bash
task versions:sync  # Updates all manifests including test_apps
```

### Version Constraints

All test_apps use **minimum version constraints** to ensure compatibility:

```json
// Python (pyproject.toml)
"html-to-markdown>=2.18.0"

// Node (package.json)
"html-to-markdown": ">=2.18.0"

// Ruby (Gemfile)
gem 'html-to-markdown', '>= 2.18.0'

// PHP (composer.json)
"kreuzberg-dev/html-to-markdown": ">=2.18.0"
```

## Adding a New Test Case

### Step 1: Add to Shared Fixtures

**File**: `fixtures/basic-html.json`

```json
[
  {
    "name": "Code block",
    "html": "<pre><code>const x = 42;</code></pre>",
    "expectedMarkdown": "```\nconst x = 42;\n```",
    "options": {
      "codeBlockStyle": "Fenced"
    }
  }
]
```

### Step 2: Verify in Comprehensive Tests

Comprehensive tests automatically pick up new fixtures (parametrized):

```bash
# Python
task e2e:test:python
# ✓ test_basic_html_conversion[Code block] PASSED

# Node
task e2e:test:node
# ✓ Code block PASSED

# All languages
task e2e:test:all
# ✓ All 8 languages pass new test case
```

## Adding a New Language Test App

Let's add **Kotlin** as an example:

### Step 1: Create Directory Structure

```bash
mkdir -p tests/test_apps/kotlin/src/test/kotlin/io/github/goldziher/htmltomarkdown
```

### Step 2: Create Package Manifest

**File**: `tests/test_apps/kotlin/build.gradle.kts`

```kotlin
plugins {
    kotlin("jvm") version "1.9.20"
    id("org.jetbrains.kotlinx.kover") version "0.7.4"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("io.github.goldziher:html-to-markdown:2.18.0")
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
}

tasks.test {
    useJUnitPlatform()
}
```

### Step 3: Create Smoke Tests

**File**: `src/test/kotlin/.../SmokeTest.kt`

```kotlin
package io.github.goldziher.htmltomarkdown

import org.junit.jupiter.api.Test
import kotlin.test.assertTrue
import kotlin.test.assertNotNull

class SmokeTest {
    @Test
    fun `package imports correctly`() {
        val converter = HtmlToMarkdown()
        assertNotNull(converter)
    }

    @Test
    fun `basic conversion works`() {
        val converter = HtmlToMarkdown()
        val html = "<p>Hello World</p>"
        val result = converter.convert(html)
        assertTrue(result.contains("Hello World"))
    }
}
```

### Step 4: Create Comprehensive Tests

**File**: `src/test/kotlin/.../ComprehensiveTest.kt`

```kotlin
package io.github.goldziher.htmltomarkdown

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory
import java.nio.file.Files
import java.nio.file.Paths
import kotlin.test.assertEquals

@Serializable
data class TestCase(
    val name: String,
    val html: String,
    val expectedMarkdown: String,
    val options: Map<String, String> = emptyMap()
)

class ComprehensiveTest {
    private fun loadFixtures(filename: String): List<TestCase> {
        val path = Paths.get("../fixtures/$filename")
        val json = Files.readString(path)
        return Json.decodeFromString(json)
    }

    @TestFactory
    fun `basic HTML conversions`() = loadFixtures("basic-html.json").map { tc ->
        DynamicTest.dynamicTest(tc.name) {
            val converter = HtmlToMarkdown()
            val result = converter.convert(tc.html, tc.options)
            assertEquals(tc.expectedMarkdown.trim(), result.trim())
        }
    }
}
```

### Step 5: Add to Taskfile

**File**: `Taskfile.yml`

```yaml
tasks:
  e2e:smoke:kotlin:
    desc: "Smoke test: Kotlin from Maven Central"
    dir: tests/test_apps/kotlin
    cmds:
      - gradle test --tests SmokeTest

  e2e:test:kotlin:
    desc: "Comprehensive test: Kotlin"
    dir: tests/test_apps/kotlin
    cmds:
      - gradle test

  e2e:smoke:all:
    cmds:
      - task: e2e:smoke:python
      # ... existing languages
      - task: e2e:smoke:kotlin  # ADD THIS

  e2e:test:all:
    cmds:
      - task: e2e:test:python
      # ... existing languages
      - task: e2e:test:kotlin  # ADD THIS
```

### Step 6: Extend Version Sync

**File**: `scripts/sync_versions.py`

```python
def update_test_apps_versions(repo_root: Path, version: str) -> None:
    # ... existing updates

    # Kotlin: build.gradle.kts
    kotlin_gradle = test_apps / "kotlin" / "build.gradle.kts"
    if kotlin_gradle.exists():
        update_gradle_dependency(kotlin_gradle, "html-to-markdown", version)
```

## Language-Specific Notes

### Python (PyPI)

**Package Manager**: uv (fast pip replacement)
**Test Framework**: pytest
**Coverage**: pytest-cov

```bash
cd tests/test_apps/python
uv sync
uv run pytest -v
```

### Node.js (npm)

**Package Manager**: pnpm
**Test Framework**: vitest
**Coverage**: vitest built-in

```bash
cd tests/test_apps/node
pnpm install
pnpm test
```

### Ruby (RubyGems)

**Package Manager**: bundler
**Test Framework**: RSpec
**Coverage**: simplecov

```bash
cd tests/test_apps/ruby
bundle install
bundle exec rspec
```

### Go (pkg.go.dev)

**Package Manager**: go modules
**Test Framework**: testing (stdlib)
**Coverage**: go test -cover

```bash
cd tests/test_apps/go
go test -v
go test -cover
```

### Java (Maven Central)

**Package Manager**: Maven/Gradle
**Test Framework**: JUnit 5
**Coverage**: JaCoCo

```bash
cd tests/test_apps/java
mvn test
```

### C# (NuGet)

**Package Manager**: NuGet
**Test Framework**: xUnit
**Coverage**: Coverlet

```bash
cd tests/test_apps/csharp
dotnet test
```

### PHP (Packagist)

**Package Manager**: Composer
**Test Framework**: PHPUnit
**Coverage**: PHPUnit built-in

```bash
cd tests/test_apps/php
composer install
composer test
```

### Elixir (Hex.pm)

**Package Manager**: Mix
**Test Framework**: ExUnit
**Coverage**: excoveralls

```bash
cd tests/test_apps/elixir
mix deps.get
mix test
```

## Troubleshooting

### Package Not Found

**Problem**: `pip install html-to-markdown` fails with "No matching distribution found"

**Solution**: Package hasn't been published yet. Wait for publish workflow to complete.

### Version Mismatch

**Problem**: Tests pass locally but fail in CI with version errors

**Solution**: Run `task versions:sync` to ensure all test_apps manifests use correct version:
```bash
task versions:sync
git diff tests/test_apps/  # Review changes
```

### Fixture Loading Fails

**Problem**: `FileNotFoundError: ../fixtures/basic-html.json`

**Solution**: Ensure tests load fixtures relative to `test_apps/` directory:
```python
# ✅ Correct
fixture_path = Path(__file__).parent.parent / "fixtures" / "basic-html.json"

# ❌ Incorrect
fixture_path = Path("../fixtures/basic-html.json")  # Depends on cwd
```

### Test Timeout

**Problem**: Comprehensive tests timeout in CI

**Solution**: Reduce fixture count or increase CI timeout:
```yaml
# .github/workflows/post-release-validation.yaml
- run: task e2e:test:all
  timeout-minutes: 30  # Increase from default 10
```

## Best Practices

1. **Test published packages only** - Don't mount local code
2. **Use version constraints** - `>=2.18.0` not exact versions
3. **Share fixtures** - All languages test same inputs/outputs
4. **Fast smoke tests** - Keep under 30 seconds per language
5. **Parametrize comprehensive** - Use language-native parametrization
6. **Clear assertions** - Include test case name in failure messages

## References

- **Fixture Format**: fixtures/README.md
- **Task Commands**: ../../Taskfile.yml
- **Version Sync**: ../../scripts/sync_versions.py
- **CI Workflows**: ../../.github/workflows/publish.yaml

---

**Last Updated**: 2025-12-28
**Maintainers**: html-to-markdown contributors
