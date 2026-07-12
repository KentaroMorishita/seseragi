use super::type_ref::{
    inferred_type_from_expr, typed_type_contains_hole, typed_type_from_type_ref,
};
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
    semantically_compatible: bool,
) -> Option<FunctionBodyIssue> {
    let body = body?;
    let expected = typed_type_from_type_ref(return_type);
    let actual = inferred_type_from_expr(body);
    if typed_type_contains_hole(&expected)
        || typed_type_contains_hole(&actual)
        || expected == actual
        || semantically_compatible
    {
        return None;
    }

    Some(FunctionBodyIssue {
        body: body_span?,
        expected,
        actual,
    })
}
