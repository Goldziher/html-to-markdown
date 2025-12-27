using HtmlToMarkdown.Visitor;
using Xunit;

namespace HtmlToMarkdown.Tests;

public class VisitorTests
{
    [Fact]
    public void VisitHeading_H1()
    {
        var html = "<h1>Title</h1>";
        var results = new List<(int level, string text)>();

        var visitor = new HeadingTrackingVisitor(results);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.NotEmpty(results);
        Assert.Equal(1, results[0].level);
        Assert.Contains("Title", results[0].text);
    }

    [Fact]
    public void VisitLink_Tracked()
    {
        var html = "<a href=\"https://example.com\">Example</a>";
        var links = new List<(string href, string text)>();

        var visitor = new LinkTrackingVisitor(links);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.NotEmpty(links);
        Assert.Equal("https://example.com", links[0].href);
        Assert.Contains("Example", links[0].text);
    }

    [Fact]
    public void VisitImage_Tracked()
    {
        var html = "<img src=\"image.png\" alt=\"An image\" />";
        var images = new List<(string src, string alt)>();

        var visitor = new ImageTrackingVisitor(images);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.NotEmpty(images);
        Assert.Equal("image.png", images[0].src);
        Assert.Equal("An image", images[0].alt);
    }

    [Fact]
    public void VisitCodeInline_Tracked()
    {
        var html = "<p>Use <code>Console.WriteLine()</code> to print.</p>";
        var inlineCode = new List<string>();

        var visitor = new CodeInlineTrackingVisitor(inlineCode);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.Contains("Console.WriteLine()", inlineCode);
    }

    [Fact]
    public void VisitListItem_Tracked()
    {
        var html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        var items = new List<string>();

        var visitor = new ListItemTrackingVisitor(items);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.NotEmpty(items);
    }

    [Fact]
    public void VisitStrong_Tracked()
    {
        var html = "<strong>Bold text</strong>";
        var strongTexts = new List<string>();

        var visitor = new StrongTrackingVisitor(strongTexts);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.Contains("Bold text", strongTexts);
    }

    [Fact]
    public void VisitEmphasis_Tracked()
    {
        var html = "<em>Italic text</em>";
        var emphasisTexts = new List<string>();

        var visitor = new EmphasisTrackingVisitor(emphasisTexts);
        HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.Contains("Italic text", emphasisTexts);
    }

    [Fact]
    public void VisitResult_Continue()
    {
        var result = VisitResult.Continue();
        Assert.IsType<ContinueResult>(result);
        Assert.Equal(VisitResultType.Continue, result.ResultType);
    }

    [Fact]
    public void VisitResult_Custom()
    {
        var result = VisitResult.Custom("**custom**");
        Assert.IsType<CustomResult>(result);
        Assert.Equal(VisitResultType.Custom, result.ResultType);
        Assert.Equal("**custom**", ((CustomResult)result).CustomOutput);
    }

    [Fact]
    public void VisitResult_Skip()
    {
        var result = VisitResult.Skip();
        Assert.IsType<SkipResult>(result);
        Assert.Equal(VisitResultType.Skip, result.ResultType);
    }

    [Fact]
    public void VisitResult_PreserveHtml()
    {
        var result = VisitResult.PreserveHtml();
        Assert.IsType<PreserveHtmlResult>(result);
        Assert.Equal(VisitResultType.PreserveHtml, result.ResultType);
    }

    [Fact]
    public void VisitResult_Error()
    {
        var result = VisitResult.Error("Something went wrong");
        Assert.IsType<ErrorResult>(result);
        Assert.Equal(VisitResultType.Error, result.ResultType);
        Assert.Equal("Something went wrong", ((ErrorResult)result).ErrorMessage);
    }

    [Fact]
    public void NodeContext_HasAttribute()
    {
        var attrs = new List<Visitor.Attribute> { new("class", "container"), new("id", "main") };
        var context = new NodeContext(
            NodeType.Div,
            "div",
            attrs.AsReadOnly(),
            0, 0, null, false);

        Assert.True(context.HasAttribute("class"));
        Assert.True(context.HasAttribute("id"));
        Assert.False(context.HasAttribute("nonexistent"));
    }

    [Fact]
    public void NodeContext_GetAttribute()
    {
        var attrs = new List<Visitor.Attribute> { new("class", "container"), new("id", "main") };
        var context = new NodeContext(
            NodeType.Div,
            "div",
            attrs.AsReadOnly(),
            0, 0, null, false);

        Assert.Equal("container", context.GetAttribute("class"));
        Assert.Equal("main", context.GetAttribute("id"));
        Assert.Null(context.GetAttribute("nonexistent"));
    }

