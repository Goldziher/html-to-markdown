# frozen_string_literal: true

# rubocop:disable RSpec/ContextWording, RSpec/VerifiedDoubles
require 'spec_helper'

RSpec.describe HtmlToMarkdown do
  describe '.convert_with_visitor' do
    # ============================================================================
    # ============================================================================

    def create_visitor(**overrides)
      visitor = double(Object)

      default_methods = {
        visit_element_start: { type: :continue },
        visit_element_end: { type: :continue },
        visit_text: { type: :continue },
        visit_link: { type: :continue },
        visit_image: { type: :continue },
        visit_heading: { type: :continue },
        visit_code_block: { type: :continue },
        visit_code_inline: { type: :continue },
        visit_list_item: { type: :continue },
        visit_list_start: { type: :continue },
        visit_list_end: { type: :continue },
        visit_table_start: { type: :continue },
        visit_table_row: { type: :continue },
        visit_table_end: { type: :continue },
        visit_blockquote: { type: :continue },
        visit_strong: { type: :continue },
        visit_emphasis: { type: :continue },
        visit_strikethrough: { type: :continue },
        visit_underline: { type: :continue },
        visit_subscript: { type: :continue },
        visit_superscript: { type: :continue },
        visit_mark: { type: :continue },
        visit_line_break: { type: :continue },
        visit_horizontal_rule: { type: :continue },
        visit_custom_element: { type: :continue },
        visit_definition_list_start: { type: :continue },
        visit_definition_term: { type: :continue },
        visit_definition_description: { type: :continue },
        visit_definition_list_end: { type: :continue },
        visit_form: { type: :continue },
        visit_input: { type: :continue },
        visit_button: { type: :continue },
        visit_audio: { type: :continue },
        visit_video: { type: :continue },
        visit_iframe: { type: :continue },
        visit_details: { type: :continue },
        visit_summary: { type: :continue },
        visit_figure_start: { type: :continue },
        visit_figcaption: { type: :continue },
        visit_figure_end: { type: :continue },
      }

      default_methods.each do |method_name, return_value|
        allow(visitor).to receive(method_name).and_return(return_value)
      end

      overrides.each do |method_name, behavior|
        if behavior.is_a?(Proc)
          allow(visitor).to receive(method_name, &behavior)
        else
          allow(visitor).to receive(method_name).and_return(behavior)
        end
      end

      visitor
    end


    context 'visit_text callback' do
      it 'is called for text nodes' do
        html = '<p>Hello World</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :continue })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_text)
        expect(result).to include('Hello World')
      end

      it 'receives text content and context' do
        html = '<p>Test content</p>'
        visited_texts = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          visited_texts << text
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visited_texts).to include('Test content')
      end

      it 'validates node context for text node' do
        html = '<p>Hello</p>'
        visitor = create_visitor
        captured_ctx = nil

        allow(visitor).to receive(:visit_text) do |ctx, _text|
          captured_ctx = ctx
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(captured_ctx).not_to be_nil
        expect(captured_ctx[:tag_name]).to be_a(String)
        expect(captured_ctx[:depth]).to be_an(Integer)
      end
    end

    context 'visit_link callback' do
      it 'is called for anchor links' do
        html = '<a href="https://example.com">Click here</a>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_link).and_return({ type: :continue })

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_link)
      end

      it 'receives href, text, and optional title' do
        html = '<a href="https://example.com" title="Example">Click</a>'
        link_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_link) do |ctx, href, text, title|
          link_data = { ctx: ctx, href: href, text: text, title: title }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(link_data).not_to be_nil
        expect(link_data[:href]).to eq('https://example.com')
        expect(link_data[:text]).to eq('Click')
        expect(link_data[:title]).to eq('Example')
      end

      it 'handles links without title attribute' do
        html = '<a href="/path">Link</a>'
        link_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_link) do |_ctx, href, text, title|
          link_data = { href: href, text: text, title: title }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(link_data[:title]).to be_nil
      end

      it 'validates node context contains link metadata' do
        html = '<a href="https://example.com">Link</a>'
        visitor = create_visitor
        captured_ctx = nil

        allow(visitor).to receive(:visit_link) do |ctx, _href, _text, _title|
          captured_ctx = ctx
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(captured_ctx[:attributes]).to be_a(Hash)
        expect(captured_ctx[:attributes]['href']).to eq('https://example.com')
      end
    end

    context 'visit_image callback' do
      it 'is called for image elements' do
        html = '<img src="image.jpg" alt="An image">'
        visitor = create_visitor

        allow(visitor).to receive(:visit_image).and_return({ type: :continue })

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_image)
      end

      it 'receives src, alt, and optional title' do
        html = '<img src="photo.jpg" alt="Beautiful" title="Photo">'
        image_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_image) do |ctx, src, alt, title|
          image_data = { ctx: ctx, src: src, alt: alt, title: title }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(image_data[:src]).to eq('photo.jpg')
        expect(image_data[:alt]).to eq('Beautiful')
        expect(image_data[:title]).to eq('Photo')
      end

      it 'handles images without title attribute' do
        html = '<img src="pic.png" alt="Picture">'
        image_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_image) do |_ctx, src, alt, title|
          image_data = { src: src, alt: alt, title: title }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(image_data[:title]).to be_nil
      end
    end

    context 'visit_heading callback' do
      it 'is called for heading elements' do
        html = '<h1>Title</h1>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading).and_return({ type: :continue })

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_heading)
      end

      it 'receives heading level, text, and optional id' do
        html = '<h2 id="section">Chapter</h2>'
        heading_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading) do |ctx, level, text, id|
          heading_data = { ctx: ctx, level: level, text: text, id: id }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(heading_data[:level]).to eq(2)
        expect(heading_data[:text]).to eq('Chapter')
        expect(heading_data[:id]).to eq('section')
      end

      it 'handles headings without id attribute' do
        html = '<h3>Subsection</h3>'
        heading_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading) do |_ctx, level, text, id|
          heading_data = { level: level, text: text, id: id }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(heading_data[:id]).to be_nil
      end

      it 'supports all heading levels (h1-h6)' do
        heading_levels = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading) do |_ctx, level, _text, _id|
          heading_levels << level
          { type: :continue }
        end

        html = '<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>'
        described_class.convert_with_visitor(html, nil, visitor)

        expect(heading_levels).to contain_exactly(1, 2, 3, 4, 5, 6)
      end
    end

    context 'visit_element_start callback' do
      it 'is called when entering an element' do
        html = '<div><p>Content</p></div>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start).and_return({ type: :continue })

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_element_start).at_least(:once)
      end

      it 'receives node context with tag information' do
        html = '<section id="main" class="container">Text</section>'
        contexts = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          contexts << ctx
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        section_ctx = contexts.find { |ctx| ctx[:tag_name] == 'section' }
        expect(section_ctx).not_to be_nil
        expect(section_ctx[:attributes]['id']).to eq('main')
      end
    end

    context 'visit_element_end callback' do
      it 'is called when exiting an element' do
        html = '<div>Content</div>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_end).and_return({ type: :continue })

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_element_end)
      end

      it 'receives context and generated output' do
        html = '<p>Text content</p>'
        element_end_data = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_end) do |ctx, output|
          element_end_data = { ctx: ctx, output: output }
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(element_end_data).not_to be_nil
        expect(element_end_data[:output]).to be_a(String)
      end
    end


    context 'VisitResult::Continue' do
      it 'continues with default behavior' do
        html = '<p>Hello</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :continue })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('Hello')
      end

      it 'allows chaining of multiple visitors' do
        html = '<p>Test</p>'
        calls = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          calls << text
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)
        expect(calls).to include('Test')
      end
    end

    context 'VisitResult::Custom' do
      it 'replaces element output with custom text' do
        html = '<p>Original</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :custom, output: 'MODIFIED' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('MODIFIED')
      end

      it 'overrides link rendering' do
        html = '<a href="https://example.com">Link</a>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_link).and_return({ type: :custom, output: '**CUSTOM LINK**' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('CUSTOM LINK')
      end

      it 'overrides image rendering' do
        html = '<img src="test.jpg" alt="Test">'
        visitor = create_visitor

        allow(visitor).to receive(:visit_image).and_return({ type: :custom, output: '[IMAGE PLACEHOLDER]' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('IMAGE PLACEHOLDER')
      end

      it 'overrides heading rendering' do
        html = '<h1>Title</h1>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading).and_return({ type: :custom, output: '>>> TITLE <<<' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('TITLE')
      end

      it 'supports unicode in custom output' do
        html = '<p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :custom, output: '✓ Custom ✨' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('✓')
        expect(result).to include('✨')
      end
    end

    context 'VisitResult::Skip' do
      it 'removes element from output entirely' do
        html = '<p>Keep</p><img src="skip.jpg" alt="Skip">'
        visitor = create_visitor

        allow(visitor).to receive(:visit_image).and_return({ type: :skip })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('Keep')
        expect(result).not_to include('skip')
      end

      it 'skips link entirely' do
        html = '<p>Before <a href="#">hidden</a> after</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_link).and_return({ type: :skip })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('Before')
        expect(result).to include('after')
      end

      it 'skips multiple elements selectively' do
        html = '<p>1</p><img src="a.jpg" alt="A"><p>2</p><img src="b.jpg" alt="B"><p>3</p>'
        image_count = 0
        visitor = create_visitor

        allow(visitor).to receive(:visit_image) do
          image_count += 1
          { type: :skip }
        end

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(image_count).to eq(2)
        expect(result).to include('1')
        expect(result).to include('2')
        expect(result).to include('3')
      end
    end

    context 'VisitResult::PreserveHtml' do
      it 'preserves element as raw HTML in output' do
        html = '<p>Text <span class="custom">styled</span> here</p>'
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to be_a(String)
      end

      it 'preserves links as HTML' do
        html = '<p><a href="javascript:alert()">Click</a></p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_link).and_return({ type: :preserve_html })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to be_a(String)
      end
    end

    context 'VisitResult::Error' do
      it 'stops conversion with error message' do
        html = '<p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :error, message: 'Custom conversion error' })

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.to raise_error(StandardError)
      end

      it 'includes custom error message' do
        html = '<img src="invalid" alt="Bad">'
        visitor = create_visitor

        allow(visitor).to receive(:visit_image).and_return({ type: :error, message: 'Unsupported image format' })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue }
        )

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.to raise_error(StandardError) { |err| expect(err.message).to include('Unsupported') }
      end

      it 'halts conversion at error point' do
        html = '<h1>Title</h1><p>Paragraph</p>'
        visited_elements = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading) do |_ctx, _level, _text, _id|
          visited_elements << :heading
          { type: :error, message: 'Stop here' }
        end

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.to raise_error(StandardError)

        expect(visited_elements).to include(:heading)
      end
    end


    context 'NodeContext validation' do
      it 'provides tag_name in context' do
        html = '<article>Content</article>'
        contexts = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          contexts << ctx
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        article_ctx = contexts.find { |ctx| ctx[:tag_name] == 'article' }
        expect(article_ctx[:tag_name]).to eq('article')
      end

      it 'provides attributes hash in context' do
        html = '<div data-id="123" class="box">Content</div>'
        visitor = create_visitor
        captured_ctx = nil

        allow(visitor).to receive(:visit_element_start) do |ctx|
          captured_ctx = ctx if ctx[:tag_name] == 'div'
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(captured_ctx[:attributes]).to be_a(Hash)
        expect(captured_ctx[:attributes]['data-id']).to eq('123')
        expect(captured_ctx[:attributes]['class']).to eq('box')
      end

      it 'provides depth information' do
        html = '<div><section><p>Nested</p></section></div>'
        depths = {}
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          depths[ctx[:tag_name]] = ctx[:depth]
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(depths['div']).to be < depths['section']
        expect(depths['section']).to be < depths['p']
      end

      it 'provides parent_tag information' do
        html = '<ul><li>Item</li></ul>'
        visitor = create_visitor
        li_parent = nil

        allow(visitor).to receive(:visit_element_start) do |ctx|
          li_parent = ctx[:parent_tag] if ctx[:tag_name] == 'li'
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(li_parent).to eq('ul')
      end

      it 'provides is_inline flag' do
        html = '<p><strong>Bold</strong> and <em>italic</em></p>'
        inline_elements = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          inline_elements << ctx[:tag_name] if ctx[:is_inline]
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(inline_elements).to include('strong')
        expect(inline_elements).to include('em')
      end

      it 'provides index_in_parent information' do
        html = '<ol><li>First</li><li>Second</li><li>Third</li></ol>'
        indices = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          indices << ctx[:index_in_parent] if ctx[:tag_name] == 'li'
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)

        expect(indices.length).to eq(3)
        expect(indices).to include(0, 1, 2)
      end
    end


    context 'error handling' do
      it 'handles visitor exceptions gracefully' do
        html = '<p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text) do
          raise 'Visitor error'
        end

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.to raise_error(RuntimeError)
      end

      it 'handles nil visitor gracefully' do
        html = '<p>Content</p>'
        result = described_class.convert_with_visitor(html, nil, nil)
        expect(result).to include('Content')
      end

      it 'handles missing visitor methods' do
        html = '<p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue }
        )

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.not_to raise_error
      end
    end


    context 'integration with ConversionOptions' do
      it 'accepts ConversionOptions with visitor' do
        html = '<h1>Title</h1>'
        options = described_class.options(heading_style: :atx)
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue },
          visit_heading: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, options, visitor)
        expect(result).to include('# Title')
      end

      it 'accepts options hash with visitor' do
        html = '<h2>Heading</h2>'
        options = { heading_style: :atx }
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue },
          visit_heading: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, options, visitor)
        expect(result).to include('## Heading')
      end

      it 'respects heading_style in options with visitor override' do
        html = '<h1>Title</h1>'
        options = { heading_style: :atx_closed }
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue },
          visit_heading: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, options, visitor)
        expect(result).to include('#')
      end
    end


    context 'multiple visitor methods' do
      it 'calls multiple methods for complex HTML' do
        html = '<h1>Title</h1><p>Text with <a href="#link">link</a> and <img src="pic.jpg" alt="pic"></p>'
        calls = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_heading) do |_ctx, _level, _text, _id|
          calls << :heading
          { type: :continue }
        end

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          calls << :text unless text.strip.empty?
          { type: :continue }
        end

        allow(visitor).to receive(:visit_link) do |_ctx, _href, _text, _title|
          calls << :link
          { type: :continue }
        end

        allow(visitor).to receive(:visit_image) do |_ctx, _src, _alt, _title|
          calls << :image
          { type: :continue }
        end

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)

        expect(calls).to include(:heading)
        expect(calls).to include(:link)
        expect(calls).to include(:image)
      end

      it 'allows selective overrides of specific callbacks' do
        html = '<h1>Title</h1><p><a href="#">Link</a></p>'
        visitor = create_visitor

        allow(visitor).to receive_messages(visit_heading: { type: :custom, output: '>>> HEADING <<<' },
                                           visit_text: { type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_link: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('HEADING')
        expect(result).to include('Link')
      end

      it 'supports different results for different elements' do
        html = '<h1>Header</h1><img src="skip.jpg" alt="skip"><p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive_messages(visit_heading: { type: :custom, output: 'CUSTOM HEADING' },
                                           visit_image: { type: :skip }, visit_text: { type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('CUSTOM HEADING')
        expect(result).to include('Text')
      end
    end


    context 'nested elements' do
      it 'visits deeply nested elements in order' do
        html = '<div><ul><li><strong>Nested <em>content</em></strong></li></ul></div>'
        visited_tags = []
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          visited_tags << ctx[:tag_name]
          { type: :continue }
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)

        expect(visited_tags).to include('div')
        expect(visited_tags).to include('ul')
        expect(visited_tags).to include('li')
        expect(visited_tags).to include('strong')
        expect(visited_tags).to include('em')
      end

      it 'provides correct depth for nested elements' do
        html = '<div><div><p>Deep</p></div></div>'
        depths = {}
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          depths[ctx[:tag_name]] ||= []
          depths[ctx[:tag_name]] << ctx[:depth]
          { type: :continue }
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)

        expect(depths['div'].first).to be < depths['div'].last
        expect(depths['p'].first).to be > depths['div'].last
      end

      it 'handles custom output in nested context' do
        html = '<ul><li><a href="#">link</a></li></ul>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_link).and_return({ type: :custom, output: '[MODIFIED]' })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('[MODIFIED]')
      end

      it 'allows skipping nested elements' do
        html = '<div><p>Keep</p><span>Skip this</span><p>Keep too</p></div>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_element_start) do |ctx|
          if ctx[:tag_name] == 'span'
            { type: :skip }
          else
            { type: :continue }
          end
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('Keep')
        expect(result).to include('Keep too')
      end
    end


    context 'less common visitor methods' do
      it 'calls visit_strong for bold elements' do
        html = '<strong>Bold</strong>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_strong).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_strong)
      end

      it 'calls visit_emphasis for italic elements' do
        html = '<em>Italic</em>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_emphasis).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_emphasis)
      end

      it 'calls visit_code_block for pre/code' do
        html = '<pre><code>function() {}</code></pre>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_code_block).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_code_block)
      end

      it 'calls visit_blockquote for quotes' do
        html = '<blockquote>Quote text</blockquote>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_blockquote).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_blockquote)
      end

      it 'calls visit_list_item for list items' do
        html = '<ul><li>Item</li></ul>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_list_item).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue },
          visit_list_start: { type: :continue },
          visit_list_end: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(visitor).to have_received(:visit_list_item)
      end
    end


    context 'unicode and special characters' do
      it 'handles unicode text in visitor' do
        html = '<p>日本語テキスト</p>'
        text_received = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          text_received = text
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)
        expect(text_received).to eq('日本語テキスト')
      end

      it 'handles unicode in custom output' do
        html = '<p>Text</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_text).and_return({ type: :custom, output: '引用：引用' })

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('引用')
      end

      it 'handles HTML entities in visitor' do
        html = '<p>&lt;code&gt; and &amp; symbols</p>'
        text_received = nil
        visitor = create_visitor

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          text_received = text
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)
        expect(text_received).to include('<code>')
        expect(text_received).to include('&')
      end
    end


    context 'state management in visitor' do
      it 'allows visitor to maintain state across calls' do
        html = '<p>One</p><p>Two</p><p>Three</p>'
        visitor = create_visitor
        element_count = 0

        allow(visitor).to receive(:visit_element_start) do |ctx|
          element_count += 1 if ctx[:tag_name] == 'p'
          { type: :continue }
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        described_class.convert_with_visitor(html, nil, visitor)
        expect(element_count).to eq(3)
      end

      it 'allows visitor to conditionally modify based on accumulated state' do
        html = '<p>A</p><p>B</p><p>C</p>'
        visitor = create_visitor
        paragraph_count = 0

        allow(visitor).to receive(:visit_element_start) do |ctx|
          if ctx[:tag_name] == 'p'
            paragraph_count += 1
            if paragraph_count == 2
              { type: :custom, output: '[SECOND_PARAGRAPH]' }
            else
              { type: :continue }
            end
          else
            { type: :continue }
          end
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('[SECOND_PARAGRAPH]')
      end
    end


    context 'edge cases' do
      it 'handles empty HTML' do
        html = ''
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to be_a(String)
      end

      it 'handles HTML with only whitespace' do
        html = '   \n\t   '
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to be_a(String)
      end

      it 'handles very long text content' do
        long_text = 'A' * 10_000
        html = "<p>#{long_text}</p>"
        visitor = create_visitor
        text_received = nil

        allow(visitor).to receive(:visit_text) do |_ctx, text|
          text_received = text
          { type: :continue }
        end

        described_class.convert_with_visitor(html, nil, visitor)
        expect(text_received.length).to eq(10_000)
      end

      it 'handles deeply nested HTML (stress test)' do
        html = "#{'<div>' * 50}Deep#{'</div>' * 50}"
        visitor = create_visitor
        element_count = 0

        allow(visitor).to receive(:visit_element_start) do |_ctx|
          element_count += 1
          { type: :continue }
        end

        allow(visitor).to receive_messages(
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(element_count).to be > 0
        expect(result).to include('Deep')
      end

      it 'handles self-closing tags' do
        html = '<p>Before<br/>After</p>'
        visitor = create_visitor

        allow(visitor).to receive(:visit_line_break).and_return({ type: :continue })

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        result = described_class.convert_with_visitor(html, nil, visitor)
        expect(result).to include('Before')
        expect(result).to include('After')
      end

      it 'handles malformed HTML gracefully' do
        html = '<p>Unclosed <div>tag<p>Another'
        visitor = create_visitor

        allow(visitor).to receive_messages(
          visit_element_start: { type: :continue },
          visit_element_end: { type: :continue },
          visit_text: { type: :continue }
        )

        expect do
          described_class.convert_with_visitor(html, nil, visitor)
        end.not_to raise_error
      end
    end
  end
end
# rubocop:enable RSpec/ContextWording, RSpec/VerifiedDoubles
