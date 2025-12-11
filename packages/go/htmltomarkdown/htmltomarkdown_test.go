package htmltomarkdown

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestConvert(t *testing.T) {
	tests := []struct {
		name     string
		html     string
		contains string
		wantErr  bool
	}{
		{
			name:     "simple heading",
			html:     "<h1>Hello World</h1>",
			contains: "Hello World",
			wantErr:  false,
		},
		{
			name:     "paragraph",
			html:     "<p>This is a paragraph.</p>",
			contains: "This is a paragraph",
			wantErr:  false,
		},
		{
			name:     "link",
			html:     `<a href="https://example.com">Example</a>`,
			contains: "Example",
			wantErr:  false,
		},
		{
			name:     "strong text",
			html:     "<strong>Bold text</strong>",
			contains: "Bold text",
			wantErr:  false,
		},
		{
			name:     "emphasis",
			html:     "<em>Italic text</em>",
			contains: "Italic text",
			wantErr:  false,
		},
		{
			name:     "list",
			html:     "<ul><li>Item 1</li><li>Item 2</li></ul>",
			contains: "Item 1",
			wantErr:  false,
		},
		{
			name:     "empty string",
			html:     "",
			contains: "",
			wantErr:  false,
		},
		{
			name:     "complex HTML",
			html:     "<div><h1>Title</h1><p>Content</p></div>",
			contains: "Title",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := Convert(tt.html)
			if (err != nil) != tt.wantErr {
				t.Errorf("Convert() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !strings.Contains(got, tt.contains) {
				t.Errorf("Convert() = %v, want to contain %v", got, tt.contains)
			}
		})
	}
}

func TestMustConvert(t *testing.T) {
	t.Run("successful conversion", func(t *testing.T) {
		html := "<h1>Test</h1>"
		result := MustConvert(html)
		if !strings.Contains(result, "Test") {
			t.Errorf("MustConvert() = %v, want to contain 'Test'", result)
		}
	})

	t.Run("empty string", func(t *testing.T) {
		result := MustConvert("")
		if result != "" {
			t.Errorf("MustConvert(\"\") = %v, want empty string", result)
		}
	})
}

func TestVersion(t *testing.T) {
	version := Version()
	if version == "" {
		t.Error("Version() returned empty string")
	}
	if version == "unknown" {
		t.Error("Version() returned 'unknown'")
	}
	t.Logf("Library version: %s", version)
}

