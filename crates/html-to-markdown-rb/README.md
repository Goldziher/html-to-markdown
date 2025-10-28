# html-to-markdown-rb

Ruby bindings for the `html-to-markdown` Rust library, powered by [Magnus](https://github.com/matsadler/magnus).

This crate pairs with the Ruby gem exposed from the `lib/` directory and provides high-level conversion helpers for transforming HTML into Markdown from Ruby.

## Requirements

- Ruby 3.2 or newer (Magnus requires fiber APIs not available on Ruby 3.0/3.1)
- A working Rust toolchain (1.85+) and the Ruby development headers
- On Windows, install [RubyInstaller with MSYS2](https://rubyinstaller.org/) and ensure the
  `UCRT64` toolchain is available. Run `ridk exec pacman -S --needed --noconfirm base-devel
  mingw-w64-ucrt-x86_64-toolchain` once to install the headers (including `strings.h`) required to
  compile the extension.
