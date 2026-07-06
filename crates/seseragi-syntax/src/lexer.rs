use crate::token::{Token, TokenKind, TokenStream};

pub fn lex(source_name: impl Into<String>, source: &str) -> TokenStream {
    let mut lexer = Lexer {
        source,
        cursor: 0,
        tokens: Vec::new(),
    };
    lexer.scan_all();
    TokenStream::new(source_name, lexer.tokens)
}

struct Lexer<'source> {
    source: &'source str,
    cursor: usize,
    tokens: Vec<Token>,
}

impl Lexer<'_> {
    fn scan_all(&mut self) {
        while self.cursor < self.source.len() {
            let start = self.cursor;
            let Some(char) = self.peek_char() else {
                break;
            };
            match char {
                ' ' => self.scan_run(TokenKind::TriviaSpace, |char| char == ' '),
                '\n' => self.scan_newline(),
                ':' => self.bump_fixed(TokenKind::PunctuationColon, start, char),
                '=' => self.bump_fixed(TokenKind::OperatorEquals, start, char),
                '0'..='9' => self.scan_run(TokenKind::LiteralInteger, |char| char.is_ascii_digit()),
                '_' | 'a'..='z' | 'A'..='Z' => self.scan_identifier(),
                _ => self.bump_fixed(TokenKind::Unknown, start, char),
            }
        }
        self.tokens
            .push(Token::new(TokenKind::Eof, self.cursor, self.cursor, ""));
    }

    fn scan_newline(&mut self) {
        let start = self.cursor;
        self.cursor += 1;
        self.push(TokenKind::TriviaNewline, start, self.cursor);
    }

    fn scan_identifier(&mut self) {
        let start = self.cursor;
        self.take_char();
        while matches!(
            self.peek_char(),
            Some('_' | 'a'..='z' | 'A'..='Z' | '0'..='9')
        ) {
            self.take_char();
        }
        let raw = &self.source[start..self.cursor];
        let kind = match raw {
            "pub" => TokenKind::KeywordPub,
            "let" => TokenKind::KeywordLet,
            _ if raw
                .chars()
                .next()
                .is_some_and(|char| char.is_ascii_uppercase()) =>
            {
                TokenKind::IdentifierUpper
            }
            _ => TokenKind::IdentifierLower,
        };
        self.push(kind, start, self.cursor);
    }

    fn scan_run(&mut self, kind: TokenKind, predicate: impl Fn(char) -> bool) {
        let start = self.cursor;
        while let Some(char) = self.peek_char() {
            if !predicate(char) {
                break;
            }
            self.take_char();
        }
        self.push(kind, start, self.cursor);
    }

    fn bump_fixed(&mut self, kind: TokenKind, start: usize, char: char) {
        self.cursor += char.len_utf8();
        self.push(kind, start, self.cursor);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens
            .push(Token::new(kind, start, end, &self.source[start..end]));
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn take_char(&mut self) -> Option<char> {
        let char = self.peek_char()?;
        self.cursor += char.len_utf8();
        Some(char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn lexes_basic_answer_fixture() {
        let stream = lex("main.ssrg", "pub let answer: Int = 42\n");
        let actual = stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.start, token.end, token.raw.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                (TokenKind::KeywordPub, 0, 3, "pub"),
                (TokenKind::TriviaSpace, 3, 4, " "),
                (TokenKind::KeywordLet, 4, 7, "let"),
                (TokenKind::TriviaSpace, 7, 8, " "),
                (TokenKind::IdentifierLower, 8, 14, "answer"),
                (TokenKind::PunctuationColon, 14, 15, ":"),
                (TokenKind::TriviaSpace, 15, 16, " "),
                (TokenKind::IdentifierUpper, 16, 19, "Int"),
                (TokenKind::TriviaSpace, 19, 20, " "),
                (TokenKind::OperatorEquals, 20, 21, "="),
                (TokenKind::TriviaSpace, 21, 22, " "),
                (TokenKind::LiteralInteger, 22, 24, "42"),
                (TokenKind::TriviaNewline, 24, 25, "\n"),
                (TokenKind::Eof, 25, 25, ""),
            ]
        );
        assert_eq!(stream.reconstructed_text(), "pub let answer: Int = 42\n");
    }

    #[test]
    fn lexes_recovery_answer_fixture_without_missing_expression() {
        let stream = lex("main.ssrg", "pub let answer: Int =\n");
        let actual = stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.start, token.end, token.raw.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                (TokenKind::KeywordPub, 0, 3, "pub"),
                (TokenKind::TriviaSpace, 3, 4, " "),
                (TokenKind::KeywordLet, 4, 7, "let"),
                (TokenKind::TriviaSpace, 7, 8, " "),
                (TokenKind::IdentifierLower, 8, 14, "answer"),
                (TokenKind::PunctuationColon, 14, 15, ":"),
                (TokenKind::TriviaSpace, 15, 16, " "),
                (TokenKind::IdentifierUpper, 16, 19, "Int"),
                (TokenKind::TriviaSpace, 19, 20, " "),
                (TokenKind::OperatorEquals, 20, 21, "="),
                (TokenKind::TriviaNewline, 21, 22, "\n"),
                (TokenKind::Eof, 22, 22, ""),
            ]
        );
        assert_eq!(stream.reconstructed_text(), "pub let answer: Int =\n");
    }
}
