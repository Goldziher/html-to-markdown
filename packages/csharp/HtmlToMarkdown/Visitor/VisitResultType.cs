namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Result type from a visitor callback indicating how the converter should proceed.
/// </summary>
public enum VisitResultType
{
    /// <summary>
    /// Continue with default conversion behavior for this node.
    /// </summary>
    Continue = 0,

    /// <summary>
    /// Replace output with custom markdown from the callback.
    /// </summary>
    Custom = 1,

    /// <summary>
    /// Skip this element and all children entirely.
    /// </summary>
    Skip = 2,

    /// <summary>
    /// Preserve original HTML instead of converting to markdown.
    /// </summary>
    PreserveHtml = 3,

    /// <summary>
    /// Stop conversion and report error.
    /// </summary>
    Error = 4
}