func BenchmarkConvert(b *testing.B) {
	html := `
		<html>
			<head><title>Test</title></head>
			<body>
				<h1>Title</h1>
				<p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
				<ul>
					<li>Item 1</li>
					<li>Item 2</li>
					<li>Item 3</li>
				</ul>
			</body>
		</html>
	`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := Convert(html)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkConvertSimple(b *testing.B) {
	html := "<h1>Hello World</h1>"

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := Convert(html)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func ExampleConvert() {
	markdown, err := Convert("<h1>Hello World</h1>")
	if err != nil {
		panic(err)
	}
	println(markdown)
}

func ExampleMustConvert() {
	markdown := MustConvert("<p>This is a paragraph.</p>")
	println(markdown)
}

func ExampleVersion() {
	version := Version()
	println("html-to-markdown version:", version)
}

func TestConvertWithMetadata(t *testing.T) {
	tests := []struct {
		name          string
		html          string
		checkMarkdown func(string) bool
		checkMetadata func(*testing.T, ExtendedMetadata)
		wantErr       bool
	}{
		{
			name: "simple document with title",
			html: "<html><head><title>Test Page</title></head><body><h1>Hello</h1></body></html>",
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "Hello")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if meta.Document.Title == nil || *meta.Document.Title != "Test Page" {
					t.Errorf("Expected title 'Test Page', got %v", meta.Document.Title)
				}
			},
			wantErr: false,
		},
		{
			name: "document with headers",
			html: `<html>
				<head><title>Headers Test</title></head>
				<body>
					<h1>Main Title</h1>
					<h2>Subtitle</h2>
					<h3>Sub-subtitle</h3>
				</body>
			</html>`,
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "Main Title") && strings.Contains(md, "Subtitle")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if len(meta.Headers) < 3 {
					t.Errorf("Expected at least 3 headers, got %d", len(meta.Headers))
				}
				// Check first header is level 1
				if len(meta.Headers) > 0 && meta.Headers[0].Level != 1 {
					t.Errorf("Expected first header level 1, got %d", meta.Headers[0].Level)
				}
			},
			wantErr: false,
		},
		{
			name: "document with links",
			html: `<html>
				<body>
					<a href="https://example.com">External Link</a>
					<a href="/internal">Internal Link</a>
					<a href="#anchor">Anchor Link</a>
					<a href="mailto:test@example.com">Email Link</a>
				</body>
			</html>`,
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "External Link") && strings.Contains(md, "Internal Link")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if len(meta.Links) < 4 {
					t.Errorf("Expected at least 4 links, got %d", len(meta.Links))
				}
				// Check link types
				linkTypes := make(map[string]int)
				for _, link := range meta.Links {
					linkTypes[string(link.LinkType)]++
				}
				if linkTypes[string(LinkTypeExternal)] == 0 {
					t.Error("Expected at least one external link")
				}
			},
			wantErr: false,
		},
		{
			name: "document with images",
			html: `<html>
				<body>
					<img src="https://example.com/image.jpg" alt="Example image">
					<img src="/relative/path.png" alt="Relative image">
				</body>
			</html>`,
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "Example image")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if len(meta.Images) < 2 {
					t.Errorf("Expected at least 2 images, got %d", len(meta.Images))
				}
				// Check image metadata
				for _, img := range meta.Images {
					if img.Alt == nil || *img.Alt == "" {
						t.Error("Expected image to have alt text")
					}
				}
			},
			wantErr: false,
		},
		{
			name: "document with meta tags",
			html: `<html>
				<head>
					<title>Meta Test</title>
					<meta name="description" content="Test description">
					<meta name="keywords" content="test,metadata,extraction">
					<meta name="author" content="Test Author">
					<link rel="canonical" href="https://example.com/canonical">
				</head>
				<body><p>Content</p></body>
			</html>`,
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "Content")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if meta.Document.Title == nil || *meta.Document.Title != "Meta Test" {
					t.Errorf("Expected title 'Meta Test', got %v", meta.Document.Title)
				}
				if meta.Document.Description == nil || *meta.Document.Description != "Test description" {
					t.Errorf("Expected description 'Test description', got %v", meta.Document.Description)
				}
				if meta.Document.Author == nil || *meta.Document.Author != "Test Author" {
					t.Errorf("Expected author 'Test Author', got %v", meta.Document.Author)
				}
			},
			wantErr: false,
		},
		{
			name: "empty HTML",
			html: "",
			checkMarkdown: func(md string) bool {
				return md == ""
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				// Empty HTML should produce empty metadata
			},
			wantErr: false,
		},
		{
			name: "complex document",
			html: `<html lang="en">
				<head>
					<title>Complex Article</title>
					<meta name="description" content="A comprehensive test article">
					<meta property="og:title" content="Complex Article">
					<meta property="og:description" content="Article description">
					<meta property="og:image" content="https://example.com/image.jpg">
				</head>
				<body>
					<h1>Main Article Title</h1>
					<p>Introduction paragraph.</p>
					<h2>Section 1</h2>
					<p>Content with <a href="https://example.com">a link</a> and <img src="img.jpg" alt="article image">.</p>
					<h2>Section 2</h2>
					<p>More content.</p>
				</body>
			</html>`,
			checkMarkdown: func(md string) bool {
				return strings.Contains(md, "Main Article Title") && strings.Contains(md, "Section 1")
			},
			checkMetadata: func(t *testing.T, meta ExtendedMetadata) {
				if meta.Document.Title == nil || *meta.Document.Title != "Complex Article" {
					t.Errorf("Expected title, got %v", meta.Document.Title)
				}
				if meta.Document.Language == nil || *meta.Document.Language != "en" {
					t.Errorf("Expected language 'en', got %v", meta.Document.Language)
				}
				if len(meta.Headers) < 3 {
					t.Errorf("Expected at least 3 headers, got %d", len(meta.Headers))
				}
				if len(meta.Links) == 0 {
					t.Error("Expected at least one link")
				}
				if len(meta.Images) == 0 {
					t.Error("Expected at least one image")
				}
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := ConvertWithMetadata(tt.html)
			if (err != nil) != tt.wantErr {
				t.Errorf("ConvertWithMetadata() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.checkMarkdown(result.Markdown) {
				t.Errorf("ConvertWithMetadata() markdown check failed: %s", result.Markdown)
			}

			tt.checkMetadata(t, result.Metadata)
		})
	}
}

