use crate::typed::{
    analyze_resolved_expression, inferred_type_from_expr, typed_type_contains_hole,
    typed_type_from_type_ref, PureExpressionContext, TypedResolution,
};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceExpr, TypeRef,
};

use super::type_labels::type_label;

pub(super) fn collect_let_binding_diagnostics(
    annotation: Option<&TypeRef>,
    body: Option<&SurfaceExpr>,
    declaration: ByteSpan,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (Some(annotation), Some(body)) = (annotation, body) else {
        return;
    };
    let context = PureExpressionContext::new(&[], resolution);
    let analysis = analyze_resolved_expression(body, &context);
    if analysis.conditional_issue.is_some() || analysis.pure_call_issue.is_some() {
        return;
    }

    let expected = typed_type_from_type_ref(annotation);
    let actual = inferred_type_from_expr(&analysis.value);
    if typed_type_contains_hole(&expected)
        || typed_type_contains_hole(&actual)
        || expected == actual
    {
        return;
    }

    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "binding.annotation-type-mismatch".to_owned(),
        primary: byte_range(body.span()),
        related: vec![RelatedDiagnostic {
            message: format!(
                "declared {}, initializer produces {}",
                type_label(&expected),
                type_label(&actual)
            ),
            primary: byte_range(declaration),
        }],
        fixes: Vec::new(),
    });
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_adt_constructor_annotation() {
        let artifact = crate::semantic_diagnostics(
            "artifact/let-matching-adt/main.ssrg",
            "type Hand = | Rock\nlet hand: Hand = Rock\n",
        );

        assert!(artifact.diagnostics.is_empty());
    }

    #[test]
    fn reports_mismatched_adt_constructor_annotation() {
        let artifact = crate::semantic_diagnostics(
            "artifact/let-mismatched-adt/main.ssrg",
            "type Hand = | Rock\ntype Optional = | Missing\nlet hand: Hand = Missing\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "binding.annotation-type-mismatch"
        );
        assert_eq!(
            artifact.diagnostics[0].primary,
            ByteRange { start: 62, end: 69 }
        );
        assert_eq!(artifact.diagnostics[0].related.len(), 1);
        assert_eq!(
            artifact.diagnostics[0].related[0].message,
            "declared Hand, initializer produces Optional"
        );
        assert_eq!(
            artifact.diagnostics[0].related[0].primary,
            ByteRange { start: 45, end: 69 }
        );
    }
}
