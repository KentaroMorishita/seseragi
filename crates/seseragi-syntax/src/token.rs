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
    KeywordPub,
    KeywordLet,
    IdentifierLower,
    IdentifierUpper,
    LiteralInteger,
    OperatorEquals,
    PunctuationColon,
    TriviaSpace,
    TriviaNewline,
    Unknown,
    Eof,
}

impl TokenKind {
    pub fn artifact_kind(self) -> &'static str {
        match self {
            Self::KeywordPub => "keyword.pub",
            Self::KeywordLet => "keyword.let",
            Self::IdentifierLower => "identifier.lower",
            Self::IdentifierUpper => "identifier.upper",
            Self::LiteralInteger => "literal.integer",
            Self::OperatorEquals => "operator.equals",
            Self::PunctuationColon => "punctuation.colon",
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
