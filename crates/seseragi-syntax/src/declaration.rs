use crate::{decode_string_literal, ByteSpan, Deprecation, Token, TokenKind};

pub(crate) fn is_contextual_declaration_start(raw: &str) -> bool {
    matches!(
        raw,
        "import"
            | "newtype"
            | "alias"
            | "type"
            | "struct"
            | "opaque"
            | "operator"
            | "impl"
            | "instance"
            | "trait"
            | "foreign"
    )
}

pub(crate) struct DeclarationPrefix {
    pub keyword: usize,
    pub public: bool,
    pub opaque: bool,
    pub deprecation: Option<Deprecation>,
}

fn significant(tokens: &[Token], start: usize, end: usize) -> Option<usize> {
    (start..end).find(|&i| {
        !matches!(
            tokens[i].kind,
            TokenKind::TriviaSpace
                | TokenKind::TriviaNewline
                | TokenKind::TriviaComment
                | TokenKind::Eof
        )
    })
}

/// One modifier grammar for lossless CST, surface declarations and validation.
/// Semicolons are significant: a prefix cannot attach across a statement end.
pub(crate) fn declaration_prefix(
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Result<DeclarationPrefix, ByteSpan> {
    let span = |i: usize| ByteSpan {
        start: tokens[i].start,
        end: tokens[i].end,
    };
    let first = significant(tokens, start, end).ok_or_else(|| span(start))?;
    let public = tokens[first].kind == TokenKind::KeywordPub;
    let mut keyword = if public {
        significant(tokens, first + 1, end).ok_or_else(|| span(first))?
    } else {
        first
    };
    let mut deprecation = None;
    if tokens[keyword].raw == "deprecated" {
        let clause = keyword;
        let message = significant(tokens, keyword + 1, end).ok_or_else(|| span(clause))?;
        if tokens[message].kind != TokenKind::LiteralString {
            return Err(span(message));
        }
        let text = decode_string_literal(&tokens[message].raw).map_err(|_| span(message))?;
        if text.is_empty() {
            return Err(span(message));
        }
        keyword = significant(tokens, message + 1, end).ok_or_else(|| span(clause))?;
        let mut since = None;
        let mut last = message;
        if tokens[keyword].raw == "since" {
            let version = significant(tokens, keyword + 1, end).ok_or_else(|| span(keyword))?;
            if tokens[version].kind != TokenKind::LiteralString {
                return Err(span(version));
            }
            let value = decode_string_literal(&tokens[version].raw).map_err(|_| span(version))?;
            let parsed = semver::Version::parse(&value).map_err(|_| span(version))?;
            if parsed.to_string() != value {
                return Err(span(version));
            }
            since = Some(value);
            last = version;
            keyword = significant(tokens, version + 1, end).ok_or_else(|| span(clause))?;
        }
        deprecation = Some(Deprecation {
            source: String::new(),
            message: text,
            since,
            origin: ByteSpan {
                start: tokens[clause].start,
                end: tokens[last].end,
            },
        });
    }
    let opaque = tokens[keyword].raw == "opaque";
    if opaque {
        keyword = significant(tokens, keyword + 1, end).ok_or_else(|| span(keyword))?;
    }
    if deprecation.is_some()
        && (!matches!(
            tokens[keyword].raw.as_str(),
            "let"
                | "fn"
                | "effect"
                | "type"
                | "newtype"
                | "alias"
                | "struct"
                | "trait"
                | "operator"
        ) || opaque && !matches!(tokens[keyword].raw.as_str(), "type" | "newtype" | "struct"))
    {
        return Err(span(keyword));
    }
    Ok(DeclarationPrefix {
        keyword,
        public,
        opaque,
        deprecation,
    })
}

pub(crate) fn top_level_declaration_starts(tokens: &[Token], end: usize) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut braces = 0usize;
    let mut prefix_end = None;
    for i in 0..end {
        match tokens[i].kind {
            TokenKind::PunctuationBraceLeft => braces += 1,
            TokenKind::PunctuationBraceRight => braces = braces.saturating_sub(1),
            _ if braces == 0 && !prefix_end.is_some_and(|end| i <= end) => {
                let candidate = matches!(
                    tokens[i].kind,
                    TokenKind::KeywordPub
                        | TokenKind::KeywordLet
                        | TokenKind::KeywordFn
                        | TokenKind::KeywordEffect
                ) || matches!(
                    tokens[i].kind,
                    TokenKind::IdentifierLower | TokenKind::IdentifierUpper
                ) && (is_contextual_declaration_start(&tokens[i].raw)
                    || tokens[i].raw == "deprecated"
                        && significant(tokens, i + 1, end)
                            .is_none_or(|next| tokens[next].kind != TokenKind::PunctuationColon));
                if candidate && declaration_boundary(tokens, i) {
                    starts.push(i);
                    prefix_end = declaration_prefix(tokens, i, end)
                        .ok()
                        .map(|prefix| prefix.keyword);
                }
            }
            _ => {}
        }
    }
    starts
}

pub(crate) fn declaration_boundary(tokens: &[Token], index: usize) -> bool {
    let Some(previous) = (0..index).rev().find(|&i| {
        !matches!(
            tokens[i].kind,
            TokenKind::TriviaSpace | TokenKind::TriviaNewline | TokenKind::TriviaComment
        )
    }) else {
        return true;
    };
    tokens[previous].kind == TokenKind::PunctuationSemicolon
        || tokens[previous + 1..index]
            .iter()
            .any(|token| token.kind == TokenKind::TriviaNewline)
}

#[cfg(test)]
mod tests;
