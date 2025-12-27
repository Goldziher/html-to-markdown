using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

namespace HtmlToMarkdown.Examples;

/// <summary>
/// Visitor pattern example that filters out certain elements during conversion.
/// </summary>
public class VisitorFilter
{
    public static void Main()
    {
        var html = @"
            <article>
                <h1>Article Title</h1>
                <p>First paragraph of content.</p>
                <aside>
                    <p>This is a sidebar - we'll skip it.</p>
                </aside>
                <p>Second paragraph of content.</p>
                <figure>
                    <img src=""image.png"" alt=""An image"" />
                    <figcaption>Figure caption</figcaption>
                </figure>
                <p>Final paragraph.</p>
            </article>";

        var visitor = new FilteringVisitor();

        var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Console.WriteLine("=== Original HTML (abridged) ===");
        Console.WriteLine(html);
        Console.WriteLine("\n=== Filtered Markdown ===");
        Console.WriteLine(markdown);
        Console.WriteLine($"\n=== Statistics ===");
        Console.WriteLine($"Elements skipped: {visitor.SkippedCount}");
        Console.WriteLine($"Elements preserved: {visitor.PreservedCount}");
    }
}

/// <summary>
/// A visitor that filters out sidebars and figures from the output.
/// </summary>
public class FilteringVisitor : IVisitor
{
    public int SkippedCount { get; private set; }
    public int PreservedCount { get; private set; }

    public VisitResult VisitElementStart(NodeContext context)
    {
        if (context.TagName == "aside")
        {
            SkippedCount++;
            return VisitResult.Skip();
        }

        if (context.TagName == "figure")
        {
            SkippedCount++;
            return VisitResult.Skip();
        }

        return VisitResult.Continue();
    }

    public VisitResult VisitImage(NodeContext context, string src, string alt, string? title)
    {
        PreservedCount++;
        return VisitResult.PreserveHtml();
    }
}
