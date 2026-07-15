use crate::token::{Token, TokenKind};

/// Returns whether the next logical line starts with an operator that cannot
/// begin an expression on its own. Such a line continues the expression before
/// the newline instead of starting a new declaration, statement, or match arm.
pub(crate) fn starts_with_operator(tokens: &[Token], start: usize, end: usize) -> bool {
    tokens
        .iter()
        .take(end)
        .skip(start)
        .find(|token| !is_trivia(token.kind))
        .is_some_and(|token| is_leading_operator(token.kind))
}

/// Returns whether the current logical line ends while an operator is still
/// waiting for its right-hand side.
pub(crate) fn ends_after_operator(tokens: &[Token], start: usize, end: usize) -> bool {
    tokens
        .iter()
        .take(end)
        .skip(start)
        .rev()
        .find(|token| !is_trivia(token.kind))
        .is_some_and(|token| is_trailing_operator(token.kind))
}

fn is_leading_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::OperatorPipeline
            | TokenKind::OperatorArithmetic
            | TokenKind::OperatorComparison
            | TokenKind::OperatorApply
    )
}

fn is_trailing_operator(kind: TokenKind) -> bool {
    is_leading_operator(kind)
        || matches!(
            kind,
            TokenKind::OperatorArrow | TokenKind::OperatorBind | TokenKind::OperatorEquals
        )
}

fn is_trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::Eof
    )
}

#[cfg(test)]
mod tests {
    use super::{ends_after_operator, starts_with_operator};
    use crate::lexer::lex;

    #[test]
    fn finds_an_operator_after_whitespace_comments_and_newlines() {
        let stream = lex("main.ssrg", "\n  // continue\n  |> render");

        assert!(starts_with_operator(&stream.tokens, 0, stream.tokens.len()));
    }

    #[test]
    fn rejects_the_start_of_an_independent_expression() {
        let stream = lex("main.ssrg", "\n  render value");

        assert!(!starts_with_operator(
            &stream.tokens,
            0,
            stream.tokens.len()
        ));
    }

    #[test]
    fn finds_a_bind_waiting_for_its_right_hand_side() {
        let stream = lex("main.ssrg", "input <-  // continue\n");

        assert!(ends_after_operator(&stream.tokens, 0, stream.tokens.len()));
    }
}
