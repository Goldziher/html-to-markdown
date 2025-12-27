using System.Runtime.InteropServices;
using System.Text.Json;
using HtmlToMarkdown.Metadata;
using HtmlToMarkdown.Serialization;
using HtmlToMarkdown.Visitor;

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
        nuint resultLen = 0;

        try
        {
            htmlPtr = StringToUtf8Ptr(html);

            resultPtr = NativeMethods.html_to_markdown_convert_with_len(htmlPtr, out resultLen);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion failed");
            }

            return PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
        }
        finally
        {
            if (htmlPtr != IntPtr.Zero)
            {
                Marshal.FreeCoTaskMem(htmlPtr);
            }

            if (resultPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(resultPtr);
            }
        }
    }

    /// <summary>
    /// Converts UTF-8 HTML bytes to Markdown using default options.
    /// </summary>
    /// <param name="html">UTF-8 encoded HTML bytes</param>
    /// <returns>The converted Markdown string</returns>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion fails</exception>
    public static unsafe string Convert(ReadOnlySpan<byte> html)
    {
        if (html.IsEmpty)
        {
            return string.Empty;
        }

        IntPtr resultPtr = IntPtr.Zero;
        nuint resultLen = 0;

        fixed (byte* htmlPtr = html)
        {
            resultPtr = NativeMethods.html_to_markdown_convert_bytes_with_len(
                (IntPtr)htmlPtr,
                (nuint)html.Length,
                out resultLen);
        }

        if (resultPtr == IntPtr.Zero)
        {
            IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
            string? errorMsg = errorPtr != IntPtr.Zero
                ? PtrToStringUtf8(errorPtr)
                : null;

            throw new HtmlToMarkdownException(
                errorMsg ?? "HTML to Markdown conversion failed");
        }

        try
        {
            return PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
        }
        finally
        {
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
            ? PtrToStringUtf8(versionPtr) ?? "unknown"
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
            outputPtr = StringToUtf8Ptr(outputPath);
            bool ok = NativeMethods.html_to_markdown_profile_start(outputPtr, frequency);
            if (!ok)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;
                throw new HtmlToMarkdownException(errorMsg ?? "Profiling start failed");
            }
        }
        finally
        {
            if (outputPtr != IntPtr.Zero)
            {
                Marshal.FreeCoTaskMem(outputPtr);
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
                ? PtrToStringUtf8(errorPtr)
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
        nuint resultLen = 0;
        nuint metadataLen = 0;

        try
        {
            htmlPtr = StringToUtf8Ptr(html);

            resultPtr = NativeMethods.html_to_markdown_convert_with_metadata_with_len(
                htmlPtr,
                out metadataPtr,
                out resultLen,
                out metadataLen);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion with metadata failed");
            }

            string markdown = PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
            ExtendedMetadata metadata = DeserializeMetadata(metadataPtr, metadataLen);

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
                Marshal.FreeCoTaskMem(htmlPtr);
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

    /// <summary>
    /// Converts UTF-8 HTML bytes to Markdown and extracts comprehensive metadata.
    /// </summary>
    /// <param name="html">UTF-8 encoded HTML bytes</param>
    /// <returns>A MetadataExtraction result containing both markdown and extracted metadata</returns>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion or metadata extraction fails</exception>
    /// <exception cref="JsonException">Thrown when metadata JSON deserialization fails</exception>
    public static unsafe MetadataExtraction ConvertWithMetadata(ReadOnlySpan<byte> html)
    {
        if (html.IsEmpty)
        {
            return new MetadataExtraction
            {
                Markdown = string.Empty,
                Metadata = new ExtendedMetadata()
            };
        }

        IntPtr resultPtr = IntPtr.Zero;
        IntPtr metadataPtr = IntPtr.Zero;
        nuint resultLen = 0;
        nuint metadataLen = 0;

        fixed (byte* htmlPtr = html)
        {
            resultPtr = NativeMethods.html_to_markdown_convert_with_metadata_bytes_with_len(
                (IntPtr)htmlPtr,
                (nuint)html.Length,
                out metadataPtr,
                out resultLen,
                out metadataLen);
        }

        if (resultPtr == IntPtr.Zero)
        {
            IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
            string? errorMsg = errorPtr != IntPtr.Zero
                ? PtrToStringUtf8(errorPtr)
                : null;

            throw new HtmlToMarkdownException(
                errorMsg ?? "HTML to Markdown conversion with metadata failed");
        }

        try
        {
            string markdown = PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
            ExtendedMetadata metadata = DeserializeMetadata(metadataPtr, metadataLen);

            return new MetadataExtraction
            {
                Markdown = markdown,
                Metadata = metadata
            };
        }
        finally
        {
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

    /// <summary>
    /// Converts HTML to Markdown and returns metadata JSON without deserializing it.
    /// </summary>
    /// <param name="html">The HTML string to convert</param>
    /// <returns>A MetadataExtractionJson result containing markdown and raw metadata JSON</returns>
    /// <exception cref="ArgumentNullException">Thrown when html is null</exception>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion or metadata extraction fails</exception>
    public static MetadataExtractionJson ConvertWithMetadataJson(string html)
    {
        if (html == null)
        {
            throw new ArgumentNullException(nameof(html));
        }

        if (string.IsNullOrEmpty(html))
        {
            return new MetadataExtractionJson
            {
                Markdown = string.Empty,
                MetadataJson = string.Empty
            };
        }

        IntPtr htmlPtr = IntPtr.Zero;
        IntPtr resultPtr = IntPtr.Zero;
        IntPtr metadataPtr = IntPtr.Zero;
        nuint resultLen = 0;
        nuint metadataLen = 0;

        try
        {
            htmlPtr = StringToUtf8Ptr(html);

            resultPtr = NativeMethods.html_to_markdown_convert_with_metadata_with_len(
                htmlPtr,
                out metadataPtr,
                out resultLen,
                out metadataLen);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion with metadata failed");
            }

            string markdown = PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
            string metadataJson = PtrToStringUtf8(metadataPtr, metadataLen) ?? string.Empty;

            return new MetadataExtractionJson
            {
                Markdown = markdown,
                MetadataJson = metadataJson
            };
        }
        finally
        {
            if (htmlPtr != IntPtr.Zero)
            {
                Marshal.FreeCoTaskMem(htmlPtr);
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

    /// <summary>
    /// Converts UTF-8 HTML bytes to Markdown and returns metadata JSON without deserializing it.
    /// </summary>
    /// <param name="html">UTF-8 encoded HTML bytes</param>
    /// <returns>A MetadataExtractionJson result containing markdown and raw metadata JSON</returns>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion or metadata extraction fails</exception>
    public static unsafe MetadataExtractionJson ConvertWithMetadataJson(ReadOnlySpan<byte> html)
    {
        if (html.IsEmpty)
        {
            return new MetadataExtractionJson
            {
                Markdown = string.Empty,
                MetadataJson = string.Empty
            };
        }

        IntPtr resultPtr = IntPtr.Zero;
        IntPtr metadataPtr = IntPtr.Zero;
        nuint resultLen = 0;
        nuint metadataLen = 0;

        fixed (byte* htmlPtr = html)
        {
            resultPtr = NativeMethods.html_to_markdown_convert_with_metadata_bytes_with_len(
                (IntPtr)htmlPtr,
                (nuint)html.Length,
                out metadataPtr,
                out resultLen,
                out metadataLen);
        }

        if (resultPtr == IntPtr.Zero)
        {
            IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
            string? errorMsg = errorPtr != IntPtr.Zero
                ? PtrToStringUtf8(errorPtr)
                : null;

            throw new HtmlToMarkdownException(
                errorMsg ?? "HTML to Markdown conversion with metadata failed");
        }

        try
        {
            string markdown = PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
            string metadataJson = PtrToStringUtf8(metadataPtr, metadataLen) ?? string.Empty;

            return new MetadataExtractionJson
            {
                Markdown = markdown,
                MetadataJson = metadataJson
            };
        }
        finally
        {
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

    /// <summary>
    /// Converts HTML to Markdown using a custom visitor pattern.
    /// </summary>
    /// <param name="html">The HTML string to convert</param>
    /// <param name="visitor">The custom visitor implementation</param>
    /// <returns>The converted Markdown string</returns>
    /// <exception cref="ArgumentNullException">Thrown when html or visitor is null</exception>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion fails</exception>
    public static string ConvertWithVisitor(string html, IVisitor visitor)
    {
        if (html == null)
        {
            throw new ArgumentNullException(nameof(html));
        }

        if (visitor == null)
        {
            throw new ArgumentNullException(nameof(visitor));
        }

        if (string.IsNullOrEmpty(html))
        {
            return string.Empty;
        }

        var bridge = new VisitorBridge(visitor);
        IntPtr htmlPtr = IntPtr.Zero;
        IntPtr resultPtr = IntPtr.Zero;
        nuint resultLen = 0;

        try
        {
            // Create native visitor
            var nativeVisitorHandle = bridge.CreateNativeVisitor();

            // Convert HTML to UTF-8 ptr
            htmlPtr = StringToUtf8Ptr(html);

            // Call FFI conversion with visitor
            resultPtr = NativeMethods.html_to_markdown_convert_with_visitor(
                htmlPtr,
                nativeVisitorHandle,
                out resultLen);

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion with visitor failed");
            }

            return PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
        }
        finally
        {
            if (htmlPtr != IntPtr.Zero)
            {
                Marshal.FreeCoTaskMem(htmlPtr);
            }

            if (resultPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(resultPtr);
            }

            bridge.Dispose();
        }
    }

    /// <summary>
    /// Converts UTF-8 HTML bytes to Markdown using a custom visitor pattern.
    /// </summary>
    /// <param name="html">UTF-8 encoded HTML bytes</param>
    /// <param name="visitor">The custom visitor implementation</param>
    /// <returns>The converted Markdown string</returns>
    /// <exception cref="ArgumentNullException">Thrown when visitor is null</exception>
    /// <exception cref="HtmlToMarkdownException">Thrown when conversion fails</exception>
    public static unsafe string ConvertWithVisitor(ReadOnlySpan<byte> html, IVisitor visitor)
    {
        if (visitor == null)
        {
            throw new ArgumentNullException(nameof(visitor));
        }

        if (html.IsEmpty)
        {
            return string.Empty;
        }

        var bridge = new VisitorBridge(visitor);
        IntPtr resultPtr = IntPtr.Zero;
        nuint resultLen = 0;

        try
        {
            // Create native visitor
            var nativeVisitorHandle = bridge.CreateNativeVisitor();

            // Call FFI conversion with visitor
            fixed (byte* htmlPtr = html)
            {
                resultPtr = NativeMethods.html_to_markdown_convert_bytes_with_visitor(
                    (IntPtr)htmlPtr,
                    (nuint)html.Length,
                    nativeVisitorHandle,
                    out resultLen);
            }

            if (resultPtr == IntPtr.Zero)
            {
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? PtrToStringUtf8(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion with visitor failed");
            }

            return PtrToStringUtf8(resultPtr, resultLen) ?? string.Empty;
        }
        finally
        {
            if (resultPtr != IntPtr.Zero)
            {
                NativeMethods.html_to_markdown_free_string(resultPtr);
            }

            bridge.Dispose();
        }
    }

    private static ExtendedMetadata DeserializeMetadata(IntPtr metadataPtr, nuint metadataLen)
    {
        if (metadataPtr == IntPtr.Zero || metadataLen == 0)
        {
            return new ExtendedMetadata();
        }

        try
        {
            return DeserializeMetadataUtf8(metadataPtr, metadataLen) ?? new ExtendedMetadata();
        }
        catch (JsonException ex)
        {
            throw new HtmlToMarkdownException(
                $"Failed to deserialize metadata JSON: {ex.Message}", ex);
        }
    }

    private static string? PtrToStringUtf8(IntPtr ptr)
    {
        return ptr == IntPtr.Zero ? null : Marshal.PtrToStringUTF8(ptr);
    }

    private static string? PtrToStringUtf8(IntPtr ptr, nuint length)
    {
        if (ptr == IntPtr.Zero || length == 0)
        {
            return null;
        }

        int normalizedLength = NormalizeLength(ptr, length, "Converted string exceeds maximum length.");
        if (normalizedLength == 0)
        {
            return string.Empty;
        }

        return Marshal.PtrToStringUTF8(ptr, normalizedLength);
    }

    private static IntPtr StringToUtf8Ptr(string value)
    {
        return Marshal.StringToCoTaskMemUTF8(value);
    }

    private static unsafe ExtendedMetadata? DeserializeMetadataUtf8(IntPtr metadataPtr, nuint metadataLen)
    {
        byte* data = (byte*)metadataPtr;
        if (data == null)
        {
            return null;
        }

        int length = NormalizeLength(metadataPtr, metadataLen, "Metadata JSON exceeds maximum length.");
        if (length == 0)
        {
            return null;
        }
        ReadOnlySpan<byte> json = new ReadOnlySpan<byte>(data, length);
        return JsonSerializer.Deserialize(json, MetadataJsonContext.Default.ExtendedMetadata);
    }

    private static unsafe int NormalizeLength(IntPtr ptr, nuint length, string overflowMessage)
    {
        if (length == 0)
        {
            return 0;
        }

        if (length > int.MaxValue)
        {
            throw new HtmlToMarkdownException(overflowMessage);
        }

        byte* data = (byte*)ptr;
        if (data != null && data[length - 1] == 0)
        {
            length--;
        }

        return (int)length;
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
