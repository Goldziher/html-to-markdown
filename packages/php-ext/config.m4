dnl config.m4 for html_to_markdown PHP extension built with Rust

PHP_ARG_ENABLE(html_to_markdown, whether to enable html_to_markdown support,
[  --enable-html_to_markdown    Enable html_to_markdown support], yes)

if test "$PHP_HTML_TO_MARKDOWN" != "no"; then
  AC_PATH_PROG([HTMLTOMARKDOWN_CARGO_BIN], [cargo], [no])

  if test "$HTMLTOMARKDOWN_CARGO_BIN" = "no"; then
    AC_MSG_ERROR([cargo binary not found; install Rust (https://rustup.rs) or provide --with-cargo-bin=<path>])
  fi

  PHP_ARG_WITH(cargo-bin, path to cargo binary used to build html_to_markdown,
  [  --with-cargo-bin[=PATH]    Provide an explicit cargo binary], no, no)

  if test "$PHP_CARGO_BIN" != "no"; then
    HTMLTOMARKDOWN_CARGO_BIN=$PHP_CARGO_BIN
  fi

  if test "x$abs_srcdir" = "x"; then
    HTMLTOMARKDOWN_ABS_SRCDIR=`pwd`
  else
    HTMLTOMARKDOWN_ABS_SRCDIR=$abs_srcdir
  fi

  if test -d "$HTMLTOMARKDOWN_ABS_SRCDIR/workspace"; then
    HTMLTOMARKDOWN_WORKSPACE_ROOT=`cd "$HTMLTOMARKDOWN_ABS_SRCDIR/workspace" && pwd`
  elif test -f "$HTMLTOMARKDOWN_ABS_SRCDIR/../../Cargo.toml"; then
    HTMLTOMARKDOWN_WORKSPACE_ROOT=`cd "$HTMLTOMARKDOWN_ABS_SRCDIR/../.." && pwd`
  elif test -f "$HTMLTOMARKDOWN_ABS_SRCDIR/../Cargo.toml"; then
    HTMLTOMARKDOWN_WORKSPACE_ROOT=`cd "$HTMLTOMARKDOWN_ABS_SRCDIR/.." && pwd`
  else
    HTMLTOMARKDOWN_WORKSPACE_ROOT=`cd "$HTMLTOMARKDOWN_ABS_SRCDIR" && pwd`
  fi
  AC_MSG_NOTICE([html_to_markdown workspace root: $HTMLTOMARKDOWN_WORKSPACE_ROOT])
  if test ! -f "$HTMLTOMARKDOWN_WORKSPACE_ROOT/Cargo.toml"; then
    AC_MSG_ERROR([Rust workspace snapshot missing (expected $HTMLTOMARKDOWN_WORKSPACE_ROOT/Cargo.toml)])
  fi

  AC_SUBST([HTMLTOMARKDOWN_CARGO_BIN])
  AC_SUBST([HTMLTOMARKDOWN_PACKAGE], [html-to-markdown-php])
  AC_SUBST([HTMLTOMARKDOWN_ARTIFACT_NAME], [libhtml_to_markdown_php])
  AC_SUBST([HTMLTOMARKDOWN_WORKSPACE_ROOT])

  PHP_SUBST(HTMLTOMARKDOWN_CARGO_BIN)
  PHP_SUBST(HTMLTOMARKDOWN_PACKAGE)
  PHP_SUBST(HTMLTOMARKDOWN_ARTIFACT_NAME)
  PHP_SUBST(HTMLTOMARKDOWN_WORKSPACE_ROOT)

  PHP_ADD_MAKEFILE_FRAGMENT([Makefile.frag])
fi
