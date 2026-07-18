use std::collections::BTreeMap;

use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceDecl,
    SurfaceImplMember, TypeRef,
};

use crate::{ResolvedModule, SymbolId};

use super::TypedResolution;

pub(super) fn collect_impl_diagnostics(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut methods = BTreeMap::<(SymbolId, String), ByteSpan>::new();

    for declaration in &resolved.declarations {
        let SurfaceDecl::Impl {
            target,
            members,
            span,
            ..
        } = declaration
        else {
            continue;
        };
        let Some(owner) = resolution.local_nominal_owner(target) else {
            diagnostics.push(Diagnostic {
                id: String::new(),
                code: "SES-T0503".to_owned(),
                severity: DiagnosticSeverity::Error,
                message_key: "impl.target-not-local-nominal".to_owned(),
                primary: byte_range(type_ref_span(target)),
                related: vec![RelatedDiagnostic {
                    message: "inherent impl target must be a nominal type defined in this module"
                        .to_owned(),
                    primary: byte_range(*span),
                }],
                fixes: Vec::new(),
            });
            continue;
        };

        for member in members {
            let SurfaceImplMember::Method { method, .. } = member else {
                continue;
            };
            let Some(self_parameter) = method.parameters.first() else {
                diagnostics.push(invalid_self_diagnostic(
                    method.name_span,
                    method.span,
                    "inherent method must declare self as its first parameter",
                ));
                continue;
            };
            if self_parameter.name != "self" {
                diagnostics.push(invalid_self_diagnostic(
                    self_parameter.name_span,
                    method.span,
                    "inherent method first parameter must be named self",
                ));
                continue;
            }
            if !resolution.same_semantic_type(&self_parameter.type_ref, target) {
                diagnostics.push(invalid_self_diagnostic(
                    type_ref_span(&self_parameter.type_ref),
                    method.span,
                    "inherent method self type must match the impl target",
                ));
                continue;
            }

            let key = (owner, method.name.clone());
            if let Some(first) = methods.get(&key) {
                diagnostics.push(Diagnostic {
                    id: String::new(),
                    code: "SES-N0002".to_owned(),
                    severity: DiagnosticSeverity::Error,
                    message_key: "name.duplicate-definition".to_owned(),
                    primary: byte_range(method.name_span),
                    related: vec![RelatedDiagnostic {
                        message: format!("inherent method {} was first defined here", method.name),
                        primary: byte_range(*first),
                    }],
                    fixes: Vec::new(),
                });
            } else {
                methods.insert(key, method.name_span);
            }
        }
    }
}

fn invalid_self_diagnostic(primary: ByteSpan, declaration: ByteSpan, message: &str) -> Diagnostic {
    Diagnostic {
        id: String::new(),
        code: "SES-T0503".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "impl.invalid-self-parameter".to_owned(),
        primary: byte_range(primary),
        related: vec![RelatedDiagnostic {
            message: message.to_owned(),
            primary: byte_range(declaration),
        }],
        fixes: Vec::new(),
    }
}

fn type_ref_span(type_ref: &TypeRef) -> ByteSpan {
    match type_ref {
        TypeRef::Named { span, .. }
        | TypeRef::Hole { span }
        | TypeRef::Record { span, .. }
        | TypeRef::Tuple { span, .. }
        | TypeRef::Function { span, .. } => *span,
    }
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::super::semantic_diagnostics;

    #[test]
    fn rejects_an_impl_for_a_non_local_nominal_type() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-impl-target/main.ssrg",
            "impl Int { fn value self: Int -> Int = self }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0503");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "impl.target-not-local-nominal"
        );
    }

    #[test]
    fn rejects_an_inherent_method_without_a_self_parameter() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-impl-self/main.ssrg",
            "struct Box { value: Int }\nimpl Box { fn value unit: Unit -> Int = 1 }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0503");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "impl.invalid-self-parameter"
        );
    }

    #[test]
    fn rejects_duplicate_inherent_methods_across_impl_blocks() {
        let diagnostics = semantic_diagnostics(
            "artifact/duplicate-inherent-method/main.ssrg",
            "struct Box { value: Int }\nimpl Box { fn get self: Box -> Int = self.value }\nimpl Box { fn get self: Box -> Int = self.value }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-N0002");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "name.duplicate-definition"
        );
    }
}
