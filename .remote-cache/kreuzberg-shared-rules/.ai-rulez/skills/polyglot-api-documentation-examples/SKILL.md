______________________________________________________________________

## priority: high

# Polyglot API Documentation Examples

**CRITICAL: Complete language parity for API documentation.** ALL public APIs MUST be documented with examples in all 10 supported languages: Rust, Python, TypeScript, Ruby, PHP, Java, Go, C#, Elixir, and WebAssembly.

## Documentation Tools by Language

| Language | Tool | Output Format | File Extension | Key Strengths |
|----------|------|---------------|-----------------|---------------|
| **Rust** | rustdoc | HTML, integrated with cargo doc | `.rs` | Markdown, examples as doctests, cross-referencing |
| **Python** | Sphinx | HTML, PDF, ePub | `.py` | reStructuredText integration, autodoc directives |
| **TypeScript** | TypeDoc | HTML, Markdown, JSON | `.ts` / `.tsx` | JSDoc parsing, template customization, JSON exports |
| **Ruby** | YARD | HTML, Markdown, tags | `.rb` | @param/@return/@example, @overload support |
| **PHP** | PHPDocumentor | HTML, PDF | `.php` | @param/@return/@throws, markdown descriptions |
| **Java** | Javadoc | HTML | `.java` | @param/@return/@throws/@since/@deprecated tags |
| **Go** | godoc | HTML, plaintext | `.go` | doc.go files, package-level docs, examples in \_test.go |
| **C#** | DocFX | HTML, Markdown | `.cs` | XML doc comments, xref links, TOC generation |
| **Elixir** | ExDoc | HTML, EPUB | `.ex` | Markdown, @doc/@spec, @deprecated, live examples |
| **WebAssembly** | wasm-doc | Markdown, HTML | `.wasm` / `.js` | JSDoc for JS wrapper, inline comments |

## API Documentation Structure

Every public API must include:

1. **Summary**: One-line description of what the function/method does
1. **Description**: Detailed explanation of behavior, edge cases, preconditions
1. **Parameters**: Type, description, constraints, default values
1. **Return Value**: Type, description, possible error states
1. **Examples**: Minimal working code in ALL 10 languages
1. **Related APIs**: Cross-references to related functions
1. **Error Cases**: Common errors and how to handle them
1. **Performance Notes**: Complexity, memory usage, when applicable
1. **Deprecation Status**: If applicable, mention alternatives

## Language-Specific Documentation Patterns

### Rust (rustdoc)

````rust
/// Processes HTML and converts it to Markdown.
///
/// Takes HTML input and converts it to semantically equivalent Markdown,
/// preserving all text content, links, lists, and formatting.
///
/// # Arguments
/// * `html` - Valid HTML string (can be partial HTML, e.g., fragments without DOCTYPE)
/// * `options` - Configuration options for the conversion
///
/// # Returns
/// `Result<String, Error>` containing the Markdown output or an error
///
/// # Errors
/// * `Error::InvalidUtf8` - If HTML contains invalid UTF-8 sequences
/// * `Error::ProcessingFailed` - If internal processing fails
///
/// # Examples
///
/// ```
/// use kreuzberg::HtmlConverter;
///
/// let html = "<h1>Hello</h1><p>World</p>";
/// let md = HtmlConverter::convert(html)?;
/// assert!(md.contains("# Hello"));
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// # Performance
/// - Time: O(n) where n is HTML input size
/// - Memory: O(n) for output string
pub fn convert(html: &str, options: &Options) -> Result<String, Error> {
    // implementation
}
````

**Documentation tools**: cargo doc, rustdoc with mdBook for guides

______________________________________________________________________

### Python (Sphinx + Google Style)

