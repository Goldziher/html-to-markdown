# frozen_string_literal: true

require 'spec_helper'

RSpec.describe HtmlToMarkdown do
  describe '.convert' do
    it 'converts simple headings' do
      expect(described_class.convert('<h1>Hello</h1>')).to eq("# Hello\n")
    end

    it 'accepts options hash' do
      result = described_class.convert(
        '<h1>Hello</h1>',
        heading_style: :atx_closed,
        default_title: true
      )
      expect(result).to include('Hello')
    end
  end

  describe '.convert_with_inline_images' do
    it 'returns inline images metadata' do
      html = '<p><img src="data:image/png;base64,ZmFrZQ==" alt="fake"></p>'
      extraction = described_class.convert_with_inline_images(html)
      expect(extraction).to include(:markdown, :inline_images, :warnings)
      expect(extraction[:inline_images].first[:description]).to eq('fake')
    end
  end

  describe '.options' do
    it 'returns a reusable options handle' do
      handle = described_class.options(heading_style: :atx_closed)
      expect(handle).to be_a(HtmlToMarkdown::Options)
      result = described_class.convert_with_options('<h1>Hello</h1>', handle)
      expect(result).to include('# Hello #')
    end
  end

  describe 'panic handling' do
    context 'when a Rust panic would occur' do
      it 'catches panics in convert method' do
        malformed_html = "#{'<' * 100_000}div#{'>' * 100_000}"

        begin
          result = described_class.convert(malformed_html)
          expect(result).to be_a(String)
        rescue RuntimeError => e
          expect(e.message).to match(/html-to-markdown panic during conversion/)
        end
      end

      it 'catches panics in convert_with_options method' do
        malformed_html = "#{'<' * 100_000}div#{'>' * 100_000}"
        handle = described_class.options(heading_style: :atx)

        begin
          result = described_class.convert_with_options(malformed_html, handle)
          expect(result).to be_a(String)
        rescue RuntimeError => e
          expect(e.message).to match(/html-to-markdown panic during conversion/)
        end
      end

      it 'catches panics in convert_with_inline_images method' do
        malformed_html = "#{'<' * 100_000}div#{'>' * 100_000}"

        begin
          result = described_class.convert_with_inline_images(malformed_html)
          expect(result).to be_a(Hash)
          expect(result).to include(:markdown, :inline_images, :warnings)
        rescue RuntimeError => e
          expect(e.message).to match(/html-to-markdown panic during conversion/)
        end
      end
    end
  end
end
