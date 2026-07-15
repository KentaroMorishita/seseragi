use crate::declaration::is_contextual_declaration_start;
use crate::lexer::lex;
pub use crate::surface_model::{
    ByteSpan, SurfaceComprehensionClause, SurfaceConstraint, SurfaceDecl, SurfaceDoItem,
    SurfaceExpr, SurfaceImport, SurfaceImportItem, SurfaceMatchArm, SurfaceMethod, SurfaceModule,
    SurfaceParameter, SurfacePattern, SurfaceRequirement, SurfaceVariant, TypeParameter, TypeRef,
    Visibility,
};
use crate::token::{Token, TokenKind};

mod constraints;
mod effects;
mod expression;
mod functions;
#[cfg(test)]
mod import_tests;
mod imports;
mod instances;
mod methods;
#[cfg(test)]
mod nominal_tests;
#[cfg(test)]
mod operator_tests;
mod operators;
mod pattern;
mod signatures;
mod traits;
mod type_refs;
mod types;

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
                Some(
                    TokenKind::KeywordPub
                    | TokenKind::KeywordLet
                    | TokenKind::KeywordFn
                    | TokenKind::KeywordEffect,
                ) if brace_depth == 0 && self.is_declaration_boundary(index) => {
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
        if self.kind_at(previous) == Some(TokenKind::PunctuationSemicolon) {
            return true;
        }
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
        let (opaque, decl_start) = if self.raw_at(decl_start) == Some("opaque") {
            (true, self.next_significant_token(decl_start + 1, end)?)
        } else {
            (false, decl_start)
        };

        match self.kind_at(decl_start) {
            Some(TokenKind::KeywordLet) => self.parse_let_decl(visibility, start, decl_start, end),
            Some(TokenKind::KeywordFn) => self.parse_fn_decl(visibility, start, decl_start, end),
            Some(TokenKind::KeywordEffect) => {
                self.parse_effect_fn_decl(visibility, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("newtype") =>
            {
                self.parse_newtype_decl(visibility, opaque, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("alias") =>
            {
                self.parse_alias_decl(visibility, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("type") =>
            {
                self.parse_type_decl(visibility, opaque, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("struct") =>
            {
                self.parse_struct_decl(visibility, opaque, start, decl_start, end)
            }
            Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                if self.raw_at(decl_start) == Some("trait") =>
            {
                self.parse_trait_decl(visibility, start, decl_start, end)
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
        let equals = self.find_significant_token(name_index + 1, end, |kind| {
            kind == TokenKind::OperatorEquals
        });
        let body = equals.and_then(|equals| self.parse_expression(equals + 1, end));

        Some(SurfaceDecl::Let {
            visibility,
            name,
            name_span: self.byte_span(name_index)?,
            type_ref,
            body,
            span: self.declaration_span(top_start, end)?,
        })
    }

    fn parse_optional_type_parameters(
        &self,
        start: usize,
        end: usize,
    ) -> (Vec<TypeParameter>, usize) {
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
            let mut after_parameter = next + 1;
            let arity = self
                .next_significant_token(after_parameter, end)
                .filter(|index| self.is_angle_left(*index))
                .and_then(|open| {
                    let mut cursor = open + 1;
                    let mut arity = 0_u32;
                    loop {
                        let token = self.next_significant_token(cursor, end)?;
                        if self.is_angle_right(token) {
                            after_parameter = token + 1;
                            return (arity > 0).then_some(arity);
                        }
                        if self.raw_at(token) != Some("_") {
                            return None;
                        }
                        arity += 1;
                        let separator = self.next_significant_token(token + 1, end)?;
                        if self.is_angle_right(separator) {
                            after_parameter = separator + 1;
                            return Some(arity);
                        }
                        if self.kind_at(separator) != Some(TokenKind::PunctuationComma) {
                            return None;
                        }
                        cursor = separator + 1;
                    }
                })
                .unwrap_or(0);
            parameters.push(if arity == 0 {
                TypeParameter::value(name)
            } else {
                TypeParameter::constructor(name, arity)
            });
            let Some(separator) = self.next_significant_token(after_parameter, end) else {
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
        self.raw_at(index)
            .is_some_and(is_contextual_declaration_start)
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

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod type_ref_tests;
