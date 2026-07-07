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
        if self.non_eof_token_count > 0 {
            children.push(self.parse_top_decl(0, self.non_eof_token_count));
        }
        CstNode::new("module", 0, self.non_eof_token_count, children)
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
}
