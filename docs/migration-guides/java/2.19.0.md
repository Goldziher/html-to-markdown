# Migration Guide: Java v2.18.x → v2.19.0

## Breaking Change: Package Namespace

In v2.19.0, the Java package namespace changed from `io.github.goldziher` to `dev.kreuzberg` to reflect the new Kreuzberg.dev organization.

### Maven Dependency Update

**Before (v2.18.x):**
```xml
<dependency>
    <groupId>io.github.goldziher</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.18.x</version>
</dependency>
```

**After (v2.19.0+):**
```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.19.0</version>
    <classifier>linux</classifier> <!-- or macos, windows -->
</dependency>
```

### Import Statement Updates

Update all Java import statements to use the new namespace:

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.*;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.*;
```

### Gradle Build Updates

**Kotlin DSL - Before:**
```kotlin
implementation("io.github.goldziher:html-to-markdown:2.18.x")
```

**Kotlin DSL - After:**
```kotlin
implementation("dev.kreuzberg:html-to-markdown:2.19.0:linux") // or macos, windows
```

**Groovy DSL - Before:**
```groovy
implementation 'io.github.goldziher:html-to-markdown:2.18.x'
```

**Groovy DSL - After:**
```groovy
implementation 'dev.kreuzberg:html-to-markdown:2.19.0:linux' // or macos, windows
```

### Code Migration Example

**Before (v2.18.x):**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

**After (v2.19.0+):**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

### Metadata Extraction Update

If you use metadata extraction, update the imports as well:

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction;
```

### Visitor Pattern Update

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.visitor.Visitor;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.visitor.Visitor;
```

## Summary of Changes

- All public classes moved to `dev.kreuzberg.htmltomarkdown` package
- All metadata classes moved to `dev.kreuzberg.htmltomarkdown.metadata` package
- All visitor classes moved to `dev.kreuzberg.htmltomarkdown.visitor` package
- Maven Central groupId changed from `io.github.goldziher` to `dev.kreuzberg`
- Platform classifiers (linux, macos, windows) are now required in dependency declarations
- No functional changes to the API
- Full backward compatibility after import updates
