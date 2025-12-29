```python
from html_to_markdown import convert_with_metadata, MetadataConfig

metadata_config = MetadataConfig(
    extract_headers=True,
    extract_links=True,
    extract_images=True,
    extract_structured_data=True,
    max_structured_data_size=100000,
)
markdown, metadata = convert_with_metadata(html, metadata_config=metadata_config)
```
