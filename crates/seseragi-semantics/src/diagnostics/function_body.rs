use crate::typed::{function_body_issue, FunctionBodyIssue, TopLevelPureFunction};
use seseragi_syntax::{
    ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceParameter, Token, TypeRef,
};
use std::collections::BTreeMap;

use super::type_labels::type_label;

pub(super) fn collect_function_body_diagnostics(
    tokens: &[Token],
    span: seseragi_syntax::ByteSpan,
    parameters: &[SurfaceParameter],
    return_type: &TypeRef,
    top_level_values: &BTreeMap<String, crate::TypedType>,
    top_level_functions: &BTreeMap<String, TopLevelPureFunction>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(FunctionBodyIssue {
        body,
        expected,
        actual,
    }) = function_body_issue(
        tokens,
        span,
        parameters,
        return_type,
        top_level_values,
        top_level_functions,
    )
    else {
        return;
    };

    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "function.return-type-mismatch".to_owned(),
        primary: byte_range(body),
        related: vec![RelatedDiagnostic {
            message: format!(
                "declared {}, body produces {}",
                type_label(&expected),
                type_label(&actual)
            ),
            primary: byte_range(span),
        }],
        fixes: Vec::new(),
    });
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