```python
def convert_html_to_markdown(
    html: str,
    options: Optional[ConversionOptions] = None
) -> str:
    """Convert HTML to Markdown with semantic preservation.

    Processes HTML input and produces semantically equivalent Markdown output.
    Supports partial HTML fragments (no DOCTYPE required).

    Args:
        html: Valid HTML string to convert. Can be a complete document or fragment.
        options: Configuration for the conversion process. Defaults to standard settings.

    Returns:
        Markdown representation of the input HTML.

    Raises:
        ValueError: If HTML is invalid or contains unsupported encodings.
        ProcessingError: If internal conversion fails.

    Examples:
        Basic conversion:

        >>> from kreuzberg import convert_html_to_markdown
        >>> html = "<h1>Hello</h1><p>World</p>"
        >>> md = convert_html_to_markdown(html)
        >>> "# Hello" in md
        True

        With custom options:

        >>> opts = ConversionOptions(preserve_attributes=True)
        >>> result = convert_html_to_markdown(html, opts)

    Note:
        This function is CPU-bound. For processing large documents,
        consider using ``convert_html_to_markdown_async()`` in an event loop.

    Performance:
        - Time complexity: O(n) where n is HTML input size
        - Memory usage: O(n) for output string
    """
    # implementation
```

**Documentation tools**: Sphinx, autodoc, napoleon extension for Google style

______________________________________________________________________

### TypeScript (JSDoc + TypeDoc)

````typescript
/**
 * Converts HTML to Markdown with complete semantic preservation.
 *
 * Takes HTML input (full documents or fragments) and produces semantically
 * equivalent Markdown output. Handles all standard HTML elements and preserves
 * formatting, links, and structure.
 *
 * @param html - The HTML string to convert. Can be a complete document or fragment.
 * @param options - Configuration options for the conversion process.
 * @returns Promise<string> - The converted Markdown text.
 * @throws {InvalidHtmlError} If the HTML is malformed or unsupported.
 * @throws {ProcessingError} If conversion fails internally.
 *
 * @example
 * Basic usage:
 * ```typescript
 * import { convertHtmlToMarkdown } from 'kreuzberg';
 *
 * const html = '<h1>Hello</h1><p>World</p>';
 * const markdown = await convertHtmlToMarkdown(html);
 * console.log(markdown); // # Hello\n\nWorld
 * ```
 *
 * @example
 * With options:
 * ```typescript
 * const options = { preserveAttributes: true };
 * const result = await convertHtmlToMarkdown(html, options);
 * ```
 *
 * @remarks
 * - Async operation for better performance with large documents
 * - Fully typed with TypeScript strict mode compatibility
 * - Uses native browser APIs when available
 *
 * @see {@link ConversionOptions} for available configuration
 * @see {@link convertHtmlToMarkdownSync} for synchronous version
 * @beta This API may change in future versions
 */
export async function convertHtmlToMarkdown(
  html: string,
  options?: ConversionOptions
): Promise<string> {
  // implementation
}
````

**Documentation tools**: TypeDoc, JSDoc, TypeScript compiler, markdown output

______________________________________________________________________

### Ruby (YARD)

```ruby
# Converts HTML to Markdown with complete semantic preservation.
#
# Processes HTML input (complete documents or fragments) and generates
# semantically equivalent Markdown output. All formatting, links, and
# document structure are preserved in the conversion.
#
# @param html [String] The HTML string to convert. Can be a complete
#   document or fragment (no DOCTYPE required).
# @param options [ConversionOptions, Hash] Configuration for the conversion.
#   Defaults to standard conversion settings.
#
# @return [String] The converted Markdown text.
#
# @raise [InvalidHtmlError] If the HTML is malformed or contains
#   unsupported syntax.
# @raise [ProcessingError] If the conversion process fails internally.
#
# @example Basic conversion
#   require 'kreuzberg'
#
#   html = '<h1>Hello</h1><p>World</p>'
#   markdown = Kreuzberg.convert_html_to_markdown(html)
#   markdown.include?('# Hello') #=> true
#
# @example With custom options
#   options = { preserve_attributes: true }
#   result = Kreuzberg.convert_html_to_markdown(html, options)
#
# @see ConversionOptions
# @see #convert_html_to_markdown_with_defaults
# @since 1.0.0
# @deprecated Use {#convert_with_streaming} for large documents
def self.convert_html_to_markdown(html, options = {})
  # implementation
end
```

**Documentation tools**: YARD, yard-doc gem, markdown support with kramdown

______________________________________________________________________

### PHP (PHPDocumentor)

