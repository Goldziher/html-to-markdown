package main

import (
	"testing"
	htmltomarkdown "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func TestPackageImports(t *testing.T) {
	// If we can import, test passes
	_ = htmltomarkdown.Convert
}

func TestBasicConversion(t *testing.T) {
	html := "<p>Hello World</p>"
	result, err := htmltomarkdown.Convert(html)
	if err != nil {
		t.Fatalf("conversion failed: %v", err)
	}
	if !contains(result, "Hello World") {
		t.Errorf("expected result to contain 'Hello World', got: %s", result)
	}
}

func TestHeadingConversion(t *testing.T) {
	html := "<h1>Title</h1>"
	result, err := htmltomarkdown.Convert(html)
	if err != nil {
		t.Fatalf("conversion failed: %v", err)
	}
	if result[0] != '#' {
		t.Errorf("expected result to start with '#', got: %s", result)
	}
}

func TestEmptyInput(t *testing.T) {
	result, err := htmltomarkdown.Convert("")
	if err != nil {
		t.Fatalf("conversion failed: %v", err)
	}
	if result != "" {
		t.Errorf("expected empty result, got: %s", result)
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > len(substr) && containsAny(s, substr))
}

func containsAny(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
