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
                '(' => self.bump_fixed(TokenKind::PunctuationParenLeft, start, char),
                ')' => self.bump_fixed(TokenKind::PunctuationParenRight, start, char),
                '{' => self.bump_fixed(TokenKind::PunctuationBraceLeft, start, char),
                '}' => self.bump_fixed(TokenKind::PunctuationBraceRight, start, char),
                '[' => self.bump_fixed(TokenKind::PunctuationSquareLeft, start, char),
                ']' => self.bump_fixed(TokenKind::PunctuationSquareRight, start, char),
                ',' => self.bump_fixed(TokenKind::PunctuationComma, start, char),
                '.' if self.starts_with("..=") => {
                    self.bump_fixed_raw(TokenKind::OperatorRangeInclusive, start, "..=")
                }
                '.' if self.starts_with("..") => {
                    self.bump_fixed_raw(TokenKind::OperatorRangeExclusive, start, "..")
                }
                '.' => self.bump_fixed(TokenKind::PunctuationDot, start, char),
                '<' if self.starts_with("<-")
                    || self.starts_with("<=")
                    || self.starts_with("<<") =>
                {
                    self.scan_operator_run()
                }
                '<' => self.bump_fixed(TokenKind::OperatorComparison, start, char),
                '>' if self.starts_with(">=") || self.starts_with(">>=") => {
                    self.scan_operator_run()
                }
                '>' => self.bump_fixed(TokenKind::OperatorComparison, start, char),
                '"' => self.scan_quoted(TokenKind::LiteralString, '"'),
                '`' => self.scan_quoted(TokenKind::LiteralTemplate, '`'),
                '\\' => self.bump_fixed(TokenKind::OperatorLambda, start, char),
                '0'..='9' => self.scan_run(TokenKind::LiteralInteger, |char| char.is_ascii_digit()),
                '_' | 'a'..='z' | 'A'..='Z' => self.scan_identifier(),
                '/' if self.starts_with("//") => self.scan_line_comment(),
                char if is_operator_char(char) => self.scan_operator_run(),
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
            "do" => TokenKind::KeywordDo,
            "effect" => TokenKind::KeywordEffect,
            "else" => TokenKind::KeywordElse,
            "fails" => TokenKind::KeywordFails,
            "fn" => TokenKind::KeywordFn,
            "if" => TokenKind::KeywordIf,
            "pub" => TokenKind::KeywordPub,
            "let" => TokenKind::KeywordLet,
            "then" => TokenKind::KeywordThen,
            "with" => TokenKind::KeywordWith,
            "True" | "False" => TokenKind::LiteralBoolean,
            "_" => TokenKind::Wildcard,
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

    fn scan_line_comment(&mut self) {
        let start = self.cursor;
        while let Some(char) = self.peek_char() {
            if char == '\n' {
                break;
            }
            self.take_char();
        }
        self.push(TokenKind::TriviaComment, start, self.cursor);
    }

    fn scan_quoted(&mut self, kind: TokenKind, delimiter: char) {
        let start = self.cursor;
        self.take_char();
        let mut escaped = false;
        while let Some(char) = self.peek_char() {
            self.take_char();
            if escaped {
                escaped = false;
                continue;
            }
            if char == '\\' {
                escaped = true;
                continue;
            }
            if char == delimiter {
                break;
            }
            if char == '\n' {
                break;
            }
        }
        self.push(kind, start, self.cursor);
    }

    fn scan_operator_run(&mut self) {
        let start = self.cursor;
        while self.peek_char().is_some_and(is_operator_char) {
            self.take_char();
        }
        let raw = &self.source[start..self.cursor];
        let kind = match raw {
            ":" => TokenKind::PunctuationColon,
            "=" => TokenKind::OperatorEquals,
            "->" => TokenKind::OperatorArrow,
            "<-" => TokenKind::OperatorBind,
            "|>" => TokenKind::OperatorPipeline,
            "$" => TokenKind::OperatorApply,
            ".." => TokenKind::OperatorRangeExclusive,
            "..=" => TokenKind::OperatorRangeInclusive,
            "<=" | ">=" | "==" | "!=" => TokenKind::OperatorComparison,
            "+" | "-" | "*" | "/" | "%" | "**" => TokenKind::OperatorArithmetic,
            _ => TokenKind::OperatorCustom,
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

    fn bump_fixed_raw(&mut self, kind: TokenKind, start: usize, raw: &str) {
        self.cursor += raw.len();
        self.push(kind, start, self.cursor);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens
            .push(Token::new(kind, start, end, &self.source[start..end]));
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn starts_with(&self, pattern: &str) -> bool {
        self.source[self.cursor..].starts_with(pattern)
    }

    fn take_char(&mut self) -> Option<char> {
        let char = self.peek_char()?;
        self.cursor += char.len_utf8();
        Some(char)
    }
}

fn is_operator_char(char: char) -> bool {
    matches!(
        char,
        '!' | '$'
            | '%'
            | '&'
            | '*'
            | '+'
            | '-'
            | '.'
            | '/'
            | ':'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '^'
            | '|'
            | '~'
    )
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

    #[test]
    fn lexes_operator_and_comment_fixture() {
        let stream = lex(
            "main.ssrg",
            "pub fn add x: Int -> Int = x + 1\nlet result = values |> map (\\value -> value + 1)\n// ok\n",
        );
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
                (TokenKind::KeywordFn, 4, 6, "fn"),
                (TokenKind::TriviaSpace, 6, 7, " "),
                (TokenKind::IdentifierLower, 7, 10, "add"),
                (TokenKind::TriviaSpace, 10, 11, " "),
                (TokenKind::IdentifierLower, 11, 12, "x"),
                (TokenKind::PunctuationColon, 12, 13, ":"),
                (TokenKind::TriviaSpace, 13, 14, " "),
                (TokenKind::IdentifierUpper, 14, 17, "Int"),
                (TokenKind::TriviaSpace, 17, 18, " "),
                (TokenKind::OperatorArrow, 18, 20, "->"),
                (TokenKind::TriviaSpace, 20, 21, " "),
                (TokenKind::IdentifierUpper, 21, 24, "Int"),
                (TokenKind::TriviaSpace, 24, 25, " "),
                (TokenKind::OperatorEquals, 25, 26, "="),
                (TokenKind::TriviaSpace, 26, 27, " "),
                (TokenKind::IdentifierLower, 27, 28, "x"),
                (TokenKind::TriviaSpace, 28, 29, " "),
                (TokenKind::OperatorArithmetic, 29, 30, "+"),
                (TokenKind::TriviaSpace, 30, 31, " "),
                (TokenKind::LiteralInteger, 31, 32, "1"),
                (TokenKind::TriviaNewline, 32, 33, "\n"),
                (TokenKind::KeywordLet, 33, 36, "let"),
                (TokenKind::TriviaSpace, 36, 37, " "),
                (TokenKind::IdentifierLower, 37, 43, "result"),
                (TokenKind::TriviaSpace, 43, 44, " "),
                (TokenKind::OperatorEquals, 44, 45, "="),
                (TokenKind::TriviaSpace, 45, 46, " "),
                (TokenKind::IdentifierLower, 46, 52, "values"),
                (TokenKind::TriviaSpace, 52, 53, " "),
                (TokenKind::OperatorPipeline, 53, 55, "|>"),
                (TokenKind::TriviaSpace, 55, 56, " "),
                (TokenKind::IdentifierLower, 56, 59, "map"),
                (TokenKind::TriviaSpace, 59, 60, " "),
                (TokenKind::PunctuationParenLeft, 60, 61, "("),
                (TokenKind::OperatorLambda, 61, 62, "\\"),
                (TokenKind::IdentifierLower, 62, 67, "value"),
                (TokenKind::TriviaSpace, 67, 68, " "),
                (TokenKind::OperatorArrow, 68, 70, "->"),
                (TokenKind::TriviaSpace, 70, 71, " "),
                (TokenKind::IdentifierLower, 71, 76, "value"),
                (TokenKind::TriviaSpace, 76, 77, " "),
                (TokenKind::OperatorArithmetic, 77, 78, "+"),
                (TokenKind::TriviaSpace, 78, 79, " "),
                (TokenKind::LiteralInteger, 79, 80, "1"),
                (TokenKind::PunctuationParenRight, 80, 81, ")"),
                (TokenKind::TriviaNewline, 81, 82, "\n"),
                (TokenKind::TriviaComment, 82, 87, "// ok"),
                (TokenKind::TriviaNewline, 87, 88, "\n"),
                (TokenKind::Eof, 88, 88, ""),
            ]
        );
        assert_eq!(
            stream.reconstructed_text(),
            "pub fn add x: Int -> Int = x + 1\nlet result = values |> map (\\value -> value + 1)\n// ok\n"
        );
    }

    #[test]
    fn lexes_ranges_and_member_paths() {
        let stream = lex(
            "main.ssrg",
            "let inclusive = 1..=100\nlet exclusive = 1..10\nlet mapped = arrays.map values\nlet empty = maps.empty\n// ranges and members\n",
        );
        let actual = stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.start, token.end, token.raw.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                (TokenKind::KeywordLet, 0, 3, "let"),
                (TokenKind::TriviaSpace, 3, 4, " "),
                (TokenKind::IdentifierLower, 4, 13, "inclusive"),
                (TokenKind::TriviaSpace, 13, 14, " "),
                (TokenKind::OperatorEquals, 14, 15, "="),
                (TokenKind::TriviaSpace, 15, 16, " "),
                (TokenKind::LiteralInteger, 16, 17, "1"),
                (TokenKind::OperatorRangeInclusive, 17, 20, "..="),
                (TokenKind::LiteralInteger, 20, 23, "100"),
                (TokenKind::TriviaNewline, 23, 24, "\n"),
                (TokenKind::KeywordLet, 24, 27, "let"),
                (TokenKind::TriviaSpace, 27, 28, " "),
                (TokenKind::IdentifierLower, 28, 37, "exclusive"),
                (TokenKind::TriviaSpace, 37, 38, " "),
                (TokenKind::OperatorEquals, 38, 39, "="),
                (TokenKind::TriviaSpace, 39, 40, " "),
                (TokenKind::LiteralInteger, 40, 41, "1"),
                (TokenKind::OperatorRangeExclusive, 41, 43, ".."),
                (TokenKind::LiteralInteger, 43, 45, "10"),
                (TokenKind::TriviaNewline, 45, 46, "\n"),
                (TokenKind::KeywordLet, 46, 49, "let"),
                (TokenKind::TriviaSpace, 49, 50, " "),
                (TokenKind::IdentifierLower, 50, 56, "mapped"),
                (TokenKind::TriviaSpace, 56, 57, " "),
                (TokenKind::OperatorEquals, 57, 58, "="),
                (TokenKind::TriviaSpace, 58, 59, " "),
                (TokenKind::IdentifierLower, 59, 65, "arrays"),
                (TokenKind::PunctuationDot, 65, 66, "."),
                (TokenKind::IdentifierLower, 66, 69, "map"),
                (TokenKind::TriviaSpace, 69, 70, " "),
                (TokenKind::IdentifierLower, 70, 76, "values"),
                (TokenKind::TriviaNewline, 76, 77, "\n"),
                (TokenKind::KeywordLet, 77, 80, "let"),
                (TokenKind::TriviaSpace, 80, 81, " "),
                (TokenKind::IdentifierLower, 81, 86, "empty"),
                (TokenKind::TriviaSpace, 86, 87, " "),
                (TokenKind::OperatorEquals, 87, 88, "="),
                (TokenKind::TriviaSpace, 88, 89, " "),
                (TokenKind::IdentifierLower, 89, 93, "maps"),
                (TokenKind::PunctuationDot, 93, 94, "."),
                (TokenKind::IdentifierLower, 94, 99, "empty"),
                (TokenKind::TriviaNewline, 99, 100, "\n"),
                (TokenKind::TriviaComment, 100, 121, "// ranges and members"),
                (TokenKind::TriviaNewline, 121, 122, "\n"),
                (TokenKind::Eof, 122, 122, ""),
            ]
        );
        assert_eq!(
            stream.reconstructed_text(),
            "let inclusive = 1..=100\nlet exclusive = 1..10\nlet mapped = arrays.map values\nlet empty = maps.empty\n// ranges and members\n"
        );
    }
}
