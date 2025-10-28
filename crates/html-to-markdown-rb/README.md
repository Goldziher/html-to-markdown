# html-to-markdown-rb

Ruby bindings for the `html-to-markdown` Rust library, powered by [Magnus](https://github.com/matsadler/magnus).

This crate pairs with the Ruby gem exposed from the `lib/` directory and provides high-level conversion helpers for transforming HTML into Markdown from Ruby.

## Requirements

- Ruby 3.2 or newer (Magnus requires fiber APIs not available on Ruby 3.0/3.1)
- A working Rust toolchain (1.85+) and the Ruby development headers
- Windows builds require an MSYS2 environment with the Ruby headers (including `strings.h`). The CI
  currently targets macOS and Linux; add MSYS2 setup before enabling Windows automation.
