using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Newtonsoft.Json;
using Xunit;
using HtmlToMarkdown;

namespace HtmlToMarkdownTestApp;

#nullable enable

public record TestCase(string Name, string Html, string ExpectedMarkdown, Dictionary<string, object>? Options);

public class ComprehensiveTest
{
    private static List<TestCase> LoadFixtures(string filename)
    {
        // Get the directory of the test_apps folder (parent of csharp folder)
        var testAppsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..");
        var fixturesPath = Path.Combine(testAppsDir, "fixtures", filename);
        var fullPath = Path.GetFullPath(fixturesPath);
        var json = File.ReadAllText(fullPath);
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
        var result = HtmlToMarkdownConverter.Convert(testCase.Html);
        Assert.Equal(testCase.ExpectedMarkdown.Trim(), result.Trim());
    }
}