func TestMustConvertWithMetadata(t *testing.T) {
	t.Run("successful conversion", func(t *testing.T) {
		html := "<h1>Test</h1>"
		result := MustConvertWithMetadata(html)
		if !strings.Contains(result.Markdown, "Test") {
			t.Errorf("MustConvertWithMetadata() markdown = %v, want to contain 'Test'", result.Markdown)
		}
	})

	t.Run("empty string", func(t *testing.T) {
		result := MustConvertWithMetadata("")
		if result.Markdown != "" {
			t.Errorf("MustConvertWithMetadata(\"\") = %v, want empty string", result.Markdown)
		}
	})
}

func TestMetadataJSONMarshaling(t *testing.T) {
	html := `<html>
		<head>
			<title>JSON Test</title>
			<meta name="description" content="Test">
		</head>
		<body>
			<h1>Title</h1>
			<a href="https://example.com">Link</a>
		</body>
	</html>`

	result, err := ConvertWithMetadata(html)
	if err != nil {
		t.Fatalf("ConvertWithMetadata failed: %v", err)
	}

	// Verify metadata can be marshaled back to JSON
	jsonData, err := json.Marshal(result.Metadata)
	if err != nil {
		t.Fatalf("Failed to marshal metadata to JSON: %v", err)
	}

	// Verify JSON contains expected fields
	if !strings.Contains(string(jsonData), "\"document\"") {
		t.Error("Expected 'document' field in metadata JSON")
	}
	if !strings.Contains(string(jsonData), "\"headers\"") {
		t.Error("Expected 'headers' field in metadata JSON")
	}
	if !strings.Contains(string(jsonData), "\"links\"") {
		t.Error("Expected 'links' field in metadata JSON")
	}

	// Verify round-trip unmarshaling works
	var roundTrip ExtendedMetadata
	if err := json.Unmarshal(jsonData, &roundTrip); err != nil {
		t.Fatalf("Failed to unmarshal metadata: %v", err)
	}

	if roundTrip.Document.Title == nil || *roundTrip.Document.Title != "JSON Test" {
		t.Error("Round-trip metadata lost title")
	}
}

func TestHeaderMetadataValidation(t *testing.T) {
	html := `<html>
		<body>
			<h1>H1 Title</h1>
			<h2>H2 Title</h2>
			<h3>H3 Title</h3>
			<h4>H4 Title</h4>
			<h5>H5 Title</h5>
			<h6>H6 Title</h6>
		</body>
	</html>`

	result, err := ConvertWithMetadata(html)
	if err != nil {
		t.Fatalf("ConvertWithMetadata failed: %v", err)
	}

	if len(result.Metadata.Headers) != 6 {
		t.Errorf("Expected 6 headers, got %d", len(result.Metadata.Headers))
	}

	// Verify header levels are correct
	for i, header := range result.Metadata.Headers {
		expectedLevel := uint8(i + 1)
		if header.Level != expectedLevel {
			t.Errorf("Header %d: expected level %d, got %d", i, expectedLevel, header.Level)
		}
	}
}

