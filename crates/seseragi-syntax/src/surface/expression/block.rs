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
        if parser.tokens[first].kind == TokenKind::KeywordRec {
            let parsed_members = (|| {
                let significant = do_block::significant_indices(parser.tokens, first + 1, end);
                let group_open = *significant.first()?;
                if parser.tokens[group_open].kind != TokenKind::PunctuationBraceLeft {
                    return None;
                }
                let group_close = find_matching_brace(parser.tokens, group_open, end)?;
                if significant.last().copied() != Some(group_close) {
                    return None;
                }
                let group_span = ByteSpan {
                    start: parser.tokens[first].start,
                    end: parser.tokens[group_close].end,
                };
                let mut parsed = Vec::new();
                let members = do_block::split_segments(parser.tokens, group_open + 1, group_close);
                if members.is_empty() {
                    return None;
                }
                for (member_start, member_end) in members {
                    let first =
                        *do_block::significant_indices(parser.tokens, member_start, member_end)
                            .first()?;
                    let declaration = match parser.tokens[first].kind {
                        TokenKind::KeywordFn => surface.parse_fn_decl(
                            Visibility::Private,
                            member_start,
                            first,
                            member_end,
                        ),
                        TokenKind::KeywordEffect => surface.parse_effect_fn_decl(
                            Visibility::Private,
                            member_start,
                            first,
                            member_end,
                        ),
                        _ => return None,
                    }?;
                    let mut member = block_item(declaration)?;
                    let SurfaceBlockItem::Function {
                        rec_group,
                        return_type,
                        ..
                    } = &mut member
                    else {
                        return None;
                    };
                    if matches!(return_type, crate::TypeRef::Hole { .. }) {
                        return None;
                    }
                    *rec_group = Some(group_span);
                    parsed.push(member);
                }
                Some(parsed)
            })();
            if let Some(members) = parsed_members {
                items.extend(members);
            } else {
                result = Some(SurfaceExpr::Error {
                    span: ByteSpan {
                        start: parser.tokens[first].start,
                        end: parser.tokens[end - 1].end,
                    },
                });
                break;
            }
            continue;
        }
        let declaration = match parser.tokens[first].kind {
            TokenKind::KeywordLet => surface.parse_let_decl(Visibility::Private, start, first, end),
            TokenKind::KeywordEffect => {
                surface.parse_effect_fn_decl(Visibility::Private, start, first, end)
            }
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

    let result = result.unwrap_or_else(|| {
        let span = ByteSpan {
            start: parser.tokens[close].start,
            end: parser.tokens[close].start,
        };
        if items.is_empty() {
            SurfaceExpr::Error { span }
        } else {
            SurfaceExpr::Unit { span }
        }
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
            pattern,
            type_ref,
            body,
            span,
            ..
        } => Some(SurfaceBlockItem::Let {
            pattern,
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
            rec_group: None,
            effect: None,
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
        SurfaceDecl::EffectFn {
            name,
            name_span,
            type_parameters,
            parameters,
            inferred_contract,
            return_type,
            requirements,
            failure,
            constraints,
            body,
            span,
            ..
        } => Some(SurfaceBlockItem::Function {
            rec_group: None,
            effect: Some(crate::surface_model::SurfaceLocalEffectContract {
                inferred: inferred_contract,
                requirements,
                failure,
            }),
            name,
            name_span,
            type_parameters,
            parameters,
            return_type: return_type.unwrap_or(crate::TypeRef::Hole { span: name_span }),
            constraints,
            value: body.unwrap_or(SurfaceExpr::Error { span }),
            span,
        }),
        _ => None,
    }
}
