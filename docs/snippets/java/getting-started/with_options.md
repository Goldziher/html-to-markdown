# Metadata Extraction and Advanced Options (Java)

Extract structured metadata while converting HTML to Markdown, including document information, headers, links, images, and structured data.

## Basic Metadata Extraction

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction;

public class MetadataExample {
    public static void main(String[] args) {
        String html = """
            <html>
            <head>
                <title>My Article</title>
                <meta name="description" content="An interesting read">
                <meta name="author" content="Jane Doe">
                <meta property="og:image" content="image.jpg">
            </head>
            <body>
                <h1>Welcome</h1>
                <a href="https://example.com">Link</a>
                <img src="image.jpg" alt="Featured image">
            </body>
            </html>
            """;

        try {
            MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

            // Access document metadata
            var doc = result.metadata().document();
            if (doc.title() != null) {
                System.out.println("Title: " + doc.title());
            }
            if (doc.author() != null) {
                System.out.println("Author: " + doc.author());
            }

            // Access Open Graph metadata
            doc.openGraph().forEach((key, value) ->
                System.out.println("OG " + key + ": " + value)
            );

            // Count extracted elements
            System.out.println("Headers: " + result.metadata().headers().size());
            System.out.println("Links: " + result.metadata().links().size());
            System.out.println("Images: " + result.metadata().images().size());

            // Print markdown output
            System.out.println("\nMarkdown:\n" + result.markdown());
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Conversion failed: " + e.getMessage());
        }
    }
}
```

## Extract and Analyze Document Structure

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.HeaderMetadata;

public class HeaderTraversalExample {
    public static void main(String[] args) {
        String html = """
            <html><body>
                <h1>Main Title</h1>
                <h2>Section One</h2>
                <h3>Subsection</h3>
                <h2>Section Two</h2>
            </body></html>
            """;

        try {
            var result = HtmlToMarkdown.convertWithMetadata(html);

            System.out.println("Document Structure:");
            for (HeaderMetadata header : result.metadata().headers()) {
                String indent = "  ".repeat(header.level() - 1);
                System.out.print(indent + "- Level " + header.level() + ": " + header.text());
                if (header.id() != null) {
                    System.out.print(" (#" + header.id() + ")");
                }
                System.out.println();
            }
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Failed: " + e.getMessage());
        }
    }
}
```

## Analyze Links by Type

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.LinkMetadata;
import java.util.List;
import java.util.stream.Collectors;

public class LinkAnalysisExample {
    public static void main(String[] args) {
        String html = """
            <html><body>
                <a href="https://external.com">External Site</a>
                <a href="/internal">Internal Page</a>
                <a href="#section">Anchor Link</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:+1234567890">Phone</a>
                <a href="https://external.com" rel="nofollow">Nofollow Link</a>
            </body></html>
            """;

        try {
            var result = HtmlToMarkdown.convertWithMetadata(html);

            // Group links by type
            var linksByType = result.metadata().links().stream()
                .collect(Collectors.groupingBy(LinkMetadata::linkType));

            // Process external links
            var external = linksByType.getOrDefault("external", List.of());
            if (!external.isEmpty()) {
                System.out.println("External Links: " + external.size());
                external.forEach(link ->
                    System.out.println("  - " + link.text() + ": " + link.href())
                );
            }

            // Find nofollow links
            var nofollow = result.metadata().links().stream()
                .filter(link -> link.rel().contains("nofollow"))
                .collect(Collectors.toList());

            if (!nofollow.isEmpty()) {
                System.out.println("\nNofollow Links: " + nofollow.size());
                nofollow.forEach(link ->
                    System.out.println("  - " + link.text() + ": " + link.href())
                );
            }
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Failed: " + e.getMessage());
        }
    }
}
```

## Visitor Pattern: Document Analytics

Use the visitor pattern to collect statistics during conversion:

```java
import dev.kreuzberg.htmltomarkdown.Visitor;
import dev.kreuzberg.htmltomarkdown.VisitResult;
import dev.kreuzberg.htmltomarkdown.NodeContext;
import java.util.Map;
import java.util.TreeMap;

public class DocumentAnalytics implements Visitor {
    private int linkCount = 0;
    private int imageCount = 0;
    private Map<Integer, Integer> headingLevels = new TreeMap<>();

