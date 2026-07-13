use seseragi_syntax::{CstArtifact, CstNode, Token, TokenKind, TokenStream};

pub(super) fn format_valid_module(tokens: &TokenStream, cst: &CstArtifact) -> String {
    let lines = source_lines(&tokens.tokens);
    let token_lines = token_line_map(&lines, tokens.tokens.len());
    let mut output = Vec::new();
    let mut brace_depth = 0usize;

    for line in lines {
        let Some((first, last)) = content_range(&tokens.tokens, line.start, line.end) else {
            push_blank_line(&mut output);
            continue;
        };
        let leading_closers = leading_closing_braces(&tokens.tokens, first, last);
        let structural_depth = brace_depth.saturating_sub(leading_closers);
        let continuation = declaration_continuation(cst, &token_lines, first, &tokens.tokens);
        let indent = structural_depth + continuation;
        let content = tokens.tokens[first..last]
            .iter()
            .map(|token| token.raw.as_str())
            .collect::<String>();
        output.push(format!("{}{}", "  ".repeat(indent), content));
        brace_depth = updated_brace_depth(brace_depth, &tokens.tokens[first..last]);
    }

    while output.last().is_some_and(|line| line.is_empty()) {
        output.pop();
    }
    output.push(String::new());
    output.join("\n")
}

#[derive(Clone, Copy)]
struct SourceLine {
    start: usize,
    end: usize,
}

fn source_lines(tokens: &[Token]) -> Vec<SourceLine> {
    let eof = tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(tokens.len());
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, token) in tokens.iter().enumerate().take(eof) {
        if token.kind == TokenKind::TriviaNewline {
            lines.push(SourceLine { start, end: index });
            start = index + 1;
        }
    }
    if start < eof {
        lines.push(SourceLine { start, end: eof });
    }
    lines
}

fn token_line_map(lines: &[SourceLine], token_count: usize) -> Vec<usize> {
    let mut result = vec![0; token_count];
    for (line_number, line) in lines.iter().enumerate() {
        for token_line in result.iter_mut().take(line.end).skip(line.start) {
            *token_line = line_number;
        }
    }
    result
}

fn content_range(tokens: &[Token], start: usize, end: usize) -> Option<(usize, usize)> {
    let first = (start..end).find(|index| tokens[*index].kind != TokenKind::TriviaSpace)?;
    let last = (first..end)
        .rfind(|index| tokens[*index].kind != TokenKind::TriviaSpace)
        .map(|index| index + 1)?;
    Some((first, last))
}

fn declaration_continuation(
    cst: &CstArtifact,
    token_lines: &[usize],
    first: usize,
    tokens: &[Token],
) -> usize {
    let Some(declaration) = cst
        .root
        .children
        .iter()
        .find(|node| node.start_token <= first && first < node.end_token)
    else {
        return 0;
    };
    if token_lines.get(declaration.start_token) == token_lines.get(first) {
        return 0;
    }
    let first_token = &tokens[first];
    if first_token.kind == TokenKind::TriviaComment {
        return 0;
    }
    if matches!(
        first_token.kind,
        TokenKind::KeywordWith | TokenKind::KeywordFails
    ) {
        return 0;
    }
    if is_after_closed_declaration_body(declaration, first, tokens) {
        return 0;
    }
    1
}

fn is_after_closed_declaration_body(declaration: &CstNode, first: usize, tokens: &[Token]) -> bool {
    let Some(last_significant) = (declaration.start_token..first).rev().find(|index| {
        !matches!(
            tokens[*index].kind,
            TokenKind::TriviaComment | TokenKind::TriviaNewline | TokenKind::TriviaSpace
        )
    }) else {
        return false;
    };
    tokens[last_significant].kind == TokenKind::PunctuationBraceRight
}

fn leading_closing_braces(tokens: &[Token], start: usize, end: usize) -> usize {
    tokens[start..end]
        .iter()
        .take_while(|token| token.kind == TokenKind::PunctuationBraceRight)
        .count()
}

fn updated_brace_depth(mut depth: usize, tokens: &[Token]) -> usize {
    for token in tokens {
        match token.kind {
            TokenKind::PunctuationBraceLeft => depth += 1,
            TokenKind::PunctuationBraceRight => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn push_blank_line(output: &mut Vec<String>) {
    if output.last().is_some_and(|line| !line.is_empty()) {
        output.push(String::new());
    }
}
