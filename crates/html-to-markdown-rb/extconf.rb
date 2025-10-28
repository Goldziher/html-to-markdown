# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'

default_profile = ENV.fetch('CARGO_PROFILE', 'release')

create_rust_makefile('html_to_markdown_rb') do |config|
  config.profile = default_profile.to_sym
end
