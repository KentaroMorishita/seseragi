use crate::typed::MatchIssue;
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_match_diagnostics(issues: &[MatchIssue], diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.extend(issues.iter().map(diagnostic));
}

fn diagnostic(issue: &MatchIssue) -> Diagnostic {
    match issue {
        MatchIssue::PatternMismatch { pattern, message } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "match.pattern-type-mismatch".to_owned(),
            primary: byte_range(*pattern),
            related: vec![RelatedDiagnostic {
                message: message.clone(),
                primary: byte_range(*pattern),
            }],
            fixes: Vec::new(),
        },
        MatchIssue::GuardNotBool { guard, actual } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "match.guard-not-bool".to_owned(),
            primary: byte_range(*guard),
            related: vec![RelatedDiagnostic {
                message: format!("expected Bool, received {}", type_label(actual)),
                primary: byte_range(*guard),
            }],
            fixes: Vec::new(),
        },
        MatchIssue::BranchTypeMismatch {
            expected_branch,
            actual_branch,
            expected,
            actual,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0101".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "match.branch-type-mismatch".to_owned(),
            primary: byte_range(*actual_branch),
            related: vec![
                RelatedDiagnostic {
                    message: format!("first branch has type {}", type_label(expected)),
                    primary: byte_range(*expected_branch),
                },
                RelatedDiagnostic {
                    message: format!("this branch has type {}", type_label(actual)),
                    primary: byte_range(*actual_branch),
                },
            ],
            fixes: Vec::new(),
        },
        MatchIssue::NonExhaustive {
            expression,
            missing,
        } => Diagnostic {
            id: String::new(),
            code: "SES-T0301".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "match.non-exhaustive".to_owned(),
            primary: byte_range(*expression),
            related: vec![RelatedDiagnostic {
                message: format!("missing patterns: {}", missing.join(", ")),
                primary: byte_range(*expression),
            }],
            fixes: Vec::new(),
        },
        MatchIssue::Unreachable { arm } => Diagnostic {
            id: String::new(),
            code: "SES-T0302".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "match.unreachable-arm".to_owned(),
            primary: byte_range(*arm),
            related: Vec::new(),
            fixes: Vec::new(),
        },
    }
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
