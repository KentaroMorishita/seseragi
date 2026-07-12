use crate::{type_module, type_module_public_interface, TypedInstanceImplementation, TypedType};
use seseragi_syntax::InterfaceType;

#[test]
fn selects_non_generic_derived_show_as_typed_evidence() {
    let typed = type_module(
        "artifact/derived-show/main.ssrg",
        "type AppError deriving Show =\n  | StdinFailure StdinError\n  | UnknownHand String\n  | ConsoleFailure ConsoleError\n",
    );

    assert_eq!(typed.instances.len(), 1);
    let instance = &typed.instances[0];
    assert_eq!(instance.trait_name, "Show");
    assert_eq!(instance.head, named("AppError"));
    assert_eq!(instance.type_identity, "artifact/derived-show::AppError");
    assert!(instance.constraints.is_empty());
    assert_eq!(
        instance.implementation,
        TypedInstanceImplementation::DerivedShow {
            adt_symbol: "artifact/derived-show::AppError".to_owned(),
        }
    );

    let json = serde_json::to_value(&typed).expect("typed module serializes");
    assert_eq!(json["instances"][0]["trait"], "Show");
    assert_eq!(
        json["instances"][0]["implementation"]["kind"],
        "derived-show"
    );
}

#[test]
fn accepts_payload_with_a_local_derived_show_instance() {
    let typed = type_module(
        "artifact/nested-derived-show/main.ssrg",
        "type Outer deriving Show = | Nested Detail\ntype Detail deriving Show = | Message String\n",
    );

    assert_eq!(typed.instances.len(), 2);
    assert_eq!(
        typed.instances[0].type_identity,
        "artifact/nested-derived-show::Outer"
    );
    assert_eq!(
        typed.instances[1].type_identity,
        "artifact/nested-derived-show::Detail"
    );
}

#[test]
fn excludes_derived_show_when_a_payload_instance_is_not_supported() {
    let typed = type_module(
        "artifact/unsupported-derived-show/main.ssrg",
        "type Labels deriving Show = | Labels Array<String>\n",
    );

    assert!(typed.instances.is_empty());
}

#[test]
fn projects_selected_evidence_as_show_adt_interface_instance() {
    let interface = type_module_public_interface(
        "artifact/public-derived-show/main.ssrg",
        "pub type AppError deriving Show = | UnknownHand String\n",
    );

    assert_eq!(interface.instances.len(), 1);
    assert_eq!(
        interface.instances[0].identity.as_deref(),
        Some("Show<artifact/public-derived-show::AppError>")
    );
    assert_eq!(interface.instances[0].trait_name, "Show");
    assert_eq!(
        interface.instances[0].head,
        InterfaceType::Apply {
            constructor: "Show".to_owned(),
            arguments: vec![InterfaceType::Named {
                name: "AppError".to_owned(),
                arguments: Vec::new(),
            }],
        }
    );
    assert!(interface.instances[0].constraints.is_empty());

    let json = serde_json::to_value(&interface).expect("typed interface serializes");
    assert_eq!(
        json["instances"][0]["identity"],
        "Show<artifact/public-derived-show::AppError>"
    );
}

#[test]
fn assigns_distinct_identities_to_same_spelling_in_different_modules() {
    let source = "pub type AppError deriving Show = | Failed String\n";
    let left = type_module_public_interface("fixture/left/main.ssrg", source);
    let right = type_module_public_interface("fixture/right/main.ssrg", source);

    assert_eq!(
        left.instances[0].identity.as_deref(),
        Some("Show<fixture/left::AppError>")
    );
    assert_eq!(
        right.instances[0].identity.as_deref(),
        Some("Show<fixture/right::AppError>")
    );
    assert_ne!(left.instances[0].identity, right.instances[0].identity);
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