````php
/**
 * Converts HTML to Markdown with complete semantic preservation.
 *
 * Takes HTML input (full documents or fragments) and produces semantically
 * equivalent Markdown output. All formatting, links, and document structure
 * are preserved.
 *
 * @param string $html The HTML string to convert. Can be a complete
 *                     document or fragment (no DOCTYPE required).
 * @param ConversionOptions|null $options Configuration for the conversion
 *                                        process. Null uses default settings.
 *
 * @return string The converted Markdown text.
 *
 * @throws InvalidHtmlException If HTML is malformed or unsupported
 * @throws ProcessingException If the conversion fails internally
 *
 * @example Basic usage:
 *   ```php
 *   $html = '<h1>Hello</h1><p>World</p>';
 *   $markdown = Kreuzberg::convertHtmlToMarkdown($html);
 *   echo $markdown; // # Hello\n\nWorld
 *   ```
 *
 * @example With options:
 *   ```php
 *   $options = new ConversionOptions();
 *   $options->preserveAttributes = true;
 *   $result = Kreuzberg::convertHtmlToMarkdown($html, $options);
 *   ```
 *
 * @see ConversionOptions
 * @see \Kreuzberg\Converter for streaming API
 * @since 1.0.0
 * @api
 */
public static function convertHtmlToMarkdown(
    string $html,
    ?ConversionOptions $options = null
): string {
    // implementation
}
````

**Documentation tools**: PHPDocumentor, phpdoc tags, markdown in descriptions

______________________________________________________________________

### Java (Javadoc)

```java
/**
 * Converts HTML to Markdown with complete semantic preservation.
 *
 * <p>Processes HTML input (complete documents or fragments) and generates
 * semantically equivalent Markdown output. All formatting, links, and
 * document structure are preserved.</p>
 *
 * @param html the HTML string to convert; can be a complete document
 *             or fragment (no DOCTYPE required). Must not be null.
 * @param options configuration for the conversion process; if null,
 *                uses default conversion settings
 *
 * @return the converted Markdown text; never null
 *
 * @throws IllegalArgumentException if html is null
 * @throws InvalidHtmlException if the HTML is malformed or contains
 *                             unsupported syntax
 * @throws ProcessingException if the conversion fails internally
 *
 * @example Basic usage:
 *   <pre>{@code
 *   String html = "<h1>Hello</h1><p>World</p>";
 *   String markdown = HtmlConverter.convertToMarkdown(html);
 *   System.out.println(markdown); // # Hello\n\nWorld
 *   }</pre>
 *
 * @example With custom options:
 *   <pre>{@code
 *   ConversionOptions options = new ConversionOptions()
 *       .preserveAttributes(true);
 *   String result = HtmlConverter.convertToMarkdown(html, options);
 *   }</pre>
 *
 * @apiNote This method is thread-safe for concurrent calls.
 * @since 1.0
 * @see ConversionOptions
 * @see #convertToMarkdownAsync(String, ConversionOptions)
 */
public static String convertToMarkdown(
    String html,
    ConversionOptions options
) throws InvalidHtmlException, ProcessingException {
    // implementation
}
```

**Documentation tools**: Javadoc, Maven site plugin, HTML/Markdown output

______________________________________________________________________

### Go (godoc)

```go
// convertHTMLToMarkdown converts HTML to Markdown with complete semantic preservation.
//
// Takes HTML input (full documents or fragments) and produces semantically
// equivalent Markdown output. All formatting, links, and document structure
// are preserved in the conversion process.
//
// Parameters:
//   - html: The HTML string to convert. Can be a complete document or fragment
//     (no DOCTYPE required).
//   - options: Configuration for the conversion process. Nil uses defaults.
//
// Returns:
//   - The converted Markdown string
//   - An error if conversion fails
//
// Errors:
//   - InvalidHTML: If the HTML is malformed or contains unsupported syntax
//   - ProcessingError: If the internal conversion process fails
//
// Example:
//
//	html := "<h1>Hello</h1><p>World</p>"
//	md, err := kreuzberg.ConvertHTMLToMarkdown(html, nil)
//	if err != nil {
//		log.Fatal(err)
//	}
//	fmt.Println(md) // # Hello\n\nWorld
//
// Example with options:
//
//	opts := &kreuzberg.ConversionOptions{
//		PreserveAttributes: true,
//	}
//	result, err := kreuzberg.ConvertHTMLToMarkdown(html, opts)
//
// See Also:
//   - ConversionOptions for configuration details
//   - ConvertHTMLToMarkdownAsync for concurrent operations
//
// Concurrency:
//
// This function is safe for concurrent use from multiple goroutines.
func ConvertHTMLToMarkdown(html string, options *ConversionOptions) (string, error) {
	// implementation
}
```

