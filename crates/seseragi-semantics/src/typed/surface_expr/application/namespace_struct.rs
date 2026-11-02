use super::super::PureExpressionContext;
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceRecordItem};
use std::borrow::Cow;

pub(super) fn normalize(
    callee: &SurfaceExpr,
    arguments: &[&SurfaceExpr],
    context: &PureExpressionContext<'_>,
) -> Option<SurfaceExpr> {
    if !std::iter::once(callee).chain(arguments.iter().copied()).any(|head| {
        matches!(head, SurfaceExpr::Member { span, .. } if context.type_target(*span).is_some())
    }) {
        return None;
    }
    // Qualified braces also apply constructors with record payloads. Group only
    // resolved type names so ordinary calls keep their application order.
    let mut parts = vec![Cow::Borrowed(callee)];
    let mut changed = false;
    for argument in arguments {
        if let Some(structure) = namespace_struct(parts.last().unwrap(), argument, context) {
            parts.pop();
            parts.push(Cow::Owned(structure));
            changed = true;
        } else {
            parts.push(Cow::Borrowed(*argument));
        }
    }
    if !changed {
        return None;
    }
    let mut parts = parts.into_iter();
    let mut expression = parts.next()?.into_owned();
    for argument in parts {
        let span = ByteSpan {
            start: expression.span().start,
            end: argument.span().end,
        };
        expression = SurfaceExpr::Application {
            function: Box::new(expression),
            argument: Box::new(argument.into_owned()),
            span,
        };
    }
    Some(expression)
}

fn namespace_struct(
    function: &SurfaceExpr,
    argument: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> Option<SurfaceExpr> {
    let SurfaceExpr::Member {
        receiver,
        field,
        span,
        type_arguments,
        ..
    } = function
    else {
        return None;
    };
    context.type_target(*span)?;
    let SurfaceExpr::Name { name, .. } = receiver.as_ref() else {
        return None;
    };
    let items = match argument {
        SurfaceExpr::Member {
            receiver,
            field,
            field_span,
            type_arguments,
            span,
        } => {
            let receiver = namespace_struct(function, receiver, context)?;
            return Some(SurfaceExpr::Member {
                receiver: Box::new(receiver),
                field: field.clone(),
                field_span: *field_span,
                type_arguments: type_arguments.clone(),
                span: ByteSpan {
                    start: function.span().start,
                    end: span.end,
                },
            });
        }
        SurfaceExpr::Index {
            receiver,
            index,
            span,
        } => {
            let receiver = namespace_struct(function, receiver, context)?;
            return Some(SurfaceExpr::Index {
                receiver: Box::new(receiver),
                index: index.clone(),
                span: ByteSpan {
                    start: function.span().start,
                    end: span.end,
                },
            });
        }
        SurfaceExpr::Record { items, .. } => items.clone(),
        SurfaceExpr::Block { items, result, .. } if items.is_empty() => {
            let SurfaceExpr::Name {
                name,
                span,
                type_arguments: None,
            } = result.as_ref()
            else {
                return None;
            };
            vec![SurfaceRecordItem::Field {
                name: name.clone(),
                name_span: *span,
                value: result.as_ref().clone(),
                span: *span,
            }]
        }
        _ => return None,
    };
    Some(SurfaceExpr::Struct {
        name: format!("{name}.{field}"),
        name_span: *span,
        type_arguments: type_arguments.clone(),
        items,
        span: ByteSpan {
            start: span.start,
            end: argument.span().end,
        },
    })
}
