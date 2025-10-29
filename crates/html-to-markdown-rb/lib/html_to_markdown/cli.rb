# frozen_string_literal: true

require 'html_to_markdown/cli_proxy'

module HtmlToMarkdown
  module CLI
    module_function

    def run(argv = ARGV, stdout: $stdout, stderr: $stderr)
      output = CLIProxy.call(argv)
      stdout.print(output)
      0
    rescue CLIProxy::CLIExecutionError => e
      stderr.print(e.stderr)
      e.status || 1
    rescue CLIProxy::MissingBinaryError, CLIProxy::Error => e
      stderr.puts(e.message)
      1
    end
  end
end