**Documentation tools**: godoc, go doc, markdown in comments

______________________________________________________________________

### C# (XML Documentation)

```csharp
/// <summary>
/// Converts HTML to Markdown with complete semantic preservation.
/// </summary>
///
/// <remarks>
/// <para>
/// Processes HTML input (complete documents or fragments) and generates
/// semantically equivalent Markdown output. All formatting, links, and
/// document structure are preserved in the conversion.
/// </para>
/// <para>
/// This method is thread-safe and can be called concurrently from multiple
/// threads.
/// </para>
/// </remarks>
///
/// <param name="html">
/// The HTML string to convert. Can be a complete document or fragment
/// (no DOCTYPE required). Must not be null.
/// </param>
/// <param name="options">
/// Configuration for the conversion process. If null, uses default settings.
/// </param>
///
/// <returns>
/// The converted Markdown text as a string. Never null.
/// </returns>
///
/// <exception cref="ArgumentNullException">
/// Thrown when <paramref name="html"/> is null.
/// </exception>
/// <exception cref="InvalidHtmlException">
/// Thrown when the HTML is malformed or contains unsupported syntax.
/// </exception>
/// <exception cref="ProcessingException">
/// Thrown when the conversion process fails internally.
/// </exception>
///
/// <example>
/// <para>Basic usage:</para>
/// <code>
/// string html = "<h1>Hello</h1><p>World</p>";
/// string markdown = HtmlConverter.ConvertToMarkdown(html);
/// Console.WriteLine(markdown); // # Hello\n\nWorld
/// </code>
/// </example>
///
/// <example>
/// <para>With custom options:</para>
/// <code>
/// var options = new ConversionOptions { PreserveAttributes = true };
/// string result = HtmlConverter.ConvertToMarkdown(html, options);
/// </code>
/// </example>
///
/// <seealso cref="ConversionOptions"/>
/// <seealso cref="ConvertToMarkdownAsync(string, ConversionOptions)"/>
/// <since>1.0</since>
public static string ConvertToMarkdown(
    string html,
    ConversionOptions? options = null
) {
    // implementation
}
```

**Documentation tools**: DocFX, Sandcastle Help File Builder, XML to Markdown conversion

______________________________________________________________________

### Elixir (ExDoc)

```elixir
@doc """
Converts HTML to Markdown with complete semantic preservation.

Takes HTML input (full documents or fragments) and produces semantically
equivalent Markdown output. All formatting, links, and document structure
are preserved.

## Parameters

  * `html` - The HTML string to convert. Can be a complete document or
    fragment (no DOCTYPE required).
  * `options` - Configuration for the conversion process. Defaults to
    standard settings.

## Return value

The converted Markdown text as a binary string.

## Errors

  * `{:error, :invalid_html}` - If the HTML is malformed or unsupported
  * `{:error, :processing_failed}` - If conversion fails internally

## Examples

Basic conversion:

    iex> html = "<h1>Hello</h1><p>World</p>"
    iex> {:ok, md} = Kreuzberg.convert_html_to_markdown(html)
    iex> String.contains?(md, "# Hello")
    true

With custom options:

    iex> opts = [preserve_attributes: true]
    iex> {:ok, result} = Kreuzberg.convert_html_to_markdown(html, opts)

## Performance

  * Time complexity: O(n) where n is HTML input size
  * Memory usage: O(n) for output string

## See also

  * `Kreuzberg.ConversionOptions` for configuration details
  * `Kreuzberg.convert_html_to_markdown_async/2` for concurrent operations

## Since

1.0.0
"""
@spec convert_html_to_markdown(String.t(), keyword()) :: {:ok, String.t()} | {:error, atom()}
def convert_html_to_markdown(html, options \\ []) do
  # implementation
end
```

