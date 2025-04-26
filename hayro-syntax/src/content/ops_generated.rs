// THIS FILE IS AUTO-GENERATED, DO NOT EDIT MANUALLY

#[derive(Debug, PartialEq, Clone)]
pub struct BeginCompatibility;
op0!(BeginCompatibility, "BX");

#[derive(Debug, PartialEq, Clone)]
pub struct EndCompatibility;
op0!(EndCompatibility, "EX");

#[derive(Debug, PartialEq, Clone)]
pub struct SaveState;
op0!(SaveState, "q");

#[derive(Debug, PartialEq, Clone)]
pub struct RestoreState;
op0!(RestoreState, "Q");

#[derive(Debug, PartialEq, Clone)]
pub struct Transform(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op6!(Transform, "cm");

#[derive(Debug, PartialEq, Clone)]
pub struct LineWidth(pub Number);
op1!(LineWidth, "w");

#[derive(Debug, PartialEq, Clone)]
pub struct LineCap(pub Number);
op1!(LineCap, "J");

#[derive(Debug, PartialEq, Clone)]
pub struct LineJoin(pub Number);
op1!(LineJoin, "j");

#[derive(Debug, PartialEq, Clone)]
pub struct MiterLimit(pub Number);
op1!(MiterLimit, "M");

#[derive(Debug, PartialEq, Clone)]
pub struct DashPattern<'a>(
    pub Array<'a>,
    pub Number,
);
op2!(DashPattern<'a>, "d");

#[derive(Debug, PartialEq, Clone)]
pub struct RenderingIntent<'a>(pub Name<'a>);
op1!(RenderingIntent<'a>, "ri");

#[derive(Debug, PartialEq, Clone)]
pub struct FlatnessTolerance(pub Number);
op1!(FlatnessTolerance, "i");

#[derive(Debug, PartialEq, Clone)]
pub struct SetGraphicsState<'a>(pub Name<'a>);
op1!(SetGraphicsState<'a>, "gs");

#[derive(Debug, PartialEq, Clone)]
pub struct MoveTo(
    pub Number,
    pub Number,
);
op2!(MoveTo, "m");

#[derive(Debug, PartialEq, Clone)]
pub struct LineTo(
    pub Number,
    pub Number,
);
op2!(LineTo, "l");

#[derive(Debug, PartialEq, Clone)]
pub struct CubicTo(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op6!(CubicTo, "c");

#[derive(Debug, PartialEq, Clone)]
pub struct CubicStartTo(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op4!(CubicStartTo, "v");

#[derive(Debug, PartialEq, Clone)]
pub struct CubicEndTo(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op4!(CubicEndTo, "y");

#[derive(Debug, PartialEq, Clone)]
pub struct ClosePath;
op0!(ClosePath, "h");

#[derive(Debug, PartialEq, Clone)]
pub struct RectPath(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op4!(RectPath, "re");

#[derive(Debug, PartialEq, Clone)]
pub struct StrokePath;
op0!(StrokePath, "S");

#[derive(Debug, PartialEq, Clone)]
pub struct CloseAndStrokePath;
op0!(CloseAndStrokePath, "s");

#[derive(Debug, PartialEq, Clone)]
pub struct FillPathNonZero;
op0!(FillPathNonZero, "f");

#[derive(Debug, PartialEq, Clone)]
pub struct FillPathNonZeroCompatibility;
op0!(FillPathNonZeroCompatibility, "F");

#[derive(Debug, PartialEq, Clone)]
pub struct FillPathEvenOdd;
op0!(FillPathEvenOdd, "f*");

#[derive(Debug, PartialEq, Clone)]
pub struct FillAndStrokeNonZero;
op0!(FillAndStrokeNonZero, "B");

#[derive(Debug, PartialEq, Clone)]
pub struct FillAndStrokeEvenOdd;
op0!(FillAndStrokeEvenOdd, "B*");

#[derive(Debug, PartialEq, Clone)]
pub struct CloseFillAndStrokeNonZero;
op0!(CloseFillAndStrokeNonZero, "b");

#[derive(Debug, PartialEq, Clone)]
pub struct CloseFillAndStrokeEvenOdd;
op0!(CloseFillAndStrokeEvenOdd, "b*");

#[derive(Debug, PartialEq, Clone)]
pub struct EndPath;
op0!(EndPath, "n");

#[derive(Debug, PartialEq, Clone)]
pub struct ClipNonZero;
op0!(ClipNonZero, "W");

#[derive(Debug, PartialEq, Clone)]
pub struct ClipEvenOdd;
op0!(ClipEvenOdd, "W*");

#[derive(Debug, PartialEq, Clone)]
pub struct ColorSpaceStroke<'a>(pub Name<'a>);
op1!(ColorSpaceStroke<'a>, "CS");

#[derive(Debug, PartialEq, Clone)]
pub struct ColorSpaceNonStroke<'a>(pub Name<'a>);
op1!(ColorSpaceNonStroke<'a>, "cs");

#[derive(Debug, PartialEq, Clone)]
pub struct StrokeColor(pub SmallVec<[Number; OPERANDS_THRESHOLD]>);
op_all!(StrokeColor, "SC");

#[derive(Debug, PartialEq, Clone)]
pub struct NonStrokeColor(pub SmallVec<[Number; OPERANDS_THRESHOLD]>);
op_all!(NonStrokeColor, "sc");

#[derive(Debug, PartialEq, Clone)]
pub struct StrokeColorDeviceGray(pub Number);
op1!(StrokeColorDeviceGray, "G");

#[derive(Debug, PartialEq, Clone)]
pub struct NonStrokeColorDeviceGray(pub Number);
op1!(NonStrokeColorDeviceGray, "g");

#[derive(Debug, PartialEq, Clone)]
pub struct StrokeColorDeviceRgb(
    pub Number,
    pub Number,
    pub Number,
);
op3!(StrokeColorDeviceRgb, "RG");

#[derive(Debug, PartialEq, Clone)]
pub struct NonStrokeColorDeviceRgb(
    pub Number,
    pub Number,
    pub Number,
);
op3!(NonStrokeColorDeviceRgb, "rg");

#[derive(Debug, PartialEq, Clone)]
pub struct StrokeColorCmyk(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op4!(StrokeColorCmyk, "K");

#[derive(Debug, PartialEq, Clone)]
pub struct NonStrokeColorCmyk(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op4!(NonStrokeColorCmyk, "k");

#[derive(Debug, PartialEq, Clone)]
pub struct Shading<'a>(pub Name<'a>);
op1!(Shading<'a>, "sh");

#[derive(Debug, PartialEq, Clone)]
pub struct XObject<'a>(pub Name<'a>);
op1!(XObject<'a>, "Do");

#[derive(Debug, PartialEq, Clone)]
pub struct InlineImage<'a>(pub Stream<'a>);
op1!(InlineImage<'a>, "BI");

#[derive(Debug, PartialEq, Clone)]
pub struct CharacterSpacing(pub Number);
op1!(CharacterSpacing, "Tc");

#[derive(Debug, PartialEq, Clone)]
pub struct WordSpacing(pub Number);
op1!(WordSpacing, "Tw");

#[derive(Debug, PartialEq, Clone)]
pub struct HorizontalScaling(pub Number);
op1!(HorizontalScaling, "Tz");

#[derive(Debug, PartialEq, Clone)]
pub struct TextLeading(pub Number);
op1!(TextLeading, "TL");

#[derive(Debug, PartialEq, Clone)]
pub struct TextFont<'a>(
    pub Name<'a>,
    pub Number,
);
op2!(TextFont<'a>, "Tf");

#[derive(Debug, PartialEq, Clone)]
pub struct TextRenderingMode(pub Number);
op1!(TextRenderingMode, "Tr");

#[derive(Debug, PartialEq, Clone)]
pub struct TextRise(pub Number);
op1!(TextRise, "Ts");

#[derive(Debug, PartialEq, Clone)]
pub struct BeginText;
op0!(BeginText, "BT");

#[derive(Debug, PartialEq, Clone)]
pub struct EndText;
op0!(EndText, "ET");

#[derive(Debug, PartialEq, Clone)]
pub struct NextLine(
    pub Number,
    pub Number,
);
op2!(NextLine, "Td");

#[derive(Debug, PartialEq, Clone)]
pub struct NextLineAndSetLeading(
    pub Number,
    pub Number,
);
op2!(NextLineAndSetLeading, "TD");

#[derive(Debug, PartialEq, Clone)]
pub struct SetTextMatrix(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op6!(SetTextMatrix, "Tm");

#[derive(Debug, PartialEq, Clone)]
pub struct NextLineUsingLeading;
op0!(NextLineUsingLeading, "T*");

#[derive(Debug, PartialEq, Clone)]
pub struct ShowText<'a>(pub string::String<'a>);
op1!(ShowText<'a>, "Tj");

#[derive(Debug, PartialEq, Clone)]
pub struct NextLineAndShowText<'a>(pub string::String<'a>);
op1!(NextLineAndShowText<'a>, "'");

#[derive(Debug, PartialEq, Clone)]
pub struct ShowTextWithParameters<'a>(
    pub Number,
    pub Number,
    pub string::String<'a>,
);
op3!(ShowTextWithParameters<'a>, "\"");

#[derive(Debug, PartialEq, Clone)]
pub struct ShowTexts<'a>(pub Array<'a>);
op1!(ShowTexts<'a>, "TJ");

#[derive(Debug, PartialEq, Clone)]
pub struct ColorGlyph(
    pub Number,
    pub Number,
);
op2!(ColorGlyph, "d0");

#[derive(Debug, PartialEq, Clone)]
pub struct ShapeGlyph(
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
    pub Number,
);
op6!(ShapeGlyph, "d1");

#[derive(Debug, PartialEq, Clone)]
pub struct MarkedContentPoint<'a>(pub Name<'a>);
op1!(MarkedContentPoint<'a>, "MP");

#[derive(Debug, PartialEq, Clone)]
pub struct MarkedContentPointWithProperties<'a>(
    pub Name<'a>,
    pub Dict<'a>,
);
op2!(MarkedContentPointWithProperties<'a>, "DP");

#[derive(Debug, PartialEq, Clone)]
pub struct BeginMarkedContent<'a>(pub Name<'a>);
op1!(BeginMarkedContent<'a>, "DP");

#[derive(Debug, PartialEq, Clone)]
pub struct BeginMarkedContentWithProperties<'a>(
    pub Name<'a>,
    pub Dict<'a>,
);
op2!(BeginMarkedContentWithProperties<'a>, "d1");

#[derive(Debug, PartialEq, Clone)]
pub struct EndMarkedContent;
op0!(EndMarkedContent, "DP");

#[derive(Debug, PartialEq, Clone)]
pub enum TypedOperation<'a> {
    BeginCompatibility(BeginCompatibility),
    EndCompatibility(EndCompatibility),
    SaveState(SaveState),
    RestoreState(RestoreState),
    Transform(Transform),
    LineWidth(LineWidth),
    LineCap(LineCap),
    LineJoin(LineJoin),
    MiterLimit(MiterLimit),
    DashPattern(DashPattern<'a>),
    RenderingIntent(RenderingIntent<'a>),
    FlatnessTolerance(FlatnessTolerance),
    SetGraphicsState(SetGraphicsState<'a>),
    MoveTo(MoveTo),
    LineTo(LineTo),
    CubicTo(CubicTo),
    CubicStartTo(CubicStartTo),
    CubicEndTo(CubicEndTo),
    ClosePath(ClosePath),
    RectPath(RectPath),
    StrokePath(StrokePath),
    CloseAndStrokePath(CloseAndStrokePath),
    FillPathNonZero(FillPathNonZero),
    FillPathNonZeroCompatibility(FillPathNonZeroCompatibility),
    FillPathEvenOdd(FillPathEvenOdd),
    FillAndStrokeNonZero(FillAndStrokeNonZero),
    FillAndStrokeEvenOdd(FillAndStrokeEvenOdd),
    CloseFillAndStrokeNonZero(CloseFillAndStrokeNonZero),
    CloseFillAndStrokeEvenOdd(CloseFillAndStrokeEvenOdd),
    EndPath(EndPath),
    ClipNonZero(ClipNonZero),
    ClipEvenOdd(ClipEvenOdd),
    ColorSpaceStroke(ColorSpaceStroke<'a>),
    ColorSpaceNonStroke(ColorSpaceNonStroke<'a>),
    StrokeColor(StrokeColor),
    StrokeColorNamed(StrokeColorNamed<'a>),
    NonStrokeColor(NonStrokeColor),
    NonStrokeColorNamed(NonStrokeColorNamed<'a>),
    StrokeColorDeviceGray(StrokeColorDeviceGray),
    NonStrokeColorDeviceGray(NonStrokeColorDeviceGray),
    StrokeColorDeviceRgb(StrokeColorDeviceRgb),
    NonStrokeColorDeviceRgb(NonStrokeColorDeviceRgb),
    StrokeColorCmyk(StrokeColorCmyk),
    NonStrokeColorCmyk(NonStrokeColorCmyk),
    Shading(Shading<'a>),
    XObject(XObject<'a>),
    InlineImage(InlineImage<'a>),
    CharacterSpacing(CharacterSpacing),
    WordSpacing(WordSpacing),
    HorizontalScaling(HorizontalScaling),
    TextLeading(TextLeading),
    TextFont(TextFont<'a>),
    TextRenderingMode(TextRenderingMode),
    TextRise(TextRise),
    BeginText(BeginText),
    EndText(EndText),
    NextLine(NextLine),
    NextLineAndSetLeading(NextLineAndSetLeading),
    SetTextMatrix(SetTextMatrix),
    NextLineUsingLeading(NextLineUsingLeading),
    ShowText(ShowText<'a>),
    NextLineAndShowText(NextLineAndShowText<'a>),
    ShowTextWithParameters(ShowTextWithParameters<'a>),
    ShowTexts(ShowTexts<'a>),
    ColorGlyph(ColorGlyph),
    ShapeGlyph(ShapeGlyph),
    MarkedContentPoint(MarkedContentPoint<'a>),
    MarkedContentPointWithProperties(MarkedContentPointWithProperties<'a>),
    BeginMarkedContent(BeginMarkedContent<'a>),
    BeginMarkedContentWithProperties(BeginMarkedContentWithProperties<'a>),
    EndMarkedContent(EndMarkedContent),
    Fallback,
}

impl<'a> TypedOperation<'a> {
    pub(crate) fn dispatch(operation: &Operation<'a>) -> Option<TypedOperation<'a>> {
        let op_name = operation.operator.get();
        Some(match op_name.as_ref() {
            b"BX" => BeginCompatibility::from_stack(&operation.operands)?.into(),
            b"EX" => EndCompatibility::from_stack(&operation.operands)?.into(),
            b"q" => SaveState::from_stack(&operation.operands)?.into(),
            b"Q" => RestoreState::from_stack(&operation.operands)?.into(),
            b"cm" => Transform::from_stack(&operation.operands)?.into(),
            b"w" => LineWidth::from_stack(&operation.operands)?.into(),
            b"J" => LineCap::from_stack(&operation.operands)?.into(),
            b"j" => LineJoin::from_stack(&operation.operands)?.into(),
            b"M" => MiterLimit::from_stack(&operation.operands)?.into(),
            b"d" => DashPattern::from_stack(&operation.operands)?.into(),
            b"ri" => RenderingIntent::from_stack(&operation.operands)?.into(),
            b"i" => FlatnessTolerance::from_stack(&operation.operands)?.into(),
            b"gs" => SetGraphicsState::from_stack(&operation.operands)?.into(),
            b"m" => MoveTo::from_stack(&operation.operands)?.into(),
            b"l" => LineTo::from_stack(&operation.operands)?.into(),
            b"c" => CubicTo::from_stack(&operation.operands)?.into(),
            b"v" => CubicStartTo::from_stack(&operation.operands)?.into(),
            b"y" => CubicEndTo::from_stack(&operation.operands)?.into(),
            b"h" => ClosePath::from_stack(&operation.operands)?.into(),
            b"re" => RectPath::from_stack(&operation.operands)?.into(),
            b"S" => StrokePath::from_stack(&operation.operands)?.into(),
            b"s" => CloseAndStrokePath::from_stack(&operation.operands)?.into(),
            b"f" => FillPathNonZero::from_stack(&operation.operands)?.into(),
            b"F" => FillPathNonZeroCompatibility::from_stack(&operation.operands)?.into(),
            b"f*" => FillPathEvenOdd::from_stack(&operation.operands)?.into(),
            b"B" => FillAndStrokeNonZero::from_stack(&operation.operands)?.into(),
            b"B*" => FillAndStrokeEvenOdd::from_stack(&operation.operands)?.into(),
            b"b" => CloseFillAndStrokeNonZero::from_stack(&operation.operands)?.into(),
            b"b*" => CloseFillAndStrokeEvenOdd::from_stack(&operation.operands)?.into(),
            b"n" => EndPath::from_stack(&operation.operands)?.into(),
            b"W" => ClipNonZero::from_stack(&operation.operands)?.into(),
            b"W*" => ClipEvenOdd::from_stack(&operation.operands)?.into(),
            b"CS" => ColorSpaceStroke::from_stack(&operation.operands)?.into(),
            b"cs" => ColorSpaceNonStroke::from_stack(&operation.operands)?.into(),
            b"SC" => StrokeColor::from_stack(&operation.operands)?.into(),
            b"SCN" => StrokeColorNamed::from_stack(&operation.operands)?.into(),
            b"sc" => NonStrokeColor::from_stack(&operation.operands)?.into(),
            b"scn" => NonStrokeColorNamed::from_stack(&operation.operands)?.into(),
            b"G" => StrokeColorDeviceGray::from_stack(&operation.operands)?.into(),
            b"g" => NonStrokeColorDeviceGray::from_stack(&operation.operands)?.into(),
            b"RG" => StrokeColorDeviceRgb::from_stack(&operation.operands)?.into(),
            b"rg" => NonStrokeColorDeviceRgb::from_stack(&operation.operands)?.into(),
            b"K" => StrokeColorCmyk::from_stack(&operation.operands)?.into(),
            b"k" => NonStrokeColorCmyk::from_stack(&operation.operands)?.into(),
            b"sh" => Shading::from_stack(&operation.operands)?.into(),
            b"Do" => XObject::from_stack(&operation.operands)?.into(),
            b"BI" => InlineImage::from_stack(&operation.operands)?.into(),
            b"Tc" => CharacterSpacing::from_stack(&operation.operands)?.into(),
            b"Tw" => WordSpacing::from_stack(&operation.operands)?.into(),
            b"Tz" => HorizontalScaling::from_stack(&operation.operands)?.into(),
            b"TL" => TextLeading::from_stack(&operation.operands)?.into(),
            b"Tf" => TextFont::from_stack(&operation.operands)?.into(),
            b"Tr" => TextRenderingMode::from_stack(&operation.operands)?.into(),
            b"Ts" => TextRise::from_stack(&operation.operands)?.into(),
            b"BT" => BeginText::from_stack(&operation.operands)?.into(),
            b"ET" => EndText::from_stack(&operation.operands)?.into(),
            b"Td" => NextLine::from_stack(&operation.operands)?.into(),
            b"TD" => NextLineAndSetLeading::from_stack(&operation.operands)?.into(),
            b"Tm" => SetTextMatrix::from_stack(&operation.operands)?.into(),
            b"T*" => NextLineUsingLeading::from_stack(&operation.operands)?.into(),
            b"Tj" => ShowText::from_stack(&operation.operands)?.into(),
            b"'" => NextLineAndShowText::from_stack(&operation.operands)?.into(),
            b"\"" => ShowTextWithParameters::from_stack(&operation.operands)?.into(),
            b"TJ" => ShowTexts::from_stack(&operation.operands)?.into(),
            b"d0" => ColorGlyph::from_stack(&operation.operands)?.into(),
            b"d1" => ShapeGlyph::from_stack(&operation.operands)?.into(),
            b"MP" => MarkedContentPoint::from_stack(&operation.operands)?.into(),
            b"DP" => MarkedContentPointWithProperties::from_stack(&operation.operands)?.into(),
            b"DP" => BeginMarkedContent::from_stack(&operation.operands)?.into(),
            b"d1" => BeginMarkedContentWithProperties::from_stack(&operation.operands)?.into(),
            b"DP" => EndMarkedContent::from_stack(&operation.operands)?.into(),
            _ => return Self::Fallback.into(),
        })
    }
}