use std::{collections::HashMap};

use serde_json::Value;

use crate::{html_token::{HtmlToken, HtmlTokenType}, lexer::Lexer, parse_error::{ParseError}};
use crate::html_document_parser::HTMLDocumentParser;

struct AttributeBuffer {
    name: String,
    value: String
}

struct NamedCharacterReferenceObject { 
    character_reference: String,
    codepoints: String,
    characters: String
}

pub struct Tokenizer { 
    lexer: Lexer,
    tokenization_state: HTMLTokenizerState,
    pub html_tokens: Vec<HtmlToken>,
    reconsume_current_input_character: bool,
    return_state: HTMLTokenizerState,
    temporary_buffer: String,
    attribute_buffer: AttributeBuffer, 
    named_character_references: Vec<NamedCharacterReferenceObject>,
    number_character_references: HashMap<u32, u32>,
    character_reference_code: u32,
    html_document_parser: HTMLDocumentParser,
    current_html_token: Option<HtmlToken>,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum HTMLTokenizerState { 
    Data,
    RCData,
    RawText,
    ScriptData,
    PlainText,
    TagOpen,
    EndTagOpen,
    TagName,
    RcdataLessThanSign,
    RcdataEndTagOpen,
    RcdataEndTagName,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    CdataSection,
    CdataSectionBracket,
    CdataSectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumerticCharacterReferenceEnd
}

impl Tokenizer { 
    const REPLACEMENT_FEED_CHARACTER: char = '\u{FFFD}';
    const NAMED_CHARACTER_REFERENCE_JSON_DATA: &'static str = r#"
    {
        "&AElig": { "codepoints": [198], "characters": "\u00C6" },
        "&AElig;": { "codepoints": [198], "characters": "\u00C6" },
        "&AMP": { "codepoints": [38], "characters": "\u0026" },
        "&AMP;": { "codepoints": [38], "characters": "\u0026" },
        "&Aacute": { "codepoints": [193], "characters": "\u00C1" },
        "&Aacute;": { "codepoints": [193], "characters": "\u00C1" },
        "&Abreve;": { "codepoints": [258], "characters": "\u0102" },
        "&Acirc": { "codepoints": [194], "characters": "\u00C2" },
        "&Acirc;": { "codepoints": [194], "characters": "\u00C2" },
        "&Acy;": { "codepoints": [1040], "characters": "\u0410" },
        "&Afr;": { "codepoints": [120068], "characters": "\uD835\uDD04" },
        "&Agrave": { "codepoints": [192], "characters": "\u00C0" },
        "&Agrave;": { "codepoints": [192], "characters": "\u00C0" },
        "&Alpha;": { "codepoints": [913], "characters": "\u0391" },
        "&Amacr;": { "codepoints": [256], "characters": "\u0100" },
        "&And;": { "codepoints": [10835], "characters": "\u2A53" },
        "&Aogon;": { "codepoints": [260], "characters": "\u0104" },
        "&Aopf;": { "codepoints": [120120], "characters": "\uD835\uDD38" },
        "&ApplyFunction;": { "codepoints": [8289], "characters": "\u2061" },
        "&Aring": { "codepoints": [197], "characters": "\u00C5" },
        "&Aring;": { "codepoints": [197], "characters": "\u00C5" },
        "&Ascr;": { "codepoints": [119964], "characters": "\uD835\uDC9C" },
        "&Assign;": { "codepoints": [8788], "characters": "\u2254" },
        "&Atilde": { "codepoints": [195], "characters": "\u00C3" },
        "&Atilde;": { "codepoints": [195], "characters": "\u00C3" },
        "&Auml": { "codepoints": [196], "characters": "\u00C4" },
        "&Auml;": { "codepoints": [196], "characters": "\u00C4" },
        "&Backslash;": { "codepoints": [8726], "characters": "\u2216" },
        "&Barv;": { "codepoints": [10983], "characters": "\u2AE7" },
        "&Barwed;": { "codepoints": [8966], "characters": "\u2306" },
        "&Bcy;": { "codepoints": [1041], "characters": "\u0411" },
        "&Because;": { "codepoints": [8757], "characters": "\u2235" },
        "&Bernoullis;": { "codepoints": [8492], "characters": "\u212C" },
        "&Beta;": { "codepoints": [914], "characters": "\u0392" },
        "&Bfr;": { "codepoints": [120069], "characters": "\uD835\uDD05" },
        "&Bopf;": { "codepoints": [120121], "characters": "\uD835\uDD39" },
        "&Breve;": { "codepoints": [728], "characters": "\u02D8" },
        "&Bscr;": { "codepoints": [8492], "characters": "\u212C" },
        "&Bumpeq;": { "codepoints": [8782], "characters": "\u224E" },
        "&CHcy;": { "codepoints": [1063], "characters": "\u0427" },
        "&COPY": { "codepoints": [169], "characters": "\u00A9" },
        "&COPY;": { "codepoints": [169], "characters": "\u00A9" },
        "&Cacute;": { "codepoints": [262], "characters": "\u0106" },
        "&Cap;": { "codepoints": [8914], "characters": "\u22D2" },
        "&CapitalDifferentialD;": { "codepoints": [8517], "characters": "\u2145" },
        "&Cayleys;": { "codepoints": [8493], "characters": "\u212D" },
        "&Ccaron;": { "codepoints": [268], "characters": "\u010C" },
        "&Ccedil": { "codepoints": [199], "characters": "\u00C7" },
        "&Ccedil;": { "codepoints": [199], "characters": "\u00C7" },
        "&Ccirc;": { "codepoints": [264], "characters": "\u0108" },
        "&Cconint;": { "codepoints": [8752], "characters": "\u2230" },
        "&Cdot;": { "codepoints": [266], "characters": "\u010A" },
        "&Cedilla;": { "codepoints": [184], "characters": "\u00B8" },
        "&CenterDot;": { "codepoints": [183], "characters": "\u00B7" },
        "&Cfr;": { "codepoints": [8493], "characters": "\u212D" },
        "&Chi;": { "codepoints": [935], "characters": "\u03A7" },
        "&CircleDot;": { "codepoints": [8857], "characters": "\u2299" },
        "&CircleMinus;": { "codepoints": [8854], "characters": "\u2296" },
        "&CirclePlus;": { "codepoints": [8853], "characters": "\u2295" },
        "&CircleTimes;": { "codepoints": [8855], "characters": "\u2297" },
        "&ClockwiseContourIntegral;": { "codepoints": [8754], "characters": "\u2232" },
        "&CloseCurlyDoubleQuote;": { "codepoints": [8221], "characters": "\u201D" },
        "&CloseCurlyQuote;": { "codepoints": [8217], "characters": "\u2019" },
        "&Colon;": { "codepoints": [8759], "characters": "\u2237" },
        "&Colone;": { "codepoints": [10868], "characters": "\u2A74" },
        "&Congruent;": { "codepoints": [8801], "characters": "\u2261" },
        "&Conint;": { "codepoints": [8751], "characters": "\u222F" },
        "&ContourIntegral;": { "codepoints": [8750], "characters": "\u222E" },
        "&Copf;": { "codepoints": [8450], "characters": "\u2102" },
        "&Coproduct;": { "codepoints": [8720], "characters": "\u2210" },
        "&CounterClockwiseContourIntegral;": { "codepoints": [8755], "characters": "\u2233" },
        "&Cross;": { "codepoints": [10799], "characters": "\u2A2F" },
        "&Cscr;": { "codepoints": [119966], "characters": "\uD835\uDC9E" },
        "&Cup;": { "codepoints": [8915], "characters": "\u22D3" },
        "&CupCap;": { "codepoints": [8781], "characters": "\u224D" },
        "&DD;": { "codepoints": [8517], "characters": "\u2145" },
        "&DDotrahd;": { "codepoints": [10513], "characters": "\u2911" },
        "&DJcy;": { "codepoints": [1026], "characters": "\u0402" },
        "&DScy;": { "codepoints": [1029], "characters": "\u0405" },
        "&DZcy;": { "codepoints": [1039], "characters": "\u040F" },
        "&Dagger;": { "codepoints": [8225], "characters": "\u2021" },
        "&Darr;": { "codepoints": [8609], "characters": "\u21A1" },
        "&Dashv;": { "codepoints": [10980], "characters": "\u2AE4" },
        "&Dcaron;": { "codepoints": [270], "characters": "\u010E" },
        "&Dcy;": { "codepoints": [1044], "characters": "\u0414" },
        "&Del;": { "codepoints": [8711], "characters": "\u2207" },
        "&Delta;": { "codepoints": [916], "characters": "\u0394" },
        "&Dfr;": { "codepoints": [120071], "characters": "\uD835\uDD07" },
        "&DiacriticalAcute;": { "codepoints": [180], "characters": "\u00B4" },
        "&DiacriticalDot;": { "codepoints": [729], "characters": "\u02D9" },
        "&DiacriticalDoubleAcute;": { "codepoints": [733], "characters": "\u02DD" },
        "&DiacriticalGrave;": { "codepoints": [96], "characters": "\u0060" },
        "&DiacriticalTilde;": { "codepoints": [732], "characters": "\u02DC" },
        "&Diamond;": { "codepoints": [8900], "characters": "\u22C4" },
        "&DifferentialD;": { "codepoints": [8518], "characters": "\u2146" },
        "&Dopf;": { "codepoints": [120123], "characters": "\uD835\uDD3B" },
        "&Dot;": { "codepoints": [168], "characters": "\u00A8" },
        "&DotDot;": { "codepoints": [8412], "characters": "\u20DC" },
        "&DotEqual;": { "codepoints": [8784], "characters": "\u2250" },
        "&DoubleContourIntegral;": { "codepoints": [8751], "characters": "\u222F" },
        "&DoubleDot;": { "codepoints": [168], "characters": "\u00A8" },
        "&DoubleDownArrow;": { "codepoints": [8659], "characters": "\u21D3" },
        "&DoubleLeftArrow;": { "codepoints": [8656], "characters": "\u21D0" },
        "&DoubleLeftRightArrow;": { "codepoints": [8660], "characters": "\u21D4" },
        "&DoubleLeftTee;": { "codepoints": [10980], "characters": "\u2AE4" },
        "&DoubleLongLeftArrow;": { "codepoints": [10232], "characters": "\u27F8" },
        "&DoubleLongLeftRightArrow;": { "codepoints": [10234], "characters": "\u27FA" },
        "&DoubleLongRightArrow;": { "codepoints": [10233], "characters": "\u27F9" },
        "&DoubleRightArrow;": { "codepoints": [8658], "characters": "\u21D2" },
        "&DoubleRightTee;": { "codepoints": [8872], "characters": "\u22A8" },
        "&DoubleUpArrow;": { "codepoints": [8657], "characters": "\u21D1" },
        "&DoubleUpDownArrow;": { "codepoints": [8661], "characters": "\u21D5" },
        "&DoubleVerticalBar;": { "codepoints": [8741], "characters": "\u2225" },
        "&DownArrow;": { "codepoints": [8595], "characters": "\u2193" },
        "&DownArrowBar;": { "codepoints": [10515], "characters": "\u2913" },
        "&DownArrowUpArrow;": { "codepoints": [8693], "characters": "\u21F5" },
        "&DownBreve;": { "codepoints": [785], "characters": "\u0311" },
        "&DownLeftRightVector;": { "codepoints": [10576], "characters": "\u2950" },
        "&DownLeftTeeVector;": { "codepoints": [10590], "characters": "\u295E" },
        "&DownLeftVector;": { "codepoints": [8637], "characters": "\u21BD" },
        "&DownLeftVectorBar;": { "codepoints": [10582], "characters": "\u2956" },
        "&DownRightTeeVector;": { "codepoints": [10591], "characters": "\u295F" },
        "&DownRightVector;": { "codepoints": [8641], "characters": "\u21C1" },
        "&DownRightVectorBar;": { "codepoints": [10583], "characters": "\u2957" },
        "&DownTee;": { "codepoints": [8868], "characters": "\u22A4" },
        "&DownTeeArrow;": { "codepoints": [8615], "characters": "\u21A7" },
        "&Downarrow;": { "codepoints": [8659], "characters": "\u21D3" },
        "&Dscr;": { "codepoints": [119967], "characters": "\uD835\uDC9F" },
        "&Dstrok;": { "codepoints": [272], "characters": "\u0110" },
        "&ENG;": { "codepoints": [330], "characters": "\u014A" },
        "&ETH": { "codepoints": [208], "characters": "\u00D0" },
        "&ETH;": { "codepoints": [208], "characters": "\u00D0" },
        "&Eacute": { "codepoints": [201], "characters": "\u00C9" },
        "&Eacute;": { "codepoints": [201], "characters": "\u00C9" },
        "&Ecaron;": { "codepoints": [282], "characters": "\u011A" },
        "&Ecirc": { "codepoints": [202], "characters": "\u00CA" },
        "&Ecirc;": { "codepoints": [202], "characters": "\u00CA" },
        "&Ecy;": { "codepoints": [1069], "characters": "\u042D" },
        "&Edot;": { "codepoints": [278], "characters": "\u0116" },
        "&Efr;": { "codepoints": [120072], "characters": "\uD835\uDD08" },
        "&Egrave": { "codepoints": [200], "characters": "\u00C8" },
        "&Egrave;": { "codepoints": [200], "characters": "\u00C8" },
        "&Element;": { "codepoints": [8712], "characters": "\u2208" },
        "&Emacr;": { "codepoints": [274], "characters": "\u0112" },
        "&EmptySmallSquare;": { "codepoints": [9723], "characters": "\u25FB" },
        "&EmptyVerySmallSquare;": { "codepoints": [9643], "characters": "\u25AB" },
        "&Eogon;": { "codepoints": [280], "characters": "\u0118" },
        "&Eopf;": { "codepoints": [120124], "characters": "\uD835\uDD3C" },
        "&Epsilon;": { "codepoints": [917], "characters": "\u0395" },
        "&Equal;": { "codepoints": [10869], "characters": "\u2A75" },
        "&EqualTilde;": { "codepoints": [8770], "characters": "\u2242" },
        "&Equilibrium;": { "codepoints": [8652], "characters": "\u21CC" },
        "&Escr;": { "codepoints": [8496], "characters": "\u2130" },
        "&Esim;": { "codepoints": [10867], "characters": "\u2A73" },
        "&Eta;": { "codepoints": [919], "characters": "\u0397" },
        "&Euml": { "codepoints": [203], "characters": "\u00CB" },
        "&Euml;": { "codepoints": [203], "characters": "\u00CB" },
        "&Exists;": { "codepoints": [8707], "characters": "\u2203" },
        "&ExponentialE;": { "codepoints": [8519], "characters": "\u2147" },
        "&Fcy;": { "codepoints": [1060], "characters": "\u0424" },
        "&Ffr;": { "codepoints": [120073], "characters": "\uD835\uDD09" },
        "&FilledSmallSquare;": { "codepoints": [9724], "characters": "\u25FC" },
        "&FilledVerySmallSquare;": { "codepoints": [9642], "characters": "\u25AA" },
        "&Fopf;": { "codepoints": [120125], "characters": "\uD835\uDD3D" },
        "&ForAll;": { "codepoints": [8704], "characters": "\u2200" },
        "&Fouriertrf;": { "codepoints": [8497], "characters": "\u2131" },
        "&Fscr;": { "codepoints": [8497], "characters": "\u2131" },
        "&GJcy;": { "codepoints": [1027], "characters": "\u0403" },
        "&GT": { "codepoints": [62], "characters": "\u003E" },
        "&GT;": { "codepoints": [62], "characters": "\u003E" },
        "&Gamma;": { "codepoints": [915], "characters": "\u0393" },
        "&Gammad;": { "codepoints": [988], "characters": "\u03DC" },
        "&Gbreve;": { "codepoints": [286], "characters": "\u011E" },
        "&Gcedil;": { "codepoints": [290], "characters": "\u0122" },
        "&Gcirc;": { "codepoints": [284], "characters": "\u011C" },
        "&Gcy;": { "codepoints": [1043], "characters": "\u0413" },
        "&Gdot;": { "codepoints": [288], "characters": "\u0120" },
        "&Gfr;": { "codepoints": [120074], "characters": "\uD835\uDD0A" },
        "&Gg;": { "codepoints": [8921], "characters": "\u22D9" },
        "&Gopf;": { "codepoints": [120126], "characters": "\uD835\uDD3E" },
        "&GreaterEqual;": { "codepoints": [8805], "characters": "\u2265" },
        "&GreaterEqualLess;": { "codepoints": [8923], "characters": "\u22DB" },
        "&GreaterFullEqual;": { "codepoints": [8807], "characters": "\u2267" },
        "&GreaterGreater;": { "codepoints": [10914], "characters": "\u2AA2" },
        "&GreaterLess;": { "codepoints": [8823], "characters": "\u2277" },
        "&GreaterSlantEqual;": { "codepoints": [10878], "characters": "\u2A7E" },
        "&GreaterTilde;": { "codepoints": [8819], "characters": "\u2273" },
        "&Gscr;": { "codepoints": [119970], "characters": "\uD835\uDCA2" },
        "&Gt;": { "codepoints": [8811], "characters": "\u226B" },
        "&HARDcy;": { "codepoints": [1066], "characters": "\u042A" },
        "&Hacek;": { "codepoints": [711], "characters": "\u02C7" },
        "&Hat;": { "codepoints": [94], "characters": "\u005E" },
        "&Hcirc;": { "codepoints": [292], "characters": "\u0124" },
        "&Hfr;": { "codepoints": [8460], "characters": "\u210C" },
        "&HilbertSpace;": { "codepoints": [8459], "characters": "\u210B" },
        "&Hopf;": { "codepoints": [8461], "characters": "\u210D" },
        "&HorizontalLine;": { "codepoints": [9472], "characters": "\u2500" },
        "&Hscr;": { "codepoints": [8459], "characters": "\u210B" },
        "&Hstrok;": { "codepoints": [294], "characters": "\u0126" },
        "&HumpDownHump;": { "codepoints": [8782], "characters": "\u224E" },
        "&HumpEqual;": { "codepoints": [8783], "characters": "\u224F" },
        "&IEcy;": { "codepoints": [1045], "characters": "\u0415" },
        "&IJlig;": { "codepoints": [306], "characters": "\u0132" },
        "&IOcy;": { "codepoints": [1025], "characters": "\u0401" },
        "&Iacute": { "codepoints": [205], "characters": "\u00CD" },
        "&Iacute;": { "codepoints": [205], "characters": "\u00CD" },
        "&Icirc": { "codepoints": [206], "characters": "\u00CE" },
        "&Icirc;": { "codepoints": [206], "characters": "\u00CE" },
        "&Icy;": { "codepoints": [1048], "characters": "\u0418" },
        "&Idot;": { "codepoints": [304], "characters": "\u0130" },
        "&Ifr;": { "codepoints": [8465], "characters": "\u2111" },
        "&Igrave": { "codepoints": [204], "characters": "\u00CC" },
        "&Igrave;": { "codepoints": [204], "characters": "\u00CC" },
        "&Im;": { "codepoints": [8465], "characters": "\u2111" },
        "&Imacr;": { "codepoints": [298], "characters": "\u012A" },
        "&ImaginaryI;": { "codepoints": [8520], "characters": "\u2148" },
        "&Implies;": { "codepoints": [8658], "characters": "\u21D2" },
        "&Int;": { "codepoints": [8748], "characters": "\u222C" },
        "&Integral;": { "codepoints": [8747], "characters": "\u222B" },
        "&Intersection;": { "codepoints": [8898], "characters": "\u22C2" },
        "&InvisibleComma;": { "codepoints": [8291], "characters": "\u2063" },
        "&InvisibleTimes;": { "codepoints": [8290], "characters": "\u2062" },
        "&Iogon;": { "codepoints": [302], "characters": "\u012E" },
        "&Iopf;": { "codepoints": [120128], "characters": "\uD835\uDD40" },
        "&Iota;": { "codepoints": [921], "characters": "\u0399" },
        "&Iscr;": { "codepoints": [8464], "characters": "\u2110" },
        "&Itilde;": { "codepoints": [296], "characters": "\u0128" },
        "&Iukcy;": { "codepoints": [1030], "characters": "\u0406" },
        "&Iuml": { "codepoints": [207], "characters": "\u00CF" },
        "&Iuml;": { "codepoints": [207], "characters": "\u00CF" },
        "&Jcirc;": { "codepoints": [308], "characters": "\u0134" },
        "&Jcy;": { "codepoints": [1049], "characters": "\u0419" },
        "&Jfr;": { "codepoints": [120077], "characters": "\uD835\uDD0D" },
        "&Jopf;": { "codepoints": [120129], "characters": "\uD835\uDD41" },
        "&Jscr;": { "codepoints": [119973], "characters": "\uD835\uDCA5" },
        "&Jsercy;": { "codepoints": [1032], "characters": "\u0408" },
        "&Jukcy;": { "codepoints": [1028], "characters": "\u0404" },
        "&KHcy;": { "codepoints": [1061], "characters": "\u0425" },
        "&KJcy;": { "codepoints": [1036], "characters": "\u040C" },
        "&Kappa;": { "codepoints": [922], "characters": "\u039A" },
        "&Kcedil;": { "codepoints": [310], "characters": "\u0136" },
        "&Kcy;": { "codepoints": [1050], "characters": "\u041A" },
        "&Kfr;": { "codepoints": [120078], "characters": "\uD835\uDD0E" },
        "&Kopf;": { "codepoints": [120130], "characters": "\uD835\uDD42" },
        "&Kscr;": { "codepoints": [119974], "characters": "\uD835\uDCA6" },
        "&LJcy;": { "codepoints": [1033], "characters": "\u0409" },
        "&LT": { "codepoints": [60], "characters": "\u003C" },
        "&LT;": { "codepoints": [60], "characters": "\u003C" },
        "&Lacute;": { "codepoints": [313], "characters": "\u0139" },
        "&Lambda;": { "codepoints": [923], "characters": "\u039B" },
        "&Lang;": { "codepoints": [10218], "characters": "\u27EA" },
        "&Laplacetrf;": { "codepoints": [8466], "characters": "\u2112" },
        "&Larr;": { "codepoints": [8606], "characters": "\u219E" },
        "&Lcaron;": { "codepoints": [317], "characters": "\u013D" },
        "&Lcedil;": { "codepoints": [315], "characters": "\u013B" },
        "&Lcy;": { "codepoints": [1051], "characters": "\u041B" },
        "&LeftAngleBracket;": { "codepoints": [10216], "characters": "\u27E8" },
        "&LeftArrow;": { "codepoints": [8592], "characters": "\u2190" },
        "&LeftArrowBar;": { "codepoints": [8676], "characters": "\u21E4" },
        "&LeftArrowRightArrow;": { "codepoints": [8646], "characters": "\u21C6" },
        "&LeftCeiling;": { "codepoints": [8968], "characters": "\u2308" },
        "&LeftDoubleBracket;": { "codepoints": [10214], "characters": "\u27E6" },
        "&LeftDownTeeVector;": { "codepoints": [10593], "characters": "\u2961" },
        "&LeftDownVector;": { "codepoints": [8643], "characters": "\u21C3" },
        "&LeftDownVectorBar;": { "codepoints": [10585], "characters": "\u2959" },
        "&LeftFloor;": { "codepoints": [8970], "characters": "\u230A" },
        "&LeftRightArrow;": { "codepoints": [8596], "characters": "\u2194" },
        "&LeftRightVector;": { "codepoints": [10574], "characters": "\u294E" },
        "&LeftTee;": { "codepoints": [8867], "characters": "\u22A3" },
        "&LeftTeeArrow;": { "codepoints": [8612], "characters": "\u21A4" },
        "&LeftTeeVector;": { "codepoints": [10586], "characters": "\u295A" },
        "&LeftTriangle;": { "codepoints": [8882], "characters": "\u22B2" },
        "&LeftTriangleBar;": { "codepoints": [10703], "characters": "\u29CF" },
        "&LeftTriangleEqual;": { "codepoints": [8884], "characters": "\u22B4" },
        "&LeftUpDownVector;": { "codepoints": [10577], "characters": "\u2951" },
        "&LeftUpTeeVector;": { "codepoints": [10592], "characters": "\u2960" },
        "&LeftUpVector;": { "codepoints": [8639], "characters": "\u21BF" },
        "&LeftUpVectorBar;": { "codepoints": [10584], "characters": "\u2958" },
        "&LeftVector;": { "codepoints": [8636], "characters": "\u21BC" },
        "&LeftVectorBar;": { "codepoints": [10578], "characters": "\u2952" },
        "&Leftarrow;": { "codepoints": [8656], "characters": "\u21D0" },
        "&Leftrightarrow;": { "codepoints": [8660], "characters": "\u21D4" },
        "&LessEqualGreater;": { "codepoints": [8922], "characters": "\u22DA" },
        "&LessFullEqual;": { "codepoints": [8806], "characters": "\u2266" },
        "&LessGreater;": { "codepoints": [8822], "characters": "\u2276" },
        "&LessLess;": { "codepoints": [10913], "characters": "\u2AA1" },
        "&LessSlantEqual;": { "codepoints": [10877], "characters": "\u2A7D" },
        "&LessTilde;": { "codepoints": [8818], "characters": "\u2272" },
        "&Lfr;": { "codepoints": [120079], "characters": "\uD835\uDD0F" },
        "&Ll;": { "codepoints": [8920], "characters": "\u22D8" },
        "&Lleftarrow;": { "codepoints": [8666], "characters": "\u21DA" },
        "&Lmidot;": { "codepoints": [319], "characters": "\u013F" },
        "&LongLeftArrow;": { "codepoints": [10229], "characters": "\u27F5" },
        "&LongLeftRightArrow;": { "codepoints": [10231], "characters": "\u27F7" },
        "&LongRightArrow;": { "codepoints": [10230], "characters": "\u27F6" },
        "&Longleftarrow;": { "codepoints": [10232], "characters": "\u27F8" },
        "&Longleftrightarrow;": { "codepoints": [10234], "characters": "\u27FA" },
        "&Longrightarrow;": { "codepoints": [10233], "characters": "\u27F9" },
        "&Lopf;": { "codepoints": [120131], "characters": "\uD835\uDD43" },
        "&LowerLeftArrow;": { "codepoints": [8601], "characters": "\u2199" },
        "&LowerRightArrow;": { "codepoints": [8600], "characters": "\u2198" },
        "&Lscr;": { "codepoints": [8466], "characters": "\u2112" },
        "&Lsh;": { "codepoints": [8624], "characters": "\u21B0" },
        "&Lstrok;": { "codepoints": [321], "characters": "\u0141" },
        "&Lt;": { "codepoints": [8810], "characters": "\u226A" },
        "&Map;": { "codepoints": [10501], "characters": "\u2905" },
        "&Mcy;": { "codepoints": [1052], "characters": "\u041C" },
        "&MediumSpace;": { "codepoints": [8287], "characters": "\u205F" },
        "&Mellintrf;": { "codepoints": [8499], "characters": "\u2133" },
        "&Mfr;": { "codepoints": [120080], "characters": "\uD835\uDD10" },
        "&MinusPlus;": { "codepoints": [8723], "characters": "\u2213" },
        "&Mopf;": { "codepoints": [120132], "characters": "\uD835\uDD44" },
        "&Mscr;": { "codepoints": [8499], "characters": "\u2133" },
        "&Mu;": { "codepoints": [924], "characters": "\u039C" },
        "&NJcy;": { "codepoints": [1034], "characters": "\u040A" },
        "&Nacute;": { "codepoints": [323], "characters": "\u0143" },
        "&Ncaron;": { "codepoints": [327], "characters": "\u0147" },
        "&Ncedil;": { "codepoints": [325], "characters": "\u0145" },
        "&Ncy;": { "codepoints": [1053], "characters": "\u041D" },
        "&NegativeMediumSpace;": { "codepoints": [8203], "characters": "\u200B" },
        "&NegativeThickSpace;": { "codepoints": [8203], "characters": "\u200B" },
        "&NegativeThinSpace;": { "codepoints": [8203], "characters": "\u200B" },
        "&NegativeVeryThinSpace;": { "codepoints": [8203], "characters": "\u200B" },
        "&NestedGreaterGreater;": { "codepoints": [8811], "characters": "\u226B" },
        "&NestedLessLess;": { "codepoints": [8810], "characters": "\u226A" },
        "&NewLine;": { "codepoints": [10], "characters": "\u000A" },
        "&Nfr;": { "codepoints": [120081], "characters": "\uD835\uDD11" },
        "&NoBreak;": { "codepoints": [8288], "characters": "\u2060" },
        "&NonBreakingSpace;": { "codepoints": [160], "characters": "\u00A0" },
        "&Nopf;": { "codepoints": [8469], "characters": "\u2115" },
        "&Not;": { "codepoints": [10988], "characters": "\u2AEC" },
        "&NotCongruent;": { "codepoints": [8802], "characters": "\u2262" },
        "&NotCupCap;": { "codepoints": [8813], "characters": "\u226D" },
        "&NotDoubleVerticalBar;": { "codepoints": [8742], "characters": "\u2226" },
        "&NotElement;": { "codepoints": [8713], "characters": "\u2209" },
        "&NotEqual;": { "codepoints": [8800], "characters": "\u2260" },
        "&NotEqualTilde;": { "codepoints": [8770, 824], "characters": "\u2242\u0338" },
        "&NotExists;": { "codepoints": [8708], "characters": "\u2204" },
        "&NotGreater;": { "codepoints": [8815], "characters": "\u226F" },
        "&NotGreaterEqual;": { "codepoints": [8817], "characters": "\u2271" },
        "&NotGreaterFullEqual;": { "codepoints": [8807, 824], "characters": "\u2267\u0338" },
        "&NotGreaterGreater;": { "codepoints": [8811, 824], "characters": "\u226B\u0338" },
        "&NotGreaterLess;": { "codepoints": [8825], "characters": "\u2279" },
        "&NotGreaterSlantEqual;": { "codepoints": [10878, 824], "characters": "\u2A7E\u0338" },
        "&NotGreaterTilde;": { "codepoints": [8821], "characters": "\u2275" },
        "&NotHumpDownHump;": { "codepoints": [8782, 824], "characters": "\u224E\u0338" },
        "&NotHumpEqual;": { "codepoints": [8783, 824], "characters": "\u224F\u0338" },
        "&NotLeftTriangle;": { "codepoints": [8938], "characters": "\u22EA" },
        "&NotLeftTriangleBar;": { "codepoints": [10703, 824], "characters": "\u29CF\u0338" },
        "&NotLeftTriangleEqual;": { "codepoints": [8940], "characters": "\u22EC" },
        "&NotLess;": { "codepoints": [8814], "characters": "\u226E" },
        "&NotLessEqual;": { "codepoints": [8816], "characters": "\u2270" },
        "&NotLessGreater;": { "codepoints": [8824], "characters": "\u2278" },
        "&NotLessLess;": { "codepoints": [8810, 824], "characters": "\u226A\u0338" },
        "&NotLessSlantEqual;": { "codepoints": [10877, 824], "characters": "\u2A7D\u0338" },
        "&NotLessTilde;": { "codepoints": [8820], "characters": "\u2274" },
        "&NotNestedGreaterGreater;": { "codepoints": [10914, 824], "characters": "\u2AA2\u0338" },
        "&NotNestedLessLess;": { "codepoints": [10913, 824], "characters": "\u2AA1\u0338" },
        "&NotPrecedes;": { "codepoints": [8832], "characters": "\u2280" },
        "&NotPrecedesEqual;": { "codepoints": [10927, 824], "characters": "\u2AAF\u0338" },
        "&NotPrecedesSlantEqual;": { "codepoints": [8928], "characters": "\u22E0" },
        "&NotReverseElement;": { "codepoints": [8716], "characters": "\u220C" },
        "&NotRightTriangle;": { "codepoints": [8939], "characters": "\u22EB" },
        "&NotRightTriangleBar;": { "codepoints": [10704, 824], "characters": "\u29D0\u0338" },
        "&NotRightTriangleEqual;": { "codepoints": [8941], "characters": "\u22ED" },
        "&NotSquareSubset;": { "codepoints": [8847, 824], "characters": "\u228F\u0338" },
        "&NotSquareSubsetEqual;": { "codepoints": [8930], "characters": "\u22E2" },
        "&NotSquareSuperset;": { "codepoints": [8848, 824], "characters": "\u2290\u0338" },
        "&NotSquareSupersetEqual;": { "codepoints": [8931], "characters": "\u22E3" },
        "&NotSubset;": { "codepoints": [8834, 8402], "characters": "\u2282\u20D2" },
        "&NotSubsetEqual;": { "codepoints": [8840], "characters": "\u2288" },
        "&NotSucceeds;": { "codepoints": [8833], "characters": "\u2281" },
        "&NotSucceedsEqual;": { "codepoints": [10928, 824], "characters": "\u2AB0\u0338" },
        "&NotSucceedsSlantEqual;": { "codepoints": [8929], "characters": "\u22E1" },
        "&NotSucceedsTilde;": { "codepoints": [8831, 824], "characters": "\u227F\u0338" },
        "&NotSuperset;": { "codepoints": [8835, 8402], "characters": "\u2283\u20D2" },
        "&NotSupersetEqual;": { "codepoints": [8841], "characters": "\u2289" },
        "&NotTilde;": { "codepoints": [8769], "characters": "\u2241" },
        "&NotTildeEqual;": { "codepoints": [8772], "characters": "\u2244" },
        "&NotTildeFullEqual;": { "codepoints": [8775], "characters": "\u2247" },
        "&NotTildeTilde;": { "codepoints": [8777], "characters": "\u2249" },
        "&NotVerticalBar;": { "codepoints": [8740], "characters": "\u2224" },
        "&Nscr;": { "codepoints": [119977], "characters": "\uD835\uDCA9" },
        "&Ntilde": { "codepoints": [209], "characters": "\u00D1" },
        "&Ntilde;": { "codepoints": [209], "characters": "\u00D1" },
        "&Nu;": { "codepoints": [925], "characters": "\u039D" },
        "&OElig;": { "codepoints": [338], "characters": "\u0152" },
        "&Oacute": { "codepoints": [211], "characters": "\u00D3" },
        "&Oacute;": { "codepoints": [211], "characters": "\u00D3" },
        "&Ocirc": { "codepoints": [212], "characters": "\u00D4" },
        "&Ocirc;": { "codepoints": [212], "characters": "\u00D4" },
        "&Ocy;": { "codepoints": [1054], "characters": "\u041E" },
        "&Odblac;": { "codepoints": [336], "characters": "\u0150" },
        "&Ofr;": { "codepoints": [120082], "characters": "\uD835\uDD12" },
        "&Ograve": { "codepoints": [210], "characters": "\u00D2" },
        "&Ograve;": { "codepoints": [210], "characters": "\u00D2" },
        "&Omacr;": { "codepoints": [332], "characters": "\u014C" },
        "&Omega;": { "codepoints": [937], "characters": "\u03A9" },
        "&Omicron;": { "codepoints": [927], "characters": "\u039F" },
        "&Oopf;": { "codepoints": [120134], "characters": "\uD835\uDD46" },
        "&OpenCurlyDoubleQuote;": { "codepoints": [8220], "characters": "\u201C" },
        "&OpenCurlyQuote;": { "codepoints": [8216], "characters": "\u2018" },
        "&Or;": { "codepoints": [10836], "characters": "\u2A54" },
        "&Oscr;": { "codepoints": [119978], "characters": "\uD835\uDCAA" },
        "&Oslash": { "codepoints": [216], "characters": "\u00D8" },
        "&Oslash;": { "codepoints": [216], "characters": "\u00D8" },
        "&Otilde": { "codepoints": [213], "characters": "\u00D5" },
        "&Otilde;": { "codepoints": [213], "characters": "\u00D5" },
        "&Otimes;": { "codepoints": [10807], "characters": "\u2A37" },
        "&Ouml": { "codepoints": [214], "characters": "\u00D6" },
        "&Ouml;": { "codepoints": [214], "characters": "\u00D6" },
        "&OverBar;": { "codepoints": [8254], "characters": "\u203E" },
        "&OverBrace;": { "codepoints": [9182], "characters": "\u23DE" },
        "&OverBracket;": { "codepoints": [9140], "characters": "\u23B4" },
        "&OverParenthesis;": { "codepoints": [9180], "characters": "\u23DC" },
        "&PartialD;": { "codepoints": [8706], "characters": "\u2202" },
        "&Pcy;": { "codepoints": [1055], "characters": "\u041F" },
        "&Pfr;": { "codepoints": [120083], "characters": "\uD835\uDD13" },
        "&Phi;": { "codepoints": [934], "characters": "\u03A6" },
        "&Pi;": { "codepoints": [928], "characters": "\u03A0" },
        "&PlusMinus;": { "codepoints": [177], "characters": "\u00B1" },
        "&Poincareplane;": { "codepoints": [8460], "characters": "\u210C" },
        "&Popf;": { "codepoints": [8473], "characters": "\u2119" },
        "&Pr;": { "codepoints": [10939], "characters": "\u2ABB" },
        "&Precedes;": { "codepoints": [8826], "characters": "\u227A" },
        "&PrecedesEqual;": { "codepoints": [10927], "characters": "\u2AAF" },
        "&PrecedesSlantEqual;": { "codepoints": [8828], "characters": "\u227C" },
        "&PrecedesTilde;": { "codepoints": [8830], "characters": "\u227E" },
        "&Prime;": { "codepoints": [8243], "characters": "\u2033" },
        "&Product;": { "codepoints": [8719], "characters": "\u220F" },
        "&Proportion;": { "codepoints": [8759], "characters": "\u2237" },
        "&Proportional;": { "codepoints": [8733], "characters": "\u221D" },
        "&Pscr;": { "codepoints": [119979], "characters": "\uD835\uDCAB" },
        "&Psi;": { "codepoints": [936], "characters": "\u03A8" },
        "&QUOT": { "codepoints": [34], "characters": "\u0022" },
        "&QUOT;": { "codepoints": [34], "characters": "\u0022" },
        "&Qfr;": { "codepoints": [120084], "characters": "\uD835\uDD14" },
        "&Qopf;": { "codepoints": [8474], "characters": "\u211A" },
        "&Qscr;": { "codepoints": [119980], "characters": "\uD835\uDCAC" },
        "&RBarr;": { "codepoints": [10512], "characters": "\u2910" },
        "&REG": { "codepoints": [174], "characters": "\u00AE" },
        "&REG;": { "codepoints": [174], "characters": "\u00AE" },
        "&Racute;": { "codepoints": [340], "characters": "\u0154" },
        "&Rang;": { "codepoints": [10219], "characters": "\u27EB" },
        "&Rarr;": { "codepoints": [8608], "characters": "\u21A0" },
        "&Rarrtl;": { "codepoints": [10518], "characters": "\u2916" },
        "&Rcaron;": { "codepoints": [344], "characters": "\u0158" },
        "&Rcedil;": { "codepoints": [342], "characters": "\u0156" },
        "&Rcy;": { "codepoints": [1056], "characters": "\u0420" },
        "&Re;": { "codepoints": [8476], "characters": "\u211C" },
        "&ReverseElement;": { "codepoints": [8715], "characters": "\u220B" },
        "&ReverseEquilibrium;": { "codepoints": [8651], "characters": "\u21CB" },
        "&ReverseUpEquilibrium;": { "codepoints": [10607], "characters": "\u296F" },
        "&Rfr;": { "codepoints": [8476], "characters": "\u211C" },
        "&Rho;": { "codepoints": [929], "characters": "\u03A1" },
        "&RightAngleBracket;": { "codepoints": [10217], "characters": "\u27E9" },
        "&RightArrow;": { "codepoints": [8594], "characters": "\u2192" },
        "&RightArrowBar;": { "codepoints": [8677], "characters": "\u21E5" },
        "&RightArrowLeftArrow;": { "codepoints": [8644], "characters": "\u21C4" },
        "&RightCeiling;": { "codepoints": [8969], "characters": "\u2309" },
        "&RightDoubleBracket;": { "codepoints": [10215], "characters": "\u27E7" },
        "&RightDownTeeVector;": { "codepoints": [10589], "characters": "\u295D" },
        "&RightDownVector;": { "codepoints": [8642], "characters": "\u21C2" },
        "&RightDownVectorBar;": { "codepoints": [10581], "characters": "\u2955" },
        "&RightFloor;": { "codepoints": [8971], "characters": "\u230B" },
        "&RightTee;": { "codepoints": [8866], "characters": "\u22A2" },
        "&RightTeeArrow;": { "codepoints": [8614], "characters": "\u21A6" },
        "&RightTeeVector;": { "codepoints": [10587], "characters": "\u295B" },
        "&RightTriangle;": { "codepoints": [8883], "characters": "\u22B3" },
        "&RightTriangleBar;": { "codepoints": [10704], "characters": "\u29D0" },
        "&RightTriangleEqual;": { "codepoints": [8885], "characters": "\u22B5" },
        "&RightUpDownVector;": { "codepoints": [10575], "characters": "\u294F" },
        "&RightUpTeeVector;": { "codepoints": [10588], "characters": "\u295C" },
        "&RightUpVector;": { "codepoints": [8638], "characters": "\u21BE" },
        "&RightUpVectorBar;": { "codepoints": [10580], "characters": "\u2954" },
        "&RightVector;": { "codepoints": [8640], "characters": "\u21C0" },
        "&RightVectorBar;": { "codepoints": [10579], "characters": "\u2953" },
        "&Rightarrow;": { "codepoints": [8658], "characters": "\u21D2" },
        "&Ropf;": { "codepoints": [8477], "characters": "\u211D" },
        "&RoundImplies;": { "codepoints": [10608], "characters": "\u2970" },
        "&Rrightarrow;": { "codepoints": [8667], "characters": "\u21DB" },
        "&Rscr;": { "codepoints": [8475], "characters": "\u211B" },
        "&Rsh;": { "codepoints": [8625], "characters": "\u21B1" },
        "&RuleDelayed;": { "codepoints": [10740], "characters": "\u29F4" },
        "&SHCHcy;": { "codepoints": [1065], "characters": "\u0429" },
        "&SHcy;": { "codepoints": [1064], "characters": "\u0428" },
        "&SOFTcy;": { "codepoints": [1068], "characters": "\u042C" },
        "&Sacute;": { "codepoints": [346], "characters": "\u015A" },
        "&Sc;": { "codepoints": [10940], "characters": "\u2ABC" },
        "&Scaron;": { "codepoints": [352], "characters": "\u0160" },
        "&Scedil;": { "codepoints": [350], "characters": "\u015E" },
        "&Scirc;": { "codepoints": [348], "characters": "\u015C" },
        "&Scy;": { "codepoints": [1057], "characters": "\u0421" },
        "&Sfr;": { "codepoints": [120086], "characters": "\uD835\uDD16" },
        "&ShortDownArrow;": { "codepoints": [8595], "characters": "\u2193" },
        "&ShortLeftArrow;": { "codepoints": [8592], "characters": "\u2190" },
        "&ShortRightArrow;": { "codepoints": [8594], "characters": "\u2192" },
        "&ShortUpArrow;": { "codepoints": [8593], "characters": "\u2191" },
        "&Sigma;": { "codepoints": [931], "characters": "\u03A3" },
        "&SmallCircle;": { "codepoints": [8728], "characters": "\u2218" },
        "&Sopf;": { "codepoints": [120138], "characters": "\uD835\uDD4A" },
        "&Sqrt;": { "codepoints": [8730], "characters": "\u221A" },
        "&Square;": { "codepoints": [9633], "characters": "\u25A1" },
        "&SquareIntersection;": { "codepoints": [8851], "characters": "\u2293" },
        "&SquareSubset;": { "codepoints": [8847], "characters": "\u228F" },
        "&SquareSubsetEqual;": { "codepoints": [8849], "characters": "\u2291" },
        "&SquareSuperset;": { "codepoints": [8848], "characters": "\u2290" },
        "&SquareSupersetEqual;": { "codepoints": [8850], "characters": "\u2292" },
        "&SquareUnion;": { "codepoints": [8852], "characters": "\u2294" },
        "&Sscr;": { "codepoints": [119982], "characters": "\uD835\uDCAE" },
        "&Star;": { "codepoints": [8902], "characters": "\u22C6" },
        "&Sub;": { "codepoints": [8912], "characters": "\u22D0" },
        "&Subset;": { "codepoints": [8912], "characters": "\u22D0" },
        "&SubsetEqual;": { "codepoints": [8838], "characters": "\u2286" },
        "&Succeeds;": { "codepoints": [8827], "characters": "\u227B" },
        "&SucceedsEqual;": { "codepoints": [10928], "characters": "\u2AB0" },
        "&SucceedsSlantEqual;": { "codepoints": [8829], "characters": "\u227D" },
        "&SucceedsTilde;": { "codepoints": [8831], "characters": "\u227F" },
        "&SuchThat;": { "codepoints": [8715], "characters": "\u220B" },
        "&Sum;": { "codepoints": [8721], "characters": "\u2211" },
        "&Sup;": { "codepoints": [8913], "characters": "\u22D1" },
        "&Superset;": { "codepoints": [8835], "characters": "\u2283" },
        "&SupersetEqual;": { "codepoints": [8839], "characters": "\u2287" },
        "&Supset;": { "codepoints": [8913], "characters": "\u22D1" },
        "&THORN": { "codepoints": [222], "characters": "\u00DE" },
        "&THORN;": { "codepoints": [222], "characters": "\u00DE" },
        "&TRADE;": { "codepoints": [8482], "characters": "\u2122" },
        "&TSHcy;": { "codepoints": [1035], "characters": "\u040B" },
        "&TScy;": { "codepoints": [1062], "characters": "\u0426" },
        "&Tab;": { "codepoints": [9], "characters": "\u0009" },
        "&Tau;": { "codepoints": [932], "characters": "\u03A4" },
        "&Tcaron;": { "codepoints": [356], "characters": "\u0164" },
        "&Tcedil;": { "codepoints": [354], "characters": "\u0162" },
        "&Tcy;": { "codepoints": [1058], "characters": "\u0422" },
        "&Tfr;": { "codepoints": [120087], "characters": "\uD835\uDD17" },
        "&Therefore;": { "codepoints": [8756], "characters": "\u2234" },
        "&Theta;": { "codepoints": [920], "characters": "\u0398" },
        "&ThickSpace;": { "codepoints": [8287, 8202], "characters": "\u205F\u200A" },
        "&ThinSpace;": { "codepoints": [8201], "characters": "\u2009" },
        "&Tilde;": { "codepoints": [8764], "characters": "\u223C" },
        "&TildeEqual;": { "codepoints": [8771], "characters": "\u2243" },
        "&TildeFullEqual;": { "codepoints": [8773], "characters": "\u2245" },
        "&TildeTilde;": { "codepoints": [8776], "characters": "\u2248" },
        "&Topf;": { "codepoints": [120139], "characters": "\uD835\uDD4B" },
        "&TripleDot;": { "codepoints": [8411], "characters": "\u20DB" },
        "&Tscr;": { "codepoints": [119983], "characters": "\uD835\uDCAF" },
        "&Tstrok;": { "codepoints": [358], "characters": "\u0166" },
        "&Uacute": { "codepoints": [218], "characters": "\u00DA" },
        "&Uacute;": { "codepoints": [218], "characters": "\u00DA" },
        "&Uarr;": { "codepoints": [8607], "characters": "\u219F" },
        "&Uarrocir;": { "codepoints": [10569], "characters": "\u2949" },
        "&Ubrcy;": { "codepoints": [1038], "characters": "\u040E" },
        "&Ubreve;": { "codepoints": [364], "characters": "\u016C" },
        "&Ucirc": { "codepoints": [219], "characters": "\u00DB" },
        "&Ucirc;": { "codepoints": [219], "characters": "\u00DB" },
        "&Ucy;": { "codepoints": [1059], "characters": "\u0423" },
        "&Udblac;": { "codepoints": [368], "characters": "\u0170" },
        "&Ufr;": { "codepoints": [120088], "characters": "\uD835\uDD18" },
        "&Ugrave": { "codepoints": [217], "characters": "\u00D9" },
        "&Ugrave;": { "codepoints": [217], "characters": "\u00D9" },
        "&Umacr;": { "codepoints": [362], "characters": "\u016A" },
        "&UnderBar;": { "codepoints": [95], "characters": "\u005F" },
        "&UnderBrace;": { "codepoints": [9183], "characters": "\u23DF" },
        "&UnderBracket;": { "codepoints": [9141], "characters": "\u23B5" },
        "&UnderParenthesis;": { "codepoints": [9181], "characters": "\u23DD" },
        "&Union;": { "codepoints": [8899], "characters": "\u22C3" },
        "&UnionPlus;": { "codepoints": [8846], "characters": "\u228E" },
        "&Uogon;": { "codepoints": [370], "characters": "\u0172" },
        "&Uopf;": { "codepoints": [120140], "characters": "\uD835\uDD4C" },
        "&UpArrow;": { "codepoints": [8593], "characters": "\u2191" },
        "&UpArrowBar;": { "codepoints": [10514], "characters": "\u2912" },
        "&UpArrowDownArrow;": { "codepoints": [8645], "characters": "\u21C5" },
        "&UpDownArrow;": { "codepoints": [8597], "characters": "\u2195" },
        "&UpEquilibrium;": { "codepoints": [10606], "characters": "\u296E" },
        "&UpTee;": { "codepoints": [8869], "characters": "\u22A5" },
        "&UpTeeArrow;": { "codepoints": [8613], "characters": "\u21A5" },
        "&Uparrow;": { "codepoints": [8657], "characters": "\u21D1" },
        "&Updownarrow;": { "codepoints": [8661], "characters": "\u21D5" },
        "&UpperLeftArrow;": { "codepoints": [8598], "characters": "\u2196" },
        "&UpperRightArrow;": { "codepoints": [8599], "characters": "\u2197" },
        "&Upsi;": { "codepoints": [978], "characters": "\u03D2" },
        "&Upsilon;": { "codepoints": [933], "characters": "\u03A5" },
        "&Uring;": { "codepoints": [366], "characters": "\u016E" },
        "&Uscr;": { "codepoints": [119984], "characters": "\uD835\uDCB0" },
        "&Utilde;": { "codepoints": [360], "characters": "\u0168" },
        "&Uuml": { "codepoints": [220], "characters": "\u00DC" },
        "&Uuml;": { "codepoints": [220], "characters": "\u00DC" },
        "&VDash;": { "codepoints": [8875], "characters": "\u22AB" },
        "&Vbar;": { "codepoints": [10987], "characters": "\u2AEB" },
        "&Vcy;": { "codepoints": [1042], "characters": "\u0412" },
        "&Vdash;": { "codepoints": [8873], "characters": "\u22A9" },
        "&Vdashl;": { "codepoints": [10982], "characters": "\u2AE6" },
        "&Vee;": { "codepoints": [8897], "characters": "\u22C1" },
        "&Verbar;": { "codepoints": [8214], "characters": "\u2016" },
        "&Vert;": { "codepoints": [8214], "characters": "\u2016" },
        "&VerticalBar;": { "codepoints": [8739], "characters": "\u2223" },
        "&VerticalLine;": { "codepoints": [124], "characters": "\u007C" },
        "&VerticalSeparator;": { "codepoints": [10072], "characters": "\u2758" },
        "&VerticalTilde;": { "codepoints": [8768], "characters": "\u2240" },
        "&VeryThinSpace;": { "codepoints": [8202], "characters": "\u200A" },
        "&Vfr;": { "codepoints": [120089], "characters": "\uD835\uDD19" },
        "&Vopf;": { "codepoints": [120141], "characters": "\uD835\uDD4D" },
        "&Vscr;": { "codepoints": [119985], "characters": "\uD835\uDCB1" },
        "&Vvdash;": { "codepoints": [8874], "characters": "\u22AA" },
        "&Wcirc;": { "codepoints": [372], "characters": "\u0174" },
        "&Wedge;": { "codepoints": [8896], "characters": "\u22C0" },
        "&Wfr;": { "codepoints": [120090], "characters": "\uD835\uDD1A" },
        "&Wopf;": { "codepoints": [120142], "characters": "\uD835\uDD4E" },
        "&Wscr;": { "codepoints": [119986], "characters": "\uD835\uDCB2" },
        "&Xfr;": { "codepoints": [120091], "characters": "\uD835\uDD1B" },
        "&Xi;": { "codepoints": [926], "characters": "\u039E" },
        "&Xopf;": { "codepoints": [120143], "characters": "\uD835\uDD4F" },
        "&Xscr;": { "codepoints": [119987], "characters": "\uD835\uDCB3" },
        "&YAcy;": { "codepoints": [1071], "characters": "\u042F" },
        "&YIcy;": { "codepoints": [1031], "characters": "\u0407" },
        "&YUcy;": { "codepoints": [1070], "characters": "\u042E" },
        "&Yacute": { "codepoints": [221], "characters": "\u00DD" },
        "&Yacute;": { "codepoints": [221], "characters": "\u00DD" },
        "&Ycirc;": { "codepoints": [374], "characters": "\u0176" },
        "&Ycy;": { "codepoints": [1067], "characters": "\u042B" },
        "&Yfr;": { "codepoints": [120092], "characters": "\uD835\uDD1C" },
        "&Yopf;": { "codepoints": [120144], "characters": "\uD835\uDD50" },
        "&Yscr;": { "codepoints": [119988], "characters": "\uD835\uDCB4" },
        "&Yuml;": { "codepoints": [376], "characters": "\u0178" },
        "&ZHcy;": { "codepoints": [1046], "characters": "\u0416" },
        "&Zacute;": { "codepoints": [377], "characters": "\u0179" },
        "&Zcaron;": { "codepoints": [381], "characters": "\u017D" },
        "&Zcy;": { "codepoints": [1047], "characters": "\u0417" },
        "&Zdot;": { "codepoints": [379], "characters": "\u017B" },
        "&ZeroWidthSpace;": { "codepoints": [8203], "characters": "\u200B" },
        "&Zeta;": { "codepoints": [918], "characters": "\u0396" },
        "&Zfr;": { "codepoints": [8488], "characters": "\u2128" },
        "&Zopf;": { "codepoints": [8484], "characters": "\u2124" },
        "&Zscr;": { "codepoints": [119989], "characters": "\uD835\uDCB5" },
        "&aacute": { "codepoints": [225], "characters": "\u00E1" },
        "&aacute;": { "codepoints": [225], "characters": "\u00E1" },
        "&abreve;": { "codepoints": [259], "characters": "\u0103" },
        "&ac;": { "codepoints": [8766], "characters": "\u223E" },
        "&acE;": { "codepoints": [8766, 819], "characters": "\u223E\u0333" },
        "&acd;": { "codepoints": [8767], "characters": "\u223F" },
        "&acirc": { "codepoints": [226], "characters": "\u00E2" },
        "&acirc;": { "codepoints": [226], "characters": "\u00E2" },
        "&acute": { "codepoints": [180], "characters": "\u00B4" },
        "&acute;": { "codepoints": [180], "characters": "\u00B4" },
        "&acy;": { "codepoints": [1072], "characters": "\u0430" },
        "&aelig": { "codepoints": [230], "characters": "\u00E6" },
        "&aelig;": { "codepoints": [230], "characters": "\u00E6" },
        "&af;": { "codepoints": [8289], "characters": "\u2061" },
        "&afr;": { "codepoints": [120094], "characters": "\uD835\uDD1E" },
        "&agrave": { "codepoints": [224], "characters": "\u00E0" },
        "&agrave;": { "codepoints": [224], "characters": "\u00E0" },
        "&alefsym;": { "codepoints": [8501], "characters": "\u2135" },
        "&aleph;": { "codepoints": [8501], "characters": "\u2135" },
        "&alpha;": { "codepoints": [945], "characters": "\u03B1" },
        "&amacr;": { "codepoints": [257], "characters": "\u0101" },
        "&amalg;": { "codepoints": [10815], "characters": "\u2A3F" },
        "&amp": { "codepoints": [38], "characters": "\u0026" },
        "&amp;": { "codepoints": [38], "characters": "\u0026" },
        "&and;": { "codepoints": [8743], "characters": "\u2227" },
        "&andand;": { "codepoints": [10837], "characters": "\u2A55" },
        "&andd;": { "codepoints": [10844], "characters": "\u2A5C" },
        "&andslope;": { "codepoints": [10840], "characters": "\u2A58" },
        "&andv;": { "codepoints": [10842], "characters": "\u2A5A" },
        "&ang;": { "codepoints": [8736], "characters": "\u2220" },
        "&ange;": { "codepoints": [10660], "characters": "\u29A4" },
        "&angle;": { "codepoints": [8736], "characters": "\u2220" },
        "&angmsd;": { "codepoints": [8737], "characters": "\u2221" },
        "&angmsdaa;": { "codepoints": [10664], "characters": "\u29A8" },
        "&angmsdab;": { "codepoints": [10665], "characters": "\u29A9" },
        "&angmsdac;": { "codepoints": [10666], "characters": "\u29AA" },
        "&angmsdad;": { "codepoints": [10667], "characters": "\u29AB" },
        "&angmsdae;": { "codepoints": [10668], "characters": "\u29AC" },
        "&angmsdaf;": { "codepoints": [10669], "characters": "\u29AD" },
        "&angmsdag;": { "codepoints": [10670], "characters": "\u29AE" },
        "&angmsdah;": { "codepoints": [10671], "characters": "\u29AF" },
        "&angrt;": { "codepoints": [8735], "characters": "\u221F" },
        "&angrtvb;": { "codepoints": [8894], "characters": "\u22BE" },
        "&angrtvbd;": { "codepoints": [10653], "characters": "\u299D" },
        "&angsph;": { "codepoints": [8738], "characters": "\u2222" },
        "&angst;": { "codepoints": [197], "characters": "\u00C5" },
        "&angzarr;": { "codepoints": [9084], "characters": "\u237C" },
        "&aogon;": { "codepoints": [261], "characters": "\u0105" },
        "&aopf;": { "codepoints": [120146], "characters": "\uD835\uDD52" },
        "&ap;": { "codepoints": [8776], "characters": "\u2248" },
        "&apE;": { "codepoints": [10864], "characters": "\u2A70" },
        "&apacir;": { "codepoints": [10863], "characters": "\u2A6F" },
        "&ape;": { "codepoints": [8778], "characters": "\u224A" },
        "&apid;": { "codepoints": [8779], "characters": "\u224B" },
        "&apos;": { "codepoints": [39], "characters": "\u0027" },
        "&approx;": { "codepoints": [8776], "characters": "\u2248" },
        "&approxeq;": { "codepoints": [8778], "characters": "\u224A" },
        "&aring": { "codepoints": [229], "characters": "\u00E5" },
        "&aring;": { "codepoints": [229], "characters": "\u00E5" },
        "&ascr;": { "codepoints": [119990], "characters": "\uD835\uDCB6" },
        "&ast;": { "codepoints": [42], "characters": "\u002A" },
        "&asymp;": { "codepoints": [8776], "characters": "\u2248" },
        "&asympeq;": { "codepoints": [8781], "characters": "\u224D" },
        "&atilde": { "codepoints": [227], "characters": "\u00E3" },
        "&atilde;": { "codepoints": [227], "characters": "\u00E3" },
        "&auml": { "codepoints": [228], "characters": "\u00E4" },
        "&auml;": { "codepoints": [228], "characters": "\u00E4" },
        "&awconint;": { "codepoints": [8755], "characters": "\u2233" },
        "&awint;": { "codepoints": [10769], "characters": "\u2A11" },
        "&bNot;": { "codepoints": [10989], "characters": "\u2AED" },
        "&backcong;": { "codepoints": [8780], "characters": "\u224C" },
        "&backepsilon;": { "codepoints": [1014], "characters": "\u03F6" },
        "&backprime;": { "codepoints": [8245], "characters": "\u2035" },
        "&backsim;": { "codepoints": [8765], "characters": "\u223D" },
        "&backsimeq;": { "codepoints": [8909], "characters": "\u22CD" },
        "&barvee;": { "codepoints": [8893], "characters": "\u22BD" },
        "&barwed;": { "codepoints": [8965], "characters": "\u2305" },
        "&barwedge;": { "codepoints": [8965], "characters": "\u2305" },
        "&bbrk;": { "codepoints": [9141], "characters": "\u23B5" },
        "&bbrktbrk;": { "codepoints": [9142], "characters": "\u23B6" },
        "&bcong;": { "codepoints": [8780], "characters": "\u224C" },
        "&bcy;": { "codepoints": [1073], "characters": "\u0431" },
        "&bdquo;": { "codepoints": [8222], "characters": "\u201E" },
        "&becaus;": { "codepoints": [8757], "characters": "\u2235" },
        "&because;": { "codepoints": [8757], "characters": "\u2235" },
        "&bemptyv;": { "codepoints": [10672], "characters": "\u29B0" },
        "&bepsi;": { "codepoints": [1014], "characters": "\u03F6" },
        "&bernou;": { "codepoints": [8492], "characters": "\u212C" },
        "&beta;": { "codepoints": [946], "characters": "\u03B2" },
        "&beth;": { "codepoints": [8502], "characters": "\u2136" },
        "&between;": { "codepoints": [8812], "characters": "\u226C" },
        "&bfr;": { "codepoints": [120095], "characters": "\uD835\uDD1F" },
        "&bigcap;": { "codepoints": [8898], "characters": "\u22C2" },
        "&bigcirc;": { "codepoints": [9711], "characters": "\u25EF" },
        "&bigcup;": { "codepoints": [8899], "characters": "\u22C3" },
        "&bigodot;": { "codepoints": [10752], "characters": "\u2A00" },
        "&bigoplus;": { "codepoints": [10753], "characters": "\u2A01" },
        "&bigotimes;": { "codepoints": [10754], "characters": "\u2A02" },
        "&bigsqcup;": { "codepoints": [10758], "characters": "\u2A06" },
        "&bigstar;": { "codepoints": [9733], "characters": "\u2605" },
        "&bigtriangledown;": { "codepoints": [9661], "characters": "\u25BD" },
        "&bigtriangleup;": { "codepoints": [9651], "characters": "\u25B3" },
        "&biguplus;": { "codepoints": [10756], "characters": "\u2A04" },
        "&bigvee;": { "codepoints": [8897], "characters": "\u22C1" },
        "&bigwedge;": { "codepoints": [8896], "characters": "\u22C0" },
        "&bkarow;": { "codepoints": [10509], "characters": "\u290D" },
        "&blacklozenge;": { "codepoints": [10731], "characters": "\u29EB" },
        "&blacksquare;": { "codepoints": [9642], "characters": "\u25AA" },
        "&blacktriangle;": { "codepoints": [9652], "characters": "\u25B4" },
        "&blacktriangledown;": { "codepoints": [9662], "characters": "\u25BE" },
        "&blacktriangleleft;": { "codepoints": [9666], "characters": "\u25C2" },
        "&blacktriangleright;": { "codepoints": [9656], "characters": "\u25B8" },
        "&blank;": { "codepoints": [9251], "characters": "\u2423" },
        "&blk12;": { "codepoints": [9618], "characters": "\u2592" },
        "&blk14;": { "codepoints": [9617], "characters": "\u2591" },
        "&blk34;": { "codepoints": [9619], "characters": "\u2593" },
        "&block;": { "codepoints": [9608], "characters": "\u2588" },
        "&bne;": { "codepoints": [61, 8421], "characters": "\u003D\u20E5" },
        "&bnequiv;": { "codepoints": [8801, 8421], "characters": "\u2261\u20E5" },
        "&bnot;": { "codepoints": [8976], "characters": "\u2310" },
        "&bopf;": { "codepoints": [120147], "characters": "\uD835\uDD53" },
        "&bot;": { "codepoints": [8869], "characters": "\u22A5" },
        "&bottom;": { "codepoints": [8869], "characters": "\u22A5" },
        "&bowtie;": { "codepoints": [8904], "characters": "\u22C8" },
        "&boxDL;": { "codepoints": [9559], "characters": "\u2557" },
        "&boxDR;": { "codepoints": [9556], "characters": "\u2554" },
        "&boxDl;": { "codepoints": [9558], "characters": "\u2556" },
        "&boxDr;": { "codepoints": [9555], "characters": "\u2553" },
        "&boxH;": { "codepoints": [9552], "characters": "\u2550" },
        "&boxHD;": { "codepoints": [9574], "characters": "\u2566" },
        "&boxHU;": { "codepoints": [9577], "characters": "\u2569" },
        "&boxHd;": { "codepoints": [9572], "characters": "\u2564" },
        "&boxHu;": { "codepoints": [9575], "characters": "\u2567" },
        "&boxUL;": { "codepoints": [9565], "characters": "\u255D" },
        "&boxUR;": { "codepoints": [9562], "characters": "\u255A" },
        "&boxUl;": { "codepoints": [9564], "characters": "\u255C" },
        "&boxUr;": { "codepoints": [9561], "characters": "\u2559" },
        "&boxV;": { "codepoints": [9553], "characters": "\u2551" },
        "&boxVH;": { "codepoints": [9580], "characters": "\u256C" },
        "&boxVL;": { "codepoints": [9571], "characters": "\u2563" },
        "&boxVR;": { "codepoints": [9568], "characters": "\u2560" },
        "&boxVh;": { "codepoints": [9579], "characters": "\u256B" },
        "&boxVl;": { "codepoints": [9570], "characters": "\u2562" },
        "&boxVr;": { "codepoints": [9567], "characters": "\u255F" },
        "&boxbox;": { "codepoints": [10697], "characters": "\u29C9" },
        "&boxdL;": { "codepoints": [9557], "characters": "\u2555" },
        "&boxdR;": { "codepoints": [9554], "characters": "\u2552" },
        "&boxdl;": { "codepoints": [9488], "characters": "\u2510" },
        "&boxdr;": { "codepoints": [9484], "characters": "\u250C" },
        "&boxh;": { "codepoints": [9472], "characters": "\u2500" },
        "&boxhD;": { "codepoints": [9573], "characters": "\u2565" },
        "&boxhU;": { "codepoints": [9576], "characters": "\u2568" },
        "&boxhd;": { "codepoints": [9516], "characters": "\u252C" },
        "&boxhu;": { "codepoints": [9524], "characters": "\u2534" },
        "&boxminus;": { "codepoints": [8863], "characters": "\u229F" },
        "&boxplus;": { "codepoints": [8862], "characters": "\u229E" },
        "&boxtimes;": { "codepoints": [8864], "characters": "\u22A0" },
        "&boxuL;": { "codepoints": [9563], "characters": "\u255B" },
        "&boxuR;": { "codepoints": [9560], "characters": "\u2558" },
        "&boxul;": { "codepoints": [9496], "characters": "\u2518" },
        "&boxur;": { "codepoints": [9492], "characters": "\u2514" },
        "&boxv;": { "codepoints": [9474], "characters": "\u2502" },
        "&boxvH;": { "codepoints": [9578], "characters": "\u256A" },
        "&boxvL;": { "codepoints": [9569], "characters": "\u2561" },
        "&boxvR;": { "codepoints": [9566], "characters": "\u255E" },
        "&boxvh;": { "codepoints": [9532], "characters": "\u253C" },
        "&boxvl;": { "codepoints": [9508], "characters": "\u2524" },
        "&boxvr;": { "codepoints": [9500], "characters": "\u251C" },
        "&bprime;": { "codepoints": [8245], "characters": "\u2035" },
        "&breve;": { "codepoints": [728], "characters": "\u02D8" },
        "&brvbar": { "codepoints": [166], "characters": "\u00A6" },
        "&brvbar;": { "codepoints": [166], "characters": "\u00A6" },
        "&bscr;": { "codepoints": [119991], "characters": "\uD835\uDCB7" },
        "&bsemi;": { "codepoints": [8271], "characters": "\u204F" },
        "&bsim;": { "codepoints": [8765], "characters": "\u223D" },
        "&bsime;": { "codepoints": [8909], "characters": "\u22CD" },
        "&bsol;": { "codepoints": [92], "characters": "\u005C" },
        "&bsolb;": { "codepoints": [10693], "characters": "\u29C5" },
        "&bsolhsub;": { "codepoints": [10184], "characters": "\u27C8" },
        "&bull;": { "codepoints": [8226], "characters": "\u2022" },
        "&bullet;": { "codepoints": [8226], "characters": "\u2022" },
        "&bump;": { "codepoints": [8782], "characters": "\u224E" },
        "&bumpE;": { "codepoints": [10926], "characters": "\u2AAE" },
        "&bumpe;": { "codepoints": [8783], "characters": "\u224F" },
        "&bumpeq;": { "codepoints": [8783], "characters": "\u224F" },
        "&cacute;": { "codepoints": [263], "characters": "\u0107" },
        "&cap;": { "codepoints": [8745], "characters": "\u2229" },
        "&capand;": { "codepoints": [10820], "characters": "\u2A44" },
        "&capbrcup;": { "codepoints": [10825], "characters": "\u2A49" },
        "&capcap;": { "codepoints": [10827], "characters": "\u2A4B" },
        "&capcup;": { "codepoints": [10823], "characters": "\u2A47" },
        "&capdot;": { "codepoints": [10816], "characters": "\u2A40" },
        "&caps;": { "codepoints": [8745, 65024], "characters": "\u2229\uFE00" },
        "&caret;": { "codepoints": [8257], "characters": "\u2041" },
        "&caron;": { "codepoints": [711], "characters": "\u02C7" },
        "&ccaps;": { "codepoints": [10829], "characters": "\u2A4D" },
        "&ccaron;": { "codepoints": [269], "characters": "\u010D" },
        "&ccedil": { "codepoints": [231], "characters": "\u00E7" },
        "&ccedil;": { "codepoints": [231], "characters": "\u00E7" },
        "&ccirc;": { "codepoints": [265], "characters": "\u0109" },
        "&ccups;": { "codepoints": [10828], "characters": "\u2A4C" },
        "&ccupssm;": { "codepoints": [10832], "characters": "\u2A50" },
        "&cdot;": { "codepoints": [267], "characters": "\u010B" },
        "&cedil": { "codepoints": [184], "characters": "\u00B8" },
        "&cedil;": { "codepoints": [184], "characters": "\u00B8" },
        "&cemptyv;": { "codepoints": [10674], "characters": "\u29B2" },
        "&cent": { "codepoints": [162], "characters": "\u00A2" },
        "&cent;": { "codepoints": [162], "characters": "\u00A2" },
        "&centerdot;": { "codepoints": [183], "characters": "\u00B7" },
        "&cfr;": { "codepoints": [120096], "characters": "\uD835\uDD20" },
        "&chcy;": { "codepoints": [1095], "characters": "\u0447" },
        "&check;": { "codepoints": [10003], "characters": "\u2713" },
        "&checkmark;": { "codepoints": [10003], "characters": "\u2713" },
        "&chi;": { "codepoints": [967], "characters": "\u03C7" },
        "&cir;": { "codepoints": [9675], "characters": "\u25CB" },
        "&cirE;": { "codepoints": [10691], "characters": "\u29C3" },
        "&circ;": { "codepoints": [710], "characters": "\u02C6" },
        "&circeq;": { "codepoints": [8791], "characters": "\u2257" },
        "&circlearrowleft;": { "codepoints": [8634], "characters": "\u21BA" },
        "&circlearrowright;": { "codepoints": [8635], "characters": "\u21BB" },
        "&circledR;": { "codepoints": [174], "characters": "\u00AE" },
        "&circledS;": { "codepoints": [9416], "characters": "\u24C8" },
        "&circledast;": { "codepoints": [8859], "characters": "\u229B" },
        "&circledcirc;": { "codepoints": [8858], "characters": "\u229A" },
        "&circleddash;": { "codepoints": [8861], "characters": "\u229D" },
        "&cire;": { "codepoints": [8791], "characters": "\u2257" },
        "&cirfnint;": { "codepoints": [10768], "characters": "\u2A10" },
        "&cirmid;": { "codepoints": [10991], "characters": "\u2AEF" },
        "&cirscir;": { "codepoints": [10690], "characters": "\u29C2" },
        "&clubs;": { "codepoints": [9827], "characters": "\u2663" },
        "&clubsuit;": { "codepoints": [9827], "characters": "\u2663" },
        "&colon;": { "codepoints": [58], "characters": "\u003A" },
        "&colone;": { "codepoints": [8788], "characters": "\u2254" },
        "&coloneq;": { "codepoints": [8788], "characters": "\u2254" },
        "&comma;": { "codepoints": [44], "characters": "\u002C" },
        "&commat;": { "codepoints": [64], "characters": "\u0040" },
        "&comp;": { "codepoints": [8705], "characters": "\u2201" },
        "&compfn;": { "codepoints": [8728], "characters": "\u2218" },
        "&complement;": { "codepoints": [8705], "characters": "\u2201" },
        "&complexes;": { "codepoints": [8450], "characters": "\u2102" },
        "&cong;": { "codepoints": [8773], "characters": "\u2245" },
        "&congdot;": { "codepoints": [10861], "characters": "\u2A6D" },
        "&conint;": { "codepoints": [8750], "characters": "\u222E" },
        "&copf;": { "codepoints": [120148], "characters": "\uD835\uDD54" },
        "&coprod;": { "codepoints": [8720], "characters": "\u2210" },
        "&copy": { "codepoints": [169], "characters": "\u00A9" },
        "&copy;": { "codepoints": [169], "characters": "\u00A9" },
        "&copysr;": { "codepoints": [8471], "characters": "\u2117" },
        "&crarr;": { "codepoints": [8629], "characters": "\u21B5" },
        "&cross;": { "codepoints": [10007], "characters": "\u2717" },
        "&cscr;": { "codepoints": [119992], "characters": "\uD835\uDCB8" },
        "&csub;": { "codepoints": [10959], "characters": "\u2ACF" },
        "&csube;": { "codepoints": [10961], "characters": "\u2AD1" },
        "&csup;": { "codepoints": [10960], "characters": "\u2AD0" },
        "&csupe;": { "codepoints": [10962], "characters": "\u2AD2" },
        "&ctdot;": { "codepoints": [8943], "characters": "\u22EF" },
        "&cudarrl;": { "codepoints": [10552], "characters": "\u2938" },
        "&cudarrr;": { "codepoints": [10549], "characters": "\u2935" },
        "&cuepr;": { "codepoints": [8926], "characters": "\u22DE" },
        "&cuesc;": { "codepoints": [8927], "characters": "\u22DF" },
        "&cularr;": { "codepoints": [8630], "characters": "\u21B6" },
        "&cularrp;": { "codepoints": [10557], "characters": "\u293D" },
        "&cup;": { "codepoints": [8746], "characters": "\u222A" },
        "&cupbrcap;": { "codepoints": [10824], "characters": "\u2A48" },
        "&cupcap;": { "codepoints": [10822], "characters": "\u2A46" },
        "&cupcup;": { "codepoints": [10826], "characters": "\u2A4A" },
        "&cupdot;": { "codepoints": [8845], "characters": "\u228D" },
        "&cupor;": { "codepoints": [10821], "characters": "\u2A45" },
        "&cups;": { "codepoints": [8746, 65024], "characters": "\u222A\uFE00" },
        "&curarr;": { "codepoints": [8631], "characters": "\u21B7" },
        "&curarrm;": { "codepoints": [10556], "characters": "\u293C" },
        "&curlyeqprec;": { "codepoints": [8926], "characters": "\u22DE" },
        "&curlyeqsucc;": { "codepoints": [8927], "characters": "\u22DF" },
        "&curlyvee;": { "codepoints": [8910], "characters": "\u22CE" },
        "&curlywedge;": { "codepoints": [8911], "characters": "\u22CF" },
        "&curren": { "codepoints": [164], "characters": "\u00A4" },
        "&curren;": { "codepoints": [164], "characters": "\u00A4" },
        "&curvearrowleft;": { "codepoints": [8630], "characters": "\u21B6" },
        "&curvearrowright;": { "codepoints": [8631], "characters": "\u21B7" },
        "&cuvee;": { "codepoints": [8910], "characters": "\u22CE" },
        "&cuwed;": { "codepoints": [8911], "characters": "\u22CF" },
        "&cwconint;": { "codepoints": [8754], "characters": "\u2232" },
        "&cwint;": { "codepoints": [8753], "characters": "\u2231" },
        "&cylcty;": { "codepoints": [9005], "characters": "\u232D" },
        "&dArr;": { "codepoints": [8659], "characters": "\u21D3" },
        "&dHar;": { "codepoints": [10597], "characters": "\u2965" },
        "&dagger;": { "codepoints": [8224], "characters": "\u2020" },
        "&daleth;": { "codepoints": [8504], "characters": "\u2138" },
        "&darr;": { "codepoints": [8595], "characters": "\u2193" },
        "&dash;": { "codepoints": [8208], "characters": "\u2010" },
        "&dashv;": { "codepoints": [8867], "characters": "\u22A3" },
        "&dbkarow;": { "codepoints": [10511], "characters": "\u290F" },
        "&dblac;": { "codepoints": [733], "characters": "\u02DD" },
        "&dcaron;": { "codepoints": [271], "characters": "\u010F" },
        "&dcy;": { "codepoints": [1076], "characters": "\u0434" },
        "&dd;": { "codepoints": [8518], "characters": "\u2146" },
        "&ddagger;": { "codepoints": [8225], "characters": "\u2021" },
        "&ddarr;": { "codepoints": [8650], "characters": "\u21CA" },
        "&ddotseq;": { "codepoints": [10871], "characters": "\u2A77" },
        "&deg": { "codepoints": [176], "characters": "\u00B0" },
        "&deg;": { "codepoints": [176], "characters": "\u00B0" },
        "&delta;": { "codepoints": [948], "characters": "\u03B4" },
        "&demptyv;": { "codepoints": [10673], "characters": "\u29B1" },
        "&dfisht;": { "codepoints": [10623], "characters": "\u297F" },
        "&dfr;": { "codepoints": [120097], "characters": "\uD835\uDD21" },
        "&dharl;": { "codepoints": [8643], "characters": "\u21C3" },
        "&dharr;": { "codepoints": [8642], "characters": "\u21C2" },
        "&diam;": { "codepoints": [8900], "characters": "\u22C4" },
        "&diamond;": { "codepoints": [8900], "characters": "\u22C4" },
        "&diamondsuit;": { "codepoints": [9830], "characters": "\u2666" },
        "&diams;": { "codepoints": [9830], "characters": "\u2666" },
        "&die;": { "codepoints": [168], "characters": "\u00A8" },
        "&digamma;": { "codepoints": [989], "characters": "\u03DD" },
        "&disin;": { "codepoints": [8946], "characters": "\u22F2" },
        "&div;": { "codepoints": [247], "characters": "\u00F7" },
        "&divide": { "codepoints": [247], "characters": "\u00F7" },
        "&divide;": { "codepoints": [247], "characters": "\u00F7" },
        "&divideontimes;": { "codepoints": [8903], "characters": "\u22C7" },
        "&divonx;": { "codepoints": [8903], "characters": "\u22C7" },
        "&djcy;": { "codepoints": [1106], "characters": "\u0452" },
        "&dlcorn;": { "codepoints": [8990], "characters": "\u231E" },
        "&dlcrop;": { "codepoints": [8973], "characters": "\u230D" },
        "&dollar;": { "codepoints": [36], "characters": "\u0024" },
        "&dopf;": { "codepoints": [120149], "characters": "\uD835\uDD55" },
        "&dot;": { "codepoints": [729], "characters": "\u02D9" },
        "&doteq;": { "codepoints": [8784], "characters": "\u2250" },
        "&doteqdot;": { "codepoints": [8785], "characters": "\u2251" },
        "&dotminus;": { "codepoints": [8760], "characters": "\u2238" },
        "&dotplus;": { "codepoints": [8724], "characters": "\u2214" },
        "&dotsquare;": { "codepoints": [8865], "characters": "\u22A1" },
        "&doublebarwedge;": { "codepoints": [8966], "characters": "\u2306" },
        "&downarrow;": { "codepoints": [8595], "characters": "\u2193" },
        "&downdownarrows;": { "codepoints": [8650], "characters": "\u21CA" },
        "&downharpoonleft;": { "codepoints": [8643], "characters": "\u21C3" },
        "&downharpoonright;": { "codepoints": [8642], "characters": "\u21C2" },
        "&drbkarow;": { "codepoints": [10512], "characters": "\u2910" },
        "&drcorn;": { "codepoints": [8991], "characters": "\u231F" },
        "&drcrop;": { "codepoints": [8972], "characters": "\u230C" },
        "&dscr;": { "codepoints": [119993], "characters": "\uD835\uDCB9" },
        "&dscy;": { "codepoints": [1109], "characters": "\u0455" },
        "&dsol;": { "codepoints": [10742], "characters": "\u29F6" },
        "&dstrok;": { "codepoints": [273], "characters": "\u0111" },
        "&dtdot;": { "codepoints": [8945], "characters": "\u22F1" },
        "&dtri;": { "codepoints": [9663], "characters": "\u25BF" },
        "&dtrif;": { "codepoints": [9662], "characters": "\u25BE" },
        "&duarr;": { "codepoints": [8693], "characters": "\u21F5" },
        "&duhar;": { "codepoints": [10607], "characters": "\u296F" },
        "&dwangle;": { "codepoints": [10662], "characters": "\u29A6" },
        "&dzcy;": { "codepoints": [1119], "characters": "\u045F" },
        "&dzigrarr;": { "codepoints": [10239], "characters": "\u27FF" },
        "&eDDot;": { "codepoints": [10871], "characters": "\u2A77" },
        "&eDot;": { "codepoints": [8785], "characters": "\u2251" },
        "&eacute": { "codepoints": [233], "characters": "\u00E9" },
        "&eacute;": { "codepoints": [233], "characters": "\u00E9" },
        "&easter;": { "codepoints": [10862], "characters": "\u2A6E" },
        "&ecaron;": { "codepoints": [283], "characters": "\u011B" },
        "&ecir;": { "codepoints": [8790], "characters": "\u2256" },
        "&ecirc": { "codepoints": [234], "characters": "\u00EA" },
        "&ecirc;": { "codepoints": [234], "characters": "\u00EA" },
        "&ecolon;": { "codepoints": [8789], "characters": "\u2255" },
        "&ecy;": { "codepoints": [1101], "characters": "\u044D" },
        "&edot;": { "codepoints": [279], "characters": "\u0117" },
        "&ee;": { "codepoints": [8519], "characters": "\u2147" },
        "&efDot;": { "codepoints": [8786], "characters": "\u2252" },
        "&efr;": { "codepoints": [120098], "characters": "\uD835\uDD22" },
        "&eg;": { "codepoints": [10906], "characters": "\u2A9A" },
        "&egrave": { "codepoints": [232], "characters": "\u00E8" },
        "&egrave;": { "codepoints": [232], "characters": "\u00E8" },
        "&egs;": { "codepoints": [10902], "characters": "\u2A96" },
        "&egsdot;": { "codepoints": [10904], "characters": "\u2A98" },
        "&el;": { "codepoints": [10905], "characters": "\u2A99" },
        "&elinters;": { "codepoints": [9191], "characters": "\u23E7" },
        "&ell;": { "codepoints": [8467], "characters": "\u2113" },
        "&els;": { "codepoints": [10901], "characters": "\u2A95" },
        "&elsdot;": { "codepoints": [10903], "characters": "\u2A97" },
        "&emacr;": { "codepoints": [275], "characters": "\u0113" },
        "&empty;": { "codepoints": [8709], "characters": "\u2205" },
        "&emptyset;": { "codepoints": [8709], "characters": "\u2205" },
        "&emptyv;": { "codepoints": [8709], "characters": "\u2205" },
        "&emsp13;": { "codepoints": [8196], "characters": "\u2004" },
        "&emsp14;": { "codepoints": [8197], "characters": "\u2005" },
        "&emsp;": { "codepoints": [8195], "characters": "\u2003" },
        "&eng;": { "codepoints": [331], "characters": "\u014B" },
        "&ensp;": { "codepoints": [8194], "characters": "\u2002" },
        "&eogon;": { "codepoints": [281], "characters": "\u0119" },
        "&eopf;": { "codepoints": [120150], "characters": "\uD835\uDD56" },
        "&epar;": { "codepoints": [8917], "characters": "\u22D5" },
        "&eparsl;": { "codepoints": [10723], "characters": "\u29E3" },
        "&eplus;": { "codepoints": [10865], "characters": "\u2A71" },
        "&epsi;": { "codepoints": [949], "characters": "\u03B5" },
        "&epsilon;": { "codepoints": [949], "characters": "\u03B5" },
        "&epsiv;": { "codepoints": [1013], "characters": "\u03F5" },
        "&eqcirc;": { "codepoints": [8790], "characters": "\u2256" },
        "&eqcolon;": { "codepoints": [8789], "characters": "\u2255" },
        "&eqsim;": { "codepoints": [8770], "characters": "\u2242" },
        "&eqslantgtr;": { "codepoints": [10902], "characters": "\u2A96" },
        "&eqslantless;": { "codepoints": [10901], "characters": "\u2A95" },
        "&equals;": { "codepoints": [61], "characters": "\u003D" },
        "&equest;": { "codepoints": [8799], "characters": "\u225F" },
        "&equiv;": { "codepoints": [8801], "characters": "\u2261" },
        "&equivDD;": { "codepoints": [10872], "characters": "\u2A78" },
        "&eqvparsl;": { "codepoints": [10725], "characters": "\u29E5" },
        "&erDot;": { "codepoints": [8787], "characters": "\u2253" },
        "&erarr;": { "codepoints": [10609], "characters": "\u2971" },
        "&escr;": { "codepoints": [8495], "characters": "\u212F" },
        "&esdot;": { "codepoints": [8784], "characters": "\u2250" },
        "&esim;": { "codepoints": [8770], "characters": "\u2242" },
        "&eta;": { "codepoints": [951], "characters": "\u03B7" },
        "&eth": { "codepoints": [240], "characters": "\u00F0" },
        "&eth;": { "codepoints": [240], "characters": "\u00F0" },
        "&euml": { "codepoints": [235], "characters": "\u00EB" },
        "&euml;": { "codepoints": [235], "characters": "\u00EB" },
        "&euro;": { "codepoints": [8364], "characters": "\u20AC" },
        "&excl;": { "codepoints": [33], "characters": "\u0021" },
        "&exist;": { "codepoints": [8707], "characters": "\u2203" },
        "&expectation;": { "codepoints": [8496], "characters": "\u2130" },
        "&exponentiale;": { "codepoints": [8519], "characters": "\u2147" },
        "&fallingdotseq;": { "codepoints": [8786], "characters": "\u2252" },
        "&fcy;": { "codepoints": [1092], "characters": "\u0444" },
        "&female;": { "codepoints": [9792], "characters": "\u2640" },
        "&ffilig;": { "codepoints": [64259], "characters": "\uFB03" },
        "&fflig;": { "codepoints": [64256], "characters": "\uFB00" },
        "&ffllig;": { "codepoints": [64260], "characters": "\uFB04" },
        "&ffr;": { "codepoints": [120099], "characters": "\uD835\uDD23" },
        "&filig;": { "codepoints": [64257], "characters": "\uFB01" },
        "&fjlig;": { "codepoints": [102, 106], "characters": "\u0066\u006A" },
        "&flat;": { "codepoints": [9837], "characters": "\u266D" },
        "&fllig;": { "codepoints": [64258], "characters": "\uFB02" },
        "&fltns;": { "codepoints": [9649], "characters": "\u25B1" },
        "&fnof;": { "codepoints": [402], "characters": "\u0192" },
        "&fopf;": { "codepoints": [120151], "characters": "\uD835\uDD57" },
        "&forall;": { "codepoints": [8704], "characters": "\u2200" },
        "&fork;": { "codepoints": [8916], "characters": "\u22D4" },
        "&forkv;": { "codepoints": [10969], "characters": "\u2AD9" },
        "&fpartint;": { "codepoints": [10765], "characters": "\u2A0D" },
        "&frac12": { "codepoints": [189], "characters": "\u00BD" },
        "&frac12;": { "codepoints": [189], "characters": "\u00BD" },
        "&frac13;": { "codepoints": [8531], "characters": "\u2153" },
        "&frac14": { "codepoints": [188], "characters": "\u00BC" },
        "&frac14;": { "codepoints": [188], "characters": "\u00BC" },
        "&frac15;": { "codepoints": [8533], "characters": "\u2155" },
        "&frac16;": { "codepoints": [8537], "characters": "\u2159" },
        "&frac18;": { "codepoints": [8539], "characters": "\u215B" },
        "&frac23;": { "codepoints": [8532], "characters": "\u2154" },
        "&frac25;": { "codepoints": [8534], "characters": "\u2156" },
        "&frac34": { "codepoints": [190], "characters": "\u00BE" },
        "&frac34;": { "codepoints": [190], "characters": "\u00BE" },
        "&frac35;": { "codepoints": [8535], "characters": "\u2157" },
        "&frac38;": { "codepoints": [8540], "characters": "\u215C" },
        "&frac45;": { "codepoints": [8536], "characters": "\u2158" },
        "&frac56;": { "codepoints": [8538], "characters": "\u215A" },
        "&frac58;": { "codepoints": [8541], "characters": "\u215D" },
        "&frac78;": { "codepoints": [8542], "characters": "\u215E" },
        "&frasl;": { "codepoints": [8260], "characters": "\u2044" },
        "&frown;": { "codepoints": [8994], "characters": "\u2322" },
        "&fscr;": { "codepoints": [119995], "characters": "\uD835\uDCBB" },
        "&gE;": { "codepoints": [8807], "characters": "\u2267" },
        "&gEl;": { "codepoints": [10892], "characters": "\u2A8C" },
        "&gacute;": { "codepoints": [501], "characters": "\u01F5" },
        "&gamma;": { "codepoints": [947], "characters": "\u03B3" },
        "&gammad;": { "codepoints": [989], "characters": "\u03DD" },
        "&gap;": { "codepoints": [10886], "characters": "\u2A86" },
        "&gbreve;": { "codepoints": [287], "characters": "\u011F" },
        "&gcirc;": { "codepoints": [285], "characters": "\u011D" },
        "&gcy;": { "codepoints": [1075], "characters": "\u0433" },
        "&gdot;": { "codepoints": [289], "characters": "\u0121" },
        "&ge;": { "codepoints": [8805], "characters": "\u2265" },
        "&gel;": { "codepoints": [8923], "characters": "\u22DB" },
        "&geq;": { "codepoints": [8805], "characters": "\u2265" },
        "&geqq;": { "codepoints": [8807], "characters": "\u2267" },
        "&geqslant;": { "codepoints": [10878], "characters": "\u2A7E" },
        "&ges;": { "codepoints": [10878], "characters": "\u2A7E" },
        "&gescc;": { "codepoints": [10921], "characters": "\u2AA9" },
        "&gesdot;": { "codepoints": [10880], "characters": "\u2A80" },
        "&gesdoto;": { "codepoints": [10882], "characters": "\u2A82" },
        "&gesdotol;": { "codepoints": [10884], "characters": "\u2A84" },
        "&gesl;": { "codepoints": [8923, 65024], "characters": "\u22DB\uFE00" },
        "&gesles;": { "codepoints": [10900], "characters": "\u2A94" },
        "&gfr;": { "codepoints": [120100], "characters": "\uD835\uDD24" },
        "&gg;": { "codepoints": [8811], "characters": "\u226B" },
        "&ggg;": { "codepoints": [8921], "characters": "\u22D9" },
        "&gimel;": { "codepoints": [8503], "characters": "\u2137" },
        "&gjcy;": { "codepoints": [1107], "characters": "\u0453" },
        "&gl;": { "codepoints": [8823], "characters": "\u2277" },
        "&glE;": { "codepoints": [10898], "characters": "\u2A92" },
        "&gla;": { "codepoints": [10917], "characters": "\u2AA5" },
        "&glj;": { "codepoints": [10916], "characters": "\u2AA4" },
        "&gnE;": { "codepoints": [8809], "characters": "\u2269" },
        "&gnap;": { "codepoints": [10890], "characters": "\u2A8A" },
        "&gnapprox;": { "codepoints": [10890], "characters": "\u2A8A" },
        "&gne;": { "codepoints": [10888], "characters": "\u2A88" },
        "&gneq;": { "codepoints": [10888], "characters": "\u2A88" },
        "&gneqq;": { "codepoints": [8809], "characters": "\u2269" },
        "&gnsim;": { "codepoints": [8935], "characters": "\u22E7" },
        "&gopf;": { "codepoints": [120152], "characters": "\uD835\uDD58" },
        "&grave;": { "codepoints": [96], "characters": "\u0060" },
        "&gscr;": { "codepoints": [8458], "characters": "\u210A" },
        "&gsim;": { "codepoints": [8819], "characters": "\u2273" },
        "&gsime;": { "codepoints": [10894], "characters": "\u2A8E" },
        "&gsiml;": { "codepoints": [10896], "characters": "\u2A90" },
        "&gt": { "codepoints": [62], "characters": "\u003E" },
        "&gt;": { "codepoints": [62], "characters": "\u003E" },
        "&gtcc;": { "codepoints": [10919], "characters": "\u2AA7" },
        "&gtcir;": { "codepoints": [10874], "characters": "\u2A7A" },
        "&gtdot;": { "codepoints": [8919], "characters": "\u22D7" },
        "&gtlPar;": { "codepoints": [10645], "characters": "\u2995" },
        "&gtquest;": { "codepoints": [10876], "characters": "\u2A7C" },
        "&gtrapprox;": { "codepoints": [10886], "characters": "\u2A86" },
        "&gtrarr;": { "codepoints": [10616], "characters": "\u2978" },
        "&gtrdot;": { "codepoints": [8919], "characters": "\u22D7" },
        "&gtreqless;": { "codepoints": [8923], "characters": "\u22DB" },
        "&gtreqqless;": { "codepoints": [10892], "characters": "\u2A8C" },
        "&gtrless;": { "codepoints": [8823], "characters": "\u2277" },
        "&gtrsim;": { "codepoints": [8819], "characters": "\u2273" },
        "&gvertneqq;": { "codepoints": [8809, 65024], "characters": "\u2269\uFE00" },
        "&gvnE;": { "codepoints": [8809, 65024], "characters": "\u2269\uFE00" },
        "&hArr;": { "codepoints": [8660], "characters": "\u21D4" },
        "&hairsp;": { "codepoints": [8202], "characters": "\u200A" },
        "&half;": { "codepoints": [189], "characters": "\u00BD" },
        "&hamilt;": { "codepoints": [8459], "characters": "\u210B" },
        "&hardcy;": { "codepoints": [1098], "characters": "\u044A" },
        "&harr;": { "codepoints": [8596], "characters": "\u2194" },
        "&harrcir;": { "codepoints": [10568], "characters": "\u2948" },
        "&harrw;": { "codepoints": [8621], "characters": "\u21AD" },
        "&hbar;": { "codepoints": [8463], "characters": "\u210F" },
        "&hcirc;": { "codepoints": [293], "characters": "\u0125" },
        "&hearts;": { "codepoints": [9829], "characters": "\u2665" },
        "&heartsuit;": { "codepoints": [9829], "characters": "\u2665" },
        "&hellip;": { "codepoints": [8230], "characters": "\u2026" },
        "&hercon;": { "codepoints": [8889], "characters": "\u22B9" },
        "&hfr;": { "codepoints": [120101], "characters": "\uD835\uDD25" },
        "&hksearow;": { "codepoints": [10533], "characters": "\u2925" },
        "&hkswarow;": { "codepoints": [10534], "characters": "\u2926" },
        "&hoarr;": { "codepoints": [8703], "characters": "\u21FF" },
        "&homtht;": { "codepoints": [8763], "characters": "\u223B" },
        "&hookleftarrow;": { "codepoints": [8617], "characters": "\u21A9" },
        "&hookrightarrow;": { "codepoints": [8618], "characters": "\u21AA" },
        "&hopf;": { "codepoints": [120153], "characters": "\uD835\uDD59" },
        "&horbar;": { "codepoints": [8213], "characters": "\u2015" },
        "&hscr;": { "codepoints": [119997], "characters": "\uD835\uDCBD" },
        "&hslash;": { "codepoints": [8463], "characters": "\u210F" },
        "&hstrok;": { "codepoints": [295], "characters": "\u0127" },
        "&hybull;": { "codepoints": [8259], "characters": "\u2043" },
        "&hyphen;": { "codepoints": [8208], "characters": "\u2010" },
        "&iacute": { "codepoints": [237], "characters": "\u00ED" },
        "&iacute;": { "codepoints": [237], "characters": "\u00ED" },
        "&ic;": { "codepoints": [8291], "characters": "\u2063" },
        "&icirc": { "codepoints": [238], "characters": "\u00EE" },
        "&icirc;": { "codepoints": [238], "characters": "\u00EE" },
        "&icy;": { "codepoints": [1080], "characters": "\u0438" },
        "&iecy;": { "codepoints": [1077], "characters": "\u0435" },
        "&iexcl": { "codepoints": [161], "characters": "\u00A1" },
        "&iexcl;": { "codepoints": [161], "characters": "\u00A1" },
        "&iff;": { "codepoints": [8660], "characters": "\u21D4" },
        "&ifr;": { "codepoints": [120102], "characters": "\uD835\uDD26" },
        "&igrave": { "codepoints": [236], "characters": "\u00EC" },
        "&igrave;": { "codepoints": [236], "characters": "\u00EC" },
        "&ii;": { "codepoints": [8520], "characters": "\u2148" },
        "&iiiint;": { "codepoints": [10764], "characters": "\u2A0C" },
        "&iiint;": { "codepoints": [8749], "characters": "\u222D" },
        "&iinfin;": { "codepoints": [10716], "characters": "\u29DC" },
        "&iiota;": { "codepoints": [8489], "characters": "\u2129" },
        "&ijlig;": { "codepoints": [307], "characters": "\u0133" },
        "&imacr;": { "codepoints": [299], "characters": "\u012B" },
        "&image;": { "codepoints": [8465], "characters": "\u2111" },
        "&imagline;": { "codepoints": [8464], "characters": "\u2110" },
        "&imagpart;": { "codepoints": [8465], "characters": "\u2111" },
        "&imath;": { "codepoints": [305], "characters": "\u0131" },
        "&imof;": { "codepoints": [8887], "characters": "\u22B7" },
        "&imped;": { "codepoints": [437], "characters": "\u01B5" },
        "&in;": { "codepoints": [8712], "characters": "\u2208" },
        "&incare;": { "codepoints": [8453], "characters": "\u2105" },
        "&infin;": { "codepoints": [8734], "characters": "\u221E" },
        "&infintie;": { "codepoints": [10717], "characters": "\u29DD" },
        "&inodot;": { "codepoints": [305], "characters": "\u0131" },
        "&int;": { "codepoints": [8747], "characters": "\u222B" },
        "&intcal;": { "codepoints": [8890], "characters": "\u22BA" },
        "&integers;": { "codepoints": [8484], "characters": "\u2124" },
        "&intercal;": { "codepoints": [8890], "characters": "\u22BA" },
        "&intlarhk;": { "codepoints": [10775], "characters": "\u2A17" },
        "&intprod;": { "codepoints": [10812], "characters": "\u2A3C" },
        "&iocy;": { "codepoints": [1105], "characters": "\u0451" },
        "&iogon;": { "codepoints": [303], "characters": "\u012F" },
        "&iopf;": { "codepoints": [120154], "characters": "\uD835\uDD5A" },
        "&iota;": { "codepoints": [953], "characters": "\u03B9" },
        "&iprod;": { "codepoints": [10812], "characters": "\u2A3C" },
        "&iquest": { "codepoints": [191], "characters": "\u00BF" },
        "&iquest;": { "codepoints": [191], "characters": "\u00BF" },
        "&iscr;": { "codepoints": [119998], "characters": "\uD835\uDCBE" },
        "&isin;": { "codepoints": [8712], "characters": "\u2208" },
        "&isinE;": { "codepoints": [8953], "characters": "\u22F9" },
        "&isindot;": { "codepoints": [8949], "characters": "\u22F5" },
        "&isins;": { "codepoints": [8948], "characters": "\u22F4" },
        "&isinsv;": { "codepoints": [8947], "characters": "\u22F3" },
        "&isinv;": { "codepoints": [8712], "characters": "\u2208" },
        "&it;": { "codepoints": [8290], "characters": "\u2062" },
        "&itilde;": { "codepoints": [297], "characters": "\u0129" },
        "&iukcy;": { "codepoints": [1110], "characters": "\u0456" },
        "&iuml": { "codepoints": [239], "characters": "\u00EF" },
        "&iuml;": { "codepoints": [239], "characters": "\u00EF" },
        "&jcirc;": { "codepoints": [309], "characters": "\u0135" },
        "&jcy;": { "codepoints": [1081], "characters": "\u0439" },
        "&jfr;": { "codepoints": [120103], "characters": "\uD835\uDD27" },
        "&jmath;": { "codepoints": [567], "characters": "\u0237" },
        "&jopf;": { "codepoints": [120155], "characters": "\uD835\uDD5B" },
        "&jscr;": { "codepoints": [119999], "characters": "\uD835\uDCBF" },
        "&jsercy;": { "codepoints": [1112], "characters": "\u0458" },
        "&jukcy;": { "codepoints": [1108], "characters": "\u0454" },
        "&kappa;": { "codepoints": [954], "characters": "\u03BA" },
        "&kappav;": { "codepoints": [1008], "characters": "\u03F0" },
        "&kcedil;": { "codepoints": [311], "characters": "\u0137" },
        "&kcy;": { "codepoints": [1082], "characters": "\u043A" },
        "&kfr;": { "codepoints": [120104], "characters": "\uD835\uDD28" },
        "&kgreen;": { "codepoints": [312], "characters": "\u0138" },
        "&khcy;": { "codepoints": [1093], "characters": "\u0445" },
        "&kjcy;": { "codepoints": [1116], "characters": "\u045C" },
        "&kopf;": { "codepoints": [120156], "characters": "\uD835\uDD5C" },
        "&kscr;": { "codepoints": [120000], "characters": "\uD835\uDCC0" },
        "&lAarr;": { "codepoints": [8666], "characters": "\u21DA" },
        "&lArr;": { "codepoints": [8656], "characters": "\u21D0" },
        "&lAtail;": { "codepoints": [10523], "characters": "\u291B" },
        "&lBarr;": { "codepoints": [10510], "characters": "\u290E" },
        "&lE;": { "codepoints": [8806], "characters": "\u2266" },
        "&lEg;": { "codepoints": [10891], "characters": "\u2A8B" },
        "&lHar;": { "codepoints": [10594], "characters": "\u2962" },
        "&lacute;": { "codepoints": [314], "characters": "\u013A" },
        "&laemptyv;": { "codepoints": [10676], "characters": "\u29B4" },
        "&lagran;": { "codepoints": [8466], "characters": "\u2112" },
        "&lambda;": { "codepoints": [955], "characters": "\u03BB" },
        "&lang;": { "codepoints": [10216], "characters": "\u27E8" },
        "&langd;": { "codepoints": [10641], "characters": "\u2991" },
        "&langle;": { "codepoints": [10216], "characters": "\u27E8" },
        "&lap;": { "codepoints": [10885], "characters": "\u2A85" },
        "&laquo": { "codepoints": [171], "characters": "\u00AB" },
        "&laquo;": { "codepoints": [171], "characters": "\u00AB" },
        "&larr;": { "codepoints": [8592], "characters": "\u2190" },
        "&larrb;": { "codepoints": [8676], "characters": "\u21E4" },
        "&larrbfs;": { "codepoints": [10527], "characters": "\u291F" },
        "&larrfs;": { "codepoints": [10525], "characters": "\u291D" },
        "&larrhk;": { "codepoints": [8617], "characters": "\u21A9" },
        "&larrlp;": { "codepoints": [8619], "characters": "\u21AB" },
        "&larrpl;": { "codepoints": [10553], "characters": "\u2939" },
        "&larrsim;": { "codepoints": [10611], "characters": "\u2973" },
        "&larrtl;": { "codepoints": [8610], "characters": "\u21A2" },
        "&lat;": { "codepoints": [10923], "characters": "\u2AAB" },
        "&latail;": { "codepoints": [10521], "characters": "\u2919" },
        "&late;": { "codepoints": [10925], "characters": "\u2AAD" },
        "&lates;": { "codepoints": [10925, 65024], "characters": "\u2AAD\uFE00" },
        "&lbarr;": { "codepoints": [10508], "characters": "\u290C" },
        "&lbbrk;": { "codepoints": [10098], "characters": "\u2772" },
        "&lbrace;": { "codepoints": [123], "characters": "\u007B" },
        "&lbrack;": { "codepoints": [91], "characters": "\u005B" },
        "&lbrke;": { "codepoints": [10635], "characters": "\u298B" },
        "&lbrksld;": { "codepoints": [10639], "characters": "\u298F" },
        "&lbrkslu;": { "codepoints": [10637], "characters": "\u298D" },
        "&lcaron;": { "codepoints": [318], "characters": "\u013E" },
        "&lcedil;": { "codepoints": [316], "characters": "\u013C" },
        "&lceil;": { "codepoints": [8968], "characters": "\u2308" },
        "&lcub;": { "codepoints": [123], "characters": "\u007B" },
        "&lcy;": { "codepoints": [1083], "characters": "\u043B" },
        "&ldca;": { "codepoints": [10550], "characters": "\u2936" },
        "&ldquo;": { "codepoints": [8220], "characters": "\u201C" },
        "&ldquor;": { "codepoints": [8222], "characters": "\u201E" },
        "&ldrdhar;": { "codepoints": [10599], "characters": "\u2967" },
        "&ldrushar;": { "codepoints": [10571], "characters": "\u294B" },
        "&ldsh;": { "codepoints": [8626], "characters": "\u21B2" },
        "&le;": { "codepoints": [8804], "characters": "\u2264" },
        "&leftarrow;": { "codepoints": [8592], "characters": "\u2190" },
        "&leftarrowtail;": { "codepoints": [8610], "characters": "\u21A2" },
        "&leftharpoondown;": { "codepoints": [8637], "characters": "\u21BD" },
        "&leftharpoonup;": { "codepoints": [8636], "characters": "\u21BC" },
        "&leftleftarrows;": { "codepoints": [8647], "characters": "\u21C7" },
        "&leftrightarrow;": { "codepoints": [8596], "characters": "\u2194" },
        "&leftrightarrows;": { "codepoints": [8646], "characters": "\u21C6" },
        "&leftrightharpoons;": { "codepoints": [8651], "characters": "\u21CB" },
        "&leftrightsquigarrow;": { "codepoints": [8621], "characters": "\u21AD" },
        "&leftthreetimes;": { "codepoints": [8907], "characters": "\u22CB" },
        "&leg;": { "codepoints": [8922], "characters": "\u22DA" },
        "&leq;": { "codepoints": [8804], "characters": "\u2264" },
        "&leqq;": { "codepoints": [8806], "characters": "\u2266" },
        "&leqslant;": { "codepoints": [10877], "characters": "\u2A7D" },
        "&les;": { "codepoints": [10877], "characters": "\u2A7D" },
        "&lescc;": { "codepoints": [10920], "characters": "\u2AA8" },
        "&lesdot;": { "codepoints": [10879], "characters": "\u2A7F" },
        "&lesdoto;": { "codepoints": [10881], "characters": "\u2A81" },
        "&lesdotor;": { "codepoints": [10883], "characters": "\u2A83" },
        "&lesg;": { "codepoints": [8922, 65024], "characters": "\u22DA\uFE00" },
        "&lesges;": { "codepoints": [10899], "characters": "\u2A93" },
        "&lessapprox;": { "codepoints": [10885], "characters": "\u2A85" },
        "&lessdot;": { "codepoints": [8918], "characters": "\u22D6" },
        "&lesseqgtr;": { "codepoints": [8922], "characters": "\u22DA" },
        "&lesseqqgtr;": { "codepoints": [10891], "characters": "\u2A8B" },
        "&lessgtr;": { "codepoints": [8822], "characters": "\u2276" },
        "&lesssim;": { "codepoints": [8818], "characters": "\u2272" },
        "&lfisht;": { "codepoints": [10620], "characters": "\u297C" },
        "&lfloor;": { "codepoints": [8970], "characters": "\u230A" },
        "&lfr;": { "codepoints": [120105], "characters": "\uD835\uDD29" },
        "&lg;": { "codepoints": [8822], "characters": "\u2276" },
        "&lgE;": { "codepoints": [10897], "characters": "\u2A91" },
        "&lhard;": { "codepoints": [8637], "characters": "\u21BD" },
        "&lharu;": { "codepoints": [8636], "characters": "\u21BC" },
        "&lharul;": { "codepoints": [10602], "characters": "\u296A" },
        "&lhblk;": { "codepoints": [9604], "characters": "\u2584" },
        "&ljcy;": { "codepoints": [1113], "characters": "\u0459" },
        "&ll;": { "codepoints": [8810], "characters": "\u226A" },
        "&llarr;": { "codepoints": [8647], "characters": "\u21C7" },
        "&llcorner;": { "codepoints": [8990], "characters": "\u231E" },
        "&llhard;": { "codepoints": [10603], "characters": "\u296B" },
        "&lltri;": { "codepoints": [9722], "characters": "\u25FA" },
        "&lmidot;": { "codepoints": [320], "characters": "\u0140" },
        "&lmoust;": { "codepoints": [9136], "characters": "\u23B0" },
        "&lmoustache;": { "codepoints": [9136], "characters": "\u23B0" },
        "&lnE;": { "codepoints": [8808], "characters": "\u2268" },
        "&lnap;": { "codepoints": [10889], "characters": "\u2A89" },
        "&lnapprox;": { "codepoints": [10889], "characters": "\u2A89" },
        "&lne;": { "codepoints": [10887], "characters": "\u2A87" },
        "&lneq;": { "codepoints": [10887], "characters": "\u2A87" },
        "&lneqq;": { "codepoints": [8808], "characters": "\u2268" },
        "&lnsim;": { "codepoints": [8934], "characters": "\u22E6" },
        "&loang;": { "codepoints": [10220], "characters": "\u27EC" },
        "&loarr;": { "codepoints": [8701], "characters": "\u21FD" },
        "&lobrk;": { "codepoints": [10214], "characters": "\u27E6" },
        "&longleftarrow;": { "codepoints": [10229], "characters": "\u27F5" },
        "&longleftrightarrow;": { "codepoints": [10231], "characters": "\u27F7" },
        "&longmapsto;": { "codepoints": [10236], "characters": "\u27FC" },
        "&longrightarrow;": { "codepoints": [10230], "characters": "\u27F6" },
        "&looparrowleft;": { "codepoints": [8619], "characters": "\u21AB" },
        "&looparrowright;": { "codepoints": [8620], "characters": "\u21AC" },
        "&lopar;": { "codepoints": [10629], "characters": "\u2985" },
        "&lopf;": { "codepoints": [120157], "characters": "\uD835\uDD5D" },
        "&loplus;": { "codepoints": [10797], "characters": "\u2A2D" },
        "&lotimes;": { "codepoints": [10804], "characters": "\u2A34" },
        "&lowast;": { "codepoints": [8727], "characters": "\u2217" },
        "&lowbar;": { "codepoints": [95], "characters": "\u005F" },
        "&loz;": { "codepoints": [9674], "characters": "\u25CA" },
        "&lozenge;": { "codepoints": [9674], "characters": "\u25CA" },
        "&lozf;": { "codepoints": [10731], "characters": "\u29EB" },
        "&lpar;": { "codepoints": [40], "characters": "\u0028" },
        "&lparlt;": { "codepoints": [10643], "characters": "\u2993" },
        "&lrarr;": { "codepoints": [8646], "characters": "\u21C6" },
        "&lrcorner;": { "codepoints": [8991], "characters": "\u231F" },
        "&lrhar;": { "codepoints": [8651], "characters": "\u21CB" },
        "&lrhard;": { "codepoints": [10605], "characters": "\u296D" },
        "&lrm;": { "codepoints": [8206], "characters": "\u200E" },
        "&lrtri;": { "codepoints": [8895], "characters": "\u22BF" },
        "&lsaquo;": { "codepoints": [8249], "characters": "\u2039" },
        "&lscr;": { "codepoints": [120001], "characters": "\uD835\uDCC1" },
        "&lsh;": { "codepoints": [8624], "characters": "\u21B0" },
        "&lsim;": { "codepoints": [8818], "characters": "\u2272" },
        "&lsime;": { "codepoints": [10893], "characters": "\u2A8D" },
        "&lsimg;": { "codepoints": [10895], "characters": "\u2A8F" },
        "&lsqb;": { "codepoints": [91], "characters": "\u005B" },
        "&lsquo;": { "codepoints": [8216], "characters": "\u2018" },
        "&lsquor;": { "codepoints": [8218], "characters": "\u201A" },
        "&lstrok;": { "codepoints": [322], "characters": "\u0142" },
        "&lt": { "codepoints": [60], "characters": "\u003C" },
        "&lt;": { "codepoints": [60], "characters": "\u003C" },
        "&ltcc;": { "codepoints": [10918], "characters": "\u2AA6" },
        "&ltcir;": { "codepoints": [10873], "characters": "\u2A79" },
        "&ltdot;": { "codepoints": [8918], "characters": "\u22D6" },
        "&lthree;": { "codepoints": [8907], "characters": "\u22CB" },
        "&ltimes;": { "codepoints": [8905], "characters": "\u22C9" },
        "&ltlarr;": { "codepoints": [10614], "characters": "\u2976" },
        "&ltquest;": { "codepoints": [10875], "characters": "\u2A7B" },
        "&ltrPar;": { "codepoints": [10646], "characters": "\u2996" },
        "&ltri;": { "codepoints": [9667], "characters": "\u25C3" },
        "&ltrie;": { "codepoints": [8884], "characters": "\u22B4" },
        "&ltrif;": { "codepoints": [9666], "characters": "\u25C2" },
        "&lurdshar;": { "codepoints": [10570], "characters": "\u294A" },
        "&luruhar;": { "codepoints": [10598], "characters": "\u2966" },
        "&lvertneqq;": { "codepoints": [8808, 65024], "characters": "\u2268\uFE00" },
        "&lvnE;": { "codepoints": [8808, 65024], "characters": "\u2268\uFE00" },
        "&mDDot;": { "codepoints": [8762], "characters": "\u223A" },
        "&macr": { "codepoints": [175], "characters": "\u00AF" },
        "&macr;": { "codepoints": [175], "characters": "\u00AF" },
        "&male;": { "codepoints": [9794], "characters": "\u2642" },
        "&malt;": { "codepoints": [10016], "characters": "\u2720" },
        "&maltese;": { "codepoints": [10016], "characters": "\u2720" },
        "&map;": { "codepoints": [8614], "characters": "\u21A6" },
        "&mapsto;": { "codepoints": [8614], "characters": "\u21A6" },
        "&mapstodown;": { "codepoints": [8615], "characters": "\u21A7" },
        "&mapstoleft;": { "codepoints": [8612], "characters": "\u21A4" },
        "&mapstoup;": { "codepoints": [8613], "characters": "\u21A5" },
        "&marker;": { "codepoints": [9646], "characters": "\u25AE" },
        "&mcomma;": { "codepoints": [10793], "characters": "\u2A29" },
        "&mcy;": { "codepoints": [1084], "characters": "\u043C" },
        "&mdash;": { "codepoints": [8212], "characters": "\u2014" },
        "&measuredangle;": { "codepoints": [8737], "characters": "\u2221" },
        "&mfr;": { "codepoints": [120106], "characters": "\uD835\uDD2A" },
        "&mho;": { "codepoints": [8487], "characters": "\u2127" },
        "&micro": { "codepoints": [181], "characters": "\u00B5" },
        "&micro;": { "codepoints": [181], "characters": "\u00B5" },
        "&mid;": { "codepoints": [8739], "characters": "\u2223" },
        "&midast;": { "codepoints": [42], "characters": "\u002A" },
        "&midcir;": { "codepoints": [10992], "characters": "\u2AF0" },
        "&middot": { "codepoints": [183], "characters": "\u00B7" },
        "&middot;": { "codepoints": [183], "characters": "\u00B7" },
        "&minus;": { "codepoints": [8722], "characters": "\u2212" },
        "&minusb;": { "codepoints": [8863], "characters": "\u229F" },
        "&minusd;": { "codepoints": [8760], "characters": "\u2238" },
        "&minusdu;": { "codepoints": [10794], "characters": "\u2A2A" },
        "&mlcp;": { "codepoints": [10971], "characters": "\u2ADB" },
        "&mldr;": { "codepoints": [8230], "characters": "\u2026" },
        "&mnplus;": { "codepoints": [8723], "characters": "\u2213" },
        "&models;": { "codepoints": [8871], "characters": "\u22A7" },
        "&mopf;": { "codepoints": [120158], "characters": "\uD835\uDD5E" },
        "&mp;": { "codepoints": [8723], "characters": "\u2213" },
        "&mscr;": { "codepoints": [120002], "characters": "\uD835\uDCC2" },
        "&mstpos;": { "codepoints": [8766], "characters": "\u223E" },
        "&mu;": { "codepoints": [956], "characters": "\u03BC" },
        "&multimap;": { "codepoints": [8888], "characters": "\u22B8" },
        "&mumap;": { "codepoints": [8888], "characters": "\u22B8" },
        "&nGg;": { "codepoints": [8921, 824], "characters": "\u22D9\u0338" },
        "&nGt;": { "codepoints": [8811, 8402], "characters": "\u226B\u20D2" },
        "&nGtv;": { "codepoints": [8811, 824], "characters": "\u226B\u0338" },
        "&nLeftarrow;": { "codepoints": [8653], "characters": "\u21CD" },
        "&nLeftrightarrow;": { "codepoints": [8654], "characters": "\u21CE" },
        "&nLl;": { "codepoints": [8920, 824], "characters": "\u22D8\u0338" },
        "&nLt;": { "codepoints": [8810, 8402], "characters": "\u226A\u20D2" },
        "&nLtv;": { "codepoints": [8810, 824], "characters": "\u226A\u0338" },
        "&nRightarrow;": { "codepoints": [8655], "characters": "\u21CF" },
        "&nVDash;": { "codepoints": [8879], "characters": "\u22AF" },
        "&nVdash;": { "codepoints": [8878], "characters": "\u22AE" },
        "&nabla;": { "codepoints": [8711], "characters": "\u2207" },
        "&nacute;": { "codepoints": [324], "characters": "\u0144" },
        "&nang;": { "codepoints": [8736, 8402], "characters": "\u2220\u20D2" },
        "&nap;": { "codepoints": [8777], "characters": "\u2249" },
        "&napE;": { "codepoints": [10864, 824], "characters": "\u2A70\u0338" },
        "&napid;": { "codepoints": [8779, 824], "characters": "\u224B\u0338" },
        "&napos;": { "codepoints": [329], "characters": "\u0149" },
        "&napprox;": { "codepoints": [8777], "characters": "\u2249" },
        "&natur;": { "codepoints": [9838], "characters": "\u266E" },
        "&natural;": { "codepoints": [9838], "characters": "\u266E" },
        "&naturals;": { "codepoints": [8469], "characters": "\u2115" },
        "&nbsp": { "codepoints": [160], "characters": "\u00A0" },
        "&nbsp;": { "codepoints": [160], "characters": "\u00A0" },
        "&nbump;": { "codepoints": [8782, 824], "characters": "\u224E\u0338" },
        "&nbumpe;": { "codepoints": [8783, 824], "characters": "\u224F\u0338" },
        "&ncap;": { "codepoints": [10819], "characters": "\u2A43" },
        "&ncaron;": { "codepoints": [328], "characters": "\u0148" },
        "&ncedil;": { "codepoints": [326], "characters": "\u0146" },
        "&ncong;": { "codepoints": [8775], "characters": "\u2247" },
        "&ncongdot;": { "codepoints": [10861, 824], "characters": "\u2A6D\u0338" },
        "&ncup;": { "codepoints": [10818], "characters": "\u2A42" },
        "&ncy;": { "codepoints": [1085], "characters": "\u043D" },
        "&ndash;": { "codepoints": [8211], "characters": "\u2013" },
        "&ne;": { "codepoints": [8800], "characters": "\u2260" },
        "&neArr;": { "codepoints": [8663], "characters": "\u21D7" },
        "&nearhk;": { "codepoints": [10532], "characters": "\u2924" },
        "&nearr;": { "codepoints": [8599], "characters": "\u2197" },
        "&nearrow;": { "codepoints": [8599], "characters": "\u2197" },
        "&nedot;": { "codepoints": [8784, 824], "characters": "\u2250\u0338" },
        "&nequiv;": { "codepoints": [8802], "characters": "\u2262" },
        "&nesear;": { "codepoints": [10536], "characters": "\u2928" },
        "&nesim;": { "codepoints": [8770, 824], "characters": "\u2242\u0338" },
        "&nexist;": { "codepoints": [8708], "characters": "\u2204" },
        "&nexists;": { "codepoints": [8708], "characters": "\u2204" },
        "&nfr;": { "codepoints": [120107], "characters": "\uD835\uDD2B" },
        "&ngE;": { "codepoints": [8807, 824], "characters": "\u2267\u0338" },
        "&nge;": { "codepoints": [8817], "characters": "\u2271" },
        "&ngeq;": { "codepoints": [8817], "characters": "\u2271" },
        "&ngeqq;": { "codepoints": [8807, 824], "characters": "\u2267\u0338" },
        "&ngeqslant;": { "codepoints": [10878, 824], "characters": "\u2A7E\u0338" },
        "&nges;": { "codepoints": [10878, 824], "characters": "\u2A7E\u0338" },
        "&ngsim;": { "codepoints": [8821], "characters": "\u2275" },
        "&ngt;": { "codepoints": [8815], "characters": "\u226F" },
        "&ngtr;": { "codepoints": [8815], "characters": "\u226F" },
        "&nhArr;": { "codepoints": [8654], "characters": "\u21CE" },
        "&nharr;": { "codepoints": [8622], "characters": "\u21AE" },
        "&nhpar;": { "codepoints": [10994], "characters": "\u2AF2" },
        "&ni;": { "codepoints": [8715], "characters": "\u220B" },
        "&nis;": { "codepoints": [8956], "characters": "\u22FC" },
        "&nisd;": { "codepoints": [8954], "characters": "\u22FA" },
        "&niv;": { "codepoints": [8715], "characters": "\u220B" },
        "&njcy;": { "codepoints": [1114], "characters": "\u045A" },
        "&nlArr;": { "codepoints": [8653], "characters": "\u21CD" },
        "&nlE;": { "codepoints": [8806, 824], "characters": "\u2266\u0338" },
        "&nlarr;": { "codepoints": [8602], "characters": "\u219A" },
        "&nldr;": { "codepoints": [8229], "characters": "\u2025" },
        "&nle;": { "codepoints": [8816], "characters": "\u2270" },
        "&nleftarrow;": { "codepoints": [8602], "characters": "\u219A" },
        "&nleftrightarrow;": { "codepoints": [8622], "characters": "\u21AE" },
        "&nleq;": { "codepoints": [8816], "characters": "\u2270" },
        "&nleqq;": { "codepoints": [8806, 824], "characters": "\u2266\u0338" },
        "&nleqslant;": { "codepoints": [10877, 824], "characters": "\u2A7D\u0338" },
        "&nles;": { "codepoints": [10877, 824], "characters": "\u2A7D\u0338" },
        "&nless;": { "codepoints": [8814], "characters": "\u226E" },
        "&nlsim;": { "codepoints": [8820], "characters": "\u2274" },
        "&nlt;": { "codepoints": [8814], "characters": "\u226E" },
        "&nltri;": { "codepoints": [8938], "characters": "\u22EA" },
        "&nltrie;": { "codepoints": [8940], "characters": "\u22EC" },
        "&nmid;": { "codepoints": [8740], "characters": "\u2224" },
        "&nopf;": { "codepoints": [120159], "characters": "\uD835\uDD5F" },
        "&not": { "codepoints": [172], "characters": "\u00AC" },
        "&not;": { "codepoints": [172], "characters": "\u00AC" },
        "&notin;": { "codepoints": [8713], "characters": "\u2209" },
        "&notinE;": { "codepoints": [8953, 824], "characters": "\u22F9\u0338" },
        "&notindot;": { "codepoints": [8949, 824], "characters": "\u22F5\u0338" },
        "&notinva;": { "codepoints": [8713], "characters": "\u2209" },
        "&notinvb;": { "codepoints": [8951], "characters": "\u22F7" },
        "&notinvc;": { "codepoints": [8950], "characters": "\u22F6" },
        "&notni;": { "codepoints": [8716], "characters": "\u220C" },
        "&notniva;": { "codepoints": [8716], "characters": "\u220C" },
        "&notnivb;": { "codepoints": [8958], "characters": "\u22FE" },
        "&notnivc;": { "codepoints": [8957], "characters": "\u22FD" },
        "&npar;": { "codepoints": [8742], "characters": "\u2226" },
        "&nparallel;": { "codepoints": [8742], "characters": "\u2226" },
        "&nparsl;": { "codepoints": [11005, 8421], "characters": "\u2AFD\u20E5" },
        "&npart;": { "codepoints": [8706, 824], "characters": "\u2202\u0338" },
        "&npolint;": { "codepoints": [10772], "characters": "\u2A14" },
        "&npr;": { "codepoints": [8832], "characters": "\u2280" },
        "&nprcue;": { "codepoints": [8928], "characters": "\u22E0" },
        "&npre;": { "codepoints": [10927, 824], "characters": "\u2AAF\u0338" },
        "&nprec;": { "codepoints": [8832], "characters": "\u2280" },
        "&npreceq;": { "codepoints": [10927, 824], "characters": "\u2AAF\u0338" },
        "&nrArr;": { "codepoints": [8655], "characters": "\u21CF" },
        "&nrarr;": { "codepoints": [8603], "characters": "\u219B" },
        "&nrarrc;": { "codepoints": [10547, 824], "characters": "\u2933\u0338" },
        "&nrarrw;": { "codepoints": [8605, 824], "characters": "\u219D\u0338" },
        "&nrightarrow;": { "codepoints": [8603], "characters": "\u219B" },
        "&nrtri;": { "codepoints": [8939], "characters": "\u22EB" },
        "&nrtrie;": { "codepoints": [8941], "characters": "\u22ED" },
        "&nsc;": { "codepoints": [8833], "characters": "\u2281" },
        "&nsccue;": { "codepoints": [8929], "characters": "\u22E1" },
        "&nsce;": { "codepoints": [10928, 824], "characters": "\u2AB0\u0338" },
        "&nscr;": { "codepoints": [120003], "characters": "\uD835\uDCC3" },
        "&nshortmid;": { "codepoints": [8740], "characters": "\u2224" },
        "&nshortparallel;": { "codepoints": [8742], "characters": "\u2226" },
        "&nsim;": { "codepoints": [8769], "characters": "\u2241" },
        "&nsime;": { "codepoints": [8772], "characters": "\u2244" },
        "&nsimeq;": { "codepoints": [8772], "characters": "\u2244" },
        "&nsmid;": { "codepoints": [8740], "characters": "\u2224" },
        "&nspar;": { "codepoints": [8742], "characters": "\u2226" },
        "&nsqsube;": { "codepoints": [8930], "characters": "\u22E2" },
        "&nsqsupe;": { "codepoints": [8931], "characters": "\u22E3" },
        "&nsub;": { "codepoints": [8836], "characters": "\u2284" },
        "&nsubE;": { "codepoints": [10949, 824], "characters": "\u2AC5\u0338" },
        "&nsube;": { "codepoints": [8840], "characters": "\u2288" },
        "&nsubset;": { "codepoints": [8834, 8402], "characters": "\u2282\u20D2" },
        "&nsubseteq;": { "codepoints": [8840], "characters": "\u2288" },
        "&nsubseteqq;": { "codepoints": [10949, 824], "characters": "\u2AC5\u0338" },
        "&nsucc;": { "codepoints": [8833], "characters": "\u2281" },
        "&nsucceq;": { "codepoints": [10928, 824], "characters": "\u2AB0\u0338" },
        "&nsup;": { "codepoints": [8837], "characters": "\u2285" },
        "&nsupE;": { "codepoints": [10950, 824], "characters": "\u2AC6\u0338" },
        "&nsupe;": { "codepoints": [8841], "characters": "\u2289" },
        "&nsupset;": { "codepoints": [8835, 8402], "characters": "\u2283\u20D2" },
        "&nsupseteq;": { "codepoints": [8841], "characters": "\u2289" },
        "&nsupseteqq;": { "codepoints": [10950, 824], "characters": "\u2AC6\u0338" },
        "&ntgl;": { "codepoints": [8825], "characters": "\u2279" },
        "&ntilde": { "codepoints": [241], "characters": "\u00F1" },
        "&ntilde;": { "codepoints": [241], "characters": "\u00F1" },
        "&ntlg;": { "codepoints": [8824], "characters": "\u2278" },
        "&ntriangleleft;": { "codepoints": [8938], "characters": "\u22EA" },
        "&ntrianglelefteq;": { "codepoints": [8940], "characters": "\u22EC" },
        "&ntriangleright;": { "codepoints": [8939], "characters": "\u22EB" },
        "&ntrianglerighteq;": { "codepoints": [8941], "characters": "\u22ED" },
        "&nu;": { "codepoints": [957], "characters": "\u03BD" },
        "&num;": { "codepoints": [35], "characters": "\u0023" },
        "&numero;": { "codepoints": [8470], "characters": "\u2116" },
        "&numsp;": { "codepoints": [8199], "characters": "\u2007" },
        "&nvDash;": { "codepoints": [8877], "characters": "\u22AD" },
        "&nvHarr;": { "codepoints": [10500], "characters": "\u2904" },
        "&nvap;": { "codepoints": [8781, 8402], "characters": "\u224D\u20D2" },
        "&nvdash;": { "codepoints": [8876], "characters": "\u22AC" },
        "&nvge;": { "codepoints": [8805, 8402], "characters": "\u2265\u20D2" },
        "&nvgt;": { "codepoints": [62, 8402], "characters": "\u003E\u20D2" },
        "&nvinfin;": { "codepoints": [10718], "characters": "\u29DE" },
        "&nvlArr;": { "codepoints": [10498], "characters": "\u2902" },
        "&nvle;": { "codepoints": [8804, 8402], "characters": "\u2264\u20D2" },
        "&nvlt;": { "codepoints": [60, 8402], "characters": "\u003C\u20D2" },
        "&nvltrie;": { "codepoints": [8884, 8402], "characters": "\u22B4\u20D2" },
        "&nvrArr;": { "codepoints": [10499], "characters": "\u2903" },
        "&nvrtrie;": { "codepoints": [8885, 8402], "characters": "\u22B5\u20D2" },
        "&nvsim;": { "codepoints": [8764, 8402], "characters": "\u223C\u20D2" },
        "&nwArr;": { "codepoints": [8662], "characters": "\u21D6" },
        "&nwarhk;": { "codepoints": [10531], "characters": "\u2923" },
        "&nwarr;": { "codepoints": [8598], "characters": "\u2196" },
        "&nwarrow;": { "codepoints": [8598], "characters": "\u2196" },
        "&nwnear;": { "codepoints": [10535], "characters": "\u2927" },
        "&oS;": { "codepoints": [9416], "characters": "\u24C8" },
        "&oacute": { "codepoints": [243], "characters": "\u00F3" },
        "&oacute;": { "codepoints": [243], "characters": "\u00F3" },
        "&oast;": { "codepoints": [8859], "characters": "\u229B" },
        "&ocir;": { "codepoints": [8858], "characters": "\u229A" },
        "&ocirc": { "codepoints": [244], "characters": "\u00F4" },
        "&ocirc;": { "codepoints": [244], "characters": "\u00F4" },
        "&ocy;": { "codepoints": [1086], "characters": "\u043E" },
        "&odash;": { "codepoints": [8861], "characters": "\u229D" },
        "&odblac;": { "codepoints": [337], "characters": "\u0151" },
        "&odiv;": { "codepoints": [10808], "characters": "\u2A38" },
        "&odot;": { "codepoints": [8857], "characters": "\u2299" },
        "&odsold;": { "codepoints": [10684], "characters": "\u29BC" },
        "&oelig;": { "codepoints": [339], "characters": "\u0153" },
        "&ofcir;": { "codepoints": [10687], "characters": "\u29BF" },
        "&ofr;": { "codepoints": [120108], "characters": "\uD835\uDD2C" },
        "&ogon;": { "codepoints": [731], "characters": "\u02DB" },
        "&ograve": { "codepoints": [242], "characters": "\u00F2" },
        "&ograve;": { "codepoints": [242], "characters": "\u00F2" },
        "&ogt;": { "codepoints": [10689], "characters": "\u29C1" },
        "&ohbar;": { "codepoints": [10677], "characters": "\u29B5" },
        "&ohm;": { "codepoints": [937], "characters": "\u03A9" },
        "&oint;": { "codepoints": [8750], "characters": "\u222E" },
        "&olarr;": { "codepoints": [8634], "characters": "\u21BA" },
        "&olcir;": { "codepoints": [10686], "characters": "\u29BE" },
        "&olcross;": { "codepoints": [10683], "characters": "\u29BB" },
        "&oline;": { "codepoints": [8254], "characters": "\u203E" },
        "&olt;": { "codepoints": [10688], "characters": "\u29C0" },
        "&omacr;": { "codepoints": [333], "characters": "\u014D" },
        "&omega;": { "codepoints": [969], "characters": "\u03C9" },
        "&omicron;": { "codepoints": [959], "characters": "\u03BF" },
        "&omid;": { "codepoints": [10678], "characters": "\u29B6" },
        "&ominus;": { "codepoints": [8854], "characters": "\u2296" },
        "&oopf;": { "codepoints": [120160], "characters": "\uD835\uDD60" },
        "&opar;": { "codepoints": [10679], "characters": "\u29B7" },
        "&operp;": { "codepoints": [10681], "characters": "\u29B9" },
        "&oplus;": { "codepoints": [8853], "characters": "\u2295" },
        "&or;": { "codepoints": [8744], "characters": "\u2228" },
        "&orarr;": { "codepoints": [8635], "characters": "\u21BB" },
        "&ord;": { "codepoints": [10845], "characters": "\u2A5D" },
        "&order;": { "codepoints": [8500], "characters": "\u2134" },
        "&orderof;": { "codepoints": [8500], "characters": "\u2134" },
        "&ordf": { "codepoints": [170], "characters": "\u00AA" },
        "&ordf;": { "codepoints": [170], "characters": "\u00AA" },
        "&ordm": { "codepoints": [186], "characters": "\u00BA" },
        "&ordm;": { "codepoints": [186], "characters": "\u00BA" },
        "&origof;": { "codepoints": [8886], "characters": "\u22B6" },
        "&oror;": { "codepoints": [10838], "characters": "\u2A56" },
        "&orslope;": { "codepoints": [10839], "characters": "\u2A57" },
        "&orv;": { "codepoints": [10843], "characters": "\u2A5B" },
        "&oscr;": { "codepoints": [8500], "characters": "\u2134" },
        "&oslash": { "codepoints": [248], "characters": "\u00F8" },
        "&oslash;": { "codepoints": [248], "characters": "\u00F8" },
        "&osol;": { "codepoints": [8856], "characters": "\u2298" },
        "&otilde": { "codepoints": [245], "characters": "\u00F5" },
        "&otilde;": { "codepoints": [245], "characters": "\u00F5" },
        "&otimes;": { "codepoints": [8855], "characters": "\u2297" },
        "&otimesas;": { "codepoints": [10806], "characters": "\u2A36" },
        "&ouml": { "codepoints": [246], "characters": "\u00F6" },
        "&ouml;": { "codepoints": [246], "characters": "\u00F6" },
        "&ovbar;": { "codepoints": [9021], "characters": "\u233D" },
        "&par;": { "codepoints": [8741], "characters": "\u2225" },
        "&para": { "codepoints": [182], "characters": "\u00B6" },
        "&para;": { "codepoints": [182], "characters": "\u00B6" },
        "&parallel;": { "codepoints": [8741], "characters": "\u2225" },
        "&parsim;": { "codepoints": [10995], "characters": "\u2AF3" },
        "&parsl;": { "codepoints": [11005], "characters": "\u2AFD" },
        "&part;": { "codepoints": [8706], "characters": "\u2202" },
        "&pcy;": { "codepoints": [1087], "characters": "\u043F" },
        "&percnt;": { "codepoints": [37], "characters": "\u0025" },
        "&period;": { "codepoints": [46], "characters": "\u002E" },
        "&permil;": { "codepoints": [8240], "characters": "\u2030" },
        "&perp;": { "codepoints": [8869], "characters": "\u22A5" },
        "&pertenk;": { "codepoints": [8241], "characters": "\u2031" },
        "&pfr;": { "codepoints": [120109], "characters": "\uD835\uDD2D" },
        "&phi;": { "codepoints": [966], "characters": "\u03C6" },
        "&phiv;": { "codepoints": [981], "characters": "\u03D5" },
        "&phmmat;": { "codepoints": [8499], "characters": "\u2133" },
        "&phone;": { "codepoints": [9742], "characters": "\u260E" },
        "&pi;": { "codepoints": [960], "characters": "\u03C0" },
        "&pitchfork;": { "codepoints": [8916], "characters": "\u22D4" },
        "&piv;": { "codepoints": [982], "characters": "\u03D6" },
        "&planck;": { "codepoints": [8463], "characters": "\u210F" },
        "&planckh;": { "codepoints": [8462], "characters": "\u210E" },
        "&plankv;": { "codepoints": [8463], "characters": "\u210F" },
        "&plus;": { "codepoints": [43], "characters": "\u002B" },
        "&plusacir;": { "codepoints": [10787], "characters": "\u2A23" },
        "&plusb;": { "codepoints": [8862], "characters": "\u229E" },
        "&pluscir;": { "codepoints": [10786], "characters": "\u2A22" },
        "&plusdo;": { "codepoints": [8724], "characters": "\u2214" },
        "&plusdu;": { "codepoints": [10789], "characters": "\u2A25" },
        "&pluse;": { "codepoints": [10866], "characters": "\u2A72" },
        "&plusmn": { "codepoints": [177], "characters": "\u00B1" },
        "&plusmn;": { "codepoints": [177], "characters": "\u00B1" },
        "&plussim;": { "codepoints": [10790], "characters": "\u2A26" },
        "&plustwo;": { "codepoints": [10791], "characters": "\u2A27" },
        "&pm;": { "codepoints": [177], "characters": "\u00B1" },
        "&pointint;": { "codepoints": [10773], "characters": "\u2A15" },
        "&popf;": { "codepoints": [120161], "characters": "\uD835\uDD61" },
        "&pound": { "codepoints": [163], "characters": "\u00A3" },
        "&pound;": { "codepoints": [163], "characters": "\u00A3" },
        "&pr;": { "codepoints": [8826], "characters": "\u227A" },
        "&prE;": { "codepoints": [10931], "characters": "\u2AB3" },
        "&prap;": { "codepoints": [10935], "characters": "\u2AB7" },
        "&prcue;": { "codepoints": [8828], "characters": "\u227C" },
        "&pre;": { "codepoints": [10927], "characters": "\u2AAF" },
        "&prec;": { "codepoints": [8826], "characters": "\u227A" },
        "&precapprox;": { "codepoints": [10935], "characters": "\u2AB7" },
        "&preccurlyeq;": { "codepoints": [8828], "characters": "\u227C" },
        "&preceq;": { "codepoints": [10927], "characters": "\u2AAF" },
        "&precnapprox;": { "codepoints": [10937], "characters": "\u2AB9" },
        "&precneqq;": { "codepoints": [10933], "characters": "\u2AB5" },
        "&precnsim;": { "codepoints": [8936], "characters": "\u22E8" },
        "&precsim;": { "codepoints": [8830], "characters": "\u227E" },
        "&prime;": { "codepoints": [8242], "characters": "\u2032" },
        "&primes;": { "codepoints": [8473], "characters": "\u2119" },
        "&prnE;": { "codepoints": [10933], "characters": "\u2AB5" },
        "&prnap;": { "codepoints": [10937], "characters": "\u2AB9" },
        "&prnsim;": { "codepoints": [8936], "characters": "\u22E8" },
        "&prod;": { "codepoints": [8719], "characters": "\u220F" },
        "&profalar;": { "codepoints": [9006], "characters": "\u232E" },
        "&profline;": { "codepoints": [8978], "characters": "\u2312" },
        "&profsurf;": { "codepoints": [8979], "characters": "\u2313" },
        "&prop;": { "codepoints": [8733], "characters": "\u221D" },
        "&propto;": { "codepoints": [8733], "characters": "\u221D" },
        "&prsim;": { "codepoints": [8830], "characters": "\u227E" },
        "&prurel;": { "codepoints": [8880], "characters": "\u22B0" },
        "&pscr;": { "codepoints": [120005], "characters": "\uD835\uDCC5" },
        "&psi;": { "codepoints": [968], "characters": "\u03C8" },
        "&puncsp;": { "codepoints": [8200], "characters": "\u2008" },
        "&qfr;": { "codepoints": [120110], "characters": "\uD835\uDD2E" },
        "&qint;": { "codepoints": [10764], "characters": "\u2A0C" },
        "&qopf;": { "codepoints": [120162], "characters": "\uD835\uDD62" },
        "&qprime;": { "codepoints": [8279], "characters": "\u2057" },
        "&qscr;": { "codepoints": [120006], "characters": "\uD835\uDCC6" },
        "&quaternions;": { "codepoints": [8461], "characters": "\u210D" },
        "&quatint;": { "codepoints": [10774], "characters": "\u2A16" },
        "&quest;": { "codepoints": [63], "characters": "\u003F" },
        "&questeq;": { "codepoints": [8799], "characters": "\u225F" },
        "&quot": { "codepoints": [34], "characters": "\u0022" },
        "&quot;": { "codepoints": [34], "characters": "\u0022" },
        "&rAarr;": { "codepoints": [8667], "characters": "\u21DB" },
        "&rArr;": { "codepoints": [8658], "characters": "\u21D2" },
        "&rAtail;": { "codepoints": [10524], "characters": "\u291C" },
        "&rBarr;": { "codepoints": [10511], "characters": "\u290F" },
        "&rHar;": { "codepoints": [10596], "characters": "\u2964" },
        "&race;": { "codepoints": [8765, 817], "characters": "\u223D\u0331" },
        "&racute;": { "codepoints": [341], "characters": "\u0155" },
        "&radic;": { "codepoints": [8730], "characters": "\u221A" },
        "&raemptyv;": { "codepoints": [10675], "characters": "\u29B3" },
        "&rang;": { "codepoints": [10217], "characters": "\u27E9" },
        "&rangd;": { "codepoints": [10642], "characters": "\u2992" },
        "&range;": { "codepoints": [10661], "characters": "\u29A5" },
        "&rangle;": { "codepoints": [10217], "characters": "\u27E9" },
        "&raquo": { "codepoints": [187], "characters": "\u00BB" },
        "&raquo;": { "codepoints": [187], "characters": "\u00BB" },
        "&rarr;": { "codepoints": [8594], "characters": "\u2192" },
        "&rarrap;": { "codepoints": [10613], "characters": "\u2975" },
        "&rarrb;": { "codepoints": [8677], "characters": "\u21E5" },
        "&rarrbfs;": { "codepoints": [10528], "characters": "\u2920" },
        "&rarrc;": { "codepoints": [10547], "characters": "\u2933" },
        "&rarrfs;": { "codepoints": [10526], "characters": "\u291E" },
        "&rarrhk;": { "codepoints": [8618], "characters": "\u21AA" },
        "&rarrlp;": { "codepoints": [8620], "characters": "\u21AC" },
        "&rarrpl;": { "codepoints": [10565], "characters": "\u2945" },
        "&rarrsim;": { "codepoints": [10612], "characters": "\u2974" },
        "&rarrtl;": { "codepoints": [8611], "characters": "\u21A3" },
        "&rarrw;": { "codepoints": [8605], "characters": "\u219D" },
        "&ratail;": { "codepoints": [10522], "characters": "\u291A" },
        "&ratio;": { "codepoints": [8758], "characters": "\u2236" },
        "&rationals;": { "codepoints": [8474], "characters": "\u211A" },
        "&rbarr;": { "codepoints": [10509], "characters": "\u290D" },
        "&rbbrk;": { "codepoints": [10099], "characters": "\u2773" },
        "&rbrace;": { "codepoints": [125], "characters": "\u007D" },
        "&rbrack;": { "codepoints": [93], "characters": "\u005D" },
        "&rbrke;": { "codepoints": [10636], "characters": "\u298C" },
        "&rbrksld;": { "codepoints": [10638], "characters": "\u298E" },
        "&rbrkslu;": { "codepoints": [10640], "characters": "\u2990" },
        "&rcaron;": { "codepoints": [345], "characters": "\u0159" },
        "&rcedil;": { "codepoints": [343], "characters": "\u0157" },
        "&rceil;": { "codepoints": [8969], "characters": "\u2309" },
        "&rcub;": { "codepoints": [125], "characters": "\u007D" },
        "&rcy;": { "codepoints": [1088], "characters": "\u0440" },
        "&rdca;": { "codepoints": [10551], "characters": "\u2937" },
        "&rdldhar;": { "codepoints": [10601], "characters": "\u2969" },
        "&rdquo;": { "codepoints": [8221], "characters": "\u201D" },
        "&rdquor;": { "codepoints": [8221], "characters": "\u201D" },
        "&rdsh;": { "codepoints": [8627], "characters": "\u21B3" },
        "&real;": { "codepoints": [8476], "characters": "\u211C" },
        "&realine;": { "codepoints": [8475], "characters": "\u211B" },
        "&realpart;": { "codepoints": [8476], "characters": "\u211C" },
        "&reals;": { "codepoints": [8477], "characters": "\u211D" },
        "&rect;": { "codepoints": [9645], "characters": "\u25AD" },
        "&reg": { "codepoints": [174], "characters": "\u00AE" },
        "&reg;": { "codepoints": [174], "characters": "\u00AE" },
        "&rfisht;": { "codepoints": [10621], "characters": "\u297D" },
        "&rfloor;": { "codepoints": [8971], "characters": "\u230B" },
        "&rfr;": { "codepoints": [120111], "characters": "\uD835\uDD2F" },
        "&rhard;": { "codepoints": [8641], "characters": "\u21C1" },
        "&rharu;": { "codepoints": [8640], "characters": "\u21C0" },
        "&rharul;": { "codepoints": [10604], "characters": "\u296C" },
        "&rho;": { "codepoints": [961], "characters": "\u03C1" },
        "&rhov;": { "codepoints": [1009], "characters": "\u03F1" },
        "&rightarrow;": { "codepoints": [8594], "characters": "\u2192" },
        "&rightarrowtail;": { "codepoints": [8611], "characters": "\u21A3" },
        "&rightharpoondown;": { "codepoints": [8641], "characters": "\u21C1" },
        "&rightharpoonup;": { "codepoints": [8640], "characters": "\u21C0" },
        "&rightleftarrows;": { "codepoints": [8644], "characters": "\u21C4" },
        "&rightleftharpoons;": { "codepoints": [8652], "characters": "\u21CC" },
        "&rightrightarrows;": { "codepoints": [8649], "characters": "\u21C9" },
        "&rightsquigarrow;": { "codepoints": [8605], "characters": "\u219D" },
        "&rightthreetimes;": { "codepoints": [8908], "characters": "\u22CC" },
        "&ring;": { "codepoints": [730], "characters": "\u02DA" },
        "&risingdotseq;": { "codepoints": [8787], "characters": "\u2253" },
        "&rlarr;": { "codepoints": [8644], "characters": "\u21C4" },
        "&rlhar;": { "codepoints": [8652], "characters": "\u21CC" },
        "&rlm;": { "codepoints": [8207], "characters": "\u200F" },
        "&rmoust;": { "codepoints": [9137], "characters": "\u23B1" },
        "&rmoustache;": { "codepoints": [9137], "characters": "\u23B1" },
        "&rnmid;": { "codepoints": [10990], "characters": "\u2AEE" },
        "&roang;": { "codepoints": [10221], "characters": "\u27ED" },
        "&roarr;": { "codepoints": [8702], "characters": "\u21FE" },
        "&robrk;": { "codepoints": [10215], "characters": "\u27E7" },
        "&ropar;": { "codepoints": [10630], "characters": "\u2986" },
        "&ropf;": { "codepoints": [120163], "characters": "\uD835\uDD63" },
        "&roplus;": { "codepoints": [10798], "characters": "\u2A2E" },
        "&rotimes;": { "codepoints": [10805], "characters": "\u2A35" },
        "&rpar;": { "codepoints": [41], "characters": "\u0029" },
        "&rpargt;": { "codepoints": [10644], "characters": "\u2994" },
        "&rppolint;": { "codepoints": [10770], "characters": "\u2A12" },
        "&rrarr;": { "codepoints": [8649], "characters": "\u21C9" },
        "&rsaquo;": { "codepoints": [8250], "characters": "\u203A" },
        "&rscr;": { "codepoints": [120007], "characters": "\uD835\uDCC7" },
        "&rsh;": { "codepoints": [8625], "characters": "\u21B1" },
        "&rsqb;": { "codepoints": [93], "characters": "\u005D" },
        "&rsquo;": { "codepoints": [8217], "characters": "\u2019" },
        "&rsquor;": { "codepoints": [8217], "characters": "\u2019" },
        "&rthree;": { "codepoints": [8908], "characters": "\u22CC" },
        "&rtimes;": { "codepoints": [8906], "characters": "\u22CA" },
        "&rtri;": { "codepoints": [9657], "characters": "\u25B9" },
        "&rtrie;": { "codepoints": [8885], "characters": "\u22B5" },
        "&rtrif;": { "codepoints": [9656], "characters": "\u25B8" },
        "&rtriltri;": { "codepoints": [10702], "characters": "\u29CE" },
        "&ruluhar;": { "codepoints": [10600], "characters": "\u2968" },
        "&rx;": { "codepoints": [8478], "characters": "\u211E" },
        "&sacute;": { "codepoints": [347], "characters": "\u015B" },
        "&sbquo;": { "codepoints": [8218], "characters": "\u201A" },
        "&sc;": { "codepoints": [8827], "characters": "\u227B" },
        "&scE;": { "codepoints": [10932], "characters": "\u2AB4" },
        "&scap;": { "codepoints": [10936], "characters": "\u2AB8" },
        "&scaron;": { "codepoints": [353], "characters": "\u0161" },
        "&sccue;": { "codepoints": [8829], "characters": "\u227D" },
        "&sce;": { "codepoints": [10928], "characters": "\u2AB0" },
        "&scedil;": { "codepoints": [351], "characters": "\u015F" },
        "&scirc;": { "codepoints": [349], "characters": "\u015D" },
        "&scnE;": { "codepoints": [10934], "characters": "\u2AB6" },
        "&scnap;": { "codepoints": [10938], "characters": "\u2ABA" },
        "&scnsim;": { "codepoints": [8937], "characters": "\u22E9" },
        "&scpolint;": { "codepoints": [10771], "characters": "\u2A13" },
        "&scsim;": { "codepoints": [8831], "characters": "\u227F" },
        "&scy;": { "codepoints": [1089], "characters": "\u0441" },
        "&sdot;": { "codepoints": [8901], "characters": "\u22C5" },
        "&sdotb;": { "codepoints": [8865], "characters": "\u22A1" },
        "&sdote;": { "codepoints": [10854], "characters": "\u2A66" },
        "&seArr;": { "codepoints": [8664], "characters": "\u21D8" },
        "&searhk;": { "codepoints": [10533], "characters": "\u2925" },
        "&searr;": { "codepoints": [8600], "characters": "\u2198" },
        "&searrow;": { "codepoints": [8600], "characters": "\u2198" },
        "&sect": { "codepoints": [167], "characters": "\u00A7" },
        "&sect;": { "codepoints": [167], "characters": "\u00A7" },
        "&semi;": { "codepoints": [59], "characters": "\u003B" },
        "&seswar;": { "codepoints": [10537], "characters": "\u2929" },
        "&setminus;": { "codepoints": [8726], "characters": "\u2216" },
        "&setmn;": { "codepoints": [8726], "characters": "\u2216" },
        "&sext;": { "codepoints": [10038], "characters": "\u2736" },
        "&sfr;": { "codepoints": [120112], "characters": "\uD835\uDD30" },
        "&sfrown;": { "codepoints": [8994], "characters": "\u2322" },
        "&sharp;": { "codepoints": [9839], "characters": "\u266F" },
        "&shchcy;": { "codepoints": [1097], "characters": "\u0449" },
        "&shcy;": { "codepoints": [1096], "characters": "\u0448" },
        "&shortmid;": { "codepoints": [8739], "characters": "\u2223" },
        "&shortparallel;": { "codepoints": [8741], "characters": "\u2225" },
        "&shy": { "codepoints": [173], "characters": "\u00AD" },
        "&shy;": { "codepoints": [173], "characters": "\u00AD" },
        "&sigma;": { "codepoints": [963], "characters": "\u03C3" },
        "&sigmaf;": { "codepoints": [962], "characters": "\u03C2" },
        "&sigmav;": { "codepoints": [962], "characters": "\u03C2" },
        "&sim;": { "codepoints": [8764], "characters": "\u223C" },
        "&simdot;": { "codepoints": [10858], "characters": "\u2A6A" },
        "&sime;": { "codepoints": [8771], "characters": "\u2243" },
        "&simeq;": { "codepoints": [8771], "characters": "\u2243" },
        "&simg;": { "codepoints": [10910], "characters": "\u2A9E" },
        "&simgE;": { "codepoints": [10912], "characters": "\u2AA0" },
        "&siml;": { "codepoints": [10909], "characters": "\u2A9D" },
        "&simlE;": { "codepoints": [10911], "characters": "\u2A9F" },
        "&simne;": { "codepoints": [8774], "characters": "\u2246" },
        "&simplus;": { "codepoints": [10788], "characters": "\u2A24" },
        "&simrarr;": { "codepoints": [10610], "characters": "\u2972" },
        "&slarr;": { "codepoints": [8592], "characters": "\u2190" },
        "&smallsetminus;": { "codepoints": [8726], "characters": "\u2216" },
        "&smashp;": { "codepoints": [10803], "characters": "\u2A33" },
        "&smeparsl;": { "codepoints": [10724], "characters": "\u29E4" },
        "&smid;": { "codepoints": [8739], "characters": "\u2223" },
        "&smile;": { "codepoints": [8995], "characters": "\u2323" },
        "&smt;": { "codepoints": [10922], "characters": "\u2AAA" },
        "&smte;": { "codepoints": [10924], "characters": "\u2AAC" },
        "&smtes;": { "codepoints": [10924, 65024], "characters": "\u2AAC\uFE00" },
        "&softcy;": { "codepoints": [1100], "characters": "\u044C" },
        "&sol;": { "codepoints": [47], "characters": "\u002F" },
        "&solb;": { "codepoints": [10692], "characters": "\u29C4" },
        "&solbar;": { "codepoints": [9023], "characters": "\u233F" },
        "&sopf;": { "codepoints": [120164], "characters": "\uD835\uDD64" },
        "&spades;": { "codepoints": [9824], "characters": "\u2660" },
        "&spadesuit;": { "codepoints": [9824], "characters": "\u2660" },
        "&spar;": { "codepoints": [8741], "characters": "\u2225" },
        "&sqcap;": { "codepoints": [8851], "characters": "\u2293" },
        "&sqcaps;": { "codepoints": [8851, 65024], "characters": "\u2293\uFE00" },
        "&sqcup;": { "codepoints": [8852], "characters": "\u2294" },
        "&sqcups;": { "codepoints": [8852, 65024], "characters": "\u2294\uFE00" },
        "&sqsub;": { "codepoints": [8847], "characters": "\u228F" },
        "&sqsube;": { "codepoints": [8849], "characters": "\u2291" },
        "&sqsubset;": { "codepoints": [8847], "characters": "\u228F" },
        "&sqsubseteq;": { "codepoints": [8849], "characters": "\u2291" },
        "&sqsup;": { "codepoints": [8848], "characters": "\u2290" },
        "&sqsupe;": { "codepoints": [8850], "characters": "\u2292" },
        "&sqsupset;": { "codepoints": [8848], "characters": "\u2290" },
        "&sqsupseteq;": { "codepoints": [8850], "characters": "\u2292" },
        "&squ;": { "codepoints": [9633], "characters": "\u25A1" },
        "&square;": { "codepoints": [9633], "characters": "\u25A1" },
        "&squarf;": { "codepoints": [9642], "characters": "\u25AA" },
        "&squf;": { "codepoints": [9642], "characters": "\u25AA" },
        "&srarr;": { "codepoints": [8594], "characters": "\u2192" },
        "&sscr;": { "codepoints": [120008], "characters": "\uD835\uDCC8" },
        "&ssetmn;": { "codepoints": [8726], "characters": "\u2216" },
        "&ssmile;": { "codepoints": [8995], "characters": "\u2323" },
        "&sstarf;": { "codepoints": [8902], "characters": "\u22C6" },
        "&star;": { "codepoints": [9734], "characters": "\u2606" },
        "&starf;": { "codepoints": [9733], "characters": "\u2605" },
        "&straightepsilon;": { "codepoints": [1013], "characters": "\u03F5" },
        "&straightphi;": { "codepoints": [981], "characters": "\u03D5" },
        "&strns;": { "codepoints": [175], "characters": "\u00AF" },
        "&sub;": { "codepoints": [8834], "characters": "\u2282" },
        "&subE;": { "codepoints": [10949], "characters": "\u2AC5" },
        "&subdot;": { "codepoints": [10941], "characters": "\u2ABD" },
        "&sube;": { "codepoints": [8838], "characters": "\u2286" },
        "&subedot;": { "codepoints": [10947], "characters": "\u2AC3" },
        "&submult;": { "codepoints": [10945], "characters": "\u2AC1" },
        "&subnE;": { "codepoints": [10955], "characters": "\u2ACB" },
        "&subne;": { "codepoints": [8842], "characters": "\u228A" },
        "&subplus;": { "codepoints": [10943], "characters": "\u2ABF" },
        "&subrarr;": { "codepoints": [10617], "characters": "\u2979" },
        "&subset;": { "codepoints": [8834], "characters": "\u2282" },
        "&subseteq;": { "codepoints": [8838], "characters": "\u2286" },
        "&subseteqq;": { "codepoints": [10949], "characters": "\u2AC5" },
        "&subsetneq;": { "codepoints": [8842], "characters": "\u228A" },
        "&subsetneqq;": { "codepoints": [10955], "characters": "\u2ACB" },
        "&subsim;": { "codepoints": [10951], "characters": "\u2AC7" },
        "&subsub;": { "codepoints": [10965], "characters": "\u2AD5" },
        "&subsup;": { "codepoints": [10963], "characters": "\u2AD3" },
        "&succ;": { "codepoints": [8827], "characters": "\u227B" },
        "&succapprox;": { "codepoints": [10936], "characters": "\u2AB8" },
        "&succcurlyeq;": { "codepoints": [8829], "characters": "\u227D" },
        "&succeq;": { "codepoints": [10928], "characters": "\u2AB0" },
        "&succnapprox;": { "codepoints": [10938], "characters": "\u2ABA" },
        "&succneqq;": { "codepoints": [10934], "characters": "\u2AB6" },
        "&succnsim;": { "codepoints": [8937], "characters": "\u22E9" },
        "&succsim;": { "codepoints": [8831], "characters": "\u227F" },
        "&sum;": { "codepoints": [8721], "characters": "\u2211" },
        "&sung;": { "codepoints": [9834], "characters": "\u266A" },
        "&sup1": { "codepoints": [185], "characters": "\u00B9" },
        "&sup1;": { "codepoints": [185], "characters": "\u00B9" },
        "&sup2": { "codepoints": [178], "characters": "\u00B2" },
        "&sup2;": { "codepoints": [178], "characters": "\u00B2" },
        "&sup3": { "codepoints": [179], "characters": "\u00B3" },
        "&sup3;": { "codepoints": [179], "characters": "\u00B3" },
        "&sup;": { "codepoints": [8835], "characters": "\u2283" },
        "&supE;": { "codepoints": [10950], "characters": "\u2AC6" },
        "&supdot;": { "codepoints": [10942], "characters": "\u2ABE" },
        "&supdsub;": { "codepoints": [10968], "characters": "\u2AD8" },
        "&supe;": { "codepoints": [8839], "characters": "\u2287" },
        "&supedot;": { "codepoints": [10948], "characters": "\u2AC4" },
        "&suphsol;": { "codepoints": [10185], "characters": "\u27C9" },
        "&suphsub;": { "codepoints": [10967], "characters": "\u2AD7" },
        "&suplarr;": { "codepoints": [10619], "characters": "\u297B" },
        "&supmult;": { "codepoints": [10946], "characters": "\u2AC2" },
        "&supnE;": { "codepoints": [10956], "characters": "\u2ACC" },
        "&supne;": { "codepoints": [8843], "characters": "\u228B" },
        "&supplus;": { "codepoints": [10944], "characters": "\u2AC0" },
        "&supset;": { "codepoints": [8835], "characters": "\u2283" },
        "&supseteq;": { "codepoints": [8839], "characters": "\u2287" },
        "&supseteqq;": { "codepoints": [10950], "characters": "\u2AC6" },
        "&supsetneq;": { "codepoints": [8843], "characters": "\u228B" },
        "&supsetneqq;": { "codepoints": [10956], "characters": "\u2ACC" },
        "&supsim;": { "codepoints": [10952], "characters": "\u2AC8" },
        "&supsub;": { "codepoints": [10964], "characters": "\u2AD4" },
        "&supsup;": { "codepoints": [10966], "characters": "\u2AD6" },
        "&swArr;": { "codepoints": [8665], "characters": "\u21D9" },
        "&swarhk;": { "codepoints": [10534], "characters": "\u2926" },
        "&swarr;": { "codepoints": [8601], "characters": "\u2199" },
        "&swarrow;": { "codepoints": [8601], "characters": "\u2199" },
        "&swnwar;": { "codepoints": [10538], "characters": "\u292A" },
        "&szlig": { "codepoints": [223], "characters": "\u00DF" },
        "&szlig;": { "codepoints": [223], "characters": "\u00DF" },
        "&target;": { "codepoints": [8982], "characters": "\u2316" },
        "&tau;": { "codepoints": [964], "characters": "\u03C4" },
        "&tbrk;": { "codepoints": [9140], "characters": "\u23B4" },
        "&tcaron;": { "codepoints": [357], "characters": "\u0165" },
        "&tcedil;": { "codepoints": [355], "characters": "\u0163" },
        "&tcy;": { "codepoints": [1090], "characters": "\u0442" },
        "&tdot;": { "codepoints": [8411], "characters": "\u20DB" },
        "&telrec;": { "codepoints": [8981], "characters": "\u2315" },
        "&tfr;": { "codepoints": [120113], "characters": "\uD835\uDD31" },
        "&there4;": { "codepoints": [8756], "characters": "\u2234" },
        "&therefore;": { "codepoints": [8756], "characters": "\u2234" },
        "&theta;": { "codepoints": [952], "characters": "\u03B8" },
        "&thetasym;": { "codepoints": [977], "characters": "\u03D1" },
        "&thetav;": { "codepoints": [977], "characters": "\u03D1" },
        "&thickapprox;": { "codepoints": [8776], "characters": "\u2248" },
        "&thicksim;": { "codepoints": [8764], "characters": "\u223C" },
        "&thinsp;": { "codepoints": [8201], "characters": "\u2009" },
        "&thkap;": { "codepoints": [8776], "characters": "\u2248" },
        "&thksim;": { "codepoints": [8764], "characters": "\u223C" },
        "&thorn": { "codepoints": [254], "characters": "\u00FE" },
        "&thorn;": { "codepoints": [254], "characters": "\u00FE" },
        "&tilde;": { "codepoints": [732], "characters": "\u02DC" },
        "&times": { "codepoints": [215], "characters": "\u00D7" },
        "&times;": { "codepoints": [215], "characters": "\u00D7" },
        "&timesb;": { "codepoints": [8864], "characters": "\u22A0" },
        "&timesbar;": { "codepoints": [10801], "characters": "\u2A31" },
        "&timesd;": { "codepoints": [10800], "characters": "\u2A30" },
        "&tint;": { "codepoints": [8749], "characters": "\u222D" },
        "&toea;": { "codepoints": [10536], "characters": "\u2928" },
        "&top;": { "codepoints": [8868], "characters": "\u22A4" },
        "&topbot;": { "codepoints": [9014], "characters": "\u2336" },
        "&topcir;": { "codepoints": [10993], "characters": "\u2AF1" },
        "&topf;": { "codepoints": [120165], "characters": "\uD835\uDD65" },
        "&topfork;": { "codepoints": [10970], "characters": "\u2ADA" },
        "&tosa;": { "codepoints": [10537], "characters": "\u2929" },
        "&tprime;": { "codepoints": [8244], "characters": "\u2034" },
        "&trade;": { "codepoints": [8482], "characters": "\u2122" },
        "&triangle;": { "codepoints": [9653], "characters": "\u25B5" },
        "&triangledown;": { "codepoints": [9663], "characters": "\u25BF" },
        "&triangleleft;": { "codepoints": [9667], "characters": "\u25C3" },
        "&trianglelefteq;": { "codepoints": [8884], "characters": "\u22B4" },
        "&triangleq;": { "codepoints": [8796], "characters": "\u225C" },
        "&triangleright;": { "codepoints": [9657], "characters": "\u25B9" },
        "&trianglerighteq;": { "codepoints": [8885], "characters": "\u22B5" },
        "&tridot;": { "codepoints": [9708], "characters": "\u25EC" },
        "&trie;": { "codepoints": [8796], "characters": "\u225C" },
        "&triminus;": { "codepoints": [10810], "characters": "\u2A3A" },
        "&triplus;": { "codepoints": [10809], "characters": "\u2A39" },
        "&trisb;": { "codepoints": [10701], "characters": "\u29CD" },
        "&tritime;": { "codepoints": [10811], "characters": "\u2A3B" },
        "&trpezium;": { "codepoints": [9186], "characters": "\u23E2" },
        "&tscr;": { "codepoints": [120009], "characters": "\uD835\uDCC9" },
        "&tscy;": { "codepoints": [1094], "characters": "\u0446" },
        "&tshcy;": { "codepoints": [1115], "characters": "\u045B" },
        "&tstrok;": { "codepoints": [359], "characters": "\u0167" },
        "&twixt;": { "codepoints": [8812], "characters": "\u226C" },
        "&twoheadleftarrow;": { "codepoints": [8606], "characters": "\u219E" },
        "&twoheadrightarrow;": { "codepoints": [8608], "characters": "\u21A0" },
        "&uArr;": { "codepoints": [8657], "characters": "\u21D1" },
        "&uHar;": { "codepoints": [10595], "characters": "\u2963" },
        "&uacute": { "codepoints": [250], "characters": "\u00FA" },
        "&uacute;": { "codepoints": [250], "characters": "\u00FA" },
        "&uarr;": { "codepoints": [8593], "characters": "\u2191" },
        "&ubrcy;": { "codepoints": [1118], "characters": "\u045E" },
        "&ubreve;": { "codepoints": [365], "characters": "\u016D" },
        "&ucirc": { "codepoints": [251], "characters": "\u00FB" },
        "&ucirc;": { "codepoints": [251], "characters": "\u00FB" },
        "&ucy;": { "codepoints": [1091], "characters": "\u0443" },
        "&udarr;": { "codepoints": [8645], "characters": "\u21C5" },
        "&udblac;": { "codepoints": [369], "characters": "\u0171" },
        "&udhar;": { "codepoints": [10606], "characters": "\u296E" },
        "&ufisht;": { "codepoints": [10622], "characters": "\u297E" },
        "&ufr;": { "codepoints": [120114], "characters": "\uD835\uDD32" },
        "&ugrave": { "codepoints": [249], "characters": "\u00F9" },
        "&ugrave;": { "codepoints": [249], "characters": "\u00F9" },
        "&uharl;": { "codepoints": [8639], "characters": "\u21BF" },
        "&uharr;": { "codepoints": [8638], "characters": "\u21BE" },
        "&uhblk;": { "codepoints": [9600], "characters": "\u2580" },
        "&ulcorn;": { "codepoints": [8988], "characters": "\u231C" },
        "&ulcorner;": { "codepoints": [8988], "characters": "\u231C" },
        "&ulcrop;": { "codepoints": [8975], "characters": "\u230F" },
        "&ultri;": { "codepoints": [9720], "characters": "\u25F8" },
        "&umacr;": { "codepoints": [363], "characters": "\u016B" },
        "&uml": { "codepoints": [168], "characters": "\u00A8" },
        "&uml;": { "codepoints": [168], "characters": "\u00A8" },
        "&uogon;": { "codepoints": [371], "characters": "\u0173" },
        "&uopf;": { "codepoints": [120166], "characters": "\uD835\uDD66" },
        "&uparrow;": { "codepoints": [8593], "characters": "\u2191" },
        "&updownarrow;": { "codepoints": [8597], "characters": "\u2195" },
        "&upharpoonleft;": { "codepoints": [8639], "characters": "\u21BF" },
        "&upharpoonright;": { "codepoints": [8638], "characters": "\u21BE" },
        "&uplus;": { "codepoints": [8846], "characters": "\u228E" },
        "&upsi;": { "codepoints": [965], "characters": "\u03C5" },
        "&upsih;": { "codepoints": [978], "characters": "\u03D2" },
        "&upsilon;": { "codepoints": [965], "characters": "\u03C5" },
        "&upuparrows;": { "codepoints": [8648], "characters": "\u21C8" },
        "&urcorn;": { "codepoints": [8989], "characters": "\u231D" },
        "&urcorner;": { "codepoints": [8989], "characters": "\u231D" },
        "&urcrop;": { "codepoints": [8974], "characters": "\u230E" },
        "&uring;": { "codepoints": [367], "characters": "\u016F" },
        "&urtri;": { "codepoints": [9721], "characters": "\u25F9" },
        "&uscr;": { "codepoints": [120010], "characters": "\uD835\uDCCA" },
        "&utdot;": { "codepoints": [8944], "characters": "\u22F0" },
        "&utilde;": { "codepoints": [361], "characters": "\u0169" },
        "&utri;": { "codepoints": [9653], "characters": "\u25B5" },
        "&utrif;": { "codepoints": [9652], "characters": "\u25B4" },
        "&uuarr;": { "codepoints": [8648], "characters": "\u21C8" },
        "&uuml": { "codepoints": [252], "characters": "\u00FC" },
        "&uuml;": { "codepoints": [252], "characters": "\u00FC" },
        "&uwangle;": { "codepoints": [10663], "characters": "\u29A7" },
        "&vArr;": { "codepoints": [8661], "characters": "\u21D5" },
        "&vBar;": { "codepoints": [10984], "characters": "\u2AE8" },
        "&vBarv;": { "codepoints": [10985], "characters": "\u2AE9" },
        "&vDash;": { "codepoints": [8872], "characters": "\u22A8" },
        "&vangrt;": { "codepoints": [10652], "characters": "\u299C" },
        "&varepsilon;": { "codepoints": [1013], "characters": "\u03F5" },
        "&varkappa;": { "codepoints": [1008], "characters": "\u03F0" },
        "&varnothing;": { "codepoints": [8709], "characters": "\u2205" },
        "&varphi;": { "codepoints": [981], "characters": "\u03D5" },
        "&varpi;": { "codepoints": [982], "characters": "\u03D6" },
        "&varpropto;": { "codepoints": [8733], "characters": "\u221D" },
        "&varr;": { "codepoints": [8597], "characters": "\u2195" },
        "&varrho;": { "codepoints": [1009], "characters": "\u03F1" },
        "&varsigma;": { "codepoints": [962], "characters": "\u03C2" },
        "&varsubsetneq;": { "codepoints": [8842, 65024], "characters": "\u228A\uFE00" },
        "&varsubsetneqq;": { "codepoints": [10955, 65024], "characters": "\u2ACB\uFE00" },
        "&varsupsetneq;": { "codepoints": [8843, 65024], "characters": "\u228B\uFE00" },
        "&varsupsetneqq;": { "codepoints": [10956, 65024], "characters": "\u2ACC\uFE00" },
        "&vartheta;": { "codepoints": [977], "characters": "\u03D1" },
        "&vartriangleleft;": { "codepoints": [8882], "characters": "\u22B2" },
        "&vartriangleright;": { "codepoints": [8883], "characters": "\u22B3" },
        "&vcy;": { "codepoints": [1074], "characters": "\u0432" },
        "&vdash;": { "codepoints": [8866], "characters": "\u22A2" },
        "&vee;": { "codepoints": [8744], "characters": "\u2228" },
        "&veebar;": { "codepoints": [8891], "characters": "\u22BB" },
        "&veeeq;": { "codepoints": [8794], "characters": "\u225A" },
        "&vellip;": { "codepoints": [8942], "characters": "\u22EE" },
        "&verbar;": { "codepoints": [124], "characters": "\u007C" },
        "&vert;": { "codepoints": [124], "characters": "\u007C" },
        "&vfr;": { "codepoints": [120115], "characters": "\uD835\uDD33" },
        "&vltri;": { "codepoints": [8882], "characters": "\u22B2" },
        "&vnsub;": { "codepoints": [8834, 8402], "characters": "\u2282\u20D2" },
        "&vnsup;": { "codepoints": [8835, 8402], "characters": "\u2283\u20D2" },
        "&vopf;": { "codepoints": [120167], "characters": "\uD835\uDD67" },
        "&vprop;": { "codepoints": [8733], "characters": "\u221D" },
        "&vrtri;": { "codepoints": [8883], "characters": "\u22B3" },
        "&vscr;": { "codepoints": [120011], "characters": "\uD835\uDCCB" },
        "&vsubnE;": { "codepoints": [10955, 65024], "characters": "\u2ACB\uFE00" },
        "&vsubne;": { "codepoints": [8842, 65024], "characters": "\u228A\uFE00" },
        "&vsupnE;": { "codepoints": [10956, 65024], "characters": "\u2ACC\uFE00" },
        "&vsupne;": { "codepoints": [8843, 65024], "characters": "\u228B\uFE00" },
        "&vzigzag;": { "codepoints": [10650], "characters": "\u299A" },
        "&wcirc;": { "codepoints": [373], "characters": "\u0175" },
        "&wedbar;": { "codepoints": [10847], "characters": "\u2A5F" },
        "&wedge;": { "codepoints": [8743], "characters": "\u2227" },
        "&wedgeq;": { "codepoints": [8793], "characters": "\u2259" },
        "&weierp;": { "codepoints": [8472], "characters": "\u2118" },
        "&wfr;": { "codepoints": [120116], "characters": "\uD835\uDD34" },
        "&wopf;": { "codepoints": [120168], "characters": "\uD835\uDD68" },
        "&wp;": { "codepoints": [8472], "characters": "\u2118" },
        "&wr;": { "codepoints": [8768], "characters": "\u2240" },
        "&wreath;": { "codepoints": [8768], "characters": "\u2240" },
        "&wscr;": { "codepoints": [120012], "characters": "\uD835\uDCCC" },
        "&xcap;": { "codepoints": [8898], "characters": "\u22C2" },
        "&xcirc;": { "codepoints": [9711], "characters": "\u25EF" },
        "&xcup;": { "codepoints": [8899], "characters": "\u22C3" },
        "&xdtri;": { "codepoints": [9661], "characters": "\u25BD" },
        "&xfr;": { "codepoints": [120117], "characters": "\uD835\uDD35" },
        "&xhArr;": { "codepoints": [10234], "characters": "\u27FA" },
        "&xharr;": { "codepoints": [10231], "characters": "\u27F7" },
        "&xi;": { "codepoints": [958], "characters": "\u03BE" },
        "&xlArr;": { "codepoints": [10232], "characters": "\u27F8" },
        "&xlarr;": { "codepoints": [10229], "characters": "\u27F5" },
        "&xmap;": { "codepoints": [10236], "characters": "\u27FC" },
        "&xnis;": { "codepoints": [8955], "characters": "\u22FB" },
        "&xodot;": { "codepoints": [10752], "characters": "\u2A00" },
        "&xopf;": { "codepoints": [120169], "characters": "\uD835\uDD69" },
        "&xoplus;": { "codepoints": [10753], "characters": "\u2A01" },
        "&xotime;": { "codepoints": [10754], "characters": "\u2A02" },
        "&xrArr;": { "codepoints": [10233], "characters": "\u27F9" },
        "&xrarr;": { "codepoints": [10230], "characters": "\u27F6" },
        "&xscr;": { "codepoints": [120013], "characters": "\uD835\uDCCD" },
        "&xsqcup;": { "codepoints": [10758], "characters": "\u2A06" },
        "&xuplus;": { "codepoints": [10756], "characters": "\u2A04" },
        "&xutri;": { "codepoints": [9651], "characters": "\u25B3" },
        "&xvee;": { "codepoints": [8897], "characters": "\u22C1" },
        "&xwedge;": { "codepoints": [8896], "characters": "\u22C0" },
        "&yacute": { "codepoints": [253], "characters": "\u00FD" },
        "&yacute;": { "codepoints": [253], "characters": "\u00FD" },
        "&yacy;": { "codepoints": [1103], "characters": "\u044F" },
        "&ycirc;": { "codepoints": [375], "characters": "\u0177" },
        "&ycy;": { "codepoints": [1099], "characters": "\u044B" },
        "&yen": { "codepoints": [165], "characters": "\u00A5" },
        "&yen;": { "codepoints": [165], "characters": "\u00A5" },
        "&yfr;": { "codepoints": [120118], "characters": "\uD835\uDD36" },
        "&yicy;": { "codepoints": [1111], "characters": "\u0457" },
        "&yopf;": { "codepoints": [120170], "characters": "\uD835\uDD6A" },
        "&yscr;": { "codepoints": [120014], "characters": "\uD835\uDCCE" },
        "&yucy;": { "codepoints": [1102], "characters": "\u044E" },
        "&yuml": { "codepoints": [255], "characters": "\u00FF" },
        "&yuml;": { "codepoints": [255], "characters": "\u00FF" },
        "&zacute;": { "codepoints": [378], "characters": "\u017A" },
        "&zcaron;": { "codepoints": [382], "characters": "\u017E" },
        "&zcy;": { "codepoints": [1079], "characters": "\u0437" },
        "&zdot;": { "codepoints": [380], "characters": "\u017C" },
        "&zeetrf;": { "codepoints": [8488], "characters": "\u2128" },
        "&zeta;": { "codepoints": [950], "characters": "\u03B6" },
        "&zfr;": { "codepoints": [120119], "characters": "\uD835\uDD37" },
        "&zhcy;": { "codepoints": [1078], "characters": "\u0436" },
        "&zigrarr;": { "codepoints": [8669], "characters": "\u21DD" },
        "&zopf;": { "codepoints": [120171], "characters": "\uD835\uDD6B" },
        "&zscr;": { "codepoints": [120015], "characters": "\uD835\uDCCF" },
        "&zwj;": { "codepoints": [8205], "characters": "\u200D" },
        "&zwnj;": { "codepoints": [8204], "characters": "\u200C" }
      }
    "#;

