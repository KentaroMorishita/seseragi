//! A parse error written the way another language spells it is reported at
//! the token that gives it away, with the Seseragi spelling. Each rule fires
//! only when the tokens leave no other reading.
use super::{ByteRange, Diagnostic, DiagnosticEdit, DiagnosticFix};
use crate::{SurfaceDecl, Token, TokenKind};

pub(super) fn guide(
    diagnostics: &mut [Diagnostic],
    tokens: &[Token],
    declarations: &[SurfaceDecl],
) {
    let tokens = tokens
        .iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::TriviaComment | TokenKind::TriviaSpace | TokenKind::Eof
            )
        })
        .collect::<Vec<_>>();
    for diagnostic in diagnostics {
        let guided = match diagnostic.message_key.as_str() {
            "parser.expected-pattern" => match_arm_pattern(&tokens, diagnostic.primary),
            "parser.invalid-type-declaration" => variant_payloads(&tokens, diagnostic.primary),
            "parser.expected-expression" => {
                block_sequence(&tokens, diagnostic.primary, declarations)
            }
            _ => None,
        };
        if let Some(guided) = guided {
            diagnostic.message_key = guided.message_key.to_owned();
            diagnostic.primary = guided.primary;
            diagnostic.fixes = guided.fix.into_iter().collect();
        }
    }
}

struct Guided {
    message_key: &'static str,
    primary: ByteRange,
    fix: Option<DiagnosticFix>,
}

fn range(token: &Token) -> ByteRange {
    ByteRange {
        start: token.start,
        end: token.end,
    }
}

fn opens(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::PunctuationParenLeft
            | TokenKind::PunctuationBraceLeft
            | TokenKind::PunctuationSquareLeft
            | TokenKind::PunctuationListLeft
    )
}

fn closes(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::PunctuationParenRight
            | TokenKind::PunctuationBraceRight
            | TokenKind::PunctuationSquareRight
    )
}

/// The significant tokens inside `primary` with their bracket depth.
fn within<'a>(tokens: &'a [&'a Token], primary: ByteRange) -> Vec<(usize, &'a Token)> {
    let mut depth = 0usize;
    tokens
        .iter()
        .filter(|token| primary.start <= token.start && token.end <= primary.end)
        .filter(|token| token.kind != TokenKind::TriviaNewline)
        .map(|token| {
            if closes(token) {
                depth = depth.saturating_sub(1);
            }
            let at = depth;
            if opens(token) {
                depth += 1;
            }
            (at, *token)
        })
        .collect()
}

/// `x if x < 0 -> ...` guards with `when`; `1 | 2 -> ...` is two arms.
fn match_arm_pattern(tokens: &[&Token], primary: ByteRange) -> Option<Guided> {
    let pattern = within(tokens, primary);
    if let Some((_, keyword)) = pattern
        .iter()
        .skip(1)
        .find(|(depth, token)| *depth == 0 && token.kind == TokenKind::KeywordIf)
    {
        return Some(Guided {
            message_key: "parser.match-guard-if",
            primary: range(keyword),
            fix: Some(DiagnosticFix {
                title: "Replace `if` with `when`".to_owned(),
                edits: vec![DiagnosticEdit {
                    range: range(keyword),
                    replacement: "when".to_owned(),
                }],
            }),
        });
    }
    pattern
        .iter()
        .skip(1)
        .find(|(depth, token)| *depth == 0 && token.raw == "|")
        .map(|(_, bar)| Guided {
            message_key: "parser.match-or-pattern",
            primary: range(bar),
            fix: None,
        })
}

/// `| Rect Float Float` gives a variant two payload types; it takes one.
fn variant_payloads(tokens: &[&Token], primary: ByteRange) -> Option<Guided> {
    let declaration = within(tokens, primary);
    let mut index = 0;
    while index < declaration.len() {
        let (depth, token) = declaration[index];
        index += 1;
        if depth != 0 || token.raw != "|" {
            continue;
        }
        let Some((_, constructor)) = declaration.get(index) else {
            break;
        };
        if constructor.kind != TokenKind::IdentifierUpper {
            continue;
        }
        index += 1;
        let mut payloads = Vec::new();
        while let Some(end) = type_atom(&declaration, index) {
            payloads.push((declaration[index].1, declaration[end - 1].1));
            index = end;
        }
        if payloads.len() >= 2 {
            let (second, _) = payloads[1];
            let (first, _) = payloads[0];
            let (_, last) = payloads[payloads.len() - 1];
            let written = payloads
                .iter()
                .map(|(start, end)| source_between(tokens, start.start, end.end))
                .collect::<Vec<_>>();
            return Some(Guided {
                message_key: "parser.variant-payloads",
                primary: ByteRange {
                    start: second.start,
                    end: last.end,
                },
                fix: Some(DiagnosticFix {
                    title: "Wrap the payloads in a tuple".to_owned(),
                    edits: vec![DiagnosticEdit {
                        range: ByteRange {
                            start: first.start,
                            end: last.end,
                        },
                        replacement: format!("({})", written.join(", ")),
                    }],
                }),
            });
        }
    }
    None
}

