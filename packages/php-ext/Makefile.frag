HTM2MD_BUILD_DIR        ?= $(top_builddir)/modules
HTM2MD_TARGET_DIR       ?= $(HTM2MD_WORKSPACE_ROOT)/target
HTM2MD_RELEASE_DIR      ?= $(HTM2MD_TARGET_DIR)/release
HTM2MD_ARTIFACT_EXT     ?= $(SHLIB_SUFFIX_NAME)
HTM2MD_ARTIFACT         ?= $(HTM2MD_RELEASE_DIR)/$(HTM2MD_ARTIFACT_NAME).$(HTM2MD_ARTIFACT_EXT)
HTM2MD_OUTPUT_SO        ?= $(HTM2MD_BUILD_DIR)/html_to_markdown.so
MKDIR_P                 ?= $(mkinstalldirs)

PHP_MODULES += $(HTM2MD_OUTPUT_SO)
all_targets += $(HTM2MD_OUTPUT_SO)

.PHONY: html_to_markdown-build

all: $(HTM2MD_OUTPUT_SO)

$(HTM2MD_OUTPUT_SO): html_to_markdown-build
	@$(MKDIR_P) "$(dir $@)"
	@if test "$(HTM2MD_ARTIFACT_EXT)" = "dylib"; then \
		cp "$(HTM2MD_ARTIFACT)" "$@.tmp"; \
		mv "$@.tmp" "$@"; \
	else \
		cp "$(HTM2MD_ARTIFACT)" "$@"; \
	fi

html_to_markdown-build:
	@"$(HTM2MD_CARGO_BIN)" build --manifest-path="$(HTM2MD_WORKSPACE_ROOT)/Cargo.toml" --package "$(HTM2MD_PACKAGE)" --release
	@if test ! -f "$(HTM2MD_ARTIFACT)"; then \
		echo "cargo did not produce expected artifact: $(HTM2MD_ARTIFACT)"; \
		exit 1; \
	fi

install-modules: $(HTM2MD_OUTPUT_SO)
	$(INSTALL) -m 0755 $(HTM2MD_OUTPUT_SO) $(INSTALL_ROOT)$(PHP_EXTENSION_DIR)/html_to_markdown.so

clean:
	-@"$(HTM2MD_CARGO_BIN)" clean --manifest-path="$(HTM2MD_WORKSPACE_ROOT)/Cargo.toml"
	-rm -f $(HTM2MD_OUTPUT_SO)
