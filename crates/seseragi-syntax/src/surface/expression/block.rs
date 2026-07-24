use super::{do_block, find_matching_brace, ExpressionParser};
use crate::surface::SurfaceParser;
use crate::surface_model::{ByteSpan, SurfaceBlockItem, SurfaceDecl, SurfaceExpr, Visibility};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, open: &Token) -> Option<SurfaceExpr> {
    let open_index = parser.cursor.checked_sub(1)?;
    let close = find_matching_brace(parser.tokens, open_index, parser.end)?;
    let segments = do_block::split_segments(parser.tokens, open_index + 1, close);
    let surface = SurfaceParser {
        tokens: parser.tokens,
        non_eof_token_count: parser.end,
    };
    let mut items = Vec::new();
    let mut result = None;

    for (position, (start, end)) in segments.iter().copied().enumerate() {
        let is_last = position + 1 == segments.len();
        let first = do_block::significant_indices(parser.tokens, start, end)
            .first()
            .copied()?;
        let declaration = match parser.tokens[first].kind {
            TokenKind::KeywordLet => surface.parse_let_decl(Visibility::Private, start, first, end),
            TokenKind::KeywordFn => surface.parse_fn_decl(Visibility::Private, start, first, end),
            _ => None,
        };
        if let Some(declaration) = declaration {
            items.push(block_item(declaration)?);
        } else if is_last {
            result = surface.parse_expression(start, end);
        } else {
            result = Some(SurfaceExpr::Error {
                span: ByteSpan {
                    start: parser.tokens[first].start,
                    end: parser.tokens[first].end,
                },
            });
            break;
        }
    }

    let result = result.unwrap_or_else(|| SurfaceExpr::Error {
        span: ByteSpan {
            start: parser.tokens[close].start,
            end: parser.tokens[close].start,
        },
    });
    parser.cursor = close + 1;
    Some(SurfaceExpr::Block {
        items,
        result: Box::new(result),
        span: ByteSpan {
            start: open.start,
            end: parser.tokens[close].end,
        },
    })
}

fn block_item(declaration: SurfaceDecl) -> Option<SurfaceBlockItem> {
    match declaration {
        SurfaceDecl::Let {
            name,
            name_span,
            type_ref,
            body,
            span,
            ..
        } => Some(SurfaceBlockItem::Let {
            name,
            name_span,
            type_ref,
            value: body.unwrap_or(SurfaceExpr::Error {
                span: ByteSpan {
                    start: span.end,
                    end: span.end,
                },
            }),
            span,
        }),
        SurfaceDecl::Fn {
            name,
            name_span,
            type_parameters,
            parameters,
            return_type,
            constraints,
            body,
            span,
            ..
        } => Some(SurfaceBlockItem::Function {
            name,
            name_span,
            type_parameters,
            parameters,
            return_type,
            constraints,
            value: body.unwrap_or(SurfaceExpr::Error {
                span: ByteSpan {
                    start: span.end,
                    end: span.end,
                },
            }),
            span,
        }),
        _ => None,
    }
}
