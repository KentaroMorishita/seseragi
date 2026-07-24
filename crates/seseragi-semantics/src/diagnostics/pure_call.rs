use crate::typed::{PureCallIssue, PureFunctionAnalysis};
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_pure_function_diagnostics(
    analysis: &PureFunctionAnalysis,
    span: seseragi_syntax::ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(issue) = &analysis.pure_call_issue {
        diagnostics.push(call_diagnostic(issue.clone(), span));
    }
}

pub(super) fn call_diagnostic(
    issue: PureCallIssue,
    function_span: seseragi_syntax::ByteSpan,
) -> Diagnostic {
    let (code, message_key, primary, related_message) = match issue {
        PureCallIssue::InvalidExpression { expression } => (
            "SES-T0101",
            "expression.invalid",
            expression,
            "the expression could not be resolved to a typed language construct".to_owned(),
        ),
        PureCallIssue::Arity {
            callee,
            expected,
            actual,
        } => (
            "SES-T0101",
            "call.arity-mismatch",
            callee,
            format!(
                "expected {} {}, received {actual}",
                expected,
                argument_word(expected)
            ),
        ),
        PureCallIssue::ArgumentType {
            argument,
            index,
            expected,
            actual,
        } => (
            "SES-T0101",
            "call.argument-type-mismatch",
            argument,
            format!(
                "argument {} expected {}, received {}",
                index + 1,
                type_label(&expected),
                type_label(&actual)
            ),
        ),
        PureCallIssue::MissingInstance { callee, constraint } => (
            "SES-T0201",
            "instance.missing",
            callee,
            format!(
                "no {} instance matches the inferred call arguments",
                constraint.name
            ),
        ),
        PureCallIssue::TraitMethodAmbiguous { callee } => (
            "SES-T0202",
            "trait.method-ambiguous",
            callee,
            "multiple trait methods remain valid after type and instance selection".to_owned(),
        ),
        PureCallIssue::TraitMethodNoMatch { callee } => (
            "SES-T0101",
            "trait.method-no-match",
            callee,
            "no same-named trait method accepts the inferred call arguments".to_owned(),
        ),
        PureCallIssue::LambdaParameterTypeUnresolved { parameter } => (
            "SES-T0101",
            "lambda.parameter-type-unresolved",
            parameter,
            "add a parameter type annotation or use the lambda where a function type is expected"
                .to_owned(),
        ),
        PureCallIssue::LambdaParameterTypeMismatch {
            parameter,
            expected,
            actual,
        } => (
            "SES-T0101",
            "lambda.parameter-type-mismatch",
            parameter,
            format!(
                "lambda context expects {}, annotation declares {}",
                type_label(&expected),
                type_label(&actual)
            ),
        ),
        PureCallIssue::LambdaBodyTypeMismatch {
            body,
            expected,
            actual,
        } => (
            "SES-T0101",
            "lambda.body-type-mismatch",
            body,
            format!(
                "lambda context expects {}, body produces {}",
                type_label(&expected),
                type_label(&actual)
            ),
        ),
        PureCallIssue::LocalBindingTypeMismatch {
            binding,
            expected,
            actual,
        } => (
            "SES-T0101",
            "let.type-mismatch",
            binding,
            format!(
                "local binding declares {}, value produces {}",
                type_label(&expected),
                type_label(&actual)
            ),
        ),
        PureCallIssue::LocalFunctionBodyTypeMismatch {
            body,
            expected,
            actual,
        } => (
            "SES-T0101",
            "function.return-type-mismatch",
            body,
            format!(
                "local function declares {}, body produces {}",
                type_label(&expected),
                type_label(&actual)
            ),
        ),
        PureCallIssue::EffectfulForBodyNotEffect { body, actual } => (
            "SES-T0101",
            "for.body-not-effect",
            body,
            format!(
                "effectful for expects an Effect body, received {}",
                type_label(&actual)
            ),
        ),
        PureCallIssue::EffectfulForBodyNotUnit { body, actual } => (
            "SES-T0101",
            "for.body-not-unit",
            body,
            format!(
                "effectful for expects a Unit-producing body, received {}",
                type_label(&actual)
            ),
        ),
        PureCallIssue::EffectfulForRefutablePattern { pattern } => (
            "SES-T0101",
            "for.refutable-pattern",
            pattern,
            "effectful for requires an irrefutable binding pattern".to_owned(),
        ),
    };
    Diagnostic {
        id: String::new(),
        code: code.to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: primary.start,
            end: primary.end,
        },
        related: vec![RelatedDiagnostic {
            message: related_message,
            primary: ByteRange {
                start: function_span.start,
                end: function_span.end,
            },
        }],
        fixes: Vec::new(),
    }
}

