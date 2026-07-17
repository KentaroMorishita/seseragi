use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
    pub raw: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    KeywordDo,
    KeywordEffect,
    KeywordElse,
    KeywordFails,
    KeywordFn,
    KeywordIf,
    KeywordPub,
    KeywordLet,
    KeywordMatch,
    KeywordThen,
    KeywordWhen,
    KeywordWith,
    IdentifierLower,
    IdentifierUpper,
    LiteralBoolean,
    LiteralInteger,
    LiteralString,
    LiteralTemplate,
    OperatorArrow,
    OperatorArithmetic,
    OperatorApply,
    OperatorBind,
    OperatorComparison,
    OperatorEquals,
    OperatorLambda,
    OperatorPipeline,
    OperatorRangeExclusive,
    OperatorRangeInclusive,
    OperatorCustom,
    PunctuationBraceLeft,
    PunctuationBraceRight,
    PunctuationColon,
    PunctuationComma,
    PunctuationDot,
    PunctuationEllipsis,
    PunctuationParenLeft,
    PunctuationParenRight,
    PunctuationSemicolon,
    PunctuationListLeft,
    PunctuationSquareLeft,
    PunctuationSquareRight,
    TriviaComment,
    TriviaSpace,
    TriviaNewline,
    Wildcard,
    Unknown,
    Eof,
}

impl TokenKind {
    pub fn artifact_kind(self) -> &'static str {
        match self {
            Self::KeywordDo => "keyword.do",
            Self::KeywordEffect => "keyword.effect",
            Self::KeywordElse => "keyword.else",
            Self::KeywordFails => "keyword.fails",
            Self::KeywordFn => "keyword.fn",
            Self::KeywordIf => "keyword.if",
            Self::KeywordPub => "keyword.pub",
            Self::KeywordLet => "keyword.let",
            Self::KeywordMatch => "keyword.match",
            Self::KeywordThen => "keyword.then",
            Self::KeywordWhen => "keyword.when",
            Self::KeywordWith => "keyword.with",
            Self::IdentifierLower => "identifier.lower",
            Self::IdentifierUpper => "identifier.upper",
            Self::LiteralBoolean => "literal.boolean",
            Self::LiteralInteger => "literal.integer",
            Self::LiteralString => "literal.string",
            Self::LiteralTemplate => "literal.template",
            Self::OperatorArrow => "operator.arrow",
            Self::OperatorArithmetic => "operator.arithmetic",
            Self::OperatorApply => "operator.apply",
            Self::OperatorBind => "operator.bind",
            Self::OperatorComparison => "operator.comparison",
            Self::OperatorEquals => "operator.equals",
            Self::OperatorLambda => "operator.lambda",
            Self::OperatorPipeline => "operator.pipeline",
            Self::OperatorRangeExclusive => "operator.range.exclusive",
            Self::OperatorRangeInclusive => "operator.range.inclusive",
            Self::OperatorCustom => "operator.custom",
            Self::PunctuationBraceLeft => "punctuation.brace.left",
            Self::PunctuationBraceRight => "punctuation.brace.right",
            Self::PunctuationColon => "punctuation.colon",
            Self::PunctuationComma => "punctuation.comma",
            Self::PunctuationDot => "punctuation.dot",
            Self::PunctuationEllipsis => "punctuation.ellipsis",
            Self::PunctuationParenLeft => "punctuation.paren.left",
            Self::PunctuationParenRight => "punctuation.paren.right",
            Self::PunctuationSemicolon => "punctuation.semicolon",
            Self::PunctuationListLeft => "punctuation.list.left",
            Self::PunctuationSquareLeft => "punctuation.square.left",
            Self::PunctuationSquareRight => "punctuation.square.right",
            Self::TriviaComment => "trivia.comment",
            Self::TriviaSpace => "trivia.space",
            Self::TriviaNewline => "trivia.newline",
            Self::Wildcard => "wildcard",
            Self::Unknown => "unknown",
            Self::Eof => "eof",
        }
    }
}

impl Serialize for TokenKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.artifact_kind())
    }
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize, raw: &str) -> Self {
        Self {
            kind,
            start,
            end,
            raw: raw.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TokenStream {
    pub schema: u32,
    pub source: String,
    #[serde(rename = "positionEncoding")]
    pub position_encoding: String,
    pub tokens: Vec<Token>,
}

impl TokenStream {
    pub fn new(source: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            schema: 1,
            source: source.into(),
            position_encoding: "utf-8".to_owned(),
            tokens,
        }
    }

    pub fn reconstructed_text(&self) -> String {
        self.tokens
            .iter()
            .filter(|token| token.kind != TokenKind::Eof)
            .map(|token| token.raw.as_str())
            .collect()
    }
}
