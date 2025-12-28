using Newtonsoft.Json;
using Xunit;
using HtmlToMarkdown;

namespace HtmlToMarkdownTestApp;

public record TestCase(string Name, string Html, string ExpectedMarkdown, Dictionary<string, object>? Options);

public class ComprehensiveTest
{
    private static List<TestCase> LoadFixtures(string filename)
    {
        var path = Path.Combine("..", "..", "fixtures", filename);
        var json = File.ReadAllText(path);
        return JsonConvert.DeserializeObject<List<TestCase>>(json)!;
    }

    public static IEnumerable<object[]> BasicHtmlData()
    {
        var fixtures = LoadFixtures("basic-html.json");
        return fixtures.Select(f => new object[] { f });
    }

    [Theory]
    [MemberData(nameof(BasicHtmlData))]
    public void TestBasicHtmlConversion(TestCase testCase)
    {
        var result = Converter.Convert(testCase.Html);
        Assert.Equal(testCase.ExpectedMarkdown.Trim(), result.Trim());
    }
}
