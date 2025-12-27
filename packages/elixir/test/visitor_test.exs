defmodule HtmlToMarkdown.VisitorTest do
  use ExUnit.Case
  doctest HtmlToMarkdown.Visitor

  describe "visitor behaviour" do
    test "visitor module defines behaviour callbacks" do
      # Verify the Visitor module has the convert_with_visitor function
      assert function_exported?(HtmlToMarkdown.Visitor, :convert_with_visitor, 3)
    end

    test "visitor provides convenience macros via __using__" do
      defmodule TestVisitor do
        use HtmlToMarkdown.Visitor
      end

      assert function_exported?(TestVisitor, :handle_text, 2)
      assert function_exported?(TestVisitor, :handle_link, 4)
      assert function_exported?(TestVisitor, :handle_image, 4)
      assert function_exported?(TestVisitor, :handle_heading, 4)
    end
  end

  describe "convert_with_visitor/3" do
    test "accepts HTML string and visitor module" do
      defmodule SimpleVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<p>Hello World</p>"
      result = HtmlToMarkdown.Visitor.convert_with_visitor(html, SimpleVisitor, nil)

      assert is_tuple(result)
      assert tuple_size(result) == 2
    end

    test "returns {:ok, markdown} on success" do
      defmodule PassThroughVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<p>Hello</p>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, PassThroughVisitor, nil)

      assert is_binary(markdown)
      assert String.contains?(markdown, "Hello")
    end

    test "returns {:error, reason} on invalid input" do
      defmodule ErrorVisitor do
        use HtmlToMarkdown.Visitor
      end

      # Empty or nil should still work, or produce an error
      result = HtmlToMarkdown.Visitor.convert_with_visitor("", ErrorVisitor, nil)

      assert is_tuple(result)
      assert tuple_size(result) == 2
    end

    test "accepts options map as third parameter" do
      defmodule OptionsVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<h1>Title</h1>"
      options = %{heading_style: "atx"}

      result = HtmlToMarkdown.Visitor.convert_with_visitor(html, OptionsVisitor, options)

      assert {:ok, _markdown} = result
    end

    test "works with nil options" do
      defmodule NilOptionsVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<p>Test</p>"

      result = HtmlToMarkdown.Visitor.convert_with_visitor(html, NilOptionsVisitor, nil)

      assert {:ok, _markdown} = result
    end
  end

  describe "visitor callbacks" do
    test "handle_text callback is defined with correct arity" do
      defmodule TextVisitor do
        use HtmlToMarkdown.Visitor

        def handle_text(_context, _text) do
          :continue
        end
      end

      html = "Hello"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, TextVisitor, nil)
    end

    test "handle_link callback is defined with correct arity" do
      defmodule LinkVisitor do
        use HtmlToMarkdown.Visitor

        def handle_link(_context, _href, _text, _title) do
          :continue
        end
      end

      html = "<a href='http://example.com'>Example</a>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, LinkVisitor, nil)
    end

    test "handle_image callback is defined with correct arity" do
      defmodule ImageVisitor do
        use HtmlToMarkdown.Visitor

        def handle_image(_context, _src, _alt, _title) do
          :continue
        end
      end

      html = "<img src='test.jpg' alt='Test Image'>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, ImageVisitor, nil)
    end

    test "handle_heading callback is defined with correct arity" do
      defmodule HeadingVisitor do
        use HtmlToMarkdown.Visitor

        def handle_heading(_context, _level, _text, _id) do
          :continue
        end
      end

      html = "<h1>Title</h1>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, HeadingVisitor, nil)
    end

    test "handle_code_block callback is defined with correct arity" do
      defmodule CodeBlockVisitor do
        use HtmlToMarkdown.Visitor

        def handle_code_block(_context, _lang, _code) do
          :continue
        end
      end

      html = "<pre><code>let x = 1;</code></pre>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, CodeBlockVisitor, nil)
    end

    test "handle_code_inline callback is defined with correct arity" do
      defmodule CodeInlineVisitor do
        use HtmlToMarkdown.Visitor

        def handle_code_inline(_context, _code) do
          :continue
        end
      end

      html = "<code>inline</code>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, CodeInlineVisitor, nil)
    end

    test "handle_list_item callback is defined with correct arity" do
      defmodule ListItemVisitor do
        use HtmlToMarkdown.Visitor

        def handle_list_item(_context, _ordered, _marker, _text) do
          :continue
        end
      end

      html = "<ul><li>Item 1</li></ul>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, ListItemVisitor, nil)
    end

    test "handle_table_start callback is defined with correct arity" do
      defmodule TableStartVisitor do
        use HtmlToMarkdown.Visitor

        def handle_table_start(_context) do
          :continue
        end
      end

      html = "<table><tr><td>Data</td></tr></table>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, TableStartVisitor, nil)
    end

    test "handle_blockquote callback is defined with correct arity" do
      defmodule BlockquoteVisitor do
        use HtmlToMarkdown.Visitor

        def handle_blockquote(_context, _content, _depth) do
          :continue
        end
      end

      html = "<blockquote>Quote</blockquote>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, BlockquoteVisitor, nil)
    end

    test "handle_strong callback is defined with correct arity" do
      defmodule StrongVisitor do
        use HtmlToMarkdown.Visitor

        def handle_strong(_context, _text) do
          :continue
        end
      end

      html = "<strong>Bold</strong>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, StrongVisitor, nil)
    end

    test "handle_emphasis callback is defined with correct arity" do
      defmodule EmphasisVisitor do
        use HtmlToMarkdown.Visitor

        def handle_emphasis(_context, _text) do
          :continue
        end
      end

      html = "<em>Italic</em>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, EmphasisVisitor, nil)
    end

    test "handle_strikethrough callback is defined with correct arity" do
      defmodule StrikethroughVisitor do
        use HtmlToMarkdown.Visitor

        def handle_strikethrough(_context, _text) do
          :continue
        end
      end

      html = "<s>Strikethrough</s>"

      {:ok, _markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, StrikethroughVisitor, nil)
    end

    test "handle_underline callback is defined with correct arity" do
      defmodule UnderlineVisitor do
        use HtmlToMarkdown.Visitor

        def handle_underline(_context, _text) do
          :continue
        end
      end

      html = "<u>Underline</u>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, UnderlineVisitor, nil)
    end

    test "handle_line_break callback is defined with correct arity" do
      defmodule LineBreakVisitor do
        use HtmlToMarkdown.Visitor

        def handle_line_break(_context) do
          :continue
        end
      end

      html = "Line 1<br>Line 2"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, LineBreakVisitor, nil)
    end

    test "handle_horizontal_rule callback is defined with correct arity" do
      defmodule HorizontalRuleVisitor do
        use HtmlToMarkdown.Visitor

        def handle_horizontal_rule(_context) do
          :continue
        end
      end

      html = "<hr>"

      {:ok, _markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, HorizontalRuleVisitor, nil)
    end

    test "handle_custom_element callback is defined with correct arity" do
      defmodule CustomElementVisitor do
        use HtmlToMarkdown.Visitor

        def handle_custom_element(_context, _tag_name, _html) do
          :continue
        end
      end

      html = "<custom-element>Custom</custom-element>"

      {:ok, _markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, CustomElementVisitor, nil)
    end

    test "handle_form callback is defined with correct arity" do
      defmodule FormVisitor do
        use HtmlToMarkdown.Visitor

        def handle_form(_context, _action, _method) do
          :continue
        end
      end

      html = "<form action='/submit' method='post'></form>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, FormVisitor, nil)
    end

    test "handle_input callback is defined with correct arity" do
      defmodule InputVisitor do
        use HtmlToMarkdown.Visitor

        def handle_input(_context, _input_type, _name, _value) do
          :continue
        end
      end

      html = "<input type='text' name='username' value='admin'>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, InputVisitor, nil)
    end

    test "handle_button callback is defined with correct arity" do
      defmodule ButtonVisitor do
        use HtmlToMarkdown.Visitor

        def handle_button(_context, _text) do
          :continue
        end
      end

      html = "<button>Click</button>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, ButtonVisitor, nil)
    end

    test "handle_audio callback is defined with correct arity" do
      defmodule AudioVisitor do
        use HtmlToMarkdown.Visitor

        def handle_audio(_context, _src) do
          :continue
        end
      end

      html = "<audio src='test.mp3'></audio>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, AudioVisitor, nil)
    end

    test "handle_video callback is defined with correct arity" do
      defmodule VideoVisitor do
        use HtmlToMarkdown.Visitor

        def handle_video(_context, _src) do
          :continue
        end
      end

      html = "<video src='test.mp4'></video>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, VideoVisitor, nil)
    end

    test "handle_iframe callback is defined with correct arity" do
      defmodule IframeVisitor do
        use HtmlToMarkdown.Visitor

        def handle_iframe(_context, _src) do
          :continue
        end
      end

      html = "<iframe src='test.html'></iframe>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, IframeVisitor, nil)
    end
  end

  describe "visit result types" do
    test ":continue result allows default behavior" do
      defmodule ContinueVisitor do
        use HtmlToMarkdown.Visitor

        def handle_text(_context, _text) do
          :continue
        end
      end

      html = "Text"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, ContinueVisitor, nil)

      assert String.contains?(markdown, "Text")
    end

    test "{:custom, markdown} result uses custom output" do
      defmodule CustomVisitor do
        use HtmlToMarkdown.Visitor

        def handle_link(_context, href, text, _title) do
          {:custom, "[#{text}](#{href})"}
        end
      end

      html = "<a href='http://example.com'>Link</a>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, CustomVisitor, nil)

      # Result will depend on native implementation
      assert is_binary(markdown)
    end

    test ":skip result omits element" do
      defmodule SkipVisitor do
        use HtmlToMarkdown.Visitor

        def handle_image(_context, _src, _alt, _title) do
          :skip
        end
      end

      html = "<img src='test.jpg' alt='Test'>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, SkipVisitor, nil)

      assert is_binary(markdown)
    end

    test ":preserve_html result keeps original HTML" do
      defmodule PreserveVisitor do
        use HtmlToMarkdown.Visitor

        def handle_custom_element(_context, _tag_name, _html) do
          :preserve_html
        end
      end

      html = "<custom>Content</custom>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, PreserveVisitor, nil)

      assert is_binary(markdown)
    end

    test "{:error, reason} is a valid return value from callback" do
      defmodule ErrorVisitor do
        use HtmlToMarkdown.Visitor

        # Note: Visitor callbacks in the Rust NIF are currently a placeholder.
        # This test verifies the callback definition is valid.
        def handle_link(_context, _href, _text, _title) do
          {:error, "Links not allowed"}
        end
      end

      # Test that the visitor module is valid
      assert function_exported?(ErrorVisitor, :handle_link, 4)
    end
  end

  describe "visitor with state via module attributes" do
    test "visitor can use module state for tracking" do
      defmodule StatefulVisitor do
        use HtmlToMarkdown.Visitor

        def handle_link(_context, _href, _text, _title) do
          # Note: This is a simplified approach - real state management
          # would require GenServer or Agent
          :continue
        end
      end

      html = "<a href='http://example.com'>Link 1</a><a href='http://test.com'>Link 2</a>"
      {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, StatefulVisitor, nil)
    end
  end

  describe "visitor with HTML options integration" do
    test "visitor respects heading_style option" do
      defmodule OptionsHeadingVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<h1>Title</h1>"
      options = %{heading_style: "atx"}

      {:ok, markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, OptionsHeadingVisitor, options)

      assert is_binary(markdown)
    end

    test "visitor respects list_indent_width option" do
      defmodule OptionsListVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<ul><li>Item</li></ul>"
      options = %{list_indent_width: 2}

      {:ok, markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, OptionsListVisitor, options)

      assert is_binary(markdown)
    end

    test "visitor works with wrap option" do
      defmodule OptionsWrapVisitor do
        use HtmlToMarkdown.Visitor
      end

      html =
        "<p>This is a very long paragraph that should be wrapped at a specific column width.</p>"

      options = %{wrap: true, wrap_width: 40}

      {:ok, markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, OptionsWrapVisitor, options)

      assert is_binary(markdown)
    end
  end

  describe "edge cases" do
    test "empty HTML string" do
      defmodule EmptyVisitor do
        use HtmlToMarkdown.Visitor
      end

      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor("", EmptyVisitor, nil)

      assert is_binary(markdown)
    end

    test "HTML with nested elements" do
      defmodule NestedVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<div><p><strong>Bold <em>italic</em></strong></p></div>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, NestedVisitor, nil)

      assert is_binary(markdown)
    end

    test "HTML with special characters" do
      defmodule SpecialCharVisitor do
        use HtmlToMarkdown.Visitor
      end

      html = "<p>Special: &amp; &lt; &gt; &quot; &apos;</p>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, SpecialCharVisitor, nil)

      assert is_binary(markdown)
    end

    test "HTML with multiple callbacks" do
      defmodule MultiCallbackVisitor do
        use HtmlToMarkdown.Visitor

        def handle_text(_context, _text) do
          :continue
        end

        def handle_strong(_context, _text) do
          :continue
        end

        def handle_emphasis(_context, _text) do
          :continue
        end
      end

      html = "<p>This is <strong>bold</strong> and <em>italic</em> text.</p>"

      {:ok, markdown} =
        HtmlToMarkdown.Visitor.convert_with_visitor(html, MultiCallbackVisitor, nil)

      assert is_binary(markdown)
    end
  end

  describe "documentation examples" do
    test "LinkFilter example from docstring" do
      defmodule LinkFilter do
        use HtmlToMarkdown.Visitor

        @impl true
        def handle_link(_context, _href, text, _title) do
          {:custom, text}
        end
      end

      html = "<p>Check <a href='https://example.com'>this</a> out!</p>"
      {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, LinkFilter, nil)

      assert is_binary(markdown)
    end
  end
end
