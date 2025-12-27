namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Enumeration of all HTML node types recognized by the converter.
/// Used to classify elements during visitor callbacks.
/// </summary>
public enum NodeType
{
    // Text and Generic
    Text = 0,
    Element = 1,

    // Block Elements
    Heading = 2,
    Paragraph = 3,
    Div = 4,
    Blockquote = 5,
    Pre = 6,
    HorizontalRule = 7,

    // Lists
    List = 8,
    ListItem = 9,
    DefinitionList = 10,
    DefinitionTerm = 11,
    DefinitionDescription = 12,

    // Tables
    Table = 13,
    TableRow = 14,
    TableCell = 15,
    TableHeader = 16,
    TableBody = 17,
    TableHead = 18,
    TableFoot = 19,

    // Inline Formatting
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
    LineBreak = 31,
    Span = 32,

    // Semantic HTML5
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

    // Forms
    Form = 45,
    Input = 46,
    Select = 47,
    Option = 48,
    Button = 49,
    TextArea = 50,
    Label = 51,
    FieldSet = 52,
    Legend = 53,

    // Media
    Audio = 54,
    Video = 55,
    Picture = 56,
    Source = 57,
    IFrame = 58,
    Svg = 59,
    Canvas = 60,

    // Advanced/Semantic
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

    // Document Structure
    Html = 78,
    Head = 79,
    Body = 80,
    Title = 81,
    Meta = 82,
    LinkTag = 83,
    Style = 84,
    Script = 85,
    Base = 86,

    // Custom/Unknown
    Custom = 87
}
