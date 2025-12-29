package dev.kreuzberg.htmltomarkdown;

import dev.kreuzberg.htmltomarkdown.util.StringUtils;
import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

/**
 * Low-level Foreign Function Interface (FFI) bindings to the native html-to-markdown library.
 *
 * <p>This class provides direct access to the C FFI functions using Java's Foreign Function &amp;
 * Memory API (Panama). For a higher-level, more ergonomic API, use {@link HtmlToMarkdown} instead.
 *
 * @since 2.7.3
 */
final class HtmlToMarkdownFFI {

  /** Library name constant. */
  private static final String LIBRARY_NAME = "html_to_markdown_ffi";

  /** Native linker instance. */
  private static final Linker LINKER = Linker.nativeLinker();

  /** Symbol lookup for native functions. */
  private static final SymbolLookup SYMBOL_LOOKUP;

  /** Function descriptor for convert. */
  private static final FunctionDescriptor CONVERT_DESC =
      FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS);

  /** Function descriptor for free string. */
  private static final FunctionDescriptor FREE_STRING_DESC =
      FunctionDescriptor.ofVoid(ValueLayout.ADDRESS);

  /** Function descriptor for version. */
  private static final FunctionDescriptor VERSION_DESC = FunctionDescriptor.of(ValueLayout.ADDRESS);

  /** Function descriptor for last error. */
  private static final FunctionDescriptor LAST_ERROR_DESC =
      FunctionDescriptor.of(ValueLayout.ADDRESS);

  /** Function descriptor for convert with metadata. */
  private static final FunctionDescriptor CONVERT_WITH_METADATA_DESC =
      FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS);

  /** Function descriptor for profile start. */
  private static final FunctionDescriptor PROFILE_START_DESC =
      FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS, ValueLayout.JAVA_INT);

  /** Function descriptor for profile stop. */
  private static final FunctionDescriptor PROFILE_STOP_DESC =
      FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN);

  /** Method handle for html_to_markdown_convert. */
  static final MethodHandle html_to_markdown_convert;

  /** Method handle for html_to_markdown_free_string. */
  static final MethodHandle html_to_markdown_free_string;

  /** Method handle for html_to_markdown_version. */
  static final MethodHandle html_to_markdown_version;

  /** Method handle for html_to_markdown_last_error. */
  static final MethodHandle html_to_markdown_last_error;

  /** Method handle for html_to_markdown_convert_with_metadata. */
  static final MethodHandle html_to_markdown_convert_with_metadata;

  /** Method handle for html_to_markdown_profile_start. */
  static final MethodHandle html_to_markdown_profile_start;

  /** Method handle for html_to_markdown_profile_stop. */
  static final MethodHandle html_to_markdown_profile_stop;

  static {
    System.loadLibrary(LIBRARY_NAME);

    SYMBOL_LOOKUP = SymbolLookup.loaderLookup();

    html_to_markdown_convert =
        LINKER.downcallHandle(findSymbol("html_to_markdown_convert"), CONVERT_DESC);

    html_to_markdown_free_string =
        LINKER.downcallHandle(findSymbol("html_to_markdown_free_string"), FREE_STRING_DESC);

    html_to_markdown_version =
        LINKER.downcallHandle(findSymbol("html_to_markdown_version"), VERSION_DESC);

    html_to_markdown_last_error =
        LINKER.downcallHandle(findSymbol("html_to_markdown_last_error"), LAST_ERROR_DESC);

    html_to_markdown_convert_with_metadata =
        LINKER.downcallHandle(
            findSymbol("html_to_markdown_convert_with_metadata"), CONVERT_WITH_METADATA_DESC);

    html_to_markdown_profile_start =
        LINKER.downcallHandle(findSymbol("html_to_markdown_profile_start"), PROFILE_START_DESC);

    html_to_markdown_profile_stop =
        LINKER.downcallHandle(findSymbol("html_to_markdown_profile_stop"), PROFILE_STOP_DESC);
  }

  /**
   * Find a native symbol in the loaded library.
   *
   * @param name the symbol name
   * @return the memory address of the symbol
   * @throws UnsatisfiedLinkError if the symbol cannot be found
   */
  private static MemorySegment findSymbol(final String name) {
    return SYMBOL_LOOKUP
        .find(name)
        .orElseThrow(() -> new UnsatisfiedLinkError("Symbol not found: " + name));
  }

  /**
   * Convert a Java String to a native C string (null-terminated).
   *
   * <p>The returned MemorySegment must be closed by the caller to free the native memory.
   *
   * @param arena the arena allocator to use
   * @param str the Java string to convert
   * @return a MemorySegment containing the null-terminated C string
   */
  static MemorySegment toCString(final Arena arena, final String str) {
    return StringUtils.toCString(arena, str);
  }

  /**
   * Convert a native C string to a Java String.
   *
   * @param addr the memory address of the C string
   * @return the Java string, or null if addr is NULL
   */
  static String fromCString(final MemorySegment addr) {
    return StringUtils.fromCString(addr);
  }

  /** Private constructor to prevent instantiation. */
  private HtmlToMarkdownFFI() {
    throw new UnsupportedOperationException("Utility class");
  }
}
