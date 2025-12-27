// Package main provides basic visitor pattern examples.
package main

import (
	"fmt"
	"log"
	"strings"

	"github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

// ExampleVisitorBasic demonstrates basic visitor pattern usage.
// It transforms all links to show both the URL and text.
func ExampleVisitorBasic() {
	html := `
	<html>
		<body>
			<h1>Article Title</h1>
			<p>This is a paragraph with a <a href="https://example.com">link to example</a>.</p>
			<p>Another paragraph with <a href="https://golang.org">Go documentation</a>.</p>
		</body>
	</html>
	`

	visitor := &htmltomarkdown.Visitor{
		OnLink: func(ctx *htmltomarkdown.NodeContext, href, text, title string) *htmltomarkdown.VisitResult {
			customOutput := fmt.Sprintf("[%s](%s) [%s]", strings.TrimSpace(text), href, href)
			return &htmltomarkdown.VisitResult{
				ResultType:   htmltomarkdown.VisitCustom,
				CustomOutput: customOutput,
			}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Converted Markdown:")
	fmt.Println(markdown)
	fmt.Println()

}

// ExampleVisitorHeadings demonstrates extracting heading information.
// It collects all headings and prints them with their levels.
func ExampleVisitorHeadings() {
	html := `
	<h1>Main Title</h1>
	<h2>Section 1</h2>
	<p>Content under section 1</p>
	<h2>Section 2</h2>
	<h3>Subsection 2.1</h3>
	<p>Content under subsection</p>
	`

	type Heading struct {
		Level int
		Text  string
		ID    string
	}

	var headings []Heading

	visitor := &htmltomarkdown.Visitor{
		OnHeading: func(ctx *htmltomarkdown.NodeContext, level uint32, text, id string) *htmltomarkdown.VisitResult {
			headings = append(headings, Heading{
				Level: int(level),
				Text:  strings.TrimSpace(text),
				ID:    id,
			})
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Document Outline:")
	for _, h := range headings {
		indent := strings.Repeat("  ", h.Level-1)
		fmt.Printf("%sH%d: %s\n", indent, h.Level, h.Text)
	}
	fmt.Println()
	fmt.Println("Markdown output:")
	fmt.Println(markdown)
}

// ExampleVisitorElementTracking demonstrates tracking element depth and position.
func ExampleVisitorElementTracking() {
	html := `
	<div>
		<h1>Title</h1>
		<p>First paragraph</p>
		<p>Second paragraph</p>
		<ul>
			<li>Item 1</li>
			<li>Item 2</li>
		</ul>
	</div>
	`

	depthMap := make(map[string][]uint64)

	visitor := &htmltomarkdown.Visitor{
		OnElementStart: func(ctx *htmltomarkdown.NodeContext) *htmltomarkdown.VisitResult {
			tagName := ctx.TagName
			if tagName != "" {
				depthMap[tagName] = append(depthMap[tagName], ctx.Depth)
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Element Depth Analysis:")
	for tag, depths := range depthMap {
		if tag != "" && tag != "body" && tag != "html" {
			fmt.Printf("<%s>: depths %v\n", tag, depths)
		}
	}
	fmt.Println()
	fmt.Println("Markdown output:")
	fmt.Println(markdown)
}
