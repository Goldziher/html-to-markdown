```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert_with_metadata;

$html = '<html><head><title>Example</title></head><body><h1>Welcome</h1><a href="https://example.com">Link</a></body></html>';

// Object-oriented API
$converter = Converter::create();
$result = $converter->convertWithMetadata(
    $html,
    new ConversionOptions(headingStyle: 'Atx'),
    [
        'extract_headers' => true,
        'extract_links' => true,
        'extract_images' => true,
    ]
);

echo $result['markdown'];
echo $result['metadata']->document->title;
foreach ($result['metadata']->links as $link) {
    echo $link->href . ': ' . $link->text;
}

// Procedural API
$result = convert_with_metadata(
    $html,
    new ConversionOptions(headingStyle: 'Atx'),
    ['extract_headers' => true, 'extract_links' => true]
);
```
