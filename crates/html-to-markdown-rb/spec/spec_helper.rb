# frozen_string_literal: true

require 'bundler/setup'
require 'html_to_markdown'

RSpec.configure do |config|
  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
end
