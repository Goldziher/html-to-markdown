package htmltomarkdown

import (
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
