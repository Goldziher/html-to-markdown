# frozen_string_literal: true

require 'spec_helper'
require 'html_to_markdown/cli_proxy'
require 'html_to_markdown/cli'
require 'stringio'

RSpec.describe HtmlToMarkdown::CLIProxy do
  describe '.call' do
    it 'executes the CLI binary' do
      begin
        binary = described_class.find_cli_binary
      rescue HtmlToMarkdown::CLIProxy::MissingBinaryError
        skip 'CLI binary not built'
      end

      expect(binary).to be_file

      output = described_class.call(['--version'])
      expect(output).to include(HtmlToMarkdown::VERSION)
    end
  end

  describe HtmlToMarkdown::CLI do
    it 'writes CLI output to stdout' do
      begin
        HtmlToMarkdown::CLIProxy.find_cli_binary
      rescue HtmlToMarkdown::CLIProxy::MissingBinaryError
        skip 'CLI binary not built'
      end

      stdout = StringIO.new
      stderr = StringIO.new

      exit_code = described_class.run(['--version'], stdout: stdout, stderr: stderr)

      expect(exit_code).to eq(0)
      expect(stdout.string).to include(HtmlToMarkdown::VERSION)
      expect(stderr.string).to be_empty
    end
  end
end
