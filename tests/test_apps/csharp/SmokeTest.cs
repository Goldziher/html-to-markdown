using Xunit;
using HtmlToMarkdown;

namespace HtmlToMarkdownTestApp;

public class SmokeTest
{
    [Fact]
    public void TestPackageLoads()
    {
        Assert.NotNull(typeof(HtmlToMarkdownConverter));
    }

    [Fact]
    public void TestBasicConversion()
    {
        var html = "<p>Hello World</p>";
        var result = HtmlToMarkdownConverter.Convert(html);
        Assert.Contains("Hello World", result);
    }

    [Fact]
    public void TestWithOptions()
    {
        var html = "<h1>Title</h1>";
        var result = HtmlToMarkdownConverter.Convert(html);
        Assert.StartsWith("#", result);
    }

    [Fact]
    public void TestEmptyInput()
    {
        var result = HtmlToMarkdownConverter.Convert("");
        Assert.Equal("", result);
    }
}