func TestLinkTypeClassification(t *testing.T) {
	html := `<html>
		<body>
			<a href="https://example.com">External</a>
			<a href="http://example.com">HTTP External</a>
			<a href="/internal/page">Internal</a>
			<a href="./relative">Relative</a>
			<a href="#section">Anchor</a>
			<a href="mailto:test@example.com">Email</a>
			<a href="tel:+1234567890">Phone</a>
		</body>
	</html>`

	result, err := ConvertWithMetadata(html)
	if err != nil {
		t.Fatalf("ConvertWithMetadata failed: %v", err)
	}

	linkTypeCount := make(map[LinkType]int)
	for _, link := range result.Metadata.Links {
		linkTypeCount[link.LinkType]++
	}

	if linkTypeCount[LinkTypeExternal] < 2 {
		t.Errorf("Expected at least 2 external links, got %d", linkTypeCount[LinkTypeExternal])
	}
	if linkTypeCount[LinkTypeInternal] == 0 {
		t.Error("Expected at least 1 internal link")
	}
	if linkTypeCount[LinkTypeAnchor] == 0 {
		t.Error("Expected at least 1 anchor link")
	}
	if linkTypeCount[LinkTypeEmail] == 0 {
		t.Error("Expected at least 1 email link")
	}
	if linkTypeCount[LinkTypePhone] == 0 {
		t.Error("Expected at least 1 phone link")
	}
}

func TestImageTypeClassification(t *testing.T) {
	html := `<html>
		<body>
			<img src="https://example.com/image.jpg" alt="External">
			<img src="http://cdn.example.com/image.png" alt="HTTP External">
			<img src="/images/local.jpg" alt="Relative">
			<img src="./images/local2.png" alt="Relative dot slash">
		</body>
	</html>`

	result, err := ConvertWithMetadata(html)
	if err != nil {
		t.Fatalf("ConvertWithMetadata failed: %v", err)
	}

	imageTypeCount := make(map[ImageType]int)
	for _, img := range result.Metadata.Images {
		imageTypeCount[img.ImageType]++
	}

	if imageTypeCount[ImageTypeExternal] < 2 {
		t.Errorf("Expected at least 2 external images, got %d", imageTypeCount[ImageTypeExternal])
	}
	if imageTypeCount[ImageTypeRelative] < 2 {
		t.Errorf("Expected at least 2 relative images, got %d", imageTypeCount[ImageTypeRelative])
	}
}

func BenchmarkConvertWithMetadata(b *testing.B) {
	html := `<html>
		<head>
			<title>Benchmark Test</title>
			<meta name="description" content="Benchmark article">
			<meta name="author" content="Test Author">
		</head>
		<body>
			<h1>Main Title</h1>
			<p>Introduction paragraph with <a href="https://example.com">external link</a> and <img src="image.jpg" alt="image">.</p>
			<h2>Section 1</h2>
			<p>Content section.</p>
			<h2>Section 2</h2>
			<p>More content with <a href="/internal">internal link</a>.</p>
		</body>
	</html>`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := ConvertWithMetadata(html)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkConvertWithMetadataSimple(b *testing.B) {
	html := "<h1>Hello World</h1>"

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := ConvertWithMetadata(html)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func ExampleConvertWithMetadata() {
	html := `<html>
		<head>
			<title>My Article</title>
			<meta name="description" content="A great article">
		</head>
		<body>
			<h1>Article Title</h1>
			<p>Content with <a href="https://example.com">a link</a></p>
			<img src="image.jpg" alt="Article image">
		</body>
	</html>`

	result, err := ConvertWithMetadata(html)
	if err != nil {
		panic(err)
	}

	println("Markdown:", result.Markdown)
	if result.Metadata.Document.Title != nil {
		println("Title:", *result.Metadata.Document.Title)
	}
	println("Headers:", len(result.Metadata.Headers))
	println("Links:", len(result.Metadata.Links))
	println("Images:", len(result.Metadata.Images))
}

func ExampleMustConvertWithMetadata() {
	result := MustConvertWithMetadata("<h1>Title</h1>")
	println("Markdown:", result.Markdown)
}
