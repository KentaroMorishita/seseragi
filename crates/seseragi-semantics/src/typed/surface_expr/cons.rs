use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{inferred_type_from_expr, typed_type_contains_hole};
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

fn list_type(element: SemanticValueType) -> SemanticValueType {
    SemanticValueType {
        type_ref: TypedType::Named {
            name: "List".to_owned(),
            arguments: vec![element.type_ref.clone()],
        },
        key: SemanticTypeKey::NamedGeneric {
            name: "List".to_owned(),
            arguments: vec![element],
        },
    }
}

fn list_element(key: &SemanticTypeKey) -> Option<SemanticValueType> {
    match key {
        SemanticTypeKey::NamedGeneric { name, arguments }
            if name == "List" && arguments.len() == 1 =>
        {
            Some(arguments[0].clone())
        }
        _ => None,
    }
}

// Nullary constructors and empty collections may need the tail's element
// expectation. Bound generic parameters retain their semantic identity.
fn needs_element_expectation(value: &SemanticValueType) -> bool {
    if typed_type_contains_hole(&value.type_ref) {
        return true;
    }
    match &value.key {
        SemanticTypeKey::SchemeParameter(_) => true,
        SemanticTypeKey::Adt { arguments, .. }
        | SemanticTypeKey::Struct { arguments, .. }
        | SemanticTypeKey::NamedGeneric { arguments, .. }
        | SemanticTypeKey::ExternalNominal { arguments, .. } => {
            arguments.iter().any(needs_element_expectation)
        }
        SemanticTypeKey::Other => {
            matches!(&value.type_ref, TypedType::Named { name, arguments } if arguments.is_empty() && !crate::prelude::is_standalone_symbol(crate::SymbolNamespace::Type, name))
        }
        _ => false,
    }
}

pub(super) fn type_cons(
    left: &SurfaceExpr,
    right: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_cons_with(left, right, span, context, type_surface_expression)
}

pub(super) fn type_cons_with(
    left: &SurfaceExpr,
    right: &SurfaceExpr,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
    mut type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let expected = context
        .expected()
        .and_then(|value| list_element(&value.key));
    let mut head = type_argument(left, &context.with_expected(expected));
    let mut head_type = SemanticValueType {
        type_ref: inferred_type_from_expr(&head.value),
        key: head.semantic_type.clone(),
    };
    let infer_head = needs_element_expectation(&head_type);
    let tail = type_argument(
        right,
        &context.with_expected((!infer_head).then(|| list_type(head_type.clone()))),
    );
    let element = list_element(&tail.semantic_type);
    if infer_head {
        if let Some(element) = element
            .as_ref()
            .filter(|element| !needs_element_expectation(element))
        {
            head = type_argument(left, &context.with_expected(Some(element.clone())));
            head_type = SemanticValueType {
                type_ref: inferred_type_from_expr(&head.value),
                key: head.semantic_type.clone(),
            };
        }
    }
    let issue = match &element {
        None => Some(PureCallIssue::ArgumentType {
            argument: right.span(),
            index: 1,
            expected: list_type(head_type.clone()).type_ref,
            actual: inferred_type_from_expr(&tail.value),
        }),
        Some(element) if !semantic_values_are_compatible(element, &head_type) => {
            Some(PureCallIssue::ArgumentType {
                argument: left.span(),
                index: 0,
                expected: element.type_ref.clone(),
                actual: head_type.type_ref.clone(),
            })
        }
        _ => None,
    };
    let result_type = list_type(element.unwrap_or(head_type));
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Binary {
            operator: ":".to_owned(),
            left: Box::new(head.value.clone()),
            right: Box::new(tail.value.clone()),
            evidence: vec![],
            type_ref: result_type.type_ref,
            origin: span,
        },
        result_type.key,
    );
    result.merge_issues_from(head);
    result.merge_issues_from(tail);
    result.pure_call_issue = result.pure_call_issue.or(issue);
    result
}
