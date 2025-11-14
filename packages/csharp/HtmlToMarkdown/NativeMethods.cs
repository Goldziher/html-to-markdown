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
}
