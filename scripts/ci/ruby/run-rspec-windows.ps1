$ErrorActionPreference = "Stop"

$unixPath = ridk exec bash -lc "cygpath -au '$env:GITHUB_WORKSPACE/packages/ruby'"
ridk exec bash -lc "cd $unixPath && export RUSTUP_TOOLCHAIN=stable-gnu CC=x86_64-w64-mingw32-gcc CXX=x86_64-w64-mingw32-g++ && bundle exec rspec --format progress"
