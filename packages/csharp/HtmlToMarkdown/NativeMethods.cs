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
    /// Convert HTML to Markdown and return the output length.
    /// </summary>
    /// <param name="html">Null-terminated HTML string</param>
    /// <param name="len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_convert_with_len(
        IntPtr html,
        out nuint len_out);

    /// <summary>
    /// Convert UTF-8 HTML bytes to Markdown and return the output length.
    /// </summary>
    /// <param name="html">Pointer to UTF-8 bytes</param>
    /// <param name="html_len">Length of UTF-8 bytes</param>
    /// <param name="len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr html_to_markdown_convert_bytes_with_len(
        IntPtr html,
        nuint html_len,
        out nuint len_out);

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
    /// Convert HTML to Markdown with metadata extraction and return output lengths.
    /// </summary>
    /// <param name="html">Null-terminated HTML string</param>
    /// <param name="metadata_json_out">Pointer to char pointer for metadata JSON output</param>
    /// <param name="markdown_len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <param name="metadata_len_out">Output length of metadata JSON bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_convert_with_metadata_with_len(
        IntPtr html,
        out IntPtr metadata_json_out,
        out nuint markdown_len_out,
        out nuint metadata_len_out);

    /// <summary>
    /// Convert UTF-8 HTML bytes to Markdown with metadata extraction and return output lengths.
    /// </summary>
    /// <param name="html">Pointer to UTF-8 bytes</param>
    /// <param name="html_len">Length of UTF-8 bytes</param>
    /// <param name="metadata_json_out">Pointer to char pointer for metadata JSON output</param>
    /// <param name="markdown_len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <param name="metadata_len_out">Output length of metadata JSON bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr html_to_markdown_convert_with_metadata_bytes_with_len(
        IntPtr html,
        nuint html_len,
        out IntPtr metadata_json_out,
        out nuint markdown_len_out,
        out nuint metadata_len_out);

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

    // === Visitor Pattern API ===

    /// <summary>
    /// Create a visitor instance from a visitor callbacks structure.
    /// </summary>
    /// <param name="visitor">Pointer to visitor callbacks structure</param>
    /// <returns>Opaque visitor handle, or IntPtr.Zero on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr html_to_markdown_visitor_create(IntPtr visitor);

    /// <summary>
    /// Free a visitor instance.
    /// </summary>
    /// <param name="visitor">Opaque visitor handle from html_to_markdown_visitor_create</param>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void html_to_markdown_visitor_free(IntPtr visitor);

    /// <summary>
    /// Convert HTML using a visitor pattern.
    /// </summary>
    /// <param name="html">Null-terminated HTML string</param>
    /// <param name="visitor">Opaque visitor handle</param>
    /// <param name="len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr html_to_markdown_convert_with_visitor(
        IntPtr html,
        IntPtr visitor,
        out nuint len_out);

    /// <summary>
    /// Convert UTF-8 HTML bytes using a visitor pattern.
    /// </summary>
    /// <param name="html">Pointer to UTF-8 bytes</param>
    /// <param name="html_len">Length of UTF-8 bytes</param>
    /// <param name="visitor">Opaque visitor handle</param>
    /// <param name="len_out">Output length of markdown bytes (excluding null terminator)</param>
    /// <returns>Pointer to null-terminated Markdown string, or NULL on error</returns>
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr html_to_markdown_convert_bytes_with_visitor(
        IntPtr html,
        nuint html_len,
        IntPtr visitor,
        out nuint len_out);
}
