using System.Runtime.InteropServices;
using System.Text;

namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Internal bridge for marshalling between C# visitor pattern and C FFI.
/// </summary>
internal sealed class VisitorBridge : IDisposable
{
    private static class NativeMethods
    {
        public const string LibraryName = "html_to_markdown_ffi";

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr html_to_markdown_visitor_create(IntPtr visitor);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void html_to_markdown_visitor_free(IntPtr visitor);

        [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr html_to_markdown_last_error();
    }

    private readonly IVisitor _visitor;
    private readonly GCHandle _gcHandle;
    private IntPtr _nativeVisitorHandle = IntPtr.Zero;

    private readonly Dictionary<string, Delegate> _delegateCache = new();

    public VisitorBridge(IVisitor visitor)
    {
        _visitor = visitor ?? throw new ArgumentNullException(nameof(visitor));
        _gcHandle = GCHandle.Alloc(this);
    }

    public IntPtr CreateNativeVisitor()
    {
        var visitTextCb = (NativeVisitorStructures.VisitTextCallback)VisitTextCallback;
        var visitElementStartCb = (NativeVisitorStructures.VisitElementStartCallback)VisitElementStartCallback;
        var visitElementEndCb = (NativeVisitorStructures.VisitElementEndCallback)VisitElementEndCallback;
        var visitLinkCb = (NativeVisitorStructures.VisitLinkCallback)VisitLinkCallback;
        var visitImageCb = (NativeVisitorStructures.VisitImageCallback)VisitImageCallback;
        var visitHeadingCb = (NativeVisitorStructures.VisitHeadingCallback)VisitHeadingCallback;
        var visitCodeBlockCb = (NativeVisitorStructures.VisitCodeBlockCallback)VisitCodeBlockCallback;
        var visitCodeInlineCb = (NativeVisitorStructures.VisitCodeInlineCallback)VisitCodeInlineCallback;
        var visitListItemCb = (NativeVisitorStructures.VisitListItemCallback)VisitListItemCallback;
        var visitListStartCb = (NativeVisitorStructures.VisitListStartCallback)VisitListStartCallback;
        var visitListEndCb = (NativeVisitorStructures.VisitListEndCallback)VisitListEndCallback;
        var visitTableStartCb = (NativeVisitorStructures.VisitTableStartCallback)VisitTableStartCallback;
        var visitTableRowCb = (NativeVisitorStructures.VisitTableRowCallback)VisitTableRowCallback;
        var visitTableEndCb = (NativeVisitorStructures.VisitTableEndCallback)VisitTableEndCallback;
        var visitBlockquoteCb = (NativeVisitorStructures.VisitBlockquoteCallback)VisitBlockquoteCallback;
        var visitStrongCb = (NativeVisitorStructures.VisitStrongCallback)VisitStrongCallback;
        var visitEmphasisCb = (NativeVisitorStructures.VisitEmphasisCallback)VisitEmphasisCallback;
        var visitStrikethroughCb = (NativeVisitorStructures.VisitStrikethroughCallback)VisitStrikethroughCallback;
        var visitUnderlineCb = (NativeVisitorStructures.VisitUnderlineCallback)VisitUnderlineCallback;
        var visitSubscriptCb = (NativeVisitorStructures.VisitSubscriptCallback)VisitSubscriptCallback;
        var visitSuperscriptCb = (NativeVisitorStructures.VisitSuperscriptCallback)VisitSuperscriptCallback;
        var visitMarkCb = (NativeVisitorStructures.VisitMarkCallback)VisitMarkCallback;
        var visitLineBreakCb = (NativeVisitorStructures.VisitLineBreakCallback)VisitLineBreakCallback;
        var visitHorizontalRuleCb = (NativeVisitorStructures.VisitHorizontalRuleCallback)VisitHorizontalRuleCallback;
        var visitCustomElementCb = (NativeVisitorStructures.VisitCustomElementCallback)VisitCustomElementCallback;
        var visitDefinitionListStartCb = (NativeVisitorStructures.VisitDefinitionListStartCallback)VisitDefinitionListStartCallback;
        var visitDefinitionTermCb = (NativeVisitorStructures.VisitDefinitionTermCallback)VisitDefinitionTermCallback;
        var visitDefinitionDescriptionCb = (NativeVisitorStructures.VisitDefinitionDescriptionCallback)VisitDefinitionDescriptionCallback;
        var visitDefinitionListEndCb = (NativeVisitorStructures.VisitDefinitionListEndCallback)VisitDefinitionListEndCallback;
        var visitFormCb = (NativeVisitorStructures.VisitFormCallback)VisitFormCallback;
        var visitInputCb = (NativeVisitorStructures.VisitInputCallback)VisitInputCallback;
        var visitButtonCb = (NativeVisitorStructures.VisitButtonCallback)VisitButtonCallback;
        var visitAudioCb = (NativeVisitorStructures.VisitAudioCallback)VisitAudioCallback;
        var visitVideoCb = (NativeVisitorStructures.VisitVideoCallback)VisitVideoCallback;
        var visitIFrameCb = (NativeVisitorStructures.VisitIFrameCallback)VisitIFrameCallback;
        var visitDetailsCb = (NativeVisitorStructures.VisitDetailsCallback)VisitDetailsCallback;
        var visitSummaryCb = (NativeVisitorStructures.VisitSummaryCallback)VisitSummaryCallback;
        var visitFigureStartCb = (NativeVisitorStructures.VisitFigureStartCallback)VisitFigureStartCallback;
        var visitFigCaptionCb = (NativeVisitorStructures.VisitFigCaptionCallback)VisitFigCaptionCallback;
        var visitFigureEndCb = (NativeVisitorStructures.VisitFigureEndCallback)VisitFigureEndCallback;

        _delegateCache["VisitText"] = visitTextCb;
        _delegateCache["VisitElementStart"] = visitElementStartCb;
        _delegateCache["VisitElementEnd"] = visitElementEndCb;
        _delegateCache["VisitLink"] = visitLinkCb;
        _delegateCache["VisitImage"] = visitImageCb;
        _delegateCache["VisitHeading"] = visitHeadingCb;
        _delegateCache["VisitCodeBlock"] = visitCodeBlockCb;
        _delegateCache["VisitCodeInline"] = visitCodeInlineCb;
        _delegateCache["VisitListItem"] = visitListItemCb;
        _delegateCache["VisitListStart"] = visitListStartCb;
        _delegateCache["VisitListEnd"] = visitListEndCb;
        _delegateCache["VisitTableStart"] = visitTableStartCb;
        _delegateCache["VisitTableRow"] = visitTableRowCb;
        _delegateCache["VisitTableEnd"] = visitTableEndCb;
        _delegateCache["VisitBlockquote"] = visitBlockquoteCb;
        _delegateCache["VisitStrong"] = visitStrongCb;
        _delegateCache["VisitEmphasis"] = visitEmphasisCb;
        _delegateCache["VisitStrikethrough"] = visitStrikethroughCb;
        _delegateCache["VisitUnderline"] = visitUnderlineCb;
        _delegateCache["VisitSubscript"] = visitSubscriptCb;
        _delegateCache["VisitSuperscript"] = visitSuperscriptCb;
        _delegateCache["VisitMark"] = visitMarkCb;
        _delegateCache["VisitLineBreak"] = visitLineBreakCb;
        _delegateCache["VisitHorizontalRule"] = visitHorizontalRuleCb;
        _delegateCache["VisitCustomElement"] = visitCustomElementCb;
        _delegateCache["VisitDefinitionListStart"] = visitDefinitionListStartCb;
        _delegateCache["VisitDefinitionTerm"] = visitDefinitionTermCb;
        _delegateCache["VisitDefinitionDescription"] = visitDefinitionDescriptionCb;
        _delegateCache["VisitDefinitionListEnd"] = visitDefinitionListEndCb;
        _delegateCache["VisitForm"] = visitFormCb;
        _delegateCache["VisitInput"] = visitInputCb;
        _delegateCache["VisitButton"] = visitButtonCb;
        _delegateCache["VisitAudio"] = visitAudioCb;
        _delegateCache["VisitVideo"] = visitVideoCb;
        _delegateCache["VisitIFrame"] = visitIFrameCb;
        _delegateCache["VisitDetails"] = visitDetailsCb;
        _delegateCache["VisitSummary"] = visitSummaryCb;
        _delegateCache["VisitFigureStart"] = visitFigureStartCb;
        _delegateCache["VisitFigCaption"] = visitFigCaptionCb;
        _delegateCache["VisitFigureEnd"] = visitFigureEndCb;

        var nativeVisitor = new NativeVisitorStructures.NativeVisitor
        {
            UserData = GCHandle.ToIntPtr(_gcHandle),
            VisitText = Marshal.GetFunctionPointerForDelegate(visitTextCb),
            VisitElementStart = Marshal.GetFunctionPointerForDelegate(visitElementStartCb),
            VisitElementEnd = Marshal.GetFunctionPointerForDelegate(visitElementEndCb),
            VisitLink = Marshal.GetFunctionPointerForDelegate(visitLinkCb),
            VisitImage = Marshal.GetFunctionPointerForDelegate(visitImageCb),
            VisitHeading = Marshal.GetFunctionPointerForDelegate(visitHeadingCb),
            VisitCodeBlock = Marshal.GetFunctionPointerForDelegate(visitCodeBlockCb),
            VisitCodeInline = Marshal.GetFunctionPointerForDelegate(visitCodeInlineCb),
            VisitListItem = Marshal.GetFunctionPointerForDelegate(visitListItemCb),
            VisitListStart = Marshal.GetFunctionPointerForDelegate(visitListStartCb),
            VisitListEnd = Marshal.GetFunctionPointerForDelegate(visitListEndCb),
            VisitTableStart = Marshal.GetFunctionPointerForDelegate(visitTableStartCb),
            VisitTableRow = Marshal.GetFunctionPointerForDelegate(visitTableRowCb),
            VisitTableEnd = Marshal.GetFunctionPointerForDelegate(visitTableEndCb),
            VisitBlockquote = Marshal.GetFunctionPointerForDelegate(visitBlockquoteCb),
            VisitStrong = Marshal.GetFunctionPointerForDelegate(visitStrongCb),
            VisitEmphasis = Marshal.GetFunctionPointerForDelegate(visitEmphasisCb),
            VisitStrikethrough = Marshal.GetFunctionPointerForDelegate(visitStrikethroughCb),
            VisitUnderline = Marshal.GetFunctionPointerForDelegate(visitUnderlineCb),
            VisitSubscript = Marshal.GetFunctionPointerForDelegate(visitSubscriptCb),
            VisitSuperscript = Marshal.GetFunctionPointerForDelegate(visitSuperscriptCb),
            VisitMark = Marshal.GetFunctionPointerForDelegate(visitMarkCb),
            VisitLineBreak = Marshal.GetFunctionPointerForDelegate(visitLineBreakCb),
            VisitHorizontalRule = Marshal.GetFunctionPointerForDelegate(visitHorizontalRuleCb),
            VisitCustomElement = Marshal.GetFunctionPointerForDelegate(visitCustomElementCb),
            VisitDefinitionListStart = Marshal.GetFunctionPointerForDelegate(visitDefinitionListStartCb),
            VisitDefinitionTerm = Marshal.GetFunctionPointerForDelegate(visitDefinitionTermCb),
            VisitDefinitionDescription = Marshal.GetFunctionPointerForDelegate(visitDefinitionDescriptionCb),
            VisitDefinitionListEnd = Marshal.GetFunctionPointerForDelegate(visitDefinitionListEndCb),
            VisitForm = Marshal.GetFunctionPointerForDelegate(visitFormCb),
            VisitInput = Marshal.GetFunctionPointerForDelegate(visitInputCb),
            VisitButton = Marshal.GetFunctionPointerForDelegate(visitButtonCb),
            VisitAudio = Marshal.GetFunctionPointerForDelegate(visitAudioCb),
            VisitVideo = Marshal.GetFunctionPointerForDelegate(visitVideoCb),
            VisitIFrame = Marshal.GetFunctionPointerForDelegate(visitIFrameCb),
            VisitDetails = Marshal.GetFunctionPointerForDelegate(visitDetailsCb),
            VisitSummary = Marshal.GetFunctionPointerForDelegate(visitSummaryCb),
            VisitFigureStart = Marshal.GetFunctionPointerForDelegate(visitFigureStartCb),
            VisitFigCaption = Marshal.GetFunctionPointerForDelegate(visitFigCaptionCb),
            VisitFigureEnd = Marshal.GetFunctionPointerForDelegate(visitFigureEndCb),
        };

        var nativePtr = Marshal.AllocHGlobal(Marshal.SizeOf<NativeVisitorStructures.NativeVisitor>());
        Marshal.StructureToPtr(nativeVisitor, nativePtr, false);

        try
        {
            _nativeVisitorHandle = NativeMethods.html_to_markdown_visitor_create(nativePtr);
            if (_nativeVisitorHandle == IntPtr.Zero)
            {
                throw new HtmlToMarkdownException("Failed to create native visitor");
            }

            return _nativeVisitorHandle;
        }
        finally
        {
            Marshal.FreeHGlobal(nativePtr);
        }
    }

