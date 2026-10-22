<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:33dd90268a7637ff545d129b1751d8afce361b4d9a4ffac98c0b9a3ae2d83466
Source-Hash: blake3:4237e14c4d547597faf915235dde5b5d71e8c9d84681d23919d03302b738a0b4
Schema-Version: v1
-->

# Other Language Bindings

Brief reference for Go, Ruby, PHP, Java, C#, Elixir, R, WASM, and C FFI.

---

## Go

**Module:** `github.com/xberg-io/html-to-markdown/packages/go/v3`
**Package:** `htmltomarkdown`
**Install:** `go get github.com/xberg-io/html-to-markdown/packages/go/v3`

Uses cgo with the C FFI layer. Options are passed as JSON strings internally.

```go
import htmltomarkdown "github.com/xberg-io/html-to-markdown/packages/go/v3"

// Primary function — func Convert(html string, options *ConversionOptions) (*ConversionResult, error)
// Pass nil for defaults.
result, err := htmltomarkdown.Convert(html, nil)
if err != nil {
    log.Fatal(err)
}
fmt.Println(*result.Content)       // Content is *string
fmt.Println(len(result.Tables))    // extracted tables
fmt.Println(len(result.Warnings))  // processing warnings

// With options
result, err = htmltomarkdown.Convert(html, &htmltomarkdown.ConversionOptions{
    HeadingStyle: "atx",
})

// Metadata is in result.Metadata when metadata extraction is enabled
fmt.Println(result.Metadata)

// Tables — in result.Tables when IncludeDocumentStructure is enabled
for _, table := range result.Tables {
    fmt.Println(table.Markdown)
}
```

### Go ConversionOptions

Options are passed as a pointer to `ConversionOptions`. Fields use Go naming conventions (PascalCase). Pass `nil` for defaults (the argument is required, not variadic).

---

## Ruby

**Gem:** `html-to-markdown`
**Install:** `gem install html-to-markdown`
**Require:** `require 'html_to_markdown'`

Uses Magnus (native extension via Rust). The native module is `HtmlToMarkdownRs`
(the top-level `html_to_markdown` require loads it).

```ruby
require 'html_to_markdown'

# Primary function — returns a HtmlToMarkdownRs::ConversionResult OBJECT (methods, not a Hash)
result = HtmlToMarkdownRs.convert(html)
puts result.content          # markdown string
puts result.tables.length    # extracted tables
puts result.warnings.length  # processing warnings
puts result.metadata         # metadata object (or nil)

# With options — pass a Hash (it is serialized to ConversionOptions) or a ConversionOptions object
result = HtmlToMarkdownRs.convert(html, {
    heading_style: "atx",
    code_block_style: "backticks",
    autolinks: true,
})

# Tables — in result.tables when include_document_structure is enabled; each is a
# TableData with .markdown and .grid
result.tables.each { |t| puts t.markdown }
```

### Ruby convert() return object

`HtmlToMarkdownRs.convert` returns a `HtmlToMarkdownRs::ConversionResult` with
accessor methods: `content`, `document`, `metadata`, `tables`, `warnings`.
There is no reusable options handle and no top-level `images` accessor — only the
`convert` module function is exposed.

---

## PHP

**Composer:** `xberg-io/html-to-markdown`
**Install:** `composer require xberg-io/html-to-markdown`
**PHP requirement:** 8.2+

Uses ext-php-rs (native PHP extension). The facade class is `HtmlToMarkdown` under
the `HtmlToMarkdown` namespace. Conversion failures throw
`HtmlToMarkdown\HtmlToMarkdownException`. Option properties are camelCase.

```php
<?php
declare(strict_types=1);

use HtmlToMarkdown\HtmlToMarkdown;
use HtmlToMarkdown\ConversionOptions;

// Primary function — public static function convert(string $html, ?ConversionOptions $options = null): ConversionResult
$result = HtmlToMarkdown::convert('<h1>Hello</h1>');
$markdown = $result->content;

// With options (ConversionOptions object, camelCase properties)
$options = new ConversionOptions();
$options->headingStyle = 'atx';
$options->codeBlockStyle = 'backticks';
$options->autolinks = true;

$result = HtmlToMarkdown::convert('<h1>Hello</h1>', $options);

// Metadata — in $result->metadata
$metadata = $result->metadata;
echo $metadata->document->title;

// Tables — in $result->tables when includeDocumentStructure is enabled
foreach ($result->tables as $table) {
    echo $table->markdown;
}
```

---

## Java

