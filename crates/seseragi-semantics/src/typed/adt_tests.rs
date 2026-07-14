use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedType};

#[test]
fn types_nullary_adt_constructors_from_resolved_symbols() {
    let typed = type_module(
        "artifact/hand/main.ssrg",
        "pub type Hand = | Rock | Paper | Scissors\npub fn opening unit: Unit -> Hand = Rock\n",
    );

    let TypedDecl::Adt {
        symbol,
        name,
        variants,
        ..
    } = &typed.declarations[0]
    else {
        panic!("expected typed ADT");
    };
    assert_eq!(symbol, "artifact/hand::Hand");
    assert_eq!(name, "Hand");
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0].symbol, "artifact/hand::Rock");
    assert_eq!(variants[0].scheme.type_ref, named("Hand"));

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected opening function");
    };
    assert_eq!(
        body,
        &TypedExpr::Variable {
            name: "artifact/hand::Rock".to_owned(),
            evidence: Vec::new(),
            type_ref: named("Hand"),
            origin: seseragi_syntax::ByteSpan { start: 78, end: 82 },
        }
    );
}

#[test]
fn types_payload_constructor_application() {
    let typed = type_module(
        "artifact/label/main.ssrg",
        "type Label = | Missing | Present String\nfn wrap value: String -> Label = Present value\n",
    );

    let TypedDecl::Adt { variants, .. } = &typed.declarations[0] else {
        panic!("expected typed ADT");
    };
    assert_eq!(variants[1].payload, Some(named("String")));
    assert_eq!(
        variants[1].scheme.type_ref,
        TypedType::Function {
            parameter: Box::new(named("String")),
            result: Box::new(named("Label")),
        }
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected wrapper function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } if callee == "artifact/label::Present"
            && arguments.len() == 1
            && type_ref == &named("Label")
    ));
}

#[test]
fn instantiates_generic_payload_constructor_from_resolved_argument_type() {
    let typed = type_module(
        "artifact/maybe/main.ssrg",
        "type Maybe<A> = | Nothing | Just A\nfn wrap value: String -> Maybe<String> = Just value\n",
    );

    let TypedDecl::Adt { variants, .. } = &typed.declarations[0] else {
        panic!("expected typed ADT");
    };
    assert_eq!(variants[1].scheme.type_parameters, vec!["A"]);
    assert_eq!(
        variants[1].scheme.type_ref,
        TypedType::Function {
            parameter: Box::new(named("A")),
            result: Box::new(TypedType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![named("A")],
            }),
        }
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected wrapper function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            arguments,
            type_ref: TypedType::Named {
                name,
                arguments: result_arguments,
            },
            ..
        } if callee == "artifact/maybe::Just"
            && arguments.len() == 1
            && name == "Maybe"
            && result_arguments == &vec![named("String")]
    ));
}

#[test]
fn keeps_nullary_generic_constructor_polymorphic_instead_of_hole() {
    let typed = type_module(
        "artifact/maybe/main.ssrg",
        "type Maybe<A> = | Nothing | Just A\nfn missing<A> unit: Unit -> Maybe<A> = Nothing\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected missing function");
    };
    assert_eq!(
        body,
        &TypedExpr::Variable {
            name: "artifact/maybe::Nothing".to_owned(),
            evidence: Vec::new(),
            type_ref: TypedType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![named("A")],
            },
            origin: seseragi_syntax::ByteSpan { start: 74, end: 81 },
        }
    );
}

#[test]
fn does_not_recreate_unresolved_payload_types_from_source_spelling() {
    let typed = type_module("artifact/label/main.ssrg", "type Label = | Present Strng\n");

    assert!(typed.declarations.is_empty());
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
