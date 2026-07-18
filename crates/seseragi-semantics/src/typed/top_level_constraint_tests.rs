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

#[test]
fn captures_concrete_evidence_for_a_generic_custom_operator_value() {
    let typed = type_module(
        "artifact/constrained-operator-value/main.ssrg",
        "pub trait Combine<A> { fn combine left: A -> right: A -> A }\n\
         instance Combine<Int> { fn combine left: Int -> right: Int -> Int = left + right }\n\
         operator<A> infixr 4 <^> left: A -> right: A -> A\n\
         where Combine<A> =\n\
           combine left right\n\
         fn applyPair step: (Int -> Int -> Int) -> left: Int -> right: Int -> Int =\n\
           step left right\n\
         pub fn total left: Int -> right: Int -> Int = applyPair (<^>) left right\n",
    );
    let body = typed.declarations.iter().find_map(|declaration| {
        let TypedDecl::Fn { symbol, body, .. } = declaration else {
            return None;
        };
        symbol.ends_with("::total").then_some(body)
    });
    let Some(TypedExpr::Call { arguments, .. }) = body else {
        panic!("expected total function call");
    };

    assert!(matches!(
        arguments.as_slice(),
        [TypedExpr::Call {
            arguments,
            evidence,
            deferred_evidence_parameters,
            ..
        }, _, _]
            if arguments.is_empty()
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Local { .. },
                    ..
                }])
                && matches!(deferred_evidence_parameters.as_slice(), [
                    crate::TypedType::Named { name: left, arguments: left_arguments },
                    crate::TypedType::Named { name: right, arguments: right_arguments },
                ] if left == "Int" && left_arguments.is_empty()
                    && right == "Int" && right_arguments.is_empty())
    ));
}
