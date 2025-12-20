using System.Runtime.InteropServices;

namespace HtmlToMarkdown;

/// <summary>
/// P/Invoke declarations for the native html-to-markdown library.
/// </summary>
internal static class NativeMethods
{
    private const string LibraryName = "html_to_markdown_ffi";

    /// <summary>
    /// Convert HTML to Markdown using default options.
    /// </summary>
    /// <param name="html">Null-terminated HTML string</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_convert(IntPtr html);

    /// <summary>
    /// Free a string returned by html_to_markdown_convert.
    /// </summary>
    /// <param name="ptr">Pointer to string to free</param>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void html_to_markdown_free_string(IntPtr ptr);

    /// <summary>
    /// Get the library version string.
    /// </summary>
    /// <returns>Pointer to static null-terminated version string</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_version();

    /// <summary>
    /// Get the last error message.
    /// </summary>
    /// <returns>Pointer to static null-terminated error string, or NULL</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_last_error();

    /// <summary>
    /// Convert HTML to Markdown with metadata extraction.
    /// </summary>
    /// <param name="html">Null-terminated HTML string</param>
    /// <param name="metadata_json_out">Pointer to char pointer for metadata JSON output</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    /// <remarks>
    /// Both the returned markdown string and the metadata JSON string (written to metadata_json_out)
    /// must be freed with html_to_markdown_free_string.
    /// </remarks>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_convert_with_metadata(
        IntPtr html,
        out IntPtr metadata_json_out);

    /// <summary>
    /// Start Rust-side profiling and write a flamegraph to the given path.
    /// </summary>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern bool html_to_markdown_profile_start(IntPtr outputPath, int frequency);

    /// <summary>
    /// Stop Rust-side profiling and flush the flamegraph.
    /// </summary>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern bool html_to_markdown_profile_stop();
}