    @Override
    public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
        linkCount++;
        return VisitResult.Continue.INSTANCE;
    }

    @Override
    public VisitResult visitImage(NodeContext ctx, String src, String alt, String title) {
        imageCount++;
        return VisitResult.Continue.INSTANCE;
    }

    @Override
    public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
        headingLevels.put(level, headingLevels.getOrDefault(level, 0) + 1);
        return VisitResult.Continue.INSTANCE;
    }

    public void printStats() {
        System.out.println("Links: " + linkCount);
        System.out.println("Images: " + imageCount);
        System.out.println("Headings by level: " + headingLevels);
    }
}
```

## Visitor Pattern: Custom Heading Format

```java
import dev.kreuzberg.htmltomarkdown.Visitor;
import dev.kreuzberg.htmltomarkdown.VisitResult;
import dev.kreuzberg.htmltomarkdown.NodeContext;

public class CustomHeadingFormat implements Visitor {
    @Override
    public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
        // Custom format: >>> Heading <<<
        String custom = ">".repeat(level) + " " + text + " " + "<".repeat(level);
        return new VisitResult.Custom(custom);
    }
}
```

## Visitor Pattern: Filter External Links

```java
import dev.kreuzberg.htmltomarkdown.Visitor;
import dev.kreuzberg.htmltomarkdown.VisitResult;
import dev.kreuzberg.htmltomarkdown.NodeContext;

public class ExternalLinkFilter implements Visitor {
    private static final String INTERNAL_DOMAIN = "example.com";

    @Override
    public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
        if (href != null && !href.startsWith("/") && !href.contains(INTERNAL_DOMAIN)) {
            // Skip external links
            return VisitResult.Skip.INSTANCE;
        }
        return VisitResult.Continue.INSTANCE;
    }
}
```

## Visitor Pattern: Content Sanitizer

```java
import dev.kreuzberg.htmltomarkdown.Visitor;
import dev.kreuzberg.htmltomarkdown.VisitResult;
import dev.kreuzberg.htmltomarkdown.NodeContext;

public class ContentSanitizer implements Visitor {
    @Override
    public VisitResult visitCustomElement(NodeContext ctx, String tagName, String html) {
        if ("script".equals(tagName) || "style".equals(tagName)) {
            return VisitResult.Skip.INSTANCE;
        }
        return VisitResult.Continue.INSTANCE;
    }
}
```

## Metadata Record Structures

Access structured metadata through type-safe Java records:

### DocumentMetadata
```java
// From MetadataExtraction result
var doc = result.metadata().document();
String title = doc.title();                           // <title> content
String description = doc.description();               // meta[name="description"]
List<String> keywords = doc.keywords();               // meta[name="keywords"]
String author = doc.author();                         // meta[name="author"]
String canonicalUrl = doc.canonicalUrl();             // <link rel="canonical">
String baseHref = doc.baseHref();                     // <base href="">
String language = doc.language();                     // lang attribute
String textDirection = doc.textDirection();           // dir attribute
Map<String, String> openGraph = doc.openGraph();      // og:* properties
Map<String, String> twitterCard = doc.twitterCard();  // twitter:* properties
```

### HeaderMetadata
```java
for (HeaderMetadata header : result.metadata().headers()) {
    int level = header.level();        // 1-6
    String text = header.text();       // Header text
    String id = header.id();           // HTML id attribute
    int depth = header.depth();        // DOM depth
    int htmlOffset = header.htmlOffset(); // Byte offset in HTML
}
```

### LinkMetadata
```java
for (LinkMetadata link : result.metadata().links()) {
    String href = link.href();                    // href attribute
    String text = link.text();                    // Link text
    String title = link.title();                  // title attribute
    String linkType = link.linkType();            // Classification
    List<String> rel = link.rel();                // rel attribute values
    Map<String, String> attributes = link.attributes(); // Additional attributes
}
```

### ImageMetadata
```java
for (ImageMetadata image : result.metadata().images()) {
    String src = image.src();                     // Image source
    String alt = image.alt();                     // alt text
    String title = image.title();                 // title attribute
    int[] dimensions = image.dimensions();        // [width, height]
    String imageType = image.imageType();         // Classification
    Map<String, String> attributes = image.attributes(); // Additional attributes
}
```
