//! Validate declaration regions even when recovery cannot produce a surface
//! declaration. Otherwise an unused malformed header can disappear before
//! semantic analysis, leaving an apparently valid module.
use super::{expression::split_segments, SurfaceParser};
use crate::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, SurfaceDecl, SurfaceModule,
    SurfacePattern, Token, TokenKind, Visibility,
};

pub(crate) fn declaration_diagnostics(tokens: &[Token], module: &SurfaceModule) -> Vec<Diagnostic> {
    let end = tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(tokens.len());
    let parser = SurfaceParser {
        tokens,
        non_eof_token_count: end,
    };
    let starts = parser.top_level_declaration_starts();
    let mut diagnostics = Vec::new();
    parser.unexpected_statements(0, starts.first().copied().unwrap_or(end), &mut diagnostics);
    for (position, start) in starts.iter().copied().enumerate() {
        let end = starts.get(position + 1).copied().unwrap_or(end);
        parser.validate_declaration(start, end, module, &mut diagnostics);
    }
    diagnostics
}

impl SurfaceParser<'_> {
    fn validate_declaration(
        &self,
        start: usize,
        end: usize,
        module: &SurfaceModule,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(mut keyword) = self.next_significant_token(start, end) else {
            return;
        };
        while matches!(self.raw_at(keyword), Some("pub" | "opaque")) {
            let Some(next) = self.next_significant_token(keyword + 1, end) else {
                return;
            };
            keyword = next;
        }
        let declaration = module
            .declarations
            .iter()
            .find(|declaration| declaration.span().start == self.tokens[start].start);
        let callable = matches!(
            self.kind_at(keyword),
            Some(TokenKind::KeywordLet | TokenKind::KeywordFn | TokenKind::KeywordEffect)
        ) || self.raw_at(keyword) == Some("operator");
        let equals = self.find_top_level_token(keyword, end, TokenKind::OperatorEquals);
        let header_end = equals.unwrap_or(end);

        if let Some(SurfaceDecl::Fn { parameters, .. } | SurfaceDecl::EffectFn { parameters, .. }) =
            declaration
        {
            for parameter in parameters {
                let token = self
                    .tokens
                    .iter()
                    .find(|token| token.start == parameter.name_span.start);
                if token.is_some_and(|token| {
                    token.kind != TokenKind::IdentifierLower || is_reserved(&token.raw)
                }) {
                    diagnostics.push(error(
                        "parser.invalid-declaration-name",
                        parameter.name_span,
                    ));
                }
            }
        }

        if self.kind_at(keyword) == Some(TokenKind::KeywordLet) {
            if let Some(SurfaceDecl::Let {
                visibility,
                pattern,
                type_ref,
                ..
            }) = declaration
            {
                validate_pattern(pattern, diagnostics);
                if *visibility == Visibility::Public && type_ref.is_none() {
                    diagnostics.push(error(
                        "parser.public-let-annotation-required",
                        pattern.span(),
                    ));
                }
            }
        } else {
            let name_keyword = if self.kind_at(keyword) == Some(TokenKind::KeywordEffect) {
                self.next_significant_token(keyword + 1, header_end)
                    .filter(|index| self.kind_at(*index) == Some(TokenKind::KeywordFn))
            } else if matches!(
                self.raw_at(keyword),
                Some("fn" | "type" | "alias" | "newtype" | "struct" | "trait")
            ) {
                Some(keyword)
            } else {
                None
            };
            if let Some(name_keyword) = name_keyword {
                let name_index = self.next_significant_token(name_keyword + 1, header_end);
                let expected = if self.kind_at(name_keyword) == Some(TokenKind::KeywordFn) {
                    TokenKind::IdentifierLower
                } else {
                    TokenKind::IdentifierUpper
                };
                if let Some(name_index) = name_index {
                    if self.kind_at(name_index) != Some(expected)
                        || self.raw_at(name_index).is_some_and(is_reserved)
                    {
                        let mut span = self.byte_span(name_index).expect("name token");
                        // A digit-leading word is two lexical tokens. Highlight
                        // the whole invalid spelling instead of just its digit.
                        let mut next = name_index + 1;
                        while next < header_end
                            && self.tokens[next].start == span.end
                            && matches!(
                                self.tokens[next].kind,
                                TokenKind::IdentifierLower | TokenKind::IdentifierUpper
                            )
                        {
                            span.end = self.tokens[next].end;
                            next += 1;
                        }
                        diagnostics.push(error("parser.invalid-declaration-name", span));
                    } else if self.kind_at(keyword) == Some(TokenKind::KeywordFn)
                        && !matches!(declaration, Some(SurfaceDecl::Fn { .. }))
                    {
                        diagnostics.push(error(
                            "parser.function-annotations-required",
                            self.declaration_span(keyword, header_end)
                                .expect("function header"),
                        ));
                    }
                } else {
                    diagnostics.push(error(
                        "parser.invalid-declaration-name",
                        self.byte_span(name_keyword).expect("declaration keyword"),
                    ));
                }
            }
        }

        if callable {
            if let Some(equals) = equals {
                // Reuse the block-expression boundary parser: a completed RHS
                // followed by a newline or semicolon ends the declaration,
                // while incomplete expressions and operator continuations stay
                // inside it. The later segments are forbidden module statements.
                for (start, end) in split_segments(self.tokens, equals + 1, end)
                    .into_iter()
                    .skip(1)
                {
                    self.unexpected_statement(start, end, diagnostics);
                }
            }
        } else if let Some(import) = module
            .imports
            .iter()
            .find(|import| import.span.start == self.tokens[start].start)
        {
            let tail = self
                .tokens
                .partition_point(|token| token.end <= import.span.end);
            self.unexpected_statements(tail, end, diagnostics);
        } else if let Some(foreign) = module
            .foreign_modules
            .iter()
            .find(|foreign| foreign.span.start == self.tokens[start].start)
        {
            let tail = self
                .tokens
                .partition_point(|token| token.end <= foreign.span.end);
            self.unexpected_statements(tail, end, diagnostics);
        } else if matches!(
            self.raw_at(keyword),
            Some("struct" | "trait" | "impl" | "instance")
        ) {
            if let Some(open) = self.find_significant_token(keyword, end, |kind| {
                kind == TokenKind::PunctuationBraceLeft
            }) {
                if let Some(close) = self.find_matching_brace(open, end) {
                    self.unexpected_statements(close + 1, end, diagnostics);
                }
            }
        } else if matches!(self.raw_at(keyword), Some("alias" | "newtype")) {
            if let Some(equals) = equals {
                if let Some((_, after_type)) = self.parse_type_ref(equals + 1, end) {
                    self.unexpected_statements(after_type, end, diagnostics);
                }
            }
        } else if self.raw_at(keyword) == Some("type") {
            // ADT payloads can contain multiline types. Ask the declaration
            // parser whether each complete delimiter-free line forms a type,
            // and keep a following variant pipe inside the same declaration.
            let mut depth = 0usize;
            for index in keyword..end {
                match self.tokens[index].kind {
                    TokenKind::PunctuationBraceLeft
                    | TokenKind::PunctuationParenLeft
                    | TokenKind::PunctuationSquareLeft
                    | TokenKind::PunctuationListLeft => depth += 1,
                    TokenKind::PunctuationBraceRight
                    | TokenKind::PunctuationParenRight
                    | TokenKind::PunctuationSquareRight => depth = depth.saturating_sub(1),
                    TokenKind::TriviaNewline | TokenKind::PunctuationSemicolon if depth == 0 => {
                        let next = self.next_significant_token(index + 1, end);
                        if self.tokens[index].kind == TokenKind::TriviaNewline
                            && next.is_some_and(|next| self.raw_at(next) == Some("|"))
                        {
                            continue;
                        }
                        if self.parse_top_level_declaration(start, index).is_some() {
                            self.unexpected_statements(index + 1, end, diagnostics);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn unexpected_statements(&self, start: usize, end: usize, diagnostics: &mut Vec<Diagnostic>) {
        for (start, end) in split_segments(self.tokens, start, end) {
            self.unexpected_statement(start, end, diagnostics);
        }
    }

    fn unexpected_statement(&self, start: usize, end: usize, diagnostics: &mut Vec<Diagnostic>) {
        let Some(first) = self.next_significant_token(start, end) else {
            return;
        };
        if let Some(span) = self.declaration_span(first, end) {
            diagnostics.push(error("parser.module-expression-statement", span));
        }
    }
}

fn validate_pattern(pattern: &SurfacePattern, diagnostics: &mut Vec<Diagnostic>) {
    if matches!(pattern, SurfacePattern::Error { .. }) {
        diagnostics.push(error("parser.invalid-declaration-name", pattern.span()));
        return;
    }
    for binding in pattern.bindings() {
        if is_reserved(&binding.name) {
            diagnostics.push(error("parser.invalid-declaration-name", binding.name_span));
        }
    }
}

fn is_reserved(name: &str) -> bool {
    matches!(
        name,
        "as" | "alias"
            | "deprecated"
            | "deriving"
            | "do"
            | "effect"
            | "else"
            | "fails"
            | "False"
            | "fn"
            | "for"
            | "foreign"
            | "from"
            | "if"
            | "impl"
            | "import"
            | "infix"
            | "infixl"
            | "infixr"
            | "instance"
            | "let"
            | "match"
            | "newtype"
            | "opaque"
            | "operator"
            | "pub"
            | "rec"
            | "struct"
            | "then"
            | "trait"
            | "True"
            | "type"
            | "when"
            | "where"
            | "with"
    )
}

fn error(message_key: &str, span: ByteSpan) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-P0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: span.start,
            end: span.end,
        },
        related: Vec::new(),
        fixes: Vec::new(),
        type_difference: None,
    }
}
