using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

namespace HtmlToMarkdown.Examples;

/// <summary>
/// Visitor pattern example that collects analytics about the HTML document.
/// </summary>
public class VisitorAnalytics
{
    public static void Main()
    {
        var html = @"
            <!DOCTYPE html>
            <html>
            <head><title>My Blog</title></head>
            <body>
                <h1>Welcome to My Blog</h1>
                <p>Published on <time>2025-01-15</time></p>

                <section>
                    <h2>First Article</h2>
                    <p>Article content with a <strong>bold statement</strong> and <em>emphasis</em>.</p>
                    <a href=""/article1"">Read more</a>
                </section>

                <section>
                    <h2>Second Article</h2>
                    <p>More content here.</p>
                    <img src=""image1.jpg"" alt=""Article image"" />
                    <p>And more text with a <code>code snippet</code>.</p>
                </section>

                <footer>
                    <p>Contact me at <a href=""mailto:author@example.com"">author@example.com</a></p>
                </footer>
            </body>
            </html>";

        var analytics = new AnalyticsVisitor();

        var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, analytics);

        Console.WriteLine("=== Document Analytics ===\n");
        Console.WriteLine($"Headings: {analytics.HeadingCount}");
        Console.WriteLine($"Links: {analytics.LinkCount}");
        Console.WriteLine($"Images: {analytics.ImageCount}");
        Console.WriteLine($"Code snippets: {analytics.CodeInlineCount}");
        Console.WriteLine($"Strong elements: {analytics.StrongCount}");
        Console.WriteLine($"Emphasis elements: {analytics.EmphasisCount}");
        Console.WriteLine($"Total elements visited: {analytics.TotalElements}");

        Console.WriteLine("\n=== Links Found ===");
        foreach (var (href, text) in analytics.Links)
        {
            Console.WriteLine($"- [{text}]({href})");
        }

        Console.WriteLine("\n=== Heading Structure ===");
        foreach (var (level, text) in analytics.Headings)
        {
            var indent = new string(' ', (level - 1) * 2);
            Console.WriteLine($"{indent}L{level}: {text}");
        }

        Console.WriteLine("\n=== Markdown Output (first 500 chars) ===");
        Console.WriteLine(markdown[..Math.Min(500, markdown.Length)]);
        if (markdown.Length > 500)
            Console.WriteLine("...");
    }
}

/// <summary>
/// A visitor that collects analytics about the HTML document.
/// </summary>
public class AnalyticsVisitor : IVisitor
{
    public int HeadingCount { get; private set; }
    public int LinkCount { get; private set; }
    public int ImageCount { get; private set; }
    public int CodeInlineCount { get; private set; }
    public int StrongCount { get; private set; }
    public int EmphasisCount { get; private set; }
    public int TotalElements { get; private set; }

    public List<(string href, string text)> Links { get; } = new();
    public List<(int level, string text)> Headings { get; } = new();
    public Dictionary<string, int> ElementCounts { get; } = new();

    public VisitResult VisitElementStart(NodeContext context)
    {
        TotalElements++;

        if (!string.IsNullOrEmpty(context.TagName))
        {
            if (!ElementCounts.ContainsKey(context.TagName))
                ElementCounts[context.TagName] = 0;
            ElementCounts[context.TagName]++;
        }

        return VisitResult.Continue();
    }

    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        HeadingCount++;
        Headings.Add((level, text));
        return VisitResult.Continue();
    }

    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        LinkCount++;
        Links.Add((href, text));
        return VisitResult.Continue();
    }

    public VisitResult VisitImage(NodeContext context, string src, string alt, string? title)
    {
        ImageCount++;
        return VisitResult.Continue();
    }

    public VisitResult VisitCodeInline(NodeContext context, string code)
    {
        CodeInlineCount++;
        return VisitResult.Continue();
    }

    public VisitResult VisitStrong(NodeContext context, string text)
    {
        StrongCount++;
        return VisitResult.Continue();
    }

    public VisitResult VisitEmphasis(NodeContext context, string text)
    {
        EmphasisCount++;
        return VisitResult.Continue();
    }
}
