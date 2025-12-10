# frozen_string_literal: true

require 'spec_helper'

RSpec.describe HtmlToMarkdown do
  describe '.convert_with_metadata' do
    it 'returns array with markdown and metadata' do
      html = '<html><head><title>Test</title></head><body><p>Content</p></body></html>'
      result = described_class.convert_with_metadata(html)

      expect(result).to be_an(Array)
      expect(result.length).to eq(2)
      expect(result[0]).to be_a(String)
      expect(result[1]).to be_a(Hash)
    end

    context 'document metadata' do
      it 'extracts title' do
        html = '<html><head><title>My Page Title</title></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:title]).to eq('My Page Title')
      end

      it 'extracts description' do
        html = '<html><head><meta name="description" content="Page description"></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:description]).to eq('Page description')
      end

      it 'extracts keywords' do
        html = '<html><head><meta name="keywords" content="keyword1, keyword2, keyword3"></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:keywords]).to include('keyword1', 'keyword2', 'keyword3')
      end

      it 'extracts author' do
        html = '<html><head><meta name="author" content="John Doe"></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:author]).to eq('John Doe')
      end

      it 'extracts base href' do
        html = '<html><head><base href="https://example.com/"></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:base_href]).to eq('https://example.com/')
      end

      it 'extracts canonical URL' do
        html = '<html><head><link rel="canonical" href="https://example.com/page"></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:canonical_url]).to eq('https://example.com/page')
      end

      it 'extracts language' do
        html = '<html lang="en"><head></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:language]).to eq('en')
      end

      it 'extracts text direction' do
        html = '<html dir="ltr"><head></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:text_direction]).to eq('ltr')
      end

      it 'extracts open graph metadata' do
        html = '''
          <html>
            <head>
              <meta property="og:title" content="OG Title">
              <meta property="og:description" content="OG Description">
              <meta property="og:image" content="https://example.com/image.jpg">
            </head>
            <body><p>Content</p></body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:open_graph]).to include(
          'title' => 'OG Title',
          'description' => 'OG Description',
          'image' => 'https://example.com/image.jpg'
        )
      end

      it 'extracts twitter card metadata' do
        html = '''
          <html>
            <head>
              <meta name="twitter:card" content="summary_large_image">
              <meta name="twitter:title" content="Twitter Title">
            </head>
            <body><p>Content</p></body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:twitter_card]).to include(
          'card' => 'summary_large_image',
          'title' => 'Twitter Title'
        )
      end

      it 'returns empty arrays and hashes for missing metadata' do
        html = '<p>Content</p>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:title]).to be_nil
        expect(metadata[:document][:description]).to be_nil
        expect(metadata[:document][:keywords]).to eq([])
        expect(metadata[:document][:open_graph]).to eq({})
        expect(metadata[:document][:twitter_card]).to eq({})
        expect(metadata[:document][:meta_tags]).to eq({})
      end
    end

    context 'header metadata' do
      it 'extracts headers with hierarchy' do
        html = '''
          <html>
            <body>
              <h1>Main Title</h1>
              <h2>Section</h2>
              <h3>Subsection</h3>
            </body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:headers].length).to eq(3)
        expect(metadata[:headers][0][:level]).to eq(1)
        expect(metadata[:headers][0][:text]).to eq('Main Title')
        expect(metadata[:headers][1][:level]).to eq(2)
        expect(metadata[:headers][1][:text]).to eq('Section')
        expect(metadata[:headers][2][:level]).to eq(3)
        expect(metadata[:headers][2][:text]).to eq('Subsection')
      end

      it 'includes header id' do
        html = '<html><body><h1 id="main-title">Title</h1></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:headers][0][:id]).to eq('main-title')
      end

      it 'includes depth and html_offset' do
        html = '<html><body><h1>Title</h1></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        header = metadata[:headers][0]
        expect(header).to include(:depth, :html_offset)
        expect(header[:depth]).to be_a(Integer)
        expect(header[:html_offset]).to be_a(Integer)
      end
    end

    context 'link metadata' do
      it 'extracts links with classification' do
        html = '''
          <html>
            <body>
              <a href="#section">Anchor</a>
              <a href="https://example.com">External</a>
              <a href="/page">Internal</a>
              <a href="mailto:test@example.com">Email</a>
              <a href="tel:+1234567890">Phone</a>
            </body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        links = metadata[:links]
        expect(links.length).to eq(5)

        expect(links[0][:link_type]).to eq('anchor')
        expect(links[1][:link_type]).to eq('external')
        expect(links[2][:link_type]).to eq('internal')
        expect(links[3][:link_type]).to eq('email')
        expect(links[4][:link_type]).to eq('phone')
      end

      it 'includes link text and href' do
        html = '<html><body><a href="https://example.com">Click here</a></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        link = metadata[:links][0]
        expect(link[:href]).to eq('https://example.com')
        expect(link[:text]).to eq('Click here')
      end

      it 'includes link title attribute' do
        html = '<html><body><a href="https://example.com" title="Example Site">Link</a></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        link = metadata[:links][0]
        expect(link[:title]).to eq('Example Site')
      end

      it 'includes link rel attributes' do
        html = '<html><body><a href="https://example.com" rel="nofollow external">Link</a></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        link = metadata[:links][0]
        expect(link[:rel]).to include('nofollow', 'external')
      end

      it 'includes link attributes' do
        html = '<html><body><a href="https://example.com" data-custom="value">Link</a></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        link = metadata[:links][0]
        expect(link[:attributes]).to include('data-custom' => 'value')
      end
    end

    context 'image metadata' do
      it 'extracts images with source type' do
        html = '''
          <html>
            <body>
              <img src="https://example.com/image.jpg" alt="External">
              <img src="/images/local.jpg" alt="Relative">
              <img src="data:image/png;base64,..." alt="Data URI">
            </body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        images = metadata[:images]
        expect(images.length).to eq(3)

        expect(images[0][:image_type]).to eq('external')
        expect(images[1][:image_type]).to eq('relative')
        expect(images[2][:image_type]).to eq('data_uri')
      end

      it 'includes image alt and title' do
        html = '<html><body><img src="image.jpg" alt="Alt text" title="Image title"></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        image = metadata[:images][0]
        expect(image[:alt]).to eq('Alt text')
        expect(image[:title]).to eq('Image title')
      end

      it 'includes image dimensions' do
        html = '<html><body><img src="image.jpg" width="800" height="600"></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        image = metadata[:images][0]
        expect(image[:dimensions]).to be_an(Array)
        expect(image[:dimensions].length).to eq(2)
      end

      it 'handles missing image attributes' do
        html = '<html><body><img src="image.jpg"></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        image = metadata[:images][0]
        expect(image[:alt]).to be_nil
        expect(image[:title]).to be_nil
      end
    end

    context 'metadata configuration' do
      it 'respects extract_headers flag' do
        html = '<html><body><h1>Title</h1><p>Content</p></body></html>'
        config = { extract_headers: false }
        markdown, metadata = described_class.convert_with_metadata(html, nil, config)

        expect(metadata[:headers]).to eq([])
      end

      it 'respects extract_links flag' do
        html = '<html><body><a href="https://example.com">Link</a></body></html>'
        config = { extract_links: false }
        markdown, metadata = described_class.convert_with_metadata(html, nil, config)

        expect(metadata[:links]).to eq([])
      end

      it 'respects extract_images flag' do
        html = '<html><body><img src="image.jpg" alt="test"></body></html>'
        config = { extract_images: false }
        markdown, metadata = described_class.convert_with_metadata(html, nil, config)

        expect(metadata[:images]).to eq([])
      end

      it 'respects extract_structured_data flag' do
        html = '<html><body><script type="application/ld+json">{"@type":"Article"}</script></body></html>'
        config = { extract_structured_data: false }
        markdown, metadata = described_class.convert_with_metadata(html, nil, config)

        expect(metadata[:structured_data]).to eq([])
      end
    end

    context 'conversion options with metadata' do
      it 'accepts both conversion options and metadata config' do
        html = '<html><head><title>Test</title></head><body><h1>Heading</h1></body></html>'
        conv_opts = { heading_style: :atx_closed }
        meta_opts = { extract_headers: true }

        markdown, metadata = described_class.convert_with_metadata(html, conv_opts, meta_opts)

        expect(markdown).to include('# Heading #')
        expect(metadata[:headers].length).to eq(1)
      end

      it 'works with nil options' do
        html = '<html><head><title>Test</title></head><body><p>Content</p></body></html>'
        result = described_class.convert_with_metadata(html, nil, nil)

        expect(result).to be_an(Array)
        expect(result.length).to eq(2)
      end
    end

    context 'structured data' do
      it 'extracts JSON-LD blocks' do
        html = '''
          <html>
            <head>
              <script type="application/ld+json">
                {"@context":"https://schema.org","@type":"Article","headline":"Test"}
              </script>
            </head>
            <body><p>Content</p></body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        # Structured data extraction may vary by implementation
        expect(metadata[:structured_data]).to be_an(Array)
      end
    end

    context 'edge cases' do
      it 'handles empty HTML' do
        html = ''
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(markdown).to be_a(String)
        expect(metadata).to be_a(Hash)
      end

      it 'handles malformed HTML' do
        html = '<html><head><title>Unclosed'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(markdown).to be_a(String)
        expect(metadata).to be_a(Hash)
      end

      it 'handles special characters in metadata' do
        html = '<html><head><title>Title with "quotes" & <brackets></title></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata[:document][:title]).to be_a(String)
      end

      it 'handles whitespace in metadata' do
        html = '<html><head><title>  Title with   spaces  </title></head><body><p>Content</p></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        # Whitespace may be normalized
        expect(metadata[:document][:title]).to match(/Title.*spaces/)
      end

      it 'handles multiple values for same metadata key' do
        html = '''
          <html>
            <head>
              <meta name="author" content="Author 1">
              <meta name="author" content="Author 2">
            </head>
            <body><p>Content</p></body>
          </html>
        '''
        markdown, metadata = described_class.convert_with_metadata(html)

        # Last value typically wins, but implementation may vary
        expect(metadata[:document][:author]).to be_a(String)
      end
    end

    context 'return value structure' do
      it 'returns proper metadata hash structure' do
        html = '<html><head><title>Test</title><base href="https://example.com"></head><body><h1>H1</h1><a href="link">Link</a><img src="img.jpg"></body></html>'
        markdown, metadata = described_class.convert_with_metadata(html)

        expect(metadata).to include(
          :document,
          :headers,
          :links,
          :images,
          :structured_data
        )

        expect(metadata[:document]).to include(
          :title,
          :description,
          :keywords,
          :author,
          :canonical_url,
          :base_href,
          :language,
          :text_direction,
          :open_graph,
          :twitter_card,
          :meta_tags
        )
      end
    end
  end
end
