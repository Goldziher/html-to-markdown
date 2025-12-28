package io.github.goldziher.htmltomarkdown.util;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;

/**
 * Utility methods for converting between Java strings and C strings.
 *
 * @since 2.17.0
 */
public final class StringUtils {

  /**
   * Convert a Java String to a native C string (null-terminated).
   *
   * <p>The returned MemorySegment must be closed by the caller to free the native memory.
   *
   * @param arena the arena allocator to use
   * @param str the Java string to convert
   * @return a MemorySegment containing the null-terminated C string
   */
  public static MemorySegment toCString(final Arena arena, final String str) {
    try {
      java.lang.reflect.Method method = Arena.class.getMethod("allocateFrom", String.class);
      return (MemorySegment) method.invoke(arena, str);
    } catch (Exception e) {
      byte[] bytes = str.getBytes(java.nio.charset.StandardCharsets.UTF_8);
      long allocationSize = (long) bytes.length + 1;
      MemorySegment segment = arena.allocate(allocationSize, ValueLayout.JAVA_BYTE.byteAlignment());
      for (int i = 0; i < bytes.length; i++) {
        segment.setAtIndex(ValueLayout.JAVA_BYTE, i, bytes[i]);
      }
      segment.setAtIndex(ValueLayout.JAVA_BYTE, bytes.length, (byte) 0);
      return segment;
    }
  }

  /**
   * Convert a native C string to a Java String.
   *
   * @param addr the memory address of the C string
   * @return the Java string, or null if addr is NULL
   */
  public static String fromCString(final MemorySegment addr) {
    if (addr == null || addr.address() == 0) {
      return null;
    }

    try {
      java.lang.reflect.Method method = MemorySegment.class.getMethod("getString", long.class);
      return (String) method.invoke(addr.reinterpret(Long.MAX_VALUE), 0L);
    } catch (Exception e) {
      java.util.List<Byte> bytes = new java.util.ArrayList<>();
      int offset = 0;
      while (true) {
        try {
          byte b = addr.getAtIndex(ValueLayout.JAVA_BYTE, offset);
          if (b == 0) {
            break;
          }
          bytes.add(b);
          offset++;
        } catch (IndexOutOfBoundsException ex) {
          break;
        }
      }
      byte[] result = new byte[bytes.size()];
      for (int i = 0; i < bytes.size(); i++) {
        result[i] = bytes.get(i);
      }
      return new String(result, java.nio.charset.StandardCharsets.UTF_8);
    }
  }

  /** Private constructor to prevent instantiation. */
  private StringUtils() {
    throw new UnsupportedOperationException("Utility class");
  }
}
