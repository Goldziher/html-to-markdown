```php
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert;

// Object-oriented usage
$converter = Converter::create();
$markdown = $converter->convert('<h1>Hello</h1><p>This is <strong>fast</strong>!</p>');

// Procedural helper
$markdown = convert('<h1>Hello</h1>');
```
