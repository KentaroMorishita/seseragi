use super::unambiguous_runtime_type;
use seseragi_semantics::ExternalTypeBinding;

#[test]
fn resolves_only_canonical_runtime_binding() {
    let bindings = vec![ExternalTypeBinding {
        spelling: "StdinError".to_owned(),
        canonical: "std/prelude::StdinError".to_owned(),
    }];

    assert_eq!(
        unambiguous_runtime_type("StdinError", &bindings)
            .map(|type_import| type_import.runtime_feature),
        Some("effect.stdin.error")
    );
}

#[test]
fn rejects_local_shadow_and_ambiguous_bindings() {
    let local = vec![ExternalTypeBinding {
        spelling: "StdinError".to_owned(),
        canonical: "artifact/domain::StdinError".to_owned(),
    }];
    assert!(unambiguous_runtime_type("StdinError", &local).is_none());

    let ambiguous = vec![
        ExternalTypeBinding {
            spelling: "StdinError".to_owned(),
            canonical: "std/prelude::StdinError".to_owned(),
        },
        ExternalTypeBinding {
            spelling: "StdinError".to_owned(),
            canonical: "artifact/domain::StdinError".to_owned(),
        },
    ];
    assert!(unambiguous_runtime_type("StdinError", &ambiguous).is_none());
}