**Documentation tools**: ExDoc, markdown with code blocks, @spec annotations

______________________________________________________________________

### WebAssembly (JSDoc wrapper)

```javascript
/**
 * Converts HTML to Markdown with complete semantic preservation.
 *
 * Takes HTML input (full documents or fragments) and produces semantically
 * equivalent Markdown output. All formatting, links, and document structure
 * are preserved.
 *
 * @async
 * @param {string} html - The HTML string to convert. Can be a complete
 *                        document or fragment (no DOCTYPE required).
 * @param {Object} [options] - Configuration for the conversion process
 * @param {boolean} [options.preserveAttributes=false] - Keep HTML attributes
 * @param {boolean} [options.stripComments=true] - Remove HTML comments
 *
 * @returns {Promise<string>} The converted Markdown text
 *
 * @throws {InvalidHtmlError} If the HTML is malformed or unsupported
 * @throws {ProcessingError} If conversion fails internally
 *
 * @example
 * // Basic conversion
 * const html = '<h1>Hello</h1><p>World</p>';
 * const markdown = await convertHtmlToMarkdown(html);
 * console.log(markdown); // # Hello\n\nWorld
 *
 * @example
 * // With options
 * const options = { preserveAttributes: true };
 * const result = await convertHtmlToMarkdown(html, options);
 *
 * @remarks
 * - Requires WASM module to be initialized
 * - Works in both Node.js and browser environments
 * - Uses streaming internally for large documents
 *
 * @see {@link ConversionOptions} for all available configuration
 * @see {@link convertHtmlToMarkdownSync} for synchronous version
 * @deprecated Use convertHtmlToMarkdownV2 in future versions
 */
async function convertHtmlToMarkdown(html, options = {}) {
  // implementation
}
```

**Documentation tools**: JSDoc, TypeDoc for TypeScript wrapper, markdown generation

______________________________________________________________________

## API Documentation Parity Checklist

When documenting a new API, ensure ALL language bindings include:

- [ ] **Rust**: rustdoc with examples, SAFETY comments if unsafe
- [ ] **Python**: Google-style docstrings with Sphinx-compatible formatting
- [ ] **TypeScript**: JSDoc with all @param/@returns/@example/@throws tags
- [ ] **Ruby**: YARD documentation with @param/@return/@example/@raise tags
- [ ] **PHP**: PHPDocumentor comments with @param/@return/@throws tags
- [ ] **Java**: Javadoc with @param/@return/@throws/@since/@see tags
- [ ] **Go**: godoc comments in doc.go or function-level comments
- [ ] **C#**: XML documentation with <summary> and <param> elements
- [ ] **Elixir**: ExDoc documentation with @spec and @doc annotations
- [ ] **WebAssembly**: JSDoc comments on JavaScript wrapper functions

## Migration Guide: Adding Documentation to Existing APIs

### Step 1: Document Rust Core First

Start with the Rust implementation since all bindings depend on it:

````rust
/// New public API function.
///
/// # Examples
///
/// ```
/// // doctest here
/// ```
pub fn new_function(param: Type) -> Result<Output, Error> {
    // implementation
}
````

### Step 2: Update All Language Bindings

For each language binding, add equivalent documentation:

1. **Python**: Add docstring following Google style
1. **TypeScript**: Add JSDoc with @param/@returns/@example
1. **Ruby**: Add YARD documentation
1. **PHP**: Add PHPDocumentor comments
1. **Java**: Add Javadoc comments
1. **Go**: Add godoc comments
1. **C#**: Add XML documentation
1. **Elixir**: Add @doc and @spec annotations
1. **WASM**: Add JSDoc to wrapper function

### Step 3: Add Examples in docs/snippets/

Create language-specific examples:

```
docs/snippets/
├── rust/api/new_function.rs
├── python/api/new_function.py
├── typescript/api/new_function.ts
├── ruby/api/new_function.rb
├── php/api/new_function.php
├── java/api/NewFunction.java
├── go/api/new_function.go
├── csharp/api/NewFunction.cs
├── elixir/api/new_function.exs
└── wasm/api/new_function.js
```