    public IntPtr GetNativeVisitorHandle() => _nativeVisitorHandle;


    private NodeContext UnmarshalNodeContext(IntPtr ctx)
    {
        var nativeCtx = Marshal.PtrToStructure<NativeVisitorStructures.NativeNodeContext>(ctx);

        var attributes = new List<Attribute>();
        if (nativeCtx.Attributes != IntPtr.Zero)
        {
            int i = 0;
            var attrPtr = nativeCtx.Attributes;
            var attrSize = Marshal.SizeOf<NativeVisitorStructures.NativeAttribute>();

            while (true)
            {
                var attr = Marshal.PtrToStructure<NativeVisitorStructures.NativeAttribute>(attrPtr);
                if (attr.Key == IntPtr.Zero || attr.Value == IntPtr.Zero)
                {
                    break;
                }

                var key = PtrToStringUtf8(attr.Key);
                var value = PtrToStringUtf8(attr.Value);
                attributes.Add(new Attribute(key ?? "", value ?? ""));

                attrPtr = IntPtr.Add(attrPtr, attrSize);
                i++;
            }
        }

        var nodeType = (NodeType)nativeCtx.NodeType;
        var tagName = PtrToStringUtf8(nativeCtx.TagName) ?? "";
        var parentTag = nativeCtx.ParentTag != IntPtr.Zero ? PtrToStringUtf8(nativeCtx.ParentTag) : null;

        return new NodeContext(
            nodeType,
            tagName,
            attributes.AsReadOnly(),
            (int)nativeCtx.Depth,
            (int)nativeCtx.IndexInParent,
            parentTag,
            nativeCtx.IsInline);
    }

