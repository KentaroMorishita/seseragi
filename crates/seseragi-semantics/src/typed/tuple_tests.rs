use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedType};

#[test]
fn types_tuple_elements_and_preserves_their_order() {
    let typed = type_module(
        "artifact/tuple-value/main.ssrg",
        "pub fn pair left: Int -> right: Bool -> (Int, Bool) = (left, right)\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected function declaration");
    };
    let TypedExpr::Tuple {
        elements, type_ref, ..
    } = body
    else {
        panic!("expected tuple expression");
    };
    assert!(matches!(
        elements.as_slice(),
        [
            TypedExpr::Variable { name: left, .. },
            TypedExpr::Variable { name: right, .. }
        ] if left == "left" && right == "right"
    ));
    assert_eq!(
        type_ref,
        &TypedType::Tuple {
            elements: vec![named_type("Int"), named_type("Bool")],
        }
    );
}

#[test]
fn reports_an_unresolved_name_inside_a_tuple() {
    let diagnostics = crate::semantic_diagnostics(
        "artifact/tuple-unknown/main.ssrg",
        "pub fn pair value: Int -> (Int, Int) = (value, missing)\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
}

#[test]
fn reports_a_tuple_element_type_mismatch_at_the_function_boundary() {
    let diagnostics = crate::semantic_diagnostics(
        "artifact/tuple-body-mismatch/main.ssrg",
        "pub fn pair value: Int -> (Int, Bool) = (value, \"wrong\")\n",
    );

    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-T0101"));
}

#[test]
fn does_not_report_a_call_type_mismatch_for_an_unresolved_tuple_element() {
    let diagnostics = crate::semantic_diagnostics(
        "artifact/tuple-call-unknown/main.ssrg",
        "pub fn first pair: (Int, Int) -> Int = 0\npub fn use value: Int -> Int = first (value, missing)\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
