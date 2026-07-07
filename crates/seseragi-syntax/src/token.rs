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
    KeywordFn,
    KeywordPub,
    KeywordLet,
    IdentifierLower,
    IdentifierUpper,
    LiteralInteger,
    OperatorArrow,
    OperatorArithmetic,
    OperatorEquals,
    OperatorLambda,
    OperatorPipeline,
    OperatorCustom,
    PunctuationColon,
    PunctuationParenLeft,
    PunctuationParenRight,
    TriviaComment,
    TriviaSpace,
    TriviaNewline,
    Unknown,
    Eof,
}

impl TokenKind {
    pub fn artifact_kind(self) -> &'static str {
        match self {
            Self::KeywordFn => "keyword.fn",
            Self::KeywordPub => "keyword.pub",
            Self::KeywordLet => "keyword.let",
            Self::IdentifierLower => "identifier.lower",
            Self::IdentifierUpper => "identifier.upper",
            Self::LiteralInteger => "literal.integer",
            Self::OperatorArrow => "operator.arrow",
            Self::OperatorArithmetic => "operator.arithmetic",
            Self::OperatorEquals => "operator.equals",
            Self::OperatorLambda => "operator.lambda",
            Self::OperatorPipeline => "operator.pipeline",
            Self::OperatorCustom => "operator.custom",
            Self::PunctuationColon => "punctuation.colon",
            Self::PunctuationParenLeft => "punctuation.paren.left",
            Self::PunctuationParenRight => "punctuation.paren.right",
            Self::TriviaComment => "trivia.comment",
            Self::TriviaSpace => "trivia.space",
            Self::TriviaNewline => "trivia.newline",
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
