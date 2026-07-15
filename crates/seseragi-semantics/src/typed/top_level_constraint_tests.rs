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