    private NativeVisitorStructures.NativeVisitResult MarshalVisitResult(VisitResult result)
    {
        var nativeResult = new NativeVisitorStructures.NativeVisitResult
        {
            ResultType = (NativeVisitorStructures.NativeVisitResultType)result.ResultType,
            CustomOutput = IntPtr.Zero,
            ErrorMessage = IntPtr.Zero
        };

        if (result is CustomResult customResult)
        {
            nativeResult.CustomOutput = StringToUtf8Ptr(customResult.CustomOutput);
        }
        else if (result is ErrorResult errorResult)
        {
            nativeResult.ErrorMessage = StringToUtf8Ptr(errorResult.ErrorMessage);
        }

        return nativeResult;
    }

    private string? PtrToStringUtf8(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUTF8(ptr);
        }
        catch (AccessViolationException)
        {
            return string.Empty;
        }
    }

    private IntPtr StringToUtf8Ptr(string value)
    {
        if (string.IsNullOrEmpty(value))
        {
            return IntPtr.Zero;
        }

        var bytes = Encoding.UTF8.GetBytes(value);
        var ptr = Marshal.AllocHGlobal(bytes.Length + 1);
        Marshal.Copy(bytes, 0, ptr, bytes.Length);
        Marshal.WriteByte(ptr, bytes.Length, 0);
        return ptr;
    }

