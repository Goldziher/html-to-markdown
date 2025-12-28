require 'html_to_markdown'
require 'json'

def load_fixtures(filename)
  fixture_path = File.join(__dir__, '../fixtures', filename)
  JSON.parse(File.read(fixture_path))
end

RSpec.describe 'comprehensive html-to-markdown tests' do
  basic_fixtures = load_fixtures('basic-html.json')

  basic_fixtures.each do |test_case|
    it test_case['name'] do
      result = HtmlToMarkdown.convert(test_case['html'], test_case['options'] || {})
      expect(result.strip).to eq(test_case['expectedMarkdown'].strip)
    end
  end
end
