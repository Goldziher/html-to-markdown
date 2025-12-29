package dev.kreuzberg.htmltomarkdown;

import static org.junit.jupiter.api.Assertions.*;

import dev.kreuzberg.htmltomarkdown.visitor.*;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

/**
 * Comprehensive test suite for visitor pattern support.
 *
 * @since 2.17.0
 */
@DisplayName("Visitor Pattern Tests")
class VisitorTest {

  @Nested
  @DisplayName("NodeType Tests")
  class NodeTypeTests {
    @Test
    @DisplayName("Convert from C value to Java enum")
    void testNodeTypeFromCValue() {
      NodeType type = NodeType.fromCValue(0);
      assertEquals(NodeType.TEXT, type);

      type = NodeType.fromCValue(2);
      assertEquals(NodeType.HEADING, type);

      type = NodeType.fromCValue(87);
      assertEquals(NodeType.CUSTOM, type);
    }

    @Test
    @DisplayName("Convert to C value")
    void testNodeTypeToCValue() {
      assertEquals(0, NodeType.TEXT.toCValue());
      assertEquals(1, NodeType.ELEMENT.toCValue());
      assertEquals(2, NodeType.HEADING.toCValue());
    }

    @Test
    @DisplayName("All node types are valid")
    void testAllNodeTypesValid() {
      for (NodeType type : NodeType.values()) {
        int cValue = type.toCValue();
        NodeType roundTrip = NodeType.fromCValue(cValue);
        assertEquals(type, roundTrip);
      }
    }
  }

  @Nested
  @DisplayName("Attribute Tests")
  class AttributeTests {
    @Test
    @DisplayName("Create valid attribute")
    void testCreateAttribute() {
      Attribute attr = new Attribute("class", "container");
      assertEquals("class", attr.key());
      assertEquals("container", attr.value());
    }

    @Test
    @DisplayName("Attribute with empty value")
    void testEmptyValueAttribute() {
      Attribute attr = new Attribute("disabled", "");
      assertTrue(attr.isEmpty());
    }

    @Test
    @DisplayName("Attribute toString")
    void testAttributeToString() {
      Attribute attr = new Attribute("id", "main");
      assertEquals("id=\"main\"", attr.toString());

      Attribute empty = new Attribute("checked", "");
      assertEquals("checked", empty.toString());
    }

    @Test
    @DisplayName("Attribute rejects null key")
    void testAttributeNullKey() {
      assertThrows(NullPointerException.class, () -> new Attribute(null, "value"));
    }

    @Test
    @DisplayName("Attribute rejects null value")
    void testAttributeNullValue() {
      assertThrows(NullPointerException.class, () -> new Attribute("key", null));
    }
  }

  @Nested
  @DisplayName("NodeContext Tests")
  class NodeContextTests {
    @Test
    @DisplayName("Create valid node context")
    void testCreateNodeContext() {
      NodeContext ctx = new NodeContext(NodeType.PARAGRAPH, "p", List.of(), 1, 0, "body", true);

      assertEquals(NodeType.PARAGRAPH, ctx.nodeType());
      assertEquals("p", ctx.tagName());
      assertEquals(1, ctx.depth());
      assertEquals(0, ctx.indexInParent());
      assertEquals("body", ctx.parentTag());
      assertTrue(ctx.isInline());
    }

    @Test
    @DisplayName("NodeContext with attributes")
    void testNodeContextWithAttributes() {
      List<Attribute> attrs = List.of(new Attribute("class", "btn"), new Attribute("id", "submit"));
      NodeContext ctx = new NodeContext(NodeType.ELEMENT, "button", attrs, 2, 1, "form", true);

      assertEquals(2, ctx.attributes().size());
      assertTrue(ctx.hasAttribute("class"));
      assertEquals("btn", ctx.getAttributeValue("class"));
    }

    @Test
    @DisplayName("Get missing attribute returns null")
    void testGetMissingAttribute() {
      NodeContext ctx = new NodeContext(NodeType.ELEMENT, "div", List.of(), 0, 0, null, false);
      assertNull(ctx.getAttributeValue("nonexistent"));
    }

    @Test
    @DisplayName("Check text node")
    void testIsTextNode() {
      NodeContext textCtx = new NodeContext(NodeType.TEXT, "", List.of(), 1, 0, "p", true);
      assertTrue(textCtx.isTextNode());

      NodeContext elemCtx = new NodeContext(NodeType.ELEMENT, "div", List.of(), 0, 0, null, false);
      assertFalse(elemCtx.isTextNode());
    }

    @Test
    @DisplayName("Check heading")
    void testIsHeading() {
      NodeContext headingCtx =
          new NodeContext(NodeType.HEADING, "h1", List.of(), 1, 0, "body", false);
      assertTrue(headingCtx.isHeading());

      NodeContext paraCtx =
          new NodeContext(NodeType.PARAGRAPH, "p", List.of(), 1, 1, "body", false);
      assertFalse(paraCtx.isHeading());
    }