    pub fn new(source: String) -> Self { 
        let lexer = Lexer::new(String::from(source));
        let tokenization_state = HTMLTokenizerState::Data;
        let html_tokens = Vec::new();
        let reconsume_current_input_character = false;
        let temporary_buffer = String::from("");
        let attribute_buffer = AttributeBuffer { name: String::from(""), value: String::from("") };
        let return_state = HTMLTokenizerState::Data;
        let mut named_character_references = Vec::new();
        let character_reference_code = 0;
        let html_document_parser = HTMLDocumentParser::new();
        let current_html_token = None;

        // Table reference can be found in this section: https://html.spec.whatwg.org/#numeric-character-reference-end-state
        let number_character_references = HashMap::from([
            (0x80, 0x20AC),
            (0x82, 0x201A),
            (0x83, 0x0192),
            (0x84, 0x201E),
            (0x85, 0x2026),
            (0x86, 0x2020),
            (0x87, 0x2021),
            (0x88, 0x02C6),
            (0x89, 0x2030),
            (0x8A, 0x0160),
            (0x8B, 0x2039),
            (0x8C, 0x0152),
            (0x8E, 0x017D),
            (0x91, 0x2018),
            (0x92, 0x2019),
            (0x93, 0x201C),
            (0x94, 0x201D),
            (0x95, 0x2022),
            (0x96, 0x2013),
            (0x97, 0x2014),
            (0x98, 0x02DC),
            (0x99, 0x2122),
            (0x9A, 0x0161),
            (0x9B, 0x203A),
            (0x9C, 0x0153),
            (0x9E, 0x017E),
            (0x9F, 0x0178)
        ]);

        let value: Value = serde_json::from_str(Tokenizer::NAMED_CHARACTER_REFERENCE_JSON_DATA).unwrap();
        
        for obj in value.as_object().unwrap() { 
            named_character_references.push(NamedCharacterReferenceObject { character_reference: obj.0.to_string(), codepoints: obj.1["codepoints"].to_string(), characters: obj.1["characters"].to_string().replacen("\"", "", 2) });
        }

        Self { lexer, tokenization_state, html_tokens, reconsume_current_input_character, temporary_buffer, attribute_buffer, return_state, named_character_references, character_reference_code, number_character_references, html_document_parser, current_html_token }
    }