/// One payload type from `start`: a qualified upper name with optional
/// `<...>` arguments, or a bracketed group. Returns the index after it.
fn type_atom(tokens: &[(usize, &Token)], start: usize) -> Option<usize> {
    let (depth, first) = *tokens.get(start)?;
    if depth != 0 {
        return None;
    }
    let mut index = start + 1;
    if opens(first) {
        while let Some((inner, token)) = tokens.get(index) {
            index += 1;
            if *inner == depth && closes(token) {
                return Some(index);
            }
        }
        return None;
    }
    if first.kind != TokenKind::IdentifierUpper {
        return None;
    }
    while tokens
        .get(index)
        .is_some_and(|(_, token)| token.kind == TokenKind::PunctuationDot)
        && tokens
            .get(index + 1)
            .is_some_and(|(_, token)| token.kind == TokenKind::IdentifierUpper)
    {
        index += 2;
    }
    if tokens.get(index).is_some_and(|(_, token)| token.raw == "<") {
        let mut angles = 0usize;
        while let Some((_, token)) = tokens.get(index) {
            index += 1;
            match token.raw.as_str() {
                "<" => angles += 1,
                ">" => angles = angles.saturating_sub(1),
                ">>" => angles = angles.saturating_sub(2),
                _ => {}
            }
            if angles == 0 {
                break;
            }
        }
    }
    Some(index)
}

fn source_between(tokens: &[&Token], start: usize, end: usize) -> String {
    let mut text = String::new();
    let mut previous_end = None;
    for token in tokens
        .iter()
        .filter(|token| start <= token.start && token.end <= end)
    {
        if previous_end.is_some_and(|previous| previous < token.start) {
            text.push(' ');
        }
        text.push_str(&token.raw);
        previous_end = Some(token.end);
    }
    text
}

/// An `effect fn` body `= { a \n b }` sequences statements, which `do`
/// does; a `{ }` block ends with one expression.
fn block_sequence(
    tokens: &[&Token],
    primary: ByteRange,
    declarations: &[SurfaceDecl],
) -> Option<Guided> {
    let in_effect_fn = declarations.iter().any(|declaration| {
        let span = declaration.span();
        matches!(declaration, SurfaceDecl::EffectFn { .. })
            && span.start <= primary.start
            && primary.end <= span.end
    });
    if !in_effect_fn {
        return None;
    }
    let at = tokens
        .iter()
        .position(|token| token.start == primary.start)?;
    let mut depth = 0usize;
    let open = (0..at).rev().find(|index| {
        let token = tokens[*index];
        if closes(token) {
            depth += 1;
        } else if opens(token) {
            if depth == 0 {
                return true;
            }
            depth -= 1;
        }
        false
    })?;
    let brace = tokens[open];
    let before = tokens[..open]
        .iter()
        .rev()
        .find(|token| token.kind != TokenKind::TriviaNewline)?;
    if brace.kind != TokenKind::PunctuationBraceLeft || before.kind != TokenKind::OperatorEquals {
        return None;
    }
    // Count the lines inside the braces that start a statement at their top.
    let mut depth = 0usize;
    let mut lines = 0usize;
    let mut line_started = false;
    for token in &tokens[open + 1..] {
        if token.kind == TokenKind::TriviaNewline {
            line_started = false;
            continue;
        }
        if closes(token) {
            if depth == 0 {
                break;
            }
            depth -= 1;
        }
        if depth == 0 && !line_started {
            if token.kind == TokenKind::KeywordLet {
                return None;
            }
            lines += 1;
            line_started = true;
        }
        if opens(token) {
            depth += 1;
        }
    }
    (lines >= 2).then(|| Guided {
        message_key: "parser.block-sequence",
        primary,
        fix: Some(DiagnosticFix {
            title: "Write the body as `do { ... }`".to_owned(),
            edits: vec![DiagnosticEdit {
                range: ByteRange {
                    start: brace.start,
                    end: brace.start,
                },
                replacement: "do ".to_owned(),
            }],
        }),
    })
}