**Maven:** `io.xberg:html-to-markdown`
**GroupId:** `io.xberg`
**ArtifactId:** `html-to-markdown`
**Java requirement:** 25+ (uses Panama FFM API)

```xml
<dependency>
  <groupId>io.xberg</groupId>
  <artifactId>html-to-markdown</artifactId>
  <version>3.8.0</version>
</dependency>
```

```java
import io.xberg.htmltomarkdown.HtmlToMarkdown;
import io.xberg.htmltomarkdown.ConversionResult;
import io.xberg.htmltomarkdown.HtmlToMarkdownRsException;

// Primary function — returns ConversionResult
try {
    ConversionResult result = HtmlToMarkdown.convert("<h1>Hello</h1>");
    System.out.println(result.content());    // markdown string
    System.out.println(result.tables());     // List<TableData>
    System.out.println(result.warnings());   // List<ProcessingWarning>
    System.out.println(result.metadata());   // metadata map (when enabled)
} catch (HtmlToMarkdownRsException e) {
    System.err.println("Conversion failed: " + e.getMessage());
}

// With options
import io.xberg.htmltomarkdown.ConversionOptions;

ConversionOptions options = new ConversionOptions();
options.setHeadingStyle("atx");
ConversionResult result = HtmlToMarkdown.convert("<h1>Hello</h1>", options);

// Tables — in result.tables() when includeDocumentStructure is enabled
for (var table : result.tables()) {
    System.out.println(table.markdown());
}
```

---

## C# (.NET)

**NuGet:** `XbergIo.HtmlToMarkdown`
**Install:** `dotnet add package XbergIo.HtmlToMarkdown`
**.NET requirement:** 10 (`net10.0`)

Namespace is `HtmlToMarkdown`. The static entry-point class is
`HtmlToMarkdownConverter`. The exception type is `HtmlToMarkdownRsException`.
`ConversionResult` is a record with `Content`, `Document`, `Metadata`, `Tables`,
and `Warnings`.

```csharp
using HtmlToMarkdown;

// Primary function — public static ConversionResult Convert(string html, ConversionOptions? options)
var result = HtmlToMarkdownConverter.Convert("<h1>Hello</h1>", null);
Console.WriteLine(result.Content);         // markdown string
Console.WriteLine(result.Tables.Count);    // table count
Console.WriteLine(result.Warnings.Count);  // warning count
Console.WriteLine(result.Metadata?.Document?.Title);  // metadata (when enabled)

// With options (enum-typed fields use the HeadingStyle/etc. enums, PascalCase)
var options = new ConversionOptions { HeadingStyle = HeadingStyle.Atx };
var result2 = HtmlToMarkdownConverter.Convert(html, options);

// Tables — in result.Tables when IncludeDocumentStructure is enabled
foreach (var table in result.Tables) {
    Console.WriteLine(table.Markdown);
}
```

### HtmlToMarkdownRsException

```csharp
try {
    var result = HtmlToMarkdownConverter.Convert(html, null);
} catch (HtmlToMarkdownRsException e) {
    Console.Error.WriteLine($"Conversion failed: {e.Message}");
}
```

---

## Elixir

**Hex:** `html_to_markdown`
**Module:** `HtmlToMarkdown`
**Elixir requirement:** 1.14+ (uses Rustler NIFs)

```elixir
# mix.exs
{:html_to_markdown, "~> 3.8"}
```

```elixir
# Primary function — convert(html, options \\ nil)
# returns {:ok, map()} | {:error, atom, String.t()}
{:ok, result} = HtmlToMarkdown.convert("<h1>Hello</h1>")
IO.puts result.content      # markdown string
IO.inspect result.tables    # list of table maps
IO.inspect result.warnings  # list of warning maps
IO.inspect result.metadata  # metadata map (when enabled)

# With options — pass a map
{:ok, result} = HtmlToMarkdown.convert(html, %{
    heading_style: "atx",
    code_block_style: "backticks",
})

# Tables — in result.tables when include_document_structure is enabled
Enum.each(result.tables, fn table -> IO.puts table.markdown end)
```

Only `convert/1` and `convert/2` are exposed — there is no `convert!` bang
variant and no `create_options_handle`.

---

## R

**Package:** `htmltomarkdown`
**Install:** `install.packages("htmltomarkdown", repos = "https://xberg-io.r-universe.dev")`
**R requirement:** 4.2+

Uses extendr (Rust bindings for R).

