package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
	htmltomarkdown "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

type TestCase struct {
	Name             string                 `json:"name"`
	HTML             string                 `json:"html"`
	ExpectedMarkdown string                 `json:"expectedMarkdown"`
	Options          map[string]interface{} `json:"options"`
}

func loadFixtures(t *testing.T, filename string) []TestCase {
	fixturePath := filepath.Join("..", "fixtures", filename)
	data, err := os.ReadFile(fixturePath)
	if err != nil {
		t.Fatalf("failed to read fixture: %v", err)
	}

	var cases []TestCase
	if err := json.Unmarshal(data, &cases); err != nil {
		t.Fatalf("failed to parse fixture: %v", err)
	}

	return cases
}

func TestBasicHTMLConversions(t *testing.T) {
	fixtures := loadFixtures(t, "basic-html.json")

	for _, tc := range fixtures {
		t.Run(tc.Name, func(t *testing.T) {
			result, err := htmltomarkdown.Convert(tc.HTML)
			if err != nil {
				t.Fatalf("conversion failed: %v", err)
			}

			if strings.TrimSpace(result) != strings.TrimSpace(tc.ExpectedMarkdown) {
				t.Errorf("expected: %q, got: %q", tc.ExpectedMarkdown, result)
			}
		})
	}
}
