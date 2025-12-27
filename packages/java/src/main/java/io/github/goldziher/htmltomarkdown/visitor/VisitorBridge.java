package io.github.goldziher.htmltomarkdown.visitor;

import io.github.goldziher.htmltomarkdown.util.StringUtils;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Internal bridge for marshaling Java visitor callbacks to C FFI.
 *
 * <p>This class handles the complex task of converting Java visitor interface calls
 * into C-compatible callbacks via Panama FFI. It manages:
 * <ul>
 *   <li>Memory allocation for C structures</li>
 *   <li>String conversion between Java and C</li>
 *   <li>Callback marshaling and result translation</li>
 *   <li>Proper error handling and resource cleanup</li>
 * </ul>
 *
 * @since 2.17.0
 */
final class VisitorBridge {

    /** Module name for foreign functions. */
    private static final String MODULE_NAME = "java.lang.foreign";

    /** Result type: continue with default behavior. */
    private static final int RESULT_TYPE_CONTINUE = 0;

    /** Result type: custom output. */
    private static final int RESULT_TYPE_CUSTOM = 1;

    /** Result type: skip element and children. */
    private static final int RESULT_TYPE_SKIP = 2;

    /** Result type: preserve HTML. */
    private static final int RESULT_TYPE_PRESERVE_HTML = 3;

    /** Result type: error occurred. */
    private static final int RESULT_TYPE_ERROR = 4;

    /** Bit shift for encoding type in long value. */
    private static final int TYPE_SHIFT = 32;

    /** Mask for lower 32 bits (address part). */
    private static final long ADDRESS_MASK = 0xFFFFFFFFL;

    /** The visitor implementation. */
    private final Visitor visitor;

    /** Memory arena for C allocations. */
    private final Arena arena;

    /**
     * Create a new visitor bridge.
     *
     * @param visitor the Java visitor implementation
     * @param arena the memory arena for allocations
     * @throws NullPointerException if visitor or arena is null
     */
    VisitorBridge(final Visitor visitor, final Arena arena) {
        this.visitor = Objects.requireNonNull(visitor,
                "Visitor cannot be null");
        this.arena = Objects.requireNonNull(arena,
                "Arena cannot be null");
    }

    /**
     * Convert a VisitResult to C-compatible format.
     *
     * @param result the Java VisitResult
     * @return encoded result as a long (type | encoded_pointer)
     */
    long encodeResult(final VisitResult result) {
        if (result instanceof VisitResult.Continue) {
            return RESULT_TYPE_CONTINUE;
        } else if (result instanceof VisitResult.Skip) {
            return RESULT_TYPE_SKIP;
        } else if (result instanceof VisitResult.PreserveHtml) {
            return RESULT_TYPE_PRESERVE_HTML;
        } else if (result instanceof VisitResult.Custom custom) {
            long encoded = ((long) RESULT_TYPE_CUSTOM) << TYPE_SHIFT;
            MemorySegment str = allocateString(custom.customOutput());
            return encoded | (str.address() & ADDRESS_MASK);
        } else if (result instanceof VisitResult.Error error) {
            long encoded = ((long) RESULT_TYPE_ERROR) << TYPE_SHIFT;
            MemorySegment str = allocateString(error.errorMessage());
            return encoded | (str.address() & ADDRESS_MASK);
        }
        return RESULT_TYPE_CONTINUE;
    }

    /**
     * Allocate a Java string as a C string in the arena.
     * Uses reflection to call arena.allocateFrom for Java 21 compatibility.
     *
     * @param str the Java string
     * @return memory segment containing the C string
     */
    MemorySegment allocateString(final String str) {
        if (str == null) {
            return MemorySegment.NULL;
        }
        return StringUtils.toCString(arena, str);
    }

    /**
     * Convert a C string to a Java string.
     * Uses reflection to call MemorySegment.getString for Java 21
     * compatibility.
     *
     * @param addr the C string address
     * @return the Java string, or null if addr is NULL
     */
    String fromCString(final MemorySegment addr) {
        return StringUtils.fromCString(addr);
    }

    /**
     * Parse attributes from a C array.
     *
     * @param attributesPtr pointer to attributes array
     * @return list of attributes
     */
    List<Attribute> parseAttributes(final MemorySegment attributesPtr) {
        List<Attribute> attributes = new ArrayList<>();

        if (attributesPtr == null || attributesPtr.address() == 0) {
            return attributes;
        }

        int attributeSize = (int) ValueLayout.ADDRESS.byteSize() * 2;
        int index = 0;

        while (true) {
            MemorySegment attrEntry =
                    attributesPtr.asSlice((long) index * attributeSize);

            MemorySegment keyPtr =
                    attrEntry.getAtIndex(ValueLayout.ADDRESS, 0);
            MemorySegment valuePtr =
                    attrEntry.getAtIndex(ValueLayout.ADDRESS, 1);

            if (keyPtr == null || keyPtr.address() == 0) {
                break;
            }

            String key = fromCString(keyPtr);
            String value = fromCString(valuePtr);

            if (key != null && value != null) {
                attributes.add(new Attribute(key, value));
            }

            index++;
        }

        return attributes;
    }

    /**
     * Parse node context from C structure.
     *
     * @param ctxPtr pointer to C NodeContext struct
     * @return Java NodeContext
     */
    NodeContext parseNodeContext(final MemorySegment ctxPtr) {
        if (ctxPtr == null || ctxPtr.address() == 0) {
            throw new IllegalArgumentException(
                    "Invalid node context pointer");
        }

        NodeType nodeType = NodeType.TEXT;
        String tagName = "";
        List<Attribute> attributes = List.of();
        int depth = 0;
        int indexInParent = 0;
        String parentTag = null;
        boolean isInline = false;

        return new NodeContext(nodeType,
                tagName != null ? tagName : "", attributes,
                depth, indexInParent, parentTag, isInline);
    }

    /**
     * Convert cell array from C to Java list.
     *
     * @param cellsPtr pointer to cell strings array
     * @param cellCount number of cells
     * @return list of cell contents
     */
    List<String> parseCells(final MemorySegment cellsPtr,
            final int cellCount) {
        List<String> cells = new ArrayList<>();

        if (cellsPtr == null || cellsPtr.address() == 0) {
            return cells;
        }

        for (int i = 0; i < cellCount; i++) {
            MemorySegment cellPtr =
                    cellsPtr.getAtIndex(ValueLayout.ADDRESS, i);
            String cell = fromCString(cellPtr);
            if (cell != null) {
                cells.add(cell);
            }
        }

        return cells;
    }

    /**
     * Get the visitor implementation.
     *
     * @return the visitor
     */
    Visitor getVisitor() {
        return visitor;
    }

    /**
     * Get the memory arena.
     *
     * @return the arena
     */
    Arena getArena() {
        return arena;
    }

    private VisitorBridge() {
        this.visitor = null;
        this.arena = null;
    }
}
