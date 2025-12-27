package htmltomarkdown

import (
	"strings"
	"testing"
)

func TestVisitResultType(t *testing.T) {
	tests := []struct {
		name     string
		input    VisitResultType
		expected int
	}{
		{
			name:     "VisitContinue",
			input:    VisitContinue,
			expected: 0,
		},
		{
			name:     "VisitCustom",
			input:    VisitCustom,
			expected: 1,
		},
		{
			name:     "VisitSkip",
			input:    VisitSkip,
			expected: 2,
		},
		{
			name:     "VisitPreserveHTML",
			input:    VisitPreserveHTML,
			expected: 3,
		},
		{
			name:     "VisitError",
			input:    VisitError,
			expected: 4,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if int(tt.input) != tt.expected {
				t.Errorf("VisitResultType %v = %d, expected %d", tt.name, int(tt.input), tt.expected)
			}
		})
	}
}

func TestNodeContext(t *testing.T) {
	ctx := &NodeContext{
		NodeType:      1,
		TagName:       "div",
		ParentTag:     "body",
		Depth:         2,
		IndexInParent: 0,
		IsInline:      false,
	}

	if ctx.NodeType != 1 {
		t.Errorf("NodeContext.NodeType = %d, expected 1", ctx.NodeType)
	}
	if ctx.TagName != "div" {
		t.Errorf("NodeContext.TagName = %s, expected 'div'", ctx.TagName)
	}
	if ctx.ParentTag != "body" {
		t.Errorf("NodeContext.ParentTag = %s, expected 'body'", ctx.ParentTag)
	}
	if ctx.Depth != 2 {
		t.Errorf("NodeContext.Depth = %d, expected 2", ctx.Depth)
	}
	if ctx.IndexInParent != 0 {
		t.Errorf("NodeContext.IndexInParent = %d, expected 0", ctx.IndexInParent)
	}
	if ctx.IsInline {
		t.Error("NodeContext.IsInline should be false")
	}
}

func TestVisitResult(t *testing.T) {
	t.Run("Continue result", func(t *testing.T) {
		vr := &VisitResult{
			ResultType: VisitContinue,
		}
		if vr.ResultType != VisitContinue {
			t.Errorf("VisitResult.ResultType = %v, expected VisitContinue", vr.ResultType)
		}
	})

	t.Run("Custom result", func(t *testing.T) {
		vr := &VisitResult{
			ResultType:   VisitCustom,
			CustomOutput: "**custom**",
		}
		if vr.ResultType != VisitCustom {
			t.Errorf("VisitResult.ResultType = %v, expected VisitCustom", vr.ResultType)
		}
		if vr.CustomOutput != "**custom**" {
			t.Errorf("VisitResult.CustomOutput = %s, expected '**custom**'", vr.CustomOutput)
		}
	})

	t.Run("Error result", func(t *testing.T) {
		vr := &VisitResult{
			ResultType:   VisitError,
			ErrorMessage: "conversion error",
		}
		if vr.ResultType != VisitError {
			t.Errorf("VisitResult.ResultType = %v, expected VisitError", vr.ResultType)
		}
		if vr.ErrorMessage != "conversion error" {
			t.Errorf("VisitResult.ErrorMessage = %s, expected 'conversion error'", vr.ErrorMessage)
		}
	})
}

func TestConvertWithVisitor_NilVisitor(t *testing.T) {
	html := "<h1>Test</h1>"
	result, err := ConvertWithVisitor(html, nil)
	if err != nil {
		t.Errorf("ConvertWithVisitor with nil visitor should not error: %v", err)
	}
	if !strings.Contains(result, "Test") {
		t.Errorf("ConvertWithVisitor returned %s, expected to contain 'Test'", result)
	}
}