    pub fn start(&mut self) { 
        let mut next_input_character = self.next_input_character();

        while next_input_character.is_some() { 
            self.next_token(next_input_character);

            if !self.reconsume_current_input_character { 
                next_input_character = self.next_input_character();
            } else {
                next_input_character = self.current_input_character();
                self.reconsume_current_input_character = false;
            }
        }
        

        // The tokenizer has reached the end of file so consume None to produce an end of file token
        if next_input_character.is_none() { 
            self.next_token(None);
        }

        self.html_document_parser.print_document();
    }

    fn next_token(&mut self, current_input_character: Option<char>) { 
            match self.tokenization_state { 
                HTMLTokenizerState::Data => { 
                    match current_input_character { 
                        Some(charcater) => { 
                            match charcater { 
                                '&' => { 
                                    self.return_state = HTMLTokenizerState::Data;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CharacterReference)
                                }
                                '<' => { 
                                    self.switch_to_tokenization_state(HTMLTokenizerState::TagOpen);
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                }
                            }
                        }
                        None => { 
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                        }
                    }
                }
                HTMLTokenizerState::RCData => {
                    match current_input_character { 
                        Some(charcater) => { 
                            match charcater { 
                                '&' => { 
                                    self.return_state = HTMLTokenizerState::RCData;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CharacterReference)
                                }
                                '<' => { 
                                    self.switch_to_tokenization_state(HTMLTokenizerState::RcdataLessThanSign);
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                    self.emit_current_html_token();
                                }
                            }
                        }
                        None => { 
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                },
                HTMLTokenizerState::RawText => {
                    match current_input_character { 
                        Some(charcater) => { 
                            match charcater { 
                                '<' => { 
                                    self.switch_to_tokenization_state(HTMLTokenizerState::RawTextLessThanSign)
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                }
                            }
                        }
                        None => { 
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                HTMLTokenizerState::ScriptData => {
                    match current_input_character {
                        Some(charcater) => {
                            match charcater {
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataLessThanSign)
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                    self.emit_current_html_token();
                                }
                            }
                        }
                        None => {
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                HTMLTokenizerState::PlainText => {
                    match current_input_character {
                        Some(charcater) => {
                            match charcater {
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(charcater));
                                    self.emit_current_html_token();
                                }
                            }
                        }
                        None => {
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#tag-open-state
                HTMLTokenizerState::TagOpen => {
                    match current_input_character {

                        Some(charcater) => {
                            match charcater {
                                '!' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::MarkupDeclarationOpen);
                                },
                                '/' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::EndTagOpen);
                                },
                                // https://infra.spec.whatwg.org/#ascii-alpha
                                'A'..='Z' | 'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_start_tag_html_token());
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::TagName);
                                },
                                '?' => {
                                    // https://html.spec.whatwg.org/#parse-error-unexpected-question-mark-instead-of-tag-name
                                    Tokenizer::parse_error(ParseError::UnexpectedQuestionMarkInsteadOfTagName);

                                    self.html_tokens.push(Tokenizer::create_comment_html_token(String::from("")));
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusComment);
                                },
                                _ => {
                                   // https://html.spec.whatwg.org/#parse-error-invalid-first-character-of-tag-name
                                   Tokenizer::parse_error(ParseError::InvalidFirstCharacterOfTagName);

                                   self.html_tokens.push((Tokenizer::create_character_html_token('<')));
                                   self.emit_current_html_token();
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Data);

                                },
                            }
                        }
                        None => {
                            // https://html.spec.whatwg.org/#parse-error-eof-before-tag-name
                            Tokenizer::parse_error(ParseError::EndOfFileBeforeTagName);

                            self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                            self.emit_current_html_token();

                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#end-tag-open-state
                HTMLTokenizerState::EndTagOpen => {
                    match current_input_character {
                        Some(charcater) => {
                            match charcater {
                                // https://infra.spec.whatwg.org/#ascii-alpha
                                'A'..='Z' | 'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_end_tag_html_token());
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::TagName);
                                },
                                '>' => {
                                    // https://html.spec.whatwg.org/#parse-error-missing-end-tag-name
                                    Tokenizer::parse_error(ParseError::MissingEndTagName);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                },
                                _ => {
                                    // https://html.spec.whatwg.org/#parse-error-invalid-first-character-of-tag-name
                                    Tokenizer::parse_error(ParseError::InvalidFirstCharacterOfTagName);

                                    self.html_tokens.push(Tokenizer::create_comment_html_token(String::from("")));
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusComment);
                                }
                            }
                        }
                        None => {
                            // https://html.spec.whatwg.org/#parse-error-eof-before-tag-name
                            Tokenizer::parse_error(ParseError::EndOfFileBeforeTagName);

                            self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                            self.emit_current_html_token();

                            self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                            self.emit_current_html_token();

                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#tag-name-state
                HTMLTokenizerState::TagName => {
                    match current_input_character {
                        Some(charcater) => {
                            match charcater {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName)
                                },
                                '/' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag);
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                'A'..='Z'  => {
                                    let lowercase_current_input_character = charcater.to_ascii_lowercase();
                                    self.current_tag_token().tag_name.push(lowercase_current_input_character);
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().tag_name.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                _ => {
                                    self.current_tag_token().tag_name.push(charcater);
                                }
                            }
                        }
                        None => {
                            // https://html.spec.whatwg.org/#parse-error-eof-in-tag
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);

                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#rcdata-less-than-sign-state
                HTMLTokenizerState::RcdataLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '/' => {
                                    self.temporary_buffer = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::RcdataEndTagOpen);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    self.switch_to_tokenization_state(HTMLTokenizerState::RCData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#rcdata-end-tag-open-state
                HTMLTokenizerState::RcdataEndTagOpen => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                'A'..='Z' |  'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_end_tag_html_token());

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::RCData);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::RCData);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#rcdata-end-tag-name-state
                HTMLTokenizerState::RcdataEndTagName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '/' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag)
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '>' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                        self.emit_current_html_token();
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                'A'..='Z' => {
                                    let lowercase_current_input_character = character.to_ascii_lowercase();
                                    self.current_tag_token().tag_name.push(lowercase_current_input_character);
                                    self.temporary_buffer.push(lowercase_current_input_character);
                                },
                                'a'..='z' => {
                                    self.current_tag_token().tag_name.push(character);
                                    self.temporary_buffer.push(character);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    // Create a copy of the characters to avoid borrowing self during iteration
                                    let characters: Vec<char> = self.temporary_buffer.chars().collect();
                                    for character in characters {
                                        self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                        self.emit_current_html_token();
                                    }

                                    self.switch_to_tokenization_state(HTMLTokenizerState::RCData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#rawtext-less-than-sign-state
                HTMLTokenizerState::RawTextLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '/' => {
                                    self.temporary_buffer = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::RawTextEndTagOpen);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();


                                    self.switch_to_tokenization_state(HTMLTokenizerState::RawText);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#rawtext-end-tag-open-state
                HTMLTokenizerState::RawTextEndTagOpen => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                'A'..='Z' |  'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_end_tag_html_token());

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::RawTextEndTagName);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::RawText);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#rawtext-end-tag-name-state
                HTMLTokenizerState::RawTextEndTagName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '/' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag)
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '>' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                        self.emit_current_html_token();
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                'A'..='Z' => {
                                    let lowercase_current_input_character = character.to_ascii_lowercase();
                                    self.current_tag_token().tag_name.push(lowercase_current_input_character);
                                    self.temporary_buffer.push(lowercase_current_input_character);
                                },
                                'a'..='z' => {
                                    self.current_tag_token().tag_name.push(character);
                                    self.temporary_buffer.push(character);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    let characters: Vec<char> = self.temporary_buffer.chars().collect();
                                    for character in characters {
                                        self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                        self.emit_current_html_token();
                                    }

                                    self.switch_to_tokenization_state(HTMLTokenizerState::RawText);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-less-than-sign-state
                HTMLTokenizerState::ScriptDataLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '/' => {
                                    self.temporary_buffer = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEndTagOpen);
                                },
                                '!' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapeStart);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('!'));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                         None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-end-tag-open-state
                HTMLTokenizerState::ScriptDataEndTagOpen => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                'A'..='Z' | 'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_end_tag_html_token());
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataEndTagName);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptData);
                                }
                            }
                        }
                         None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-end-tag-name-state
                HTMLTokenizerState::ScriptDataEndTagName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '/' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag)
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '>' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                        self.emit_current_html_token();
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                'A'..='Z' => {
                                    let lowercase_current_input_character = character.to_ascii_lowercase();
                                    self.current_tag_token().tag_name.push(lowercase_current_input_character);
                                    self.temporary_buffer.push(lowercase_current_input_character);
                                },
                                'a'..='z' => {
                                    self.current_tag_token().tag_name.push(character);
                                    self.temporary_buffer.push(character);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    let characters: Vec<char> = self.temporary_buffer.chars().collect();
                                    for character in characters {
                                        self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                        self.emit_current_html_token();
                                    }

                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escape-start-state
                HTMLTokenizerState::ScriptDataEscapeStart => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapeStartDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                         None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escaped-state
                HTMLTokenizerState::ScriptDataEscapeStartDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedDashDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }

                        None => ()
                    }

                }
                // https://html.spec.whatwg.org/#script-data-escaped-state
                HTMLTokenizerState::ScriptDataEscaped => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedLessThanSign);
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                }
                            }
                        }


                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escaped-dash-state
                HTMLTokenizerState::ScriptDataEscapedDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedDashDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedLessThanSign);
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped)
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                }
                            }
                        }


                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escaped-dash-dash-state
                HTMLTokenizerState::ScriptDataEscapedDashDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedLessThanSign);
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('>'));
                                    self.emit_current_html_token();
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped)
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                }
                            }
                        }

                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escaped-end-tag-open-state
                HTMLTokenizerState::ScriptDataEscapedLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '/' => {
                                    self.temporary_buffer = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscapedEndTagOpen);

                                },
                                'A'..='Z' | 'a'..='z' => {
                                    self.temporary_buffer = String::from("");
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapeStart);
                                    self.reconsume_current_input_character();
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }

                }
                HTMLTokenizerState::ScriptDataEscapedEndTagOpen => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                'A'..='Z' | 'a'..='z' => {
                                    self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                                    self.emit_current_html_token();

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataEscapedEndTagName);
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-escaped-end-tag-name-state
                HTMLTokenizerState::ScriptDataEscapedEndTagName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '/' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag)
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                '>' => {
                                    if self.appropriate_end_tag_token() {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                        self.emit_current_html_token();
                                    } else {
                                        todo!();
                                        // TODO: This should go to the `Anything else` _ => match statemant
                                    }
                                },
                                'A'..='Z' => {
                                    let lowercase_current_input_character = character.to_ascii_lowercase();
                                    self.current_tag_token().tag_name.push(lowercase_current_input_character);
                                    self.temporary_buffer.push(lowercase_current_input_character);
                                },
                                'a'..='z' => {
                                    self.current_tag_token().tag_name.push(character);
                                    self.temporary_buffer.push(character);
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();

                                    self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();

                                    let characters: Vec<char> = self.temporary_buffer.chars().collect();
                                    for character in characters {
                                        self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                        self.emit_current_html_token();
                                    }

                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                    self.reconsume_current_input_character();
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escape-start-state
                HTMLTokenizerState::ScriptDataDoubleEscapeStart => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' | '/' | '>' => {
                                    if self.temporary_buffer == "script" {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    } else {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                    }

                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                'A'..='Z' => {
                                    self.temporary_buffer.push(character.to_ascii_lowercase());
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                'a'..='z' => {
                                    self.temporary_buffer.push(character);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escaped-state
                HTMLTokenizerState::ScriptDataDoubleEscaped => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapedDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapedLessThanSign);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                }
                            }
                        }

                         None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                             self.emit_current_html_token();
                         }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escaped-dash-state
                HTMLTokenizerState::ScriptDataDoubleEscapedDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapedDashDash);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapedLessThanSign);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                }
                            }
                        }

                         None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                             self.emit_current_html_token();
                         }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escaped-dash-dash-state
                HTMLTokenizerState::ScriptDataDoubleEscapedDashDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token('-'));
                                    self.emit_current_html_token();
                                },
                                '<' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapedLessThanSign);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('<'));
                                    self.emit_current_html_token();
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptData);
                                    self.html_tokens.push(Tokenizer::create_character_html_token('>'));
                                    self.emit_current_html_token();
                                }
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                }
                            }
                        }

                         None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                             self.emit_current_html_token();
                         }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escaped-less-than-sign-state
                HTMLTokenizerState::ScriptDataDoubleEscapedLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '/' => {
                                   self.temporary_buffer = String::from("");
                                   self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscapeEnd);
                                   self.html_tokens.push(Tokenizer::create_character_html_token('/'));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                }
                            }
                        }

                         None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInScriptHtmlCommentLikeText);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                             self.emit_current_html_token();
                         }
                    }
                }
                // https://html.spec.whatwg.org/#script-data-double-escape-end-state
                HTMLTokenizerState::ScriptDataDoubleEscapeEnd => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' | '/' | '>' => {
                                    if self.temporary_buffer == "script" {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataEscaped);
                                    } else {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                    }

                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                'A'..='Z' => {
                                    self.temporary_buffer.push(character.to_ascii_lowercase());
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                'a'..='z' => {
                                    self.temporary_buffer.push(character);
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::ScriptDataDoubleEscaped);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#before-attribute-name-state
                HTMLTokenizerState::BeforeAttributeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                   // Ignore these characters
                                },
                                '/' | '>' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::AfterAttributeName)
                                 }
                                '=' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedEqualsSignBeforeAttributeName);

                                    self.switch_to_tokenization_state(HTMLTokenizerState::AttributeName)
                                }
                                _ => {
                                    self.attribute_buffer = AttributeBuffer { name: String::from(""), value: String::from("") };

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::AttributeName)
                                }
                            }
                        }
                        None => {
                            self.reconsume_in_tokenization_state(HTMLTokenizerState::AfterAttributeName)
                        }
                    }
                }
                // https://html.spec.whatwg.org/#attribute-name-state
                HTMLTokenizerState::AttributeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' ' | '/' | '>' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::AfterAttributeName)
                                },
                                '=' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeValue);
                                },
                                'A'..='Z' => {
                                    self.attribute_buffer.name.push(character.to_ascii_lowercase());
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.attribute_buffer.name.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                '"' | '\'' | '<' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedCharacterInAttributeName);
                                    self.attribute_buffer.name.push(character);
                                }
                                _ => {
                                    self.attribute_buffer.name.push(character);
                                }
                            }
                        }
                        None => {
                            self.reconsume_in_tokenization_state(HTMLTokenizerState::AfterAttributeName)
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-attribute-name-state
                HTMLTokenizerState::AfterAttributeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                '/' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag);
                                }
                                '=' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeValue);
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    // Emitted current tag token

                                     /* When the user agent leaves the attribute name state (and before emitting the tag token, if appropriate), the complete attribute's name must be compared to the other attributes on the same token;
                                    if there is already an attribute on the token with the exact same name, then this is a duplicate-attribute parse error and the new attribute must be removed from the token. */
                                    let add_attribute_result = self.add_attribute_to_current_tag_token(self.attribute_buffer.name.to_string(), self.attribute_buffer.value.to_string());

                                    if add_attribute_result.is_err() {
                                        Tokenizer::parse_error(ParseError::DuplicateAttribute);
                                    }
                                },
                                _ => {
                                    self.attribute_buffer = AttributeBuffer { name: String::from(""), value: String::from("") };

                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::AttributeName)
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#before-attribute-value-state
                HTMLTokenizerState::BeforeAttributeValue => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                '"' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AttributeValueDoubleQuoted);
                                }
                                '\'' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AttributeValueSingleQuoted);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingAttributeValue);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data)
                                    // Emitted current tag token
                                },
                                _ => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::AttributeValueUnquoted)
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#attribute-value-(double-quoted)-state
                HTMLTokenizerState::AttributeValueDoubleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '"' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterAttributeValueQuoted);

                                    /* When the user agent leaves the attribute name state (and before emitting the tag token, if appropriate), the complete attribute's name must be compared to the other attributes on the same token;
                                    if there is already an attribute on the token with the exact same name, then this is a duplicate-attribute parse error and the new attribute must be removed from the token. */
                                    let add_attribute_result = self.add_attribute_to_current_tag_token(self.attribute_buffer.name.to_string(), self.attribute_buffer.value.to_string());

                                    if add_attribute_result.is_err() {
                                        Tokenizer::parse_error(ParseError::DuplicateAttribute);
                                    }
                                }
                                '&' => {
                                    self.return_state = HTMLTokenizerState::AttributeValueDoubleQuoted;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CharacterReference)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.attribute_buffer.value.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                _ => {
                                    self.attribute_buffer.value.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#attribute-value-(single-quoted)-state
                HTMLTokenizerState::AttributeValueSingleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '\'' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterAttributeValueQuoted);

                                    /* When the user agent leaves the attribute name state (and before emitting the tag token, if appropriate), the complete attribute's name must be compared to the other attributes on the same token;
                                    if there is already an attribute on the token with the exact same name, then this is a duplicate-attribute parse error and the new attribute must be removed from the token. */
                                    let add_attribute_result = self.add_attribute_to_current_tag_token(self.attribute_buffer.name.to_string(), self.attribute_buffer.value.to_string());

                                    if add_attribute_result.is_err() {
                                        Tokenizer::parse_error(ParseError::DuplicateAttribute);
                                    }
                                }
                                '&' => {
                                    self.return_state = HTMLTokenizerState::AttributeValueSingleQuoted;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CharacterReference)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.attribute_buffer.value.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                _ => {
                                    self.attribute_buffer.value.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-attribute-value-(quoted)-state
                HTMLTokenizerState::AttributeValueUnquoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);

                                     /* When the user agent leaves the attribute name state (and before emitting the tag token, if appropriate), the complete attribute's name must be compared to the other attributes on the same token;
                                    if there is already an attribute on the token with the exact same name, then this is a duplicate-attribute parse error and the new attribute must be removed from the token. */
                                    let add_attribute_result = self.add_attribute_to_current_tag_token(self.attribute_buffer.name.to_string(), self.attribute_buffer.value.to_string());

                                    if add_attribute_result.is_err() {
                                        Tokenizer::parse_error(ParseError::DuplicateAttribute);
                                    }
                                },
                                '&' => {
                                    self.return_state = HTMLTokenizerState::AttributeValueUnquoted;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CharacterReference)
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    // Emitted current tag token
                                },
                                '"' | '\'' | '<' | '=' | '`' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedCharacterInUnquotedAttributeValue);
                                }
                                _ => {
                                    self.attribute_buffer.value.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-attribute-value-(quoted)-state
                HTMLTokenizerState::AfterAttributeValueQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                },
                                '/' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::SelfClosingStartTag);
                                }
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    // Emitted current tag token
                                },
                                _ => {
                                    Tokenizer::parse_error(ParseError::WhitespaceMissingBetweenAttributes);
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#self-closing-start-tag-state
                HTMLTokenizerState::SelfClosingStartTag => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '>' => {
                                    self.current_tag_token().self_closing = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    Tokenizer::parse_error(ParseError::UnexpectedSolidusInTag);
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BeforeAttributeName);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInTag);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#bogus-comment-state
                HTMLTokenizerState::BogusComment=> {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().data.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                }
                                _ => {
                                   self.current_tag_token().data.push(character);
                                }
                            }
                        }
                        None => {
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#markup-declaration-open-state
                HTMLTokenizerState::MarkupDeclarationOpen => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // Two U+002D HYPHEN-MINUS characters (-)
                                '-' => {
                                    if self.match_characters(String::from("--")) {
                                        self.html_tokens.push(Tokenizer::create_comment_html_token(String::from("")));
                                        self.switch_to_tokenization_state(HTMLTokenizerState::CommentStart)
                                    }
                                },
                                'D' | 'd' => {
                                    if self.match_characters(String::from("DOCTYPE")) {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::Doctype);
                                    }
                                }
                                '[' => {
                                    if self.match_characters(String::from("[CDATA[")) {
                                        /* Consume those characters. If there is an adjusted current node and it is not an element in the HTML namespace,
                                        then switch to the CDATA section state. Otherwise, this is a cdata-in-html-content parse error.
                                        Create a comment token whose data is the "[CDATA[" string. Switch to the bogus comment state. */

                                        todo!();
                                    }
                                }
                                _ => {
                                    Tokenizer::parse_error(ParseError::IncorrectlyOpenedComment);
                                    self.html_tokens.push(Tokenizer::create_comment_html_token(String::from("")));
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BogusComment);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#comment-start-state
                HTMLTokenizerState::CommentStart => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentStartDash)
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptClosingOfEmptyComment);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#comment-start-dash-state
                HTMLTokenizerState::CommentStartDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentEnd)
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptClosingOfEmptyComment);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                   self.current_tag_token().data.push('-');
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInComment);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#comment
                HTMLTokenizerState::Comment => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '<' => {
                                    self.current_tag_token().data.push(character);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentLessThanSign);
                                },
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentEndDash)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().data.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                }
                                _ => {
                                    self.current_tag_token().data.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInComment);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#comment-less-than-sign-state
                HTMLTokenizerState::CommentLessThanSign => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '!' => {
                                    self.current_tag_token().data.push(character);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentLessThanSignBang);
                                },
                                '<' => {
                                    self.current_tag_token().data.push(character);
                                }
                                _ => {
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment)
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#comment-less-than-sign-bang-state
                HTMLTokenizerState::CommentLessThanSignBang => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentLessThanSignBangDash);
                                },
                                _ => {
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment)
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#comment-less-than-sign-bang-dash-state
                HTMLTokenizerState::CommentLessThanSignBangDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentLessThanSignBangDashDash);
                                },
                                _ => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::CommentEnd)
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#comment-less-than-sign-bang-dash-dash-state
                HTMLTokenizerState::CommentLessThanSignBangDashDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '>' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::CommentEnd)
                                },
                                _ => {
                                   Tokenizer::parse_error(ParseError::NestedComment);
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::CommentEnd);
                                }
                            }
                        }
                        None => {
                            self.reconsume_in_tokenization_state(HTMLTokenizerState::CommentEnd)
                        }
                    }
                }
                // https://html.spec.whatwg.org/#comment-end-dash-state
                HTMLTokenizerState::CommentEndDash => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                  self.switch_to_tokenization_state(HTMLTokenizerState::CommentEnd)
                                },
                                _ => {
                                   self.current_tag_token().data.push('-');
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment)
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInComment);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#comment-end-state
                HTMLTokenizerState::CommentEnd => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                '!' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentEndBang)
                                },
                                '-' => {
                                    self.current_tag_token().data.push('-');
                                }
                                _ => {
                                   self.current_tag_token().data.push_str("--");
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::Comment);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInComment);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#comment-end-bang-state
                HTMLTokenizerState::CommentEndBang => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '-' => {
                                    self.current_tag_token().data.push_str("--!");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CommentEndDash);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::IncorrectlyClosedComment);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                    self.current_tag_token().data.push_str("--!");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Comment);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInComment);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-state
                HTMLTokenizerState::Doctype => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeDoctypeName)
                                },
                                '>' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BeforeDoctypeName)
                                }
                                _ => {

                                    Tokenizer::parse_error(ParseError::MissingWhitespaceBeforeDoctypeName);
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BeforeDoctypeName)
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.html_tokens.push(Tokenizer::create_doctype_html_token(String::from(""), true));
                            self.current_tag_token().force_quirks = true;
                            self.emit_current_html_token();

                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#before-doctype-name-state
                HTMLTokenizerState::BeforeDoctypeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                'A'..='Z' => {
                                    self.html_tokens.push(Tokenizer::create_doctype_html_token(character.to_string().to_ascii_lowercase(), false));
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeName)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.html_tokens.push(Tokenizer::create_doctype_html_token(Tokenizer::REPLACEMENT_FEED_CHARACTER.to_string(), false));
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeName)
                                }
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingDoctypeName);
                                    self.html_tokens.push(Tokenizer::create_doctype_html_token(String::from(""), true));
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                }
                                _ => {
                                    self.html_tokens.push(Tokenizer::create_doctype_html_token(character.to_string(), false));
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeName)
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.html_tokens.push(Tokenizer::create_doctype_html_token(String::from(""), true));
                            self.current_tag_token().force_quirks = true;
                            self.emit_current_html_token();

                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-name-state
                HTMLTokenizerState::DoctypeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypeName);
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                'A'..='Z' => {
                                    self.current_tag_token().name.push(character.to_ascii_lowercase());
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().name.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                _ => {
                                    self.current_tag_token().name.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-doctype-name-state
                HTMLTokenizerState::AfterDoctypeName => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                    if self.match_characters(String::from("PUBLIC")) {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypePublicKeyword)
                                    } else if self.match_characters(String::from("SYSTEM")) {
                                        self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypeSystemKeyword)
                                    } else {
                                        Tokenizer::parse_error(ParseError::InvalidCharacterSequenceAfterDoctypeName);
                                        self.current_tag_token().force_quirks = true;
                                        self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                    }
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-doctype-public-keyword-state
                HTMLTokenizerState::AfterDoctypePublicKeyword => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeDoctypePublicIdentifier);
                                },
                                '"' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceAfterDoctypePublicKeyword);
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypePublicIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceAfterDoctypePublicKeyword);
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypePublicIdentifierSingleQuoted);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#before-doctype-public-identifier-state
                HTMLTokenizerState::BeforeDoctypePublicIdentifier => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore the character
                                },
                                '"' => {
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypePublicIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypePublicIdentifierSingleQuoted);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                }
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        },
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-public-identifier-(double-quoted)-state
                HTMLTokenizerState::DoctypePublicIdentifierDoubleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '"' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypePublicIdentifier)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().public_identifier.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                   self.current_tag_token().public_identifier.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-public-identifier-(single-quoted)-state
                HTMLTokenizerState::DoctypePublicIdentifierSingleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '\'' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypePublicIdentifier)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().public_identifier.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptDoctypePublicIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                   self.current_tag_token().public_identifier.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-doctype-public-identifier-state
                HTMLTokenizerState::AfterDoctypePublicIdentifier => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BetweenDoctypePublicAndSystemIdentifiers);
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                '"' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers);
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers);
                                    self.current_tag_token().public_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierSingleQuoted);
                                }
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#between-doctype-public-and-system-identifiers-state
                HTMLTokenizerState::BetweenDoctypePublicAndSystemIdentifiers => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                   // Ignore these characters
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                '"' => {
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierSingleQuoted);
                                }
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-doctype-system-keyword-state
                HTMLTokenizerState::AfterDoctypeSystemKeyword => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::BeforeDoctypeSystemIdentifier);
                                },
                                '"' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceAfterDoctypeSystemKeyword);
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    Tokenizer::parse_error(ParseError::MissingWhitespaceAfterDoctypeSystemKeyword);
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierSingleQuoted);
                                }
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#between-doctype-public-and-system-identifiers-state
                HTMLTokenizerState::BeforeDoctypeSystemIdentifier => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                '"' => {
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierDoubleQuoted);
                                },
                                '\'' => {
                                    self.current_tag_token().system_identifier = String::from("");
                                    self.switch_to_tokenization_state(HTMLTokenizerState::DoctypeSystemIdentifierSingleQuoted);
                                }
                                '>' => {
                                    Tokenizer::parse_error(ParseError::MissingDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    Tokenizer::parse_error(ParseError::MissingQuoteBeforeDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-system-identifier-(double-quoted)-state
                HTMLTokenizerState::DoctypeSystemIdentifierDoubleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '"' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypeSystemIdentifier)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().system_identifier.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                   self.current_tag_token().system_identifier.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#doctype-system-identifier-(single-quoted)-state
                HTMLTokenizerState::DoctypeSystemIdentifierSingleQuoted => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '\'' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::AfterDoctypeSystemIdentifier)
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    self.current_tag_token().system_identifier.push(Tokenizer::REPLACEMENT_FEED_CHARACTER);
                                },
                                '>' => {
                                    Tokenizer::parse_error(ParseError::AbruptDoctypeSystemIdentifier);
                                    self.current_tag_token().force_quirks = true;
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                   self.current_tag_token().system_identifier.push(character);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#after-doctype-system-identifier-state
                HTMLTokenizerState::AfterDoctypeSystemIdentifier => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // \u{000C} is a form feed character
                                '\t' | '\n' | '\r' | '\u{000C}' | ' '  => {
                                    // Ignore these characters
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                _ => {
                                    Tokenizer::parse_error(ParseError::UnexpectedCharacterAfterDoctypeSystemIdentifier);
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::BogusDoctype);
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOFileInDoctype);
                            self.current_tag_token().force_quirks = true;
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#bogus-doctype-state
                HTMLTokenizerState::BogusDoctype => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                '>' => {
                                  self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                    self.emit_current_html_token();
                                },
                                '\0' => {
                                    Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                                    // Ignore this character
                                },
                                _ => {
                                   // Ignore this character
                                }
                            }
                        }
                        None => {
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#cdata-section-state
                HTMLTokenizerState::CdataSection => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                ']' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CdataSectionBracket);
                                },
                                _ => {
                                   self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                    self.emit_current_html_token();
                                }
                            }
                        }
                        None => {
                            Tokenizer::parse_error(ParseError::EndOfFileInCData);
                            self.html_tokens.push(Tokenizer::create_end_of_file_html_token());
                            self.emit_current_html_token();
                        }
                    }
                }
                // https://html.spec.whatwg.org/#cdata-section-bracket-state
                HTMLTokenizerState::CdataSectionBracket => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                ']' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::CdataSectionEnd);
                                },
                                _ => {
                                   self.html_tokens.push(Tokenizer::create_character_html_token(']'));
                                    self.emit_current_html_token();
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::CdataSection);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#cdata-section-end-state
                HTMLTokenizerState::CdataSectionEnd => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                ']' => {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(']'));
                                    self.emit_current_html_token();
                                },
                                '>' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::Data);
                                }
                                _ => {
                                   self.html_tokens.push(Tokenizer::create_character_html_token(']'));
                                    self.emit_current_html_token();

                                   self.html_tokens.push(Tokenizer::create_character_html_token(']'));
                                    self.emit_current_html_token();

                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::CdataSection);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#character-reference-state
                HTMLTokenizerState::CharacterReference => {
                    self.temporary_buffer = String::from("");
                    self.temporary_buffer.push('&');

                    match current_input_character {
                        Some(character) => {
                            match character {
                                // https://infra.spec.whatwg.org/#ascii-alphanumeric
                                'A'..='Z' | 'a'..='z' | '0'..='9'  => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::NamedCharacterReference);
                                },
                                '#' => {
                                    self.temporary_buffer.push(character);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::NumericCharacterReference);
                                }
                                _ => {
                                    match self.return_state {
                                        HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => {
                                            // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                                self.attribute_buffer.value.push(character_in_temporary_buffer);
                                            }
                                        },
                                        _ => {
                                            // TODO: Use emit_html_tokens instead of directly pushing?
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                                self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                                            }
                                        }
                                    }
                                    self.reconsume_in_tokenization_state(self.return_state);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#named-character-reference-state
                HTMLTokenizerState::NamedCharacterReference => {
                    let mut current_character_reference = String::from("");
                    let mut character = current_input_character;
                    let mut match_found = false;

                    while match_found != true && character.is_some() {
                        current_character_reference.push(character.unwrap());
                        self.temporary_buffer.push(character.unwrap());

                        let mut find_string = String::from("&");
                        find_string.push_str(&current_character_reference);

                        let has_any = &self.named_character_references.iter().filter(|obj| obj.character_reference.starts_with(&find_string)).count() > &0;

                        if has_any {
                            if self.found_in_named_character_reference_table(current_character_reference.to_string()) {
                                match_found = true;
                                /* Because some character references don't have ';' at the end of them, we match them before we can read and consume the ';'
                                   So if the last one was a ';' character then we should consume it and set the current character to it */
                                if self.lexer.peek().unwrap() == ';' {
                                    self.lexer.advance();
                                    character = self.lexer.peek();
                                }
                            } else {
                                character = self.lexer.peek();
                                self.lexer.advance();
                            }
                        } else {
                            break;
                        }
                    }

                    if match_found {
                           if matches!(self.return_state,
                             HTMLTokenizerState::AttributeValueDoubleQuoted |
                             HTMLTokenizerState::AttributeValueSingleQuoted |
                             HTMLTokenizerState::AttributeValueUnquoted) && self.lexer.rewindAndPeek(1).unwrap() != ';' &&
                             (character.unwrap() == '=' || character.unwrap().is_ascii_alphanumeric()) {
                                // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                                for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                    self.attribute_buffer.value.push(character_in_temporary_buffer);
                                }

                                self.switch_to_tokenization_state(self.return_state);
                            } else {


                                /* FIXME: This will incorrectly report the MissingSemicolonAfterCharacterReference
                                   as the parser has consumed a named character reference that does not have a semicolon in the reference table
                                   then this will treat it as an error. The semicolon is consumed so does not affect parser correctness
                                   but currently is not taken into account when checking for this error  */
                                if character.unwrap() != ';' {
                                    Tokenizer::parse_error(ParseError::MissingSemicolonAfterCharacterReference);
                                }

                                self.temporary_buffer = String::from("");
                                match self.get_characters_by_character_reference(current_character_reference.to_string()) {
                                    Some (characters) => {
                                        self.append_to_temporary_buffer(characters.to_string());
                                    },
                                    None => ()
                                }

                                // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                                match self.return_state {
                                    HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => {
                                        for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                            self.attribute_buffer.value.push(character_in_temporary_buffer);
                                        }

                                    },
                                    _ => {
                                        for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                            self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                                        }
                                    }
                                }

                                self.switch_to_tokenization_state(self.return_state);
                            }
                    } else {

                        // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                        match self.return_state {
                            HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => {
                                for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                    self.attribute_buffer.value.push(character_in_temporary_buffer);
                                }
                            },
                            _ => {
                                for character_in_temporary_buffer in self.temporary_buffer.chars() {
                                    self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                                }
                            }
                        }

                        self.switch_to_tokenization_state(HTMLTokenizerState::AmbiguousAmpersand);
                    }



                }
                // https://html.spec.whatwg.org/#ambiguous-ampersand-state
                HTMLTokenizerState::AmbiguousAmpersand => {
                    match current_input_character {
                        Some(character) => {
                            match character {
                                // https://infra.spec.whatwg.org/#ascii-alphanumeric
                                'A'..='Z' | 'a'..='z' | '0'..='9' => {
                                    match self.return_state {
                                        HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => {
                                            self.attribute_buffer.value.push(character);
                                        },
                                        _ => {
                                            self.html_tokens.push(Tokenizer::create_character_html_token(character));
                                            self.emit_current_html_token();
                                        }
                                    }
                                },
                                ';' => {
                                    Tokenizer::parse_error(ParseError::UnknownNamedCharacterReference);
                                    self.reconsume_in_tokenization_state(self.return_state);
                                }
                                _ => { 
                                    self.reconsume_in_tokenization_state(self.return_state);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#numeric-character-reference-state
                HTMLTokenizerState::NumericCharacterReference => {
                    self.character_reference_code = 0;

                    match current_input_character { 
                        Some(character) => { 
                            match character { 
                                // U+0078 LATIN SMALL LETTER X | U+0058 LATIN CAPITAL LETTER X
                                '\u{0078}' | '\u{0058}' => {
                                    self.temporary_buffer.push(character);
                                    self.switch_to_tokenization_state(HTMLTokenizerState::HexadecimalCharacterReferenceStart);
                                },
                                _ => { 
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::DecimalCharacterReferenceStart)
                                }
                            }
                        }
                        None => ()
                    }
                    
                }
                // https://html.spec.whatwg.org/#hexadecimal-character-reference-start-state
                HTMLTokenizerState::HexadecimalCharacterReferenceStart => {
                    match current_input_character { 
                        Some(character) => { 
                            match character { 
                                // https://infra.spec.whatwg.org/#ascii-hex-digit
                                '0'..='9' | 'A'..='F' | 'a'..='f' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::HexadecimalCharacterReference);
                                },
                                _ => { 
                                   Tokenizer::parse_error(ParseError::AbsenceOfDigitsInNumericCharacterReference);

                                     // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                                     match self.return_state { 
                                        HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => { 
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                                self.attribute_buffer.value.push(character_in_temporary_buffer);
                                            }
                                        },
                                        _ => {
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                                self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                                            }
                                        }
                                     }

                                     self.reconsume_in_tokenization_state(self.return_state);
                                }
                            }
                        }
                        None => ()
                    }
                    
                }
                // https://html.spec.whatwg.org/#decimal-character-reference-start-state
                HTMLTokenizerState::DecimalCharacterReferenceStart => {
                    match current_input_character { 
                        Some(character) => { 
                            match character { 
                                // https://infra.spec.whatwg.org/#ascii-hex-digit
                                '0'..='9' | 'A'..='F' | 'a'..='f' => {
                                    self.reconsume_in_tokenization_state(HTMLTokenizerState::DecimalCharacterReference);
                                },
                                _ => { 
                                   Tokenizer::parse_error(ParseError::AbsenceOfDigitsInNumericCharacterReference);

                                     // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                                     match self.return_state { 
                                        HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => { 
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                                self.attribute_buffer.value.push(character_in_temporary_buffer);
                                            }
                                        },
                                        _ => {
                                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                                self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                                            }
                                        }
                                     }

                                     self.reconsume_in_tokenization_state(self.return_state);
                                }
                            }
                        }
                        None => ()
                    }
                }
                // https://html.spec.whatwg.org/#hexadecimal-character-reference-state
                HTMLTokenizerState::HexadecimalCharacterReference => {
                    match current_input_character { 
                        Some(character) => { 
                            match character { 
                                // https://infra.spec.whatwg.org/#ascii-digit
                                '0'..='9' => {
                                    self.character_reference_code = self.character_reference_code * 16;
                                    
                                    self.character_reference_code = self.character_reference_code + character as u32 - 0x0030;
                                },
                                'A'..='F' => {
                                    self.character_reference_code = self.character_reference_code * 16;
                                    
                                    self.character_reference_code = self.character_reference_code + character as u32 - 0x0037;
                                },
                                // https://infra.spec.whatwg.org/#ascii-lower-hex-digit
                                'a'..='f' => {
                                    self.character_reference_code = self.character_reference_code * 16;
                                    
                                    self.character_reference_code = self.character_reference_code + character as u32 - 0x0057;
                                },
                                ';' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::NumerticCharacterReferenceEnd);
                                }
                                _ => { 
                                   Tokenizer::parse_error(ParseError::MissingSemicolonAfterCharacterReference);
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::NumerticCharacterReferenceEnd);
                                }
                            }
                        }
                        None => ()
                    }
                    
                }
                HTMLTokenizerState::DecimalCharacterReference => {
                    match current_input_character { 
                        Some(character) => { 
                            match character { 
                                // https://infra.spec.whatwg.org/#ascii-digit
                                '0'..='9' => {
                                    self.character_reference_code = self.character_reference_code * 10;
                                    
                                    self.character_reference_code = self.character_reference_code + character as u32 - 0x0030;
                                },
                                ';' => {
                                    self.switch_to_tokenization_state(HTMLTokenizerState::NumerticCharacterReferenceEnd);
                                }
                                _ => { 
                                   Tokenizer::parse_error(ParseError::MissingSemicolonAfterCharacterReference);
                                   self.reconsume_in_tokenization_state(HTMLTokenizerState::NumerticCharacterReferenceEnd);
                                }
                            }
                        }
                        None => ()
                    }
                    
                }
                HTMLTokenizerState::NumerticCharacterReferenceEnd => {
                    if self.character_reference_code == 0x00 {
                        Tokenizer::parse_error(ParseError::UnexpectedNullCharacter);
                        self.character_reference_code = 0xFFFD;
                    } else if self.character_reference_code > 0x10FFFF { 
                        Tokenizer::parse_error(ParseError::CharacterReferenceOutsideUnicodeRange);
                        self.character_reference_code = 0xFFFD;
                    // Surrogate character check
                    } else if self.character_reference_code >= 0x0000D800 ||  self.character_reference_code <= 0x0000DFFF {
                        Tokenizer::parse_error(ParseError::SurrogateCharacterReference);
                        self.character_reference_code = 0xFFFD;
                    } else if Tokenizer::is_non_character(self.character_reference_code as u8) { 
                        Tokenizer::parse_error(ParseError::NonCharacterReference);
                    } else if self.character_reference_code == 0x0D || (Tokenizer::is_control(self.character_reference_code as u8) && !Tokenizer::is_ascii_whitespace(self.character_reference_code as u8)) {
                        Tokenizer::parse_error(ParseError::ControlCharacterReference);
                    } else {
                        if self.number_character_references.contains_key(&self.character_reference_code) {
                            self.character_reference_code = self.number_character_references[&self.character_reference_code];
                        }
                    }

                    self.temporary_buffer = String::from("");
                    self.temporary_buffer.push(char::from_u32(self.character_reference_code).unwrap());

                    // https://html.spec.whatwg.org/#flush-code-points-consumed-as-a-character-reference
                    match self.return_state { 
                        HTMLTokenizerState::AttributeValueDoubleQuoted | HTMLTokenizerState::AttributeValueSingleQuoted | HTMLTokenizerState::AttributeValueUnquoted => { 
                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                self.attribute_buffer.value.push(character_in_temporary_buffer);
                            }
                        },
                        _ => {
                            for character_in_temporary_buffer in self.temporary_buffer.chars() { 
                                self.html_tokens.push(Tokenizer::create_character_html_token(character_in_temporary_buffer))
                            }
                        }
                    }

                    self.switch_to_tokenization_state(self.return_state);
                    
                }
            }

        
    }
    
    // https://html.spec.whatwg.org/#next-input-character
    fn next_input_character(&mut self) -> Option<char> { 
        let result = self.lexer.peek();
        self.lexer.advance();   
    
        match result { 
            Some(c) => return Some(c),
            None => return None
        }
    }
    
    // https://html.spec.whatwg.org/#current-input-character
    fn current_input_character(&mut self) -> Option<char> { 
        let result = self.lexer.previous();
        
        match result { 
            Some(c) => return Some(c),
            None => return None
        }
    }

    // TODO: We need to handle if the lexer hits the end of file while trying to get previous
    // This handling is required for End of file parse errors
    fn match_characters(&mut self, word: String) -> bool { 
        let mut all_characters_match = false;
        let mut characters_matched_amount = 0;
        
        for character in word.chars() { 
            let peeked_character = self.lexer.previous().unwrap();

            if peeked_character.to_ascii_lowercase() == character.to_ascii_lowercase() { 
                self.lexer.advance();

                characters_matched_amount += 1;
                all_characters_match = true;
                
            } else 
            {
                self.lexer.rewind(characters_matched_amount);
                all_characters_match = false;
            }
        }

        /* We always want to rewind the lexer position back by one to ensure the tokenizer's current input character is the last  
        character of the word being matched so that we don't ignore any characters directly after the word being matched (e.g. a whitespace) */ 
        self.lexer.rewind(1);

        return all_characters_match;
    }

    fn found_in_named_character_reference_table(&self, characters: String) -> bool { 
        let mut matched_string: String = String::from("&");
        matched_string.push_str(&characters);

        return self.named_character_references.iter()
        .filter(|obj| obj.character_reference == matched_string).count() > 0;
    }

    fn append_to_temporary_buffer(&mut self, chars: String) { 
        self.temporary_buffer.push_str(&chars);
    }

    fn get_characters_by_character_reference(&self, character_reference: String) -> Option<String> { 
        let mut matched_string = String::from("&");
        matched_string.push_str(&character_reference);
        for obj in self.named_character_references.iter() { 
            if obj.character_reference == matched_string { 
                
                return Some(obj.characters.to_string());
            } 
        }

        return None;
    }

    // https://html.spec.whatwg.org/#reconsume
    fn reconsume_current_input_character(&mut self) { 
        self.reconsume_current_input_character = true;
    }

    // https://html.spec.whatwg.org/#appropriate-end-tag-token
    fn appropriate_end_tag_token(&mut self) -> bool { 
        let mut index = self.html_tokens.len() - 1;

        let current_end_tag_token = &self.html_tokens[index];

        // Traverse from the end of the tokens list back to the start to find a matching start tag
        while index != 0 {
            match self.html_tokens[index].token_type { 
                HtmlTokenType::StartTag => { 
                    if self.html_tokens[index].tag_name == current_end_tag_token.tag_name { 
                        return true;
                    }
                }
                _ => return false
            }
            index -= 1;
        }

        return false;
    }
    
    fn switch_to_tokenization_state(&mut self, new_tokenization_state: HTMLTokenizerState) { 
        self.tokenization_state = new_tokenization_state;
    }

    fn reconsume_in_tokenization_state(&mut self, new_tokenization_state: HTMLTokenizerState) { 
        self.tokenization_state = new_tokenization_state;
        self.reconsume_current_input_character();
    }

    fn emit_current_html_token(&mut self) {
        let last_html_token_index = self.html_tokens.len();
        let current_tag_token = &self.html_tokens[last_html_token_index - 1];

        self.html_document_parser.parse_html_token(current_tag_token);
    }

    fn current_tag_token(&mut self) -> &mut HtmlToken {
        let last_html_token_index = self.html_tokens.len();
        return &mut self.html_tokens[last_html_token_index - 1];
    }

    fn add_attribute_to_current_tag_token(&mut self, name: String, value: String) -> Result<(), ()> {
        if self.current_tag_token().attributes.contains_key(&name) { 
            return Err(());
        } else 
        {
            self.current_tag_token().attributes.insert(name.to_string(), value.to_string());
            return Ok(());
        }
    }

    // https://infra.spec.whatwg.org/#noncharacter
    fn is_non_character(value: u8) -> bool {
        if value >= '\u{FDD0}' as u8 || value  <= '\u{FDEF}' as u8 {
            return true;
        }

        match value as char {
            '\u{FFFE}' |
            '\u{FFFF}' |
            '\u{1FFFE}' | 
            '\u{1FFFF}' | 
            '\u{2FFFE}' | 
            '\u{2FFFF}' | 
            '\u{3FFFE}' | 
            '\u{3FFFF}' | 
            '\u{4FFFE}' | 
            '\u{4FFFF}' | 
            '\u{5FFFE}' | 
            '\u{5FFFF}' | 
            '\u{6FFFE}' | 
            '\u{6FFFF}' | 
            '\u{7FFFE}' | 
            '\u{7FFFF}' | 
            '\u{8FFFE}' | 
            '\u{8FFFF}' | 
            '\u{9FFFE}' | 
            '\u{9FFFF}' | 
            '\u{AFFFE}' | 
            '\u{AFFFF}' | 
            '\u{BFFFE}' | 
            '\u{BFFFF}' | 
            '\u{CFFFE}' | 
            '\u{CFFFF}' | 
            '\u{DFFFE}' | 
            '\u{DFFFF}' | 
            '\u{EFFFE}' | 
            '\u{EFFFF}' | 
            '\u{FFFFE}' | 
            '\u{FFFFF}' | 
            '\u{10FFFE}' |
            '\u{10FFFF}' => {
                return true;
            },
            _ => ()
        }

        return false;
    }

    // https://infra.spec.whatwg.org/#control
    fn is_control(value: u8) -> bool {
        // https://infra.spec.whatwg.org/#c0-control
        if value as char >= '\u{0000}' || value as char <= '\u{001F}' {
            return true;
        }

        if value as char >= '\u{007F}' || value as char <= '\u{009F}' {
            return true;
        }

        return false;
    }

    // https://infra.spec.whatwg.org/#ascii-whitespace
    fn is_ascii_whitespace(value: u8) -> bool {
        match value as char {
            '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' | '\u{0020}' => {
                return true;
            },
            _ => { return false; }
        }
    }

    fn create_doctype_html_token(name: String, force_quirks: bool) -> HtmlToken { 
        let doctype_html_token = HtmlToken { 
            token_type: HtmlTokenType::DocType,
            name: name,
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: force_quirks,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: String::from("")
        };

        return doctype_html_token;
    }

    fn create_comment_html_token(character_data: String) -> HtmlToken { 
        let comment_html_token = HtmlToken { 
            token_type: HtmlTokenType::Comment,
            name: String::from(""),
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: false,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: character_data
        };

        return comment_html_token;
    }
    

    fn create_start_tag_html_token() -> HtmlToken { 
        let start_tag_html_token = HtmlToken { 
            token_type: HtmlTokenType::StartTag,
            name: String::from(""),
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: false,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: String::from("")
        };

        return start_tag_html_token;
    }

    fn create_end_tag_html_token() -> HtmlToken { 
        let end_tag_html_token = HtmlToken { 
            token_type: HtmlTokenType::EndTag,
            name: String::from(""),
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: false,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: String::from("")
        };

        return end_tag_html_token;
    }

    fn create_character_html_token(character_data: char) -> HtmlToken { 
        let character_html_token = HtmlToken { 
            token_type: HtmlTokenType::Character,
            name: String::from(""),
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: false,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: character_data.to_string()
        };

        return character_html_token;
    }

    fn create_end_of_file_html_token() -> HtmlToken { 
        let end_of_file_html_token = HtmlToken { 
            token_type: HtmlTokenType::EndOfFile,
            name: String::from(""),
            public_identifier: String::from(""),
            system_identifier: String::from(""),
            force_quirks: false,
            tag_name: String::from(""),
            self_closing: false,
            attributes: HashMap::new(),
            data: String::from("")
        };

        return end_of_file_html_token;
    }

    // https://html.spec.whatwg.org/#parse-errors
    fn parse_error(parse_error: ParseError) { 
        println!("[HTML::Tokenizer] Parse error found '{}'", parse_error.to_string());
    }


}