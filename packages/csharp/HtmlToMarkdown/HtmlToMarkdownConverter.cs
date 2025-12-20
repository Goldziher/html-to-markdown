using System.Runtime.InteropServices;
using System.Text.Json;
using HtmlToMarkdown.Metadata;

namespace HtmlToMarkdown;

/// <summary>
/// High-performance HTML to Markdown converter with .NET bindings to Rust core.
/// </summary>
public static class HtmlToMarkdownConverter
{
    /// <summary>
    /// Converts HTML to Markdown using default options.
    /// </summary>
    /// <param name="html">The HTML string to convert</param>
    /// <returns>The converted Markdown string</returns>
    /// <exception cref="ArgumentNullException">Thrown when html is null</exception>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion fails</exception>
    public static string Convert(string html)
    {
        if (html == null)
        {
            throw new ArgumentNullException(nameof(html));
        }

        if (string.IsNullOrEmpty(html))
        {
            return string.Empty;
        }

        IntPtr htmlPtr = IntPtr.Zero;
        IntPtr resultPtr = IntPtr.Zero;

        try
        {
            htmlPtr = Marshal.StringToHGlobalAnsi(html);

            resultPtr = NativeMethods.html_to_markdown_convert(htmlPtr);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringAnsi(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion failed");
            }

            string? markdown = Marshal.PtrToStringAnsi(resultPtr);
            return markdown ?? string.Empty;
        }
        finally
        {
            if (htmlPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(htmlPtr);
            }

            if (resultPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(resultPtr);
            }
        }
    }

    /// <summary>
    /// Gets the version of the underlying html-to-markdown library.
    /// </summary>
    /// <returns>The library version string</returns>
    public static string GetVersion()
    {
        IntPtr versionPtr = NativeMethods.html_to_markdown_version();
        return versionPtr != IntPtr.Zero
            ? Marshal.PtrToStringAnsi(versionPtr) ?? "unknown"
            : "unknown";
    }

    /// <summary>
    /// Start Rust-side profiling and write a flamegraph to the given output path.
    /// </summary>
    public static void StartProfiling(string outputPath, int frequency = 1000)
    {
        if (string.IsNullOrWhiteSpace(outputPath))
        {
            throw new ArgumentException("outputPath is required", nameof(outputPath));
        }

        if (frequency <= 0)
        {
            frequency = 1000;
        }

        IntPtr outputPtr = IntPtr.Zero;

        try
        {
            outputPtr = Marshal.StringToHGlobalAnsi(outputPath);
            bool ok = NativeMethods.html_to_markdown_profile_start(outputPtr, frequency);
            if (!ok)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringAnsi(errorPtr)
                    : null;
                throw new HtmlToMarkdownException(errorMsg ?? "Profiling start failed");
            }
        }
        finally
        {
            if (outputPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(outputPtr);
            }
        }
    }

    /// <summary>
    /// Stop Rust-side profiling and flush the flamegraph to disk.
    /// </summary>
    public static void StopProfiling()
    {
        bool ok = NativeMethods.html_to_markdown_profile_stop();
        if (!ok)
        {
            IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
            string? errorMsg = errorPtr != IntPtr.Zero
                ? Marshal.PtrToStringAnsi(errorPtr)
                : null;
            throw new HtmlToMarkdownException(errorMsg ?? "Profiling stop failed");
        }
    }

    /// <summary>
    /// Converts HTML to Markdown and extracts comprehensive metadata.
    /// </summary>
    /// <param name="html">The HTML string to convert</param>
    /// <returns>A MetadataExtraction result containing both markdown and extracted metadata</returns>
    /// <exception cref="ArgumentNullException">Thrown when html is null</exception>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion or metadata extraction fails</exception>
    /// <exception cref="JsonException">Thrown when metadata JSON deserialization fails</exception>
    /// <remarks>
    /// This method extracts document metadata, headers, links, images, and structured data
    /// from the HTML document in a single pass. All metadata types are included in the result.
    /// </remarks>
    public static MetadataExtraction ConvertWithMetadata(string html)
    {
        if (html == null)
        {
            throw new ArgumentNullException(nameof(html));
        }

        if (string.IsNullOrEmpty(html))
        {
            return new MetadataExtraction
            {
                Markdown = string.Empty,
                Metadata = new ExtendedMetadata()
            };
        }

        IntPtr htmlPtr = IntPtr.Zero;
        IntPtr resultPtr = IntPtr.Zero;
        IntPtr metadataPtr = IntPtr.Zero;

        try
        {
            htmlPtr = Marshal.StringToHGlobalAnsi(html);

            resultPtr = NativeMethods.html_to_markdown_convert_with_metadata(htmlPtr, out metadataPtr);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringAnsi(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion with metadata failed");
            }

            string? markdown = Marshal.PtrToStringAnsi(resultPtr) ?? string.Empty;

            ExtendedMetadata metadata = new();
            if (metadataPtr != IntPtr.Zero)
            {
                string? metadataJson = Marshal.PtrToStringAnsi(metadataPtr);
                if (!string.IsNullOrEmpty(metadataJson))
                {
                    try
                    {
                        var options = new JsonSerializerOptions
                        {
                            PropertyNameCaseInsensitive = true,
                            DefaultBufferSize = 16384
                        };
                        var deserializedMetadata = JsonSerializer.Deserialize<ExtendedMetadata>(metadataJson, options);
                        if (deserializedMetadata != null)
                        {
                            metadata = deserializedMetadata;
                        }
                    }
                    catch (JsonException ex)
                    {
                        throw new HtmlToMarkdownException(
                            $"Failed to deserialize metadata JSON: {ex.Message}", ex);
                    }
                }
            }

            return new MetadataExtraction
            {
                Markdown = markdown,
                Metadata = metadata
            };
        }
        finally
        {
            if (htmlPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(htmlPtr);
            }

            if (resultPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(resultPtr);
            }

            if (metadataPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(metadataPtr);
            }
        }
    }
}

/// <summary>
/// Exception thrown when HTML to Markdown conversion fails.
/// </summary>
public class HtmlToMarkdownException : Exception
{
    public HtmlToMarkdownException(string message) : base(message) { }

    public HtmlToMarkdownException(string message, Exception innerException)
        : base(message, innerException) { }
}
