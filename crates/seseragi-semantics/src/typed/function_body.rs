use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, TypeRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionBodyIssue {
    pub(crate) body: ByteSpan,
    pub(crate) expected: TypedType,
    pub(crate) actual: TypedType,
}

pub(crate) fn function_body_issue(
    body: Option<&TypedExpr>,
    body_span: Option<ByteSpan>,
    return_type: &TypeRef,
) -> Option<FunctionBodyIssue> {
    let body = body?;
    let expected = typed_type_from_type_ref(return_type);
    let actual = inferred_type_from_expr(body);
    if expected == TypedType::Hole || actual == TypedType::Hole || expected == actual {
        return None;
    }

    Some(FunctionBodyIssue {
        body: body_span?,
        expected,
        actual,
    })
}
