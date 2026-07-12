use super::parse_unlinked_module_interface;
use crate::Visibility;

#[test]
fn keeps_private_names_without_function_bodies_or_type_schemes() {
    let unlinked = parse_unlinked_module_interface(
        "src/domain.ssrg",
        "fixture/game::domain",
        "fn hidden value: Int -> Int = value\npub fn visible value: Int -> Int = hidden value\n",
    );

    assert_eq!(unlinked.header.names.len(), 2);
    assert_eq!(unlinked.header.names[0].name, "hidden");
    assert_eq!(unlinked.header.names[0].visibility, Visibility::Private);
    assert_eq!(unlinked.header.names[1].name, "visible");
    assert_eq!(unlinked.header.names[1].visibility, Visibility::Public);
}

#[test]
fn records_hidden_and_public_nominal_constructors_by_namespace() {
    let unlinked = parse_unlinked_module_interface(
        "src/domain.ssrg",
        "fixture/game::domain",
        "pub opaque type Secret =\n  | Reveal Int\n\npub type Hand =\n  | Rock\n\nnewtype LocalId = Int\n",
    );

    let header = &unlinked.header.names;
    assert!(header.iter().any(|entry| {
        entry.name == "Secret"
            && entry.namespace == "type"
            && entry.visibility == Visibility::Public
    }));
    assert!(header.iter().any(|entry| {
        entry.name == "Reveal"
            && entry.namespace == "value"
            && entry.visibility == Visibility::Private
            && entry.constructor_of.as_deref() == Some("fixture/game::domain::Secret")
    }));
    assert!(header.iter().any(|entry| {
        entry.name == "Rock" && entry.namespace == "value" && entry.visibility == Visibility::Public
    }));
    assert_eq!(
        header
            .iter()
            .filter(|entry| entry.name == "LocalId")
            .map(|entry| (entry.namespace.as_str(), entry.visibility))
            .collect::<Vec<_>>(),
        vec![
            ("type", Visibility::Private),
            ("value", Visibility::Private),
        ]
    );
}

#[test]
fn omits_names_when_frontend_recovery_rejects_the_module() {
    let unlinked = parse_unlinked_module_interface(
        "src/domain.ssrg",
        "fixture/game::domain",
        "pub type bad =\n  | Broken\n",
    );

    assert!(unlinked.header.names.is_empty());
    assert!(unlinked.interface.exports.is_empty());
}
