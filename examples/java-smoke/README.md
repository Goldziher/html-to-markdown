# Java Smoke Test

This directory contains a minimal smoke test for the `html-to-markdown` Java bindings.

## Purpose

Verifies that the published Java JAR can be:
- Installed via Maven
- Imported and used in Java code
- Successfully converts HTML to Markdown

## Structure

- `pom.xml` - Maven project configuration with html-to-markdown dependency
- `src/main/java/io/github/goldziher/htmltomarkdown/SmokeTest.java` - Simple test that converts HTML and verifies output

## Running Locally

```bash
cd examples/java-smoke

# Install html-to-markdown from local build
pushd ../../packages/java
mvn install -DskipTests
popd

# Run the smoke test
mvn compile exec:java -Dexec.mainClass="io.github.goldziher.htmltomarkdown.SmokeTest"
```

## CI Usage

The smoke test runs automatically in the publish workflow via the `smoke-java` composite action in `.github/actions/smoke-java/`.
