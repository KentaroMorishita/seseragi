use crate::typed::instances::InstanceContractIssue;
use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

pub(super) fn diagnostic(issue: &InstanceContractIssue) -> Diagnostic {
    let (code, message_key, primary, related) = match issue {
        InstanceContractIssue::ArityMismatch {
            trait_name,
            expected,
            actual,
            primary,
            declaration,
        } => (
            "SES-T0101",
            "trait.instance-arity-mismatch",
            *primary,
            RelatedDiagnostic {
                message: format!(
                    "trait {trait_name} expects {expected} type argument(s), but the instance provides {actual}"
                ),
                primary: byte_range(*declaration),
            },
        ),
        InstanceContractIssue::MissingMethod {
            method,
            primary,
            contract,
        } => (
            "SES-T0101",
            "trait.instance-method-missing",
            *primary,
            RelatedDiagnostic {
                message: format!("trait contract requires method {method}"),
                primary: byte_range(*contract),
            },
        ),
        InstanceContractIssue::UnexpectedMethod {
            method,
            primary,
            contract,
        } => (
            "SES-T0101",
            "trait.instance-method-unexpected",
            *primary,
            RelatedDiagnostic {
                message: format!("trait contract does not declare method {method}"),
                primary: byte_range(*contract),
            },
        ),
        InstanceContractIssue::DuplicateMethod {
            method,
            primary,
            declaration,
        } => (
            "SES-N0002",
            "name.duplicate-definition",
            *primary,
            RelatedDiagnostic {
                message: format!("instance method {method} is already defined here"),
                primary: byte_range(*declaration),
            },
        ),
        InstanceContractIssue::SignatureMismatch {
            method,
            primary,
            contract,
        } => (
            "SES-T0101",
            "trait.instance-method-signature-mismatch",
            *primary,
            RelatedDiagnostic {
                message: format!("instance method {method} must match this trait contract"),
                primary: byte_range(*contract),
            },
        ),
        InstanceContractIssue::MissingBody {
            method,
            primary,
            contract,
        } => (
            "SES-T0101",
            "trait.instance-method-body-missing",
            *primary,
            RelatedDiagnostic {
                message: format!("instance method {method} must implement this trait contract"),
                primary: byte_range(*contract),
            },
        ),
    };
    Diagnostic {
        id: String::new(),
        code: code.to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: byte_range(primary),
        related: vec![related],
        fixes: Vec::new(),
    }
}

fn byte_range(span: seseragi_syntax::ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
