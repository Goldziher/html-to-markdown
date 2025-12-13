using HtmlToMarkdown;

Console.WriteLine($"html-to-markdown version: {HtmlToMarkdownConverter.GetVersion()}\n");

// Test 1: Simple conversion
Console.WriteLine("Test 1: Simple HTML");
var html1 = "<h1>Hello World</h1><p>This is a test.</p>";
Console.WriteLine($"Input:  {html1}");
var markdown1 = HtmlToMarkdownConverter.Convert(html1);
Console.WriteLine($"Output:\n{markdown1}\n");

Console.WriteLine("Test 2: Complex HTML");
var html2 = @"
<html>
    <head><title>Test Page</title></head>
    <body>
        <h1>Main Title</h1>
        <p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
        <ul>
            <li>First item</li>
            <li>Second item</li>
            <li>Third item</li>
        </ul>
        <a href=""https://example.com"">Example link</a>
    </body>
</html>";
var markdown2 = HtmlToMarkdownConverter.Convert(html2);
Console.WriteLine($"Output:\n{markdown2}\n");

Console.WriteLine("Test 3: Empty string");
var markdown3 = HtmlToMarkdownConverter.Convert("");
if (markdown3 == "")
{
    Console.WriteLine("✓ Empty string correctly returns empty result\n");
}
else
{
    Console.WriteLine($"✗ Expected empty result, got: {markdown3}\n");
    return 1;
}

Console.WriteLine("Test 4: Validation checks");
var html4 = @"
<h1>Heading 1</h1>
<h2>Heading 2</h2>
<p>Regular <strong>bold</strong> and <em>italic</em> text.</p>
<ul>
    <li>Bullet 1</li>
    <li>Bullet 2</li>
</ul>";
var markdown4 = HtmlToMarkdownConverter.Convert(html4);

var checks = new[] { "Heading 1", "Heading 2", "bold", "italic", "Bullet 1", "Bullet 2" };
bool allPassed = true;

foreach (var check in checks)
{
    if (!markdown4.Contains(check))
    {
        Console.WriteLine($"✗ Output missing expected text: {check}");
        allPassed = false;
    }
}

if (allPassed)
{
    Console.WriteLine("✓ All validation checks passed");
}
else
{
    Console.WriteLine("Output was:");
    Console.WriteLine(markdown4);
    return 1;
}

Console.WriteLine("\n✅ All smoke tests passed!");
return 0;
