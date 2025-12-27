using System.Runtime.InteropServices;

namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Native C FFI structures for visitor pattern interop.
/// These structures mirror the C FFI definitions in the Rust core.
/// </summary>
internal static class NativeVisitorStructures
{
    // Native callback function delegates
    public delegate NativeVisitResult VisitElementStartCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitElementEndCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr output);

    public delegate NativeVisitResult VisitTextCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitLinkCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr href,
        IntPtr text,
        IntPtr title);

    public delegate NativeVisitResult VisitImageCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr src,
        IntPtr alt,
        IntPtr title);

    public delegate NativeVisitResult VisitHeadingCallback(
        IntPtr userData,
        IntPtr ctx,
        uint level,
        IntPtr text,
        IntPtr id);

    public delegate NativeVisitResult VisitCodeBlockCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr lang,
        IntPtr code);

    public delegate NativeVisitResult VisitCodeInlineCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr code);

    public delegate NativeVisitResult VisitListItemCallback(
        IntPtr userData,
        IntPtr ctx,
        bool ordered,
        IntPtr marker,
        IntPtr text);

    public delegate NativeVisitResult VisitListStartCallback(
        IntPtr userData,
        IntPtr ctx,
        bool ordered);

    public delegate NativeVisitResult VisitListEndCallback(
        IntPtr userData,
        IntPtr ctx,
        bool ordered,
        IntPtr output);

    public delegate NativeVisitResult VisitTableStartCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitTableRowCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr cells,
        nuint cellCount,
        bool isHeader);

    public delegate NativeVisitResult VisitTableEndCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr output);

    public delegate NativeVisitResult VisitBlockquoteCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr content,
        nuint depth);

    public delegate NativeVisitResult VisitStrongCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitEmphasisCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitStrikethroughCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitUnderlineCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitSubscriptCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitSuperscriptCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitMarkCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitLineBreakCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitHorizontalRuleCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitCustomElementCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr tagName,
        IntPtr html);

    public delegate NativeVisitResult VisitDefinitionListStartCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitDefinitionTermCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitDefinitionDescriptionCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitDefinitionListEndCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr output);

    public delegate NativeVisitResult VisitFormCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr action,
        IntPtr method);

    public delegate NativeVisitResult VisitInputCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr inputType,
        IntPtr name,
        IntPtr value);

    public delegate NativeVisitResult VisitButtonCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitAudioCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr src);

    public delegate NativeVisitResult VisitVideoCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr src);

    public delegate NativeVisitResult VisitIFrameCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr src);

    public delegate NativeVisitResult VisitDetailsCallback(
        IntPtr userData,
        IntPtr ctx,
        bool open);

    public delegate NativeVisitResult VisitSummaryCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitFigureStartCallback(
        IntPtr userData,
        IntPtr ctx);

    public delegate NativeVisitResult VisitFigCaptionCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr text);

    public delegate NativeVisitResult VisitFigureEndCallback(
        IntPtr userData,
        IntPtr ctx,
        IntPtr output);

    // Native visit result enum
    [Serializable]
    public enum NativeVisitResultType
    {
        Continue = 0,
        Custom = 1,
        Skip = 2,
        PreserveHtml = 3,
        Error = 4
    }

    // Native visit result struct
    [StructLayout(LayoutKind.Sequential)]
    public struct NativeVisitResult
    {
        public NativeVisitResultType ResultType;
        public IntPtr CustomOutput;
        public IntPtr ErrorMessage;
    }

    // Native attribute struct
    [StructLayout(LayoutKind.Sequential)]
    public struct NativeAttribute
    {
        public IntPtr Key;
        public IntPtr Value;
    }

    // Native node type enum
    public enum NativeNodeType
    {
        Text = 0,
        Element = 1,
        Heading = 2,
        Paragraph = 3,
        Div = 4,
        Blockquote = 5,
        Pre = 6,
        Hr = 7,
        List = 8,
        ListItem = 9,
        DefinitionList = 10,
        DefinitionTerm = 11,
        DefinitionDescription = 12,
        Table = 13,
        TableRow = 14,
        TableCell = 15,
        TableHeader = 16,
        TableBody = 17,
        TableHead = 18,
        TableFoot = 19,
        Link = 20,
        Image = 21,
        Strong = 22,
        Em = 23,
        Code = 24,
        Strikethrough = 25,
        Underline = 26,
        Subscript = 27,
        Superscript = 28,
        Mark = 29,
        Small = 30,
        Br = 31,
        Span = 32,
        Article = 33,
        Section = 34,
        Nav = 35,
        Aside = 36,
        Header = 37,
        Footer = 38,
        Main = 39,
        Figure = 40,
        FigCaption = 41,
        Time = 42,
        Details = 43,
        Summary = 44,
        Form = 45,
        Input = 46,
        Select = 47,
        Option = 48,
        Button = 49,
        TextArea = 50,
        Label = 51,
        FieldSet = 52,
        Legend = 53,
        Audio = 54,
        Video = 55,
        Picture = 56,
        Source = 57,
        IFrame = 58,
        Svg = 59,
        Canvas = 60,
        Ruby = 61,
        Rt = 62,
        Rp = 63,
        Abbr = 64,
        Kbd = 65,
        Samp = 66,
        Var = 67,
        Cite = 68,
        Quote = 69,
        Del = 70,
        Ins = 71,
        Data = 72,
        Meter = 73,
        Progress = 74,
        Output = 75,
        Template = 76,
        Slot = 77,
        Html = 78,
        Head = 79,
        Body = 80,
        Title = 81,
        Meta = 82,
        LinkTag = 83,
        Style = 84,
        Script = 85,
        Base = 86,
        Custom = 87
    }

    // Native node context struct
    [StructLayout(LayoutKind.Sequential)]
    public struct NativeNodeContext
    {
        public NativeNodeType NodeType;
        public IntPtr TagName;
        public IntPtr Attributes;
        public nuint Depth;
        public nuint IndexInParent;
        public IntPtr ParentTag;
        [MarshalAs(UnmanagedType.U1)]
        public bool IsInline;
    }

    // Native visitor struct with all callback pointers
    [StructLayout(LayoutKind.Sequential)]
    public struct NativeVisitor
    {
        public IntPtr UserData;
        public IntPtr VisitElementStart;
        public IntPtr VisitElementEnd;
        public IntPtr VisitText;
        public IntPtr VisitLink;
        public IntPtr VisitImage;
        public IntPtr VisitHeading;
        public IntPtr VisitCodeBlock;
        public IntPtr VisitCodeInline;
        public IntPtr VisitListItem;
        public IntPtr VisitListStart;
        public IntPtr VisitListEnd;
        public IntPtr VisitTableStart;
        public IntPtr VisitTableRow;
        public IntPtr VisitTableEnd;
        public IntPtr VisitBlockquote;
        public IntPtr VisitStrong;
        public IntPtr VisitEmphasis;
        public IntPtr VisitStrikethrough;
        public IntPtr VisitUnderline;
        public IntPtr VisitSubscript;
        public IntPtr VisitSuperscript;
        public IntPtr VisitMark;
        public IntPtr VisitLineBreak;
        public IntPtr VisitHorizontalRule;
        public IntPtr VisitCustomElement;
        public IntPtr VisitDefinitionListStart;
        public IntPtr VisitDefinitionTerm;
        public IntPtr VisitDefinitionDescription;
        public IntPtr VisitDefinitionListEnd;
        public IntPtr VisitForm;
        public IntPtr VisitInput;
        public IntPtr VisitButton;
        public IntPtr VisitAudio;
        public IntPtr VisitVideo;
        public IntPtr VisitIFrame;
        public IntPtr VisitDetails;
        public IntPtr VisitSummary;
        public IntPtr VisitFigureStart;
        public IntPtr VisitFigCaption;
        public IntPtr VisitFigureEnd;
    }
}
