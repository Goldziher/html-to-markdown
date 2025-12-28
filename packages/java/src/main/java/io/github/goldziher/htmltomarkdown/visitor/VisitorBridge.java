package io.github.goldziher.htmltomarkdown.visitor;

import io.github.goldziher.htmltomarkdown.util.StringUtils;
import java.lang.foreign.Arena;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.StructLayout;
import java.lang.foreign.ValueLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Internal bridge for marshaling Java visitor callbacks to C FFI.
 *
 * <p>This class handles the complex task of converting Java visitor interface calls into
 * C-compatible callbacks via Panama FFI. It manages:
 *
 * <ul>
 *   <li>Memory allocation for C structures
 *   <li>String conversion between Java and C
 *   <li>Callback marshaling and result translation
 *   <li>Proper error handling and resource cleanup
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

  /** Padding after node_type to align pointer fields on 64-bit. */
  private static final long NODE_CONTEXT_PADDING_AFTER_NODE_TYPE = 4;

  /** Padding after is_inline to align struct size on 64-bit. */
  private static final long NODE_CONTEXT_PADDING_AFTER_IS_INLINE = 7;

  /** C struct layout for html_to_markdown_attribute_t. */
  private static final StructLayout ATTRIBUTE_LAYOUT =
      MemoryLayout.structLayout(
              ValueLayout.ADDRESS.withName("key"), ValueLayout.ADDRESS.withName("value"))
          .withName("html_to_markdown_attribute_t");

  /**
   * C struct layout for html_to_markdown_node_context_t.
   *
   * <p>Rust struct layout: - node_type: enum (i32, 4 bytes) - padding: 4 bytes (64-bit alignment) -
   * tag_name: *const c_char (8 bytes on 64-bit) - attributes: *const html_to_markdown_attribute_t
   * (8 bytes) - depth: usize (8 bytes on 64-bit) - index_in_parent: usize (8 bytes on 64-bit) -
   * parent_tag: *const c_char (8 bytes) - is_inline: bool (1 byte)
   */
  private static final StructLayout NODE_CONTEXT_LAYOUT =
      MemoryLayout.structLayout(
              ValueLayout.JAVA_INT.withName("node_type"),
              MemoryLayout.paddingLayout(NODE_CONTEXT_PADDING_AFTER_NODE_TYPE),
              ValueLayout.ADDRESS.withName("tag_name"),
              ValueLayout.ADDRESS.withName("attributes"),
              ValueLayout.JAVA_LONG.withName("depth"),
              ValueLayout.JAVA_LONG.withName("index_in_parent"),
              ValueLayout.ADDRESS.withName("parent_tag"),
              ValueLayout.JAVA_BOOLEAN.withName("is_inline"),
              MemoryLayout.paddingLayout(NODE_CONTEXT_PADDING_AFTER_IS_INLINE))
          .withName("html_to_markdown_node_context_t");

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
    this.visitor = Objects.requireNonNull(visitor, "Visitor cannot be null");
    this.arena = Objects.requireNonNull(arena, "Arena cannot be null");
  }

  /**
   * Convert a VisitResult to C-compatible format.
   *
   * <p>For Custom and Error results, returns a struct with the allocated string pointer and type.
   * For other results, type field indicates the action.
   *
   * @param result the Java VisitResult
   * @return encoded result containing both type and pointer (without bit loss)
   */
  long encodeResult(final VisitResult result) {
    if (result instanceof VisitResult.Continue) {
      return RESULT_TYPE_CONTINUE;
    } else if (result instanceof VisitResult.Skip) {
      return RESULT_TYPE_SKIP;
    } else if (result instanceof VisitResult.PreserveHtml) {
      return RESULT_TYPE_PRESERVE_HTML;
    } else if (result instanceof VisitResult.Custom custom) {
      MemorySegment str = allocateString(custom.customOutput());
      return str.address();
    } else if (result instanceof VisitResult.Error error) {
      MemorySegment str = allocateString(error.errorMessage());
      return str.address();
    }
    return RESULT_TYPE_CONTINUE;
  }

  /**
   * Allocate a Java string as a C string in the arena. Uses reflection to call arena.allocateFrom
   * for Java 21 compatibility.
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
   * Convert a C string to a Java string. Uses reflection to call MemorySegment.getString for Java
   * 21 compatibility.
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
      MemorySegment attrEntry = attributesPtr.asSlice((long) index * attributeSize);

      MemorySegment keyPtr = attrEntry.getAtIndex(ValueLayout.ADDRESS, 0);
      MemorySegment valuePtr = attrEntry.getAtIndex(ValueLayout.ADDRESS, 1);

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
   * <p>Reads a html_to_markdown_node_context_t struct from native memory and converts all fields to
   * Java types. All string pointers are borrowed from Rust and valid only during callback
   * execution.
   *
   * @param ctxPtr pointer to C NodeContext struct
   * @return Java NodeContext with parsed data
   * @throws IllegalArgumentException if pointer is null or invalid
   */
  NodeContext parseNodeContext(final MemorySegment ctxPtr) {
    if (ctxPtr == null || ctxPtr.address() == 0) {
      throw new IllegalArgumentException("Invalid node context pointer");
    }

    MemorySegment ctx = ctxPtr.reinterpret(NODE_CONTEXT_LAYOUT.byteSize());

    int nodeTypeValue =
        ctx.get(
            ValueLayout.JAVA_INT,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("node_type")));
    NodeType nodeType = NodeType.fromCValue(nodeTypeValue);

    MemorySegment tagNamePtr =
        ctx.get(
            ValueLayout.ADDRESS,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("tag_name")));
    String tagName = fromCString(tagNamePtr);

    MemorySegment attributesPtr =
        ctx.get(
            ValueLayout.ADDRESS,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("attributes")));
    List<Attribute> attributes = parseAttributes(attributesPtr);

    long depthValue =
        ctx.get(
            ValueLayout.JAVA_LONG,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("depth")));
    int depth = (int) depthValue;

    long indexValue =
        ctx.get(
            ValueLayout.JAVA_LONG,
            NODE_CONTEXT_LAYOUT.byteOffset(
                MemoryLayout.PathElement.groupElement("index_in_parent")));
    int indexInParent = (int) indexValue;

    MemorySegment parentTagPtr =
        ctx.get(
            ValueLayout.ADDRESS,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("parent_tag")));
    String parentTag = fromCString(parentTagPtr);

    boolean isInline =
        ctx.get(
            ValueLayout.JAVA_BOOLEAN,
            NODE_CONTEXT_LAYOUT.byteOffset(MemoryLayout.PathElement.groupElement("is_inline")));

    return new NodeContext(
        nodeType,
        tagName != null ? tagName : "",
        attributes,
        depth,
        indexInParent,
        parentTag,
        isInline);
  }

  /**
   * Convert cell array from C to Java list.
   *
   * @param cellsPtr pointer to cell strings array
   * @param cellCount number of cells
   * @return list of cell contents
   */
  List<String> parseCells(final MemorySegment cellsPtr, final int cellCount) {
    List<String> cells = new ArrayList<>();

    if (cellsPtr == null || cellsPtr.address() == 0) {
      return cells;
    }

    for (int i = 0; i < cellCount; i++) {
      MemorySegment cellPtr = cellsPtr.getAtIndex(ValueLayout.ADDRESS, i);
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