```r
library(htmltomarkdown)

# Primary function — convert(html, options = ConversionOptions$default())
result <- convert("<h1>Hello</h1>")
cat(result$content)          # markdown string
length(result$tables)        # table count

# With options — build a ConversionOptions object (no plain named list)
opts <- ConversionOptions$from_json('{"heading_style":"atx","code_block_style":"backticks"}')
result <- convert("<h1>Hello</h1>", opts)

# Metadata — in result$metadata
metadata <- result$metadata

# Tables — in result$tables when include_document_structure is enabled
for (table in result$tables) {
    cat(table$markdown)
}
```

Options are a `ConversionOptions` object, constructed via `ConversionOptions$default()`
or `ConversionOptions$from_json(json)` — passing a bare R list is not supported.
There are no `create_options_handle` / `convert_handle` functions.

---

## WASM

**Package:** `@xberg-io/html-to-markdown-wasm` (built with wasm-pack; nodejs, web,
bundler, and deno targets)

The only exported conversion function is `convert`, which returns a
`WasmConversionResult` **object** (not a JSON string). On the web target you must
`init()` the module first; the nodejs target loads synchronously.

```javascript
// web target: initialize first
import init, { convert } from "@xberg-io/html-to-markdown-wasm";
await init();

// convert(html, options?) — returns a WasmConversionResult object
// convert() takes a WasmConversionOptions instance — a plain object literal is not accepted.
const options = new WasmConversionOptions();
options.headingStyle = WasmHeadingStyle.Atx;
const result = convert("<h1>Hello</h1>", options);
console.log(result.content); // markdown string
console.log(result.tables); // extracted tables
console.log(result.metadata); // metadata (when enabled)

// Tables — in result.tables when includeDocumentStructure is enabled
for (const table of result.tables) {
  console.log(table.markdown);
}
```

Option fields are camelCase with PascalCase enum string values, matching the Node
binding. There are no `convertBytes` / `createConversionOptionsHandle` /
`convertWithOptionsHandle` exports.

---

## Node.js / npm

**Package:** `@xberg-io/html-to-markdown`
**Install:** `pnpm add @xberg-io/html-to-markdown`

Uses NAPI-RS. Platform-specific native binaries are delivered as optional
dependencies named `@xberg-io/html-to-markdown-<platform>`. See the
[TypeScript API Reference](typescript-api.md) for the full surface.

```json
{
  "optionalDependencies": {
    "@xberg-io/html-to-markdown-linux-x64-gnu": "3.8.0",
    "@xberg-io/html-to-markdown-linux-arm64-gnu": "3.8.0",
    "@xberg-io/html-to-markdown-linux-x64-musl": "3.8.0",
    "@xberg-io/html-to-markdown-linux-arm64-musl": "3.8.0",
    "@xberg-io/html-to-markdown-darwin-x64": "3.8.0",
    "@xberg-io/html-to-markdown-darwin-arm64": "3.8.0",
    "@xberg-io/html-to-markdown-win32-x64-msvc": "3.8.0",
    "@xberg-io/html-to-markdown-win32-arm64-msvc": "3.8.0"
  }
}
```

---

## C FFI

**Crate:** `html-to-markdown-ffi`
**Header:** `html_to_markdown.h` (generated by cbindgen)

Used internally by Go, Java, and C# bindings. All exported symbols use the `htm_` prefix. Direct C usage:

```c
#include "html_to_markdown.h"

// htm_convert() — returns an HTMAlefHandle (uint64_t); 0 means failure
// JSON keys are the snake_case Rust field names; unknown keys are rejected.
HTMAlefHandle opts = htm_conversion_options_from_json("{\"heading_style\":\"atx\"}");
HTMAlefHandle result = htm_convert(html_cstr, opts);
htm_conversion_options_free(opts);

if (result) {
    // Extract fields as malloc'd strings
    char *content = htm_conversion_result_content(result);
    // use content ...
    htm_free_string(content);

    htm_conversion_result_free(result);
}

// Error handling
int32_t code = htm_last_error_code();
const char *msg = htm_last_error_context(); // borrowed — do NOT free

// Version
const char *ver = htm_version(); // static — do NOT free
```

**Key FFI contracts:**

- All exported functions use the `htm_` prefix.
- Strings returned by `htm_conversion_result_content()` and similar field accessors must be freed with `htm_free_string()`. Never use the system `free()`.
- `htm_last_error_context()` and `htm_version()` return borrowed/static pointers — do NOT free them.
- Every `_free()` function has a matching allocator (`htm_convert` → `htm_conversion_result_free`, `htm_conversion_options_from_json` → `htm_conversion_options_free`).
