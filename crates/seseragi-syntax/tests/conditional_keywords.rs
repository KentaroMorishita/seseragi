use seseragi_syntax::{lex, TokenKind};

#[test]
fn lexes_if_then_else_as_reserved_keywords() {
    let stream = lex("main.ssrg", "if ready then yes else no");
    let kinds = stream
        .tokens
        .iter()
        .filter(|token| token.kind != TokenKind::TriviaSpace)
        .map(|token| token.kind)
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        vec![
            TokenKind::KeywordIf,
            TokenKind::IdentifierLower,
            TokenKind::KeywordThen,
            TokenKind::IdentifierLower,
            TokenKind::KeywordElse,
            TokenKind::IdentifierLower,
            TokenKind::Eof,
        ]
    );
}
