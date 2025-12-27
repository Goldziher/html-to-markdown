package io.github.goldziher.htmltomarkdown.visitor;

/**
 * HTML node type enumeration covering all element types supported by the
 * visitor pattern.
 *
 * <p>Maps directly to the C FFI NodeType enum. This enum categorizes HTML
 * elements for coarse-grained visitor dispatch.
 *
 * @since 2.17.0
 */
public enum NodeType {
    /** Text node. */
    TEXT,

    /** Generic element node. */
    ELEMENT,

    /** Heading element (h1-h6). */
    HEADING,

    /** Paragraph element. */
    PARAGRAPH,

    /** Division element. */
    DIV,

    /** Blockquote element. */
    BLOCKQUOTE,

    /** Preformatted text element. */
    PRE,

    /** Horizontal rule element. */
    HR,

    /** List element (ul, ol, dl). */
    LIST,

    /** List item element. */
    LIST_ITEM,

    /** Definition list element. */
    DEFINITION_LIST,

    /** Definition term element. */
    DEFINITION_TERM,

    /** Definition description element. */
    DEFINITION_DESCRIPTION,

    /** Table element. */
    TABLE,

    /** Table row element. */
    TABLE_ROW,

    /** Table cell element. */
    TABLE_CELL,

    /** Table header cell element. */
    TABLE_HEADER,

    /** Table body element. */
    TABLE_BODY,

    /** Table head element. */
    TABLE_HEAD,

    /** Table foot element. */
    TABLE_FOOT,

    /** Link element. */
    LINK,

    /** Image element. */
    IMAGE,

    /** Strong/bold element. */
    STRONG,

    /** Emphasis/italic element. */
    EM,

    /** Code element. */
    CODE,

    /** Strikethrough element. */
    STRIKETHROUGH,

    /** Underline element. */
    UNDERLINE,

    /** Subscript element. */
    SUBSCRIPT,

    /** Superscript element. */
    SUPERSCRIPT,

    /** Mark element. */
    MARK,

    /** Small element. */
    SMALL,

    /** Line break element. */
    BR,

    /** Span element. */
    SPAN,

    /** Article element. */
    ARTICLE,

    /** Section element. */
    SECTION,

    /** Navigation element. */
    NAV,

    /** Aside element. */
    ASIDE,

    /** Header element. */
    HEADER,

    /** Footer element. */
    FOOTER,

    /** Main element. */
    MAIN,

    /** Figure element. */
    FIGURE,

    /** Figure caption element. */
    FIGCAPTION,

    /** Time element. */
    TIME,

    /** Details element. */
    DETAILS,

    /** Summary element. */
    SUMMARY,

    /** Form element. */
    FORM,

    /** Input element. */
    INPUT,

    /** Select element. */
    SELECT,

    /** Option element. */
    OPTION,

    /** Button element. */
    BUTTON,

    /** Textarea element. */
    TEXTAREA,

    /** Label element. */
    LABEL,

    /** Fieldset element. */
    FIELDSET,

    /** Legend element. */
    LEGEND,

    /** Audio element. */
    AUDIO,

    /** Video element. */
    VIDEO,

    /** Picture element. */
    PICTURE,

    /** Source element. */
    SOURCE,

    /** Iframe element. */
    IFRAME,

    /** SVG element. */
    SVG,

    /** Canvas element. */
    CANVAS,

    /** Ruby element. */
    RUBY,

    /** Ruby text element. */
    RT,

    /** Ruby parenthesis element. */
    RP,

    /** Abbreviation element. */
    ABBR,

    /** Keyboard element. */
    KBD,

    /** Sample element. */
    SAMP,

    /** Variable element. */
    VAR,

    /** Citation element. */
    CITE,

    /** Quote element. */
    Q,

    /** Deleted element. */
    DEL,

    /** Inserted element. */
    INS,

    /** Data element. */
    DATA,

    /** Meter element. */
    METER,

    /** Progress element. */
    PROGRESS,

    /** Output element. */
    OUTPUT,

    /** Template element. */
    TEMPLATE,

    /** Slot element. */
    SLOT,

    /** HTML root element. */
    HTML,

    /** Head element. */
    HEAD,

    /** Body element. */
    BODY,

    /** Title element. */
    TITLE,

    /** Meta element. */
    META,

    /** Link tag element. */
    LINK_TAG,

    /** Style element. */
    STYLE,

    /** Script element. */
    SCRIPT,

    /** Base element. */
    BASE,

    /** Custom/unknown element. */
    CUSTOM;

    /**
     * Convert from C FFI node type value to Java enum.
     *
     * @param cValue the C FFI enum value
     * @return the corresponding Java NodeType
     * @throws IllegalArgumentException if the value is not recognized
     */
    public static NodeType fromCValue(final int cValue) {
        return values()[cValue];
    }

    /**
     * Convert to C FFI node type value.
     *
     * @return the C FFI enum value
     */
    public int toCValue() {
        return ordinal();
    }
}