    @Test
    @DisplayName("Check root context")
    void testIsRoot() {
      NodeContext root = new NodeContext(NodeType.ELEMENT, "html", List.of(), 0, 0, null, false);
      assertTrue(root.isRoot());

      NodeContext child = new NodeContext(NodeType.ELEMENT, "body", List.of(), 1, 0, "html", false);
      assertFalse(child.isRoot());
    }

    @Test
    @DisplayName("Check parent presence")
    void testHasParent() {
      NodeContext withParent =
          new NodeContext(NodeType.ELEMENT, "p", List.of(), 1, 0, "body", false);
      assertTrue(withParent.hasParent());

      NodeContext noParent =
          new NodeContext(NodeType.ELEMENT, "html", List.of(), 0, 0, null, false);
      assertFalse(noParent.hasParent());
    }

    @Test
    @DisplayName("NodeContext rejects null nodeType")
    void testNullNodeType() {
      assertThrows(
          NullPointerException.class,
          () -> new NodeContext(null, "p", List.of(), 0, 0, null, false));
    }

    @Test
    @DisplayName("NodeContext rejects null tagName")
    void testNullTagName() {
      assertThrows(
          NullPointerException.class,
          () -> new NodeContext(NodeType.ELEMENT, null, List.of(), 0, 0, null, false));
    }

    @Test
    @DisplayName("NodeContext rejects negative depth")
    void testNegativeDepth() {
      assertThrows(
          IllegalArgumentException.class,
          () -> new NodeContext(NodeType.ELEMENT, "p", List.of(), -1, 0, null, false));
    }

    @Test
    @DisplayName("NodeContext rejects negative index")
    void testNegativeIndex() {
      assertThrows(
          IllegalArgumentException.class,
          () -> new NodeContext(NodeType.ELEMENT, "p", List.of(), 0, -1, null, false));
    }

    @Test
    @DisplayName("NodeContext makes defensive copy of attributes")
    void testAttributesDefensiveCopy() {
      List<Attribute> original = new ArrayList<>();
      original.add(new Attribute("class", "test"));

      NodeContext ctx = new NodeContext(NodeType.ELEMENT, "div", original, 0, 0, null, false);

      original.add(new Attribute("id", "main"));

      assertEquals(1, ctx.attributes().size());
    }
  }

  @Nested
  @DisplayName("VisitResult Tests")
  class VisitResultTests {
    @Test
    @DisplayName("Continue result")
    void testContinueResult() {
      VisitResult result = VisitResult.Continue.INSTANCE;
      assertTrue(result instanceof VisitResult.Continue);
      assertSame(VisitResult.Continue.INSTANCE, result);
    }

    @Test
    @DisplayName("Custom result")
    void testCustomResult() {
      VisitResult result = new VisitResult.Custom("**bold**");
      assertTrue(result instanceof VisitResult.Custom);
      assertEquals("**bold**", ((VisitResult.Custom) result).customOutput());
    }

    @Test
    @DisplayName("Custom result rejects null output")
    void testCustomResultNullOutput() {
      assertThrows(NullPointerException.class, () -> new VisitResult.Custom(null));
    }

    @Test
    @DisplayName("Skip result")
    void testSkipResult() {
      VisitResult result = VisitResult.Skip.INSTANCE;
      assertTrue(result instanceof VisitResult.Skip);
      assertSame(VisitResult.Skip.INSTANCE, result);
    }

    @Test
    @DisplayName("PreserveHtml result")
    void testPreserveHtmlResult() {
      VisitResult result = VisitResult.PreserveHtml.INSTANCE;
      assertTrue(result instanceof VisitResult.PreserveHtml);
      assertSame(VisitResult.PreserveHtml.INSTANCE, result);
    }

    @Test
    @DisplayName("Error result")
    void testErrorResult() {
      VisitResult result = new VisitResult.Error("Invalid element");
      assertTrue(result instanceof VisitResult.Error);
      assertEquals("Invalid element", ((VisitResult.Error) result).errorMessage());
    }

    @Test
    @DisplayName("Error result rejects null message")
    void testErrorResultNullMessage() {
      assertThrows(NullPointerException.class, () -> new VisitResult.Error(null));
    }

    @Test
    @DisplayName("Result type matching")
    void testResultTypeMatching() {
      VisitResult cont = VisitResult.Continue.INSTANCE;
      VisitResult custom = new VisitResult.Custom("test");
      VisitResult skip = VisitResult.Skip.INSTANCE;
      VisitResult preserve = VisitResult.PreserveHtml.INSTANCE;
      VisitResult error = new VisitResult.Error("error");

      assertTrue(cont instanceof VisitResult.Continue);
      assertTrue(custom instanceof VisitResult.Custom);
      assertTrue(skip instanceof VisitResult.Skip);
      assertTrue(preserve instanceof VisitResult.PreserveHtml);
      assertTrue(error instanceof VisitResult.Error);
    }
  }

