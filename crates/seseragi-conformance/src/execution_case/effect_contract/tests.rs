use std::collections::BTreeMap;

use seseragi_syntax::InterfaceType;

use super::{compare_required_environment, effect_environment, validate_effect_entry_contract};

fn named(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn effect_entry(environment: InterfaceType) -> InterfaceType {
    InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(InterfaceType::Named {
            name: "Effect".to_owned(),
            arguments: vec![environment, named("Never"), named("Unit")],
        }),
    }
}

#[test]
fn extracts_canonical_effect_environment_fields_order_insensitively() {
    let entry = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![
            seseragi_syntax::InterfaceRecordField {
                name: "stdin".to_owned(),
                optional: false,
                type_ref: named("Stdin"),
            },
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
        ],
    });

    assert_eq!(
        effect_environment(&entry, "main").unwrap(),
        BTreeMap::from([
            ("console".to_owned(), "Console".to_owned()),
            ("stdin".to_owned(), "Stdin".to_owned()),
        ])
    );
}

#[test]
fn follows_curried_parameters_to_the_effect_result() {
    let entry = InterfaceType::Function {
        parameter: Box::new(named("String")),
        result: Box::new(effect_entry(InterfaceType::Record {
            closed: true,
            fields: Vec::new(),
        })),
    };

    assert!(effect_environment(&entry, "parse").unwrap().is_empty());
}

#[test]
fn rejects_non_effect_results_and_open_environments() {
    let pure = InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(named("Unit")),
    };
    assert!(effect_environment(&pure, "main")
        .unwrap_err()
        .contains("Effect<R, E, A>"));

    let open = effect_entry(InterfaceType::Record {
        closed: false,
        fields: Vec::new(),
    });
    assert!(effect_environment(&open, "main")
        .unwrap_err()
        .contains("closed record"));
}

#[test]
fn rejects_optional_and_duplicate_environment_fields() {
    let optional = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![seseragi_syntax::InterfaceRecordField {
            name: "console".to_owned(),
            optional: true,
            type_ref: named("Console"),
        }],
    });
    assert!(effect_environment(&optional, "main")
        .unwrap_err()
        .contains("cannot be optional"));

    let duplicate = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
        ],
    });
    assert!(effect_environment(&duplicate, "main")
        .unwrap_err()
        .contains("duplicate field console"));
}

#[test]
fn compares_required_fields_by_exact_name_and_type() {
    let compiler = BTreeMap::from([("console".to_owned(), "Console".to_owned())]);
    assert!(compare_required_environment(&compiler, &compiler, "main").is_ok());

    let misspelled = BTreeMap::from([("console".to_owned(), "console".to_owned())]);
    let error = compare_required_environment(&compiler, &misspelled, "main").unwrap_err();
    assert!(error.contains("typed interface Effect R"));
    assert!(error.contains("Console"));
}

#[test]
fn requires_a_typed_interface_reference_for_effect_execution() {
    let error = validate_effect_entry_contract(
        std::path::Path::new("."),
        &serde_json::json!({ "entry": { "module": "example", "export": "main" } }),
        "main",
    )
    .unwrap_err();

    assert!(error.contains("entry.typedInterface is required"));
}
