package io.github.goldziher.htmltomarkdown;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

/**
 * Low-level Foreign Function Interface (FFI) bindings to the native html-to-markdown library.
 * <p>
 * This class provides direct access to the C FFI functions using Java's Foreign Function &amp; Memory API (Panama).
 * For a higher-level, more ergonomic API, use {@link HtmlToMarkdown} instead.
 *
 * @since 2.7.3
 */
class HtmlToMarkdownFFI {

    private static final String LIBRARY_NAME = "html_to_markdown_ffi";
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup SYMBOL_LOOKUP;

    // Function descriptors matching the C FFI signatures
    private static final FunctionDescriptor CONVERT_DESC = FunctionDescriptor.of(
        ValueLayout.ADDRESS,  // char* return
        ValueLayout.ADDRESS   // const char* html parameter
    );

    private static final FunctionDescriptor FREE_STRING_DESC = FunctionDescriptor.ofVoid(
        ValueLayout.ADDRESS   // char* s parameter
    );

    private static final FunctionDescriptor VERSION_DESC = FunctionDescriptor.of(
        ValueLayout.ADDRESS   // const char* return
    );

    private static final FunctionDescriptor LAST_ERROR_DESC = FunctionDescriptor.of(
        ValueLayout.ADDRESS   // const char* return
    );

    private static final FunctionDescriptor CONVERT_WITH_METADATA_DESC = FunctionDescriptor.of(
        ValueLayout.ADDRESS,  // char* return (markdown)
        ValueLayout.ADDRESS,  // const char* html parameter
        ValueLayout.ADDRESS   // char** metadata_json_out parameter
    );

    // Method handles for native functions
    static final MethodHandle html_to_markdown_convert;
    static final MethodHandle html_to_markdown_free_string;
    static final MethodHandle html_to_markdown_version;
    static final MethodHandle html_to_markdown_last_error;
    static final MethodHandle html_to_markdown_convert_with_metadata;

    static {
        // Load the native library
        System.loadLibrary(LIBRARY_NAME);

        // Get symbol lookup for the loaded library
        SYMBOL_LOOKUP = SymbolLookup.loaderLookup();

        // Bind native functions to method handles
        html_to_markdown_convert = LINKER.downcallHandle(
            findSymbol("html_to_markdown_convert"),
            CONVERT_DESC
        );

        html_to_markdown_free_string = LINKER.downcallHandle(
            findSymbol("html_to_markdown_free_string"),
            FREE_STRING_DESC
        );

        html_to_markdown_version = LINKER.downcallHandle(
            findSymbol("html_to_markdown_version"),
            VERSION_DESC
        );

        html_to_markdown_last_error = LINKER.downcallHandle(
            findSymbol("html_to_markdown_last_error"),
            LAST_ERROR_DESC
        );

        html_to_markdown_convert_with_metadata = LINKER.downcallHandle(
            findSymbol("html_to_markdown_convert_with_metadata"),
            CONVERT_WITH_METADATA_DESC
        );
    }

    /**
     * Find a native symbol in the loaded library.
     *
     * @param name the symbol name
     * @return the memory address of the symbol
     * @throws UnsatisfiedLinkError if the symbol cannot be found
     */
    private static MemorySegment findSymbol(String name) {
        return SYMBOL_LOOKUP.find(name)
            .orElseThrow(() -> new UnsatisfiedLinkError("Symbol not found: " + name));
    }

    /**
     * Convert a Java String to a native C string (null-terminated).
     * <p>
     * The returned MemorySegment must be closed by the caller to free the native memory.
     *
     * @param arena the arena allocator to use
     * @param str the Java string to convert
     * @return a MemorySegment containing the null-terminated C string
     */
    static MemorySegment toCString(Arena arena, String str) {
        return arena.allocateFrom(str);
    }

    /**
     * Convert a native C string to a Java String.
     *
     * @param addr the memory address of the C string
     * @return the Java string, or null if addr is NULL
     */
    static String fromCString(MemorySegment addr) {
        if (addr == null || addr.address() == 0) {
            return null;
        }
        return addr.reinterpret(Long.MAX_VALUE).getString(0);
    }

    // Private constructor to prevent instantiation
    private HtmlToMarkdownFFI() {
        throw new UnsupportedOperationException("Utility class");
    }
}
