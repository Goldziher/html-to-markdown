// swift-format-ignore-file
import RustBridgeC

public func headerMetadataIsValidFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> RustString {
    try { let val = __swift_bridge__$header_metadata_is_valid_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return RustString(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func nodeContextAttributesFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> RustString {
    try { let val = __swift_bridge__$node_context_attributes_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return RustString(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func nodeContextIntoOwnedFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> RustString {
    try { let val = __swift_bridge__$node_context_into_owned_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return RustString(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func visitor_handle_noop(_ client: VisitorHandleRef) {
    __swift_bridge__$visitor_handle_noop(client.ptr)
}
public func convert<GenericIntoRustString: IntoRustString>(_ html: GenericIntoRustString, _ options: Optional<ConversionOptions>) throws -> ConversionResult {
    try { let val = __swift_bridge__$convert({ let rustString = html.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { if let val = options { val.isOwned = false; return val.ptr } else { return nil } }()); if val.is_ok { return ConversionResult(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func alef_phantom_vec_html_visitor() -> RustVec<HtmlVisitorBox> {
    RustVec(ptr: __swift_bridge__$alef_phantom_vec_html_visitor())
}
@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_text")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_text (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_text(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_element_start")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_element_start (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_element_start(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_element_end")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_element_end (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ output: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_element_end(ctx: RustString(ptr: ctx), output: RustString(ptr: output)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_link")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_link (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ href: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer, _ title: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_link(ctx: RustString(ptr: ctx), href: RustString(ptr: href), text: RustString(ptr: text), title: { let val = title; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_image")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_image (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ src: UnsafeMutableRawPointer, _ alt: UnsafeMutableRawPointer, _ title: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_image(ctx: RustString(ptr: ctx), src: RustString(ptr: src), alt: RustString(ptr: alt), title: { let val = title; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_heading")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_heading (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ level: UInt32, _ text: UnsafeMutableRawPointer, _ id: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_heading(ctx: RustString(ptr: ctx), level: level, text: RustString(ptr: text), id: { let val = id; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_code_block")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_code_block (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ lang: UnsafeMutableRawPointer?, _ code: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_code_block(ctx: RustString(ptr: ctx), lang: { let val = lang; if val != nil { return RustString(ptr: val!) } else { return nil } }(), code: RustString(ptr: code)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_code_inline")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_code_inline (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ code: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_code_inline(ctx: RustString(ptr: ctx), code: RustString(ptr: code)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_list_item")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_list_item (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ ordered: Bool, _ marker: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_list_item(ctx: RustString(ptr: ctx), ordered: ordered, marker: RustString(ptr: marker), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_list_start")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_list_start (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ ordered: Bool) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_list_start(ctx: RustString(ptr: ctx), ordered: ordered).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_list_end")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_list_end (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ ordered: Bool, _ output: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_list_end(ctx: RustString(ptr: ctx), ordered: ordered, output: RustString(ptr: output)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_table_start")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_table_start (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_table_start(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_table_row")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_table_row (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ cells: UnsafeMutableRawPointer, _ is_header: Bool) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_table_row(ctx: RustString(ptr: ctx), cells: RustVec(ptr: cells), is_header: is_header).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_table_end")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_table_end (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ output: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_table_end(ctx: RustString(ptr: ctx), output: RustString(ptr: output)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_blockquote")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_blockquote (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ content: UnsafeMutableRawPointer, _ depth: UInt) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_blockquote(ctx: RustString(ptr: ctx), content: RustString(ptr: content), depth: depth).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_strong")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_strong (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_strong(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_emphasis")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_emphasis (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_emphasis(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_strikethrough")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_strikethrough (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_strikethrough(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_underline")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_underline (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_underline(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_subscript")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_subscript (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_subscript(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_superscript")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_superscript (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_superscript(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_mark")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_mark (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_mark(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_line_break")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_line_break (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_line_break(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_horizontal_rule")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_horizontal_rule (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_horizontal_rule(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_custom_element")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_custom_element (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ tag_name: UnsafeMutableRawPointer, _ html: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_custom_element(ctx: RustString(ptr: ctx), tag_name: RustString(ptr: tag_name), html: RustString(ptr: html)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_definition_list_start")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_definition_list_start (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_definition_list_start(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_definition_term")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_definition_term (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_definition_term(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_definition_description")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_definition_description (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_definition_description(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_definition_list_end")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_definition_list_end (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ output: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_definition_list_end(ctx: RustString(ptr: ctx), output: RustString(ptr: output)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_form")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_form (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ action: UnsafeMutableRawPointer?, _ method: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_form(ctx: RustString(ptr: ctx), action: { let val = action; if val != nil { return RustString(ptr: val!) } else { return nil } }(), method: { let val = method; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_input")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_input (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ input_type: UnsafeMutableRawPointer, _ name: UnsafeMutableRawPointer?, _ value: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_input(ctx: RustString(ptr: ctx), input_type: RustString(ptr: input_type), name: { let val = name; if val != nil { return RustString(ptr: val!) } else { return nil } }(), value: { let val = value; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_button")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_button (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_button(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_audio")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_audio (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ src: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_audio(ctx: RustString(ptr: ctx), src: { let val = src; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_video")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_video (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ src: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_video(ctx: RustString(ptr: ctx), src: { let val = src; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_iframe")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_iframe (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ src: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_iframe(ctx: RustString(ptr: ctx), src: { let val = src; if val != nil { return RustString(ptr: val!) } else { return nil } }()).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_details")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_details (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ open: Bool) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_details(ctx: RustString(ptr: ctx), open: open).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_summary")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_summary (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_summary(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_figure_start")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_figure_start (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_figure_start(ctx: RustString(ptr: ctx)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_figcaption")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_figcaption (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ text: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_figcaption(ctx: RustString(ptr: ctx), text: RustString(ptr: text)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$alef_visit_figure_end")
func __swift_bridge__SwiftHtmlVisitorBox_alef_visit_figure_end (_ this: UnsafeMutableRawPointer, _ ctx: UnsafeMutableRawPointer, _ output: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    { let rustString = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(this).takeUnretainedValue().alef_visit_figure_end(ctx: RustString(ptr: ctx), output: RustString(ptr: output)).intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}

public func makeHtmlVisitorHandle(_ swift_box: SwiftHtmlVisitorBox) -> VisitorHandle {
    VisitorHandle(ptr: __swift_bridge__$make_html_visitor_visitor_handle(Unmanaged.passRetained(swift_box).toOpaque()))
}
public func conversionOptionsFromJsonWithVisitor<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString, _ visitor: Optional<VisitorHandle>) throws -> ConversionOptions {
    try { let val = __swift_bridge__$conversion_options_from_json_with_visitor({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { if let val = visitor { val.isOwned = false; return val.ptr } else { return nil } }()); if val.is_ok { return ConversionOptions(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func conversionOptionsFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ConversionOptions {
    try { let val = __swift_bridge__$conversion_options_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ConversionOptions(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func conversionResultFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ConversionResult {
    try { let val = __swift_bridge__$conversion_result_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ConversionResult(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func documentMetadataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> DocumentMetadata {
    try { let val = __swift_bridge__$document_metadata_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return DocumentMetadata(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func documentNodeFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> DocumentNode {
    try { let val = __swift_bridge__$document_node_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return DocumentNode(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func documentStructureFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> DocumentStructure {
    try { let val = __swift_bridge__$document_structure_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return DocumentStructure(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func gridCellFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> GridCell {
    try { let val = __swift_bridge__$grid_cell_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return GridCell(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func headerMetadataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> HeaderMetadata {
    try { let val = __swift_bridge__$header_metadata_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return HeaderMetadata(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func htmlMetadataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> HtmlMetadata {
    try { let val = __swift_bridge__$html_metadata_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return HtmlMetadata(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func imageDimensionsFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ImageDimensions {
    try { let val = __swift_bridge__$image_dimensions_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ImageDimensions(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func imageMetadataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ImageMetadata {
    try { let val = __swift_bridge__$image_metadata_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ImageMetadata(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func linkMetadataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> LinkMetadata {
    try { let val = __swift_bridge__$link_metadata_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return LinkMetadata(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func nodeContextFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> NodeContext {
    try { let val = __swift_bridge__$node_context_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return NodeContext(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func preprocessingOptionsFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> PreprocessingOptions {
    try { let val = __swift_bridge__$preprocessing_options_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return PreprocessingOptions(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func preprocessingOptionsUpdateFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> PreprocessingOptionsUpdate {
    try { let val = __swift_bridge__$preprocessing_options_update_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return PreprocessingOptionsUpdate(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func processingWarningFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ProcessingWarning {
    try { let val = __swift_bridge__$processing_warning_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ProcessingWarning(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func structuredDataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> StructuredData {
    try { let val = __swift_bridge__$structured_data_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return StructuredData(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func tableDataFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> TableData {
    try { let val = __swift_bridge__$table_data_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return TableData(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func tableGridFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> TableGrid {
    try { let val = __swift_bridge__$table_grid_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return TableGrid(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func textAnnotationFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> TextAnnotation {
    try { let val = __swift_bridge__$text_annotation_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return TextAnnotation(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func conversionOptionsUpdateFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ConversionOptionsUpdate {
    try { let val = __swift_bridge__$conversion_options_update_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ConversionOptionsUpdate(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func metadataEntryFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> MetadataEntry {
    try { let val = __swift_bridge__$metadata_entry_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return MetadataEntry(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func annotationKindFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> AnnotationKind {
    try { let val = __swift_bridge__$annotation_kind_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return AnnotationKind(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func imageTypeFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> ImageType {
    try { let val = __swift_bridge__$image_type_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return ImageType(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func linkTypeFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> LinkType {
    try { let val = __swift_bridge__$link_type_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return LinkType(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func nodeContentFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> NodeContent {
    try { let val = __swift_bridge__$node_content_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return NodeContent(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func nodeTypeFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> NodeType {
    try { let val = __swift_bridge__$node_type_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return NodeType(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func structuredDataTypeFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> StructuredDataType {
    try { let val = __swift_bridge__$structured_data_type_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return StructuredDataType(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func textDirectionFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> TextDirection {
    try { let val = __swift_bridge__$text_direction_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return TextDirection(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func tierStrategyFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> TierStrategy {
    try { let val = __swift_bridge__$tier_strategy_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return TierStrategy(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func warningKindFromJson<GenericIntoRustString: IntoRustString>(_ json: GenericIntoRustString) throws -> WarningKind {
    try { let val = __swift_bridge__$warning_kind_from_json({ let rustString = json.intoRustString(); rustString.isOwned = false; return rustString.ptr }()); if val.is_ok { return WarningKind(ptr: val.ok_or_err!) } else { throw RustString(ptr: val.ok_or_err!) } }()
}
public func __alef_phantom_vec_conversion_options() -> RustVec<ConversionOptions> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_conversion_options())
}
public func __alef_phantom_vec_conversion_options_update() -> RustVec<ConversionOptionsUpdate> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_conversion_options_update())
}
public func __alef_phantom_vec_conversion_result() -> RustVec<ConversionResult> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_conversion_result())
}
public func __alef_phantom_vec_document_node() -> RustVec<DocumentNode> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_document_node())
}
public func __alef_phantom_vec_document_structure() -> RustVec<DocumentStructure> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_document_structure())
}
public func __alef_phantom_vec_grid_cell() -> RustVec<GridCell> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_grid_cell())
}
public func __alef_phantom_vec_image_dimensions() -> RustVec<ImageDimensions> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_image_dimensions())
}
public func __alef_phantom_vec_metadata_entry() -> RustVec<MetadataEntry> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_metadata_entry())
}
public func __alef_phantom_vec_preprocessing_options() -> RustVec<PreprocessingOptions> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_preprocessing_options())
}
public func __alef_phantom_vec_preprocessing_options_update() -> RustVec<PreprocessingOptionsUpdate> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_preprocessing_options_update())
}
public func __alef_phantom_vec_processing_warning() -> RustVec<ProcessingWarning> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_processing_warning())
}
public func __alef_phantom_vec_table_data() -> RustVec<TableData> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_table_data())
}
public func __alef_phantom_vec_table_grid() -> RustVec<TableGrid> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_table_grid())
}
public func __alef_phantom_vec_text_annotation() -> RustVec<TextAnnotation> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_text_annotation())
}
public func __alef_phantom_vec_annotation_kind() -> RustVec<AnnotationKind> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_annotation_kind())
}
public func __alef_phantom_vec_code_block_style() -> RustVec<CodeBlockStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_code_block_style())
}
public func __alef_phantom_vec_heading_style() -> RustVec<HeadingStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_heading_style())
}
public func __alef_phantom_vec_highlight_style() -> RustVec<HighlightStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_highlight_style())
}
public func __alef_phantom_vec_inline_data_media() -> RustVec<InlineDataMedia> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_inline_data_media())
}
public func __alef_phantom_vec_link_style() -> RustVec<LinkStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_link_style())
}
public func __alef_phantom_vec_list_indent_type() -> RustVec<ListIndentType> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_list_indent_type())
}
public func __alef_phantom_vec_newline_style() -> RustVec<NewlineStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_newline_style())
}
public func __alef_phantom_vec_node_content() -> RustVec<NodeContent> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_node_content())
}
public func __alef_phantom_vec_output_format() -> RustVec<OutputFormat> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_output_format())
}
public func __alef_phantom_vec_preprocessing_preset() -> RustVec<PreprocessingPreset> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_preprocessing_preset())
}
public func __alef_phantom_vec_tier_strategy() -> RustVec<TierStrategy> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_tier_strategy())
}
public func __alef_phantom_vec_url_escape_style() -> RustVec<UrlEscapeStyle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_url_escape_style())
}
public func __alef_phantom_vec_warning_kind() -> RustVec<WarningKind> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_warning_kind())
}
public func __alef_phantom_vec_whitespace_mode() -> RustVec<WhitespaceMode> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_whitespace_mode())
}
public func __alef_phantom_vec_document_metadata() -> RustVec<DocumentMetadata> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_document_metadata())
}
public func __alef_phantom_vec_header_metadata() -> RustVec<HeaderMetadata> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_header_metadata())
}
public func __alef_phantom_vec_html_metadata() -> RustVec<HtmlMetadata> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_html_metadata())
}
public func __alef_phantom_vec_image_metadata() -> RustVec<ImageMetadata> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_image_metadata())
}
public func __alef_phantom_vec_link_metadata() -> RustVec<LinkMetadata> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_link_metadata())
}
public func __alef_phantom_vec_node_context() -> RustVec<NodeContext> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_node_context())
}
public func __alef_phantom_vec_structured_data() -> RustVec<StructuredData> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_structured_data())
}
public func __alef_phantom_vec_visitor_handle() -> RustVec<VisitorHandle> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_visitor_handle())
}
public func __alef_phantom_vec_image_type() -> RustVec<ImageType> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_image_type())
}
public func __alef_phantom_vec_link_type() -> RustVec<LinkType> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_link_type())
}
public func __alef_phantom_vec_node_type() -> RustVec<NodeType> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_node_type())
}
public func __alef_phantom_vec_structured_data_type() -> RustVec<StructuredDataType> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_structured_data_type())
}
public func __alef_phantom_vec_text_direction() -> RustVec<TextDirection> {
    RustVec(ptr: __swift_bridge__$__alef_phantom_vec_text_direction())
}

public class ConversionOptions: ConversionOptionsRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ConversionOptions$_free(ptr)
        }
    }
}
extension ConversionOptions {
    public convenience init<GenericIntoRustString: IntoRustString>(_ heading_style: HeadingStyle, _ list_indent_type: ListIndentType, _ list_indent_width: UInt, _ bullets: GenericIntoRustString, _ strong_em_symbol: GenericIntoRustString, _ escape_asterisks: Bool, _ escape_underscores: Bool, _ escape_misc: Bool, _ escape_ascii: Bool, _ code_language: GenericIntoRustString, _ autolinks: Bool, _ expand_abbreviations: Bool, _ include_blockquote_citations: Bool, _ default_title: Bool, _ br_in_tables: Bool, _ compact_tables: Bool, _ highlight_style: HighlightStyle, _ extract_metadata: Bool, _ whitespace_mode: WhitespaceMode, _ strip_newlines: Bool, _ wrap: Bool, _ wrap_width: UInt, _ convert_as_inline: Bool, _ sub_symbol: GenericIntoRustString, _ sup_symbol: GenericIntoRustString, _ newline_style: NewlineStyle, _ code_block_style: CodeBlockStyle, _ keep_inline_images_in: RustVec<GenericIntoRustString>, _ preprocessing: PreprocessingOptions, _ encoding: GenericIntoRustString, _ debug: Bool, _ strip_tags: RustVec<GenericIntoRustString>, _ preserve_tags: RustVec<GenericIntoRustString>, _ skip_images: Bool, _ inline_data_media: InlineDataMedia, _ url_escape_style: UrlEscapeStyle, _ link_style: LinkStyle, _ output_format: OutputFormat, _ include_document_structure: Bool, _ extract_images: Bool, _ max_image_size: UInt64, _ max_input_size: Optional<UInt64>, _ capture_svg: Bool, _ infer_dimensions: Bool, _ max_depth: Optional<UInt>, _ exclude_selectors: RustVec<GenericIntoRustString>, _ tier_strategy: TierStrategy, _ base_url: Optional<GenericIntoRustString>, _ visitor: Optional<VisitorHandle>) {
        self.init(ptr: __swift_bridge__$ConversionOptions$new({heading_style.isOwned = false; return heading_style.ptr;}(), {list_indent_type.isOwned = false; return list_indent_type.ptr;}(), list_indent_width, { let rustString = bullets.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { let rustString = strong_em_symbol.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), escape_asterisks, escape_underscores, escape_misc, escape_ascii, { let rustString = code_language.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), autolinks, expand_abbreviations, include_blockquote_citations, default_title, br_in_tables, compact_tables, {highlight_style.isOwned = false; return highlight_style.ptr;}(), extract_metadata, {whitespace_mode.isOwned = false; return whitespace_mode.ptr;}(), strip_newlines, wrap, wrap_width, convert_as_inline, { let rustString = sub_symbol.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), { let rustString = sup_symbol.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), {newline_style.isOwned = false; return newline_style.ptr;}(), {code_block_style.isOwned = false; return code_block_style.ptr;}(), { let val = keep_inline_images_in; val.isOwned = false; return val.ptr }(), {preprocessing.isOwned = false; return preprocessing.ptr;}(), { let rustString = encoding.intoRustString(); rustString.isOwned = false; return rustString.ptr }(), debug, { let val = strip_tags; val.isOwned = false; return val.ptr }(), { let val = preserve_tags; val.isOwned = false; return val.ptr }(), skip_images, {inline_data_media.isOwned = false; return inline_data_media.ptr;}(), {url_escape_style.isOwned = false; return url_escape_style.ptr;}(), {link_style.isOwned = false; return link_style.ptr;}(), {output_format.isOwned = false; return output_format.ptr;}(), include_document_structure, extract_images, max_image_size, max_input_size.intoFfiRepr(), capture_svg, infer_dimensions, max_depth.intoFfiRepr(), { let val = exclude_selectors; val.isOwned = false; return val.ptr }(), {tier_strategy.isOwned = false; return tier_strategy.ptr;}(), { if let rustString = optionalStringIntoRustString(base_url) { rustString.isOwned = false; return rustString.ptr } else { return nil } }(), { if let val = visitor { val.isOwned = false; return val.ptr } else { return nil } }()))
    }
}
public class ConversionOptionsRefMut: ConversionOptionsRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ConversionOptionsRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ConversionOptionsRef {
    public func headingStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$heading_style(ptr))
    }

    public func listIndentType() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$list_indent_type(ptr))
    }

    public func listIndentWidth() -> UInt {
        __swift_bridge__$ConversionOptions$list_indent_width(ptr)
    }

    public func bullets() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$bullets(ptr))
    }

    public func strongEmSymbol() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$strong_em_symbol(ptr))
    }

    public func escapeAsterisks() -> Bool {
        __swift_bridge__$ConversionOptions$escape_asterisks(ptr)
    }

    public func escapeUnderscores() -> Bool {
        __swift_bridge__$ConversionOptions$escape_underscores(ptr)
    }

    public func escapeMisc() -> Bool {
        __swift_bridge__$ConversionOptions$escape_misc(ptr)
    }

    public func escapeAscii() -> Bool {
        __swift_bridge__$ConversionOptions$escape_ascii(ptr)
    }

    public func codeLanguage() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$code_language(ptr))
    }

    public func autolinks() -> Bool {
        __swift_bridge__$ConversionOptions$autolinks(ptr)
    }

    public func expandAbbreviations() -> Bool {
        __swift_bridge__$ConversionOptions$expand_abbreviations(ptr)
    }

    public func includeBlockquoteCitations() -> Bool {
        __swift_bridge__$ConversionOptions$include_blockquote_citations(ptr)
    }

    public func defaultTitle() -> Bool {
        __swift_bridge__$ConversionOptions$default_title(ptr)
    }

    public func brInTables() -> Bool {
        __swift_bridge__$ConversionOptions$br_in_tables(ptr)
    }

    public func compactTables() -> Bool {
        __swift_bridge__$ConversionOptions$compact_tables(ptr)
    }

    public func highlightStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$highlight_style(ptr))
    }

    public func extractMetadata() -> Bool {
        __swift_bridge__$ConversionOptions$extract_metadata(ptr)
    }

    public func whitespaceMode() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$whitespace_mode(ptr))
    }

    public func stripNewlines() -> Bool {
        __swift_bridge__$ConversionOptions$strip_newlines(ptr)
    }

    public func wrap() -> Bool {
        __swift_bridge__$ConversionOptions$wrap(ptr)
    }

    public func wrapWidth() -> UInt {
        __swift_bridge__$ConversionOptions$wrap_width(ptr)
    }

    public func convertAsInline() -> Bool {
        __swift_bridge__$ConversionOptions$convert_as_inline(ptr)
    }

    public func subSymbol() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$sub_symbol(ptr))
    }

    public func supSymbol() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$sup_symbol(ptr))
    }

    public func newlineStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$newline_style(ptr))
    }

    public func codeBlockStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$code_block_style(ptr))
    }

    public func keepInlineImagesIn() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionOptions$keep_inline_images_in(ptr))
    }

    public func preprocessing() -> PreprocessingOptions {
        PreprocessingOptions(ptr: __swift_bridge__$ConversionOptions$preprocessing(ptr))
    }

    public func encoding() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$encoding(ptr))
    }

    public func debug() -> Bool {
        __swift_bridge__$ConversionOptions$debug(ptr)
    }

    public func stripTags() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionOptions$strip_tags(ptr))
    }

    public func preserveTags() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionOptions$preserve_tags(ptr))
    }

    public func skipImages() -> Bool {
        __swift_bridge__$ConversionOptions$skip_images(ptr)
    }

    public func inlineDataMedia() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$inline_data_media(ptr))
    }

    public func urlEscapeStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$url_escape_style(ptr))
    }

    public func linkStyle() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$link_style(ptr))
    }

    public func outputFormat() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$output_format(ptr))
    }

    public func includeDocumentStructure() -> Bool {
        __swift_bridge__$ConversionOptions$include_document_structure(ptr)
    }

    public func extractImages() -> Bool {
        __swift_bridge__$ConversionOptions$extract_images(ptr)
    }

    public func maxImageSize() -> UInt64 {
        __swift_bridge__$ConversionOptions$max_image_size(ptr)
    }

    public func maxInputSize() -> Optional<UInt64> {
        __swift_bridge__$ConversionOptions$max_input_size(ptr).intoSwiftRepr()
    }

    public func captureSvg() -> Bool {
        __swift_bridge__$ConversionOptions$capture_svg(ptr)
    }

    public func inferDimensions() -> Bool {
        __swift_bridge__$ConversionOptions$infer_dimensions(ptr)
    }

    public func maxDepth() -> Optional<UInt> {
        __swift_bridge__$ConversionOptions$max_depth(ptr).intoSwiftRepr()
    }

    public func excludeSelectors() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionOptions$exclude_selectors(ptr))
    }

    public func tierStrategy() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptions$tier_strategy(ptr))
    }

    public func baseUrl() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptions$base_url(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func visitor() -> Optional<VisitorHandle> {
        { let val = __swift_bridge__$ConversionOptions$visitor(ptr); if val != nil { return VisitorHandle(ptr: val!) } else { return nil } }()
    }
}
extension ConversionOptions: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ConversionOptions$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ConversionOptions$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ConversionOptions) {
        __swift_bridge__$Vec_ConversionOptions$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ConversionOptions$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ConversionOptions(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionOptionsRef> {
        let pointer = __swift_bridge__$Vec_ConversionOptions$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionOptionsRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionOptionsRefMut> {
        let pointer = __swift_bridge__$Vec_ConversionOptions$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionOptionsRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ConversionOptionsRef> {
        UnsafePointer<ConversionOptionsRef>(OpaquePointer(__swift_bridge__$Vec_ConversionOptions$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ConversionOptions$len(vecPtr)
    }
}


public class ConversionOptionsUpdate: ConversionOptionsUpdateRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ConversionOptionsUpdate$_free(ptr)
        }
    }
}
public class ConversionOptionsUpdateRefMut: ConversionOptionsUpdateRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ConversionOptionsUpdateRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ConversionOptionsUpdateRef {
    public func headingStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$heading_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func listIndentType() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$list_indent_type(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func listIndentWidth() -> Optional<UInt> {
        __swift_bridge__$ConversionOptionsUpdate$list_indent_width(ptr).intoSwiftRepr()
    }

    public func bullets() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$bullets(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func strongEmSymbol() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$strong_em_symbol(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func escapeAsterisks() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$escape_asterisks(ptr).intoSwiftRepr()
    }

    public func escapeUnderscores() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$escape_underscores(ptr).intoSwiftRepr()
    }

    public func escapeMisc() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$escape_misc(ptr).intoSwiftRepr()
    }

    public func escapeAscii() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$escape_ascii(ptr).intoSwiftRepr()
    }

    public func codeLanguage() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$code_language(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func autolinks() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$autolinks(ptr).intoSwiftRepr()
    }

    public func expandAbbreviations() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$expand_abbreviations(ptr).intoSwiftRepr()
    }

