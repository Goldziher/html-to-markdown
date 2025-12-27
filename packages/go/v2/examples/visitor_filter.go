// Package main provides content filtering visitor examples.
package main

import (
	"fmt"
	"log"
	"regexp"
	"strings"

	"github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

// ExampleVisitorFilter demonstrates filtering content during conversion.
// It removes all external links and custom elements.
func ExampleVisitorFilter() {
	html := `
	<html>
		<body>
			<h1>Article</h1>
			<p>This has a <a href="https://external.com">external link</a> and
			   a <a href="/internal">internal link</a>.</p>
			<p>Some <custom-element>custom content</custom-element> here.</p>
			<p>Regular paragraph with <strong>formatting</strong>.</p>
		</body>
	</html>
	`

	visitor := &htmltomarkdown.Visitor{
		OnLink: func(ctx *htmltomarkdown.NodeContext, href, text, title string) *htmltomarkdown.VisitResult {
			if strings.HasPrefix(href, "/") || strings.HasPrefix(href, "#") {
				return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
		OnCustomElement: func(ctx *htmltomarkdown.NodeContext, tagName, html string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Filtered Markdown (external links and custom elements removed):")
	fmt.Println(markdown)
	fmt.Println()
}

// ExampleVisitorSanitize demonstrates removing specific formatting.
// It strips all code blocks and converts them to regular text.
func ExampleVisitorSanitize() {
	html := `
	<html>
		<body>
			<h1>Code Example</h1>
			<p>Here is a code block:</p>
			<pre><code class="language-go">package main

func main() {
    fmt.Println("Hello")
}</code></pre>
			<p>And here is inline <code>code snippet</code>.</p>
			<p>Regular text continues.</p>
		</body>
	</html>
	`

	visitor := &htmltomarkdown.Visitor{
		OnCodeBlock: func(ctx *htmltomarkdown.NodeContext, lang, code string) *htmltomarkdown.VisitResult {
			codeLines := strings.Split(strings.TrimSpace(code), "\n")
			var output strings.Builder
			for _, line := range codeLines {
				output.WriteString("> ")
				output.WriteString(line)
				output.WriteString("\n")
			}
			return &htmltomarkdown.VisitResult{
				ResultType:   htmltomarkdown.VisitCustom,
				CustomOutput: output.String(),
			}
		},
		OnCodeInline: func(ctx *htmltomarkdown.NodeContext, code string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{
				ResultType:   htmltomarkdown.VisitCustom,
				CustomOutput: fmt.Sprintf("\"%s\"", code),
			}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Sanitized Markdown (code converted to text):")
	fmt.Println(markdown)
	fmt.Println()
}

// ExampleVisitorValidateLinks demonstrates validating all links.
// It collects invalid links and reports them.
func ExampleVisitorValidateLinks() {
	html := `
	<html>
		<body>
			<h1>Links</h1>
			<p><a href="https://valid.com">Valid link</a></p>
			<p><a href="javascript:void(0)">JavaScript link</a></p>
			<p><a href="">Empty link</a></p>
			<p><a href="#anchor">Anchor link</a></p>
			<p><a href="ftp://files.example.com">FTP link</a></p>
		</body>
	</html>
	`

	type LinkIssue struct {
		Href  string
		Text  string
		Issue string
	}

	var issues []LinkIssue

	visitor := &htmltomarkdown.Visitor{
		OnLink: func(ctx *htmltomarkdown.NodeContext, href, text, title string) *htmltomarkdown.VisitResult {
			var issue string
			switch {
			case href == "":
				issue = "empty URL"
			case strings.HasPrefix(href, "javascript:"):
				issue = "javascript protocol"
			case !isValidURL(href):
				issue = "invalid URL format"
			}
			if issue != "" {
				issues = append(issues, LinkIssue{href, text, issue})
			}
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Link Validation Report:")
	if len(issues) > 0 {
		for i, issue := range issues {
			fmt.Printf("%d. [%s] %s - %s\n", i+1, issue.Text, issue.Href, issue.Issue)
		}
	} else {
		fmt.Println("All links are valid!")
	}
	fmt.Println()
	fmt.Println("Markdown output:")
	fmt.Println(markdown)
}

// isValidURL is a simple URL validator.
func isValidURL(url string) bool {
	pattern := `^(https?|ftp|file)://|^/|^#|^mailto:`
	match, err := regexp.MatchString(pattern, url)
	if err != nil {
		return false
	}
	return match
}

// ExampleVisitorRemoveImages demonstrates removing all images.
func ExampleVisitorRemoveImages() {
	html := `
	<html>
		<body>
			<h1>Article with Images</h1>
			<p>Introduction text.</p>
			<img src="image1.jpg" alt="First image" />
			<p>Middle paragraph.</p>
			<img src="image2.png" alt="Second image" />
			<p>Conclusion text.</p>
		</body>
	</html>
	`

	visitor := &htmltomarkdown.Visitor{
		OnImage: func(ctx *htmltomarkdown.NodeContext, src, alt, title string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Markdown without images:")
	fmt.Println(markdown)
	fmt.Println()
}

// ExampleVisitorRemoveMedia demonstrates removing media elements.
func ExampleVisitorRemoveMedia() {
	html := `
	<html>
		<body>
			<h1>Media Article</h1>
			<p>Listen to this:</p>
			<audio src="audio.mp3"></audio>
			<p>Watch this:</p>
			<video src="video.mp4"></video>
			<p>Embedded content:</p>
			<iframe src="https://example.com/embed"></iframe>
		</body>
	</html>
	`

	visitor := &htmltomarkdown.Visitor{
		OnAudio: func(ctx *htmltomarkdown.NodeContext, src string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
		OnVideo: func(ctx *htmltomarkdown.NodeContext, src string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
		OnIframe: func(ctx *htmltomarkdown.NodeContext, src string) *htmltomarkdown.VisitResult {
			return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
		},
	}

	markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Markdown without media elements:")
	fmt.Println(markdown)
}
