use crate::lexer::lex;
use crate::token::{Token, TokenKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceModule {
    pub schema: u32,
    pub source: String,
    pub declarations: Vec<SurfaceDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceDecl {
    Let {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        type_ref: Option<TypeRef>,
        span: ByteSpan,
    },
    EffectFn {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        return_type: Option<TypeRef>,
        span: ByteSpan,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TypeRef {
    Named {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<TypeRef>,
        span: ByteSpan,
    },
}

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

    fn is_angle_left(&self, index: usize) -> bool {
        self.kind_at(index) == Some(TokenKind::OperatorComparison)
            && self.tokens.get(index).is_some_and(|token| token.raw == "<")
    }

    fn is_angle_right(&self, index: usize) -> bool {
        self.kind_at(index) == Some(TokenKind::OperatorComparison)
            && self.tokens.get(index).is_some_and(|token| token.raw == ">")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_let_surface_module() {
        let module = SurfaceModule {
            schema: 1,
            source: "main.ssrg".to_owned(),
            declarations: vec![SurfaceDecl::Let {
                visibility: Visibility::Public,
                name: "answer".to_owned(),
                name_span: ByteSpan { start: 8, end: 14 },
                type_ref: Some(TypeRef::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 16, end: 19 },
                }),
                span: ByteSpan { start: 0, end: 24 },
            }],
        };

        let json = serde_json::to_value(&module).expect("surface module serializes");

        assert_eq!(json["schema"], 1);
        assert_eq!(json["source"], "main.ssrg");
        assert_eq!(json["declarations"][0]["kind"], "let");
        assert_eq!(json["declarations"][0]["visibility"], "public");
        assert_eq!(json["declarations"][0]["typeRef"]["kind"], "named");
    }

    #[test]
    fn constructs_effect_function_surface_decl() {
        let decl = SurfaceDecl::EffectFn {
            visibility: Visibility::Private,
            name: "main".to_owned(),
            name_span: ByteSpan { start: 10, end: 14 },
            return_type: Some(TypeRef::Named {
                name: "Unit".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 18, end: 22 },
            }),
            span: ByteSpan { start: 0, end: 72 },
        };

        let json = serde_json::to_value(&decl).expect("surface decl serializes");

        assert_eq!(json["kind"], "effectFn");
        assert_eq!(json["visibility"], "private");
        assert_eq!(json["returnType"]["name"], "Unit");
    }

    #[test]
    fn parses_public_let_surface_ast() {
        let module = parse_surface_ast("main.ssrg", "pub let answer: Int = 42\n");

        assert_eq!(module.schema, 1);
        assert_eq!(module.source, "main.ssrg");
        assert_eq!(
            module.declarations,
            vec![SurfaceDecl::Let {
                visibility: Visibility::Public,
                name: "answer".to_owned(),
                name_span: ByteSpan { start: 8, end: 14 },
                type_ref: Some(TypeRef::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 16, end: 19 },
                }),
                span: ByteSpan { start: 0, end: 24 },
            }]
        );
    }

    #[test]
    fn parses_multiple_lets_with_visibility_only() {
        let module = parse_surface_ast("main.ssrg", "let first = 1\npub let second = 2\n");

        assert_eq!(module.declarations.len(), 2);
        assert_eq!(
            module.declarations[0],
            SurfaceDecl::Let {
                visibility: Visibility::Private,
                name: "first".to_owned(),
                name_span: ByteSpan { start: 4, end: 9 },
                type_ref: None,
                span: ByteSpan { start: 0, end: 13 },
            }
        );
        assert_eq!(
            module.declarations[1],
            SurfaceDecl::Let {
                visibility: Visibility::Public,
                name: "second".to_owned(),
                name_span: ByteSpan { start: 22, end: 28 },
                type_ref: None,
                span: ByteSpan { start: 14, end: 32 },
            }
        );
    }

    #[test]
    fn parses_effect_do_surface_decl() {
        let module = parse_surface_ast(
            "main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    value <- console.readLine ()\n  }\n",
        );

        assert_eq!(
            module.declarations,
            vec![SurfaceDecl::EffectFn {
                visibility: Visibility::Public,
                name: "main".to_owned(),
                name_span: ByteSpan { start: 14, end: 18 },
                return_type: Some(TypeRef::Named {
                    name: "Unit".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 22, end: 26 },
                }),
                span: ByteSpan { start: 0, end: 104 },
            }]
        );
    }

    #[test]
    fn parses_nested_type_arguments_in_surface_ast() {
        let module = parse_surface_ast("main.ssrg", "pub let values: Array<Maybe<Int>> = []\n");

        assert_eq!(
            module.declarations[0],
            SurfaceDecl::Let {
                visibility: Visibility::Public,
                name: "values".to_owned(),
                name_span: ByteSpan { start: 8, end: 14 },
                type_ref: Some(TypeRef::Named {
                    name: "Array".to_owned(),
                    arguments: vec![TypeRef::Named {
                        name: "Maybe".to_owned(),
                        arguments: vec![TypeRef::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                            span: ByteSpan { start: 28, end: 31 },
                        }],
                        span: ByteSpan { start: 22, end: 32 },
                    }],
                    span: ByteSpan { start: 16, end: 33 },
                }),
                span: ByteSpan { start: 0, end: 38 },
            }
        );
    }
}
