use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedInstanceEvidence};

const SOURCE: &str = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"Badge is ready\" }
pub fn describe<T> value: T -> String
where Ready<T> =
  ready value
pub fn label value: Badge -> String = describe value
";

#[test]
fn consumes_a_function_constraint_inside_its_body() {
    let typed = type_module("artifact/constrained-function/main.ssrg", SOURCE);
    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected constrained function");
    };

    assert!(matches!(
        body,
        TypedExpr::Call { evidence, .. }
            if matches!(
                evidence.as_slice(),
                [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Parameter { index: 0 },
                    ..
                }]
            )
    ));
}

#[test]
fn passes_selected_local_evidence_to_a_constrained_function_call() {
    let typed = type_module("artifact/constrained-function/main.ssrg", SOURCE);
    let TypedDecl::Fn { body, .. } = &typed.declarations[2] else {
        panic!("expected caller function");
    };

    assert!(matches!(
        body,
        TypedExpr::Call { evidence, .. }
            if matches!(
                evidence.as_slice(),
                [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Local { .. },
                    ..
                }]
            )
    ));
}

#[test]
fn defers_selected_evidence_until_remaining_value_parameters_are_applied() {
    let typed = type_module(
        "artifact/partial-constrained-function/main.ssrg",
        "pub type Badge = | Active\n\
         pub trait Ready<A> { fn ready value: A -> String }\n\
         instance Ready<Badge> { fn ready value: Badge -> String = \"Badge is ready\" }\n\
         fn describe<T> value: T -> suffix: String -> String\n\
         where Ready<T> =\n\
           ready value + suffix\n\
         fn applyLabel labeler: (String -> String) -> String = labeler \"!\"\n\
         pub fn label value: Badge -> String = applyLabel (describe value)\n",
    );
    let TypedDecl::Fn { body, .. } = &typed.declarations[3] else {
        panic!("expected label function");
    };
    let TypedExpr::Call { arguments, .. } = body else {
        panic!("expected higher-order call");
    };

    assert!(matches!(
        arguments.as_slice(),
        [TypedExpr::Call {
            evidence,
            deferred_evidence_parameters,
            ..
        }] if matches!(
            evidence.as_slice(),
            [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Local { .. },
                ..
            }]
        ) && matches!(
            deferred_evidence_parameters.as_slice(),
            [crate::TypedType::Named { name, arguments }] if name == "String" && arguments.is_empty()
        )
    ));
}

#[test]
fn captures_scoped_evidence_in_a_polymorphic_partial_function_value() {
    let typed = type_module(
        "artifact/polymorphic-partial-constrained-function/main.ssrg",
        "pub trait Ready<A> { fn ready value: A -> String }\n\
         fn describe<T> value: T -> suffix: String -> String\n\
         where Ready<T> =\n\
           ready value + suffix\n\
         fn applyLabel labeler: (String -> String) -> String = labeler \"!\"\n\
         pub fn label<T> value: T -> String\n\
         where Ready<T> =\n\
           applyLabel (describe value)\n",
    );
    let TypedDecl::Fn { body, .. } = &typed.declarations[2] else {
        panic!("expected generic label function");
    };
    let TypedExpr::Call { arguments, .. } = body else {
        panic!("expected higher-order call");
    };

    assert!(matches!(
        arguments.as_slice(),
        [TypedExpr::Call {
            evidence,
            deferred_evidence_parameters,
            ..
        }] if matches!(
            evidence.as_slice(),
            [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Parameter { index: 0 },
                ..
            }]
        ) && matches!(
            deferred_evidence_parameters.as_slice(),
            [crate::TypedType::Named { name, arguments }] if name == "String" && arguments.is_empty()
        )
    ));
}
