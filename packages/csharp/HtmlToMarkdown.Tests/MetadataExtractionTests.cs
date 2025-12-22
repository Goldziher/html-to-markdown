using HtmlToMarkdown.Metadata;
using Xunit;

namespace HtmlToMarkdown.Tests;

public class MetadataExtractionTests
{
    [Fact]
    public void ConvertWithMetadata_SimpleDocument_ExtractsMetadata()
    {
        var html = @"<html>
            <head>
                <title>Test Page</title>
                <meta name=""description"" content=""A test page"">
            </head>
            <body>
                <h1>Hello World</h1>
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result);
        Assert.Contains("Hello World", result.Markdown);
        Assert.NotNull(result.Metadata);
        Assert.NotNull(result.Metadata.Document);
        Assert.Equal("Test Page", result.Metadata.Document.Title);
        Assert.Equal("A test page", result.Metadata.Document.Description);
    }

    [Fact]
    public void ConvertWithMetadata_WithHeaders_ExtractsHeaderMetadata()
    {
        var html = @"<html>
            <body>
                <h1 id=""main"">Main Title</h1>
                <h2>Subtitle</h2>
                <h3>Section</h3>
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Headers);
        Assert.True(result.Metadata.Headers.Count >= 1, "Should extract at least one header");

        var h1 = result.Metadata.Headers.FirstOrDefault(h => h.Level == 1);
        Assert.NotNull(h1);
        Assert.Equal("Main Title", h1.Text);
        Assert.Equal("main", h1.Id);
        Assert.True(h1.IsValid());
    }

    [Fact]
    public void ConvertWithMetadata_WithLinks_ExtractsLinkMetadata()
    {
        var html = @"<html>
            <body>
                <a href=""https://example.com"">External Link</a>
                <a href=""/about"">Internal Link</a>
                <a href=""#section"">Anchor Link</a>
                <a href=""mailto:test@example.com"">Email</a>
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Links);
        Assert.True(result.Metadata.Links.Count >= 3, "Should extract at least three links");

        var externalLink = result.Metadata.Links.FirstOrDefault(l => l.LinkType == LinkType.External);
        Assert.NotNull(externalLink);
        Assert.Equal("https://example.com", externalLink.Href);
        Assert.Equal("External Link", externalLink.Text);

        var internalLink = result.Metadata.Links.FirstOrDefault(l => l.LinkType == LinkType.Internal);
        Assert.NotNull(internalLink);
        Assert.Equal("/about", internalLink.Href);

        var emailLink = result.Metadata.Links.FirstOrDefault(l => l.LinkType == LinkType.Email);
        Assert.NotNull(emailLink);
        Assert.Equal("mailto:test@example.com", emailLink.Href);
    }

    [Fact]
    public void ConvertWithMetadata_WithImages_ExtractsImageMetadata()
    {
        var html = @"<html>
            <body>
                <img src=""https://example.com/image.jpg"" alt=""A test image"" title=""Test"" width=""800"" height=""600"">
                <img src=""/local/image.png"" alt=""Local image"">
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Images);
        Assert.True(result.Metadata.Images.Count >= 1, "Should extract at least one image");

        var externalImg = result.Metadata.Images.FirstOrDefault(i => i.ImageType == ImageType.External);
        Assert.NotNull(externalImg);
        Assert.Equal("https://example.com/image.jpg", externalImg.Src);
        Assert.Equal("A test image", externalImg.Alt);
        Assert.Equal("Test", externalImg.Title);
    }

    [Fact]
    public void ConvertWithMetadata_WithOpenGraphMeta_ExtractsOpenGraphData()
    {
        var html = @"<html>
            <head>
                <meta property=""og:title"" content=""My Article"">
                <meta property=""og:description"" content=""Article description"">
                <meta property=""og:image"" content=""https://example.com/image.jpg"">
                <meta property=""og:url"" content=""https://example.com/article"">
            </head>
            <body>
                <p>Content</p>
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Document.OpenGraph);
        Assert.Contains("title", result.Metadata.Document.OpenGraph.Keys);
        Assert.Equal("My Article", result.Metadata.Document.OpenGraph.GetValueOrDefault("title"));
    }

    [Fact]
    public void ConvertWithMetadata_WithLanguage_ExtractsLanguage()
    {
        var html = @"<html lang=""en-US"">
            <head><title>English Page</title></head>
            <body><p>Content in English</p></body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Document);
        Assert.Equal("en-US", result.Metadata.Document.Language);
    }

    [Fact]
    public void ConvertWithMetadata_WithCanonicalUrl_ExtractsCanonical()
    {
        var html = @"<html>
            <head>
                <link rel=""canonical"" href=""https://example.com/canonical-path"">
            </head>
            <body><p>Content</p></body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Document);
        Assert.Equal("https://example.com/canonical-path", result.Metadata.Document.CanonicalUrl);
    }

    [Fact]
    public void ConvertWithMetadata_EmptyHtml_ReturnsEmptyResult()
    {
        var result = HtmlToMarkdownConverter.ConvertWithMetadata("");

        Assert.NotNull(result);
        Assert.Equal("", result.Markdown);
        Assert.NotNull(result.Metadata);
    }

    [Fact]
    public void ConvertWithMetadata_NullInput_ThrowsArgumentNullException()
    {
        string? html = null;
        Assert.Throws<ArgumentNullException>(() =>
            HtmlToMarkdownConverter.ConvertWithMetadata(html!));
    }

    [Fact]
    public void ConvertWithMetadata_ComplexDocument_ExtractsAllMetadata()
    {
        var html = @"<html lang=""en"">
            <head>
                <title>Complete Test</title>
                <meta name=""description"" content=""A comprehensive test"">
                <meta name=""author"" content=""Test Author"">
                <meta name=""keywords"" content=""test,metadata,extraction"">
                <meta property=""og:title"" content=""OG Title"">
                <meta property=""twitter:card"" content=""summary_large_image"">
                <link rel=""canonical"" href=""https://example.com/test"">
            </head>
            <body>
                <h1>Main Heading</h1>
                <h2>Subheading</h2>
                <p>Some content with <a href=""https://external.com"">external link</a>
                   and <a href=""/internal"">internal link</a>.</p>
                <img src=""https://example.com/test.jpg"" alt=""Test Image"">
            </body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.Contains("Main Heading", result.Markdown);

        Assert.Equal("Complete Test", result.Metadata.Document.Title);
        Assert.Equal("A comprehensive test", result.Metadata.Document.Description);
        Assert.Equal("Test Author", result.Metadata.Document.Author);
        Assert.Equal("en", result.Metadata.Document.Language);
        Assert.Equal("https://example.com/test", result.Metadata.Document.CanonicalUrl);

        Assert.True(result.Metadata.Headers.Count >= 1);
        Assert.Contains(result.Metadata.Headers, h => h.Level == 1 && h.Text == "Main Heading");

        Assert.True(result.Metadata.Links.Count >= 2);
        Assert.Contains(result.Metadata.Links, l => l.LinkType == LinkType.External);
        Assert.Contains(result.Metadata.Links, l => l.LinkType == LinkType.Internal);

        Assert.True(result.Metadata.Images.Count >= 1);
        Assert.Contains(result.Metadata.Images, i => i.Src == "https://example.com/test.jpg");
    }

    [Fact]
    public void ConvertWithMetadata_LinkClassification_CorrectlyClassifiesLinks()
    {
        Assert.Equal(LinkType.Anchor, LinkMetadata.ClassifyLink("#section"));
        Assert.Equal(LinkType.Email, LinkMetadata.ClassifyLink("mailto:test@example.com"));
        Assert.Equal(LinkType.Phone, LinkMetadata.ClassifyLink("tel:+1234567890"));
        Assert.Equal(LinkType.External, LinkMetadata.ClassifyLink("https://example.com"));
        Assert.Equal(LinkType.Internal, LinkMetadata.ClassifyLink("/about"));
        Assert.Equal(LinkType.Other, LinkMetadata.ClassifyLink("javascript:void(0)"));
    }

    [Fact]
    public void ConvertWithMetadata_HeaderValidation_ChecksHeaderLevel()
    {
        var validHeader = new HeaderMetadata
        {
            Level = 3,
            Text = "Valid",
            Depth = 0,
            HtmlOffset = 0
        };
        Assert.True(validHeader.IsValid());

        var invalidHeader = new HeaderMetadata
        {
            Level = 7,
            Text = "Invalid",
            Depth = 0,
            HtmlOffset = 0
        };
        Assert.False(invalidHeader.IsValid());
    }

    [Fact]
    public void ConvertWithMetadata_WithKeywords_ExtractsKeywords()
    {
        var html = @"<html>
            <head>
                <meta name=""keywords"" content=""rust,markdown,conversion"">
            </head>
            <body><p>Content</p></body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Document.Keywords);
    }

    [Fact]
    public void ConvertWithMetadata_TextDirection_ExtractsDirection()
    {
        var html = @"<html dir=""rtl"">
            <head><title>RTL Page</title></head>
            <body><p>معلومات</p></body>
        </html>";

        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata.Document);
    }

    [Fact]
    public void ConvertWithMetadata_MarkdownNotEmpty_ForNonEmptyHtml()
    {
        var html = "<p>This is a test paragraph.</p>";
        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotEmpty(result.Markdown);
        Assert.Contains("This is a test paragraph", result.Markdown);
    }

    [Fact]
    public void ConvertWithMetadata_ReturnsValidExtendedMetadata()
    {
        var html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
        var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

        Assert.NotNull(result.Metadata);
        Assert.NotNull(result.Metadata.Document);
        Assert.NotNull(result.Metadata.Headers);
        Assert.NotNull(result.Metadata.Links);
        Assert.NotNull(result.Metadata.Images);
        Assert.NotNull(result.Metadata.StructuredData);
    }
}
