HTMLTOMARKDOWN_BUILD_DIR        ?= $(top_builddir)/modules
HTMLTOMARKDOWN_TARGET_DIR       ?= $(HTMLTOMARKDOWN_WORKSPACE_ROOT)/target
HTMLTOMARKDOWN_RELEASE_DIR      ?= $(HTMLTOMARKDOWN_TARGET_DIR)/release
HTMLTOMARKDOWN_ARTIFACT_EXT     ?= $(SHLIB_SUFFIX_NAME)
HTMLTOMARKDOWN_ARTIFACT         ?= $(HTMLTOMARKDOWN_RELEASE_DIR)/$(HTMLTOMARKDOWN_ARTIFACT_NAME).$(HTMLTOMARKDOWN_ARTIFACT_EXT)
HTMLTOMARKDOWN_OUTPUT_SO        ?= $(HTMLTOMARKDOWN_BUILD_DIR)/html_to_markdown.so
MKDIR_P                 ?= $(mkinstalldirs)
PHP_EXTENSION_DIR       ?= $(EXTENSION_DIR)

PHP_MODULES += $(HTMLTOMARKDOWN_OUTPUT_SO)
all_targets += $(HTMLTOMARKDOWN_OUTPUT_SO)

.PHONY: html_to_markdown-build

all: $(HTMLTOMARKDOWN_OUTPUT_SO)

$(HTMLTOMARKDOWN_OUTPUT_SO): html_to_markdown-build
	@$(MKDIR_P) "$(dir $@)"
	@if test "$(HTMLTOMARKDOWN_ARTIFACT_EXT)" = "dylib"; then \
		cp "$(HTMLTOMARKDOWN_ARTIFACT)" "$@.tmp"; \
		mv "$@.tmp" "$@"; \
	else \
		cp "$(HTMLTOMARKDOWN_ARTIFACT)" "$@"; \
	fi

html_to_markdown-build:
	@"$(HTMLTOMARKDOWN_CARGO_BIN)" build --manifest-path="$(HTMLTOMARKDOWN_WORKSPACE_ROOT)/Cargo.toml" --package "$(HTMLTOMARKDOWN_PACKAGE)" --release
	@if test ! -f "$(HTMLTOMARKDOWN_ARTIFACT)"; then \
		echo "cargo did not produce expected artifact: $(HTMLTOMARKDOWN_ARTIFACT)"; \
		exit 1; \
	fi

install-modules: $(HTMLTOMARKDOWN_OUTPUT_SO)
	$(INSTALL) -m 0755 $(HTMLTOMARKDOWN_OUTPUT_SO) $(INSTALL_ROOT)$(PHP_EXTENSION_DIR)/html_to_markdown.so

clean:
	-@"$(HTMLTOMARKDOWN_CARGO_BIN)" clean --manifest-path="$(HTMLTOMARKDOWN_WORKSPACE_ROOT)/Cargo.toml"
	-rm -f $(HTMLTOMARKDOWN_OUTPUT_SO)
