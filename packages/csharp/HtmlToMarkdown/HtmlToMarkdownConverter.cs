using System.Runtime.InteropServices;

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
            // Convert managed string to unmanaged C string
            htmlPtr = Marshal.StringToHGlobalAnsi(html);

            // Call native conversion function
            resultPtr = NativeMethods.html_to_markdown_convert(htmlPtr);

            if (resultPtr == IntPtr.Zero)
            {
                // Conversion failed - try to get error message
                IntPtr errorPtr = NativeMethods.html_to_markdown_last_error();
                string? errorMsg = errorPtr != IntPtr.Zero
                    ? Marshal.PtrToStringAnsi(errorPtr)
                    : null;

                throw new HtmlToMarkdownException(
                    errorMsg ?? "HTML to Markdown conversion failed");
            }

            // Convert result back to managed string
            string? markdown = Marshal.PtrToStringAnsi(resultPtr);
            return markdown ?? string.Empty;
        }
        finally
        {
            // Free unmanaged memory
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