    [Fact]
    public void ConvertWithVisitor_ByteInput()
    {
        var html = "<h1>Test</h1>";
        var bytes = System.Text.Encoding.UTF8.GetBytes(html);

        var visitor = new SimpleVisitor();
        var result = HtmlToMarkdownConverter.ConvertWithVisitor(bytes, visitor);

        Assert.NotEmpty(result);
        Assert.Contains("Test", result);
    }

    [Fact]
    public void ConvertWithVisitor_NullHtml_Throws()
    {
        var visitor = new SimpleVisitor();
        Assert.Throws<ArgumentNullException>(() =>
            HtmlToMarkdownConverter.ConvertWithVisitor((string)null!, visitor));
    }

    [Fact]
    public void ConvertWithVisitor_NullVisitor_Throws()
    {
        Assert.Throws<ArgumentNullException>(() =>
            HtmlToMarkdownConverter.ConvertWithVisitor("<p>test</p>", (IVisitor)null!));
    }

    [Fact]
    public void ConvertWithVisitor_EmptyHtml_ReturnsEmpty()
    {
        var visitor = new SimpleVisitor();
        var result = HtmlToMarkdownConverter.ConvertWithVisitor("", visitor);
        Assert.Empty(result);
    }

    [Fact]
    public void CustomResult_ReplaceHeading()
    {
        var html = "<h1>Original</h1>";

        var visitor = new CustomHeadingVisitor();
        var result = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

        Assert.Contains("CUSTOM", result);
    }


    private class SimpleVisitor : IVisitor
    {
    }

    private class HeadingTrackingVisitor : IVisitor
    {
        private readonly List<(int level, string text)> _results;

        public HeadingTrackingVisitor(List<(int level, string text)> results)
        {
            _results = results;
        }

        public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
        {
            _results.Add((level, text));
            return VisitResult.Continue();
        }
    }

    private class LinkTrackingVisitor : IVisitor
    {
        private readonly List<(string href, string text)> _links;

        public LinkTrackingVisitor(List<(string href, string text)> links)
        {
            _links = links;
        }

        public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
        {
            _links.Add((href, text));
            return VisitResult.Continue();
        }
    }

    private class ImageTrackingVisitor : IVisitor
    {
        private readonly List<(string src, string alt)> _images;

        public ImageTrackingVisitor(List<(string src, string alt)> images)
        {
            _images = images;
        }

        public VisitResult VisitImage(NodeContext context, string src, string alt, string? title)
        {
            _images.Add((src, alt));
            return VisitResult.Continue();
        }
    }

    private class CodeInlineTrackingVisitor : IVisitor
    {
        private readonly List<string> _inlineCode;

        public CodeInlineTrackingVisitor(List<string> inlineCode)
        {
            _inlineCode = inlineCode;
        }

        public VisitResult VisitCodeInline(NodeContext context, string code)
        {
            _inlineCode.Add(code);
            return VisitResult.Continue();
        }
    }

    private class ListItemTrackingVisitor : IVisitor
    {
        private readonly List<string> _items;

        public ListItemTrackingVisitor(List<string> items)
        {
            _items = items;
        }

        public VisitResult VisitListItem(NodeContext context, bool ordered, string marker, string text)
        {
            _items.Add(text);
            return VisitResult.Continue();
        }
    }

    private class StrongTrackingVisitor : IVisitor
    {
        private readonly List<string> _strongTexts;

        public StrongTrackingVisitor(List<string> strongTexts)
        {
            _strongTexts = strongTexts;
        }

        public VisitResult VisitStrong(NodeContext context, string text)
        {
            _strongTexts.Add(text);
            return VisitResult.Continue();
        }
    }

    private class EmphasisTrackingVisitor : IVisitor
    {
        private readonly List<string> _emphasisTexts;

        public EmphasisTrackingVisitor(List<string> emphasisTexts)
        {
            _emphasisTexts = emphasisTexts;
        }

        public VisitResult VisitEmphasis(NodeContext context, string text)
        {
            _emphasisTexts.Add(text);
            return VisitResult.Continue();
        }
    }

    private class CustomHeadingVisitor : IVisitor
    {
        public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
        {
            return VisitResult.Custom("# CUSTOM HEADING");
        }
    }
}
