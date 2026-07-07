use crate::lexer::lex;
use crate::token::{Token, TokenKind, TokenStream};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
    pub kind: String,
    pub start_token: usize,
    pub end_token: usize,
    pub children: Vec<CstNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CstMissing {
    pub expected: String,
    pub at_token: usize,
    pub at_byte: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CstError {
    pub code: String,
    pub start_token: usize,
    pub end_token: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CstArtifact {
    pub schema: u32,
    pub source: String,
    pub tokens: String,
    pub root: CstNode,
    pub missing: Vec<CstMissing>,
    pub errors: Vec<CstError>,
}

pub fn parse_cst(source_name: impl Into<String>, source: &str) -> CstArtifact {
    parse_cst_from_tokens(lex(source_name, source))
}

pub fn parse_cst_from_tokens(stream: TokenStream) -> CstArtifact {
    let non_eof_token_count = stream
        .tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(stream.tokens.len());
    let mut parser = CstParser {
        tokens: &stream.tokens,
        non_eof_token_count,
        missing: Vec::new(),
        errors: Vec::new(),
    };
    let root = parser.parse_module();
    CstArtifact {
        schema: 1,
        source: stream.source,
        tokens: "tokens.json".to_owned(),
        root,
        missing: parser.missing,
        errors: parser.errors,
    }
}

struct CstParser<'tokens> {
    tokens: &'tokens [Token],
    non_eof_token_count: usize,
    missing: Vec<CstMissing>,
    errors: Vec<CstError>,
}

impl CstParser<'_> {
    fn parse_module(&mut self) -> CstNode {
        let mut children = Vec::new();
        let declaration_starts = self.top_level_declaration_starts();
        for (position, start) in declaration_starts.iter().enumerate() {
            let end = declaration_starts
                .get(position + 1)
                .copied()
                .unwrap_or(self.non_eof_token_count);
            children.push(self.parse_top_decl(*start, end));
        }
        CstNode::new("module", 0, self.non_eof_token_count, children)
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

    fn parse_top_decl(&mut self, start: usize, end: usize) -> CstNode {
        let mut children = Vec::new();
        let first = self.next_significant_token(start, end).unwrap_or(start);
        let decl_start = if self.kind_at(first) == Some(TokenKind::KeywordPub) {
            children.push(CstNode::new("decl-modifiers", first, first + 1, vec![]));
            self.next_significant_token(first + 1, end)
                .unwrap_or(first + 1)
        } else {
            first
        };

        if self.kind_at(decl_start) == Some(TokenKind::KeywordLet) {
            children.push(self.parse_let_decl(decl_start, end));
        } else if self.kind_at(decl_start) == Some(TokenKind::KeywordEffect) {
            children.push(self.parse_effect_fn_decl(decl_start, end));
        }

        CstNode::new("top-decl", start, end, children)
    }

    fn parse_let_decl(&mut self, start: usize, end: usize) -> CstNode {
        let mut children = Vec::new();
        if let Some(equals) =
            self.find_significant_token(start, end, |kind| kind == TokenKind::OperatorEquals)
        {
            let expression_start = self.next_significant_token(equals + 1, end);
            if expression_start.is_none() {
                let at_token = equals + 1;
                let at_byte = self
                    .tokens
                    .get(at_token)
                    .map(|token| token.start)
                    .unwrap_or_else(|| self.tokens.last().map(|token| token.end).unwrap_or(0));
                children.push(CstNode::new("error-expr", at_token, at_token, vec![]));
                self.missing.push(CstMissing {
                    expected: "expression".to_owned(),
                    at_token,
                    at_byte,
                });
                self.errors.push(CstError {
                    code: "SES-P0001".to_owned(),
                    start_token: at_token,
                    end_token: at_token,
                });
            }
        }
        CstNode::new("let-decl", start, end, children)
    }

    fn parse_effect_fn_decl(&mut self, start: usize, end: usize) -> CstNode {
        let mut children = Vec::new();
        let header_end = self
            .find_significant_token(start, end, |kind| kind == TokenKind::KeywordWith)
            .unwrap_or(end);
        if header_end > start {
            children.push(CstNode::new("effect-signature", start, header_end, vec![]));
        }

        let requirements_start = self.find_significant_token(start, end, |kind| {
            kind == TokenKind::KeywordWith || kind == TokenKind::KeywordFails
        });
        let do_start = self.find_significant_token(start, end, |kind| kind == TokenKind::KeywordDo);
        if let (Some(requirements_start), Some(do_start)) = (requirements_start, do_start) {
            children.push(CstNode::new(
                "effect-requirements",
                requirements_start,
                do_start,
                vec![],
            ));
        }

        if let Some(do_start) = do_start {
            let do_end = self
                .find_last_significant_token(do_start, end, |kind| {
                    kind == TokenKind::PunctuationBraceRight
                })
                .map_or(end, |index| index + 1);
            children.push(CstNode::new("do-block", do_start, do_end, vec![]));
        }

        CstNode::new("effect-fn-decl", start, end, children)
    }

    fn find_significant_token(
        &self,
        start: usize,
        end: usize,
        predicate: impl Fn(TokenKind) -> bool,
    ) -> Option<usize> {
        (start..end).find(|index| self.kind_at(*index).is_some_and(&predicate))
    }

    fn find_last_significant_token(
        &self,
        start: usize,
        end: usize,
        predicate: impl Fn(TokenKind) -> bool,
    ) -> Option<usize> {
        (start..end)
            .rev()
            .find(|index| self.kind_at(*index).is_some_and(&predicate))
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
}

impl CstNode {
    fn new(
        kind: impl Into<String>,
        start_token: usize,
        end_token: usize,
        children: Vec<CstNode>,
    ) -> Self {
        Self {
            kind: kind.into(),
            start_token,
            end_token,
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_public_let_cst() {
        let cst = parse_cst("main.ssrg", "pub let answer: Int = 42\n");

        assert_eq!(cst.schema, 1);
        assert_eq!(cst.source, "main.ssrg");
        assert_eq!(cst.root.kind, "module");
        assert_eq!(cst.root.start_token, 0);
        assert_eq!(cst.root.end_token, 13);
        assert!(cst.missing.is_empty());
        assert!(cst.errors.is_empty());
        assert_eq!(cst.root.children[0].children[0].kind, "decl-modifiers");
        assert_eq!(cst.root.children[0].children[1].kind, "let-decl");
    }

    #[test]
    fn parses_missing_let_expression_recovery_cst() {
        let cst = parse_cst("main.ssrg", "pub let answer: Int =\n");

        assert_eq!(cst.root.end_token, 11);
        assert_eq!(
            cst.root.children[0].children[1].children[0].kind,
            "error-expr"
        );
        assert_eq!(cst.missing[0].expected, "expression");
        assert_eq!(cst.missing[0].at_token, 10);
        assert_eq!(cst.missing[0].at_byte, 21);
        assert_eq!(cst.errors[0].code, "SES-P0001");
    }

    #[test]
    fn parses_effect_function_cst_skeleton() {
        let cst = parse_cst(
            "main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    value <- console.readLine ()\n    println $ value\n  }\n",
        );

        assert_eq!(cst.root.end_token, 49);
        assert!(cst.missing.is_empty());
        assert!(cst.errors.is_empty());

        let top_decl = &cst.root.children[0];
        assert_eq!(top_decl.kind, "top-decl");
        assert_eq!(top_decl.start_token, 0);
        assert_eq!(top_decl.end_token, 49);
        assert_eq!(top_decl.children[0].kind, "decl-modifiers");

        let effect_decl = &top_decl.children[1];
        assert_eq!(effect_decl.kind, "effect-fn-decl");
        assert_eq!(effect_decl.start_token, 2);
        assert_eq!(effect_decl.end_token, 49);

        assert_eq!(effect_decl.children[0].kind, "effect-signature");
        assert_eq!(effect_decl.children[0].start_token, 2);
        assert_eq!(effect_decl.children[0].end_token, 12);

        assert_eq!(effect_decl.children[1].kind, "effect-requirements");
        assert_eq!(effect_decl.children[1].start_token, 12);
        assert_eq!(effect_decl.children[1].end_token, 23);

        assert_eq!(effect_decl.children[2].kind, "do-block");
        assert_eq!(effect_decl.children[2].start_token, 23);
        assert_eq!(effect_decl.children[2].end_token, 48);
    }

    #[test]
    fn parses_multiple_top_level_declarations() {
        let cst = parse_cst("main.ssrg", "let first = 1\npub let second: Int = 2\n");

        assert_eq!(cst.root.children.len(), 2);
        assert_eq!(cst.root.children[0].kind, "top-decl");
        assert_eq!(cst.root.children[0].start_token, 0);
        assert_eq!(cst.root.children[0].end_token, 8);
        assert_eq!(cst.root.children[0].children[0].kind, "let-decl");

        assert_eq!(cst.root.children[1].kind, "top-decl");
        assert_eq!(cst.root.children[1].start_token, 8);
        assert_eq!(cst.root.children[1].end_token, 21);
        assert_eq!(cst.root.children[1].children[0].kind, "decl-modifiers");
        assert_eq!(cst.root.children[1].children[1].kind, "let-decl");
    }
}
