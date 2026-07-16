use super::parse_expression_range;
use crate::surface_model::{ByteSpan, SurfaceExpr, SurfaceTemplatePart};
use crate::template::{decode_template_text, scan_template, TemplateChunk};
use crate::{lex, Token, TokenKind};

pub(super) fn parse(token: &Token) -> Option<SurfaceExpr> {
    let scan = scan_template(&token.raw);
    if !scan.closed {
        return Some(SurfaceExpr::Error {
            span: token_span(token),
        });
    }
    let Some(parts) = scan
        .chunks
        .into_iter()
        .map(|chunk| parse_chunk(token, chunk))
        .collect()
    else {
        return Some(SurfaceExpr::Error {
            span: token_span(token),
        });
    };
    Some(SurfaceExpr::Template {
        parts,
        span: token_span(token),
    })
}

fn parse_chunk(token: &Token, chunk: TemplateChunk) -> Option<SurfaceTemplatePart> {
    Some(match chunk {
        TemplateChunk::Text(range) => {
            let span = absolute_span(token, range.clone());
            let value = decode_template_text(&token.raw[range]).ok()?;
            SurfaceTemplatePart::Text { value, span }
        }
        TemplateChunk::Interpolation {
            expression,
            interpolation,
        } => {
            let span = absolute_span(token, interpolation);
            let value = parse_interpolation(token, expression.clone()).unwrap_or_else(|| {
                SurfaceExpr::Error {
                    span: absolute_span(token, expression),
                }
            });
            SurfaceTemplatePart::Interpolation {
                value: Box::new(value),
                span,
            }
        }
    })
}

fn parse_interpolation(token: &Token, range: std::ops::Range<usize>) -> Option<SurfaceExpr> {
    let offset = token.start + range.start;
    let stream = lex("<template>", &token.raw[range]);
    let mut tokens = stream.tokens;
    for token in &mut tokens {
        token.start += offset;
        token.end += offset;
    }
    let end = tokens
        .iter()
        .position(|token| token.kind == TokenKind::Eof)
        .unwrap_or(tokens.len());
    parse_expression_range(&tokens, 0, end)
}

fn absolute_span(token: &Token, range: std::ops::Range<usize>) -> ByteSpan {
    ByteSpan {
        start: token.start + range.start,
        end: token.start + range.end,
    }
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