fn argument_word(count: usize) -> &'static str {
    if count == 1 {
        "argument"
    } else {
        "arguments"
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic_diagnostics;

    #[test]
    fn reports_a_lambda_parameter_without_annotation_or_function_context() {
        let artifact = semantic_diagnostics(
            "lambda-unresolved.ssrg",
            "pub let identity = \\value -> value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "lambda.parameter-type-unresolved"
        );
    }

    #[test]
    fn reports_an_invalid_expression_instead_of_emitting_a_recovery_hole() {
        let artifact = semantic_diagnostics(
            "invalid-expression.ssrg",
            "pub fn broken -> Int = (\\value: Int -> value) 1\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(artifact.diagnostics[0].message_key, "expression.invalid");
    }

    #[test]
    fn reports_a_local_binding_type_mismatch() {
        let artifact = semantic_diagnostics(
            "local-binding-mismatch.ssrg",
            "fn broken -> Int = {\n  let label: String = 42\n  0\n}\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(artifact.diagnostics[0].message_key, "let.type-mismatch");
    }

    #[test]
    fn reports_a_local_function_return_type_mismatch() {
        let artifact = semantic_diagnostics(
            "local-function-mismatch.ssrg",
            "fn broken -> Int = {\n  fn label -> String = 42\n  0\n}\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "function.return-type-mismatch"
        );
    }

    #[test]
    fn reports_a_non_function_higher_order_argument() {
        let artifact = semantic_diagnostics(
            "higher-order-mismatch.ssrg",
            concat!(
                "fn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
                "fn broken value: Int -> Int = apply value value\n",
            ),
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
    }

    #[test]
    fn reports_missing_reducible_evidence_instead_of_an_unresolved_name() {
        let artifact = semantic_diagnostics(
            "missing-reducible.ssrg",
            "fn broken value: Int -> Int = reduce 0 (+) value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn reports_a_missing_local_trait_instance_at_the_call_site() {
        let artifact = semantic_diagnostics(
            "missing-render.ssrg",
            "type Badge = | Active\n\
             trait Render<A> { fn render value: A -> String }\n\
             fn label value: Badge -> String = render value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn reports_mixed_equality_operands_as_an_argument_type_mismatch() {
        let artifact = semantic_diagnostics(
            "mixed-equality.ssrg",
            "fn broken left: Int -> right: String -> Bool = left == right\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
        assert_eq!(
            artifact.diagnostics[0].related[0].message,
            "argument 2 expected Int, received String"
        );
    }

    #[test]
    fn reports_missing_show_evidence_at_a_template_interpolation() {
        let artifact = semantic_diagnostics(
            "missing-template-show.ssrg",
            "type Badge = | Active\nfn label value: Badge -> String = `badge: ${value}`\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
        assert_eq!(
            artifact.diagnostics[0].related[0].message,
            "no Show instance matches the inferred call arguments"
        );
    }

    #[test]
    fn reports_missing_show_evidence_in_an_unannotated_let() {
        let artifact = semantic_diagnostics(
            "missing-let-template-show.ssrg",
            "type Badge = | Active\nlet label = `badge: ${Active}`\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn does_not_select_a_constrained_instance_without_required_evidence() {
        let artifact = semantic_diagnostics(
            "missing-instance-evidence.ssrg",
            "trait Ready<A> { fn ready value: A -> String }\n\
             trait Render<A> { fn render value: A -> String }\n\
             instance<T> Render<Maybe<T>> where Ready<T> {\n\
               fn render value: Maybe<T> -> String = \"ready\"\n\
             }\n\
             fn label value: Maybe<Int> -> String = render value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn rejects_recursive_local_evidence_instead_of_recursing_forever() {
        let artifact = semantic_diagnostics(
            "recursive-instance-evidence.ssrg",
            "type Badge = | Active\n\
             trait Ready<A> { fn ready value: A -> String }\n\
             instance<T> Ready<T> where Ready<T> {\n\
               fn ready value: T -> String = \"ready\"\n\
             }\n\
             fn label value: Badge -> String = ready value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
        assert_eq!(artifact.diagnostics[0].message_key, "instance.missing");
    }

    #[test]
    fn reports_same_named_trait_methods_when_type_and_instances_cannot_select_one() {
        let artifact = semantic_diagnostics(
            "ambiguous-trait-method.ssrg",
            "type Badge = | Active\n\
             trait Render<A> { fn present value: A -> String }\n\
             trait Describe<A> { fn present value: A -> String }\n\
             instance Render<Badge> { fn present value: Badge -> String = \"rendered\" }\n\
             instance Describe<Badge> { fn present value: Badge -> String = \"described\" }\n\
             fn label value: Badge -> String = present value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0202");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "trait.method-ambiguous"
        );
    }

    #[test]
    fn reports_when_no_same_named_trait_method_accepts_the_argument_type() {
        let artifact = semantic_diagnostics(
            "unmatched-trait-method.ssrg",
            "type Badge = | Active\n\
             trait Render<A> { fn present value: String -> String }\n\
             trait Describe<A> { fn present value: Int -> String }\n\
             fn label value: Badge -> String = present value\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(artifact.diagnostics[0].message_key, "trait.method-no-match");
    }
}