    public func includeBlockquoteCitations() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$include_blockquote_citations(ptr).intoSwiftRepr()
    }

    public func defaultTitle() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$default_title(ptr).intoSwiftRepr()
    }

    public func brInTables() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$br_in_tables(ptr).intoSwiftRepr()
    }

    public func compactTables() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$compact_tables(ptr).intoSwiftRepr()
    }

    public func highlightStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$highlight_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func extractMetadata() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$extract_metadata(ptr).intoSwiftRepr()
    }

    public func whitespaceMode() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$whitespace_mode(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func stripNewlines() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$strip_newlines(ptr).intoSwiftRepr()
    }

    public func wrap() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$wrap(ptr).intoSwiftRepr()
    }

    public func wrapWidth() -> Optional<UInt> {
        __swift_bridge__$ConversionOptionsUpdate$wrap_width(ptr).intoSwiftRepr()
    }

    public func convertAsInline() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$convert_as_inline(ptr).intoSwiftRepr()
    }

    public func subSymbol() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$sub_symbol(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func supSymbol() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$sup_symbol(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func newlineStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$newline_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func codeBlockStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$code_block_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func keepInlineImagesIn() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$keep_inline_images_in(ptr))
    }

    public func preprocessing() -> Optional<PreprocessingOptionsUpdate> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$preprocessing(ptr); if val != nil { return PreprocessingOptionsUpdate(ptr: val!) } else { return nil } }()
    }

    public func encoding() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$encoding(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func debug() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$debug(ptr).intoSwiftRepr()
    }

    public func stripTags() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$strip_tags(ptr))
    }

    public func preserveTags() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$preserve_tags(ptr))
    }

    public func skipImages() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$skip_images(ptr).intoSwiftRepr()
    }

    public func inlineDataMedia() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$inline_data_media(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func urlEscapeStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$url_escape_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func linkStyle() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$link_style(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func outputFormat() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$output_format(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func includeDocumentStructure() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$include_document_structure(ptr).intoSwiftRepr()
    }

    public func extractImages() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$extract_images(ptr).intoSwiftRepr()
    }

    public func maxImageSize() -> Optional<UInt64> {
        __swift_bridge__$ConversionOptionsUpdate$max_image_size(ptr).intoSwiftRepr()
    }

    public func maxInputSize() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$max_input_size(ptr))
    }

    public func captureSvg() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$capture_svg(ptr).intoSwiftRepr()
    }

    public func inferDimensions() -> Optional<Bool> {
        __swift_bridge__$ConversionOptionsUpdate$infer_dimensions(ptr).intoSwiftRepr()
    }

    public func maxDepth() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$max_depth(ptr))
    }

    public func excludeSelectors() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$exclude_selectors(ptr))
    }

    public func tierStrategy() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$tier_strategy(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func baseUrl() -> RustString {
        RustString(ptr: __swift_bridge__$ConversionOptionsUpdate$base_url(ptr))
    }

    public func visitor() -> Optional<VisitorHandle> {
        { let val = __swift_bridge__$ConversionOptionsUpdate$visitor(ptr); if val != nil { return VisitorHandle(ptr: val!) } else { return nil } }()
    }
}
extension ConversionOptionsUpdate: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ConversionOptionsUpdate$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ConversionOptionsUpdate$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ConversionOptionsUpdate) {
        __swift_bridge__$Vec_ConversionOptionsUpdate$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ConversionOptionsUpdate$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ConversionOptionsUpdate(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionOptionsUpdateRef> {
        let pointer = __swift_bridge__$Vec_ConversionOptionsUpdate$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionOptionsUpdateRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionOptionsUpdateRefMut> {
        let pointer = __swift_bridge__$Vec_ConversionOptionsUpdate$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionOptionsUpdateRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ConversionOptionsUpdateRef> {
        UnsafePointer<ConversionOptionsUpdateRef>(OpaquePointer(__swift_bridge__$Vec_ConversionOptionsUpdate$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ConversionOptionsUpdate$len(vecPtr)
    }
}


public class ConversionResult: ConversionResultRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ConversionResult$_free(ptr)
        }
    }
}
extension ConversionResult {
    public convenience init<GenericIntoRustString: IntoRustString>(_ content: Optional<GenericIntoRustString>, _ document: Optional<DocumentStructure>, _ metadata: HtmlMetadata, _ tables: RustVec<TableData>, _ warnings: RustVec<ProcessingWarning>) {
        self.init(ptr: __swift_bridge__$ConversionResult$new({ if let rustString = optionalStringIntoRustString(content) { rustString.isOwned = false; return rustString.ptr } else { return nil } }(), { if let val = document { val.isOwned = false; return val.ptr } else { return nil } }(), {metadata.isOwned = false; return metadata.ptr;}(), { let val = tables; val.isOwned = false; return val.ptr }(), { let val = warnings; val.isOwned = false; return val.ptr }()))
    }
}
public class ConversionResultRefMut: ConversionResultRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ConversionResultRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ConversionResultRef {
    public func content() -> Optional<RustString> {
        { let val = __swift_bridge__$ConversionResult$content(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func document() -> Optional<DocumentStructure> {
        { let val = __swift_bridge__$ConversionResult$document(ptr); if val != nil { return DocumentStructure(ptr: val!) } else { return nil } }()
    }

    public func metadata() -> HtmlMetadata {
        HtmlMetadata(ptr: __swift_bridge__$ConversionResult$metadata(ptr))
    }

    public func tables() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionResult$tables(ptr))
    }

    public func warnings() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$ConversionResult$warnings(ptr))
    }
}
extension ConversionResult: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ConversionResult$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ConversionResult$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ConversionResult) {
        __swift_bridge__$Vec_ConversionResult$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ConversionResult$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ConversionResult(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionResultRef> {
        let pointer = __swift_bridge__$Vec_ConversionResult$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionResultRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ConversionResultRefMut> {
        let pointer = __swift_bridge__$Vec_ConversionResult$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ConversionResultRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ConversionResultRef> {
        UnsafePointer<ConversionResultRef>(OpaquePointer(__swift_bridge__$Vec_ConversionResult$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ConversionResult$len(vecPtr)
    }
}


public class DocumentMetadata: DocumentMetadataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$DocumentMetadata$_free(ptr)
        }
    }
}
public class DocumentMetadataRefMut: DocumentMetadataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class DocumentMetadataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension DocumentMetadataRef {
    public func title() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$title(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func description() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$description(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func keywords() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$DocumentMetadata$keywords(ptr))
    }

    public func author() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$author(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func canonicalUrl() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$canonical_url(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func baseHref() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$base_href(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func language() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$language(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func textDirection() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentMetadata$text_direction(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func openGraph() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentMetadata$open_graph(ptr))
    }

    public func twitterCard() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentMetadata$twitter_card(ptr))
    }

    public func metaTags() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentMetadata$meta_tags(ptr))
    }
}
extension DocumentMetadata: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_DocumentMetadata$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_DocumentMetadata$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: DocumentMetadata) {
        __swift_bridge__$Vec_DocumentMetadata$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_DocumentMetadata$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (DocumentMetadata(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentMetadataRef> {
        let pointer = __swift_bridge__$Vec_DocumentMetadata$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentMetadataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentMetadataRefMut> {
        let pointer = __swift_bridge__$Vec_DocumentMetadata$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentMetadataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<DocumentMetadataRef> {
        UnsafePointer<DocumentMetadataRef>(OpaquePointer(__swift_bridge__$Vec_DocumentMetadata$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_DocumentMetadata$len(vecPtr)
    }
}


public class DocumentNode: DocumentNodeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$DocumentNode$_free(ptr)
        }
    }
}
public class DocumentNodeRefMut: DocumentNodeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class DocumentNodeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension DocumentNodeRef {
    public func id() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentNode$id(ptr))
    }

    public func content() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentNode$content(ptr))
    }

    public func parent() -> Optional<UInt32> {
        __swift_bridge__$DocumentNode$parent(ptr).intoSwiftRepr()
    }

    public func children() -> RustVec<UInt32> {
        RustVec(ptr: __swift_bridge__$DocumentNode$children(ptr))
    }

    public func annotations() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$DocumentNode$annotations(ptr))
    }

    public func attributes() -> RustString {
        RustString(ptr: __swift_bridge__$DocumentNode$attributes(ptr))
    }
}
extension DocumentNode: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_DocumentNode$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_DocumentNode$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: DocumentNode) {
        __swift_bridge__$Vec_DocumentNode$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_DocumentNode$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (DocumentNode(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentNodeRef> {
        let pointer = __swift_bridge__$Vec_DocumentNode$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentNodeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentNodeRefMut> {
        let pointer = __swift_bridge__$Vec_DocumentNode$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentNodeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<DocumentNodeRef> {
        UnsafePointer<DocumentNodeRef>(OpaquePointer(__swift_bridge__$Vec_DocumentNode$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_DocumentNode$len(vecPtr)
    }
}


public class DocumentStructure: DocumentStructureRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$DocumentStructure$_free(ptr)
        }
    }
}
public class DocumentStructureRefMut: DocumentStructureRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class DocumentStructureRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension DocumentStructureRef {
    public func nodes() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$DocumentStructure$nodes(ptr))
    }

    public func sourceFormat() -> Optional<RustString> {
        { let val = __swift_bridge__$DocumentStructure$source_format(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }
}
extension DocumentStructure: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_DocumentStructure$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_DocumentStructure$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: DocumentStructure) {
        __swift_bridge__$Vec_DocumentStructure$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_DocumentStructure$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (DocumentStructure(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentStructureRef> {
        let pointer = __swift_bridge__$Vec_DocumentStructure$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentStructureRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<DocumentStructureRefMut> {
        let pointer = __swift_bridge__$Vec_DocumentStructure$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return DocumentStructureRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<DocumentStructureRef> {
        UnsafePointer<DocumentStructureRef>(OpaquePointer(__swift_bridge__$Vec_DocumentStructure$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_DocumentStructure$len(vecPtr)
    }
}


public class GridCell: GridCellRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$GridCell$_free(ptr)
        }
    }
}
public class GridCellRefMut: GridCellRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class GridCellRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension GridCellRef {
    public func content() -> RustString {
        RustString(ptr: __swift_bridge__$GridCell$content(ptr))
    }

    public func row() -> UInt32 {
        __swift_bridge__$GridCell$row(ptr)
    }

    public func col() -> UInt32 {
        __swift_bridge__$GridCell$col(ptr)
    }

    public func rowSpan() -> UInt32 {
        __swift_bridge__$GridCell$row_span(ptr)
    }

    public func colSpan() -> UInt32 {
        __swift_bridge__$GridCell$col_span(ptr)
    }

    public func isHeader() -> Bool {
        __swift_bridge__$GridCell$is_header(ptr)
    }
}
extension GridCell: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_GridCell$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_GridCell$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: GridCell) {
        __swift_bridge__$Vec_GridCell$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_GridCell$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (GridCell(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<GridCellRef> {
        let pointer = __swift_bridge__$Vec_GridCell$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return GridCellRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<GridCellRefMut> {
        let pointer = __swift_bridge__$Vec_GridCell$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return GridCellRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<GridCellRef> {
        UnsafePointer<GridCellRef>(OpaquePointer(__swift_bridge__$Vec_GridCell$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_GridCell$len(vecPtr)
    }
}


public class HeaderMetadata: HeaderMetadataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$HeaderMetadata$_free(ptr)
        }
    }
}
public class HeaderMetadataRefMut: HeaderMetadataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class HeaderMetadataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension HeaderMetadataRef {
    public func level() -> UInt8 {
        __swift_bridge__$HeaderMetadata$level(ptr)
    }

    public func text() -> RustString {
        RustString(ptr: __swift_bridge__$HeaderMetadata$text(ptr))
    }

    public func id() -> Optional<RustString> {
        { let val = __swift_bridge__$HeaderMetadata$id(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func depth() -> UInt {
        __swift_bridge__$HeaderMetadata$depth(ptr)
    }

    public func htmlOffset() -> UInt {
        __swift_bridge__$HeaderMetadata$html_offset(ptr)
    }
}
extension HeaderMetadata: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_HeaderMetadata$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_HeaderMetadata$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: HeaderMetadata) {
        __swift_bridge__$Vec_HeaderMetadata$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_HeaderMetadata$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (HeaderMetadata(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HeaderMetadataRef> {
        let pointer = __swift_bridge__$Vec_HeaderMetadata$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HeaderMetadataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HeaderMetadataRefMut> {
        let pointer = __swift_bridge__$Vec_HeaderMetadata$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HeaderMetadataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<HeaderMetadataRef> {
        UnsafePointer<HeaderMetadataRef>(OpaquePointer(__swift_bridge__$Vec_HeaderMetadata$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_HeaderMetadata$len(vecPtr)
    }
}


public class HtmlMetadata: HtmlMetadataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$HtmlMetadata$_free(ptr)
        }
    }
}
extension HtmlMetadata {
    public convenience init(_ document: DocumentMetadata, _ headers: RustVec<HeaderMetadata>, _ links: RustVec<LinkMetadata>, _ images: RustVec<ImageMetadata>, _ structured_data: RustVec<StructuredData>) {
        self.init(ptr: __swift_bridge__$HtmlMetadata$new({document.isOwned = false; return document.ptr;}(), { let val = headers; val.isOwned = false; return val.ptr }(), { let val = links; val.isOwned = false; return val.ptr }(), { let val = images; val.isOwned = false; return val.ptr }(), { let val = structured_data; val.isOwned = false; return val.ptr }()))
    }
}
public class HtmlMetadataRefMut: HtmlMetadataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class HtmlMetadataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension HtmlMetadataRef {
    public func document() -> DocumentMetadata {
        DocumentMetadata(ptr: __swift_bridge__$HtmlMetadata$document(ptr))
    }

    public func headers() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$HtmlMetadata$headers(ptr))
    }

    public func links() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$HtmlMetadata$links(ptr))
    }

    public func images() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$HtmlMetadata$images(ptr))
    }

    public func structuredData() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$HtmlMetadata$structured_data(ptr))
    }
}
extension HtmlMetadata: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_HtmlMetadata$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_HtmlMetadata$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: HtmlMetadata) {
        __swift_bridge__$Vec_HtmlMetadata$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_HtmlMetadata$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (HtmlMetadata(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HtmlMetadataRef> {
        let pointer = __swift_bridge__$Vec_HtmlMetadata$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HtmlMetadataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HtmlMetadataRefMut> {
        let pointer = __swift_bridge__$Vec_HtmlMetadata$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HtmlMetadataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<HtmlMetadataRef> {
        UnsafePointer<HtmlMetadataRef>(OpaquePointer(__swift_bridge__$Vec_HtmlMetadata$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_HtmlMetadata$len(vecPtr)
    }
}


public class ImageDimensions: ImageDimensionsRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ImageDimensions$_free(ptr)
        }
    }
}
extension ImageDimensions {
    public convenience init(_ width: UInt32, _ height: UInt32) {
        self.init(ptr: __swift_bridge__$ImageDimensions$new(width, height))
    }
}
public class ImageDimensionsRefMut: ImageDimensionsRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ImageDimensionsRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ImageDimensionsRef {
    public func width() -> UInt32 {
        __swift_bridge__$ImageDimensions$width(ptr)
    }

    public func height() -> UInt32 {
        __swift_bridge__$ImageDimensions$height(ptr)
    }
}
extension ImageDimensions: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ImageDimensions$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ImageDimensions$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ImageDimensions) {
        __swift_bridge__$Vec_ImageDimensions$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ImageDimensions$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ImageDimensions(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageDimensionsRef> {
        let pointer = __swift_bridge__$Vec_ImageDimensions$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageDimensionsRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageDimensionsRefMut> {
        let pointer = __swift_bridge__$Vec_ImageDimensions$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageDimensionsRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ImageDimensionsRef> {
        UnsafePointer<ImageDimensionsRef>(OpaquePointer(__swift_bridge__$Vec_ImageDimensions$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ImageDimensions$len(vecPtr)
    }
}


public class ImageMetadata: ImageMetadataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ImageMetadata$_free(ptr)
        }
    }
}
public class ImageMetadataRefMut: ImageMetadataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ImageMetadataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ImageMetadataRef {
    public func src() -> RustString {
        RustString(ptr: __swift_bridge__$ImageMetadata$src(ptr))
    }

    public func alt() -> Optional<RustString> {
        { let val = __swift_bridge__$ImageMetadata$alt(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func title() -> Optional<RustString> {
        { let val = __swift_bridge__$ImageMetadata$title(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func dimensions() -> Optional<ImageDimensions> {
        { let val = __swift_bridge__$ImageMetadata$dimensions(ptr); if val != nil { return ImageDimensions(ptr: val!) } else { return nil } }()
    }

    public func imageType() -> RustString {
        RustString(ptr: __swift_bridge__$ImageMetadata$image_type(ptr))
    }

    public func attributes() -> RustString {
        RustString(ptr: __swift_bridge__$ImageMetadata$attributes(ptr))
    }
}
extension ImageMetadata: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ImageMetadata$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ImageMetadata$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ImageMetadata) {
        __swift_bridge__$Vec_ImageMetadata$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ImageMetadata$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ImageMetadata(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageMetadataRef> {
        let pointer = __swift_bridge__$Vec_ImageMetadata$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageMetadataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageMetadataRefMut> {
        let pointer = __swift_bridge__$Vec_ImageMetadata$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageMetadataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ImageMetadataRef> {
        UnsafePointer<ImageMetadataRef>(OpaquePointer(__swift_bridge__$Vec_ImageMetadata$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ImageMetadata$len(vecPtr)
    }
}


public class LinkMetadata: LinkMetadataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$LinkMetadata$_free(ptr)
        }
    }
}
public class LinkMetadataRefMut: LinkMetadataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class LinkMetadataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension LinkMetadataRef {
    public func href() -> RustString {
        RustString(ptr: __swift_bridge__$LinkMetadata$href(ptr))
    }

    public func text() -> RustString {
        RustString(ptr: __swift_bridge__$LinkMetadata$text(ptr))
    }

    public func title() -> Optional<RustString> {
        { let val = __swift_bridge__$LinkMetadata$title(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func linkType() -> RustString {
        RustString(ptr: __swift_bridge__$LinkMetadata$link_type(ptr))
    }

    public func rel() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$LinkMetadata$rel(ptr))
    }

    public func attributes() -> RustString {
        RustString(ptr: __swift_bridge__$LinkMetadata$attributes(ptr))
    }
}
extension LinkMetadata: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_LinkMetadata$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_LinkMetadata$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: LinkMetadata) {
        __swift_bridge__$Vec_LinkMetadata$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_LinkMetadata$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (LinkMetadata(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkMetadataRef> {
        let pointer = __swift_bridge__$Vec_LinkMetadata$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkMetadataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkMetadataRefMut> {
        let pointer = __swift_bridge__$Vec_LinkMetadata$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkMetadataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<LinkMetadataRef> {
        UnsafePointer<LinkMetadataRef>(OpaquePointer(__swift_bridge__$Vec_LinkMetadata$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_LinkMetadata$len(vecPtr)
    }
}


public class MetadataEntry: MetadataEntryRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$MetadataEntry$_free(ptr)
        }
    }
}
public class MetadataEntryRefMut: MetadataEntryRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class MetadataEntryRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension MetadataEntryRef {
    public func key() -> RustString {
        RustString(ptr: __swift_bridge__$MetadataEntry$key(ptr))
    }

    public func value() -> RustString {
        RustString(ptr: __swift_bridge__$MetadataEntry$value(ptr))
    }
}
extension MetadataEntry: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_MetadataEntry$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_MetadataEntry$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: MetadataEntry) {
        __swift_bridge__$Vec_MetadataEntry$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_MetadataEntry$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (MetadataEntry(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<MetadataEntryRef> {
        let pointer = __swift_bridge__$Vec_MetadataEntry$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return MetadataEntryRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<MetadataEntryRefMut> {
        let pointer = __swift_bridge__$Vec_MetadataEntry$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return MetadataEntryRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<MetadataEntryRef> {
        UnsafePointer<MetadataEntryRef>(OpaquePointer(__swift_bridge__$Vec_MetadataEntry$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_MetadataEntry$len(vecPtr)
    }
}


public class NodeContext: NodeContextRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$NodeContext$_free(ptr)
        }
    }
}
public class NodeContextRefMut: NodeContextRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class NodeContextRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension NodeContextRef {
    public func nodeType() -> RustString {
        RustString(ptr: __swift_bridge__$NodeContext$node_type(ptr))
    }

    public func tagName() -> RustString {
        RustString(ptr: __swift_bridge__$NodeContext$tag_name(ptr))
    }

    public func depth() -> UInt {
        __swift_bridge__$NodeContext$depth(ptr)
    }

    public func indexInParent() -> UInt {
        __swift_bridge__$NodeContext$index_in_parent(ptr)
    }

    public func parentTag() -> Optional<RustString> {
        { let val = __swift_bridge__$NodeContext$parent_tag(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func isInline() -> Bool {
        __swift_bridge__$NodeContext$is_inline(ptr)
    }
}
extension NodeContext: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_NodeContext$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_NodeContext$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: NodeContext) {
        __swift_bridge__$Vec_NodeContext$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_NodeContext$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (NodeContext(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeContextRef> {
        let pointer = __swift_bridge__$Vec_NodeContext$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeContextRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeContextRefMut> {
        let pointer = __swift_bridge__$Vec_NodeContext$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeContextRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<NodeContextRef> {
        UnsafePointer<NodeContextRef>(OpaquePointer(__swift_bridge__$Vec_NodeContext$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_NodeContext$len(vecPtr)
    }
}


public class PreprocessingOptions: PreprocessingOptionsRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$PreprocessingOptions$_free(ptr)
        }
    }
}
extension PreprocessingOptions {
    public convenience init(_ enabled: Bool, _ preset: PreprocessingPreset, _ remove_navigation: Bool, _ remove_forms: Bool) {
        self.init(ptr: __swift_bridge__$PreprocessingOptions$new(enabled, {preset.isOwned = false; return preset.ptr;}(), remove_navigation, remove_forms))
    }
}
public class PreprocessingOptionsRefMut: PreprocessingOptionsRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class PreprocessingOptionsRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension PreprocessingOptionsRef {
    public func enabled() -> Bool {
        __swift_bridge__$PreprocessingOptions$enabled(ptr)
    }

    public func preset() -> RustString {
        RustString(ptr: __swift_bridge__$PreprocessingOptions$preset(ptr))
    }

    public func removeNavigation() -> Bool {
        __swift_bridge__$PreprocessingOptions$remove_navigation(ptr)
    }

    public func removeForms() -> Bool {
        __swift_bridge__$PreprocessingOptions$remove_forms(ptr)
    }
}
extension PreprocessingOptions: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_PreprocessingOptions$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_PreprocessingOptions$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: PreprocessingOptions) {
        __swift_bridge__$Vec_PreprocessingOptions$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptions$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (PreprocessingOptions(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingOptionsRef> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptions$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingOptionsRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingOptionsRefMut> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptions$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingOptionsRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<PreprocessingOptionsRef> {
        UnsafePointer<PreprocessingOptionsRef>(OpaquePointer(__swift_bridge__$Vec_PreprocessingOptions$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_PreprocessingOptions$len(vecPtr)
    }
}


public class PreprocessingOptionsUpdate: PreprocessingOptionsUpdateRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$PreprocessingOptionsUpdate$_free(ptr)
        }
    }
}
extension PreprocessingOptionsUpdate {
    public convenience init(_ enabled: Optional<Bool>, _ preset: Optional<PreprocessingPreset>, _ remove_navigation: Optional<Bool>, _ remove_forms: Optional<Bool>) {
        self.init(ptr: __swift_bridge__$PreprocessingOptionsUpdate$new(enabled.intoFfiRepr(), { if let val = preset { val.isOwned = false; return val.ptr } else { return nil } }(), remove_navigation.intoFfiRepr(), remove_forms.intoFfiRepr()))
    }
}
public class PreprocessingOptionsUpdateRefMut: PreprocessingOptionsUpdateRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class PreprocessingOptionsUpdateRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension PreprocessingOptionsUpdateRef {
    public func enabled() -> Optional<Bool> {
        __swift_bridge__$PreprocessingOptionsUpdate$enabled(ptr).intoSwiftRepr()
    }

    public func preset() -> Optional<RustString> {
        { let val = __swift_bridge__$PreprocessingOptionsUpdate$preset(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }

    public func removeNavigation() -> Optional<Bool> {
        __swift_bridge__$PreprocessingOptionsUpdate$remove_navigation(ptr).intoSwiftRepr()
    }

    public func removeForms() -> Optional<Bool> {
        __swift_bridge__$PreprocessingOptionsUpdate$remove_forms(ptr).intoSwiftRepr()
    }
}
extension PreprocessingOptionsUpdate: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_PreprocessingOptionsUpdate$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_PreprocessingOptionsUpdate$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: PreprocessingOptionsUpdate) {
        __swift_bridge__$Vec_PreprocessingOptionsUpdate$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptionsUpdate$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (PreprocessingOptionsUpdate(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingOptionsUpdateRef> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptionsUpdate$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingOptionsUpdateRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingOptionsUpdateRefMut> {
        let pointer = __swift_bridge__$Vec_PreprocessingOptionsUpdate$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingOptionsUpdateRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<PreprocessingOptionsUpdateRef> {
        UnsafePointer<PreprocessingOptionsUpdateRef>(OpaquePointer(__swift_bridge__$Vec_PreprocessingOptionsUpdate$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_PreprocessingOptionsUpdate$len(vecPtr)
    }
}


public class ProcessingWarning: ProcessingWarningRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ProcessingWarning$_free(ptr)
        }
    }
}
public class ProcessingWarningRefMut: ProcessingWarningRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ProcessingWarningRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ProcessingWarningRef {
    public func message() -> RustString {
        RustString(ptr: __swift_bridge__$ProcessingWarning$message(ptr))
    }

    public func kind() -> RustString {
        RustString(ptr: __swift_bridge__$ProcessingWarning$kind(ptr))
    }
}
extension ProcessingWarning: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ProcessingWarning$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ProcessingWarning$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ProcessingWarning) {
        __swift_bridge__$Vec_ProcessingWarning$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ProcessingWarning$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ProcessingWarning(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ProcessingWarningRef> {
        let pointer = __swift_bridge__$Vec_ProcessingWarning$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ProcessingWarningRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ProcessingWarningRefMut> {
        let pointer = __swift_bridge__$Vec_ProcessingWarning$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ProcessingWarningRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ProcessingWarningRef> {
        UnsafePointer<ProcessingWarningRef>(OpaquePointer(__swift_bridge__$Vec_ProcessingWarning$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ProcessingWarning$len(vecPtr)
    }
}


public class StructuredData: StructuredDataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$StructuredData$_free(ptr)
        }
    }
}
public class StructuredDataRefMut: StructuredDataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class StructuredDataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension StructuredDataRef {
    public func dataType() -> RustString {
        RustString(ptr: __swift_bridge__$StructuredData$data_type(ptr))
    }

    public func rawJson() -> RustString {
        RustString(ptr: __swift_bridge__$StructuredData$raw_json(ptr))
    }

    public func schemaType() -> Optional<RustString> {
        { let val = __swift_bridge__$StructuredData$schema_type(ptr); if val != nil { return RustString(ptr: val!) } else { return nil } }()
    }
}
extension StructuredData: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_StructuredData$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_StructuredData$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: StructuredData) {
        __swift_bridge__$Vec_StructuredData$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_StructuredData$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (StructuredData(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<StructuredDataRef> {
        let pointer = __swift_bridge__$Vec_StructuredData$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return StructuredDataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<StructuredDataRefMut> {
        let pointer = __swift_bridge__$Vec_StructuredData$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return StructuredDataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<StructuredDataRef> {
        UnsafePointer<StructuredDataRef>(OpaquePointer(__swift_bridge__$Vec_StructuredData$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_StructuredData$len(vecPtr)
    }
}


public class TableData: TableDataRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$TableData$_free(ptr)
        }
    }
}
public class TableDataRefMut: TableDataRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class TableDataRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension TableDataRef {
    public func grid() -> TableGrid {
        TableGrid(ptr: __swift_bridge__$TableData$grid(ptr))
    }

    public func markdown() -> RustString {
        RustString(ptr: __swift_bridge__$TableData$markdown(ptr))
    }
}
extension TableData: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_TableData$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_TableData$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: TableData) {
        __swift_bridge__$Vec_TableData$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_TableData$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (TableData(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TableDataRef> {
        let pointer = __swift_bridge__$Vec_TableData$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TableDataRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TableDataRefMut> {
        let pointer = __swift_bridge__$Vec_TableData$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TableDataRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<TableDataRef> {
        UnsafePointer<TableDataRef>(OpaquePointer(__swift_bridge__$Vec_TableData$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_TableData$len(vecPtr)
    }
}


public class TableGrid: TableGridRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$TableGrid$_free(ptr)
        }
    }
}
extension TableGrid {
    public convenience init(_ rows: UInt32, _ cols: UInt32, _ cells: RustVec<GridCell>) {
        self.init(ptr: __swift_bridge__$TableGrid$new(rows, cols, { let val = cells; val.isOwned = false; return val.ptr }()))
    }
}
public class TableGridRefMut: TableGridRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class TableGridRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension TableGridRef {
    public func rows() -> UInt32 {
        __swift_bridge__$TableGrid$rows(ptr)
    }

    public func cols() -> UInt32 {
        __swift_bridge__$TableGrid$cols(ptr)
    }

    public func cells() -> RustVec<RustString> {
        RustVec(ptr: __swift_bridge__$TableGrid$cells(ptr))
    }
}
extension TableGrid: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_TableGrid$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_TableGrid$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: TableGrid) {
        __swift_bridge__$Vec_TableGrid$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_TableGrid$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (TableGrid(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TableGridRef> {
        let pointer = __swift_bridge__$Vec_TableGrid$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TableGridRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TableGridRefMut> {
        let pointer = __swift_bridge__$Vec_TableGrid$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TableGridRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<TableGridRef> {
        UnsafePointer<TableGridRef>(OpaquePointer(__swift_bridge__$Vec_TableGrid$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_TableGrid$len(vecPtr)
    }
}


public class TextAnnotation: TextAnnotationRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$TextAnnotation$_free(ptr)
        }
    }
}
public class TextAnnotationRefMut: TextAnnotationRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class TextAnnotationRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension TextAnnotationRef {
    public func start() -> UInt32 {
        __swift_bridge__$TextAnnotation$start(ptr)
    }

    public func end() -> UInt32 {
        __swift_bridge__$TextAnnotation$end(ptr)
    }

    public func kind() -> RustString {
        RustString(ptr: __swift_bridge__$TextAnnotation$kind(ptr))
    }
}
extension TextAnnotation: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_TextAnnotation$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_TextAnnotation$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: TextAnnotation) {
        __swift_bridge__$Vec_TextAnnotation$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_TextAnnotation$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (TextAnnotation(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TextAnnotationRef> {
        let pointer = __swift_bridge__$Vec_TextAnnotation$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TextAnnotationRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TextAnnotationRefMut> {
        let pointer = __swift_bridge__$Vec_TextAnnotation$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TextAnnotationRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<TextAnnotationRef> {
        UnsafePointer<TextAnnotationRef>(OpaquePointer(__swift_bridge__$Vec_TextAnnotation$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_TextAnnotation$len(vecPtr)
    }
}


public class VisitorHandle: VisitorHandleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$VisitorHandle$_free(ptr)
        }
    }
}
public class VisitorHandleRefMut: VisitorHandleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class VisitorHandleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension VisitorHandle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_VisitorHandle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_VisitorHandle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: VisitorHandle) {
        __swift_bridge__$Vec_VisitorHandle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_VisitorHandle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (VisitorHandle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<VisitorHandleRef> {
        let pointer = __swift_bridge__$Vec_VisitorHandle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return VisitorHandleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<VisitorHandleRefMut> {
        let pointer = __swift_bridge__$Vec_VisitorHandle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return VisitorHandleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<VisitorHandleRef> {
        UnsafePointer<VisitorHandleRef>(OpaquePointer(__swift_bridge__$Vec_VisitorHandle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_VisitorHandle$len(vecPtr)
    }
}


public class AnnotationKind: AnnotationKindRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$AnnotationKind$_free(ptr)
        }
    }
}
public class AnnotationKindRefMut: AnnotationKindRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class AnnotationKindRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension AnnotationKindRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$AnnotationKind$to_string(ptr))
    }
}
extension AnnotationKind: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_AnnotationKind$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_AnnotationKind$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: AnnotationKind) {
        __swift_bridge__$Vec_AnnotationKind$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_AnnotationKind$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (AnnotationKind(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<AnnotationKindRef> {
        let pointer = __swift_bridge__$Vec_AnnotationKind$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return AnnotationKindRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<AnnotationKindRefMut> {
        let pointer = __swift_bridge__$Vec_AnnotationKind$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return AnnotationKindRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<AnnotationKindRef> {
        UnsafePointer<AnnotationKindRef>(OpaquePointer(__swift_bridge__$Vec_AnnotationKind$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_AnnotationKind$len(vecPtr)
    }
}


public class CodeBlockStyle: CodeBlockStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$CodeBlockStyle$_free(ptr)
        }
    }
}
public class CodeBlockStyleRefMut: CodeBlockStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class CodeBlockStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension CodeBlockStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$CodeBlockStyle$to_string(ptr))
    }
}
extension CodeBlockStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_CodeBlockStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_CodeBlockStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: CodeBlockStyle) {
        __swift_bridge__$Vec_CodeBlockStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_CodeBlockStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (CodeBlockStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<CodeBlockStyleRef> {
        let pointer = __swift_bridge__$Vec_CodeBlockStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return CodeBlockStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<CodeBlockStyleRefMut> {
        let pointer = __swift_bridge__$Vec_CodeBlockStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return CodeBlockStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<CodeBlockStyleRef> {
        UnsafePointer<CodeBlockStyleRef>(OpaquePointer(__swift_bridge__$Vec_CodeBlockStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_CodeBlockStyle$len(vecPtr)
    }
}


public class HeadingStyle: HeadingStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$HeadingStyle$_free(ptr)
        }
    }
}
public class HeadingStyleRefMut: HeadingStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class HeadingStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension HeadingStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$HeadingStyle$to_string(ptr))
    }
}
extension HeadingStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_HeadingStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_HeadingStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: HeadingStyle) {
        __swift_bridge__$Vec_HeadingStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_HeadingStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (HeadingStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HeadingStyleRef> {
        let pointer = __swift_bridge__$Vec_HeadingStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HeadingStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HeadingStyleRefMut> {
        let pointer = __swift_bridge__$Vec_HeadingStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HeadingStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<HeadingStyleRef> {
        UnsafePointer<HeadingStyleRef>(OpaquePointer(__swift_bridge__$Vec_HeadingStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_HeadingStyle$len(vecPtr)
    }
}


public class HighlightStyle: HighlightStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$HighlightStyle$_free(ptr)
        }
    }
}
public class HighlightStyleRefMut: HighlightStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class HighlightStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension HighlightStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$HighlightStyle$to_string(ptr))
    }
}
extension HighlightStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_HighlightStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_HighlightStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: HighlightStyle) {
        __swift_bridge__$Vec_HighlightStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_HighlightStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (HighlightStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HighlightStyleRef> {
        let pointer = __swift_bridge__$Vec_HighlightStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HighlightStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HighlightStyleRefMut> {
        let pointer = __swift_bridge__$Vec_HighlightStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HighlightStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<HighlightStyleRef> {
        UnsafePointer<HighlightStyleRef>(OpaquePointer(__swift_bridge__$Vec_HighlightStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_HighlightStyle$len(vecPtr)
    }
}


public class ImageType: ImageTypeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ImageType$_free(ptr)
        }
    }
}
public class ImageTypeRefMut: ImageTypeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ImageTypeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ImageTypeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$ImageType$to_string(ptr))
    }
}
extension ImageType: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ImageType$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ImageType$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ImageType) {
        __swift_bridge__$Vec_ImageType$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ImageType$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ImageType(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageTypeRef> {
        let pointer = __swift_bridge__$Vec_ImageType$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageTypeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ImageTypeRefMut> {
        let pointer = __swift_bridge__$Vec_ImageType$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ImageTypeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ImageTypeRef> {
        UnsafePointer<ImageTypeRef>(OpaquePointer(__swift_bridge__$Vec_ImageType$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ImageType$len(vecPtr)
    }
}


public class InlineDataMedia: InlineDataMediaRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$InlineDataMedia$_free(ptr)
        }
    }
}
public class InlineDataMediaRefMut: InlineDataMediaRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class InlineDataMediaRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension InlineDataMediaRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$InlineDataMedia$to_string(ptr))
    }
}
extension InlineDataMedia: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_InlineDataMedia$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_InlineDataMedia$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: InlineDataMedia) {
        __swift_bridge__$Vec_InlineDataMedia$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_InlineDataMedia$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (InlineDataMedia(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<InlineDataMediaRef> {
        let pointer = __swift_bridge__$Vec_InlineDataMedia$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return InlineDataMediaRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<InlineDataMediaRefMut> {
        let pointer = __swift_bridge__$Vec_InlineDataMedia$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return InlineDataMediaRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<InlineDataMediaRef> {
        UnsafePointer<InlineDataMediaRef>(OpaquePointer(__swift_bridge__$Vec_InlineDataMedia$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_InlineDataMedia$len(vecPtr)
    }
}


public class LinkStyle: LinkStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$LinkStyle$_free(ptr)
        }
    }
}
public class LinkStyleRefMut: LinkStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class LinkStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension LinkStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$LinkStyle$to_string(ptr))
    }
}
extension LinkStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_LinkStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_LinkStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: LinkStyle) {
        __swift_bridge__$Vec_LinkStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_LinkStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (LinkStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkStyleRef> {
        let pointer = __swift_bridge__$Vec_LinkStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkStyleRefMut> {
        let pointer = __swift_bridge__$Vec_LinkStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<LinkStyleRef> {
        UnsafePointer<LinkStyleRef>(OpaquePointer(__swift_bridge__$Vec_LinkStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_LinkStyle$len(vecPtr)
    }
}


public class LinkType: LinkTypeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$LinkType$_free(ptr)
        }
    }
}
public class LinkTypeRefMut: LinkTypeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class LinkTypeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension LinkTypeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$LinkType$to_string(ptr))
    }
}
extension LinkType: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_LinkType$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_LinkType$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: LinkType) {
        __swift_bridge__$Vec_LinkType$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_LinkType$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (LinkType(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkTypeRef> {
        let pointer = __swift_bridge__$Vec_LinkType$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkTypeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<LinkTypeRefMut> {
        let pointer = __swift_bridge__$Vec_LinkType$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return LinkTypeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<LinkTypeRef> {
        UnsafePointer<LinkTypeRef>(OpaquePointer(__swift_bridge__$Vec_LinkType$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_LinkType$len(vecPtr)
    }
}


public class ListIndentType: ListIndentTypeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$ListIndentType$_free(ptr)
        }
    }
}
public class ListIndentTypeRefMut: ListIndentTypeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class ListIndentTypeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension ListIndentTypeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$ListIndentType$to_string(ptr))
    }
}
extension ListIndentType: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_ListIndentType$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_ListIndentType$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: ListIndentType) {
        __swift_bridge__$Vec_ListIndentType$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_ListIndentType$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (ListIndentType(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ListIndentTypeRef> {
        let pointer = __swift_bridge__$Vec_ListIndentType$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ListIndentTypeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<ListIndentTypeRefMut> {
        let pointer = __swift_bridge__$Vec_ListIndentType$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return ListIndentTypeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<ListIndentTypeRef> {
        UnsafePointer<ListIndentTypeRef>(OpaquePointer(__swift_bridge__$Vec_ListIndentType$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_ListIndentType$len(vecPtr)
    }
}


public class NewlineStyle: NewlineStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$NewlineStyle$_free(ptr)
        }
    }
}
public class NewlineStyleRefMut: NewlineStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class NewlineStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension NewlineStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$NewlineStyle$to_string(ptr))
    }
}
extension NewlineStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_NewlineStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_NewlineStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: NewlineStyle) {
        __swift_bridge__$Vec_NewlineStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_NewlineStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (NewlineStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NewlineStyleRef> {
        let pointer = __swift_bridge__$Vec_NewlineStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NewlineStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NewlineStyleRefMut> {
        let pointer = __swift_bridge__$Vec_NewlineStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NewlineStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<NewlineStyleRef> {
        UnsafePointer<NewlineStyleRef>(OpaquePointer(__swift_bridge__$Vec_NewlineStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_NewlineStyle$len(vecPtr)
    }
}


public class NodeContent: NodeContentRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$NodeContent$_free(ptr)
        }
    }
}
public class NodeContentRefMut: NodeContentRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class NodeContentRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension NodeContentRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$NodeContent$to_string(ptr))
    }
}
extension NodeContent: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_NodeContent$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_NodeContent$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: NodeContent) {
        __swift_bridge__$Vec_NodeContent$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_NodeContent$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (NodeContent(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeContentRef> {
        let pointer = __swift_bridge__$Vec_NodeContent$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeContentRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeContentRefMut> {
        let pointer = __swift_bridge__$Vec_NodeContent$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeContentRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<NodeContentRef> {
        UnsafePointer<NodeContentRef>(OpaquePointer(__swift_bridge__$Vec_NodeContent$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_NodeContent$len(vecPtr)
    }
}


public class NodeType: NodeTypeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$NodeType$_free(ptr)
        }
    }
}
public class NodeTypeRefMut: NodeTypeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class NodeTypeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension NodeTypeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$NodeType$to_string(ptr))
    }
}
extension NodeType: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_NodeType$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_NodeType$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: NodeType) {
        __swift_bridge__$Vec_NodeType$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_NodeType$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (NodeType(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeTypeRef> {
        let pointer = __swift_bridge__$Vec_NodeType$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeTypeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<NodeTypeRefMut> {
        let pointer = __swift_bridge__$Vec_NodeType$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return NodeTypeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<NodeTypeRef> {
        UnsafePointer<NodeTypeRef>(OpaquePointer(__swift_bridge__$Vec_NodeType$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_NodeType$len(vecPtr)
    }
}


public class OutputFormat: OutputFormatRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$OutputFormat$_free(ptr)
        }
    }
}
public class OutputFormatRefMut: OutputFormatRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class OutputFormatRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension OutputFormatRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$OutputFormat$to_string(ptr))
    }
}
extension OutputFormat: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_OutputFormat$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_OutputFormat$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: OutputFormat) {
        __swift_bridge__$Vec_OutputFormat$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_OutputFormat$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (OutputFormat(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<OutputFormatRef> {
        let pointer = __swift_bridge__$Vec_OutputFormat$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return OutputFormatRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<OutputFormatRefMut> {
        let pointer = __swift_bridge__$Vec_OutputFormat$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return OutputFormatRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<OutputFormatRef> {
        UnsafePointer<OutputFormatRef>(OpaquePointer(__swift_bridge__$Vec_OutputFormat$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_OutputFormat$len(vecPtr)
    }
}


public class PreprocessingPreset: PreprocessingPresetRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$PreprocessingPreset$_free(ptr)
        }
    }
}
public class PreprocessingPresetRefMut: PreprocessingPresetRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class PreprocessingPresetRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension PreprocessingPresetRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$PreprocessingPreset$to_string(ptr))
    }
}
extension PreprocessingPreset: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_PreprocessingPreset$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_PreprocessingPreset$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: PreprocessingPreset) {
        __swift_bridge__$Vec_PreprocessingPreset$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_PreprocessingPreset$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (PreprocessingPreset(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingPresetRef> {
        let pointer = __swift_bridge__$Vec_PreprocessingPreset$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingPresetRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PreprocessingPresetRefMut> {
        let pointer = __swift_bridge__$Vec_PreprocessingPreset$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PreprocessingPresetRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<PreprocessingPresetRef> {
        UnsafePointer<PreprocessingPresetRef>(OpaquePointer(__swift_bridge__$Vec_PreprocessingPreset$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_PreprocessingPreset$len(vecPtr)
    }
}


public class StructuredDataType: StructuredDataTypeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$StructuredDataType$_free(ptr)
        }
    }
}
public class StructuredDataTypeRefMut: StructuredDataTypeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class StructuredDataTypeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension StructuredDataTypeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$StructuredDataType$to_string(ptr))
    }
}
extension StructuredDataType: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_StructuredDataType$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_StructuredDataType$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: StructuredDataType) {
        __swift_bridge__$Vec_StructuredDataType$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_StructuredDataType$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (StructuredDataType(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<StructuredDataTypeRef> {
        let pointer = __swift_bridge__$Vec_StructuredDataType$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return StructuredDataTypeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<StructuredDataTypeRefMut> {
        let pointer = __swift_bridge__$Vec_StructuredDataType$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return StructuredDataTypeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<StructuredDataTypeRef> {
        UnsafePointer<StructuredDataTypeRef>(OpaquePointer(__swift_bridge__$Vec_StructuredDataType$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_StructuredDataType$len(vecPtr)
    }
}


public class TextDirection: TextDirectionRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$TextDirection$_free(ptr)
        }
    }
}
public class TextDirectionRefMut: TextDirectionRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class TextDirectionRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension TextDirectionRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$TextDirection$to_string(ptr))
    }
}
extension TextDirection: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_TextDirection$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_TextDirection$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: TextDirection) {
        __swift_bridge__$Vec_TextDirection$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_TextDirection$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (TextDirection(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TextDirectionRef> {
        let pointer = __swift_bridge__$Vec_TextDirection$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TextDirectionRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TextDirectionRefMut> {
        let pointer = __swift_bridge__$Vec_TextDirection$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TextDirectionRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<TextDirectionRef> {
        UnsafePointer<TextDirectionRef>(OpaquePointer(__swift_bridge__$Vec_TextDirection$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_TextDirection$len(vecPtr)
    }
}


public class TierStrategy: TierStrategyRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$TierStrategy$_free(ptr)
        }
    }
}
public class TierStrategyRefMut: TierStrategyRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class TierStrategyRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension TierStrategyRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$TierStrategy$to_string(ptr))
    }
}
extension TierStrategy: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_TierStrategy$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_TierStrategy$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: TierStrategy) {
        __swift_bridge__$Vec_TierStrategy$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_TierStrategy$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (TierStrategy(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TierStrategyRef> {
        let pointer = __swift_bridge__$Vec_TierStrategy$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TierStrategyRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<TierStrategyRefMut> {
        let pointer = __swift_bridge__$Vec_TierStrategy$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return TierStrategyRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<TierStrategyRef> {
        UnsafePointer<TierStrategyRef>(OpaquePointer(__swift_bridge__$Vec_TierStrategy$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_TierStrategy$len(vecPtr)
    }
}


public class UrlEscapeStyle: UrlEscapeStyleRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$UrlEscapeStyle$_free(ptr)
        }
    }
}
public class UrlEscapeStyleRefMut: UrlEscapeStyleRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class UrlEscapeStyleRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension UrlEscapeStyleRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$UrlEscapeStyle$to_string(ptr))
    }
}
extension UrlEscapeStyle: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_UrlEscapeStyle$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_UrlEscapeStyle$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: UrlEscapeStyle) {
        __swift_bridge__$Vec_UrlEscapeStyle$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_UrlEscapeStyle$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (UrlEscapeStyle(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<UrlEscapeStyleRef> {
        let pointer = __swift_bridge__$Vec_UrlEscapeStyle$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return UrlEscapeStyleRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<UrlEscapeStyleRefMut> {
        let pointer = __swift_bridge__$Vec_UrlEscapeStyle$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return UrlEscapeStyleRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UrlEscapeStyleRef> {
        UnsafePointer<UrlEscapeStyleRef>(OpaquePointer(__swift_bridge__$Vec_UrlEscapeStyle$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_UrlEscapeStyle$len(vecPtr)
    }
}


public class WarningKind: WarningKindRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$WarningKind$_free(ptr)
        }
    }
}
public class WarningKindRefMut: WarningKindRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class WarningKindRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension WarningKindRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$WarningKind$to_string(ptr))
    }
}
extension WarningKind: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_WarningKind$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_WarningKind$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: WarningKind) {
        __swift_bridge__$Vec_WarningKind$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_WarningKind$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (WarningKind(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WarningKindRef> {
        let pointer = __swift_bridge__$Vec_WarningKind$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WarningKindRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WarningKindRefMut> {
        let pointer = __swift_bridge__$Vec_WarningKind$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WarningKindRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<WarningKindRef> {
        UnsafePointer<WarningKindRef>(OpaquePointer(__swift_bridge__$Vec_WarningKind$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_WarningKind$len(vecPtr)
    }
}


public class WhitespaceMode: WhitespaceModeRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$WhitespaceMode$_free(ptr)
        }
    }
}
public class WhitespaceModeRefMut: WhitespaceModeRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class WhitespaceModeRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension WhitespaceModeRef {
    public func to_string() -> RustString {
        RustString(ptr: __swift_bridge__$WhitespaceMode$to_string(ptr))
    }
}
extension WhitespaceMode: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_WhitespaceMode$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_WhitespaceMode$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: WhitespaceMode) {
        __swift_bridge__$Vec_WhitespaceMode$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_WhitespaceMode$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (WhitespaceMode(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WhitespaceModeRef> {
        let pointer = __swift_bridge__$Vec_WhitespaceMode$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WhitespaceModeRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<WhitespaceModeRefMut> {
        let pointer = __swift_bridge__$Vec_WhitespaceMode$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return WhitespaceModeRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<WhitespaceModeRef> {
        UnsafePointer<WhitespaceModeRef>(OpaquePointer(__swift_bridge__$Vec_WhitespaceMode$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_WhitespaceMode$len(vecPtr)
    }
}


public class HtmlVisitorBox: HtmlVisitorBoxRefMut {
    public var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$HtmlVisitorBox$_free(ptr)
        }
    }
}
public class HtmlVisitorBoxRefMut: HtmlVisitorBoxRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class HtmlVisitorBoxRef {
    public var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension HtmlVisitorBox: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_HtmlVisitorBox$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_HtmlVisitorBox$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: HtmlVisitorBox) {
        __swift_bridge__$Vec_HtmlVisitorBox$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_HtmlVisitorBox$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (HtmlVisitorBox(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HtmlVisitorBoxRef> {
        let pointer = __swift_bridge__$Vec_HtmlVisitorBox$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HtmlVisitorBoxRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<HtmlVisitorBoxRefMut> {
        let pointer = __swift_bridge__$Vec_HtmlVisitorBox$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return HtmlVisitorBoxRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<HtmlVisitorBoxRef> {
        UnsafePointer<HtmlVisitorBoxRef>(OpaquePointer(__swift_bridge__$Vec_HtmlVisitorBox$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_HtmlVisitorBox$len(vecPtr)
    }
}


@_cdecl("__swift_bridge__$SwiftHtmlVisitorBox$_free")
func __swift_bridge__SwiftHtmlVisitorBox__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<SwiftHtmlVisitorBox>.fromOpaque(ptr).takeRetainedValue()
}
