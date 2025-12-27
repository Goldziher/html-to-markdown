namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Base class for visit result types returned from visitor callbacks.
/// Use factory methods to create instances.
/// </summary>
public abstract class VisitResult
{
    /// <summary>
    /// Gets the result type indicating what action the converter should take.
    /// </summary>
    public abstract VisitResultType ResultType { get; }

    /// <summary>
    /// Creates a result that continues with default conversion behavior.
    /// </summary>
    /// <returns>A Continue result</returns>
    public static ContinueResult Continue() => new();

    /// <summary>
    /// Creates a result that replaces the default output with custom markdown.
    /// </summary>
    /// <param name="customOutput">The custom markdown output to use</param>
    /// <returns>A Custom result</returns>
    /// <exception cref="ArgumentNullException">Thrown when customOutput is null</exception>
    public static CustomResult Custom(string customOutput)
    {
        if (customOutput == null)
        {
            throw new ArgumentNullException(nameof(customOutput));
        }

        return new CustomResult(customOutput);
    }

    /// <summary>
    /// Creates a result that skips this element and all children.
    /// </summary>
    /// <returns>A Skip result</returns>
    public static SkipResult Skip() => new();

    /// <summary>
    /// Creates a result that preserves the original HTML instead of converting.
    /// </summary>
    /// <returns>A PreserveHtml result</returns>
    public static PreserveHtmlResult PreserveHtml() => new();

    /// <summary>
    /// Creates a result that halts conversion with an error.
    /// </summary>
    /// <param name="errorMessage">The error message</param>
    /// <returns>An Error result</returns>
    /// <exception cref="ArgumentNullException">Thrown when errorMessage is null</exception>
    public static ErrorResult Error(string errorMessage)
    {
        if (errorMessage == null)
        {
            throw new ArgumentNullException(nameof(errorMessage));
        }

        return new ErrorResult(errorMessage);
    }
}

/// <summary>
/// Result indicating default conversion behavior should continue.
/// </summary>
public sealed class ContinueResult : VisitResult
{
    public override VisitResultType ResultType => VisitResultType.Continue;
}

/// <summary>
/// Result providing custom markdown output to replace the default.
/// </summary>
public sealed class CustomResult : VisitResult
{
    /// <summary>
    /// Gets the custom markdown output.
    /// </summary>
    public string CustomOutput { get; }

    public override VisitResultType ResultType => VisitResultType.Custom;

    internal CustomResult(string customOutput)
    {
        CustomOutput = customOutput ?? throw new ArgumentNullException(nameof(customOutput));
    }
}

/// <summary>
/// Result indicating the element and all children should be skipped.
/// </summary>
public sealed class SkipResult : VisitResult
{
    public override VisitResultType ResultType => VisitResultType.Skip;
}

/// <summary>
/// Result indicating the original HTML should be preserved instead of converting.
/// </summary>
public sealed class PreserveHtmlResult : VisitResult
{
    public override VisitResultType ResultType => VisitResultType.PreserveHtml;
}

/// <summary>
/// Result indicating an error occurred and conversion should stop.
/// </summary>
public sealed class ErrorResult : VisitResult
{
    /// <summary>
    /// Gets the error message.
    /// </summary>
    public string ErrorMessage { get; }

    public override VisitResultType ResultType => VisitResultType.Error;

    internal ErrorResult(string errorMessage)
    {
        ErrorMessage = errorMessage ?? throw new ArgumentNullException(nameof(errorMessage));
    }
}
