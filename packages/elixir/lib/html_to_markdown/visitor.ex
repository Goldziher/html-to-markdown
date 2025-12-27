defmodule HtmlToMarkdown.Visitor do
  @moduledoc """
  Visitor pattern support for HTML to Markdown conversion.

  This module provides a callback-based interface that allows you to intervene
  in the HTML→Markdown conversion process at any point. Visitors can inspect,
  modify, or replace the default conversion behavior for any HTML element type.

  ## Overview

  The visitor pattern is useful when you need to:

  - Filter or modify elements during conversion (e.g., remove all links)
  - Collect metadata about specific elements (e.g., all images)
  - Apply custom formatting logic (e.g., style-specific handling)
  - Implement content policies (e.g., sanitize external links)
  - Skip or preserve certain elements as-is

  ## Architecture

  Conversion with a visitor involves:

  1. Define a visitor module with callback implementations
  2. Call `convert_with_visitor/3` with HTML, options, and visitor
  3. The converter dispatches callbacks as it traverses the DOM
  4. Each callback returns a `VisitResult` to control continuation

  ## Visitor Callbacks

  All callbacks receive a `NodeContext` struct with metadata about the current node:

  - `node_type`: Coarse-grained classification (e.g., `:text`, `:link`, `:heading`)
  - `tag_name`: Raw HTML tag name (e.g., "a", "h1", "div")
  - `attributes`: Map of HTML attributes
  - `depth`: Nesting depth in the DOM tree (0 = root)
  - `index_in_parent`: Zero-based index among siblings
  - `parent_tag`: Parent element's tag name (nil if root)
  - `is_inline`: Whether this element is treated as inline vs block

  ## Visit Results

  Each callback must return one of:

  - `:continue` - Proceed with default conversion behavior
  - `{:custom, markdown}` - Replace output with custom markdown string
  - `:skip` - Omit this element entirely (don't output anything)
  - `:preserve_html` - Include raw HTML verbatim in output
  - `{:error, message}` - Stop conversion with an error

  ## Method Naming Convention

  - `handle_*_start`: Called before entering an element (pre-order)
  - `handle_*_end`: Called after exiting an element (post-order)
  - `handle_*`: Called for specific element types (e.g., `handle_link`, `handle_image`)

  ## Execution Order

  For a typical element like `<div><p>text</p></div>`:

  1. `handle_element_start` for `<div>`
  2. `handle_element_start` for `<p>`
  3. `handle_text` for "text"
  4. `handle_element_end` for `<p>`
  5. `handle_element_end` for `</div>`

  ## Example

  ```elixir
  defmodule LinkFilter do
    @behaviour HtmlToMarkdown.Visitor

    @impl true
    def handle_link(_context, _href, text, _title) do
      # Convert all links to plain text
      {:custom, text}
    end

    @impl true
    def handle_other(_callback, _context, _args) do
      :continue
    end
  end

  html = "<p>Check <a href='https://example.com'>this</a> out!</p>"
  {:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, LinkFilter, nil)
  # markdown == "Check this out!\\n"
  ```

  ## GenServer Integration

  For more complex scenarios, you can use a GenServer to maintain state:

  ```elixir
  defmodule ImageCollector do
    @behaviour HtmlToMarkdown.Visitor
    use GenServer

    def start_link(_) do
      GenServer.start_link(__MODULE__, [])
    end

    def init(_) do
      {:ok, []}
    end

    @impl true
    def handle_image(_context, src, alt, _title) do
      # Store image metadata
      GenServer.cast(self(), {:collect_image, src, alt})
      :continue
    end

    @impl true
    def handle_other(_callback, _context, _args) do
      :continue
    end

    def handle_cast({:collect_image, src, alt}, images) do
      {:noreply, [%{src: src, alt: alt} | images]}
    end
  end

  {:ok, pid} = ImageCollector.start_link(nil)
  {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, pid, nil)
  # Can query collected images via GenServer API
  ```
  """

  @type node_type ::
          :text
          | :element
          | :heading
          | :paragraph
          | :div
          | :blockquote
          | :pre
          | :hr
          | :list
          | :list_item
          | :definition_list
          | :definition_term
          | :definition_description
          | :table
          | :table_row
          | :table_cell
          | :table_header
          | :table_body
          | :table_head
          | :table_foot
          | :link
          | :image
          | :strong
          | :em
          | :code
          | :strikethrough
          | :underline
          | :subscript
          | :superscript
          | :mark
          | :small
          | :br
          | :span
          | :article
          | :section
          | :nav
          | :aside
          | :header
          | :footer
          | :main
          | :figure
          | :figcaption
          | :time
          | :details
          | :summary
          | :form
          | :input
          | :select
          | :option
          | :button
          | :textarea
          | :label
          | :fieldset
          | :legend
          | :audio
          | :video
          | :picture
          | :source
          | :iframe
          | :svg
          | :canvas
          | :ruby
          | :rt
          | :rp
          | :abbr
          | :kbd
          | :samp
          | :var
          | :cite
          | :q
          | :del
          | :ins
          | :data
          | :meter
          | :progress
          | :output
          | :template
          | :slot
          | :html
          | :head
          | :body
          | :title
          | :meta
          | :link_tag
          | :style
          | :script
          | :base
          | :custom

  @type node_context :: %{
          node_type: node_type,
          tag_name: String.t(),
          attributes: map(),
          depth: non_neg_integer(),
          index_in_parent: non_neg_integer(),
          parent_tag: String.t() | nil,
          is_inline: boolean()
        }

  @type visit_result ::
          :continue
          | {:custom, String.t()}
          | :skip
          | :preserve_html
          | {:error, String.t()}

  @doc """
  Callback for handling generic element start.

  Called before entering any element.
  """
  @callback handle_element_start(context :: node_context) :: visit_result

  @doc """
  Callback for handling generic element end.

  Called after exiting any element, receives the default markdown output.
  """
  @callback handle_element_end(context :: node_context, output :: String.t()) :: visit_result

  @doc """
  Callback for handling text nodes.

  This is the most frequently called callback (~100+ times per document).
  """
  @callback handle_text(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling anchor links.
  """
  @callback handle_link(
              context :: node_context,
              href :: String.t(),
              text :: String.t(),
              title :: String.t() | nil
            ) :: visit_result

  @doc """
  Callback for handling images.
  """
  @callback handle_image(
              context :: node_context,
              src :: String.t(),
              alt :: String.t(),
              title :: String.t() | nil
            ) :: visit_result

  @doc """
  Callback for handling headings (h1-h6).
  """
  @callback handle_heading(
              context :: node_context,
              level :: 1..6,
              text :: String.t(),
              id :: String.t() | nil
            ) :: visit_result

  @doc """
  Callback for handling code blocks.
  """
  @callback handle_code_block(
              context :: node_context,
              lang :: String.t() | nil,
              code :: String.t()
            ) :: visit_result

  @doc """
  Callback for handling inline code.
  """
  @callback handle_code_inline(context :: node_context, code :: String.t()) :: visit_result

  @doc """
  Callback for handling list items.
  """
  @callback handle_list_item(
              context :: node_context,
              ordered :: boolean(),
              marker :: String.t(),
              text :: String.t()
            ) :: visit_result

  @doc """
  Callback for handling list start.
  """
  @callback handle_list_start(context :: node_context, ordered :: boolean()) :: visit_result

  @doc """
  Callback for handling list end.
  """
  @callback handle_list_end(context :: node_context, ordered :: boolean(), output :: String.t()) ::
              visit_result

  @doc """
  Callback for handling table start.
  """
  @callback handle_table_start(context :: node_context) :: visit_result

  @doc """
  Callback for handling table rows.
  """
  @callback handle_table_row(
              context :: node_context,
              cells :: [String.t()],
              is_header :: boolean()
            ) :: visit_result

  @doc """
  Callback for handling table end.
  """
  @callback handle_table_end(context :: node_context, output :: String.t()) :: visit_result

  @doc """
  Callback for handling blockquotes.
  """
  @callback handle_blockquote(
              context :: node_context,
              content :: String.t(),
              depth :: non_neg_integer()
            ) :: visit_result

  @doc """
  Callback for handling strong/bold elements.
  """
  @callback handle_strong(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling emphasis/italic elements.
  """
  @callback handle_emphasis(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling strikethrough elements.
  """
  @callback handle_strikethrough(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling underline elements.
  """
  @callback handle_underline(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling subscript elements.
  """
  @callback handle_subscript(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling superscript elements.
  """
  @callback handle_superscript(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling mark/highlight elements.
  """
  @callback handle_mark(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling line breaks.
  """
  @callback handle_line_break(context :: node_context) :: visit_result

  @doc """
  Callback for handling horizontal rules.
  """
  @callback handle_horizontal_rule(context :: node_context) :: visit_result

  @doc """
  Callback for handling custom/unknown elements.
  """
  @callback handle_custom_element(
              context :: node_context,
              tag_name :: String.t(),
              html :: String.t()
            ) :: visit_result

  @doc """
  Callback for handling definition list start.
  """
  @callback handle_definition_list_start(context :: node_context) :: visit_result

  @doc """
  Callback for handling definition terms.
  """
  @callback handle_definition_term(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling definition descriptions.
  """
  @callback handle_definition_description(context :: node_context, text :: String.t()) ::
              visit_result

  @doc """
  Callback for handling definition list end.
  """
  @callback handle_definition_list_end(context :: node_context, output :: String.t()) ::
              visit_result

  @doc """
  Callback for handling forms.
  """
  @callback handle_form(
              context :: node_context,
              action :: String.t() | nil,
              method :: String.t() | nil
            ) :: visit_result

  @doc """
  Callback for handling input elements.
  """
  @callback handle_input(
              context :: node_context,
              input_type :: String.t(),
              name :: String.t() | nil,
              value :: String.t() | nil
            ) :: visit_result

  @doc """
  Callback for handling button elements.
  """
  @callback handle_button(context :: node_context, text :: String.t()) :: visit_result

  @doc """
  Callback for handling audio elements.
  """
  @callback handle_audio(context :: node_context, src :: String.t() | nil) :: visit_result

  @doc """
  Callback for handling video elements.
  """
  @callback handle_video(context :: node_context, src :: String.t() | nil) :: visit_result

  @doc """
  Callback for handling iframe elements.
  """
  @callback handle_iframe(context :: node_context, src :: String.t() | nil) :: visit_result

  @doc """
  Catch-all callback for any visitor callback not explicitly implemented.

  This allows you to implement only the callbacks you need while providing
  a default behavior for all others.

  The `callback` parameter is an atom like `:link`, `:heading`, `:element_start`, etc.
  The `args` parameter is a list of the callback arguments.
  """
  @callback handle_other(callback :: atom(), context :: node_context | nil, args :: list()) ::
              visit_result

  @doc false
  defmacro __using__(_opts) do
    quote do
      @behaviour HtmlToMarkdown.Visitor

      @impl true
      def handle_element_start(_context), do: :continue

      @impl true
      def handle_element_end(_context, _output), do: :continue

      @impl true
      def handle_text(_context, _text), do: :continue

      @impl true
      def handle_link(_context, _href, _text, _title), do: :continue

      @impl true
      def handle_image(_context, _src, _alt, _title), do: :continue

      @impl true
      def handle_heading(_context, _level, _text, _id), do: :continue

      @impl true
      def handle_code_block(_context, _lang, _code), do: :continue

      @impl true
      def handle_code_inline(_context, _code), do: :continue

      @impl true
      def handle_list_item(_context, _ordered, _marker, _text), do: :continue

      @impl true
      def handle_list_start(_context, _ordered), do: :continue

      @impl true
      def handle_list_end(_context, _ordered, _output), do: :continue

      @impl true
      def handle_table_start(_context), do: :continue

      @impl true
      def handle_table_row(_context, _cells, _is_header), do: :continue

      @impl true
      def handle_table_end(_context, _output), do: :continue

      @impl true
      def handle_blockquote(_context, _content, _depth), do: :continue

      @impl true
      def handle_strong(_context, _text), do: :continue

      @impl true
      def handle_emphasis(_context, _text), do: :continue

      @impl true
      def handle_strikethrough(_context, _text), do: :continue

      @impl true
      def handle_underline(_context, _text), do: :continue

      @impl true
      def handle_subscript(_context, _text), do: :continue

      @impl true
      def handle_superscript(_context, _text), do: :continue

      @impl true
      def handle_mark(_context, _text), do: :continue

      @impl true
      def handle_line_break(_context), do: :continue

      @impl true
      def handle_horizontal_rule(_context), do: :continue

      @impl true
      def handle_custom_element(_context, _tag_name, _html), do: :continue

      @impl true
      def handle_definition_list_start(_context), do: :continue

      @impl true
      def handle_definition_term(_context, _text), do: :continue

      @impl true
      def handle_definition_description(_context, _text), do: :continue

      @impl true
      def handle_definition_list_end(_context, _output), do: :continue

      @impl true
      def handle_form(_context, _action, _method), do: :continue

      @impl true
      def handle_input(_context, _input_type, _name, _value), do: :continue

      @impl true
      def handle_button(_context, _text), do: :continue

      @impl true
      def handle_audio(_context, _src), do: :continue

      @impl true
      def handle_video(_context, _src), do: :continue

      @impl true
      def handle_iframe(_context, _src), do: :continue

      @impl true
      def handle_other(_callback, _context, _args), do: :continue

      defoverridable handle_element_start: 1,
                     handle_element_end: 2,
                     handle_text: 2,
                     handle_link: 4,
                     handle_image: 4,
                     handle_heading: 4,
                     handle_code_block: 3,
                     handle_code_inline: 2,
                     handle_list_item: 4,
                     handle_list_start: 2,
                     handle_list_end: 3,
                     handle_table_start: 1,
                     handle_table_row: 3,
                     handle_table_end: 2,
                     handle_blockquote: 3,
                     handle_strong: 2,
                     handle_emphasis: 2,
                     handle_strikethrough: 2,
                     handle_underline: 2,
                     handle_subscript: 2,
                     handle_superscript: 2,
                     handle_mark: 2,
                     handle_line_break: 1,
                     handle_horizontal_rule: 1,
                     handle_custom_element: 3,
                     handle_definition_list_start: 1,
                     handle_definition_term: 2,
                     handle_definition_description: 2,
                     handle_definition_list_end: 2,
                     handle_form: 3,
                     handle_input: 4,
                     handle_button: 2,
                     handle_audio: 2,
                     handle_video: 2,
                     handle_iframe: 2,
                     handle_other: 3
    end
  end

  @doc """
  Convert HTML to Markdown with visitor callbacks.

  This function performs HTML-to-Markdown conversion while dispatching callbacks
  to the provided visitor module or process for customization.

  ## Parameters

  - `html` - HTML string to convert
  - `visitor` - Visitor module or PID to receive callbacks
  - `options` - Optional conversion options map (see `HtmlToMarkdown.Options`)

  ## Returns

  - `{:ok, markdown}` - Successful conversion with resulting Markdown
  - `{:error, reason}` - Conversion failed with error reason

  ## Examples

  ```elixir
  defmodule MyVisitor do
    use HtmlToMarkdown.Visitor

    def handle_link(_context, href, text, _title) do
      # Format links using standard Markdown syntax
      {:custom, "[" <> text <> "](" <> href <> ")"}
    end
  end

  # Use the visitor during conversion
  html = "<a href='#'>Click me</a>"
  {:ok, _markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, MyVisitor, nil)
  ```
  """
  @spec convert_with_visitor(
          html :: String.t(),
          visitor :: atom() | pid(),
          options :: map() | nil
        ) :: {:ok, String.t()} | {:error, String.t()}
  def convert_with_visitor(html, visitor, options \\ nil) when is_binary(html) do
    native_options = HtmlToMarkdown.options(options)

    case HtmlToMarkdown.Native.convert_with_visitor(html, native_options, visitor) do
      {:ok, markdown} -> {:ok, markdown}
      {:error, reason} -> {:error, to_string(reason)}
    end
  end
end
