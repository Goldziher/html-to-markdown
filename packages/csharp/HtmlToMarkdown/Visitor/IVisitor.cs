namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Interface for custom HTML element visitors.
/// Implement this interface to customize how specific HTML elements are converted to Markdown.
/// All methods return a VisitResult indicating how the converter should proceed.
/// </summary>
public interface IVisitor
{
    // === Generic Hooks ===

    /// <summary>
    /// Called before entering any HTML element.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitElementStart(NodeContext context)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called after exiting any HTML element with the default markdown output.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="output">The default markdown output</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitElementEnd(NodeContext context, string output)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for text nodes (most frequent callback - 100+ per document).
    /// </summary>
    /// <param name="context">Context information about the text node</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitText(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for anchor links &lt;a href="..."&gt;.
    /// </summary>
    /// <param name="context">Context information about the link</param>
    /// <param name="href">The link URL</param>
    /// <param name="text">The link text (already converted to markdown)</param>
    /// <param name="title">The title attribute, or null if not present</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for image elements &lt;img src="..."&gt;.
    /// </summary>
    /// <param name="context">Context information about the image</param>
    /// <param name="src">The image source URL</param>
    /// <param name="alt">The alt text</param>
    /// <param name="title">The title attribute, or null if not present</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitImage(NodeContext context, string src, string alt, string? title)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for heading elements &lt;h1&gt; through &lt;h6&gt;.
    /// </summary>
    /// <param name="context">Context information about the heading</param>
    /// <param name="level">The heading level (1-6)</param>
    /// <param name="text">The heading text content</param>
    /// <param name="id">The ID attribute, or null if not present</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for code blocks &lt;pre&gt;&lt;code&gt;.
    /// </summary>
    /// <param name="context">Context information about the code block</param>
    /// <param name="lang">The optional language specifier, or null</param>
    /// <param name="code">The code content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitCodeBlock(NodeContext context, string? lang, string code)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for inline code &lt;code&gt;.
    /// </summary>
    /// <param name="context">Context information about the code</param>
    /// <param name="code">The code content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitCodeInline(NodeContext context, string code)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for list items &lt;li&gt;.
    /// </summary>
    /// <param name="context">Context information about the list item</param>
    /// <param name="ordered">Whether the parent list is ordered</param>
    /// <param name="marker">The list marker (e.g., "-", "1.")</param>
    /// <param name="text">The list item text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitListItem(NodeContext context, bool ordered, string marker, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called before processing a list &lt;ul&gt; or &lt;ol&gt;.
    /// </summary>
    /// <param name="context">Context information about the list</param>
    /// <param name="ordered">Whether this is an ordered list</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitListStart(NodeContext context, bool ordered)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called after processing a list &lt;/ul&gt; or &lt;/ol&gt;.
    /// </summary>
    /// <param name="context">Context information about the list</param>
    /// <param name="ordered">Whether this is an ordered list</param>
    /// <param name="output">The default markdown output for the list</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitListEnd(NodeContext context, bool ordered, string output)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called before processing a table &lt;table&gt;.
    /// </summary>
    /// <param name="context">Context information about the table</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitTableStart(NodeContext context)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for table rows &lt;tr&gt;.
    /// </summary>
    /// <param name="context">Context information about the row</param>
    /// <param name="cells">The cell contents as an array of strings</param>
    /// <param name="isHeader">Whether this is a header row</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitTableRow(NodeContext context, IReadOnlyList<string> cells, bool isHeader)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called after processing a table &lt;/table&gt;.
    /// </summary>
    /// <param name="context">Context information about the table</param>
    /// <param name="output">The default markdown output for the table</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitTableEnd(NodeContext context, string output)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for blockquote elements &lt;blockquote&gt;.
    /// </summary>
    /// <param name="context">Context information about the blockquote</param>
    /// <param name="content">The blockquote content</param>
    /// <param name="depth">The nesting depth of blockquotes</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitBlockquote(NodeContext context, string content, int depth)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for strong/bold elements &lt;strong&gt;, &lt;b&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitStrong(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for emphasis/italic elements &lt;em&gt;, &lt;i&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitEmphasis(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for strikethrough elements &lt;s&gt;, &lt;del&gt;, &lt;strike&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitStrikethrough(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for underline elements &lt;u&gt;, &lt;ins&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitUnderline(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for subscript elements &lt;sub&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitSubscript(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for superscript elements &lt;sup&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitSuperscript(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for mark/highlight elements &lt;mark&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The text content</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitMark(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for line break elements &lt;br&gt;.
    /// </summary>
    /// <param name="context">Context information about the line break</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitLineBreak(NodeContext context)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for horizontal rule elements &lt;hr&gt;.
    /// </summary>
    /// <param name="context">Context information about the rule</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitHorizontalRule(NodeContext context)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for custom elements (web components) or unknown tags.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="tagName">The tag name</param>
    /// <param name="html">The original HTML</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitCustomElement(NodeContext context, string tagName, string html)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called before processing a definition list &lt;dl&gt;.
    /// </summary>
    /// <param name="context">Context information about the list</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitDefinitionListStart(NodeContext context)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for definition terms &lt;dt&gt;.
    /// </summary>
    /// <param name="context">Context information about the term</param>
    /// <param name="text">The term text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitDefinitionTerm(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for definition descriptions &lt;dd&gt;.
    /// </summary>
    /// <param name="context">Context information about the description</param>
    /// <param name="text">The description text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitDefinitionDescription(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called after processing a definition list &lt;/dl&gt;.
    /// </summary>
    /// <param name="context">Context information about the list</param>
    /// <param name="output">The default markdown output for the list</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitDefinitionListEnd(NodeContext context, string output)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for form elements &lt;form&gt;.
    /// </summary>
    /// <param name="context">Context information about the form</param>
    /// <param name="action">The form action URL, or null</param>
    /// <param name="method">The form method, or null</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitForm(NodeContext context, string? action, string? method)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for input elements &lt;input&gt;.
    /// </summary>
    /// <param name="context">Context information about the input</param>
    /// <param name="inputType">The input type</param>
    /// <param name="name">The input name, or null</param>
    /// <param name="value">The input value, or null</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitInput(NodeContext context, string inputType, string? name, string? value)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for button elements &lt;button&gt;.
    /// </summary>
    /// <param name="context">Context information about the button</param>
    /// <param name="text">The button text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitButton(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for audio elements &lt;audio&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="src">The source URL, or null</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitAudio(NodeContext context, string? src)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for video elements &lt;video&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="src">The source URL, or null</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitVideo(NodeContext context, string? src)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for iframe elements &lt;iframe&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="src">The source URL, or null</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitIFrame(NodeContext context, string? src)
    {
        return VisitResult.Continue();
    }


    /// <summary>
    /// Called for details elements &lt;details&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="open">Whether the details element is open</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitDetails(NodeContext context, bool open)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for summary elements &lt;summary&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The summary text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitSummary(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called before processing a figure element &lt;figure&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitFigureStart(NodeContext context)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called for figcaption elements &lt;figcaption&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="text">The caption text</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitFigCaption(NodeContext context, string text)
    {
        return VisitResult.Continue();
    }

    /// <summary>
    /// Called after processing a figure element &lt;/figure&gt;.
    /// </summary>
    /// <param name="context">Context information about the element</param>
    /// <param name="output">The default markdown output for the figure</param>
    /// <returns>A VisitResult indicating how to proceed</returns>
    public VisitResult VisitFigureEnd(NodeContext context, string output)
    {
        return VisitResult.Continue();
    }
}