  @Nested
  @DisplayName("Visitor Implementation Tests")
  class VisitorImplementationTests {
    @Test
    @DisplayName("Default visitor returns Continue")
    void testDefaultVisitor() {
      Visitor visitor = new Visitor() {
            // no overrides
          };

      NodeContext ctx = new NodeContext(NodeType.PARAGRAPH, "p", List.of(), 1, 0, "body", false);

      assertEquals(VisitResult.Continue.INSTANCE, visitor.visitElementStart(ctx));
      assertEquals(VisitResult.Continue.INSTANCE, visitor.visitElementEnd(ctx, "test"));
      assertEquals(VisitResult.Continue.INSTANCE, visitor.visitText(ctx, "hello"));
      assertEquals(VisitResult.Continue.INSTANCE, visitor.visitLink(ctx, "/", "link", null));
    }

    @Test
    @DisplayName("Custom visitor implementation - filter links")
    void testCustomVisitorFilterLinks() {
      Visitor visitor =
          new Visitor() {
            @Override
            public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
              if (href.startsWith("http")) {
                return VisitResult.Skip.INSTANCE;
              }
              return VisitResult.Continue.INSTANCE;
            }
          };

      NodeContext ctx = new NodeContext(NodeType.LINK, "a", List.of(), 2, 0, "p", true);

      assertEquals(
          VisitResult.Skip.INSTANCE, visitor.visitLink(ctx, "https://example.com", "link", null));
      assertEquals(VisitResult.Continue.INSTANCE, visitor.visitLink(ctx, "/page", "link", null));
    }

    @Test
    @DisplayName("Custom visitor implementation - custom heading format")
    void testCustomVisitorHeadingFormat() {
      Visitor visitor =
          new Visitor() {
            @Override
            public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
              String prefix = "#".repeat(level);
              return new VisitResult.Custom(prefix + " [" + text + "]");
            }
          };

      NodeContext ctx = new NodeContext(NodeType.HEADING, "h1", List.of(), 1, 0, "body", false);

