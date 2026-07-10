use crate::typed::ConditionalIssue;
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_conditional_diagnostics(
    issue: Option<&ConditionalIssue>,
    span: seseragi_syntax::ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(issue) = issue else {
        return;
    };

    diagnostics.push(match issue {
        ConditionalIssue::ConditionNotBool { condition, actual } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "if.condition-not-bool".to_owned(),
            primary: byte_range(*condition),
            related: vec![RelatedDiagnostic {
                message: format!("expected Bool, received {}", type_label(actual)),
                primary: byte_range(span),
            }],
            fixes: Vec::new(),
        },
        ConditionalIssue::BranchTypeMismatch {
            then_branch,
            else_branch,
            then_type,
            else_type,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "if.branch-type-mismatch".to_owned(),
            primary: byte_range(*else_branch),
            related: vec![
                RelatedDiagnostic {
                    message: format!("then branch has type {}", type_label(then_type)),
                    primary: byte_range(*then_branch),
                },
                RelatedDiagnostic {
                    message: format!("else branch has type {}", type_label(else_type)),
                    primary: byte_range(*else_branch),
                },
            ],
            fixes: Vec::new(),
        },
    });
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