func TestConvertWithVisitor_EmptyHTML(t *testing.T) {
	visitor := &Visitor{}
	result, err := ConvertWithVisitor("", visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor with empty HTML should not error: %v", err)
	}
	if result != "" {
		t.Errorf("ConvertWithVisitor(\"\") returned %s, expected empty string", result)
	}
}

func TestConvertWithVisitor_LinkVisitor(t *testing.T) {
	html := `<a href="https://example.com">Example Link</a>`

	linkCalled := false
	visitor := &Visitor{
		OnLink: func(ctx *NodeContext, href, text, title string) *VisitResult {
			linkCalled = true
			if href != "https://example.com" {
				t.Errorf("OnLink href = %s, expected 'https://example.com'", href)
			}
			if !strings.Contains(text, "Example") {
				t.Errorf("OnLink text = %s, expected to contain 'Example'", text)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !linkCalled {
		t.Error("OnLink callback was not called")
	}
	if !strings.Contains(result, "Example") {
		t.Errorf("Result = %s, expected to contain 'Example'", result)
	}
}

func TestConvertWithVisitor_ImageVisitor(t *testing.T) {
	html := `<img src="image.jpg" alt="Test Image" />`

	imageCalled := false
	visitor := &Visitor{
		OnImage: func(ctx *NodeContext, src, alt, title string) *VisitResult {
			imageCalled = true
			if src != "image.jpg" {
				t.Errorf("OnImage src = %s, expected 'image.jpg'", src)
			}
			if alt != "Test Image" {
				t.Errorf("OnImage alt = %s, expected 'Test Image'", alt)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !imageCalled {
		t.Error("OnImage callback was not called")
	}
	if !strings.Contains(result, "image.jpg") {
		t.Errorf("Result = %s, expected to contain 'image.jpg'", result)
	}
}

func TestConvertWithVisitor_HeadingVisitor(t *testing.T) {
	html := `<h1>Main Title</h1><h2>Subtitle</h2>`

	headingCount := 0
	visitor := &Visitor{
		OnHeading: func(ctx *NodeContext, level uint32, text, id string) *VisitResult {
			headingCount++
			if level < 1 || level > 6 {
				t.Errorf("OnHeading level = %d, expected 1-6", level)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if headingCount < 2 {
		t.Errorf("OnHeading called %d times, expected at least 2", headingCount)
	}
	if !strings.Contains(result, "Main Title") {
		t.Errorf("Result = %s, expected to contain 'Main Title'", result)
	}
}

func TestConvertWithVisitor_CodeVisitor(t *testing.T) {
	html := `<code>inline code</code>`

	codeCalled := false
	visitor := &Visitor{
		OnCodeInline: func(ctx *NodeContext, code string) *VisitResult {
			codeCalled = true
			if !strings.Contains(code, "inline") {
				t.Errorf("OnCodeInline code = %s, expected to contain 'inline'", code)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !codeCalled {
		t.Error("OnCodeInline callback was not called")
	}
	if !strings.Contains(result, "code") {
		t.Errorf("Result = %s, expected to contain 'code'", result)
	}
}

func TestConvertWithVisitor_ListVisitor(t *testing.T) {
	html := `<ul><li>Item 1</li><li>Item 2</li></ul>`

	listStartCalled := false
	listItemCalled := false
	listEndCalled := false

	visitor := &Visitor{
		OnListStart: func(ctx *NodeContext, ordered bool) *VisitResult {
			listStartCalled = true
			if ordered {
				t.Error("OnListStart ordered should be false for <ul>")
			}
			return &VisitResult{ResultType: VisitContinue}
		},
		OnListItem: func(ctx *NodeContext, ordered bool, marker, text string) *VisitResult {
			listItemCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnListEnd: func(ctx *NodeContext, ordered bool, output string) *VisitResult {
			listEndCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !listStartCalled {
		t.Error("OnListStart callback was not called")
	}
	if !listItemCalled {
		t.Error("OnListItem callback was not called")
	}
	if !listEndCalled {
		t.Error("OnListEnd callback was not called")
	}
	if !strings.Contains(result, "Item 1") {
		t.Errorf("Result = %s, expected to contain 'Item 1'", result)
	}
}

func TestConvertWithVisitor_TableVisitor(t *testing.T) {
	html := `
	<table>
		<tr><th>Header 1</th><th>Header 2</th></tr>
		<tr><td>Cell 1</td><td>Cell 2</td></tr>
	</table>
	`

	tableStartCalled := false
	tableRowCalled := false
	tableEndCalled := false

	visitor := &Visitor{
		OnTableStart: func(ctx *NodeContext) *VisitResult {
			tableStartCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnTableRow: func(ctx *NodeContext, cells []string, isHeader bool) *VisitResult {
			tableRowCalled = true
			if len(cells) == 0 {
				t.Error("OnTableRow cells should not be empty")
			}
			return &VisitResult{ResultType: VisitContinue}
		},
		OnTableEnd: func(ctx *NodeContext, output string) *VisitResult {
			tableEndCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !tableStartCalled {
		t.Error("OnTableStart callback was not called")
	}
	if !tableRowCalled {
		t.Error("OnTableRow callback was not called")
	}
	if !tableEndCalled {
		t.Error("OnTableEnd callback was not called")
	}
	if result == "" {
		t.Error("Result should not be empty")
	}
}

func TestConvertWithVisitor_TextVisitor(t *testing.T) {
	html := `<p>Hello World</p>`

	textCalled := false
	visitor := &Visitor{
		OnText: func(ctx *NodeContext, text string) *VisitResult {
			if strings.Contains(text, "Hello") || strings.Contains(text, "World") {
				textCalled = true
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !textCalled {
		t.Error("OnText callback was not called for text content")
	}
	if !strings.Contains(result, "Hello") {
		t.Errorf("Result = %s, expected to contain 'Hello'", result)
	}
}

func TestConvertWithVisitor_ElementVisitors(t *testing.T) {
	html := `<div><p>Content</p></div>`

	startCalled := false
	endCalled := false

	visitor := &Visitor{
		OnElementStart: func(ctx *NodeContext) *VisitResult {
			startCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnElementEnd: func(ctx *NodeContext, output string) *VisitResult {
			endCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !startCalled {
		t.Error("OnElementStart callback was not called")
	}
	if !endCalled {
		t.Error("OnElementEnd callback was not called")
	}
	if !strings.Contains(result, "Content") {
		t.Errorf("Result = %s, expected to contain 'Content'", result)
	}
}

func TestConvertWithVisitor_FormattingVisitors(t *testing.T) {
	tests := []struct {
		name      string
		html      string
		callback  func(*Visitor)
		validator func(string) bool
	}{
		{
			name: "strong",
			html: `<strong>Bold</strong>`,
			callback: func(v *Visitor) {
				v.OnStrong = func(ctx *NodeContext, text string) *VisitResult {
					if !strings.Contains(text, "Bold") {
						t.Errorf("OnStrong text = %s, expected to contain 'Bold'", text)
					}
					return &VisitResult{ResultType: VisitContinue}
				}
			},
			validator: func(result string) bool {
				return strings.Contains(result, "Bold")
			},
		},
		{
			name: "emphasis",
			html: `<em>Italic</em>`,
			callback: func(v *Visitor) {
				v.OnEmphasis = func(ctx *NodeContext, text string) *VisitResult {
					if !strings.Contains(text, "Italic") {
						t.Errorf("OnEmphasis text = %s, expected to contain 'Italic'", text)
					}
					return &VisitResult{ResultType: VisitContinue}
				}
			},
			validator: func(result string) bool {
				return strings.Contains(result, "Italic")
			},
		},
		{
			name: "mark",
			html: `<mark>Highlighted</mark>`,
			callback: func(v *Visitor) {
				v.OnMark = func(ctx *NodeContext, text string) *VisitResult {
					if !strings.Contains(text, "Highlighted") {
						t.Errorf("OnMark text = %s, expected to contain 'Highlighted'", text)
					}
					return &VisitResult{ResultType: VisitContinue}
				}
			},
			validator: func(result string) bool {
				return strings.Contains(result, "Highlighted")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			visitor := &Visitor{}
			tt.callback(visitor)

			result, err := ConvertWithVisitor(tt.html, visitor)
			if err != nil {
				t.Errorf("ConvertWithVisitor failed: %v", err)
			}
			if !tt.validator(result) {
				t.Errorf("Result validation failed for %s: %s", tt.name, result)
			}
		})
	}
}

func TestConvertWithVisitor_BlockquoteVisitor(t *testing.T) {
	html := `<blockquote>Quote text</blockquote>`

	blockquoteCalled := false
	visitor := &Visitor{
		OnBlockquote: func(ctx *NodeContext, content string, depth uint64) *VisitResult {
			blockquoteCalled = true
			if !strings.Contains(content, "Quote") {
				t.Errorf("OnBlockquote content = %s, expected to contain 'Quote'", content)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !blockquoteCalled {
		t.Error("OnBlockquote callback was not called")
	}
	if !strings.Contains(result, "Quote") {
		t.Errorf("Result = %s, expected to contain 'Quote'", result)
	}
}

func TestConvertWithVisitor_HRVisitor(t *testing.T) {
	html := `<p>Before</p><hr/><p>After</p>`

	hrCalled := false
	visitor := &Visitor{
		OnHorizontalRule: func(ctx *NodeContext) *VisitResult {
			hrCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !hrCalled {
		t.Error("OnHorizontalRule callback was not called")
	}
	if !strings.Contains(result, "Before") || !strings.Contains(result, "After") {
		t.Errorf("Result = %s, expected to contain 'Before' and 'After'", result)
	}
}

func TestMustConvertWithVisitor_Success(t *testing.T) {
	html := "<h1>Test</h1>"
	visitor := &Visitor{}

	result := MustConvertWithVisitor(html, visitor)
	if !strings.Contains(result, "Test") {
		t.Errorf("MustConvertWithVisitor returned %s, expected to contain 'Test'", result)
	}
}

func TestMustConvertWithVisitor_Panic(t *testing.T) {
	t.Skip("Skipping panic test for MustConvertWithVisitor")
}

func TestConvertWithVisitor_DefinitionList(t *testing.T) {
	html := `<dl><dt>Term</dt><dd>Definition</dd></dl>`

	dlStartCalled := false
	dtCalled := false
	ddCalled := false
	dlEndCalled := false

	visitor := &Visitor{
		OnDefinitionListStart: func(ctx *NodeContext) *VisitResult {
			dlStartCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnDefinitionTerm: func(ctx *NodeContext, text string) *VisitResult {
			dtCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnDefinitionDescription: func(ctx *NodeContext, text string) *VisitResult {
			ddCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnDefinitionListEnd: func(ctx *NodeContext, output string) *VisitResult {
			dlEndCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !dlStartCalled {
		t.Error("OnDefinitionListStart callback was not called")
	}
	if !dtCalled {
		t.Error("OnDefinitionTerm callback was not called")
	}
	if !ddCalled {
		t.Error("OnDefinitionDescription callback was not called")
	}
	if !dlEndCalled {
		t.Error("OnDefinitionListEnd callback was not called")
	}
	if result == "" {
		t.Error("Result should not be empty")
	}
}

func TestConvertWithVisitor_MultipleCbcallbacks(t *testing.T) {
	html := `<div><h1>Title</h1><p>Text with <strong>bold</strong> and <em>italic</em>.</p></div>`

	callCount := map[string]int{}

	visitor := &Visitor{
		OnElementStart: func(ctx *NodeContext) *VisitResult {
			callCount["elementStart"]++
			return &VisitResult{ResultType: VisitContinue}
		},
		OnElementEnd: func(ctx *NodeContext, output string) *VisitResult {
			callCount["elementEnd"]++
			return &VisitResult{ResultType: VisitContinue}
		},
		OnHeading: func(ctx *NodeContext, level uint32, text, id string) *VisitResult {
			callCount["heading"]++
			return &VisitResult{ResultType: VisitContinue}
		},
		OnStrong: func(ctx *NodeContext, text string) *VisitResult {
			callCount["strong"]++
			return &VisitResult{ResultType: VisitContinue}
		},
		OnEmphasis: func(ctx *NodeContext, text string) *VisitResult {
			callCount["emphasis"]++
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}

	if callCount["elementStart"] < 2 {
		t.Errorf("OnElementStart called %d times, expected at least 2", callCount["elementStart"])
	}
	if callCount["elementEnd"] < 2 {
		t.Errorf("OnElementEnd called %d times, expected at least 2", callCount["elementEnd"])
	}
	if callCount["heading"] < 1 {
		t.Errorf("OnHeading called %d times, expected at least 1", callCount["heading"])
	}
	if callCount["strong"] < 1 {
		t.Errorf("OnStrong called %d times, expected at least 1", callCount["strong"])
	}
	if callCount["emphasis"] < 1 {
		t.Errorf("OnEmphasis called %d times, expected at least 1", callCount["emphasis"])
	}

	if !strings.Contains(result, "Title") || !strings.Contains(result, "Text") {
		t.Errorf("Result = %s, expected to contain 'Title' and 'Text'", result)
	}
}

func TestConvertWithVisitor_FigureVisitor(t *testing.T) {
	html := `<figure><img src="image.jpg" /><figcaption>Figure caption</figcaption></figure>`

	figureStartCalled := false
	figcaptionCalled := false
	figureEndCalled := false

	visitor := &Visitor{
		OnFigureStart: func(ctx *NodeContext) *VisitResult {
			figureStartCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
		OnFigcaption: func(ctx *NodeContext, text string) *VisitResult {
			figcaptionCalled = true
			if !strings.Contains(text, "caption") {
				t.Errorf("OnFigcaption text = %s, expected to contain 'caption'", text)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
		OnFigureEnd: func(ctx *NodeContext, output string) *VisitResult {
			figureEndCalled = true
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !figureStartCalled {
		t.Error("OnFigureStart callback was not called")
	}
	if !figcaptionCalled {
		t.Error("OnFigcaption callback was not called")
	}
	if !figureEndCalled {
		t.Error("OnFigureEnd callback was not called")
	}
	if result == "" {
		t.Error("Result should not be empty")
	}
}

func TestConvertWithVisitor_DetailsVisitor(t *testing.T) {
	html := `<details open><summary>Click me</summary>Hidden content</details>`

	detailsCalled := false
	summaryCalled := false

	visitor := &Visitor{
		OnDetails: func(ctx *NodeContext, open bool) *VisitResult {
			detailsCalled = true
			if !open {
				t.Error("OnDetails open should be true")
			}
			return &VisitResult{ResultType: VisitContinue}
		},
		OnSummary: func(ctx *NodeContext, text string) *VisitResult {
			summaryCalled = true
			if !strings.Contains(text, "Click") {
				t.Errorf("OnSummary text = %s, expected to contain 'Click'", text)
			}
			return &VisitResult{ResultType: VisitContinue}
		},
	}

	result, err := ConvertWithVisitor(html, visitor)
	if err != nil {
		t.Errorf("ConvertWithVisitor failed: %v", err)
	}
	if !detailsCalled {
		t.Error("OnDetails callback was not called")
	}
	if !summaryCalled {
		t.Error("OnSummary callback was not called")
	}
	if result == "" {
		t.Error("Result should not be empty")
	}
}