    private NativeVisitorStructures.NativeVisitResult VisitElementStartCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitElementStart(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitElementStart error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitElementEndCallback(IntPtr userData, IntPtr ctx, IntPtr output)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var outputStr = PtrToStringUtf8(output) ?? "";
            var result = _visitor.VisitElementEnd(context, outputStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitElementEnd error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitTextCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitText(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitText error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitLinkCallback(IntPtr userData, IntPtr ctx, IntPtr href, IntPtr text, IntPtr title)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var hrefStr = PtrToStringUtf8(href) ?? "";
            var textStr = PtrToStringUtf8(text) ?? "";
            var titleStr = title != IntPtr.Zero ? PtrToStringUtf8(title) : null;
            var result = _visitor.VisitLink(context, hrefStr, textStr, titleStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitLink error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitImageCallback(IntPtr userData, IntPtr ctx, IntPtr src, IntPtr alt, IntPtr title)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var srcStr = PtrToStringUtf8(src) ?? "";
            var altStr = PtrToStringUtf8(alt) ?? "";
            var titleStr = title != IntPtr.Zero ? PtrToStringUtf8(title) : null;
            var result = _visitor.VisitImage(context, srcStr, altStr, titleStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitImage error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitHeadingCallback(IntPtr userData, IntPtr ctx, uint level, IntPtr text, IntPtr id)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var idStr = id != IntPtr.Zero ? PtrToStringUtf8(id) : null;
            var result = _visitor.VisitHeading(context, (int)level, textStr, idStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitHeading error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitCodeBlockCallback(IntPtr userData, IntPtr ctx, IntPtr lang, IntPtr code)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var langStr = lang != IntPtr.Zero ? PtrToStringUtf8(lang) : null;
            var codeStr = PtrToStringUtf8(code) ?? "";
            var result = _visitor.VisitCodeBlock(context, langStr, codeStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitCodeBlock error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitCodeInlineCallback(IntPtr userData, IntPtr ctx, IntPtr code)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var codeStr = PtrToStringUtf8(code) ?? "";
            var result = _visitor.VisitCodeInline(context, codeStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitCodeInline error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitListItemCallback(IntPtr userData, IntPtr ctx, bool ordered, IntPtr marker, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var markerStr = PtrToStringUtf8(marker) ?? "";
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitListItem(context, ordered, markerStr, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitListItem error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitListStartCallback(IntPtr userData, IntPtr ctx, bool ordered)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitListStart(context, ordered);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitListStart error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitListEndCallback(IntPtr userData, IntPtr ctx, bool ordered, IntPtr output)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var outputStr = PtrToStringUtf8(output) ?? "";
            var result = _visitor.VisitListEnd(context, ordered, outputStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitListEnd error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitTableStartCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitTableStart(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitTableStart error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitTableRowCallback(IntPtr userData, IntPtr ctx, IntPtr cells, nuint cellCount, bool isHeader)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var cellList = new List<string>();

            for (nuint i = 0; i < cellCount; i++)
            {
                var cellPtrPtr = IntPtr.Add(cells, (int)i * IntPtr.Size);
                var cellPtr = Marshal.ReadIntPtr(cellPtrPtr);
                var cellStr = PtrToStringUtf8(cellPtr) ?? "";
                cellList.Add(cellStr);
            }

            var result = _visitor.VisitTableRow(context, cellList.AsReadOnly(), isHeader);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitTableRow error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitTableEndCallback(IntPtr userData, IntPtr ctx, IntPtr output)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var outputStr = PtrToStringUtf8(output) ?? "";
            var result = _visitor.VisitTableEnd(context, outputStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitTableEnd error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitBlockquoteCallback(IntPtr userData, IntPtr ctx, IntPtr content, nuint depth)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var contentStr = PtrToStringUtf8(content) ?? "";
            var result = _visitor.VisitBlockquote(context, contentStr, (int)depth);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitBlockquote error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitStrongCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitStrong(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitStrong error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitEmphasisCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitEmphasis(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitEmphasis error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitStrikethroughCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitStrikethrough(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitStrikethrough error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitUnderlineCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitUnderline(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitUnderline error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitSubscriptCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitSubscript(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitSubscript error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitSuperscriptCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitSuperscript(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitSuperscript error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitMarkCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitMark(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitMark error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitLineBreakCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitLineBreak(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitLineBreak error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitHorizontalRuleCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitHorizontalRule(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitHorizontalRule error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitCustomElementCallback(IntPtr userData, IntPtr ctx, IntPtr tagName, IntPtr html)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var tagNameStr = PtrToStringUtf8(tagName) ?? "";
            var htmlStr = PtrToStringUtf8(html) ?? "";
            var result = _visitor.VisitCustomElement(context, tagNameStr, htmlStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitCustomElement error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitDefinitionListStartCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitDefinitionListStart(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitDefinitionListStart error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitDefinitionTermCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitDefinitionTerm(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitDefinitionTerm error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitDefinitionDescriptionCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitDefinitionDescription(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitDefinitionDescription error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitDefinitionListEndCallback(IntPtr userData, IntPtr ctx, IntPtr output)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var outputStr = PtrToStringUtf8(output) ?? "";
            var result = _visitor.VisitDefinitionListEnd(context, outputStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitDefinitionListEnd error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitFormCallback(IntPtr userData, IntPtr ctx, IntPtr action, IntPtr method)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var actionStr = action != IntPtr.Zero ? PtrToStringUtf8(action) : null;
            var methodStr = method != IntPtr.Zero ? PtrToStringUtf8(method) : null;
            var result = _visitor.VisitForm(context, actionStr, methodStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitForm error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitInputCallback(IntPtr userData, IntPtr ctx, IntPtr inputType, IntPtr name, IntPtr value)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var inputTypeStr = PtrToStringUtf8(inputType) ?? "";
            var nameStr = name != IntPtr.Zero ? PtrToStringUtf8(name) : null;
            var valueStr = value != IntPtr.Zero ? PtrToStringUtf8(value) : null;
            var result = _visitor.VisitInput(context, inputTypeStr, nameStr, valueStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitInput error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitButtonCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitButton(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitButton error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitAudioCallback(IntPtr userData, IntPtr ctx, IntPtr src)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var srcStr = src != IntPtr.Zero ? PtrToStringUtf8(src) : null;
            var result = _visitor.VisitAudio(context, srcStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitAudio error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitVideoCallback(IntPtr userData, IntPtr ctx, IntPtr src)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var srcStr = src != IntPtr.Zero ? PtrToStringUtf8(src) : null;
            var result = _visitor.VisitVideo(context, srcStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitVideo error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitIFrameCallback(IntPtr userData, IntPtr ctx, IntPtr src)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var srcStr = src != IntPtr.Zero ? PtrToStringUtf8(src) : null;
            var result = _visitor.VisitIFrame(context, srcStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitIFrame error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitDetailsCallback(IntPtr userData, IntPtr ctx, bool open)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitDetails(context, open);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitDetails error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitSummaryCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitSummary(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitSummary error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitFigureStartCallback(IntPtr userData, IntPtr ctx)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var result = _visitor.VisitFigureStart(context);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitFigureStart error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitFigCaptionCallback(IntPtr userData, IntPtr ctx, IntPtr text)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var textStr = PtrToStringUtf8(text) ?? "";
            var result = _visitor.VisitFigCaption(context, textStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitFigCaption error: {ex.Message}"));
        }
    }

    private NativeVisitorStructures.NativeVisitResult VisitFigureEndCallback(IntPtr userData, IntPtr ctx, IntPtr output)
    {
        try
        {
            var context = UnmarshalNodeContext(ctx);
            var outputStr = PtrToStringUtf8(output) ?? "";
            var result = _visitor.VisitFigureEnd(context, outputStr);
            return MarshalVisitResult(result);
        }
        catch (Exception ex)
        {
            return MarshalVisitResult(VisitResult.Error($"VisitFigureEnd error: {ex.Message}"));
        }
    }

    public void Dispose()
    {
        if (_nativeVisitorHandle != IntPtr.Zero)
        {
            NativeMethods.html_to_markdown_visitor_free(_nativeVisitorHandle);
            _nativeVisitorHandle = IntPtr.Zero;
        }

        if (_gcHandle.IsAllocated)
        {
            _gcHandle.Free();
        }
    }
}
