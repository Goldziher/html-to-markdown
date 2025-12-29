```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();

$options = new ConversionOptions(
    headingStyle: 'Atx',
    listIndentWidth: 2,
);

$markdown = $converter->convert('<h1>Hello</h1>', $options);
```
