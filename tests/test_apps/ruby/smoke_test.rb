require 'html_to_markdown'

RSpec.describe 'html-to-markdown smoke tests' do
  it 'can load the gem' do
    expect(defined?(HtmlToMarkdown)).to be_truthy
  end

  it 'converts basic HTML' do
    html = '<p>Hello World</p>'
    result = HtmlToMarkdown.convert(html)
    expect(result).to include('Hello World')
  end

  it 'handles options' do
    html = '<h1>Title</h1>'
    result = HtmlToMarkdown.convert(html)
    expect(result).to start_with('#')
  end

  it 'handles empty input' do
    result = HtmlToMarkdown.convert('')
    expect(result).to eq('')
  end
end
