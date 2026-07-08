use crate::lexer::lex;
pub use crate::surface_model::{
    ByteSpan, SurfaceDecl, SurfaceImport, SurfaceImportItem, SurfaceModule, SurfaceParameter,
    TypeRef, Visibility,
};
use crate::token::{Token, TokenKind};

mod imports;

pub fn parse_surface_ast(source_name: impl Into<String>, source: &str) -> SurfaceModule {
    let stream = lex(source_name, source);
    let non_eof_token_count = stream
        .tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(stream.tokens.len());
    let parser = SurfaceParser {
        tokens: &stream.tokens,
        non_eof_token_count,
    };

    SurfaceModule {
        schema: 1,
        source: stream.source,
        imports: parser.parse_imports(),
        declarations: parser.parse_declarations(),
    }
}

struct SurfaceParser<'tokens> {
    tokens: &'tokens [Token],
    non_eof_token_count: usize,
}

impl SurfaceParser<'_> {
    fn parse_declarations(&self) -> Vec<SurfaceDecl> {
        let declaration_starts = self.top_level_declaration_starts();
        declaration_starts
            .iter()
            .enumerate()
            .filter_map(|(position, start)| {
                let end = declaration_starts
                    .get(position + 1)
                    .copied()
                    .unwrap_or(self.non_eof_token_count);
                self.parse_top_level_declaration(*start, end)
            })
            .collect()
    }

    fn top_level_declaration_starts(&self) -> Vec<usize> {
        let mut starts = Vec::new();
        let mut brace_depth = 0usize;
        for index in 0..self.non_eof_token_count {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => brace_depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                Some(TokenKind::KeywordPub | TokenKind::KeywordLet | TokenKind::KeywordEffect)
                    if brace_depth == 0 && self.is_declaration_boundary(index) =>
                {
                    starts.push(index);
                }
                Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                    if brace_depth == 0
                        && self.is_declaration_boundary(index)
                        && self.is_contextual_declaration_start(index) =>
                {
                    starts.push(index);
                }
                _ => {}
            }
        }
        starts
    }

    fn is_declaration_boundary(&self, index: usize) -> bool {
        let Some(previous) = self.previous_significant_token(index) else {
            return true;
        };
        (previous + 1..index).any(|candidate| {
            self.kind_at(candidate)
                .is_some_and(|kind| kind == TokenKind::TriviaNewline)
        })
    }

    fn parse_top_level_declaration(&self, start: usize, end: usize) -> Option<SurfaceDecl> {
        let first = self.next_significant_token(start, end)?;
        let (visibility, decl_start) = if self.kind_at(first) == Some(TokenKind::KeywordPub) {
            (
                Visibility::Public,
                self.next_significant_token(first + 1, end)?,
            )
        } else {
            (Visibility::Private, first)
        };

        match self.kind_at(decl_start) {
            Some(TokenKind::KeywordLet) => self.parse_let_decl(visibility, start, decl_start, end),
            Some(TokenKind::KeywordEffect) => {
                self.parse_effect_fn_decl(visibility, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("newtype") =>
            {
                self.parse_newtype_decl(visibility, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("operator") =>
            {
                self.parse_operator_decl(visibility, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if visibility == Visibility::Private
                    && self.raw_at(decl_start) == Some("instance") =>
            {
                self.parse_instance_decl(start, decl_start, end)
            }
            _ => None,
        }
    }

    fn parse_let_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let type_ref = self.parse_type_after_colon(name_index + 1, end);

        Some(SurfaceDecl::Let {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_ref,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_effect_fn_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let fn_index =
            self.find_significant_token(decl_start + 1, end, |kind| kind == TokenKind::KeywordFn)?;
        let name_index = self.next_significant_token(fn_index + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let return_type = self
            .find_significant_token(name_index + 1, end, |kind| kind == TokenKind::OperatorArrow)
            .and_then(|arrow| self.parse_type_name(arrow + 1, end));

        Some(SurfaceDecl::EffectFn {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            return_type,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_newtype_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let name_index = self.next_significant_token(decl_start + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let equals = self.find_significant_token(name_index + 1, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let representation = self.parse_type_name(equals + 1, end)?;

        Some(SurfaceDecl::Newtype {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            representation,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_operator_decl(
        &self,
        visibility: Visibility,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let (type_parameters, after_type_parameters) =
            self.parse_optional_type_parameters(decl_start + 1, end);
        let fixity_index = self.next_significant_token(after_type_parameters, end)?;
        let precedence_index = self.next_significant_token(fixity_index + 1, end)?;
        let precedence = self.tokens.get(precedence_index)?.raw.parse::<u32>().ok()?;
        let spelling_start = self.next_significant_token(precedence_index + 1, end)?;
        let (spelling, after_spelling) = self.operator_spelling(spelling_start)?;
        let equals = self.find_significant_token(after_spelling, end, |kind| {
            kind == TokenKind::OperatorEquals
        })?;
        let (parameters, return_type) = self.parse_operator_signature(after_spelling, equals)?;

        Some(SurfaceDecl::Operator {
            visibility,
            type_parameters,
            fixity: self.tokens.get(fixity_index)?.raw.clone(),
            precedence,
            spelling,
            parameters,
            return_type,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_instance_decl(
        &self,
        top_start: usize,
        decl_start: usize,
        end: usize,
    ) -> Option<SurfaceDecl> {
        let trait_index = self.next_significant_token(decl_start + 1, end)?;
        let trait_name = self.identifier_name_at(trait_index)?;
        let mut arguments = Vec::new();
        let next = self.next_significant_token(trait_index + 1, end);
        if next.is_some_and(|index| self.is_angle_left(index)) {
            let (parsed, _) = self.parse_type_arguments(next? + 1, end)?;
            arguments = parsed;
        }

        Some(SurfaceDecl::Instance {
            trait_name,
            arguments,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_operator_signature(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(Vec<SurfaceParameter>, TypeRef)> {
        let mut parameters = Vec::new();
        let mut cursor = start;

        loop {
            let name_index = self.next_significant_token(cursor, end)?;
            let name = self.identifier_name_at(name_index)?;
            let colon = self.find_significant_token(name_index + 1, end, |kind| {
                kind == TokenKind::PunctuationColon
            })?;
            let (type_ref, after_type) = self.parse_type_ref(colon + 1, end)?;
            let arrow = self.next_significant_token(after_type, end)?;
            if self.kind_at(arrow) != Some(TokenKind::OperatorArrow) {
                return None;
            }
            parameters.push(SurfaceParameter {
                name,
                name_span: self.byte_span(name_index)?,
                type_ref,
            });
            cursor = arrow + 1;

            let next = self.next_significant_token(cursor, end)?;
            let next_after_type = self.parse_type_ref(next, end);
            let Some((return_type, after_return_type)) = next_after_type else {
                continue;
            };
            if self
                .next_significant_token(after_return_type, end)
                .is_none()
            {
                return Some((parameters, return_type));
            }
        }
    }

    fn parse_optional_type_parameters(&self, start: usize, end: usize) -> (Vec<String>, usize) {
        let Some(open_angle) = self.next_significant_token(start, end) else {
            return (Vec::new(), start);
        };
        if !self.is_angle_left(open_angle) {
            return (Vec::new(), start);
        }

        let mut parameters = Vec::new();
        let mut cursor = open_angle + 1;
        loop {
            let Some(next) = self.next_significant_token(cursor, end) else {
                return (Vec::new(), start);
            };
            if self.is_angle_right(next) {
                return (parameters, next + 1);
            }
            let Some(name) = self.identifier_name_at(next) else {
                return (Vec::new(), start);
            };
            parameters.push(name);
            let Some(separator) = self.next_significant_token(next + 1, end) else {
                return (Vec::new(), start);
            };
            if self.is_angle_right(separator) {
                return (parameters, separator + 1);
            }
            if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                return (Vec::new(), start);
            }
            cursor = separator + 1;
        }
    }

    fn parse_type_after_colon(&self, start: usize, end: usize) -> Option<TypeRef> {
        self.find_significant_token(start, end, |kind| kind == TokenKind::PunctuationColon)
            .and_then(|colon| self.parse_type_name(colon + 1, end))
    }

    fn parse_type_name(&self, start: usize, end: usize) -> Option<TypeRef> {
        self.parse_type_ref(start, end)
            .map(|(type_ref, _)| type_ref)
    }

    fn parse_type_ref(&self, start: usize, end: usize) -> Option<(TypeRef, usize)> {
        let type_index = self.next_significant_token(start, end)?;
        if !matches!(
            self.kind_at(type_index),
            Some(TokenKind::IdentifierUpper | TokenKind::IdentifierLower)
        ) {
            return None;
        }

        let name = self.tokens.get(type_index)?.raw.clone();
        let name_span = self.byte_span(type_index)?;
        let after_name = type_index + 1;
        let next = self.next_significant_token(after_name, end);
        let (arguments, next_index, span_end) =
            if next.is_some_and(|index| self.is_angle_left(index)) {
                self.parse_type_arguments(next? + 1, end)
                    .map(|(arguments, closing_angle)| {
                        let span_end = self
                            .tokens
                            .get(closing_angle)
                            .map(|token| token.end)
                            .unwrap_or(name_span.end);
                        (arguments, closing_angle + 1, span_end)
                    })
                    .unwrap_or_else(|| (Vec::new(), after_name, name_span.end))
            } else {
                (Vec::new(), after_name, name_span.end)
            };

        Some((
            TypeRef::Named {
                name,
                arguments,
                span: ByteSpan {
                    start: name_span.start,
                    end: span_end,
                },
            },
            next_index,
        ))
    }

    fn parse_type_arguments(&self, start: usize, end: usize) -> Option<(Vec<TypeRef>, usize)> {
        let mut arguments = Vec::new();
        let mut cursor = start;

        loop {
            let next = self.next_significant_token(cursor, end)?;
            if self.is_angle_right(next) {
                return Some((arguments, next));
            }

            let (argument, after_argument) = self.parse_type_ref(next, end)?;
            arguments.push(argument);

            let separator = self.next_significant_token(after_argument, end)?;
            if self.is_angle_right(separator) {
                return Some((arguments, separator));
            }
            if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                return None;
            }
            cursor = separator + 1;
        }
    }

    fn identifier_name_at(&self, index: usize) -> Option<String> {
        match self.kind_at(index) {
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper) => {
                Some(self.tokens.get(index)?.raw.clone())
            }
            _ => None,
        }
    }

    fn declaration_span(&self, start: usize, end: usize) -> Option<ByteSpan> {
        let start = self.tokens.get(start)?.start;
        let end = self
            .previous_significant_token(end)
            .and_then(|index| self.tokens.get(index))
            .map(|token| token.end)
            .unwrap_or(start);
        Some(ByteSpan { start, end })
    }

    fn byte_span(&self, index: usize) -> Option<ByteSpan> {
        let token = self.tokens.get(index)?;
        Some(ByteSpan {
            start: token.start,
            end: token.end,
        })
    }

    fn find_significant_token(
        &self,
        start: usize,
        end: usize,
        predicate: impl Fn(TokenKind) -> bool,
    ) -> Option<usize> {
        (start..end).find(|index| self.kind_at(*index).is_some_and(&predicate))
    }

    fn find_raw(&self, start: usize, end: usize, raw: &str) -> Option<usize> {
        self.find_significant_token(start, end, |_| true)
            .and_then(|first| {
                (first..end).find(|index| {
                    self.tokens
                        .get(*index)
                        .is_some_and(|token| token.raw == raw)
                })
            })
    }

    fn next_significant_token(&self, start: usize, end: usize) -> Option<usize> {
        (start..end).find(|index| {
            self.kind_at(*index).is_some_and(|kind| {
                !matches!(
                    kind,
                    TokenKind::TriviaComment | TokenKind::TriviaNewline | TokenKind::TriviaSpace
                )
            })
        })
    }

    fn previous_significant_token(&self, before: usize) -> Option<usize> {
        (0..before).rev().find(|index| {
            self.kind_at(*index).is_some_and(|kind| {
                !matches!(
                    kind,
                    TokenKind::TriviaComment | TokenKind::TriviaNewline | TokenKind::TriviaSpace
                )
            })
        })
    }

    fn kind_at(&self, index: usize) -> Option<TokenKind> {
        self.tokens.get(index).map(|token| token.kind)
    }

    fn raw_at(&self, index: usize) -> Option<&str> {
        self.tokens.get(index).map(|token| token.raw.as_str())
    }

    fn is_contextual_declaration_start(&self, index: usize) -> bool {
        matches!(
            self.raw_at(index),
            Some("import" | "newtype" | "operator" | "instance")
        )
    }

    fn operator_spelling(&self, start: usize) -> Option<(String, usize)> {
        let first = self.tokens.get(start)?;
        if !is_operator_spelling_token(first.kind) {
            return None;
        }
        let mut spelling = first.raw.clone();
        let mut cursor = start + 1;
        let mut previous_end = first.end;
        while let Some(token) = self.tokens.get(cursor) {
            if token.start != previous_end || !is_operator_spelling_token(token.kind) {
                break;
            }
            spelling.push_str(&token.raw);
            previous_end = token.end;
            cursor += 1;
        }
        Some((spelling, cursor))
    }

    fn is_angle_left(&self, index: usize) -> bool {
        self.kind_at(index) == Some(TokenKind::OperatorComparison)
            && self.tokens.get(index).is_some_and(|token| token.raw == "<")
    }

    fn is_angle_right(&self, index: usize) -> bool {
        self.kind_at(index) == Some(TokenKind::OperatorComparison)
            && self.tokens.get(index).is_some_and(|token| token.raw == ">")
    }
}

fn is_operator_spelling_token(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::OperatorArithmetic
            | TokenKind::OperatorComparison
            | TokenKind::OperatorCustom
            | TokenKind::OperatorPipeline
            | TokenKind::OperatorBind
            | TokenKind::OperatorApply
            | TokenKind::OperatorRangeExclusive
            | TokenKind::OperatorRangeInclusive
    )
}

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}

#[cfg(test)]
mod tests;
