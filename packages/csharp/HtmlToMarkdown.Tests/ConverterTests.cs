using Xunit;

namespace HtmlToMarkdown.Tests;

public class ConverterTests
{
    [Fact]
    public void Convert_SimpleHeading_ReturnsMarkdown()
    {
        var html = "<h1>Hello World</h1>";
        var markdown = HtmlToMarkdownConverter.Convert(html);
        Assert.Contains("Hello World", markdown);
    }

    [Fact]
    public void Convert_EmptyString_ReturnsEmptyString()
    {
        var markdown = HtmlToMarkdownConverter.Convert("");
        Assert.Equal("", markdown);
    }

    [Fact]
    public void Convert_NullInput_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() =>
            HtmlToMarkdownConverter.Convert(null!));
    }

    [Fact]
    public void Convert_Paragraph_ReturnsMarkdown()
    {
        var html = "<p>This is a test.</p>";
        var markdown = HtmlToMarkdownConverter.Convert(html);
        Assert.Contains("This is a test", markdown);
    }

    [Fact]
    public void Convert_StrongText_ReturnsMarkdown()
    {
        var html = "<strong>Bold text</strong>";
        var markdown = HtmlToMarkdownConverter.Convert(html);
        Assert.Contains("Bold text", markdown);
    }

    [Fact]
    public void Convert_List_ReturnsMarkdown()
    {
        var html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        var markdown = HtmlToMarkdownConverter.Convert(html);
        Assert.Contains("Item 1", markdown);
        Assert.Contains("Item 2", markdown);
    }

    [Fact]
    public void GetVersion_ReturnsNonEmptyString()
    {
        var version = HtmlToMarkdownConverter.GetVersion();
        Assert.False(string.IsNullOrEmpty(version));
        Assert.NotEqual("unknown", version);
    }
}
