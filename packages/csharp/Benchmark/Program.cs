using System;
using System.Diagnostics;
using System.IO;
using System.Text.Json;
using HtmlToMarkdown;

internal class Program
{
    private static void RunScenario(byte[] html, string scenario)
    {
        if (scenario == "metadata-default")
        {
            HtmlToMarkdownConverter.ConvertWithMetadata(html);
        }
        else if (scenario == "metadata-raw")
        {
            HtmlToMarkdownConverter.ConvertWithMetadataJson(html);
        }
        else
        {
            HtmlToMarkdownConverter.Convert(html);
        }
    }

    private static void Main(string[] args)
    {
        string? filePath = null;
        int iterations = 50;
        string format = "html";
        string scenario = "convert-default";

        for (int i = 0; i < args.Length; i++)
        {
            switch (args[i])
            {
                case "--file":
                    if (i + 1 < args.Length)
                    {
                        filePath = args[++i];
                    }
                    break;
                case "--iterations":
                    if (i + 1 < args.Length)
                    {
                        iterations = int.Parse(args[++i]);
                    }
                    break;
                case "--format":
                    if (i + 1 < args.Length)
                    {
                        format = args[++i];
                    }
                    break;
                case "--scenario":
                    if (i + 1 < args.Length)
                    {
                        scenario = args[++i];
                    }
                    break;
            }
        }

        if (filePath == null)
        {
            Console.Error.WriteLine("Error: --file is required");
            Environment.Exit(1);
        }

        if (scenario != "convert-default" && scenario != "metadata-default" && scenario != "metadata-raw")
        {
            Console.Error.WriteLine($"Unsupported scenario: {scenario}");
            Environment.Exit(1);
        }

        try
        {
            byte[] html = File.ReadAllBytes(filePath);

            RunScenario(html, scenario);

            string? profileOutput = Environment.GetEnvironmentVariable("HTML_TO_MARKDOWN_PROFILE_OUTPUT");
            if (!string.IsNullOrWhiteSpace(profileOutput))
            {
                string? freqEnv = Environment.GetEnvironmentVariable("HTML_TO_MARKDOWN_PROFILE_FREQUENCY");
                int frequency = 1000;
                if (!string.IsNullOrWhiteSpace(freqEnv) && int.TryParse(freqEnv, out int parsed))
                {
                    frequency = parsed;
                }
                HtmlToMarkdownConverter.StartProfiling(profileOutput, frequency);
            }

            var stopwatch = Stopwatch.StartNew();
            for (int i = 0; i < iterations; i++)
            {
                RunScenario(html, scenario);
            }
            stopwatch.Stop();

            if (!string.IsNullOrWhiteSpace(profileOutput))
            {
                HtmlToMarkdownConverter.StopProfiling();
            }

            double elapsedSeconds = stopwatch.Elapsed.TotalSeconds;
            int bytesProcessed = html.Length * iterations;
            double opsPerSec = iterations / elapsedSeconds;
            double mbPerSec = (bytesProcessed / (1024.0 * 1024.0)) / elapsedSeconds;

            var result = new
            {
                language = "csharp",
                fixture = Path.GetFileName(filePath),
                fixture_path = filePath,
                scenario,
                iterations,
                elapsed_seconds = elapsedSeconds,
                ops_per_sec = opsPerSec,
                mb_per_sec = mbPerSec,
                bytes_processed = bytesProcessed
            };

            string json = JsonSerializer.Serialize(result);
            Console.WriteLine(json);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Error: {ex.Message}");
            Environment.Exit(1);
        }
    }
}
