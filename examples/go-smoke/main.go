package main

import (
	"fmt"
	"log"
	"os"

	"github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown"
)

func main() {
	// Display library version
	version := htmltomarkdown.Version()
	fmt.Printf("html-to-markdown version: %s\n\n", version)

	// Test 1: Simple HTML conversion
	html1 := "<h1>Hello World</h1><p>This is a test.</p>"
	fmt.Println("Test 1: Simple HTML")
	fmt.Printf("Input:  %s\n", html1)

	markdown1, err := htmltomarkdown.Convert(html1)
	if err != nil {
		log.Fatalf("Conversion failed: %v", err)
	}
	fmt.Printf("Output:\n%s\n\n", markdown1)

	// Test 2: Complex HTML with various elements
	html2 := `
		<html>
			<head><title>Test Page</title></head>
			<body>
				<h1>Main Title</h1>
				<p>This is a paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
				<ul>
					<li>First item</li>
					<li>Second item</li>
					<li>Third item</li>
				</ul>
				<a href="https://example.com">Example link</a>
			</body>
		</html>
	`

	fmt.Println("Test 2: Complex HTML")
	markdown2, err := htmltomarkdown.Convert(html2)
	if err != nil {
		log.Fatalf("Conversion failed: %v", err)
	}
	fmt.Printf("Output:\n%s\n\n", markdown2)

	// Test 3: MustConvert
	fmt.Println("Test 3: MustConvert (panics on error)")
	html3 := "<h2>Testing MustConvert</h2>"
	markdown3 := htmltomarkdown.MustConvert(html3)
	fmt.Printf("Output: %s\n\n", markdown3)

	// Test 4: Empty string handling
	fmt.Println("Test 4: Empty string")
	markdown4, err := htmltomarkdown.Convert("")
	if err != nil {
		log.Fatalf("Conversion failed: %v", err)
	}
	if markdown4 == "" {
		fmt.Println("✓ Empty string correctly returns empty result")
	} else {
		fmt.Printf("✗ Expected empty result, got: %s\n\n", markdown4)
		os.Exit(1)
	}

	// Test 5: Check for common markdown patterns
	html5 := `
		<h1>Heading 1</h1>
		<h2>Heading 2</h2>
		<p>Regular <strong>bold</strong> and <em>italic</em> text.</p>
		<ul>
			<li>Bullet 1</li>
			<li>Bullet 2</li>
		</ul>
	`

	fmt.Println("Test 5: Validation checks")
	markdown5, err := htmltomarkdown.Convert(html5)
	if err != nil {
		log.Fatalf("Conversion failed: %v", err)
	}

	// Validate output contains expected patterns
	checks := []string{"Heading 1", "Heading 2", "bold", "italic", "Bullet 1", "Bullet 2"}
	allPassed := true

	for _, check := range checks {
		if !contains(markdown5, check) {
			fmt.Printf("✗ Output missing expected text: %s\n", check)
			allPassed = false
		}
	}

	if allPassed {
		fmt.Println("✓ All validation checks passed")
	} else {
		fmt.Println("Output was:")
		fmt.Println(markdown5)
		os.Exit(1)
	}

	fmt.Println("\n✅ All smoke tests passed!")
}

func contains(str, substr string) bool {
	return len(str) >= len(substr) && (str == substr || len(str) > len(substr) &&
		(hasSubstring(str, substr)))
}

func hasSubstring(str, substr string) bool {
	for i := 0; i <= len(str)-len(substr); i++ {
		if str[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
