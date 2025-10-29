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
end