      VisitResult result = visitor.visitHeading(ctx, 1, "Title", null);
      assertTrue(result instanceof VisitResult.Custom);
      assertEquals("# [Title]", ((VisitResult.Custom) result).customOutput());
    }

    @Test
    @DisplayName("Visitor with state accumulation")
    void testStatefulVisitor() {
      class CountingVisitor implements Visitor {
        int linkCount = 0;

        @Override
        public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
          linkCount++;
          return VisitResult.Continue.INSTANCE;
        }
      }

      CountingVisitor visitor = new CountingVisitor();
      NodeContext ctx = new NodeContext(NodeType.LINK, "a", List.of(), 2, 0, "p", true);

      visitor.visitLink(ctx, "https://example.com", "link1", null);
      visitor.visitLink(ctx, "/page", "link2", null);
      visitor.visitLink(ctx, "https://other.com", "link3", null);

      assertEquals(3, visitor.linkCount);
    }
  }

  @Nested
  @DisplayName("Integration Tests")
  class IntegrationTests {
    @Test
    @DisplayName("Simple heading visitor")
    void testHeadingVisitor() {
      Visitor visitor =
          new Visitor() {
            @Override
            public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
              assertEquals(NodeType.HEADING, ctx.nodeType());
              assertEquals("h1", ctx.tagName());
              assertTrue(level >= 1 && level <= 6);
              assertNotNull(text);
              return VisitResult.Continue.INSTANCE;
            }
          };

      assertNotNull(visitor);
    }

    @Test
    @DisplayName("Link filtering visitor")
    void testLinkFilteringVisitor() {
      class ExternalLinkFilter implements Visitor {
        @Override
        public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
          if (href != null && href.contains("evil.com")) {
            return VisitResult.Skip.INSTANCE;
          }
          return VisitResult.Continue.INSTANCE;
        }
      }

      ExternalLinkFilter visitor = new ExternalLinkFilter();
      NodeContext ctx = new NodeContext(NodeType.LINK, "a", List.of(), 2, 0, "p", true);

      assertEquals(
          VisitResult.Skip.INSTANCE, visitor.visitLink(ctx, "https://evil.com", "bad", null));
      assertEquals(
          VisitResult.Continue.INSTANCE, visitor.visitLink(ctx, "https://good.com", "good", null));
    }

    @Test
    @DisplayName("Code block language detector")
    void testCodeBlockVisitor() {
      class LanguageCounter implements Visitor {
        int pythonBlocks = 0;
        int otherBlocks = 0;

        @Override
        public VisitResult visitCodeBlock(NodeContext ctx, String lang, String code) {
          if ("python".equals(lang) || "py".equals(lang)) {
            pythonBlocks++;
          } else {
            otherBlocks++;
          }
          return VisitResult.Continue.INSTANCE;
        }
      }

      LanguageCounter visitor = new LanguageCounter();
      NodeContext ctx = new NodeContext(NodeType.PRE, "pre", List.of(), 2, 0, "body", false);

      visitor.visitCodeBlock(ctx, "python", "print('hello')");
      visitor.visitCodeBlock(ctx, "javascript", "console.log('hello')");
      visitor.visitCodeBlock(ctx, "py", "x = 1");

      assertEquals(2, visitor.pythonBlocks);
      assertEquals(1, visitor.otherBlocks);
    }

    @Test
    @DisplayName("Table row analyzer")
    void testTableRowVisitor() {
      class TableAnalyzer implements Visitor {
        int totalCells = 0;

        @Override
        public VisitResult visitTableRow(NodeContext ctx, List<String> cells, boolean isHeader) {
          totalCells += cells.size();
          return VisitResult.Continue.INSTANCE;
        }
      }

      TableAnalyzer visitor = new TableAnalyzer();
      NodeContext ctx = new NodeContext(NodeType.TABLE_ROW, "tr", List.of(), 2, 0, "tbody", false);

      visitor.visitTableRow(ctx, List.of("A", "B", "C"), false);
      visitor.visitTableRow(ctx, List.of("1", "2", "3"), false);
      visitor.visitTableRow(ctx, List.of("4", "5", "6"), false);

      assertEquals(9, visitor.totalCells);
    }

    @Test
    @DisplayName("Attribute extraction visitor")
    void testAttributeExtraction() {
      class AttributeCollector implements Visitor {
        List<String> classNames = new ArrayList<>();

        @Override
        public VisitResult visitElementStart(NodeContext ctx) {
          String className = ctx.getAttributeValue("class");
          if (className != null && !className.isEmpty()) {
            classNames.add(className);
          }
          return VisitResult.Continue.INSTANCE;
        }
      }

      AttributeCollector visitor = new AttributeCollector();
      NodeContext ctx =
          new NodeContext(
              NodeType.ELEMENT,
              "div",
              List.of(new Attribute("class", "container")),
              1,
              0,
              "body",
              false);

      visitor.visitElementStart(ctx);
      assertEquals(1, visitor.classNames.size());
      assertEquals("container", visitor.classNames.get(0));
    }
  }

  @Nested
  @DisplayName("Error Handling Tests")
  class ErrorHandlingTests {
    @Test
    @DisplayName("Visitor can report conversion errors")
    void testErrorReporting() {
      Visitor visitor =
          new Visitor() {
            @Override
            public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
              if (href == null || href.isEmpty()) {
                return new VisitResult.Error("Link has no href");
              }
              return VisitResult.Continue.INSTANCE;
            }
          };

      NodeContext ctx = new NodeContext(NodeType.LINK, "a", List.of(), 2, 0, "p", true);

      VisitResult result = visitor.visitLink(ctx, null, "text", null);
      assertTrue(result instanceof VisitResult.Error);
      assertEquals("Link has no href", ((VisitResult.Error) result).errorMessage());
    }

    @Test
    @DisplayName("Custom result can contain complex markdown")
    void testComplexCustomOutput() {
      String complex = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
      VisitResult result = new VisitResult.Custom(complex);

      assertTrue(result instanceof VisitResult.Custom);
      assertEquals(complex, ((VisitResult.Custom) result).customOutput());
    }
  }

  @Nested
  @DisplayName("Memory Safety Tests")
  class MemorySafetyTests {
    @Test
    @DisplayName("NodeContext attributes are immutable")
    void testAttributesImmutable() {
      List<Attribute> attrs = new ArrayList<>();
      attrs.add(new Attribute("id", "test"));

      NodeContext ctx = new NodeContext(NodeType.ELEMENT, "div", attrs, 0, 0, null, false);

      attrs.add(new Attribute("class", "new"));

      assertEquals(1, ctx.attributes().size());

      assertThrows(
          UnsupportedOperationException.class,
          () -> ctx.attributes().add(new Attribute("data", "value")));
    }

    @Test
    @DisplayName("NodeContext creates defensive copies")
    void testDefensiveCopy() {
      List<Attribute> original = new ArrayList<>();
      original.add(new Attribute("a", "1"));
      original.add(new Attribute("b", "2"));

      NodeContext ctx = new NodeContext(NodeType.ELEMENT, "div", original, 0, 0, null, false);

      original.clear();

      assertEquals(2, ctx.attributes().size());
    }
  }
}
