# Basic HTML to Markdown Conversion (Java)

Convert HTML to Markdown with a simple static method call.

## Simple Conversion

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
        // Output:
        // # Hello World
        //
        // This is a **test**.
    }
}
```

## Get Library Version

```java
String version = HtmlToMarkdown.getVersion();
System.out.println("html-to-markdown version: " + version);
```

## Error Handling

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class ErrorHandlingExample {
    public static void main(String[] args) {
        String html = "<h1>Example</h1>";

        try {
            String markdown = HtmlToMarkdown.convert(html);
            System.out.println(markdown);
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Conversion failed: " + e.getMessage());
        }
    }
}
```

## Complex HTML Example

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class ComplexExample {
    public static void main(String[] args) {
        String html = """
            <div>
                <h2>Features</h2>
                <ul>
                    <li>Fast <em>Rust</em> core</li>
                    <li>CommonMark compliant</li>
                    <li><code>Zero</code> dependencies</li>
                </ul>
                <table>
                    <tr>
                        <th>Language</th>
                        <th>Speed</th>
                    </tr>
                    <tr>
                        <td>Java</td>
                        <td>Fast</td>
                    </tr>
                </table>
            </div>
            """;

        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

## Thread-Safe Concurrent Processing

The conversion is thread-safe and can be used concurrently:

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import java.util.List;
import java.util.stream.Collectors;

public class BatchProcessing {
    public static void main(String[] args) {
        List<String> htmlDocuments = List.of(
            "<h1>Document 1</h1><p>Content</p>",
            "<h1>Document 2</h1><p>More content</p>",
            "<h1>Document 3</h1><p>Even more</p>"
        );

        // Thread-safe: can be called concurrently
        List<String> markdownResults = htmlDocuments.stream()
            .map(HtmlToMarkdown::convert)
            .collect(Collectors.toList());

        markdownResults.forEach(System.out::println);
    }
}
```
