# frozen_string_literal: true

require 'open3'
require 'pathname'

module HtmlToMarkdown
  module CLIProxy
    Error = Class.new(StandardError)
    MissingBinaryError = Class.new(Error)

    class CLIExecutionError < Error
      attr_reader :stderr, :status

      def initialize(message, stderr:, status:)
        super(message)
        @stderr = stderr
        @status = status
      end
    end

    module_function

    def call(argv)
      binary = find_cli_binary
      args = Array(argv).map(&:to_s)
      stdout, stderr, status = Open3.capture3(binary.to_s, *args)
      return stdout if status.success?

      raise CLIExecutionError.new(
        "html-to-markdown CLI exited with status #{status.exitstatus}",
        stderr: stderr,
        status: status.exitstatus
      )
    end

    def find_cli_binary
      binary_name = Gem.win_platform? ? 'html-to-markdown.exe' : 'html-to-markdown'
      found = search_paths(binary_name).find(&:file?)
      return found if found

      raise MissingBinaryError, missing_binary_message
    end

    def root_path
      @root_path ||= Pathname(__dir__).join('../..').expand_path
    end

    def lib_path
      @lib_path ||= Pathname(__dir__).join('..').expand_path
    end

    def search_paths(binary_name)
      paths = [
        root_path.join('target', 'release', binary_name),
        lib_path.join('bin', binary_name),
        lib_path.join(binary_name)
      ]

      workspace_root = root_path.parent&.parent
      paths << workspace_root.join('target', 'release', binary_name) if workspace_root
      paths
    end

    def missing_binary_message
      <<~MSG.strip
        html-to-markdown CLI binary not found. Build it with
        `cargo build --release --package html-to-markdown-cli`.
      MSG
    end
  end
end
