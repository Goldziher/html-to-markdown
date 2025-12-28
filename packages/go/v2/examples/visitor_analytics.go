// Package main provides analytics visitor examples.
package main

import (
	"fmt"
	"log"
	"strings"

	"github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

// DocumentStats tracks statistics about an HTML document.
type DocumentStats struct {
	HeadingCount    int
	ParagraphCount  int
	LinkCount       int
	ImageCount      int
	CodeBlockCount  int
	CodeInlineCount int
	ListCount       int
	ListItemCount   int
	TableCount      int
	TableRowCount   int
	BlockquoteCount int
	TotalTextLength int
	UniqueLinks     map[string]bool
	TextNodes       []string
	MaxHeadingLevel int
}

// NewDocumentStats creates a new DocumentStats instance.
func NewDocumentStats() *DocumentStats {
	return &DocumentStats{
		UniqueLinks: make(map[string]bool),
		TextNodes:   make([]string, 0),
	}
}

// ExampleVisitorAnalytics demonstrates analyzing document structure.
// It gathers statistics about headings, links, images, and more.
func ExampleVisitorAnalytics() {
	html := `
	<html>
		<body>
			<h1>Main Title</h1>
			<p>First paragraph with <a href="https://example.com">a link</a>.</p>
			<h2>Section 1</h2>
			<p>Second paragraph with <strong>formatting</strong>.</p>
			<img src="image.jpg" alt="An image" />
			<h3>Subsection</h3>
			<p>Code example: <code>package main</code></p>
			<pre><code>func main() {}</code></pre>
			<ul>
				<li>Item 1</li>
				<li>Item 2</li>
			</ul>
			<blockquote>A quote</blockquote>
			<table>
				<tr><th>Header</th></tr>
				<tr><td>Cell</td></tr>
			</table>
		</body>
	</html>
	`

	stats := NewDocumentStats()

	visitor := &htmltomarkdown.Visitor{
		OnText: func(ctx *htmltomarkdown.NodeContext, text string) *htmltomarkdown.VisitResult {
			text = strings.TrimSpace(text)
			if text != "" {
				stats.TextNodes = append(stats.TextNodes, text)
				stats.TotalTextLength += len(text)
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnHeading: func(ctx *htmltomarkdown.NodeContext, level uint32, text, id string) *htmltomarkdown.VisitResult {
			stats.HeadingCount++
			if int(level) > stats.MaxHeadingLevel {
				stats.MaxHeadingLevel = int(level)
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnLink: func(ctx *htmltomarkdown.NodeContext, href, text, title string) *htmltomarkdown.VisitResult {
			stats.LinkCount++
			stats.UniqueLinks[href] = true
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnImage: func(ctx *htmltomarkdown.NodeContext, src, alt, title string) *htmltomarkdown.VisitResult {
			stats.ImageCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnCodeBlock: func(ctx *htmltomarkdown.NodeContext, lang, code string) *htmltomarkdown.VisitResult {
			stats.CodeBlockCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnCodeInline: func(ctx *htmltomarkdown.NodeContext, code string) *htmltomarkdown.VisitResult {
			stats.CodeInlineCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnListStart: func(ctx *htmltomarkdown.NodeContext, ordered bool) *htmltomarkdown.VisitResult {
			stats.ListCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnListItem: func(ctx *htmltomarkdown.NodeContext, ordered bool, marker, text string) *htmltomarkdown.VisitResult {
			stats.ListItemCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnTableStart: func(ctx *htmltomarkdown.NodeContext) *htmltomarkdown.VisitResult {
			stats.TableCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnTableRow: func(ctx *htmltomarkdown.NodeContext, cells []string, isHeader bool) *htmltomarkdown.VisitResult {
			stats.TableRowCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnBlockquote: func(ctx *htmltomarkdown.NodeContext, content string, depth uint64) *htmltomarkdown.VisitResult {
			stats.BlockquoteCount++
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Document Statistics:")
	fmt.Println(strings.Repeat("=", 50))
	fmt.Printf("Headings:           %d (max level: H%d)\n", stats.HeadingCount, stats.MaxHeadingLevel)
	fmt.Printf("Paragraphs:         (implicit in conversion)\n")
	fmt.Printf("Links:              %d (unique: %d)\n", stats.LinkCount, len(stats.UniqueLinks))
	fmt.Printf("Images:             %d\n", stats.ImageCount)
	fmt.Printf("Code blocks:        %d\n", stats.CodeBlockCount)
	fmt.Printf("Inline code:        %d\n", stats.CodeInlineCount)
	fmt.Printf("Lists:              %d\n", stats.ListCount)
	fmt.Printf("List items:         %d\n", stats.ListItemCount)
	fmt.Printf("Tables:             %d\n", stats.TableCount)
	fmt.Printf("Table rows:         %d\n", stats.TableRowCount)
	fmt.Printf("Blockquotes:        %d\n", stats.BlockquoteCount)
	fmt.Printf("Total text length:  %d characters\n", stats.TotalTextLength)
	fmt.Printf("Text nodes found:   %d\n", len(stats.TextNodes))
	fmt.Println(strings.Repeat("=", 50))

	fmt.Println("\nUnique Links:")
	for link := range stats.UniqueLinks {
		fmt.Printf("  - %s\n", link)
	}

	fmt.Println("\nConverted Markdown:")
	fmt.Println(markdown)
}

// ExampleVisitorReadingTime demonstrates calculating approximate reading time.
func ExampleVisitorReadingTime() {
	html := `
	<html>
		<body>
			<h1>Long Article</h1>
			<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.
			   Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
			<p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris
			   nisi ut aliquip ex ea commodo consequat.</p>
			<p>Duis aute irure dolor in reprehenderit in voluptate velit esse
			   cillum dolore eu fugiat nulla pariatur.</p>
		</body>
	</html>
	`

	const wordsPerMinute = 200

	var totalWords int

	visitor := &htmltomarkdown.Visitor{
		OnText: func(ctx *htmltomarkdown.NodeContext, text string) *htmltomarkdown.VisitResult {
			text = strings.TrimSpace(text)
			if text != "" {
				words := strings.Fields(text)
				totalWords += len(words)
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	readingTimeMinutes := totalWords / wordsPerMinute
	if readingTimeMinutes == 0 {
		readingTimeMinutes = 1
	}

	fmt.Println("Reading Time Estimate:")
	fmt.Printf("Total words: %d\n", totalWords)
	fmt.Printf("Reading speed: %d words/minute\n", wordsPerMinute)
	fmt.Printf("Estimated reading time: %d minute(s)\n", readingTimeMinutes)
	fmt.Println()
	fmt.Println("Markdown:")
	fmt.Println(markdown)
}

// ExampleVisitorContentExtraction demonstrates extracting specific content types.
func ExampleVisitorContentExtraction() {
	html := `
	<html>
		<body>
			<h1>Technical Guide</h1>
			<p>Introduction paragraph.</p>
			<h2>Installation</h2>
			<p>Installation instructions.</p>
			<pre><code>$ go get example.com/package</code></pre>
			<h2>Usage</h2>
			<p>Usage instructions with <code>example code</code>.</p>
			<blockquote>Important note for users.</blockquote>
		</body>
	</html>
	`

	type Content struct {
		Headings   []string
		CodeBlocks []string
		Quotes     []string
	}

	content := Content{
		Headings:   make([]string, 0),
		CodeBlocks: make([]string, 0),
		Quotes:     make([]string, 0),
	}

	visitor := &htmltomarkdown.Visitor{
		OnHeading: func(ctx *htmltomarkdown.NodeContext, level uint32, text, id string) *htmltomarkdown.VisitResult {
			content.Headings = append(content.Headings, fmt.Sprintf("H%d: %s", level, text))
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnCodeBlock: func(ctx *htmltomarkdown.NodeContext, lang, code string) *htmltomarkdown.VisitResult {
			content.CodeBlocks = append(content.CodeBlocks, strings.TrimSpace(code))
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
		OnBlockquote: func(ctx *htmltomarkdown.NodeContext, content_text string, depth uint64) *htmltomarkdown.VisitResult {
			content.Quotes = append(content.Quotes, strings.TrimSpace(content_text))
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Extracted Content:")
	fmt.Println()
	fmt.Println("Headings:")
	for i, h := range content.Headings {
		fmt.Printf("  %d. %s\n", i+1, h)
	}

	fmt.Println("\nCode Blocks:")
	for i, cb := range content.CodeBlocks {
		fmt.Printf("  %d. %s\n", i+1, cb)
	}

	fmt.Println("\nQuotes:")
	for i, q := range content.Quotes {
		fmt.Printf("  %d. %s\n", i+1, q)
	}

	fmt.Println("\nFull Markdown:")
	fmt.Println(markdown)
}