### Step 4: Build and Verify Documentation

For each language, generate and verify documentation builds:

```bash
task doc:rust      # cargo doc
task doc:python    # sphinx-build
task doc:typescript # typedoc
task doc:ruby      # yard
task doc:php       # phpdoc
task doc:java      # javadoc
task doc:go        # go doc
task doc:csharp    # docfx
task doc:elixir    # mix docs
task doc:wasm      # jsdoc
```

## Error Handling Documentation Pattern

ALL error documentation must be language-specific:

### Rust

```rust
/// # Errors
/// Returns `Error::InvalidInput` if preconditions not met.
```

### Python

```python
"""
Raises:
    ValueError: If input is invalid.
"""
```

### TypeScript

```typescript
/**
 * @throws {InvalidInputError} If input is invalid.
 */
```

### Ruby

```ruby
# @raise [InvalidInputError] if input is invalid
```

### PHP

```php
/**
 * @throws InvalidInputException If input is invalid
 */
```

### Java

```java
/**
 * @throws IllegalArgumentException if input is invalid
 */
```

### Go

```go
// Returns error if input is invalid.
```

### C\#

```csharp
/// <exception cref="ArgumentException">Thrown if input is invalid.</exception>
```

### Elixir

```elixir
"""
Raises:

  * `ArgumentError` - if input is invalid
"""
```

## Cross-Language API Reference Generation

For auto-generated API documentation, maintain a mapping file:

```yaml
# docs/api-mapping.yaml
functions:
  - name: convert_html_to_markdown
    rust: convert(html: &str, options: &Options) -> Result<String, Error>
    python: convert_html_to_markdown(html: str, options: Optional[ConversionOptions]) -> str
    typescript: convertHtmlToMarkdown(html: string, options?: ConversionOptions): Promise<string>
    ruby: Kreuzberg.convert_html_to_markdown(html, options = {})
    php: Kreuzberg::convertHtmlToMarkdown(string $html, ?ConversionOptions $options): string
    java: HtmlConverter.convertToMarkdown(String html, ConversionOptions options)
    go: ConvertHTMLToMarkdown(html string, options *ConversionOptions) (string, error)
    csharp: HtmlConverter.ConvertToMarkdown(string html, ConversionOptions? options)
    elixir: Kreuzberg.convert_html_to_markdown(html, options \\ [])
    wasm: convertHtmlToMarkdown(html: string, options?: object): Promise<string>
    notes: Converts HTML to Markdown with semantic preservation
    since: 1.0.0
    status: stable
    errors: InvalidHtmlError, ProcessingError
```

## Documentation Build Verification

Create verification scripts to ensure documentation consistency:

```bash
#!/bin/bash
# verify-api-docs.sh

echo "Checking API documentation parity..."

# Check all languages have examples
for lang in rust python typescript ruby php java go csharp elixir wasm; do
  if ! grep -r "@example\|@examples\|Examples:" docs/snippets/$lang/api/ 2>/dev/null; then
    echo "❌ Missing examples for $lang"
  fi
done

# Verify documentation builds
cargo doc --no-deps
sphinx-build -b html docs/python _build/python-docs
typedoc --out _build/typescript-docs
yard --output-dir _build/ruby-docs
phpdoc -d src -t _build/php-docs
javadoc -d _build/java-docs src/main/java
go doc ./...
docfx _docfx.json
mix docs

echo "✓ Documentation verification complete"
```

## Best Practices

1. **Keep examples concise**: 5-15 lines max per example
1. **Use realistic scenarios**: Examples should match actual usage patterns
1. **Include error handling**: Show how to handle common errors in each language
1. **Test all code examples**: Run examples as part of CI/CD
1. **Update all languages together**: When API changes, update docs for all 10 languages simultaneously
1. **Cross-reference consistently**: Use language-native cross-reference syntax
1. **Document performance**: Include time/space complexity and practical benchmarks
1. **Version your docs**: Track API changes and deprecations in documentation
1. **Automate generation**: Use doc generators to reduce manual maintenance burden
1. **Maintain consistency**: Follow the patterns in this guide across all public APIs
