using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

namespace HtmlToMarkdown.Examples;

/// <summary>
/// Basic visitor pattern example that tracks elements as they are visited.
/// </summary>
public class VisitorBasic
{
    public static void Main()
    {
        var html = @"
            <div>
                <h1>Hello World</h1>
                <p>This is a test paragraph.</p>
                <a href=""https://example.com"">Click here</a>
            </div>";

        // Create a custom visitor that tracks visited elements
        var visitor = new BasicTrackingVisitor();

        // Convert HTML to Markdown while tracking elements
        var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Console.WriteLine("=== Markdown Output ===");
        Console.WriteLine(markdown);
        Console.WriteLine("\n=== Visited Elements ===");

        foreach (var element in visitor.VisitedElements)
        {
            Console.WriteLine($"- {element}");
        }
    }
}

/// <summary>
/// A simple visitor that tracks which elements are visited.
/// </summary>
public class BasicTrackingVisitor : IVisitor
{
    private readonly List<string> _visitedElements = new();

    public IReadOnlyList<string> VisitedElements => _visitedElements.AsReadOnly();

    public VisitResult VisitElementStart(NodeContext context)
    {
        if (!string.IsNullOrEmpty(context.TagName))
        {
            _visitedElements.Add($"<{context.TagName}> at depth {context.Depth}");
        }

        return VisitResult.Continue();
    }

    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        _visitedElements.Add($"H{level}: {text}");
        return VisitResult.Continue();
    }

    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        _visitedElements.Add($"Link: {text} -> {href}");
        return VisitResult.Continue();
    }
}
